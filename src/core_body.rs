use crate::ast::Section;
use crate::diagnostic::Span;
use crate::graph::is_meaningful_line_text;

pub const CORE_BODY_GRAMMAR_STATUS: &str = "partial_v0";

#[derive(Debug, Clone)]
pub struct BodyGrammarReport {
    pub status: &'static str,
    pub grammar_status: &'static str,
    pub total_lines: usize,
    pub meaningful_lines: usize,
    pub recognized_lines: usize,
    pub unsupported_lines: usize,
    pub statements: Vec<BodyStatement>,
}

#[derive(Debug, Clone)]
pub struct BodyStatement {
    pub span: Span,
    pub text: String,
    pub kind: &'static str,
    pub status: &'static str,
    pub expression_kind: Option<&'static str>,
    pub reason: Option<&'static str>,
}

pub fn analyze_does_section(section: &Section) -> BodyGrammarReport {
    let mut statements = Vec::new();
    let mut meaningful_lines = 0usize;
    let mut recognized_lines = 0usize;
    let mut unsupported_lines = 0usize;

    for line in &section.lines {
        let text = line.text.trim();
        if !is_meaningful_line_text(text) {
            continue;
        }

        meaningful_lines += 1;
        let statement = classify_line(text, &line.span);
        if statement.status == "unsupported_v0" {
            unsupported_lines += 1;
        } else {
            recognized_lines += 1;
        }
        statements.push(statement);
    }

    let status = if meaningful_lines == 0 {
        "empty_body"
    } else if unsupported_lines == 0 {
        "partial_v0_all_lines_recognized"
    } else if recognized_lines > 0 {
        "partial_v0_with_unsupported_lines"
    } else {
        "unsupported_v0"
    };

    BodyGrammarReport {
        status,
        grammar_status: CORE_BODY_GRAMMAR_STATUS,
        total_lines: section.lines.len(),
        meaningful_lines,
        recognized_lines,
        unsupported_lines,
        statements,
    }
}

fn classify_line(text: &str, span: &Span) -> BodyStatement {
    if text == "}" {
        return statement(span, text, "block_close", "recognized_v0", None, None);
    }

    if is_nested_intent_header(text) {
        return statement(
            span,
            text,
            "nested_intent_header",
            "recognized_v0",
            None,
            Some("nested_intent_lowering_not_implemented"),
        );
    }

    if let Some(condition) = header_body(text, "if") {
        return statement(
            span,
            text,
            "if_header",
            "recognized_v0",
            Some(expression_kind_for_condition(condition)),
            None,
        );
    }

    if let Some(condition) = header_body(text, "while") {
        return statement(
            span,
            text,
            "while_header",
            "recognized_v0",
            Some(expression_kind_for_condition(condition)),
            None,
        );
    }

    if text == "loop {" {
        return statement(span, text, "loop_header", "recognized_v0", None, None);
    }

    if let Some(rest) = header_body(text, "for each") {
        return statement(
            span,
            text,
            "for_each_header",
            "recognized_v0",
            Some(expression_kind(rest)),
            None,
        );
    }

    if let Some(rest) = header_body(text, "for index") {
        return statement(
            span,
            text,
            "for_index_header",
            "recognized_v0",
            Some(expression_kind(rest)),
            None,
        );
    }

    if let Some(rest) = strip_keyword(text, "return") {
        return statement(
            span,
            text,
            "return",
            "recognized_v0",
            Some(expression_kind(rest)),
            None,
        );
    }

    if let Some(rest) = strip_keyword(text, "fail") {
        return statement(
            span,
            text,
            "fail",
            "recognized_v0",
            Some(expression_kind(rest)),
            None,
        );
    }

    if let Some(rest) = strip_keyword(text, "change") {
        return classify_binding(span, text, rest, "mutable_binding");
    }

    if let Some(rest) = strip_keyword(text, "let") {
        return classify_binding(span, text, rest, "let_binding");
    }

    if let Some(rest) = strip_keyword(text, "set") {
        let expression = rest.split_once('=').map(|(_place, value)| value.trim());
        return statement(
            span,
            text,
            "set_place",
            "recognized_v0",
            expression.map(expression_kind),
            None,
        );
    }

    if let Some(rest) = strip_keyword(text, "expect") {
        return statement(
            span,
            text,
            "test_expectation",
            "recognized_v0",
            Some(expression_kind(rest)),
            Some("test_body_not_core_runtime"),
        );
    }

    if text.starts_with("save ") && text.contains(" in ") {
        return statement(
            span,
            text,
            "save_in_store",
            "unsupported_v0",
            None,
            Some("surface_save_requires_store_lowering"),
        );
    }

    if is_record_field_initializer(text) {
        return statement(
            span,
            text,
            "record_field_initializer",
            "recognized_v0",
            text.split_once(':')
                .map(|(_field, value)| expression_kind(value.trim())),
            Some("record_literal_lowering_not_implemented"),
        );
    }

    statement(
        span,
        text,
        "unknown_body_line",
        "unsupported_v0",
        None,
        Some("not_in_core_body_grammar_v0"),
    )
}

fn classify_binding(span: &Span, text: &str, rest: &str, kind: &'static str) -> BodyStatement {
    let expression = rest.split_once('=').map(|(_left, value)| value.trim());
    let status = if expression.is_some() {
        "recognized_v0"
    } else {
        "unsupported_v0"
    };
    let reason = if expression.is_some() {
        None
    } else {
        Some("binding_missing_initializer")
    };
    statement(
        span,
        text,
        kind,
        status,
        expression.map(expression_kind),
        reason,
    )
}

fn statement(
    span: &Span,
    text: &str,
    kind: &'static str,
    status: &'static str,
    expression_kind: Option<&'static str>,
    reason: Option<&'static str>,
) -> BodyStatement {
    BodyStatement {
        span: Span {
            file: span.file.replace('\\', "/"),
            line: span.line,
            column: span.column,
        },
        text: text.to_string(),
        kind,
        status,
        expression_kind,
        reason,
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

fn is_nested_intent_header(text: &str) -> bool {
    matches!(
        text.strip_suffix(':').map(str::trim),
        Some("keeps" | "changes" | "needs" | "ensures" | "watch for" | "cost" | "does")
    )
}

fn is_record_field_initializer(text: &str) -> bool {
    let Some((field, value)) = text.split_once(':') else {
        return false;
    };
    !text.ends_with(':')
        && !field.trim().is_empty()
        && field
            .trim()
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == ' ')
        && !value.trim().is_empty()
}

fn expression_kind_for_condition(text: &str) -> &'static str {
    if has_binary_operator(text) || text.contains(" is ") || text.contains(" does ") {
        "condition_text"
    } else {
        expression_kind(text)
    }
}

fn expression_kind(text: &str) -> &'static str {
    let text = text.trim();
    if text.is_empty() {
        "unit"
    } else if text == "true" || text == "false" {
        "bool_literal"
    } else if text.chars().all(|ch| ch.is_ascii_digit()) {
        "int_literal"
    } else if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        "text_literal"
    } else if text.ends_with('{') {
        "record_literal_start"
    } else if text.contains('(') && text.contains(')') {
        "call_like"
    } else if has_binary_operator(text) {
        "binary_expression"
    } else if text.contains('.') {
        "path_or_name"
    } else {
        "name_or_text"
    }
}

fn has_binary_operator(text: &str) -> bool {
    [
        " == ", " != ", " <= ", " >= ", " < ", " > ", " + ", " - ", " * ", " / ", " and ", " or ",
    ]
    .iter()
    .any(|operator| text.contains(operator))
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_source;

    use super::analyze_does_section;

    #[test]
    fn recognizes_first_core_body_shapes_without_lowering() {
        let source = r#"task remember title(title: Text) -> Result WorkItem, WorkError {
  why:
    save a title

  does:
    if title is empty {
      fail WorkError.empty_title
    }

    let item = WorkItem {
      id: clock.now_text
      title: title
      done: false
    }

    save item in work items
    return item
}
"#;
        let parsed = parse_source("body.hum", source);
        let task = match &parsed.file.items[0] {
            crate::ast::Item::Task(task) => task,
            other => panic!("expected task, got {other:?}"),
        };
        let report = analyze_does_section(task.section("does").expect("does section"));

        assert_eq!(report.grammar_status, "partial_v0");
        assert_eq!(report.meaningful_lines, 10);
        assert_eq!(report.unsupported_lines, 1);
        assert!(
            report
                .statements
                .iter()
                .any(|statement| statement.kind == "if_header")
        );
        assert!(
            report
                .statements
                .iter()
                .any(|statement| statement.kind == "fail")
        );
        assert!(report.statements.iter().any(|statement| {
            statement.kind == "save_in_store"
                && statement.reason == Some("surface_save_requires_store_lowering")
        }));
    }

    #[test]
    fn recognizes_loop_mutation_and_test_expectations() {
        let source = r#"task count() -> UInt {
  why:
    count things

  does:
    change attempts: UInt = 0
    while attempts < 16 {
      set attempts = attempts + 1
    }
    return attempts
}

test count unit {
  why:
    check count

  does:
    expect count() returns UInt
}
"#;
        let parsed = parse_source("loop.hum", source);
        let task = match &parsed.file.items[0] {
            crate::ast::Item::Task(task) => task,
            other => panic!("expected task, got {other:?}"),
        };
        let test = match &parsed.file.items[1] {
            crate::ast::Item::Test(test) => test,
            other => panic!("expected test, got {other:?}"),
        };
        let task_report = analyze_does_section(task.section("does").expect("task does"));
        let test_report = analyze_does_section(test.section("does").expect("test does"));

        assert_eq!(task_report.unsupported_lines, 0);
        assert!(
            task_report
                .statements
                .iter()
                .any(|statement| statement.kind == "mutable_binding")
        );
        assert!(
            task_report
                .statements
                .iter()
                .any(|statement| statement.kind == "while_header")
        );
        assert!(
            task_report
                .statements
                .iter()
                .any(|statement| statement.kind == "set_place")
        );
        assert!(
            test_report
                .statements
                .iter()
                .any(|statement| statement.kind == "test_expectation")
        );
    }
}
