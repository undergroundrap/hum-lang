use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{Item, ParamPermission, Program, SectionLine, Task};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Span};
use crate::field_place;
use crate::graph::is_meaningful_line_text;
use crate::return_dependency;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    Success(String),
    Failure(String),
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
    Failure(Value),
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
    Fail(Value),
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
    moved_at: Option<Span>,
    moved_by: Option<String>,
    linear: bool,
}

impl RuntimeBinding {
    fn parameter(value: Value, permission: ParamPermission) -> Self {
        Self {
            value,
            permission: RuntimePermission::from(permission),
            moved_at: None,
            moved_by: None,
            linear: false,
        }
    }

    fn local(value: Value, linear: bool) -> Self {
        Self {
            value,
            permission: RuntimePermission::Local,
            moved_at: None,
            moved_by: None,
            linear,
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

pub fn run_program(program: &Program, entry: Option<&str>, raw_args: &[String]) -> RunReport {
    let interpreter = Interpreter {
        program,
        diagnostics: RefCell::new(Vec::new()),
    };
    let outcome = match interpreter.run(entry, raw_args) {
        Ok(TaskResult::Returned(value)) => RunOutcome::Success(display_value(&value)),
        Ok(TaskResult::Failed(value)) => RunOutcome::Failure(display_value(&value)),
        Ok(TaskResult::ContractViolation) => RunOutcome::ContractViolation,
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
    diagnostics: RefCell<Vec<Diagnostic>>,
}

enum TaskResult {
    Returned(Value),
    Failed(Value),
    ContractViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContractKind {
    Needs,
    Ensures,
}

impl<'a> Interpreter<'a> {
    fn run(&self, entry: Option<&str>, raw_args: &[String]) -> Result<TaskResult, String> {
        let task = self.entry_task(entry)?;
        let args = self.parse_args(task, raw_args)?;
        self.execute_task(task, args)
    }

    fn entry_task(&self, entry: Option<&str>) -> Result<&'a Task, String> {
        if let Some(name) = entry {
            return self
                .find_task(name)
                .ok_or_else(|| format!("entry task `{name}` was not found"));
        }

        let mut tasks = Vec::new();
        for file in &self.program.files {
            collect_tasks(&file.items, &mut tasks);
        }
        match tasks.as_slice() {
            [task] => Ok(*task),
            [] => Err("no task is available to run".to_string()),
            _ => Err("multiple tasks are available; pass `--entry <task>`".to_string()),
        }
    }

    fn find_task(&self, name: &str) -> Option<&'a Task> {
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

        let Some(does) = task.section("does") else {
            return Err(format!("task `{}` has no `does:` section", task.name));
        };
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

            if !validate_predicate_v0(text, &allowed_names) {
                self.diagnostics.borrow_mut().push(
                    Diagnostic::warning(
                        DiagnosticCode::UNCHECKED_PROSE_CONTRACT,
                        format!("unchecked prose {} contract: {text}", kind.section_name()),
                        Some(line.span.clone()),
                    )
                    .with_help(
                        "Predicate v0 checks one comparison such as `result == a + b`; prose remains visible but unchecked.",
                    ),
                );
                continue;
            }

            let value = match self.eval_expr(text, env, &line.span, &task.name)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => {
                    return Err(format!(
                        "contract predicate `{text}` produced failure {}",
                        display_value(&value)
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

                let name = name.trim();
                let previous = env.get(name).cloned();
                for value in values {
                    env.insert(name.to_string(), RuntimeBinding::local(value, false));
                    let flow = self.eval_block(lines, index + 1, close, env, task_name)?;
                    if flow != Flow::Continue {
                        restore_binding(env, name, previous);
                        return Ok(flow);
                    }
                }
                restore_binding(env, name, previous);
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
                return match self.eval_expr(expr, env, &line.span, task_name)? {
                    Evaluated::Value(value) | Evaluated::Failure(value) => {
                        self.ensure_linear_closed_on_exit(env, task_name, "fail", &line.span)?;
                        Ok(Flow::Fail(value))
                    }
                    Evaluated::ContractViolation => Ok(Flow::ContractViolation),
                };
            }

            if let Some(rest) = strip_keyword(text, "change") {
                if let Some(flow) = self.eval_binding(rest, env, &line.span, task_name)? {
                    return Ok(flow);
                }
                index += 1;
                continue;
            }

            if let Some(rest) = strip_keyword(text, "let") {
                if let Some(flow) = self.eval_binding(rest, env, &line.span, task_name)? {
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
    ) -> Result<Option<Flow>, String> {
        let (left, expr) = rest
            .split_once('=')
            .ok_or_else(|| format!("binding `{rest}` is missing an initializer"))?;
        let annotation = left.split_once(':').map(|(_name, ty)| ty.trim());
        let name = left.split_once(':').map_or(left, |(name, _ty)| name).trim();
        if name.is_empty() {
            return Err(format!("binding `{rest}` is missing a name"));
        }
        let linear = annotation.is_some_and(is_linear_resource_type);
        match self.eval_expr(expr.trim(), env, span, task_name)? {
            Evaluated::Value(value) => {
                env.insert(name.to_string(), RuntimeBinding::local(value, linear));
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
        let root = place_root(place);
        self.ensure_can_set(env, place, &root, span, task_name)?;
        match self.eval_expr(expr.trim(), env, span, task_name)? {
            Evaluated::Value(value) => {
                self.write_place(env, place, value)?;
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
            if return_dependency::is_closed_view_deriving_operation(callee.trim()) {
                return self.eval_slice_until(args, env, span, task_name);
            }
            let Some(task) = self.find_task(callee.trim()) else {
                return Err(format!("task `{}` was not found", callee.trim()));
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
        Ok(binding.value.clone())
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

fn validate_predicate_v0(text: &str, allowed_names: &BTreeSet<String>) -> bool {
    let text = trim_outer_parens(text.trim());
    let Some((left, _op, right)) =
        split_top_level_operator(text, &["==", "!=", "<=", ">=", "<", ">"])
    else {
        return false;
    };

    split_top_level_operator(left, &["==", "!=", "<=", ">=", "<", ">"]).is_none()
        && split_top_level_operator(right, &["==", "!=", "<=", ">=", "<", ">"]).is_none()
        && validate_contract_operand(left, allowed_names)
        && validate_contract_operand(right, allowed_names)
}

fn validate_contract_operand(text: &str, allowed_names: &BTreeSet<String>) -> bool {
    let text = trim_outer_parens(text.trim());
    if text.is_empty()
        || split_word_operator(text, "and").is_some()
        || split_word_operator(text, "or").is_some()
    {
        return false;
    }
    if let Some((left, _op, right)) = split_top_level_operator(text, &["+", "-"]) {
        return validate_contract_operand(left, allowed_names)
            && validate_contract_operand(right, allowed_names);
    }
    if let Some((left, _op, right)) = split_top_level_operator(text, &["*", "/"]) {
        return validate_contract_operand(left, allowed_names)
            && validate_contract_operand(right, allowed_names);
    }
    text == "true"
        || text == "false"
        || parse_int_literal(text).is_ok()
        || allowed_names.contains(text)
        || field_place::split_field_place(text)
            .is_some_and(|(root, _field)| allowed_names.contains(root))
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
  does:
    fail MathError.divide_by_zero
}
"#;
        let program = fixture_program("typed_failure.hum", source);
        let report = run_program(&program, Some("fail_now"), &[]);
        assert_eq!(
            report.outcome,
            RunOutcome::Failure("MathError.divide_by_zero".to_string())
        );
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
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
