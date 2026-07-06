use crate::ast::{Item, Program, Section, SectionLine, Task, Test};

pub struct TestCoverage<'a> {
    pub test_name: &'a str,
    pub modifiers: &'a [String],
    pub line: &'a SectionLine,
    pub covers: String,
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
            });
        }
    }
}

pub fn linked_test_count(covers: &str, test_coverages: &[TestCoverage<'_>]) -> usize {
    let normalized_covers = normalize_coverage(covers);
    test_coverages
        .iter()
        .filter(|coverage| coverage.covers == normalized_covers)
        .count()
}

pub fn test_obligations(task: &Task) -> Vec<TestObligation<'_>> {
    let mut obligations = Vec::new();
    for (section_name, kind) in [
        ("needs", "precondition"),
        ("ensures", "postcondition"),
        ("watch for", "edge_case"),
        ("tests", "declared_test"),
    ] {
        for section in task
            .sections
            .iter()
            .filter(|section| section.name == section_name)
        {
            for line in meaningful_lines(section) {
                obligations.push(TestObligation {
                    id: format!(
                        "{}:task:{}:{}:{}",
                        line.span.file, task.name, section_name, line.span.line
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
    section.lines.iter().filter(|line| {
        let text = line.text.trim();
        !text.is_empty() && !text.starts_with('#') && !text.starts_with("//")
    })
}

pub fn normalize_coverage(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
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
