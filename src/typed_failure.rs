use std::collections::BTreeMap;

use crate::ast::{Item, Program, Task};
use crate::core_body::BodyStatement;
use crate::diagnostic::{DiagnosticCode, Span};
use crate::graph::{hollow_contract_reason, is_meaningful_line_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskFailureSignature {
    pub name: String,
    pub success_type: Option<String>,
    pub error_root: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FailureCatalog {
    tasks: BTreeMap<String, TaskFailureSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DirectCall {
    pub callee: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FailureVariant {
    pub root: String,
    pub variant: String,
}

impl FailureVariant {
    pub fn identity(&self) -> String {
        format!("{}.{}", self.root, self.variant)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TryExpression {
    pub call: DirectCall,
    pub wrapper: Option<FailureVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FailureFact {
    pub index: usize,
    pub status: &'static str,
    pub reason: Option<&'static str>,
    pub diagnostic_code: Option<DiagnosticCode>,
    pub form: &'static str,
    pub callee: Option<String>,
    pub callee_result_root: Option<String>,
    pub caller_result_root: Option<String>,
    pub wrapper_root: Option<String>,
    pub call_span: Span,
    pub callee_span: Option<Span>,
    pub caller_span: Span,
    pub help: Option<String>,
    pub success_type: Option<String>,
    pub try_expression: Option<TryExpression>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TaskFailureAnalysis {
    pub facts: BTreeMap<usize, FailureFact>,
}

impl FailureCatalog {
    pub fn from_program(program: &Program) -> Self {
        let mut catalog = Self::default();
        for file in &program.files {
            collect_signatures(&file.items, &mut catalog.tasks);
        }
        catalog
    }

    pub fn task(&self, name: &str) -> Option<&TaskFailureSignature> {
        self.tasks.get(name)
    }
}

pub(crate) fn analyze_task(
    task: &Task,
    statements: &[BodyStatement],
    catalog: &FailureCatalog,
) -> TaskFailureAnalysis {
    let caller_root = task.result.as_deref().and_then(result_error_root);
    let has_failure_declaration = task.section("fails when").is_some_and(|section| {
        section
            .lines
            .iter()
            .any(|line| is_meaningful_failure_declaration(&line.text))
    });
    let mut facts = BTreeMap::new();

    for (index, statement) in statements.iter().enumerate() {
        let expression = statement_expression(statement);
        if statement.kind == "fail"
            && let Some(variant) = expression.and_then(parse_failure_variant)
        {
            let fact = direct_fail_fact(
                task,
                statement,
                index,
                caller_root.as_deref(),
                &variant,
                has_failure_declaration,
            );
            facts.insert(index, fact);
            continue;
        }

        let Some(expression) = expression else {
            continue;
        };
        if contains_keyword_token(expression, "try") {
            let fact = analyze_try_expression(
                task,
                statement,
                index,
                expression,
                caller_root.as_deref(),
                catalog,
                has_failure_declaration,
            );
            facts.insert(index, fact);
            continue;
        }

        if let Some((call, callee, callee_root)) = calls_in_expression(expression)
            .into_iter()
            .find_map(|call| {
                let callee = catalog.task(&call.callee)?;
                let callee_root = callee.error_root.as_deref()?;
                Some((call, callee, callee_root))
            })
        {
            facts.insert(
                index,
                issue_fact(
                    task,
                    statement,
                    index,
                    "implicit_fallible_call",
                    "fallible_call_requires_try_v0",
                    DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY,
                    Some(&call),
                    Some(callee),
                    caller_root.as_deref(),
                    None,
                    Some(format!(
                        "Fix task `{}`: call `{}` at {} returns `Result ..., {callee_root}` from callee `{}` declared at {}, so the failure cannot be implicit. Write `try {}` when the caller also returns `{callee_root}`, or write `try {} or fail CallerError.context` using this caller's declared error root.",
                        task.name,
                        call.source,
                        location(&statement.span),
                        callee.name,
                        location(&callee.span),
                        call.source,
                        call.source,
                    )),
                ),
            );
        }
    }

    TaskFailureAnalysis { facts }
}

pub(crate) fn parse_try_expression(text: &str) -> Option<TryExpression> {
    let rest = strip_keyword(text.trim(), "try")?;
    let (call_text, wrapper) = if let Some((call, wrapper)) = rest.split_once(" or fail ") {
        if wrapper.contains(" or fail ") {
            return None;
        }
        (call.trim(), Some(parse_failure_variant(wrapper.trim())?))
    } else {
        (rest.trim(), None)
    };
    let call = parse_direct_call(call_text)?;
    Some(TryExpression { call, wrapper })
}

pub(crate) fn parse_failure_variant(text: &str) -> Option<FailureVariant> {
    let (root, variant) = text.trim().split_once('.')?;
    if !is_type_ident(root) || !is_value_ident(variant) || variant.contains('.') {
        return None;
    }
    Some(FailureVariant {
        root: root.to_string(),
        variant: variant.to_string(),
    })
}

pub(crate) fn result_error_root(result: &str) -> Option<String> {
    result_parts(result).map(|(_success, error)| error)
}

pub(crate) fn result_success_type(result: &str) -> Option<String> {
    result_parts(result).map(|(success, _error)| success)
}

pub(crate) fn is_try_candidate(text: &str) -> bool {
    strip_keyword(text.trim(), "try").is_some()
}

pub(crate) fn is_meaningful_failure_declaration(text: &str) -> bool {
    is_meaningful_line_text(text) && hollow_contract_reason(text).is_none()
}

pub(crate) fn diagnostic_message(fact: &FailureFact) -> String {
    let relationship = match fact.diagnostic_code {
        Some(DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY) => {
            "a known fallible task call must use an explicit `try` form"
        }
        Some(DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION) => {
            "unwrapped propagation requires the caller and callee to declare the same error root"
        }
        Some(DiagnosticCode::FAILURE_WRAPPER_ROOT_MISMATCH) => {
            "a failure wrapper must use the caller's declared error root"
        }
        Some(DiagnosticCode::TRY_ON_INFALLIBLE_CALL) => {
            "`try` cannot be applied to a task without a declared `Result` error root"
        }
        Some(DiagnosticCode::DIRECT_FAILURE_ROOT_MISMATCH) => {
            "a direct `fail` value must use the task's declared error root"
        }
        Some(DiagnosticCode::UNSUPPORTED_TRY_EXPRESSION) => {
            "this `try` expression is outside Session W's two recognized forms"
        }
        Some(DiagnosticCode::MISSING_FAILURE_DECLARATION) => {
            "typed failure requires a meaningful `fails when:` declaration"
        }
        _ => "invalid typed-failure relationship",
    };
    format!(
        "{relationship}; call site {}, caller `{}` declared at {}{}",
        location(&fact.call_span),
        fact.caller_result_root
            .as_deref()
            .unwrap_or("non-Result task"),
        location(&fact.caller_span),
        fact.callee_span
            .as_ref()
            .map(|span| format!(", callee declared at {}", location(span)))
            .unwrap_or_default(),
    )
}

fn analyze_try_expression(
    task: &Task,
    statement: &BodyStatement,
    index: usize,
    expression: &str,
    caller_root: Option<&str>,
    catalog: &FailureCatalog,
    has_failure_declaration: bool,
) -> FailureFact {
    if statement.kind != "let_binding" || !is_exact_unannotated_binding(statement) {
        return unsupported_try_fact(
            task,
            statement,
            index,
            "try_requires_unannotated_let_binding_v0",
            None,
            caller_root,
        );
    }
    let Some(parsed) = parse_try_expression(expression) else {
        return unsupported_try_fact(
            task,
            statement,
            index,
            "unsupported_try_expression_shape_v0",
            try_call_hint(expression).as_ref(),
            caller_root,
        );
    };
    let Some(callee) = catalog.task(&parsed.call.callee) else {
        return unsupported_try_fact(
            task,
            statement,
            index,
            "try_callee_not_known_v0",
            Some(&parsed.call),
            caller_root,
        );
    };
    let Some(callee_root) = callee.error_root.as_deref() else {
        return issue_fact(
            task,
            statement,
            index,
            "try_infallible_call",
            "try_requires_fallible_callee_v0",
            DiagnosticCode::TRY_ON_INFALLIBLE_CALL,
            Some(&parsed.call),
            Some(callee),
            caller_root,
            parsed.wrapper.as_ref(),
            Some(format!(
                "Fix task `{}`: `try` at {} calls infallible task `{}` declared at {}, whose result has no error root. Remove `try` and the wrapper; use `{}` directly.",
                task.name,
                location(&statement.span),
                callee.name,
                location(&callee.span),
                parsed.call.source,
            )),
        );
    };

    if let Some(wrapper) = &parsed.wrapper {
        if caller_root != Some(wrapper.root.as_str()) {
            return issue_fact(
                task,
                statement,
                index,
                "failure_wrap",
                "failure_wrapper_root_must_match_caller_v0",
                DiagnosticCode::FAILURE_WRAPPER_ROOT_MISMATCH,
                Some(&parsed.call),
                Some(callee),
                caller_root,
                Some(wrapper),
                Some(format!(
                    "Fix task `{}`: wrapper `{}` at {} uses root `{}`, but the caller declared error root `{}` at {}; the callee `{}` declared `{callee_root}` at {}. Change the wrapper to this caller's root, or change the caller result deliberately.",
                    task.name,
                    wrapper.identity(),
                    location(&statement.span),
                    wrapper.root,
                    caller_root.unwrap_or("none"),
                    location(&task.span),
                    callee.name,
                    location(&callee.span),
                )),
            );
        }
    } else if caller_root != Some(callee_root) {
        return issue_fact(
            task,
            statement,
            index,
            "failure_propagation",
            "unwrapped_failure_roots_must_match_v0",
            DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION,
            Some(&parsed.call),
            Some(callee),
            caller_root,
            None,
            Some(format!(
                "Fix task `{}`: unwrapped `try` at {} propagates `{callee_root}` from callee `{}` declared at {}, but the caller declared error root `{}` at {}. Wrap with `or fail CallerError.context` using the caller root, or make the nominal roots equal.",
                task.name,
                location(&statement.span),
                callee.name,
                location(&callee.span),
                caller_root.unwrap_or("none"),
                location(&task.span),
            )),
        );
    }

    if !has_failure_declaration {
        return issue_fact(
            task,
            statement,
            index,
            if parsed.wrapper.is_some() {
                "failure_wrap"
            } else {
                "failure_propagation"
            },
            "typed_failure_requires_fails_when_v0",
            DiagnosticCode::MISSING_FAILURE_DECLARATION,
            Some(&parsed.call),
            Some(callee),
            caller_root,
            parsed.wrapper.as_ref(),
            Some(format!(
                "Fix task `{}`: typed failure at {} can produce `{}`, but the caller header at {} has no meaningful `fails when:` declaration. Add a concrete failure condition naming this path.",
                task.name,
                location(&statement.span),
                parsed
                    .wrapper
                    .as_ref()
                    .map_or(callee_root.to_string(), |wrapper| wrapper.root.clone()),
                location(&task.span),
            )),
        );
    }

    FailureFact {
        index,
        status: if parsed.wrapper.is_some() {
            "accepted_causal_failure_wrap_v0"
        } else {
            "accepted_same_root_failure_propagation_v0"
        },
        reason: None,
        diagnostic_code: None,
        form: if parsed.wrapper.is_some() {
            "failure_wrap"
        } else {
            "failure_propagation"
        },
        callee: Some(callee.name.clone()),
        callee_result_root: Some(callee_root.to_string()),
        caller_result_root: caller_root.map(str::to_string),
        wrapper_root: parsed.wrapper.as_ref().map(|wrapper| wrapper.root.clone()),
        call_span: portable_span(&statement.span),
        callee_span: Some(portable_span(&callee.span)),
        caller_span: portable_span(&task.span),
        help: None,
        success_type: callee.success_type.clone(),
        try_expression: Some(parsed),
    }
}

fn direct_fail_fact(
    task: &Task,
    statement: &BodyStatement,
    index: usize,
    caller_root: Option<&str>,
    variant: &FailureVariant,
    has_failure_declaration: bool,
) -> FailureFact {
    if caller_root != Some(variant.root.as_str()) {
        return issue_fact(
            task,
            statement,
            index,
            "direct_failure",
            "direct_failure_root_must_match_caller_v0",
            DiagnosticCode::DIRECT_FAILURE_ROOT_MISMATCH,
            None,
            None,
            caller_root,
            Some(variant),
            Some(format!(
                "Fix task `{}`: direct failure `{}` at {} uses root `{}`, but the caller declared error root `{}` at {}. Fail with a variant under the caller root, or change the caller result deliberately.",
                task.name,
                variant.identity(),
                location(&statement.span),
                variant.root,
                caller_root.unwrap_or("none"),
                location(&task.span),
            )),
        );
    }
    if !has_failure_declaration {
        return issue_fact(
            task,
            statement,
            index,
            "direct_failure",
            "typed_failure_requires_fails_when_v0",
            DiagnosticCode::MISSING_FAILURE_DECLARATION,
            None,
            None,
            caller_root,
            Some(variant),
            Some(format!(
                "Fix task `{}`: direct failure `{}` at {} requires a meaningful `fails when:` declaration on the caller declared at {}.",
                task.name,
                variant.identity(),
                location(&statement.span),
                location(&task.span),
            )),
        );
    }
    FailureFact {
        index,
        status: "accepted_nominal_direct_failure_v0",
        reason: None,
        diagnostic_code: None,
        form: "direct_failure",
        callee: None,
        callee_result_root: None,
        caller_result_root: caller_root.map(str::to_string),
        wrapper_root: Some(variant.root.clone()),
        call_span: portable_span(&statement.span),
        callee_span: None,
        caller_span: portable_span(&task.span),
        help: None,
        success_type: None,
        try_expression: None,
    }
}

fn unsupported_try_fact(
    task: &Task,
    statement: &BodyStatement,
    index: usize,
    reason: &'static str,
    call: Option<&DirectCall>,
    caller_root: Option<&str>,
) -> FailureFact {
    issue_fact(
        task,
        statement,
        index,
        "unsupported_try",
        reason,
        DiagnosticCode::UNSUPPORTED_TRY_EXPRESSION,
        call,
        None,
        caller_root,
        None,
        Some(format!(
            "Fix task `{}`: `try` at {} is outside Session W's exact `let value = try named_call(...)` or `let value = try named_call(...) or fail CallerError.context` forms ({}). Use one direct named call with ordinary value arguments; `borrow`, `change`, `consume`, nested calls, operators, and other expression shapes remain unsupported.",
            task.name,
            location(&statement.span),
            reason,
        )),
    )
}

#[allow(clippy::too_many_arguments)]
fn issue_fact(
    task: &Task,
    statement: &BodyStatement,
    index: usize,
    form: &'static str,
    reason: &'static str,
    code: DiagnosticCode,
    call: Option<&DirectCall>,
    callee: Option<&TaskFailureSignature>,
    caller_root: Option<&str>,
    wrapper: Option<&FailureVariant>,
    help: Option<String>,
) -> FailureFact {
    FailureFact {
        index,
        status: "rejected_typed_failure_relationship_v0",
        reason: Some(reason),
        diagnostic_code: Some(code),
        form,
        callee: call.map(|call| call.callee.clone()),
        callee_result_root: callee.and_then(|callee| callee.error_root.clone()),
        caller_result_root: caller_root.map(str::to_string),
        wrapper_root: wrapper.map(|wrapper| wrapper.root.clone()),
        call_span: portable_span(&statement.span),
        callee_span: callee.map(|callee| portable_span(&callee.span)),
        caller_span: portable_span(&task.span),
        help,
        success_type: callee.and_then(|callee| callee.success_type.clone()),
        try_expression: None,
    }
}

fn collect_signatures(items: &[Item], out: &mut BTreeMap<String, TaskFailureSignature>) {
    for item in items {
        match item {
            Item::Task(task) => {
                out.entry(task.name.clone())
                    .or_insert_with(|| TaskFailureSignature {
                        name: task.name.clone(),
                        success_type: task.result.as_deref().and_then(result_success_type),
                        error_root: task.result.as_deref().and_then(result_error_root),
                        span: portable_span(&task.span),
                    });
            }
            Item::App(app) => collect_signatures(&app.items, out),
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn result_parts(result: &str) -> Option<(String, String)> {
    let rest = strip_keyword(result.trim(), "Result")?;
    let (success, error) = rest.split_once(',')?;
    let success = success.trim();
    let error = error.trim();
    if success.is_empty() || !is_type_ident(error) {
        return None;
    }
    Some((success.to_string(), error.to_string()))
}

fn parse_direct_call(text: &str) -> Option<DirectCall> {
    let text = text.trim();
    let inside = text.strip_suffix(')')?;
    let (callee, args) = inside.split_once('(')?;
    let callee = callee.trim();
    if !is_value_ident(callee) || args.contains('(') || args.contains(')') {
        return None;
    }
    for argument in split_arguments(args) {
        let argument = argument.trim();
        if argument.is_empty()
            || ["borrow", "change", "consume", "try", "fail"]
                .iter()
                .any(|keyword| strip_keyword(argument, keyword).is_some())
            || !is_ordinary_value_argument(argument)
        {
            return None;
        }
    }
    Some(DirectCall {
        callee: callee.to_string(),
        source: text.to_string(),
    })
}

fn calls_in_expression(text: &str) -> Vec<DirectCall> {
    let bytes = text.as_bytes();
    let mut calls = Vec::new();
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
            index += 1;
            continue;
        }
        if !is_ident_start_byte(byte)
            || index.checked_sub(1).is_some_and(|previous| {
                is_ident_continue_byte(bytes[previous]) || bytes[previous] == b'.'
            })
        {
            index += 1;
            continue;
        }

        let start = index;
        index += 1;
        while index < bytes.len() && is_ident_continue_byte(bytes[index]) {
            index += 1;
        }
        let callee = &text[start..index];
        let mut open = index;
        while open < bytes.len() && bytes[open].is_ascii_whitespace() {
            open += 1;
        }
        if open >= bytes.len() || bytes[open] != b'(' || !is_value_ident(callee) {
            continue;
        }
        let end = matching_call_end(text, open).unwrap_or(bytes.len().saturating_sub(1));
        calls.push(DirectCall {
            callee: callee.to_string(),
            source: text[start..=end].trim().to_string(),
        });
    }

    calls
}

fn matching_call_end(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate().skip(open) {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        if byte == b'"' {
            in_string = true;
            continue;
        }
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn contains_keyword_token(text: &str, keyword: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
            index += 1;
            continue;
        }
        if !is_ident_start_byte(byte)
            || index
                .checked_sub(1)
                .is_some_and(|previous| bytes[previous] == b'.')
        {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len() && is_ident_continue_byte(bytes[index]) {
            index += 1;
        }
        if &text[start..index] == keyword {
            return true;
        }
    }
    false
}

fn is_ident_start_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte == b'_'
}

fn is_ident_continue_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
}

fn is_ordinary_value_argument(text: &str) -> bool {
    let text = text.trim();
    if matches!(text, "true" | "false")
        || text.parse::<i64>().is_ok()
        || (text.starts_with('"') && text.ends_with('"') && text.len() >= 2)
    {
        return true;
    }
    text.split('.').all(is_value_ident)
}

fn try_call_hint(text: &str) -> Option<DirectCall> {
    let rest = strip_keyword(text.trim(), "try")?;
    let call = rest.split_once(" or fail ").map_or(rest, |(call, _)| call);
    let callee = call.split_once('(')?.0.trim();
    is_value_ident(callee).then(|| DirectCall {
        callee: callee.to_string(),
        source: call.trim().to_string(),
    })
}

fn statement_expression(statement: &BodyStatement) -> Option<&str> {
    match statement.kind {
        "return" => strip_keyword(&statement.text, "return"),
        "fail" => strip_keyword(&statement.text, "fail"),
        "let_binding" | "mutable_binding" | "set_place" => statement
            .text
            .split_once('=')
            .map(|(_left, value)| value.trim()),
        "if_header" => header_body(&statement.text, "if"),
        "while_header" => header_body(&statement.text, "while"),
        "for_each_header" => header_body(&statement.text, "for each"),
        "for_index_header" => header_body(&statement.text, "for index"),
        "record_field_initializer" => statement
            .text
            .split_once(':')
            .map(|(_field, value)| value.trim()),
        "test_expectation" => strip_keyword(&statement.text, "expect"),
        _ => None,
    }
}

fn is_exact_unannotated_binding(statement: &BodyStatement) -> bool {
    let Some(rest) = strip_keyword(&statement.text, "let") else {
        return false;
    };
    let Some((left, _initializer)) = rest.split_once('=') else {
        return false;
    };
    is_value_ident(left.trim())
}

fn split_arguments(text: &str) -> Vec<&str> {
    if text.trim().is_empty() {
        Vec::new()
    } else {
        text.split(',').collect()
    }
}

fn header_body<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    strip_keyword(text, keyword)?
        .strip_suffix('{')
        .map(str::trim)
}

fn strip_keyword<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    if text == keyword {
        return Some("");
    }
    text.strip_prefix(keyword)
        .and_then(|rest| rest.strip_prefix(char::is_whitespace))
        .map(str::trim)
}

fn is_value_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first == '_')
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn is_type_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_uppercase() && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn location(span: &Span) -> String {
    format!(
        "{}:{}:{}",
        span.file.replace('\\', "/"),
        span.line,
        span.column
    )
}

fn portable_span(span: &Span) -> Span {
    Span::new(span.file.replace('\\', "/"), span.line, span.column)
}

#[cfg(test)]
mod tests {
    use super::{
        calls_in_expression, contains_keyword_token, is_meaningful_failure_declaration,
        is_try_candidate, parse_failure_variant, parse_try_expression, result_error_root,
    };

    #[test]
    fn failure_declarations_reject_hollow_contract_text() {
        for placeholder in ["todo", "TODO: explain later", "tbd", "safe", "true"] {
            assert!(!is_meaningful_failure_declaration(placeholder));
        }
        assert!(is_meaningful_failure_declaration(
            "the root source rejects an invalid flag"
        ));
    }

    #[test]
    fn parses_only_the_session_w_try_shapes() {
        let propagate = parse_try_expression("try load(7)").expect("propagation");
        assert_eq!(propagate.call.callee, "load");
        assert!(propagate.wrapper.is_none());

        let wrap =
            parse_try_expression("try load(7) or fail OuterError.context").expect("wrapping");
        assert_eq!(
            wrap.wrapper.expect("wrapper").identity(),
            "OuterError.context"
        );

        assert!(parse_try_expression("try borrow load(7)").is_none());
        assert!(parse_try_expression("try load(inner())").is_none());
        assert!(parse_try_expression("try value + 1").is_none());
        assert!(parse_failure_variant("Error.case.more").is_none());
        assert_eq!(
            result_error_root("Result UInt, WorkError").as_deref(),
            Some("WorkError")
        );
    }

    #[test]
    fn scans_nested_calls_and_bounds_the_try_keyword() {
        let calls = calls_in_expression("identity(source()) + source_list()");
        assert_eq!(
            calls
                .iter()
                .map(|call| call.callee.as_str())
                .collect::<Vec<_>>(),
            vec!["identity", "source", "source_list"]
        );
        assert_eq!(calls[1].source, "source()");
        assert!(
            !calls_in_expression("\"source()\"")
                .iter()
                .any(|call| call.callee == "source")
        );

        assert!(is_try_candidate("try source()"));
        assert!(!is_try_candidate("trying()"));
        assert!(!is_try_candidate("try_value()"));
        assert!(contains_keyword_token("value + try source()", "try"));
        assert!(!contains_keyword_token("trying()", "try"));
        assert!(!contains_keyword_token("object.try", "try"));
    }
}
