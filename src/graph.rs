use crate::ast::{Item, Program, Section, SectionLine, Task, Test};
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
    pub source_section: &'static str,
    pub line: &'a SectionLine,
    pub covers: String,
    pub suggested_test: String,
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
    for &(section_name, kind) in syntax::TEST_OBLIGATION_SECTIONS {
        for section in task
            .sections
            .iter()
            .filter(|section| section.name == section_name)
        {
            for line in meaningful_lines(section) {
                obligations.push(TestObligation {
                    id: node_id::span(
                        "obligation",
                        &line.span,
                        &format!("{} {} {}", task.name, section_name, line.text),
                    ),
                    kind,
                    source_section: section_name,
                    line,
                    covers: format!("{} {} {}", task.name, section_name, line.text),
                    suggested_test: suggested_test_name(&task.name, kind, &line.text),
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
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else {
            push_canonical_token(&mut tokens, &current);
            current.clear();
        }
    }
    push_canonical_token(&mut tokens, &current);

    tokens
}

fn push_canonical_token(tokens: &mut Vec<String>, token: &str) {
    if token.is_empty() {
        return;
    }

    match token {
        "a" | "an" | "the" | "of" | "to" | "for" | "that" | "is" | "are" | "be" | "being"
        | "was" | "were" | "may" | "might" | "can" | "could" => {}
        "need" | "needed" | "needing" | "require" | "requires" | "required" | "requiring"
        | "precondition" | "preconditions" => tokens.push("needs".to_string()),
        "ensure" | "ensured" | "ensuring" | "postcondition" | "postconditions" | "promise"
        | "promises" => tokens.push("ensures".to_string()),
        "handle" | "handles" | "handled" | "handling" | "risk" | "risks" | "edge" => {
            tokens.push("watch".to_string());
        }
        "case" | "cases" => {}
        "non" => tokens.push("not".to_string()),
        "nonempty" => {
            tokens.push("not".to_string());
            tokens.push("empty".to_string());
        }
        "cannot" => tokens.push("not".to_string()),
        _ => tokens.push(token.to_string()),
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
    fn coverage_key_normalizes_punctuation_case_and_small_aliases() {
        assert_eq!(
            coverage_key("Add Task REQUIRES: the title is non-empty."),
            "add task needs title not empty"
        );
    }

    #[test]
    fn coverage_matching_distinguishes_exact_and_canonical_links() {
        let modifiers = Vec::new();
        let exact_line = SectionLine {
            text: "add task needs title is not empty".to_string(),
            span: Span::new("demo.hum", 1, 1),
        };
        let canonical_line = SectionLine {
            text: "Add Task REQUIRES: title is non-empty.".to_string(),
            span: Span::new("demo.hum", 2, 1),
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

        assert_eq!(
            coverage_match_kind("add task needs title is not empty", &exact),
            Some("exact")
        );
        assert_eq!(
            coverage_match_kind("add task needs title is not empty", &canonical),
            Some("canonical")
        );
        assert_eq!(
            linked_test_count("add task needs title is not empty", &[exact, canonical]),
            2
        );
    }
}
