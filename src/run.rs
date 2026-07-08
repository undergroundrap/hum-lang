use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{Item, Program, SectionLine, Task};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::graph::is_meaningful_line_text;

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
    Return(Value),
    Fail(Value),
    ContractViolation,
}

#[derive(Debug, Clone)]
struct ExecLine {
    text: String,
    location: String,
}

type Env = BTreeMap<String, Value>;

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
            env.insert(param.name.clone(), value);
        }

        if !self.evaluate_contract_section(task, "needs", ContractKind::Needs, &mut env)? {
            return Ok(TaskResult::ContractViolation);
        }

        let Some(does) = task.section("does") else {
            return Err(format!("task `{}` has no `does:` section", task.name));
        };
        let lines = executable_lines(&does.lines);
        match self.eval_block(&lines, 0, lines.len(), &mut env)? {
            Flow::Return(value) => self.finish_success(task, value, &env),
            Flow::Fail(value) => Ok(TaskResult::Failed(value)),
            Flow::Continue => self.finish_success(task, Value::Unit, &env),
            Flow::ContractViolation => Ok(TaskResult::ContractViolation),
        }
    }

    fn finish_success(&self, task: &Task, value: Value, env: &Env) -> Result<TaskResult, String> {
        let mut exit_env = env.clone();
        exit_env.insert("result".to_string(), value.clone());
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

            let value = match self.eval_expr(text, env)? {
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
                match self.eval_expr(condition, env)? {
                    Evaluated::Value(value) if as_bool(&value)? => {
                        let flow = self.eval_block(lines, index + 1, close, env)?;
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
                let collection = match self.eval_expr(collection_expr.trim(), env)? {
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
                    env.insert(name.to_string(), value);
                    let flow = self.eval_block(lines, index + 1, close, env)?;
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
                return match self.eval_expr(expr, env)? {
                    Evaluated::Value(value) => Ok(Flow::Return(value)),
                    Evaluated::Failure(value) => Ok(Flow::Fail(value)),
                    Evaluated::ContractViolation => Ok(Flow::ContractViolation),
                };
            }

            if let Some(expr) = strip_keyword(text, "fail") {
                return match self.eval_expr(expr, env)? {
                    Evaluated::Value(value) | Evaluated::Failure(value) => Ok(Flow::Fail(value)),
                    Evaluated::ContractViolation => Ok(Flow::ContractViolation),
                };
            }

            if let Some(rest) = strip_keyword(text, "change") {
                if let Some(flow) = self.eval_binding(rest, env)? {
                    return Ok(flow);
                }
                index += 1;
                continue;
            }

            if let Some(rest) = strip_keyword(text, "let") {
                if let Some(flow) = self.eval_binding(rest, env)? {
                    return Ok(flow);
                }
                index += 1;
                continue;
            }

            if let Some(rest) = strip_keyword(text, "set") {
                if let Some(flow) = self.eval_set(rest, env)? {
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

    fn eval_binding(&self, rest: &str, env: &mut Env) -> Result<Option<Flow>, String> {
        let (left, expr) = rest
            .split_once('=')
            .ok_or_else(|| format!("binding `{rest}` is missing an initializer"))?;
        let name = left.split_once(':').map_or(left, |(name, _ty)| name).trim();
        if name.is_empty() {
            return Err(format!("binding `{rest}` is missing a name"));
        }
        match self.eval_expr(expr.trim(), env)? {
            Evaluated::Value(value) => {
                env.insert(name.to_string(), value);
                Ok(None)
            }
            Evaluated::Failure(value) => Ok(Some(Flow::Fail(value))),
            Evaluated::ContractViolation => Ok(Some(Flow::ContractViolation)),
        }
    }

    fn eval_set(&self, rest: &str, env: &mut Env) -> Result<Option<Flow>, String> {
        let (place, expr) = rest
            .split_once('=')
            .ok_or_else(|| format!("set `{rest}` is missing `=`"))?;
        let place = place.trim();
        if !env.contains_key(place) {
            return Err(format!("cannot set unknown place `{place}`"));
        }
        match self.eval_expr(expr.trim(), env)? {
            Evaluated::Value(value) => {
                env.insert(place.to_string(), value);
                Ok(None)
            }
            Evaluated::Failure(value) => Ok(Some(Flow::Fail(value))),
            Evaluated::ContractViolation => Ok(Some(Flow::ContractViolation)),
        }
    }

    fn eval_expr(&self, text: &str, env: &mut Env) -> Result<Evaluated, String> {
        let text = trim_outer_parens(text.trim());
        if text.is_empty() {
            return Ok(Evaluated::Value(Value::Unit));
        }

        if let Some((left, right)) = split_word_operator(text, "or") {
            let left = match self.eval_expr(left, env)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
            };
            if as_bool(&left)? {
                return Ok(Evaluated::Value(Value::Bool(true)));
            }
            let right = match self.eval_expr(right, env)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
            };
            return Ok(Evaluated::Value(Value::Bool(as_bool(&right)?)));
        }

        if let Some((left, right)) = split_word_operator(text, "and") {
            let left = match self.eval_expr(left, env)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
            };
            if !as_bool(&left)? {
                return Ok(Evaluated::Value(Value::Bool(false)));
            }
            let right = match self.eval_expr(right, env)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
            };
            return Ok(Evaluated::Value(Value::Bool(as_bool(&right)?)));
        }

        if let Some((left, op, right)) =
            split_top_level_operator(text, &["==", "!=", "<=", ">=", "<", ">"])
        {
            return self.eval_comparison(text, left, op, right, env);
        }

        if let Some((left, op, right)) = split_top_level_operator(text, &["+", "-"]) {
            return self.eval_integer_binary(text, left, op, right, env);
        }

        if let Some((left, op, right)) = split_top_level_operator(text, &["*", "/"]) {
            return self.eval_integer_binary(text, left, op, right, env);
        }

        self.eval_primary(text, env)
    }

    fn eval_comparison(
        &self,
        source: &str,
        left: &str,
        op: &str,
        right: &str,
        env: &mut Env,
    ) -> Result<Evaluated, String> {
        let left = match self.eval_expr(left, env)? {
            Evaluated::Value(value) => value,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let right = match self.eval_expr(right, env)? {
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
        left: &str,
        op: &str,
        right: &str,
        env: &mut Env,
    ) -> Result<Evaluated, String> {
        let left = match self.eval_expr(left, env)? {
            Evaluated::Value(value) => as_int(&value)?,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let right = match self.eval_expr(right, env)? {
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

    fn eval_primary(&self, text: &str, env: &mut Env) -> Result<Evaluated, String> {
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
                match self.eval_expr(item, env)? {
                    Evaluated::Value(value) => values.push(value),
                    Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                    Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
                }
            }
            return Ok(Evaluated::Value(Value::List(values)));
        }
        if let Some((callee, args)) = split_call(text) {
            let mut values = Vec::new();
            for arg in split_arguments(args) {
                match self.eval_expr(arg, env)? {
                    Evaluated::Value(value) => values.push(value),
                    Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
                    Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
                }
            }
            let Some(task) = self.find_task(callee.trim()) else {
                return Err(format!("task `{}` was not found", callee.trim()));
            };
            return match self.execute_task(task, values)? {
                TaskResult::Returned(value) => Ok(Evaluated::Value(value)),
                TaskResult::Failed(value) => Ok(Evaluated::Failure(value)),
                TaskResult::ContractViolation => Ok(Evaluated::ContractViolation),
            };
        }
        if let Some(value) = env.get(text) {
            return Ok(Evaluated::Value(value.clone()));
        }
        if let Some((base, field)) = text.split_once('.') {
            if let Some(value) = env.get(base) {
                let Value::Record(fields) = value else {
                    return Err(format!("`{base}` is not a record"));
                };
                return fields
                    .get(field)
                    .cloned()
                    .map(Evaluated::Value)
                    .ok_or_else(|| format!("record `{base}` has no field `{field}`"));
            }
            if base
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
            {
                return Ok(Evaluated::Value(Value::Variant(text.to_string())));
            }
        }
        Err(format!("unknown expression `{text}`"))
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
}

fn executable_lines(lines: &[SectionLine]) -> Vec<ExecLine> {
    lines
        .iter()
        .filter_map(|line| {
            let text = line.text.trim();
            is_meaningful_line_text(text).then(|| ExecLine {
                text: text.to_string(),
                location: format!("{}:{}:{}", line.span.file, line.span.line, line.span.column),
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

fn restore_binding(env: &mut Env, name: &str, previous: Option<Value>) {
    if let Some(value) = previous {
        env.insert(name.to_string(), value);
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
