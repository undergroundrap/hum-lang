use crate::ast::{Item, Program, Section, SectionLine, Task, Test};
use crate::diagnostic::{DiagnosticInvariantError, DiagnosticOccurrenceSet, DiagnosticProjection};
use crate::node_id;
use crate::syntax;

pub struct TestCoverage<'a> {
    pub test_name: &'a str,
    pub modifiers: &'a [String],
    pub line: &'a SectionLine,
    pub covers: String,
    pub coverage_key: String,
}

pub struct TestObligation<'a> {
    pub id: String,
    pub kind: &'static str,
    pub blame: &'static str,
    pub source_section: &'static str,
    pub line: &'a SectionLine,
    pub covers: String,
    pub suggested_test: String,
}

pub struct EvidenceObligation<'a> {
    pub id: String,
    pub kind: &'static str,
    pub blame: &'static str,
    pub source_section: &'static str,
    pub line: &'a SectionLine,
    pub covers: String,
    pub suggested_evidence: String,
}

pub fn collect_test_coverages(program: &Program) -> Vec<TestCoverage<'_>> {
    let mut coverages = Vec::new();
    for file in &program.files {
        collect_test_coverages_from_items(&file.items, &mut coverages);
    }
    coverages
}

fn collect_test_coverages_from_items<'a>(items: &'a [Item], coverages: &mut Vec<TestCoverage<'a>>) {
    for item in items {
        match item {
            Item::App(app) => collect_test_coverages_from_items(&app.items, coverages),
            Item::Test(test) => collect_test_coverages_from_test(test, coverages),
            _ => {}
        }
    }
}

fn collect_test_coverages_from_test<'a>(test: &'a Test, coverages: &mut Vec<TestCoverage<'a>>) {
    for section in test
        .sections
        .iter()
        .filter(|section| section.name == "covers")
    {
        for line in meaningful_lines(section) {
            coverages.push(TestCoverage {
                test_name: &test.name,
                modifiers: &test.modifiers,
                line,
                covers: normalize_coverage(&line.text),
                coverage_key: coverage_key(&line.text),
            });
        }
    }
}

pub fn linked_test_count(covers: &str, test_coverages: &[TestCoverage<'_>]) -> usize {
    test_coverages
        .iter()
        .filter(|coverage| coverage_match_kind(covers, coverage).is_some())
        .count()
}

pub fn coverage_match_kind(covers: &str, coverage: &TestCoverage<'_>) -> Option<&'static str> {
    let normalized_covers = normalize_coverage(covers);
    if coverage.covers == normalized_covers {
        return Some("exact");
    }

    let coverage_key = coverage_key(covers);
    if !coverage_key.is_empty() && coverage.coverage_key == coverage_key {
        return Some("canonical");
    }

    None
}

pub fn test_obligations(task: &Task) -> Vec<TestObligation<'_>> {
    let mut obligations = Vec::new();
    for obligation_section in syntax::TEST_OBLIGATION_SECTIONS {
        for section in task
            .sections
            .iter()
            .filter(|section| section.name == obligation_section.name)
        {
            for line in meaningful_lines(section) {
                obligations.push(TestObligation {
                    id: node_id::span(
                        "obligation",
                        &line.span,
                        &format!("{} {} {}", task.name, obligation_section.name, line.text),
                    ),
                    kind: obligation_section.kind,
                    blame: obligation_section.blame,
                    source_section: obligation_section.name,
                    line,
                    covers: format!("{} {} {}", task.name, obligation_section.name, line.text),
                    suggested_test: suggested_test_name(
                        &task.name,
                        obligation_section.kind,
                        &line.text,
                    ),
                });
            }
        }
    }
    obligations
}

pub fn evidence_obligations(task: &Task) -> Vec<EvidenceObligation<'_>> {
    let mut obligations = Vec::new();
    for obligation_section in syntax::EVIDENCE_OBLIGATION_SECTIONS {
        for section in task
            .sections
            .iter()
            .filter(|section| section.name == obligation_section.name)
        {
            for line in meaningful_lines(section) {
                obligations.push(EvidenceObligation {
                    id: node_id::span(
                        "evidence",
                        &line.span,
                        &format!("{} {} {}", task.name, obligation_section.name, line.text),
                    ),
                    kind: obligation_section.kind,
                    blame: obligation_section.blame,
                    source_section: obligation_section.name,
                    line,
                    covers: format!("{} {} {}", task.name, obligation_section.name, line.text),
                    suggested_evidence: suggested_evidence_name(
                        &task.name,
                        obligation_section.kind,
                        &line.text,
                    ),
                });
            }
        }
    }
    obligations
}

pub fn meaningful_lines(section: &Section) -> impl Iterator<Item = &SectionLine> {
    section
        .lines
        .iter()
        .filter(|line| is_meaningful_line_text(&line.text))
}

pub fn is_meaningful_line_text(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty() && !text.starts_with('#') && !text.starts_with("//")
}

pub(crate) fn validate_diagnostic_occurrence_projection(
    authoritative: &DiagnosticOccurrenceSet,
    projected: &DiagnosticProjection,
) -> Result<(), DiagnosticInvariantError> {
    projected.validate_against("graph", authoritative)
}

pub fn hollow_contract_reason(text: &str) -> Option<&'static str> {
    let lower = text.trim().to_ascii_lowercase();
    let compact = lower.split_whitespace().collect::<String>();
    if matches!(
        compact.as_str(),
        "result==result" | "value==value" | "item==item" | "state==state"
    ) {
        return Some("the claim is tautological");
    }

    let normalized = normalize_contract_text(&lower);
    if normalized.is_empty() {
        return None;
    }

    if normalized.starts_with("todo")
        || normalized.starts_with("tbd")
        || normalized.starts_with("fix later")
    {
        return Some("the claim is still a placeholder");
    }

    if matches!(
        normalized.as_str(),
        "true"
            | "always"
            | "always true"
            | "ok"
            | "okay"
            | "works"
            | "it works"
            | "valid"
            | "correct"
            | "safe"
            | "secure"
            | "fast"
            | "optimized"
            | "good"
            | "good enough"
            | "must work"
            | "everything works"
    ) {
        return Some("the claim is too generic to test or prove");
    }

    None
}

fn normalize_contract_text(text: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_space = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch);
            previous_was_space = false;
        } else if !previous_was_space {
            normalized.push(' ');
            previous_was_space = true;
        }
    }
    normalized.trim().to_string()
}

pub fn normalize_coverage(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn coverage_key(text: &str) -> String {
    canonical_coverage_tokens(text).join(" ")
}

fn canonical_coverage_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch.to_ascii_lowercase());
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

fn suggested_evidence_name(task_name: &str, kind: &str, text: &str) -> String {
    let label = match kind {
        "security_property" => "proves",
        "trust_boundary" => "reviews",
        _ => "documents",
    };
    let summary = text
        .split_whitespace()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ");
    if summary.is_empty() {
        format!("{task_name} {label}")
    } else {
        format!("{task_name} {label} {summary}")
    }
}

fn suggested_test_name(task_name: &str, kind: &str, text: &str) -> String {
    let label = match kind {
        "precondition" => "requires",
        "postcondition" => "ensures",
        "edge_case" => "handles",
        "declared_test" => "covers",
        _ => "covers",
    };
    let summary = text
        .split_whitespace()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ");
    if summary.is_empty() {
        format!("{task_name} {label}")
    } else {
        format!("{task_name} {label} {summary}")
    }
}
#[cfg(test)]
mod tests {
    use super::{
        TestCoverage, coverage_key, coverage_match_kind, linked_test_count, normalize_coverage,
    };
    use crate::ast::SectionLine;
    use crate::diagnostic::Span;

    #[test]
    fn graph_rejects_corrupt_occurrence_projection_before_serialization() {
        let source =
            include_str!("../fixtures/diagnostics/session_ap_parser_resolver_precedence_fail.hum");
        let parsed = crate::parser::parse_source(
            "fixtures/diagnostics/session_ap_parser_resolver_precedence_fail.hum",
            source,
        );
        let checked = crate::check::check_file_with_occurrences(&parsed.file);
        let mut source_occurrences = parsed.diagnostic_occurrences.clone();
        source_occurrences
            .extend_owned(&checked.diagnostic_occurrences)
            .expect("source authority");
        let mut diagnostics = parsed.diagnostics;
        diagnostics.extend(checked.diagnostics);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let mut transport = crate::profile_check::diagnostic_transport_from_source(
            &program,
            &diagnostics,
            &source_occurrences,
        )
        .expect("profile-owned graph transport");
        super::validate_diagnostic_occurrence_projection(
            transport.authoritative(),
            transport.graph_projection(),
        )
        .expect("canonical graph projection");

        let authoritative = transport.authoritative().clone();
        let canonical = transport.graph_projection().clone();
        transport
            .graph_projection_mut_for_test()
            .prior_blockers_mut_for_test()
            .pop();
        assert!(
            super::validate_diagnostic_occurrence_projection(
                &authoritative,
                transport.graph_projection(),
            )
            .is_err()
        );
        let mut substituted = canonical.clone();
        substituted.prior_blockers_mut_for_test()[0]
            .relationship_route
            .push("substituted".to_string());
        assert!(
            super::validate_diagnostic_occurrence_projection(&authoritative, &substituted).is_err()
        );

        let mut duplicate = canonical.clone();
        let first = duplicate.prior_blockers_mut_for_test()[0].clone();
        duplicate.prior_blockers_mut_for_test().push(first.clone());
        assert!(
            super::validate_diagnostic_occurrence_projection(&authoritative, &duplicate).is_err()
        );

        let mut reordered = canonical.clone();
        if reordered.prior_blockers_mut_for_test().len() > 1 {
            reordered.prior_blockers_mut_for_test().swap(0, 1);
            assert!(
                super::validate_diagnostic_occurrence_projection(&authoritative, &reordered)
                    .is_err()
            );
        }

        let mut extra = canonical;
        extra.prior_blockers_mut_for_test().push(first);
        assert!(super::validate_diagnostic_occurrence_projection(&authoritative, &extra).is_err());
    }

    #[test]
    fn coverage_key_normalizes_case_and_punctuation_only() {
        assert_eq!(
            coverage_key("Add_Task NEEDS: title is not empty."),
            "add_task needs title is not empty"
        );
        assert_eq!(
            coverage_key("Add_Task REQUIRES: title is non-empty."),
            "add_task requires title is non empty"
        );
    }

    #[test]
    fn coverage_matching_distinguishes_exact_canonical_and_synonym_misses() {
        let modifiers = Vec::new();
        let exact_line = SectionLine {
            text: "add_task needs title is not empty".to_string(),
            span: Span::new("demo.hum", 1, 1),
        };
        let canonical_line = SectionLine {
            text: "Add_Task NEEDS: title is not empty.".to_string(),
            span: Span::new("demo.hum", 2, 1),
        };
        let synonym_line = SectionLine {
            text: "Add_Task REQUIRES: title is non-empty.".to_string(),
            span: Span::new("demo.hum", 3, 1),
        };
        let exact = TestCoverage {
            test_name: "exact",
            modifiers: &modifiers,
            line: &exact_line,
            covers: normalize_coverage(&exact_line.text),
            coverage_key: coverage_key(&exact_line.text),
        };
        let canonical = TestCoverage {
            test_name: "canonical",
            modifiers: &modifiers,
            line: &canonical_line,
            covers: normalize_coverage(&canonical_line.text),
            coverage_key: coverage_key(&canonical_line.text),
        };
        let synonym = TestCoverage {
            test_name: "synonym",
            modifiers: &modifiers,
            line: &synonym_line,
            covers: normalize_coverage(&synonym_line.text),
            coverage_key: coverage_key(&synonym_line.text),
        };

        assert_eq!(
            coverage_match_kind("add_task needs title is not empty", &exact),
            Some("exact")
        );
        assert_eq!(
            coverage_match_kind("add_task needs title is not empty", &canonical),
            Some("canonical")
        );
        assert_eq!(
            coverage_match_kind("add_task needs title is not empty", &synonym),
            None
        );
        assert_eq!(
            linked_test_count(
                "add_task needs title is not empty",
                &[exact, canonical, synonym]
            ),
            2
        );
    }
}
