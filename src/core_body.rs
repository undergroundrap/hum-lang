use crate::ast::CanonicalCoreSectionExpectation;
use crate::diagnostic::Span;

pub const CORE_BODY_GRAMMAR_STATUS: &str = "partial_v0";

mod validated_construction {
    use super::{BodyStatement, CORE_BODY_GRAMMAR_STATUS};
    use crate::ast::{CanonicalExpression, ParsedBodyStatementKind, ValidatedCoreSection};
    use crate::diagnostic::Span;

    #[derive(Clone)]
    pub struct BodyGrammarReport {
        pub status: &'static str,
        pub grammar_status: &'static str,
        pub total_lines: usize,
        pub meaningful_lines: usize,
        pub recognized_lines: usize,
        pub unsupported_lines: usize,
        pub statements: Vec<BodyStatement>,
        _validated_lineage: ValidatedBodyGrammarLineage,
    }

    #[derive(Clone)]
    pub(crate) struct CanonicalBodyGrammarReport {
        pub(crate) status: &'static str,
        pub(crate) grammar_status: &'static str,
        total_lines: usize,
        pub(crate) meaningful_lines: usize,
        recognized_lines: usize,
        unsupported_lines: usize,
        pub(crate) statements: Vec<CanonicalBodyStatement>,
        _validated_lineage: ValidatedBodyGrammarLineage,
    }

    #[derive(Clone)]
    pub(crate) struct CanonicalBodyStatement {
        statement: BodyStatement,
        canonical_expression: Option<CanonicalExpression>,
        _validated_lineage: ValidatedBodyGrammarLineage,
    }

    impl CanonicalBodyStatement {
        pub(crate) fn statement(&self) -> &BodyStatement {
            &self.statement
        }

        pub(crate) fn canonical_expression(&self) -> Option<&CanonicalExpression> {
            self.canonical_expression.as_ref()
        }

        fn into_public_statement(self) -> BodyStatement {
            self.statement
        }

        #[cfg(test)]
        pub(crate) fn statement_mut_for_test(&mut self) -> &mut BodyStatement {
            &mut self.statement
        }
    }

    #[derive(Clone)]
    struct ValidatedBodyGrammarLineage;

    struct ValidatedBodyGrammarConstruction<'validated> {
        validated: ValidatedCoreSection<'validated>,
    }

    impl<'validated> ValidatedBodyGrammarConstruction<'validated> {
        fn new(validated: ValidatedCoreSection<'validated>) -> Self {
            Self { validated }
        }

        fn section(&self) -> &'validated crate::ast::Section {
            self.validated.section()
        }

        fn issue_lineage(&self) -> ValidatedBodyGrammarLineage {
            let _validated_capability = &self.validated;
            ValidatedBodyGrammarLineage
        }
    }

    impl std::fmt::Debug for BodyGrammarReport {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter
                .debug_struct("BodyGrammarReport")
                .field("status", &self.status)
                .field("grammar_status", &self.grammar_status)
                .field("total_lines", &self.total_lines)
                .field("meaningful_lines", &self.meaningful_lines)
                .field("recognized_lines", &self.recognized_lines)
                .field("unsupported_lines", &self.unsupported_lines)
                .field("statements", &self.statements)
                .finish()
        }
    }

    pub(super) fn build_body_grammar(
        validated: ValidatedCoreSection<'_>,
    ) -> CanonicalBodyGrammarReport {
        let construction = ValidatedBodyGrammarConstruction::new(validated);
        let section = construction.section();
        let mut statements = Vec::new();
        let mut meaningful_lines = 0usize;
        let mut recognized_lines = 0usize;
        let mut unsupported_lines = 0usize;

        for (line, retained) in section.lines.iter().zip(&section.body_syntax) {
            let Some(parsed) = retained.as_ref() else {
                continue;
            };

            meaningful_lines += 1;
            let canonical_expression = match &parsed.kind {
                ParsedBodyStatementKind::Return(expression) => Some(expression.canonical.clone()),
                ParsedBodyStatementKind::Binding { .. } | ParsedBodyStatementKind::Other { .. } => {
                    None
                }
            };
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
            statements.push(CanonicalBodyStatement {
                statement,
                canonical_expression,
                _validated_lineage: construction.issue_lineage(),
            });
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

        CanonicalBodyGrammarReport {
            status,
            grammar_status: CORE_BODY_GRAMMAR_STATUS,
            total_lines: section.lines.len(),
            meaningful_lines,
            recognized_lines,
            unsupported_lines,
            statements,
            _validated_lineage: construction.issue_lineage(),
        }
    }

    impl CanonicalBodyGrammarReport {
        pub(super) fn into_public_report(self) -> BodyGrammarReport {
            let statements = self
                .statements
                .into_iter()
                .map(CanonicalBodyStatement::into_public_statement)
                .collect::<Vec<_>>();
            BodyGrammarReport {
                status: self.status,
                grammar_status: self.grammar_status,
                total_lines: self.total_lines,
                meaningful_lines: self.meaningful_lines,
                recognized_lines: self.recognized_lines,
                unsupported_lines: self.unsupported_lines,
                statements,
                _validated_lineage: self._validated_lineage,
            }
        }
    }
}

pub use validated_construction::BodyGrammarReport;
pub(crate) use validated_construction::{CanonicalBodyGrammarReport, CanonicalBodyStatement};

#[derive(Debug, Clone)]
pub struct BodyStatement {
    pub span: Span,
    pub text: String,
    pub kind: &'static str,
    pub status: &'static str,
    pub expression_kind: Option<&'static str>,
    pub reason: Option<&'static str>,
}

pub(crate) fn analyze_does_section(
    expectation: CanonicalCoreSectionExpectation<'_>,
) -> BodyGrammarReport {
    try_analyze_does_section(expectation)
        .expect("canonical Core section authority invariant failed before construction")
}

pub(crate) fn try_analyze_does_section(
    expectation: CanonicalCoreSectionExpectation<'_>,
) -> Result<BodyGrammarReport, &'static str> {
    let validated = expectation.validate()?;
    Ok(validated_construction::build_body_grammar(validated).into_public_report())
}

pub(crate) fn analyze_does_section_for_lowering(
    expectation: CanonicalCoreSectionExpectation<'_>,
) -> CanonicalBodyGrammarReport {
    let validated = expectation
        .validate()
        .expect("canonical Core section authority invariant failed before construction");
    validated_construction::build_body_grammar(validated)
}

#[allow(unexpected_cfgs)]
mod validated_body_grammar_construction_compile_proof {
    #[cfg(hum_compile_fail_validated_body_grammar_construction)]
    use super::{
        BodyGrammarReport, BodyStatement, CORE_BODY_GRAMMAR_STATUS, CanonicalBodyGrammarReport,
        CanonicalBodyStatement, validated_construction,
    };
    #[cfg(hum_compile_fail_validated_body_grammar_construction)]
    use crate::diagnostic::Span;

    #[cfg(hum_compile_fail_validated_body_grammar_construction)]
    fn body_grammar_report_foreign_literal_must_not_compile() -> BodyGrammarReport {
        let body_grammar_report_foreign_literal_must_not_compile = BodyGrammarReport {
            status: "body_grammar_report_foreign_literal_must_not_compile",
            grammar_status: CORE_BODY_GRAMMAR_STATUS,
            total_lines: 0,
            meaningful_lines: 0,
            recognized_lines: 0,
            unsupported_lines: 0,
            statements: Vec::new(),
        };
        body_grammar_report_foreign_literal_must_not_compile
    }

    #[cfg(hum_compile_fail_validated_body_grammar_construction)]
    fn canonical_body_grammar_report_foreign_literal_must_not_compile() -> CanonicalBodyGrammarReport
    {
        let canonical_body_grammar_report_foreign_literal_must_not_compile = 0usize;
        let canonical_body_grammar_report_foreign_literal_must_not_compile =
            CanonicalBodyGrammarReport {
                status: "canonical_body_grammar_report_foreign_literal_must_not_compile",
                grammar_status: CORE_BODY_GRAMMAR_STATUS,
                total_lines: canonical_body_grammar_report_foreign_literal_must_not_compile,
                meaningful_lines: 0,
                recognized_lines: 0,
                unsupported_lines: 0,
                statements: Vec::new(),
            };
        canonical_body_grammar_report_foreign_literal_must_not_compile
    }

    #[cfg(hum_compile_fail_validated_body_grammar_construction)]
    fn canonical_body_statement_foreign_literal_must_not_compile() -> CanonicalBodyStatement {
        let canonical_body_statement_foreign_literal_must_not_compile = BodyStatement {
            span: Span {
                file: String::new(),
                line: 0,
                column: 0,
            },
            text: String::new(),
            kind: "compile_fail",
            status: "compile_fail",
            expression_kind: None,
            reason: None,
        };
        CanonicalBodyStatement {
            // canonical_body_statement_foreign_literal_must_not_compile
            statement: canonical_body_statement_foreign_literal_must_not_compile,
            canonical_expression: None,
        }
    }

    #[cfg(hum_compile_fail_validated_body_grammar_construction)]
    fn validated_body_grammar_permit_from_raw_section_must_not_compile(
        section: &crate::ast::Section,
    ) -> CanonicalBodyGrammarReport {
        let validated_body_grammar_permit_from_raw_section_must_not_compile = section;
        validated_construction::build_body_grammar(
            validated_body_grammar_permit_from_raw_section_must_not_compile,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::Program, parser::parse_source};

    use super::{
        BodyGrammarReport, CanonicalBodyGrammarReport, analyze_does_section,
        analyze_does_section_for_lowering, try_analyze_does_section,
    };
    use crate::core_lower::{
        CoreOperationExpectationError, with_expected_core_operations_for_item,
    };

    #[test]
    fn validated_body_grammar_construction_is_compiler_sealed() {
        fn forward_canonical(report: CanonicalBodyGrammarReport) -> CanonicalBodyGrammarReport {
            report
        }

        fn forward_public(report: BodyGrammarReport) -> BodyGrammarReport {
            report
        }

        let parsed = parse_source(
            "sealed-body.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\n",
        );
        let program = Program {
            files: vec![parsed.file],
        };
        let item = &program.files[0].items[0];
        let crate::ast::Item::Task(task) = item else {
            panic!("task")
        };
        let section = task.section("does").expect("does");

        let canonical = analyze_does_section_for_lowering(
            program
                .canonical_core_expectation(item, section)
                .expect("parser-owned expectation"),
        );
        assert!(
            canonical.statements[0].canonical_expression().is_some(),
            "the validated lowering entry retains parser-owned expression authority"
        );

        let mut forwarded = forward_canonical(canonical.clone());
        assert!(forwarded.statements[0].canonical_expression().is_some());
        forwarded.status = "mutated_after_first_construction";
        forwarded.statements[0].statement_mut_for_test().text = "mutated candidate".to_string();
        assert_ne!(forwarded.status, canonical.status);
        assert_ne!(
            forwarded.statements[0].statement().text,
            canonical.statements[0].statement().text
        );
        let owner = program
            .canonical_core_operation_owner_expectation(item, section)
            .expect("parser-owned operation owner");
        let mut visited_mutated_operation = false;
        assert_eq!(
            with_expected_core_operations_for_item(owner, &forwarded, &[], |_| {
                visited_mutated_operation = true;
            }),
            Err(CoreOperationExpectationError::Missing(0)),
            "opaque lineage must not let mutated current fields bypass production validation"
        );
        assert!(
            visited_mutated_operation,
            "the real production stream must inspect the mutated lineage-bearing artifact"
        );

        let public = canonical.into_public_report();
        assert_eq!(public.statements.len(), 1);
        assert!(!format!("{public:?}").contains("canonical_expression"));
        let mut public_clone = forward_public(public.clone());
        public_clone.status = "mutated_after_first_construction";
        assert_ne!(public_clone.status, public.status);

        let mut corrupted = parse_source(
            "sealed-body-invalid.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\n",
        );
        let crate::ast::Item::Task(task) = &mut corrupted.file.items[0] else {
            panic!("task")
        };
        task.sections
            .iter_mut()
            .find(|section| section.name == "does")
            .expect("does")
            .lines[0]
            .text = "return b + a".to_string();
        let item = &corrupted.file.items[0];
        let crate::ast::Item::Task(task) = item else {
            panic!("task")
        };
        let section = task.section("does").expect("does");
        let expectation = corrupted
            .canonical_core_expectation(item, section)
            .expect("live expectation remains locatable");
        assert!(matches!(
            try_analyze_does_section(expectation),
            Err("canonical_core_section_projection_mismatch_v0")
        ));
    }

    #[test]
    fn validated_body_transports_parser_owned_minimal_add_tree() {
        let parsed = parse_source(
            "minimal-add.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\n",
        );
        let item = &parsed.file.items[0];
        let crate::ast::Item::Task(task) = item else {
            panic!("task")
        };
        let section = task.section("does").expect("does");
        let crate::ast::ParsedBodyStatementKind::Return(original) = &task.body_syntax[0].kind
        else {
            panic!("return")
        };
        let report = analyze_does_section_for_lowering(
            parsed
                .canonical_core_expectation(item, section)
                .expect("parser-owned expectation"),
        );
        let transported = report.statements[0]
            .canonical_expression()
            .expect("transported canonical expression");

        assert_eq!(transported.node_id, original.canonical.node_id);
        assert_eq!(transported.range, original.canonical.range);
        let crate::ast::CanonicalExpressionKind::Binary {
            operator,
            left,
            right,
        } = &transported.kind
        else {
            panic!("binary")
        };
        let crate::ast::CanonicalExpressionKind::Binary {
            left: original_left,
            right: original_right,
            ..
        } = &original.canonical.kind
        else {
            panic!("original binary")
        };
        assert_eq!(*operator, crate::ast::ParsedBinaryOperator::Add);
        assert_eq!(left.node_id, original_left.node_id);
        assert_eq!(right.node_id, original_right.node_id);
        assert_eq!(left.range, original_left.range);
        assert_eq!(right.range, original_right.range);
        assert_ne!(left.node_id, right.node_id);
        assert!(matches!(
            &left.kind,
            crate::ast::CanonicalExpressionKind::Identifier(name) if name == "a"
        ));
        assert!(matches!(
            &right.kind,
            crate::ast::CanonicalExpressionKind::Identifier(name) if name == "b"
        ));

        let mixed = parse_source(
            "mixed-body.hum",
            r#"task mixed(a: Int, b: Int) -> Int {
  does:
    let sum = a + b
    save sum in scratch
    return a + b
    return b + a
}
"#,
        );
        let mixed_item = &mixed.file.items[0];
        let crate::ast::Item::Task(mixed_task) = mixed_item else {
            panic!("mixed task")
        };
        let mixed_section = mixed_task.section("does").expect("mixed does");
        let mixed_report = analyze_does_section_for_lowering(
            mixed
                .canonical_core_expectation(mixed_item, mixed_section)
                .expect("mixed parser-owned expectation"),
        );
        assert_eq!(mixed_report.statements.len(), mixed_task.body_syntax.len());

        let mut bindings = 0usize;
        let mut other_or_unsupported = 0usize;
        let mut returns = 0usize;
        for (transported, parsed_statement) in
            mixed_report.statements.iter().zip(&mixed_task.body_syntax)
        {
            assert_eq!(transported.statement().kind, parsed_statement.core_kind);
            assert_eq!(transported.statement().status, parsed_statement.core_status);
            match &parsed_statement.kind {
                crate::ast::ParsedBodyStatementKind::Return(expression) => {
                    returns += 1;
                    assert_eq!(
                        transported
                            .canonical_expression()
                            .expect("return authority"),
                        &expression.canonical
                    );
                }
                crate::ast::ParsedBodyStatementKind::Binding { .. } => {
                    bindings += 1;
                    assert!(transported.canonical_expression().is_none());
                }
                crate::ast::ParsedBodyStatementKind::Other { .. } => {
                    other_or_unsupported += 1;
                    assert!(transported.canonical_expression().is_none());
                }
            }
        }
        assert_eq!(bindings, 1);
        assert!(other_or_unsupported >= 1);
        assert!(
            mixed_report
                .statements
                .iter()
                .any(|statement| statement.statement().status == "unsupported_v0")
        );
        assert_eq!(returns, 2);
    }

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
        let section = task.section("does").expect("does section");
        let report = analyze_does_section(
            parsed
                .canonical_core_expectation(&parsed.file.items[0], section)
                .expect("parser-owned expectation"),
        );

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
        let task_section = task.section("does").expect("task does");
        let test_section = test.section("does").expect("test does");
        let task_report = analyze_does_section(
            parsed
                .canonical_core_expectation(&parsed.file.items[0], task_section)
                .expect("task expectation"),
        );
        let test_report = analyze_does_section(
            parsed
                .canonical_core_expectation(&parsed.file.items[1], test_section)
                .expect("test expectation"),
        );

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
        {
            let crate::ast::Item::Task(task) = &mut parsed.file.items[0] else {
                panic!("task")
            };
            let section = task
                .sections
                .iter_mut()
                .find(|section| section.name == "does")
                .expect("does");
            section.lines[0].text = "save fabricated in nowhere".to_string();
        }
        let item = &parsed.file.items[0];
        let crate::ast::Item::Task(task) = item else {
            panic!("task")
        };
        let section = task.section("does").expect("does");
        let expectation = parsed
            .canonical_core_expectation(item, section)
            .expect("live expectation remains locatable");
        assert!(matches!(
            try_analyze_does_section(expectation),
            Err("canonical_core_section_projection_mismatch_v0")
        ));
    }

    #[test]
    fn retained_parser_fact_mutation_is_observable() {
        let mut parsed = parse_source(
            "retained-body-mutation.hum",
            "task retained() -> UInt {\n  does:\n    return 7\n}\n",
        );
        {
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
        }
        let item = &parsed.file.items[0];
        let crate::ast::Item::Task(task) = item else {
            panic!("task")
        };
        let section = task.section("does").expect("does");
        let expectation = parsed
            .canonical_core_expectation(item, section)
            .expect("live expectation remains locatable");
        assert!(matches!(
            try_analyze_does_section(expectation),
            Err("canonical_core_section_projection_mismatch_v0")
        ));
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
        let section = task.section("does").expect("does");
        let report = analyze_does_section(
            parsed
                .canonical_core_expectation(&parsed.file.items[3], section)
                .expect("compatibility expectation"),
        );
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
