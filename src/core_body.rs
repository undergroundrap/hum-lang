use crate::ast::Section;
use crate::diagnostic::Span;

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

    for (line, retained) in section.lines.iter().zip(&section.body_syntax) {
        let Some(parsed) = retained.as_ref() else {
            continue;
        };

        meaningful_lines += 1;
        let statement = BodyStatement {
            span: Span {
                file: line.span.file.replace('\\', "/"),
                line: line.span.line,
                column: line.span.column,
            },
            text: line.text.trim().to_string(),
            kind: parsed.core_kind,
            status: parsed.core_status,
            expression_kind: parsed.core_expression_kind,
            reason: parsed.core_reason,
        };
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

#[cfg(test)]
mod tests {
    use crate::parser::parse_source;

    use super::analyze_does_section;

    #[test]
    fn recognizes_first_core_body_shapes_without_lowering() {
        let source = r#"task remember_title(title: Text) -> Result WorkItem, WorkError {
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

    save item in work_items
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

    #[test]
    fn retained_parser_facts_survive_section_text_sabotage() {
        let mut parsed = parse_source(
            "retained-body.hum",
            "task retained() -> UInt {\n  does:\n    return 7\n}\n",
        );
        let crate::ast::Item::Task(task) = &mut parsed.file.items[0] else {
            panic!("task")
        };
        let section = task
            .sections
            .iter_mut()
            .find(|section| section.name == "does")
            .expect("does");
        section.lines[0].text = "save fabricated in nowhere".to_string();
        let report = analyze_does_section(section);
        assert_eq!(report.meaningful_lines, 1);
        assert_eq!(report.statements[0].kind, "return");
        assert_eq!(report.statements[0].expression_kind, Some("int_literal"));
        assert_eq!(report.statements[0].status, "recognized_v0");
    }

    #[test]
    fn retained_parser_fact_mutation_is_observable() {
        let mut parsed = parse_source(
            "retained-body-mutation.hum",
            "task retained() -> UInt {\n  does:\n    return 7\n}\n",
        );
        let crate::ast::Item::Task(task) = &mut parsed.file.items[0] else {
            panic!("task")
        };
        let section = task
            .sections
            .iter_mut()
            .find(|section| section.name == "does")
            .expect("does");
        let retained = section.body_syntax[0].as_mut().expect("retained fact");
        retained.core_kind = "unknown_body_line";
        retained.core_status = "unsupported_v0";
        retained.core_reason = Some("mutated_parser_fact_v0");
        let report = analyze_does_section(section);
        assert_eq!(report.unsupported_lines, 1);
        assert_eq!(report.statements[0].kind, "unknown_body_line");
        assert_eq!(report.statements[0].reason, Some("mutated_parser_fact_v0"));
    }

    #[test]
    fn parser_owned_core_kinds_preserve_established_preview_pairs() {
        let parsed = parse_source(
            "core-kind-compatibility.hum",
            r#"type WorkItem {
  done: Bool
}

type SourceError {
  code: Text
}

task source(flag: Bool) -> Result UInt, SourceError {
  does:
    return 1
}

task compatibility(title: Text, flag: Bool) -> Int {
  does:
    if title is empty {
      let tried = try source(flag)
      let wrapped = borrow source(flag)
      let grouped = (flag)
      let item = WorkItem {
        done: false
      }
      return -1
    }
}
"#,
        );
        let crate::ast::Item::Task(task) = &parsed.file.items[3] else {
            panic!("compatibility task")
        };
        let report = analyze_does_section(task.section("does").expect("does"));
        let kind_for = |text: &str| {
            report
                .statements
                .iter()
                .find(|statement| statement.text == text)
                .and_then(|statement| statement.expression_kind)
        };
        assert_eq!(kind_for("if title is empty {"), Some("condition_text"));
        assert_eq!(
            crate::core_expr::analyze_expression("title is empty").kind,
            "condition_or_surface_binary"
        );
        assert_eq!(
            kind_for("let tried = try source(flag)"),
            Some("try_call_like")
        );
        assert_eq!(
            crate::core_expr::analyze_expression("try source(flag)").kind,
            "try_call_like"
        );
        assert_eq!(
            kind_for("let wrapped = borrow source(flag)"),
            Some("call_like")
        );
        assert_eq!(
            crate::core_expr::analyze_expression("borrow source(flag)").kind,
            "call_like"
        );
        assert_eq!(kind_for("let grouped = (flag)"), Some("call_like"));
        assert_eq!(
            crate::core_expr::analyze_expression("(flag)").kind,
            "call_like"
        );
        assert_eq!(kind_for("done: false"), Some("bool_literal"));
        assert_eq!(
            crate::core_expr::analyze_expression("false").kind,
            "bool_literal"
        );
        assert_eq!(kind_for("return -1"), Some("name_or_text"));
        assert_eq!(
            crate::core_expr::analyze_expression("-1").kind,
            "surface_text"
        );
    }
}
