use std::collections::BTreeMap;

use crate::ast::{Item, Program, Task};
use crate::graph::{
    TestCoverage, TestObligation, collect_test_coverages, linked_test_count, test_obligations,
};

pub fn program_to_test_skeletons(program: &Program) -> String {
    let test_coverages = collect_test_coverages(program);
    let mut out = String::new();
    let mut used_names = BTreeMap::new();

    for file in &program.files {
        write_item_skeletons(&mut out, &file.items, &test_coverages, &mut used_names);
    }

    out
}

fn write_item_skeletons(
    out: &mut String,
    items: &[Item],
    test_coverages: &[TestCoverage<'_>],
    used_names: &mut BTreeMap<String, usize>,
) {
    for item in items {
        match item {
            Item::App(app) => write_item_skeletons(out, &app.items, test_coverages, used_names),
            Item::Task(task) => write_task_skeletons(out, task, test_coverages, used_names),
            _ => {}
        }
    }
}

fn write_task_skeletons(
    out: &mut String,
    task: &Task,
    test_coverages: &[TestCoverage<'_>],
    used_names: &mut BTreeMap<String, usize>,
) {
    for obligation in test_obligations(task) {
        if linked_test_count(&obligation.covers, test_coverages) == 0 {
            write_obligation_skeleton(out, &obligation, used_names);
        }
    }
}

fn write_obligation_skeleton(
    out: &mut String,
    obligation: &TestObligation<'_>,
    used_names: &mut BTreeMap<String, usize>,
) {
    if !out.is_empty() {
        out.push('\n');
    }

    let test_name = unique_test_name(&sanitize_test_name(&obligation.suggested_test), used_names);
    out.push_str(&format!("test {test_name} {{\n"));
    out.push_str("  why:\n");
    out.push_str(&format!(
        "    generated from {} obligation at {}:{}\n\n",
        sanitize_section_text(obligation.source_section),
        sanitize_section_text(&obligation.line.span.file),
        obligation.line.span.line
    ));
    out.push_str("  covers:\n");
    out.push_str(&format!(
        "    {}\n\n",
        sanitize_section_text(&obligation.covers)
    ));
    out.push_str("  does:\n");
    out.push_str("    pending\n");
    out.push_str("}\n");
}

fn unique_test_name(name: &str, used_names: &mut BTreeMap<String, usize>) -> String {
    let count = used_names.entry(name.to_string()).or_insert(0);
    *count += 1;
    if *count == 1 {
        name.to_string()
    } else {
        format!("{name} {}", *count)
    }
}

fn sanitize_test_name(text: &str) -> String {
    let sanitized = text
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '_' | '-') {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>();
    normalize_generated_line(&sanitized).unwrap_or_else(|| "generated obligation test".to_string())
}

fn sanitize_section_text(text: &str) -> String {
    let sanitized = text
        .chars()
        .map(|ch| match ch {
            '{' | '}' => ' ',
            ch if ch.is_control() => ' ',
            ch => ch,
        })
        .collect::<String>();
    normalize_generated_line(&sanitized).unwrap_or_else(|| "unspecified".to_string())
}

fn normalize_generated_line(text: &str) -> Option<String> {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::program_to_test_skeletons;
    use crate::ast::Program;
    use crate::parser::parse_source;

    #[test]
    fn emits_only_unlinked_test_obligation_skeletons() {
        let source = r#"task add task(title: Text) -> Task {
  why:
    save a task

  needs:
    title is not empty

  ensures:
    new task is saved

  does:
    return task
}

test add task saves nonempty title property {
  why:
    prove saving behavior

  covers:
    add task ensures new task is saved

  does:
    expect task saved
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let skeletons = program_to_test_skeletons(&program);

        assert!(skeletons.contains("test add task requires title is not empty"));
        assert!(skeletons.contains("add task needs title is not empty"));
        assert!(!skeletons.contains("add task ensures new task is saved"));
    }

    #[test]
    fn generated_skeletons_parse_as_hum_tests() {
        let source = r#"task save weird title(title: Text) -> Task {
  why:
    save a task

  watch for:
    title may include braces { or }

  does:
    return task
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let skeletons = program_to_test_skeletons(&program);
        let parsed_skeletons = parse_source("generated.hum", &skeletons);

        assert!(parsed_skeletons.diagnostics.is_empty());
        assert!(skeletons.contains("test save weird title handles title may include"));
    }
}
