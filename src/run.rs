use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::app_entry;
use crate::ast::{App, Item, ParamPermission, Program, SectionLine, Task};
use crate::core_body::{self, BodyStatement};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Span};
use crate::element_place;
use crate::field_place;
use crate::graph::is_meaningful_line_text;
use crate::return_dependency;
use crate::typed_failure::{self, FailureCatalog, FailureVariant};
use crate::writable_field_alias::{self, AliasAnalysis, AliasBinding, AliasIssueKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    Success(String),
    AppSuccess,
    Failure(String),
    AppFailure(String),
    ContractViolation,
    Trap(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub outcome: RunOutcome,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Unit,
    Int(i64),
    Bool(bool),
    Text(String),
    Record(BTreeMap<String, Value>),
    List(Vec<Value>),
    Variant(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Evaluated {
    Value(Value),
    Failure(FailureValue),
    ContractViolation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Flow {
    Continue,
    Return {
        value: Value,
        root: Option<String>,
        span: Span,
    },
    Fail(FailureValue),
    ContractViolation,
}

#[derive(Debug, Clone)]
struct ExecLine {
    text: String,
    location: String,
    span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimePermission {
    Borrow,
    Change,
    Consume,
    Local,
}

#[derive(Debug, Clone)]
struct RuntimeBinding {
    value: Value,
    permission: RuntimePermission,
    writable: bool,
    moved_at: Option<Span>,
    moved_by: Option<String>,
    linear: bool,
    view: Option<RuntimeView>,
    writable_alias_source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeViewKind {
    Field,
    Element,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeViewInvalidationKind {
    FieldWrite,
    ListAppend,
}

#[derive(Debug, Clone)]
struct RuntimeViewInvalidation {
    span: Span,
    kind: RuntimeViewInvalidationKind,
}

#[derive(Debug, Clone)]
struct RuntimeView {
    kind: RuntimeViewKind,
    source_place: String,
    bound_at: Span,
    invalidated_by: Option<RuntimeViewInvalidation>,
}

impl RuntimeBinding {
    fn parameter(value: Value, permission: ParamPermission) -> Self {
        Self {
            value,
            permission: RuntimePermission::from(permission),
            writable: permission != ParamPermission::Borrow,
            moved_at: None,
            moved_by: None,
            linear: false,
            view: None,
            writable_alias_source: None,
        }
    }

    fn local(value: Value, linear: bool) -> Self {
        Self {
            value,
            permission: RuntimePermission::Local,
            writable: false,
            moved_at: None,
            moved_by: None,
            linear,
            view: None,
            writable_alias_source: None,
        }
    }

    fn mutable_local(value: Value, linear: bool) -> Self {
        Self {
            value,
            permission: RuntimePermission::Local,
            writable: true,
            moved_at: None,
            moved_by: None,
            linear,
            view: None,
            writable_alias_source: None,
        }
    }

    fn writable_alias(source_place: String) -> Self {
        Self {
            value: Value::Unit,
            permission: RuntimePermission::Local,
            writable: true,
            moved_at: None,
            moved_by: None,
            linear: false,
            view: None,
            writable_alias_source: Some(source_place),
        }
    }

    fn view(value: Value, kind: RuntimeViewKind, source_place: String, bound_at: Span) -> Self {
        Self {
            value,
            permission: RuntimePermission::Local,
            writable: false,
            moved_at: None,
            moved_by: None,
            linear: false,
            view: Some(RuntimeView {
                kind,
                source_place,
                bound_at,
                invalidated_by: None,
            }),
            writable_alias_source: None,
        }
    }
}

impl From<ParamPermission> for RuntimePermission {
    fn from(permission: ParamPermission) -> Self {
        match permission {
            ParamPermission::Borrow => RuntimePermission::Borrow,
            ParamPermission::Change => RuntimePermission::Change,
            ParamPermission::Consume => RuntimePermission::Consume,
        }
    }
}

type Env = BTreeMap<String, RuntimeBinding>;

#[derive(Debug, Clone)]
struct ActiveIteration {
    root: String,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FailureValue {
    root: FailureVariant,
    root_origin: Span,
    steps: Vec<FailureStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FailureStep {
    Propagate {
        span: Span,
        callee: String,
    },
    Wrap {
        outer: FailureVariant,
        span: Span,
        callee: String,
    },
}

impl FailureValue {
    fn root(root: FailureVariant, root_origin: Span) -> Self {
        Self {
            root,
            root_origin,
            steps: Vec::new(),
        }
    }

    fn propagate(mut self, span: Span, callee: String) -> Self {
        self.steps.push(FailureStep::Propagate { span, callee });
        self
    }

    fn wrap(mut self, outer: FailureVariant, span: Span, callee: String) -> Self {
        self.steps.push(FailureStep::Wrap {
            outer,
            span,
            callee,
        });
        self
    }

    fn identity(&self) -> String {
        self.steps
            .iter()
            .rev()
            .find_map(|step| match step {
                FailureStep::Wrap { outer, .. } => Some(outer.identity()),
                FailureStep::Propagate { .. } => None,
            })
            .unwrap_or_else(|| self.root.identity())
    }

    fn render(&self) -> String {
        let identities = self
            .steps
            .iter()
            .rev()
            .filter_map(|step| match step {
                FailureStep::Wrap { outer, .. } => Some(outer.identity()),
                FailureStep::Propagate { .. } => None,
            })
            .chain(std::iter::once(self.root.identity()))
            .collect::<Vec<_>>();
        let mut out = format!("failure: {}", identities[0]);
        let mut wrap_index = 0;
        for step in self.steps.iter().rev() {
            match step {
                FailureStep::Propagate { span, callee } => out.push_str(&format!(
                    "\n  propagated at {} while calling `{callee}`",
                    location(span)
                )),
                FailureStep::Wrap {
                    outer: _,
                    span,
                    callee,
                } => {
                    out.push_str(&format!(
                        "\n  wrapped at {} while calling `{callee}`",
                        location(span)
                    ));
                    wrap_index += 1;
                    out.push_str(&format!("\ncaused by: {}", identities[wrap_index]));
                }
            }
        }
        out.push_str(&format!(
            "\n  originated at {}",
            location(&self.root_origin)
        ));
        out
    }
}

pub fn run_program(program: &Program, entry: Option<&str>, raw_args: &[String]) -> RunReport {
    let interpreter = Interpreter {
        program,
        failure_catalog: FailureCatalog::from_program(program),
        diagnostics: RefCell::new(Vec::new()),
        active_iterations: RefCell::new(Vec::new()),
        active_app: RefCell::new(None),
    };
    let outcome = match interpreter.run(entry, raw_args) {
        Ok((TaskResult::Returned(value), false)) => RunOutcome::Success(display_value(&value)),
        Ok((TaskResult::Returned(Value::Unit), true)) => RunOutcome::AppSuccess,
        Ok((TaskResult::Returned(_), true)) => RunOutcome::Trap(
            "app start returned a non-Unit value after static checking".to_string(),
        ),
        Ok((TaskResult::Failed(value), false)) => RunOutcome::Failure(value.render()),
        Ok((TaskResult::Failed(value), true)) => RunOutcome::AppFailure(value.render()),
        Ok((TaskResult::ContractViolation, _)) => RunOutcome::ContractViolation,
        Err(message) => RunOutcome::Trap(message),
    };
    let diagnostics = interpreter.diagnostics.into_inner();
    RunReport {
        outcome,
        diagnostics,
    }
}

struct Interpreter<'a> {
    program: &'a Program,
    failure_catalog: FailureCatalog,
    diagnostics: RefCell<Vec<Diagnostic>>,
    active_iterations: RefCell<Vec<ActiveIteration>>,
    active_app: RefCell<Option<&'a App>>,
}

enum TaskResult {
    Returned(Value),
    Failed(FailureValue),
    ContractViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContractKind {
    Needs,
    Ensures,
}

impl<'a> Interpreter<'a> {
    fn run(&self, entry: Option<&str>, raw_args: &[String]) -> Result<(TaskResult, bool), String> {
        let (task, app_mode) = self.entry_task(entry)?;
        let args = self.parse_args(task, raw_args)?;
        self.execute_task(task, args)
            .map(|result| (result, app_mode))
    }

    fn entry_task(&self, entry: Option<&str>) -> Result<(&'a Task, bool), String> {
        if let Some(name) = entry {
            return self
                .find_task(name)
                .map(|task| (task, false))
                .ok_or_else(|| format!("entry task `{name}` was not found"));
        }

        let analysis = app_entry::analyze(self.program);
        if let Some(diagnostic) = analysis.diagnostic {
            let code = diagnostic.code.as_str();
            let title = diagnostic.code.title();
            self.diagnostics.borrow_mut().push(diagnostic);
            return Err(format!("{code} {title}"));
        }
        if let Some(entry) = analysis.entry {
            self.active_app.replace(Some(entry.app));
            return Ok((entry.task, true));
        }

        let mut tasks = Vec::new();
        for file in &self.program.files {
            collect_tasks(&file.items, &mut tasks);
        }
        match tasks.as_slice() {
            [task] => Ok((*task, false)),
            [] => Err("no task is available to run".to_string()),
            _ => Err("multiple tasks are available; pass `--entry <task>`".to_string()),
        }
    }

    fn find_task(&self, name: &str) -> Option<&'a Task> {
        if let Some(app) = *self.active_app.borrow() {
            return find_task_in_items(&app.items, name);
        }
        self.program
            .files
            .iter()
            .find_map(|file| find_task_in_items(&file.items, name))
    }

    fn parse_args(&self, task: &Task, raw_args: &[String]) -> Result<Vec<Value>, String> {
        if raw_args.len() != task.params.len() {
            return Err(format!(
                "task `{}` expects {} argument(s), got {}",
                task.name,
                task.params.len(),
                raw_args.len()
            ));
        }

        task.params
            .iter()
            .zip(raw_args)
            .map(|(param, raw)| parse_arg(&param.ty, raw))
            .collect()
    }

    fn execute_task(&self, task: &Task, args: Vec<Value>) -> Result<TaskResult, String> {
        let Some(does) = task.section("does") else {
            return Err(format!("task `{}` has no `does:` section", task.name));
        };
        let body = core_body::analyze_does_section(does);
        let mut existing_names = task
            .params
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<BTreeSet<_>>();
        for section_name in ["uses", "changes"] {
            let Some(section) = task.section(section_name) else {
                continue;
            };
            existing_names.extend(
                section
                    .lines
                    .iter()
                    .filter(|line| is_meaningful_line_text(&line.text))
                    .map(|line| place_root(line.text.trim())),
            );
        }
        let alias_analysis =
            writable_field_alias::analyze_with_existing_names(&body.statements, &existing_names);
        self.preflight_typed_failures(task, &body.statements)?;
        self.preflight_writable_aliases(task, &body.statements, &alias_analysis)?;

        let mut env = Env::new();
        for (param, value) in task.params.iter().zip(args) {
            env.insert(
                param.name.clone(),
                RuntimeBinding::parameter(value, param.permission),
            );
        }

        if !self.evaluate_contract_section(task, "needs", ContractKind::Needs, &mut env)? {
            return Ok(TaskResult::ContractViolation);
        }

        self.capture_old_contract_values(task, &mut env)?;

        let lines = executable_lines(&does.lines);
        match self.eval_block(&lines, 0, lines.len(), &mut env, &task.name)? {
            Flow::Return { value, root, span } => {
                self.ensure_return_dependency(task, root.as_deref(), &span)?;
                self.finish_success(task, value, &env)
            }
            Flow::Fail(value) => Ok(TaskResult::Failed(value)),
            Flow::Continue => {
                self.ensure_linear_closed_on_exit(&env, &task.name, "fallthrough", &task.span)?;
                self.finish_success(task, Value::Unit, &env)
            }
            Flow::ContractViolation => Ok(TaskResult::ContractViolation),
        }
    }

    fn preflight_writable_aliases(
        &self,
        task: &Task,
        statements: &[BodyStatement],
        analysis: &AliasAnalysis,
    ) -> Result<(), String> {
        for binding in &analysis.bindings {
            let borrowed_owner = task.params.iter().any(|param| {
                param.name == binding.owner_root && param.permission == ParamPermission::Borrow
            });
            if borrowed_owner {
                return Err(self.writable_alias_authority_trap(
                    task,
                    binding,
                    DiagnosticCode::BORROW_PARAMETER_MUTATION,
                    "borrow",
                ));
            }
        }

        if let Some(issue) = analysis.issues.first() {
            let (code, message) = match issue.kind {
                AliasIssueKind::Overlap => (
                    DiagnosticCode::WRITABLE_ALIAS_OVERLAP,
                    writable_field_alias::overlap_message(issue),
                ),
                AliasIssueKind::Unsupported => (
                    DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS,
                    writable_field_alias::unsupported_message(issue),
                ),
            };
            self.diagnostics.borrow_mut().push(
                Diagnostic::error(code, message, Some(issue.conflict_span.clone()))
                    .with_help(writable_field_alias::issue_help(&task.name, issue)),
            );
            return Err(format!("{} {}", code.as_str(), code.title()));
        }

        for binding in &analysis.bindings {
            let permission = task
                .params
                .iter()
                .find(|param| param.name == binding.owner_root)
                .map(|param| param.permission);
            match permission {
                Some(ParamPermission::Change | ParamPermission::Consume) => continue,
                Some(ParamPermission::Borrow) => {
                    return Err(self.writable_alias_authority_trap(
                        task,
                        binding,
                        DiagnosticCode::BORROW_PARAMETER_MUTATION,
                        "borrow",
                    ));
                }
                None => {}
            }

            let owner = statements
                .iter()
                .enumerate()
                .take(binding.binding_index)
                .rev()
                .find_map(|(_index, statement)| {
                    body_binding_name(statement)
                        .and_then(|name| (name == binding.owner_root).then_some(statement.kind))
                });
            if owner == Some("mutable_binding") {
                continue;
            }
            let authority = if owner == Some("let_binding") {
                "immutable let"
            } else {
                "unknown"
            };
            return Err(self.writable_alias_authority_trap(
                task,
                binding,
                DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS,
                authority,
            ));
        }
        Ok(())
    }

    fn preflight_typed_failures(
        &self,
        task: &Task,
        statements: &[BodyStatement],
    ) -> Result<(), String> {
        let scoped_catalog;
        let catalog = if let Some(app) = *self.active_app.borrow() {
            scoped_catalog = FailureCatalog::from_items(&app.items);
            &scoped_catalog
        } else {
            &self.failure_catalog
        };
        let analysis = typed_failure::analyze_task(task, statements, catalog);
        let Some(fact) = analysis
            .facts
            .values()
            .find(|fact| fact.diagnostic_code.is_some())
        else {
            return Ok(());
        };
        let code = fact.diagnostic_code.expect("checked above");
        let message = typed_failure::diagnostic_message(fact);
        let mut diagnostic = Diagnostic::error(code, message.clone(), Some(fact.call_span.clone()));
        if let Some(help) = &fact.help {
            diagnostic = diagnostic.with_help(help.clone());
        }
        self.diagnostics.borrow_mut().push(diagnostic);
        Err(format!("{}: {message}", code.as_str()))
    }

    fn writable_alias_authority_trap(
        &self,
        task: &Task,
        binding: &AliasBinding,
        code: DiagnosticCode,
        authority: &str,
    ) -> String {
        self.diagnostics.borrow_mut().push(
            Diagnostic::error(
                code,
                writable_field_alias::authority_message(binding),
                Some(binding.binding_span.clone()),
            )
            .with_help(writable_field_alias::authority_help(
                &task.name, binding, authority,
            )),
        );
        format!("{} {}", code.as_str(), code.title())
    }

    fn ensure_return_dependency(
        &self,
        task: &Task,
        returned_root: Option<&str>,
        span: &Span,
    ) -> Result<(), String> {
        let Some(dependency) = task
            .result
            .as_deref()
            .and_then(return_dependency::parse_return_dependency)
        else {
            return Ok(());
        };
        let source = dependency.source.as_str();
        let source_is_parameter = return_dependency::is_bare_source_name(source)
            && task.params.iter().any(|param| param.name == source);
        if !source_is_parameter || returned_root != Some(source) {
            return Err(self.return_dependency_trap(&task.name, source, span));
        }
        Ok(())
    }

    fn finish_success(&self, task: &Task, value: Value, env: &Env) -> Result<TaskResult, String> {
        let mut exit_env = env.clone();
        exit_env.insert(
            "result".to_string(),
            RuntimeBinding::local(value.clone(), false),
        );
        if !self.evaluate_contract_section(task, "ensures", ContractKind::Ensures, &mut exit_env)? {
            return Ok(TaskResult::ContractViolation);
        }
        Ok(TaskResult::Returned(value))
    }

    fn capture_old_contract_values(&self, task: &Task, env: &mut Env) -> Result<(), String> {
        let Some(section) = task.section("ensures") else {
            return Ok(());
        };
        let allowed_names = contract_allowed_names(task, ContractKind::Ensures);
        for line in &section.lines {
            let text = line.text.trim();
            if !is_meaningful_line_text(text) || !validate_predicate_v1(text, &allowed_names, true)
            {
                continue;
            }
            for inner in collect_old_references(text) {
                let key = format!("old({inner})");
                if env.contains_key(&key) {
                    continue;
                }
                let value = match self.eval_expr(&inner, env, &line.span, &task.name)? {
                    Evaluated::Value(value) => value,
                    Evaluated::Failure(value) => {
                        return Err(format!(
                            "old capture of `{inner}` produced failure {}",
                            value.identity()
                        ));
                    }
                    Evaluated::ContractViolation => {
                        return Err(format!("old capture of `{inner}` hit a contract violation"));
                    }
                };
                env.insert(key, RuntimeBinding::local(value, false));
            }
        }
        Ok(())
    }

    fn evaluate_contract_section(
        &self,
        task: &Task,
        section_name: &str,
        kind: ContractKind,
        env: &mut Env,
    ) -> Result<bool, String> {
        let Some(section) = task.section(section_name) else {
            return Ok(true);
        };

        let allowed_names = contract_allowed_names(task, kind);
        for line in &section.lines {
            let text = line.text.trim();
            if !is_meaningful_line_text(text) {
                continue;
            }

            if !validate_predicate_v1(text, &allowed_names, kind == ContractKind::Ensures) {
                self.diagnostics.borrow_mut().push(
                    Diagnostic::warning(
                        DiagnosticCode::UNCHECKED_PROSE_CONTRACT,
                        format!("unchecked prose {} contract: {text}", kind.section_name()),
                        Some(line.span.clone()),
                    )
                    .with_help(
                        "Predicate v1 checks one comparison over parameters, `result`, arithmetic, `list_len(...)`, and `old(...)` of entry-readable parameter places in `ensures:`; prose remains visible but unchecked.",
                    ),
                );
                continue;
            }

            let value = match self.eval_expr(text, env, &line.span, &task.name)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => {
                    return Err(format!(
                        "contract predicate `{text}` produced failure {}",
                        value.identity()
                    ));
                }
                Evaluated::ContractViolation => return Ok(false),
            };
            if as_bool(&value)? {
                continue;
            }

            let (code, message, help) = match kind {
                ContractKind::Needs => (
                    DiagnosticCode::NEEDS_CONTRACT_VIOLATION,
                    format!("caller did not satisfy needs: {text}"),
                    format!(
                        "The caller must pass arguments that make this predicate true before task `{}` runs.",
                        task.name
                    ),
                ),
                ContractKind::Ensures => (
                    DiagnosticCode::ENSURES_CONTRACT_VIOLATION,
                    format!("task `{}` did not satisfy ensures: {text}", task.name),
                    "Fix the task body or change the contract; task blame means the caller met entry conditions but the implementation broke its promise."
                        .to_string(),
                ),
            };
            self.diagnostics
                .borrow_mut()
                .push(Diagnostic::error(code, message, Some(line.span.clone())).with_help(help));
            return Ok(false);
        }

        Ok(true)
    }

    fn eval_block(
        &self,
        lines: &[ExecLine],
        start: usize,
        end: usize,
        env: &mut Env,
        task_name: &str,
    ) -> Result<Flow, String> {
        let mut index = start;
        while index < end {
            let line = &lines[index];
            let text = line.text.as_str();

            if text == "}" {
                return Ok(Flow::Continue);
            }

            if let Some(condition) = header_body(text, "if") {
                let close = matching_close(lines, index)?;
                match self.eval_expr(condition, env, &line.span, task_name)? {
                    Evaluated::Value(value) if as_bool(&value)? => {
                        let flow = self.eval_block(lines, index + 1, close, env, task_name)?;
                        if flow != Flow::Continue {
                            return Ok(flow);
                        }
                    }
                    Evaluated::Value(_) => {}
                    Evaluated::Failure(value) => return Ok(Flow::Fail(value)),
                    Evaluated::ContractViolation => return Ok(Flow::ContractViolation),
                }
                index = close + 1;
                continue;
            }

            if let Some(rest) = header_body(text, "for each") {
                let close = matching_close(lines, index)?;
                let (name, collection_expr) = rest
                    .split_once(" in ")
                    .ok_or_else(|| format!("{}: malformed `for each` header", line.location))?;
                let collection =
                    match self.eval_expr(collection_expr.trim(), env, &line.span, task_name)? {
                        Evaluated::Value(value) => value,
                        Evaluated::Failure(value) => return Ok(Flow::Fail(value)),
                        Evaluated::ContractViolation => return Ok(Flow::ContractViolation),
                    };
                let Value::List(values) = collection else {
                    return Err(format!(
                        "{}: `for each` requires a list value",
                        line.location
                    ));
                };

                let active_iteration = iteration_root(collection_expr.trim())
                    .map(|root| self.push_active_iteration(root, line.span.clone()));
                let name = name.trim();
                let previous = env.get(name).cloned();
                for value in values {
                    env.insert(name.to_string(), RuntimeBinding::local(value, false));
                    let flow = match self.eval_block(lines, index + 1, close, env, task_name) {
                        Ok(flow) => flow,
                        Err(error) => {
                            restore_binding(env, name, previous.clone());
                            if active_iteration.is_some() {
                                self.pop_active_iteration();
                            }
                            return Err(error);
                        }
                    };
                    if flow != Flow::Continue {
                        restore_binding(env, name, previous);
                        if active_iteration.is_some() {
                            self.pop_active_iteration();
                        }
                        return Ok(flow);
                    }
                }
                restore_binding(env, name, previous);
                if active_iteration.is_some() {
                    self.pop_active_iteration();
                }
                index = close + 1;
                continue;
            }

            if let Some(expr) = strip_keyword(text, "return") {
                return match self.eval_expr(expr, env, &line.span, task_name)? {
                    Evaluated::Value(value) => {
                        let root = return_dependency::visible_view_source_root(expr);
                        if let Some(root) = root.as_deref()
                            && !is_linear_binding(env, root)
                        {
                            self.mark_moved(env, root, &line.span, "return");
                        }
                        self.ensure_linear_closed_on_exit(env, task_name, "return", &line.span)?;
                        Ok(Flow::Return {
                            value,
                            root,
                            span: line.span.clone(),
                        })
                    }
                    Evaluated::Failure(value) => {
                        self.ensure_linear_closed_on_exit(env, task_name, "fail", &line.span)?;
                        Ok(Flow::Fail(value))
                    }
                    Evaluated::ContractViolation => Ok(Flow::ContractViolation),
                };
            }

            if let Some(expr) = strip_keyword(text, "fail") {
                let Some(variant) = typed_failure::parse_failure_variant(expr) else {
                    return Err(format!(
                        "{}: typed `fail` requires `ErrorRoot.variant`",
                        line.location
                    ));
                };
                self.ensure_linear_closed_on_exit(env, task_name, "fail", &line.span)?;
                return Ok(Flow::Fail(FailureValue::root(variant, line.span.clone())));
            }

            if let Some(rest) = strip_keyword(text, "change") {
                if let Some(flow) = self.eval_binding(rest, env, &line.span, task_name, true)? {
                    return Ok(flow);
                }
                index += 1;
                continue;
            }

            if let Some(rest) = strip_keyword(text, "let") {
                if let Some(flow) = self.eval_binding(rest, env, &line.span, task_name, false)? {
                    return Ok(flow);
                }
                index += 1;
                continue;
            }

            if let Some(rest) = strip_keyword(text, "set") {
                if let Some(flow) = self.eval_set(rest, env, &line.span, task_name)? {
                    return Ok(flow);
                }
                index += 1;
                continue;
            }

            return Err(format!(
                "{}: unsupported executable line `{text}`",
                line.location
            ));
        }
        Ok(Flow::Continue)
    }

    fn eval_binding(
        &self,
        rest: &str,
        env: &mut Env,
        span: &Span,
        task_name: &str,
        mutable: bool,
    ) -> Result<Option<Flow>, String> {
        let (left, expr) = rest
            .split_once('=')
            .ok_or_else(|| format!("binding `{rest}` is missing an initializer"))?;
        let annotation = left.split_once(':').map(|(_name, ty)| ty.trim());
        let name = left.split_once(':').map_or(left, |(name, _ty)| name).trim();
        if name.is_empty() {
            return Err(format!("binding `{rest}` is missing a name"));
        }
        let expr = expr.trim();
        if !mutable
            && let Some(alias) = writable_field_alias::exact_binding_text(&format!("let {rest}"))
        {
            let source_root = place_root(&alias.source_place);
            self.ensure_can_set(env, &alias.source_place, &source_root, span, task_name)?;
            env.insert(
                name.to_string(),
                RuntimeBinding::writable_alias(alias.source_place),
            );
            return Ok(None);
        }
        if !mutable && let Some((kind, source_place)) = borrowed_view_source(expr) {
            let value = match self.eval_expr(source_place, env, span, task_name)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => return Ok(Some(Flow::Fail(value))),
                Evaluated::ContractViolation => return Ok(Some(Flow::ContractViolation)),
            };
            env.insert(
                name.to_string(),
                RuntimeBinding::view(value, kind, source_place.to_string(), span.clone()),
            );
            return Ok(None);
        }
        let linear = annotation.is_some_and(is_linear_resource_type);
        match self.eval_expr(expr, env, span, task_name)? {
            Evaluated::Value(value) => {
                let binding = if mutable {
                    RuntimeBinding::mutable_local(value, linear)
                } else {
                    RuntimeBinding::local(value, linear)
                };
                env.insert(name.to_string(), binding);
                Ok(None)
            }
            Evaluated::Failure(value) => Ok(Some(Flow::Fail(value))),
            Evaluated::ContractViolation => Ok(Some(Flow::ContractViolation)),
        }
    }

    fn eval_set(
        &self,
        rest: &str,
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Option<Flow>, String> {
        let (place, expr) = rest
            .split_once('=')
            .ok_or_else(|| format!("set `{rest}` is missing `=`"))?;
        let place = place.trim();
        let effective_place = self.resolve_writable_alias_place(env, place)?;
        let root = place_root(&effective_place);
        self.ensure_can_set(env, &effective_place, &root, span, task_name)?;
        match self.eval_expr(expr.trim(), env, span, task_name)? {
            Evaluated::Value(value) => {
                self.write_place(env, &effective_place, value)?;
                invalidate_field_views(env, &effective_place, span);
                Ok(None)
            }
            Evaluated::Failure(value) => Ok(Some(Flow::Fail(value))),
            Evaluated::ContractViolation => Ok(Some(Flow::ContractViolation)),
        }
    }

    fn eval_expr(
        &self,
        text: &str,
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        let text = trim_outer_parens(text.trim());
        if text.is_empty() {
            return Ok(Evaluated::Value(Value::Unit));
        }

        if typed_failure::is_try_candidate(text) {
            let parsed = typed_failure::parse_try_expression(text).ok_or_else(|| {
                format!(
                    "{}: unsupported typed-failure propagation shape",
                    location(span)
                )
            })?;
            return match self.eval_expr(&parsed.call.source, env, span, task_name)? {
                Evaluated::Value(value) => Ok(Evaluated::Value(value)),
                Evaluated::Failure(failure) => {
                    let failure = if let Some(wrapper) = parsed.wrapper {
                        failure.wrap(wrapper, span.clone(), parsed.call.callee)
                    } else {
                        failure.propagate(span.clone(), parsed.call.callee)
                    };
                    Ok(Evaluated::Failure(failure))
                }
                Evaluated::ContractViolation => Ok(Evaluated::ContractViolation),
            };
        }

        if let Some((left, right)) = split_word_operator(text, "or") {
            let left = match self.eval_expr(left, env, span, task_name)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
            };
            if as_bool(&left)? {
                return Ok(Evaluated::Value(Value::Bool(true)));
            }
            let right = match self.eval_expr(right, env, span, task_name)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
            };
            return Ok(Evaluated::Value(Value::Bool(as_bool(&right)?)));
        }

        if let Some((left, right)) = split_word_operator(text, "and") {
            let left = match self.eval_expr(left, env, span, task_name)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
            };
            if !as_bool(&left)? {
                return Ok(Evaluated::Value(Value::Bool(false)));
            }
            let right = match self.eval_expr(right, env, span, task_name)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
            };
            return Ok(Evaluated::Value(Value::Bool(as_bool(&right)?)));
        }

        if let Some((left, op, right)) =
            split_top_level_operator(text, &["==", "!=", "<=", ">=", "<", ">"])
        {
            return self.eval_comparison(text, (left, op, right), env, span, task_name);
        }

        if let Some((left, op, right)) = split_top_level_operator(text, &["+", "-"]) {
            return self.eval_integer_binary(text, (left, op, right), env, span, task_name);
        }

        if let Some((left, op, right)) = split_top_level_operator(text, &["*", "/"]) {
            return self.eval_integer_binary(text, (left, op, right), env, span, task_name);
        }

        self.eval_primary(text, env, span, task_name)
    }

    fn eval_comparison(
        &self,
        source: &str,
        parts: (&str, &str, &str),
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        let (left, op, right) = parts;
        let left = match self.eval_expr(left, env, span, task_name)? {
            Evaluated::Value(value) => value,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let right = match self.eval_expr(right, env, span, task_name)? {
            Evaluated::Value(value) => value,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let value = match op {
            "==" => left == right,
            "!=" => left != right,
            "<" => as_int(&left)? < as_int(&right)?,
            ">" => as_int(&left)? > as_int(&right)?,
            "<=" => as_int(&left)? <= as_int(&right)?,
            ">=" => as_int(&left)? >= as_int(&right)?,
            _ => return Err(format!("unsupported comparison `{source}`")),
        };
        Ok(Evaluated::Value(Value::Bool(value)))
    }

    fn eval_integer_binary(
        &self,
        source: &str,
        parts: (&str, &str, &str),
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        let (left, op, right) = parts;
        let left = match self.eval_expr(left, env, span, task_name)? {
            Evaluated::Value(value) => as_int(&value)?,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let right = match self.eval_expr(right, env, span, task_name)? {
            Evaluated::Value(value) => as_int(&value)?,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let value = match op {
            "+" => left
                .checked_add(right)
                .ok_or_else(|| format!("integer overflow while evaluating `{source}`"))?,
            "-" => left
                .checked_sub(right)
                .ok_or_else(|| format!("integer overflow while evaluating `{source}`"))?,
            "*" => left
                .checked_mul(right)
                .ok_or_else(|| format!("integer overflow while evaluating `{source}`"))?,
            "/" => {
                if right == 0 {
                    return Err(format!("division by zero while evaluating `{source}`"));
                }
                left.checked_div(right)
                    .ok_or_else(|| format!("integer overflow while evaluating `{source}`"))?
            }
            _ => return Err(format!("unsupported integer operator `{op}`")),
        };
        Ok(Evaluated::Value(Value::Int(value)))
    }

    fn eval_primary(
        &self,
        text: &str,
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        if text == "true" {
            return Ok(Evaluated::Value(Value::Bool(true)));
        }
        if text == "false" {
            return Ok(Evaluated::Value(Value::Bool(false)));
        }
        if let Ok(value) = parse_int_literal(text) {
            return Ok(Evaluated::Value(Value::Int(value)));
        }
        if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
            return Ok(Evaluated::Value(Value::Text(
                text[1..text.len() - 1].to_string(),
            )));
        }
        if text.starts_with('[') && text.ends_with(']') {
            let inside = &text[1..text.len() - 1];
            let mut values = Vec::new();
            for item in split_arguments(inside) {
                match self.eval_expr(item, env, span, task_name)? {
                    Evaluated::Value(value) => values.push(value),
                    Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                    Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
                }
            }
            return Ok(Evaluated::Value(Value::List(values)));
        }
        if text.starts_with('{') && text.ends_with('}') {
            let inside = &text[1..text.len() - 1];
            let mut fields = BTreeMap::new();
            for field in split_arguments(inside) {
                let (name, value_text) = field
                    .split_once(':')
                    .ok_or_else(|| format!("record field `{field}` is missing `:`"))?;
                let name = name.trim();
                if !is_record_field_name(name) {
                    return Err(format!(
                        "record field name `{name}` is not a valid Hum field name"
                    ));
                }
                match self.eval_expr(value_text.trim(), env, span, task_name)? {
                    Evaluated::Value(value) => {
                        fields.insert(name.to_string(), value);
                    }
                    Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                    Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
                }
            }
            return Ok(Evaluated::Value(Value::Record(fields)));
        }
        if let Some((callee, args)) = split_call(text) {
            let callee = callee.trim();
            if return_dependency::is_closed_view_deriving_operation(callee) {
                return self.eval_slice_until(args, env, span, task_name);
            }
            if callee == "list_append" {
                return self.eval_list_append(args, env, span, task_name);
            }
            if callee == "old" {
                let key = format!("old({})", args.trim());
                if let Some(binding) = env.get(&key) {
                    return Ok(Evaluated::Value(binding.value.clone()));
                }
                return Err(format!(
                    "`{key}` was not captured at task entry; old(...) is available only in `ensures:` over parameters or parameter fields readable when the task starts"
                ));
            }
            if callee == "list_len" {
                return self.eval_list_len(args, env, span, task_name);
            }
            let Some(task) = self.find_task(callee) else {
                return Err(format!("task `{callee}` was not found"));
            };
            let raw_args = split_arguments(args);
            if raw_args.len() != task.params.len() {
                return Err(format!(
                    "task `{}` expects {} argument(s), got {}",
                    task.name,
                    task.params.len(),
                    raw_args.len()
                ));
            }
            let mut values = Vec::new();
            for arg in raw_args {
                if let Some(root) = consume_argument_root(arg) {
                    let value = self.read_consume_value(env, &root, span, task_name)?;
                    self.mark_moved(env, &root, span, &task.name);
                    values.push(value);
                    continue;
                }
                let arg = strip_borrow_or_change_argument(arg).unwrap_or(arg);
                match self.eval_expr(arg, env, span, task_name)? {
                    Evaluated::Value(value) => values.push(value),
                    Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                    Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
                }
            }
            return match self.execute_task(task, values)? {
                TaskResult::Returned(value) => Ok(Evaluated::Value(value)),
                TaskResult::Failed(value) => Ok(Evaluated::Failure(value)),
                TaskResult::ContractViolation => Ok(Evaluated::ContractViolation),
            };
        }
        if let Some((base, index)) = element_place::split_element_place(text)
            && env.contains_key(base)
        {
            let value = self.read_value(env, base, span, task_name)?;
            let Value::List(values) = value else {
                return Err(format!("{base} is not a list"));
            };
            return values
                .get(index)
                .cloned()
                .map(Evaluated::Value)
                .ok_or_else(|| format!("list {base} has no element at index {index}"));
        }
        if env.contains_key(text) {
            return self
                .read_value(env, text, span, task_name)
                .map(Evaluated::Value);
        }
        if let Some((base, field)) = field_place::split_field_place(text)
            && env.contains_key(base)
        {
            let value = self.read_value(env, base, span, task_name)?;
            let Value::Record(fields) = value else {
                return Err(format!("`{base}` is not a record"));
            };
            return fields
                .get(field)
                .cloned()
                .map(Evaluated::Value)
                .ok_or_else(|| format!("record `{base}` has no field `{field}`"));
        }
        if let Some((base, _field)) = text.split_once('.')
            && base
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
        {
            return Ok(Evaluated::Value(Value::Variant(text.to_string())));
        }
        Err(format!("unknown expression `{text}`"))
    }

    fn eval_list_append(
        &self,
        args: &str,
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        let raw_args = split_arguments(args);
        if raw_args.len() != 2 {
            return Err(format!(
                "list_append expects 2 argument(s), got {}",
                raw_args.len()
            ));
        }
        let list_place = strip_keyword(raw_args[0].trim(), "change")
            .ok_or_else(|| "list_append first argument must be `change list`".to_string())?;
        let root = place_root(list_place);
        if let Some(loop_span) = self.active_iteration_for(&root) {
            return Err(self.iteration_mutation_trap(task_name, &root, span, &loop_span));
        }
        self.ensure_can_set(env, list_place, &root, span, task_name)?;
        let item = match self.eval_expr(raw_args[1], env, span, task_name)? {
            Evaluated::Value(value) => value,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let Some(binding) = env.get_mut(&root) else {
            return Err(format!("cannot append to unknown list `{root}`"));
        };
        let Value::List(values) = &mut binding.value else {
            return Err(format!("`{root}` is not a list"));
        };
        values.push(item);
        binding.moved_at = None;
        binding.moved_by = None;
        invalidate_element_views_for_growth(env, &root, span);
        Ok(Evaluated::Value(Value::Unit))
    }

    fn eval_slice_until(
        &self,
        args: &str,
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        let raw_args = split_arguments(args);
        if raw_args.len() != 2 {
            return Err(format!(
                "slice_until expects 2 argument(s), got {}",
                raw_args.len()
            ));
        }
        let source_expr = strip_borrow_or_change_argument(raw_args[0]).unwrap_or(raw_args[0]);
        let separator_expr = strip_borrow_or_change_argument(raw_args[1]).unwrap_or(raw_args[1]);
        let source = match self.eval_expr(source_expr, env, span, task_name)? {
            Evaluated::Value(value) => as_text(&value)?.to_string(),
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let separator = match self.eval_expr(separator_expr, env, span, task_name)? {
            Evaluated::Value(value) => as_text(&value)?.to_string(),
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let prefix = if separator.is_empty() {
            ""
        } else {
            source
                .split_once(separator.as_str())
                .map_or(source.as_str(), |(head, _tail)| head)
        };
        Ok(Evaluated::Value(Value::Text(prefix.to_string())))
    }

    fn eval_list_len(
        &self,
        args: &str,
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        let raw_args = split_arguments(args);
        if raw_args.len() != 1 {
            return Err(format!(
                "list_len expects 1 argument(s), got {}",
                raw_args.len()
            ));
        }
        let list_expr = strip_borrow_or_change_argument(raw_args[0]).unwrap_or(raw_args[0]);
        let value = match self.eval_expr(list_expr, env, span, task_name)? {
            Evaluated::Value(value) => value,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let Value::List(values) = value else {
            return Err(format!("list_len expects a list, got {list_expr}"));
        };
        Ok(Evaluated::Value(Value::Int(values.len() as i64)))
    }

    fn push_active_iteration(&self, root: String, span: Span) {
        self.active_iterations
            .borrow_mut()
            .push(ActiveIteration { root, span });
    }

    fn pop_active_iteration(&self) {
        self.active_iterations.borrow_mut().pop();
    }

    fn active_iteration_for(&self, root: &str) -> Option<Span> {
        self.active_iterations
            .borrow()
            .iter()
            .rev()
            .find(|iteration| iteration.root == root)
            .map(|iteration| iteration.span.clone())
    }

    fn read_value(
        &self,
        env: &Env,
        name: &str,
        span: &Span,
        task_name: &str,
    ) -> Result<Value, String> {
        let root = place_root(name);
        let Some(binding) = env.get(&root) else {
            return Err(format!("unknown expression `{name}`"));
        };
        if let Some(move_span) = &binding.moved_at {
            return Err(self.use_after_move_trap(task_name, &root, span, move_span));
        }
        if let Some(view) = &binding.view
            && let Some(invalidation) = &view.invalidated_by
        {
            return Err(self.stale_view_trap(task_name, &root, view, span, invalidation));
        }
        if let Some(source_place) = binding.writable_alias_source.clone() {
            return self.read_direct_field_value(env, &source_place, span, task_name);
        }
        Ok(binding.value.clone())
    }

    fn read_direct_field_value(
        &self,
        env: &Env,
        place: &str,
        span: &Span,
        task_name: &str,
    ) -> Result<Value, String> {
        let Some((root, field)) = field_place::split_field_place(place) else {
            return Err(format!("unsupported writable alias source `{place}`"));
        };
        let value = self.read_value(env, root, span, task_name)?;
        let Value::Record(fields) = value else {
            return Err(format!("`{root}` is not a record"));
        };
        fields
            .get(field)
            .cloned()
            .ok_or_else(|| format!("record `{root}` has no field `{field}`"))
    }

    fn resolve_writable_alias_place(&self, env: &Env, place: &str) -> Result<String, String> {
        let root = place_root(place);
        let Some(binding) = env.get(&root) else {
            return Ok(place.to_string());
        };
        let Some(source_place) = &binding.writable_alias_source else {
            return Ok(place.to_string());
        };
        if place != root {
            return Err(format!(
                "writable alias `{root}` supports only direct local reads and writes"
            ));
        }
        Ok(source_place.clone())
    }

    fn read_consume_value(
        &self,
        env: &Env,
        name: &str,
        span: &Span,
        task_name: &str,
    ) -> Result<Value, String> {
        let root = place_root(name);
        let Some(binding) = env.get(&root) else {
            return Err(format!("unknown expression `{name}`"));
        };
        if let Some(move_span) = &binding.moved_at {
            if binding.linear {
                return Err(self.linear_double_consume_trap(
                    task_name,
                    &root,
                    span,
                    move_span,
                    binding.moved_by.as_deref().unwrap_or("consume"),
                ));
            }
            return Err(self.use_after_move_trap(task_name, &root, span, move_span));
        }
        Ok(binding.value.clone())
    }

    fn ensure_can_set(
        &self,
        env: &Env,
        place: &str,
        root: &str,
        span: &Span,
        task_name: &str,
    ) -> Result<(), String> {
        let Some(binding) = env.get(root) else {
            return Err(format!("cannot set unknown place `{root}`"));
        };
        if let Some(move_span) = &binding.moved_at {
            return Err(self.use_after_move_trap(task_name, root, span, move_span));
        }
        if binding.permission == RuntimePermission::Borrow {
            return Err(self.borrow_mutation_trap(task_name, place, root, span));
        }
        if !binding.writable {
            return Err(format!("cannot set immutable place `{root}`"));
        }
        Ok(())
    }

    fn write_place(&self, env: &mut Env, place: &str, value: Value) -> Result<(), String> {
        let root = place_root(place);
        let Some(binding) = env.get_mut(&root) else {
            return Err(format!("cannot set unknown place `{place}`"));
        };
        if let Some((_root, field)) = field_place::split_field_place(place) {
            let Value::Record(fields) = &mut binding.value else {
                return Err(format!("`{root}` is not a record"));
            };
            if !fields.contains_key(field) {
                return Err(format!("record `{root}` has no field `{field}`"));
            }
            fields.insert(field.to_string(), value);
        } else if place.contains('.') {
            return Err(format!("unsupported set place `{place}`"));
        } else {
            binding.value = value;
        }
        binding.moved_at = None;
        binding.moved_by = None;
        Ok(())
    }

    fn mark_moved(&self, env: &mut Env, root: &str, span: &Span, action: &str) {
        if let Some(binding) = env.get_mut(root)
            && matches!(
                binding.permission,
                RuntimePermission::Local | RuntimePermission::Consume
            )
        {
            binding.moved_at.get_or_insert_with(|| span.clone());
            if binding.moved_by.is_none() {
                binding.moved_by = Some(action.to_string());
            }
        }
    }

    fn ensure_linear_closed_on_exit(
        &self,
        env: &Env,
        task_name: &str,
        exit_kind: &str,
        span: &Span,
    ) -> Result<(), String> {
        if let Some((root, _binding)) = env
            .iter()
            .find(|(_root, binding)| binding.linear && binding.moved_at.is_none())
        {
            return Err(self.linear_not_consumed_trap(task_name, root, exit_kind, span));
        }
        Ok(())
    }

    fn use_after_move_trap(
        &self,
        task_name: &str,
        root: &str,
        span: &Span,
        move_span: &Span,
    ) -> String {
        self.diagnostics.borrow_mut().push(
            Diagnostic::error(
                DiagnosticCode::USE_AFTER_MOVE,
                format!("value `{root}` was used after it was moved"),
                Some(span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: `{root}` moved at {}:{}:{}; use it before that move or create a fresh owned value.",
                move_span.file, move_span.line, move_span.column
            )),
        );
        format!(
            "{} {}",
            DiagnosticCode::USE_AFTER_MOVE.as_str(),
            DiagnosticCode::USE_AFTER_MOVE.title()
        )
    }

    fn borrow_mutation_trap(
        &self,
        task_name: &str,
        place: &str,
        root: &str,
        span: &Span,
    ) -> String {
        let message = if place == root {
            format!("borrowed parameter `{root}` cannot be written")
        } else {
            format!("borrowed parameter `{root}` cannot write `{place}`")
        };
        self.diagnostics.borrow_mut().push(
            Diagnostic::error(
                DiagnosticCode::BORROW_PARAMETER_MUTATION,
                message,
                Some(span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: mark `{root}` as `change`, copy it into a `change` local, or remove the `set`."
            )),
        );
        format!(
            "{} {}",
            DiagnosticCode::BORROW_PARAMETER_MUTATION.as_str(),
            DiagnosticCode::BORROW_PARAMETER_MUTATION.title()
        )
    }

    fn iteration_mutation_trap(
        &self,
        task_name: &str,
        root: &str,
        mutation_span: &Span,
        loop_span: &Span,
    ) -> String {
        self.diagnostics.borrow_mut().push(
            Diagnostic::error(
                DiagnosticCode::ITERATION_MUTATION_CONFLICT,
                format!("cannot structurally mutate `{root}` while it is being iterated"),
                Some(mutation_span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: `list_append` mutates `{root}` at {}:{}:{} during `for each` started at {}:{}:{}; collect changes after the loop or iterate over a separate list.",
                mutation_span.file,
                mutation_span.line,
                mutation_span.column,
                loop_span.file,
                loop_span.line,
                loop_span.column
            )),
        );
        format!(
            "{} {}",
            DiagnosticCode::ITERATION_MUTATION_CONFLICT.as_str(),
            DiagnosticCode::ITERATION_MUTATION_CONFLICT.title()
        )
    }

    fn stale_view_trap(
        &self,
        task_name: &str,
        view_name: &str,
        view: &RuntimeView,
        use_span: &Span,
        invalidation: &RuntimeViewInvalidation,
    ) -> String {
        let (message, help) = match (view.kind, invalidation.kind) {
            (RuntimeViewKind::Field, RuntimeViewInvalidationKind::FieldWrite) => (
                format!(
                    "field view {view_name} was used after {} changed",
                    view.source_place
                ),
                format!(
                    "Fix task {task_name}: {view_name} borrowed {} at {}:{}:{}, but {} was written at {}:{}:{} before this use; re-borrow after the write or bind a value copy before the write.",
                    view.source_place,
                    view.bound_at.file,
                    view.bound_at.line,
                    view.bound_at.column,
                    view.source_place,
                    invalidation.span.file,
                    invalidation.span.line,
                    invalidation.span.column
                ),
            ),
            (RuntimeViewKind::Element, RuntimeViewInvalidationKind::ListAppend) => {
                let root = place_root(&view.source_place);
                (
                    format!("element view {view_name} was used after {root} grew"),
                    format!(
                        "Fix task {task_name}: {view_name} borrowed {} at {}:{}:{}, but list_append grew {root} at {}:{}:{} before this use; re-borrow after the append or copy the element value before the append.",
                        view.source_place,
                        view.bound_at.file,
                        view.bound_at.line,
                        view.bound_at.column,
                        invalidation.span.file,
                        invalidation.span.line,
                        invalidation.span.column
                    ),
                )
            }
            (RuntimeViewKind::Field, RuntimeViewInvalidationKind::ListAppend)
            | (RuntimeViewKind::Element, RuntimeViewInvalidationKind::FieldWrite) => (
                format!("view {view_name} was used after its source changed"),
                format!(
                    "Fix task {task_name}: {view_name} borrowed {} at {}:{}:{}, but the source changed at {}:{}:{} before this use; re-borrow after the change or copy the value first.",
                    view.source_place,
                    view.bound_at.file,
                    view.bound_at.line,
                    view.bound_at.column,
                    invalidation.span.file,
                    invalidation.span.line,
                    invalidation.span.column
                ),
            ),
        };
        self.diagnostics.borrow_mut().push(
            Diagnostic::error(
                DiagnosticCode::STALE_FIELD_VIEW,
                message,
                Some(use_span.clone()),
            )
            .with_help(help),
        );
        format!(
            "{} {}",
            DiagnosticCode::STALE_FIELD_VIEW.as_str(),
            DiagnosticCode::STALE_FIELD_VIEW.title()
        )
    }
    fn return_dependency_trap(&self, task_name: &str, source: &str, span: &Span) -> String {
        self.diagnostics.borrow_mut().push(
            Diagnostic::error(
                DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER,
                format!("returned view does not visibly depend on parameter `{source}`"),
                Some(span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: returned-view `from` source `{source}` must name a task parameter, and returns must visibly return that parameter or a closed-set view derivation such as `slice_until(source, separator)`; locals, internal references, and non-closed derivation chains remain rejected."
            )),
        );
        format!(
            "{} {}",
            DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER.as_str(),
            DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER.title()
        )
    }

    fn linear_not_consumed_trap(
        &self,
        task_name: &str,
        root: &str,
        exit_kind: &str,
        span: &Span,
    ) -> String {
        self.diagnostics.borrow_mut().push(
            Diagnostic::error(
                DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED,
                format!("linear resource `{root}` reached {exit_kind} without being consumed"),
                Some(span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: consume `{root}` exactly once with commit, rollback, close, or transfer before leaving this path."
            )),
        );
        format!(
            "{} {}",
            DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED.as_str(),
            DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED.title()
        )
    }

    fn linear_double_consume_trap(
        &self,
        task_name: &str,
        root: &str,
        span: &Span,
        move_span: &Span,
        moved_by: &str,
    ) -> String {
        self.diagnostics.borrow_mut().push(
            Diagnostic::error(
                DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE,
                format!("linear resource `{root}` was consumed twice"),
                Some(span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: `{root}` was already consumed by {moved_by} at {}:{}:{}; keep exactly one commit, rollback, close, or transfer on each path.",
                move_span.file, move_span.line, move_span.column
            )),
        );
        format!(
            "{} {}",
            DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE.as_str(),
            DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE.title()
        )
    }
}

impl ContractKind {
    fn section_name(self) -> &'static str {
        match self {
            ContractKind::Needs => "needs",
            ContractKind::Ensures => "ensures",
        }
    }
}

fn contract_allowed_names(task: &Task, kind: ContractKind) -> BTreeSet<String> {
    let mut names = task
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    if kind == ContractKind::Ensures {
        names.insert("result".to_string());
    }
    names
}

fn validate_predicate_v1(text: &str, allowed_names: &BTreeSet<String>, allow_old: bool) -> bool {
    let text = trim_outer_parens(text.trim());
    let Some((left, _op, right)) =
        split_top_level_operator(text, &["==", "!=", "<=", ">=", "<", ">"])
    else {
        return false;
    };

    split_top_level_operator(left, &["==", "!=", "<=", ">=", "<", ">"]).is_none()
        && split_top_level_operator(right, &["==", "!=", "<=", ">=", "<", ">"]).is_none()
        && validate_contract_operand(left, allowed_names, allow_old)
        && validate_contract_operand(right, allowed_names, allow_old)
}

fn validate_contract_operand(
    text: &str,
    allowed_names: &BTreeSet<String>,
    allow_old: bool,
) -> bool {
    let text = trim_outer_parens(text.trim());
    if text.is_empty()
        || split_word_operator(text, "and").is_some()
        || split_word_operator(text, "or").is_some()
    {
        return false;
    }
    if let Some((left, _op, right)) = split_top_level_operator(text, &["+", "-"]) {
        return validate_contract_operand(left, allowed_names, allow_old)
            && validate_contract_operand(right, allowed_names, allow_old);
    }
    if let Some((left, _op, right)) = split_top_level_operator(text, &["*", "/"]) {
        return validate_contract_operand(left, allowed_names, allow_old)
            && validate_contract_operand(right, allowed_names, allow_old);
    }
    if let Some(inner) = contract_call_operand(text, "old") {
        return allow_old && validate_old_inner(inner, allowed_names);
    }
    if let Some(inner) = contract_call_operand(text, "list_len") {
        return allowed_names.contains(inner)
            || field_place::split_field_place(inner)
                .is_some_and(|(root, _field)| allowed_names.contains(root));
    }
    text == "true"
        || text == "false"
        || parse_int_literal(text).is_ok()
        || allowed_names.contains(text)
        || field_place::split_field_place(text)
            .is_some_and(|(root, _field)| allowed_names.contains(root))
}

fn contract_call_operand<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    // Canonical strictness: the exact `name(` prefix with no gap, so spaced
    // forms fall to honest prose instead of validating without capturing.
    let inner = text
        .strip_prefix(name)?
        .strip_prefix('(')?
        .strip_suffix(')')?
        .trim();
    // The inner form must be a simple place, not a nested expression with
    // top-level parentheses or commas.
    (!inner.is_empty() && !inner.contains('(') && !inner.contains(',')).then_some(inner)
}

fn validate_old_inner(inner: &str, allowed_names: &BTreeSet<String>) -> bool {
    // old(...) captures entry state: parameters or parameter fields only,
    // never `result`, which does not exist at entry.
    validate_entry_readable_place(inner, allowed_names)
}

fn validate_entry_readable_place(inner: &str, allowed_names: &BTreeSet<String>) -> bool {
    if inner == "result" {
        return false;
    }
    if allowed_names.contains(inner) {
        return true;
    }
    field_place::split_field_place(inner)
        .is_some_and(|(root, _field)| root != "result" && allowed_names.contains(root))
}

fn collect_old_references(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while let Some(offset) = text[index..].find("old(") {
        let start = index + offset;
        let preceded_by_word =
            start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        let inner_start = start + 4;
        let Some(close_offset) = text[inner_start..].find(')') else {
            break;
        };
        let inner = text[inner_start..inner_start + close_offset].trim();
        if !preceded_by_word && !inner.is_empty() && !inner.contains('(') && !inner.contains(',') {
            let owned = inner.to_string();
            if !found.contains(&owned) {
                found.push(owned);
            }
        }
        index = inner_start + close_offset + 1;
    }
    found
}

fn executable_lines(lines: &[SectionLine]) -> Vec<ExecLine> {
    lines
        .iter()
        .filter_map(|line| {
            let text = line.text.trim();
            is_meaningful_line_text(text).then(|| ExecLine {
                text: text.to_string(),
                location: format!("{}:{}:{}", line.span.file, line.span.line, line.span.column),
                span: line.span.clone(),
            })
        })
        .collect()
}

fn collect_tasks<'a>(items: &'a [Item], tasks: &mut Vec<&'a Task>) {
    for item in items {
        match item {
            Item::Task(task) => tasks.push(task),
            Item::App(app) => collect_tasks(&app.items, tasks),
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn find_task_in_items<'a>(items: &'a [Item], name: &str) -> Option<&'a Task> {
    for item in items {
        match item {
            Item::Task(task) if task.name == name => return Some(task),
            Item::App(app) => {
                if let Some(task) = find_task_in_items(&app.items, name) {
                    return Some(task);
                }
            }
            Item::Task(_) | Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
    None
}

fn parse_arg(ty: &str, raw: &str) -> Result<Value, String> {
    match ty.trim() {
        "Int" => parse_int_literal(raw).map(Value::Int),
        "UInt" => {
            let value = parse_int_literal(raw)?;
            if value < 0 {
                Err(format!("UInt argument `{raw}` must not be negative"))
            } else {
                Ok(Value::Int(value))
            }
        }
        "Bool" => parse_bool(raw).map(Value::Bool),
        "Text" => Ok(Value::Text(raw.to_string())),
        ty if ty.starts_with("List ") => {
            let element_ty = ty.trim_start_matches("List ").trim();
            parse_list_arg(element_ty, raw)
        }
        other => parse_record_arg(other, raw),
    }
}

fn parse_list_arg(element_ty: &str, raw: &str) -> Result<Value, String> {
    let raw = raw.trim();
    let inside = raw
        .strip_prefix('[')
        .and_then(|text| text.strip_suffix(']'))
        .ok_or_else(|| format!("list argument `{raw}` must use `[a, b]` syntax"))?;
    let mut values = Vec::new();
    for item in split_arguments(inside) {
        values.push(parse_list_element(element_ty, item.trim())?);
    }
    Ok(Value::List(values))
}

fn parse_list_element(element_ty: &str, raw: &str) -> Result<Value, String> {
    match element_ty {
        "Bool" => parse_bool(raw).map(Value::Bool),
        "Int" | "UInt" | "Text" => parse_arg(element_ty, raw),
        other => parse_record_arg(other, raw),
    }
}

fn parse_record_arg(ty: &str, raw: &str) -> Result<Value, String> {
    let raw = raw.trim();
    let inside = raw
        .strip_prefix('{')
        .and_then(|text| text.strip_suffix('}'))
        .ok_or_else(|| format!("record argument for `{ty}` must use `{{field: value}}` syntax"))?;

    let mut fields = BTreeMap::new();
    for field in split_arguments(inside) {
        let (name, value_text) = field
            .split_once(':')
            .ok_or_else(|| format!("record field `{field}` is missing `:`"))?;
        let name = name.trim();
        if !is_record_field_name(name) {
            return Err(format!(
                "record field name `{name}` is not a valid Hum field name"
            ));
        }
        fields.insert(
            name.to_string(),
            parse_cli_literal_value(value_text.trim())?,
        );
    }

    Ok(Value::Record(fields))
}

fn parse_cli_literal_value(raw: &str) -> Result<Value, String> {
    let raw = raw.trim();
    if raw == "true" || raw == "false" {
        return parse_bool(raw).map(Value::Bool);
    }
    if let Ok(value) = parse_int_literal(raw) {
        return Ok(Value::Int(value));
    }
    if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
        return Ok(Value::Text(raw[1..raw.len() - 1].to_string()));
    }
    if raw.starts_with('[') && raw.ends_with(']') {
        let inside = &raw[1..raw.len() - 1];
        let mut values = Vec::new();
        for item in split_arguments(inside) {
            values.push(parse_cli_literal_value(item)?);
        }
        return Ok(Value::List(values));
    }
    if raw.starts_with('{') && raw.ends_with('}') {
        return parse_record_arg("record", raw);
    }

    Err(format!("unsupported CLI literal `{raw}`"))
}

fn is_record_field_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first == '_')
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn parse_bool(raw: &str) -> Result<bool, String> {
    match raw.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("Bool argument `{other}` must be `true` or `false`")),
    }
}

fn parse_int_literal(text: &str) -> Result<i64, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("empty integer literal".to_string());
    }
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty() || !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(format!("`{text}` is not an integer literal"));
    }
    text.parse::<i64>()
        .map_err(|_| format!("integer literal `{text}` is outside Int range"))
}

fn as_int(value: &Value) -> Result<i64, String> {
    match value {
        Value::Int(value) => Ok(*value),
        other => Err(format!("expected Int value, got {}", display_value(other))),
    }
}

fn as_bool(value: &Value) -> Result<bool, String> {
    match value {
        Value::Bool(value) => Ok(*value),
        other => Err(format!("expected Bool value, got {}", display_value(other))),
    }
}

fn as_text(value: &Value) -> Result<&str, String> {
    match value {
        Value::Text(value) => Ok(value),
        other => Err(format!("expected Text value, got {}", display_value(other))),
    }
}

fn display_value(value: &Value) -> String {
    match value {
        Value::Unit => "()".to_string(),
        Value::Int(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Text(value) => value.clone(),
        Value::Variant(value) => value.clone(),
        Value::List(values) => {
            let body = values
                .iter()
                .map(display_value)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{body}]")
        }
        Value::Record(fields) => {
            let body = fields
                .iter()
                .map(|(name, value)| format!("{name}: {}", display_value(value)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{body}}}")
        }
    }
}

fn location(span: &Span) -> String {
    format!(
        "{}:{}:{}",
        span.file.replace('\\', "/"),
        span.line,
        span.column
    )
}

fn iteration_root(text: &str) -> Option<String> {
    let text = strip_borrow_or_change_argument(text).unwrap_or(text).trim();
    if text.is_empty()
        || text.contains('(')
        || text.contains(' ')
        || text.starts_with('"')
        || text.starts_with('[')
        || text.starts_with('{')
    {
        return None;
    }
    Some(place_root(text))
}

fn consume_argument_root(text: &str) -> Option<String> {
    let rest = strip_keyword(text.trim(), "consume")?;
    let root = place_root(rest);
    if root.is_empty() { None } else { Some(root) }
}

fn strip_borrow_or_change_argument(text: &str) -> Option<&str> {
    ["borrow", "change"]
        .iter()
        .find_map(|keyword| strip_keyword(text.trim(), keyword))
}

fn borrowed_view_source(text: &str) -> Option<(RuntimeViewKind, &str)> {
    let source = strip_keyword(text.trim(), "borrow")?;
    if field_place::split_field_place(source).is_some() {
        return Some((RuntimeViewKind::Field, source));
    }
    if element_place::split_element_place(source).is_some() {
        return Some((RuntimeViewKind::Element, source));
    }
    None
}

fn body_binding_name(statement: &BodyStatement) -> Option<&str> {
    if !matches!(statement.kind, "let_binding" | "mutable_binding") {
        return None;
    }
    let keyword = if statement.kind == "let_binding" {
        "let"
    } else {
        "change"
    };
    let rest = strip_keyword(&statement.text, keyword)?;
    let (left, _initializer) = rest.split_once('=')?;
    let name = left
        .split_once(':')
        .map_or(left, |(name, _annotation)| name)
        .trim();
    (!name.is_empty()).then_some(name)
}

fn invalidate_field_views(env: &mut Env, place: &str, span: &Span) {
    if field_place::split_field_place(place).is_none() {
        return;
    }
    for binding in env.values_mut() {
        if let Some(view) = &mut binding.view
            && view.kind == RuntimeViewKind::Field
            && view.source_place == place
            && view.invalidated_by.is_none()
        {
            view.invalidated_by = Some(RuntimeViewInvalidation {
                span: span.clone(),
                kind: RuntimeViewInvalidationKind::FieldWrite,
            });
        }
    }
}

fn invalidate_element_views_for_growth(env: &mut Env, root: &str, span: &Span) {
    for binding in env.values_mut() {
        if let Some(view) = &mut binding.view
            && view.kind == RuntimeViewKind::Element
            && place_root(&view.source_place) == root
            && view.invalidated_by.is_none()
        {
            view.invalidated_by = Some(RuntimeViewInvalidation {
                span: span.clone(),
                kind: RuntimeViewInvalidationKind::ListAppend,
            });
        }
    }
}

fn is_linear_binding(env: &Env, root: &str) -> bool {
    env.get(root).is_some_and(|binding| binding.linear)
}

fn is_linear_resource_type(type_text: &str) -> bool {
    type_tokens(type_text)
        .into_iter()
        .any(|token| token.eq_ignore_ascii_case("Transaction") || token.ends_with("Transaction"))
}

fn type_tokens(type_text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in type_text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn place_root(text: &str) -> String {
    text.split(|ch: char| ch == '.' || ch == '[' || ch.is_whitespace() || ch == ',')
        .find(|part| !part.is_empty())
        .unwrap_or(text)
        .trim()
        .to_string()
}

fn restore_binding(env: &mut Env, name: &str, previous: Option<RuntimeBinding>) {
    if let Some(binding) = previous {
        env.insert(name.to_string(), binding);
    } else {
        env.remove(name);
    }
}

fn header_body<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = strip_keyword(text, keyword)?;
    rest.strip_suffix('{').map(str::trim)
}

fn strip_keyword<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    if text == keyword {
        return Some("");
    }
    text.strip_prefix(keyword)
        .and_then(|rest| rest.strip_prefix(char::is_whitespace))
        .map(str::trim)
}

fn matching_close(lines: &[ExecLine], open: usize) -> Result<usize, String> {
    let mut depth = 0usize;
    for (index, line) in lines.iter().enumerate().skip(open) {
        if line.text.ends_with('{') {
            depth = depth.saturating_add(1);
        }
        if line.text == "}" {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Ok(index);
            }
        }
    }
    Err(format!(
        "{}: block is missing closing `}}`",
        lines[open].location
    ))
}

fn split_call(text: &str) -> Option<(&str, &str)> {
    if !text.ends_with(')') {
        return None;
    }
    let open = find_top_level_char(text, '(')?;
    Some((&text[..open], &text[open + 1..text.len() - 1]))
}

fn split_arguments(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut depth = 0isize;
    let mut in_string = false;
    for (index, ch) in text.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' | '[' | '{' if !in_string => depth += 1,
            ')' | ']' | '}' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                let part = text[start..index].trim();
                if !part.is_empty() {
                    parts.push(part);
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    let part = text[start..].trim();
    if !part.is_empty() {
        parts.push(part);
    }
    parts
}

fn split_word_operator<'a>(text: &'a str, operator: &str) -> Option<(&'a str, &'a str)> {
    let pattern = format!(" {operator} ");
    let index = find_top_level_pattern(text, &pattern, Search::Leftmost)?;
    Some((&text[..index], &text[index + pattern.len()..]))
}

fn split_top_level_operator<'a>(
    text: &'a str,
    operators: &[&'a str],
) -> Option<(&'a str, &'a str, &'a str)> {
    for operator in operators {
        let pattern = format!(" {operator} ");
        if let Some(index) = find_top_level_pattern(text, &pattern, Search::Rightmost) {
            return Some((
                text[..index].trim(),
                *operator,
                text[index + pattern.len()..].trim(),
            ));
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
enum Search {
    Leftmost,
    Rightmost,
}

fn find_top_level_pattern(text: &str, pattern: &str, search: Search) -> Option<usize> {
    let mut found = None;
    let mut depth = 0isize;
    let mut in_string = false;
    for (index, ch) in text.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' | '[' | '{' if !in_string => depth += 1,
            ')' | ']' | '}' if !in_string => depth -= 1,
            _ => {}
        }
        if !in_string && depth == 0 && text[index..].starts_with(pattern) {
            match search {
                Search::Leftmost => return Some(index),
                Search::Rightmost => found = Some(index),
            }
        }
    }
    found
}

fn find_top_level_char(text: &str, needle: char) -> Option<usize> {
    let mut depth = 0isize;
    let mut in_string = false;
    for (index, ch) in text.char_indices() {
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if ch == needle && depth == 0 {
            return Some(index);
        }
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {}
        }
    }
    None
}

fn trim_outer_parens(mut text: &str) -> &str {
    loop {
        let trimmed = text.trim();
        if trimmed.starts_with('(') && trimmed.ends_with(')') && outer_parens_wrap(trimmed) {
            text = &trimmed[1..trimmed.len() - 1];
        } else {
            return trimmed;
        }
    }
}

fn outer_parens_wrap(text: &str) -> bool {
    let mut depth = 0isize;
    let mut in_string = false;
    for (index, ch) in text.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' if !in_string => depth += 1,
            ')' if !in_string => {
                depth -= 1;
                if depth == 0 && index != text.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

#[cfg(test)]
mod tests {
    use crate::ast::Program;
    use crate::check;
    use crate::diagnostic::{DiagnosticCode, Severity};
    use crate::parser;

    use super::{RunOutcome, run_program};

    #[test]
    fn runs_add_program() {
        let program = fixture_program(
            "examples/core/add.hum",
            include_str!("../examples/core/add.hum"),
        );
        let report = run_program(&program, Some("add"), &["2".to_string(), "3".to_string()]);
        assert_eq!(report.outcome, RunOutcome::Success("5".to_string()));
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn structural_app_runs_without_displaying_unit() {
        let program = fixture_program(
            "examples/probes/pure_app_entry.hum",
            include_str!("../examples/probes/pure_app_entry.hum"),
        );
        let report = run_program(&program, None, &["hello".to_string()]);
        assert_eq!(report.outcome, RunOutcome::AppSuccess);
        assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    }

    #[test]
    fn structural_app_keeps_typed_failure_distinct_from_legacy_display() {
        let program = fixture_program(
            "examples/probes/fallible_app_entry.hum",
            include_str!("../examples/probes/fallible_app_entry.hum"),
        );
        let report = run_program(&program, None, &["true".to_string()]);
        let RunOutcome::AppFailure(chain) = report.outcome else {
            panic!("expected app failure");
        };
        assert!(chain.contains("failure: LaunchError.requested"));
        assert!(chain.contains("originated at examples/probes/fallible_app_entry.hum:23:9"));
    }

    #[test]
    fn app_mode_rejects_external_same_name_but_entry_remains_direct_probe() {
        let program = fixture_program(
            "fixtures/app_entry/session_x_external_same_name_fail.hum",
            include_str!("../fixtures/app_entry/session_x_external_same_name_fail.hum"),
        );
        let app_report = run_program(&program, None, &[]);
        assert!(matches!(
            app_report.outcome,
            RunOutcome::Trap(ref message) if message == "H0614 app start task is not a child"
        ));
        assert_eq!(app_report.diagnostics.len(), 1);
        assert_eq!(
            app_report.diagnostics[0].code,
            DiagnosticCode::APP_START_NOT_CHILD
        );

        let direct_report = run_program(&program, Some("run_tool"), &[]);
        assert_eq!(direct_report.outcome, RunOutcome::Success("()".to_string()));
        assert!(direct_report.diagnostics.is_empty());
    }

    #[test]
    fn divide_zero_fails_needs_with_caller_blame() {
        let program = fixture_program(
            "examples/core/divide.hum",
            include_str!("../examples/core/divide.hum"),
        );
        let report = run_program(
            &program,
            Some("divide"),
            &["10".to_string(), "0".to_string()],
        );
        assert_eq!(report.outcome, RunOutcome::ContractViolation);
        assert_eq!(
            report.diagnostics.len(),
            1,
            "diagnostics: {:#?}",
            report.diagnostics
        );
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::NEEDS_CONTRACT_VIOLATION);
        assert_eq!(diagnostic.severity, Severity::Error);
        let rendered = diagnostic.render();
        assert!(
            rendered.contains("examples/core/divide.hum:12:"),
            "{rendered}"
        );
        assert!(
            rendered.contains("caller did not satisfy needs: b != 0"),
            "{rendered}"
        );
    }

    #[test]
    fn unchecked_task_can_return_typed_failure() {
        let source = r#"module tests.typed_failure

type MathError {
  code: Text
}

task fail_now() -> Result Int, MathError {
  fails when:
    divisor is zero

  does:
    fail MathError.divide_by_zero
}
"#;
        let program = fixture_program("typed_failure.hum", source);
        let report = run_program(&program, Some("fail_now"), &[]);
        assert_eq!(
            report.outcome,
            RunOutcome::Failure(
                "failure: MathError.divide_by_zero\n  originated at typed_failure.hum:12:5"
                    .to_string()
            )
        );
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn causal_failure_chain_keeps_outer_to_root_sites() {
        let program = fixture_program(
            "examples/probes/causal_failures.hum",
            include_str!("../examples/probes/causal_failures.hum"),
        );
        let report = run_program(&program, Some("outer_value"), &["true".to_string()]);
        let RunOutcome::Failure(chain) = report.outcome else {
            panic!("expected typed failure, got {:?}", report.outcome);
        };
        let expected = [
            "failure: OuterError.context",
            ":74:5 while calling `middle_value`",
            "caused by: MiddleError.context",
            ":59:5 while calling `root_value`",
            "caused by: RootError.origin",
            "originated at examples/probes/causal_failures.hum:27:7",
        ];
        for text in expected {
            assert!(chain.contains(text), "missing {text}: {chain}");
        }
    }

    #[test]
    fn causal_failure_renders_direct_origin_and_same_root_propagation_once() {
        let program = fixture_program(
            "examples/probes/causal_failures.hum",
            include_str!("../examples/probes/causal_failures.hum"),
        );

        let direct = run_program(&program, Some("root_value"), &["true".to_string()]);
        let RunOutcome::Failure(direct_chain) = direct.outcome else {
            panic!("expected direct typed failure, got {:?}", direct.outcome);
        };
        assert!(direct_chain.starts_with("failure: RootError.origin"));
        assert_eq!(direct_chain.matches("originated at").count(), 1);
        assert!(direct_chain.contains("causal_failures.hum:27:7"));

        let propagated = run_program(&program, Some("same_root"), &["true".to_string()]);
        let RunOutcome::Failure(propagated_chain) = propagated.outcome else {
            panic!(
                "expected propagated typed failure, got {:?}",
                propagated.outcome
            );
        };
        assert!(propagated_chain.starts_with("failure: RootError.origin"));
        assert_eq!(propagated_chain.matches("propagated at").count(), 1);
        assert_eq!(propagated_chain.matches("originated at").count(), 1);
        assert!(propagated_chain.contains("while calling `root_value`"));
    }

    #[test]
    fn implicit_fallible_call_rejects_with_shared_diagnostic() {
        let program = fixture_program(
            "fixtures/full_type_check/session_w_implicit_fallible_call_fail.hum",
            include_str!("../fixtures/full_type_check/session_w_implicit_fallible_call_fail.hum"),
        );
        let report = run_program(&program, Some("caller"), &[]);
        assert!(matches!(report.outcome, RunOutcome::Trap(ref text) if text.contains("H0901")));
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY
        );
        assert!(
            report.diagnostics[0]
                .render()
                .contains("callee declared at")
        );
    }

    #[test]
    fn count_completed_counts_done_records_with_prose_warning() {
        let program = fixture_program(
            "examples/core/count_completed.hum",
            include_str!("../examples/core/count_completed.hum"),
        );
        let report = run_program(
            &program,
            Some("count_completed"),
            &["[{done:true},{done:false},{done:true}]".to_string()],
        );
        assert_eq!(report.outcome, RunOutcome::Success("2".to_string()));
        assert_eq!(
            report.diagnostics.len(),
            1,
            "diagnostics: {:#?}",
            report.diagnostics
        );
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::UNCHECKED_PROSE_CONTRACT);
        assert_eq!(diagnostic.severity, Severity::Warning);
        let rendered = diagnostic.render();
        assert!(
            rendered.contains(
                "unchecked prose ensures contract: result is at most the number of items"
            ),
            "{rendered}"
        );
    }

    #[test]
    fn wrong_add_fixture_fails_ensures_with_task_blame() {
        let program = fixture_program(
            "fixtures/run/wrong_add_contract.hum",
            include_str!("../fixtures/run/wrong_add_contract.hum"),
        );
        let report = run_program(&program, Some("add"), &["2".to_string(), "3".to_string()]);
        assert_eq!(report.outcome, RunOutcome::ContractViolation);
        assert_eq!(
            report.diagnostics.len(),
            1,
            "diagnostics: {:#?}",
            report.diagnostics
        );
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::ENSURES_CONTRACT_VIOLATION);
        assert_eq!(diagnostic.severity, Severity::Error);
        let rendered = diagnostic.render();
        assert!(
            rendered.contains("fixtures/run/wrong_add_contract.hum:8:"),
            "{rendered}"
        );
        assert!(
            rendered.contains("task `add` did not satisfy ensures: result == a + b"),
            "{rendered}"
        );
    }

    #[test]
    fn wrong_swap_fixture_fails_old_ensures_with_task_blame() {
        let program = fixture_program(
            "fixtures/run/session_t_wrong_swap_contract.hum",
            include_str!("../fixtures/run/session_t_wrong_swap_contract.hum"),
        );
        let report = run_program(&program, Some("wrong_swap_demo"), &[]);
        assert_eq!(report.outcome, RunOutcome::ContractViolation);
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::ENSURES_CONTRACT_VIOLATION);
        let rendered = diagnostic.render();
        assert!(
            rendered.contains("task `swap_xy` did not satisfy ensures: result.x == old(point.y)"),
            "{rendered}"
        );
    }

    #[test]
    fn complete_item_preservation_is_checked_not_prose() {
        let program = fixture_program(
            "fixtures/run/session_o_complete_item_field_place.hum",
            include_str!("../fixtures/run/session_o_complete_item_field_place.hum"),
        );
        let report = run_program(&program, Some("complete_item_demo"), &[]);
        assert_eq!(
            report.outcome,
            RunOutcome::Success("{done: true, title: hum}".to_string())
        );
        assert!(
            report.diagnostics.is_empty(),
            "preservation must be a checked old() contract with no prose warning: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn builder_list_len_contract_checks_and_content_stays_prose() {
        let program = fixture_program(
            "examples/probes/list_builder.hum",
            include_str!("../examples/probes/list_builder.hum"),
        );
        let report = run_program(&program, Some("builder_demo"), &[]);
        assert_eq!(
            report.outcome,
            RunOutcome::Success("[parse, check, run]".to_string())
        );
        assert_eq!(
            report.diagnostics.len(),
            1,
            "only the content claim stays prose: {:#?}",
            report.diagnostics
        );
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::UNCHECKED_PROSE_CONTRACT
        );
    }

    #[test]
    fn spaced_old_falls_to_honest_prose_never_traps() {
        let source = r#"module tests.spaced_old

task echo_amount(amount: UInt) -> UInt {
  why:
    pin the canonical-prefix rule for old

  ensures:
    result == old (amount)

  does:
    return amount
}
"#;
        let program = fixture_program("tests/spaced_old.hum", source);
        let report = run_program(&program, Some("echo_amount"), &["7".to_string()]);
        assert_eq!(report.outcome, RunOutcome::Success("7".to_string()));
        assert_eq!(
            report.diagnostics.len(),
            1,
            "spaced old must warn as prose, not trap: {:#?}",
            report.diagnostics
        );
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::UNCHECKED_PROSE_CONTRACT
        );
    }

    #[test]
    fn spaced_list_len_falls_to_honest_prose() {
        let source = r#"module tests.spaced_list_len

task make_items() -> List Text {
  why:
    pin the canonical-prefix rule for list_len

  ensures:
    list_len (result) == 2

  does:
    change items: List Text = []
    let added_a: Unit = list_append(change items, "a")
    let added_b: Unit = list_append(change items, "b")
    return items
}
"#;
        let program = fixture_program("tests/spaced_list_len.hum", source);
        let report = run_program(&program, Some("make_items"), &[]);
        assert_eq!(report.diagnostics.len(), 1, "{:#?}", report.diagnostics);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::UNCHECKED_PROSE_CONTRACT
        );
    }

    #[test]
    fn list_len_is_reachable_from_task_bodies() {
        let source = r#"module tests.list_len_body

task count_items() -> Int {
  why:
    pin list_len as a documented body-reachable length read

  does:
    change items: List Text = []
    let added_a: Unit = list_append(change items, "a")
    let added_b: Unit = list_append(change items, "b")
    return list_len(items)
}
"#;
        let program = fixture_program("tests/list_len_body.hum", source);
        let report = run_program(&program, Some("count_items"), &[]);
        assert_eq!(report.outcome, RunOutcome::Success("2".to_string()));
    }

    #[test]
    fn old_in_task_body_traps_with_clear_message() {
        let source = r#"module tests.old_in_body

task echo_amount(amount: UInt) -> UInt {
  why:
    pin that old() stays contract-only vocabulary

  does:
    return old(amount)
}
"#;
        let program = fixture_program("tests/old_in_body.hum", source);
        let report = run_program(&program, Some("echo_amount"), &["7".to_string()]);
        let RunOutcome::Trap(message) = &report.outcome else {
            panic!("old() in a body must trap, got {:#?}", report.outcome);
        };
        assert!(
            message.contains("available only in `ensures:`"),
            "{message}"
        );
    }

    #[test]
    fn old_in_needs_stays_honest_prose() {
        let program = fixture_program(
            "fixtures/run/session_t_old_in_needs_prose.hum",
            include_str!("../fixtures/run/session_t_old_in_needs_prose.hum"),
        );
        let report = run_program(&program, Some("old_in_needs_demo"), &[]);
        assert_eq!(report.outcome, RunOutcome::Success("7".to_string()));
        assert_eq!(report.diagnostics.len(), 1);
        let rendered = report.diagnostics[0].render();
        assert!(
            rendered.contains("unchecked prose needs contract: amount == old(amount)"),
            "{rendered}"
        );
    }

    #[test]
    fn task_call_expression_runs_called_task() {
        let source = r#"module tests.calls

task add(a: Int, b: Int) -> Int {
  does:
    return a + b
}

task add_one(value: Int) -> Int {
  does:
    return add(value, 1)
}
"#;
        let program = fixture_program("calls.hum", source);
        let report = run_program(&program, Some("add_one"), &["4".to_string()]);
        assert_eq!(report.outcome, RunOutcome::Success("5".to_string()));
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn integer_overflow_traps() {
        let source = r#"module tests.overflow

task overflow(value: Int) -> Int {
  does:
    return value + 1
}
"#;
        let program = fixture_program("overflow.hum", source);
        let report = run_program(&program, Some("overflow"), &[i64::MAX.to_string()]);
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("integer overflow while evaluating `value + 1`".to_string())
        );
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn division_by_zero_traps() {
        let source = r#"module tests.divide_trap

task unchecked_divide(a: Int, b: Int) -> Int {
  does:
    return a / b
}
"#;
        let program = fixture_program("divide_trap.hum", source);
        let report = run_program(
            &program,
            Some("unchecked_divide"),
            &["10".to_string(), "0".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("division by zero while evaluating `a / b`".to_string())
        );
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn writable_field_alias_reads_and_writes_through_to_the_owner() {
        let program = fixture_program(
            "examples/probes/writable_field_aliases.hum",
            include_str!("../examples/probes/writable_field_aliases.hum"),
        );
        let report = run_program(
            &program,
            Some("swap_xy_with_aliases"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Success("{x: 2, y: 1}".to_string())
        );
        assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    }

    #[test]
    fn writable_field_alias_overlap_traps_with_shared_h0808_blame() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_program8_overlap_write_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_program8_overlap_write_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("overlapping_write"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0808 writable alias overlap".to_string())
        );
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::WRITABLE_ALIAS_OVERLAP
        );
        let rendered = report.diagnostics[0].render();
        assert!(rendered.contains("alias_to_x"), "{rendered}");
        assert!(rendered.contains("point.x"), "{rendered}");
        assert!(rendered.contains(":13:5"), "{rendered}");
        assert!(rendered.contains(":14:5"), "{rendered}");
        assert!(rendered.contains(":15:5"), "{rendered}");
        assert!(
            rendered.contains("definitely distinct direct field"),
            "{rendered}"
        );
    }

    #[test]
    fn writable_field_alias_escape_traps_with_h0809() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_escape_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_escape_fail.hum"),
        );
        let report = run_program(&program, Some("escaped_alias"), &["{x:1,y:2}".to_string()]);
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS
        );
    }

    #[test]
    fn writable_alias_to_alias_traps_with_shared_h0809_fact() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_to_alias_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_to_alias_fail.hum"),
        );
        let report = run_program(&program, Some("alias_to_alias"), &["{x:1,y:2}".to_string()]);
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert_eq!(report.diagnostics.len(), 1);
        let rendered = report.diagnostics[0].render();
        assert!(rendered.contains("writable_alias_to_alias_binding_v0"));
        assert!(rendered.contains("writable alias `first`"));
    }

    #[test]
    fn writable_field_alias_requires_visible_mutation_authority() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_borrowed_owner_alias_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_borrowed_owner_alias_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("borrowed_owner_alias"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0802 borrowed parameter written".to_string())
        );
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::BORROW_PARAMETER_MUTATION
        );
    }

    #[test]
    fn writable_field_alias_permission_wrapper_traps_with_h0809() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_permission_wrapper_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_permission_wrapper_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("permission_wrapped_alias"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert!(
            report.diagnostics[0]
                .render()
                .contains("writable_alias_permission_wrapper_v0")
        );
    }

    #[test]
    fn writable_field_alias_cannot_rebind_its_owner() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("alias_rebinds_owner"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert!(
            report.diagnostics[0]
                .render()
                .contains("writable_alias_rebinds_its_owner_v0")
        );
    }

    #[test]
    fn writable_field_alias_cannot_shadow_an_existing_name() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_name_collision_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_name_collision_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("alias_name_collision"),
            &["{x:1,y:2}".to_string(), "7".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert!(
            report.diagnostics[0]
                .render()
                .contains("writable_alias_binding_rebinding_v0")
        );
    }

    #[test]
    fn writable_field_alias_cannot_shadow_a_declared_permission() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_declared_name_collision_fail.hum",
            include_str!(
                "../fixtures/ownership_check/session_v_alias_declared_name_collision_fail.hum"
            ),
        );
        let report = run_program(
            &program,
            Some("alias_declared_name_collision"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert!(
            report.diagnostics[0]
                .render()
                .contains("writable_alias_binding_rebinding_v0")
        );
    }

    #[test]
    fn writable_alias_authority_precedes_overlap_consistently() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_borrowed_owner_overlap_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_borrowed_owner_overlap_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("borrowed_owner_overlap"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0802 borrowed parameter written".to_string())
        );
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::BORROW_PARAMETER_MUTATION
        );
    }

    fn fixture_program(path: &str, source: &str) -> Program {
        let parsed = parser::parse_source(path, source);
        let mut diagnostics = parsed.diagnostics;
        diagnostics.extend(check::check_file(&parsed.file));
        let errors = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "fixture diagnostics: {errors:#?}");
        Program {
            files: vec![parsed.file],
        }
    }
}
