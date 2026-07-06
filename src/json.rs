use crate::ast::{Item, Program, Section, Task};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::graph::{
    TestCoverage, collect_test_coverages, is_meaningful_line_text, linked_test_count,
    normalize_coverage, test_obligations,
};

pub fn program_to_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len().saturating_sub(errors);
    let test_coverages = collect_test_coverages(program);

    out.push_str("{\n");
    out.push_str("  \"schema\": \"hum.semantic_graph.v0\",\n");
    out.push_str("  \"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"errors\": {}, \"warnings\": {}",
        program.files.len(),
        count_items(program),
        count_kind(program, "task"),
        count_kind(program, "test"),
        errors,
        warnings
    ));
    out.push_str("},\n");

    out.push_str("  \"files\": [\n");
    for (file_index, file) in program.files.iter().enumerate() {
        comma_line(&mut out, file_index, 4);
        out.push_str("    {\n");
        out.push_str(&format!("      \"path\": {},\n", quote(&file.path)));
        out.push_str(&format!(
            "      \"module\": {},\n",
            file.module
                .as_ref()
                .map_or("null".to_string(), |module| quote(module))
        ));
        out.push_str("      \"items\": [\n");
        write_items(&mut out, &file.items, 0, 8, &test_coverages);
        out.push_str("\n      ]\n");
        out.push_str("    }");
    }
    out.push_str("\n  ],\n");

    out.push_str("  \"diagnostics\": [\n");
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        comma_line(&mut out, index, 4);
        write_diagnostic(&mut out, diagnostic, 4);
    }
    out.push_str("\n  ]\n");
    out.push_str("}\n");
    out
}

fn write_items(
    out: &mut String,
    items: &[Item],
    start_index: usize,
    indent: usize,
    test_coverages: &[TestCoverage<'_>],
) {
    for (offset, item) in items.iter().enumerate() {
        comma_line(out, start_index + offset, indent);
        write_item(out, item, indent, test_coverages);
    }
}

fn write_item(out: &mut String, item: &Item, indent: usize, test_coverages: &[TestCoverage<'_>]) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}{{\n"));
    out.push_str(&format!("{pad}  \"kind\": {},\n", quote(item.kind())));
    out.push_str(&format!("{pad}  \"name\": {},\n", quote(item.name())));
    out.push_str(&format!("{pad}  \"span\": "));
    write_span(out, item.span());

    match item {
        Item::App(app) => {
            out.push_str(",\n");
            write_sections_field(out, &app.sections, indent + 2);
            out.push_str(",\n");
            out.push_str(&format!("{pad}  \"items\": [\n"));
            write_items(out, &app.items, 0, indent + 4, test_coverages);
            out.push_str(&format!("\n{pad}  ]\n"));
        }
        Item::Type(type_def) => {
            out.push_str(",\n");
            out.push_str(&format!("{pad}  \"fields\": ["));
            for (index, field) in type_def.fields.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!(
                    "{{\"name\": {}, \"type\": {}, \"span\": ",
                    quote(&field.name),
                    quote(&field.ty)
                ));
                write_span(out, &field.span);
                out.push('}');
            }
            out.push_str("],\n");
            write_sections_field(out, &type_def.sections, indent + 2);
            out.push('\n');
        }
        Item::Store(store) => {
            out.push_str(",\n");
            out.push_str(&format!("{pad}  \"type\": {},\n", quote(&store.ty)));
            write_sections_field(out, &store.sections, indent + 2);
            out.push('\n');
        }
        Item::Task(task) => {
            out.push_str(",\n");
            out.push_str(&format!("{pad}  \"params\": ["));
            write_params(out, &task.params);
            out.push_str("],\n");
            out.push_str(&format!(
                "{pad}  \"result\": {},\n",
                task.result
                    .as_ref()
                    .map_or("null".to_string(), |result| quote(result))
            ));
            write_sections_field(out, &task.sections, indent + 2);
            out.push_str(",\n");
            write_test_obligations_field(out, task, indent + 2, test_coverages);
            out.push('\n');
        }
        Item::Test(test) => {
            out.push_str(",\n");
            out.push_str(&format!("{pad}  \"params\": ["));
            write_params(out, &test.params);
            out.push_str("],\n");
            out.push_str(&format!("{pad}  \"modifiers\": ["));
            for (index, modifier) in test.modifiers.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str(&quote(modifier));
            }
            out.push_str("],\n");
            write_sections_field(out, &test.sections, indent + 2);
            out.push('\n');
        }
    }
    out.push_str(&format!("{pad}}}"));
}

fn write_params(out: &mut String, params: &[crate::ast::Param]) {
    for (index, param) in params.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"name\": {}, \"type\": {}, \"span\": ",
            quote(&param.name),
            quote(&param.ty)
        ));
        write_span(out, &param.span);
        out.push('}');
    }
}

fn write_sections_field(out: &mut String, sections: &[Section], indent: usize) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}\"sections\": ["));
    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("{{\"name\": {}, \"span\": ", quote(&section.name)));
        write_span(out, &section.span);
        out.push_str(&format!(
            ", \"lines\": {}, \"line_items\": [",
            section_line_count(section)
        ));
        write_section_line_items(out, section);
        out.push_str("]}");
    }
    out.push(']');
}

fn section_line_count(section: &Section) -> usize {
    section
        .lines
        .iter()
        .filter(|line| !line.text.is_empty())
        .count()
}

fn write_section_line_items(out: &mut String, section: &Section) {
    for (index, line) in section
        .lines
        .iter()
        .filter(|line| !line.text.is_empty())
        .enumerate()
    {
        if index > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str(&format!("\"text\": {}, ", quote(&line.text)));
        out.push_str("\"span\": ");
        write_span(out, &line.span);
        out.push_str(&format!(
            ", \"meaningful\": {}",
            if is_meaningful_line_text(&line.text) {
                "true"
            } else {
                "false"
            }
        ));
        out.push('}');
    }
}

fn write_test_obligations_field(
    out: &mut String,
    task: &Task,
    indent: usize,
    test_coverages: &[TestCoverage<'_>],
) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}\"test_obligations\": ["));
    let obligations = test_obligations(task);
    for (index, obligation) in obligations.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        let linked_test_count = linked_test_count(&obligation.covers, test_coverages);
        out.push('{');
        out.push_str(&format!("\"id\": {}, ", quote(&obligation.id)));
        out.push_str(&format!("\"kind\": {}, ", quote(obligation.kind)));
        out.push_str(&format!(
            "\"source_section\": {}, ",
            quote(obligation.source_section)
        ));
        out.push_str(&format!("\"text\": {}, ", quote(&obligation.line.text)));
        out.push_str("\"span\": ");
        write_span(out, &obligation.line.span);
        out.push_str(&format!(", \"covers\": {}", quote(&obligation.covers)));
        out.push_str(&format!(
            ", \"suggested_test\": {}",
            quote(&obligation.suggested_test)
        ));
        out.push_str(&format!(
            ", \"link_status\": {}",
            quote(if linked_test_count == 0 {
                "unlinked"
            } else {
                "linked"
            })
        ));
        out.push_str(", \"linked_tests\": [");
        write_linked_tests(out, &obligation.covers, test_coverages);
        out.push(']');
        out.push('}');
    }
    out.push(']');
}

fn write_linked_tests(out: &mut String, covers: &str, test_coverages: &[TestCoverage<'_>]) {
    let normalized_covers = normalize_coverage(covers);
    for (written, coverage) in test_coverages
        .iter()
        .filter(|coverage| coverage.covers == normalized_covers)
        .enumerate()
    {
        if written > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str(&format!("\"name\": {}, ", quote(coverage.test_name)));
        out.push_str("\"modifiers\": [");
        for (index, modifier) in coverage.modifiers.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(&quote(modifier));
        }
        out.push_str("], ");
        out.push_str(&format!("\"covers\": {}, ", quote(&coverage.covers)));
        out.push_str("\"span\": ");
        write_span(out, &coverage.line.span);
        out.push('}');
    }
}

fn write_diagnostic(out: &mut String, diagnostic: &Diagnostic, indent: usize) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}{{"));
    out.push_str(&format!(
        "\"code\": {}, \"title\": {}, \"severity\": {}, \"message\": {}",
        quote(diagnostic.code.as_str()),
        quote(diagnostic.code.title()),
        quote(diagnostic.severity.as_str()),
        quote(&diagnostic.message)
    ));
    if let Some(span) = &diagnostic.span {
        out.push_str(", \"span\": ");
        write_span(out, span);
    }
    if let Some(help) = &diagnostic.help {
        out.push_str(&format!(", \"help\": {}", quote(help)));
    }
    out.push('}');
}

fn write_span(out: &mut String, span: &Span) {
    out.push_str(&format!(
        "{{\"file\": {}, \"line\": {}, \"column\": {}}}",
        quote(&span.file),
        span.line,
        span.column
    ));
}

fn comma_line(out: &mut String, index: usize, indent: usize) {
    if index > 0 {
        out.push_str(",\n");
    }
    out.push_str(&" ".repeat(indent));
}

fn count_items(program: &Program) -> usize {
    program
        .files
        .iter()
        .map(|file| count_items_slice(&file.items))
        .sum()
}

fn count_items_slice(items: &[Item]) -> usize {
    items
        .iter()
        .map(|item| match item {
            Item::App(app) => 1 + count_items_slice(&app.items),
            _ => 1,
        })
        .sum()
}

fn count_kind(program: &Program, kind: &str) -> usize {
    program
        .files
        .iter()
        .map(|file| count_kind_slice(&file.items, kind))
        .sum()
}

fn count_kind_slice(items: &[Item], kind: &str) -> usize {
    items
        .iter()
        .map(|item| {
            let nested = match item {
                Item::App(app) => count_kind_slice(&app.items, kind),
                _ => 0,
            };
            usize::from(item.kind() == kind) + nested
        })
        .sum()
}

fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}
#[cfg(test)]
mod tests {
    use super::program_to_json;
    use crate::ast::Program;
    use crate::diagnostic::{Diagnostic, DiagnosticCode, Span};
    use crate::parser::parse_source;

    #[test]
    fn diagnostic_json_includes_code_and_title() {
        let diagnostic = Diagnostic::warning(
            DiagnosticCode::SECTION_OUT_OF_ORDER,
            "section is out of order",
            Some(Span::new("bad.hum", 5, 1)),
        );
        let json = program_to_json(&Program::default(), &[diagnostic]);

        assert!(json.contains("\"code\": \"H0108\""));
        assert!(json.contains("\"title\": \"section out of order\""));
    }

    #[test]
    fn task_json_includes_test_obligations() {
        let source = r#"task add task(title: Text) -> Result Task, TaskError {
  why:
    save a task

  needs:
    title is not empty

  ensures:
    new task is saved

  watch for:
    title may be only spaces

  tests:
    empty title is rejected

  does:
    return task
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"test_obligations\""));
        assert!(json.contains("\"kind\": \"precondition\""));
        assert!(json.contains("\"kind\": \"postcondition\""));
        assert!(json.contains("\"kind\": \"edge_case\""));
        assert!(json.contains("\"kind\": \"declared_test\""));
        assert!(json.contains("\"suggested_test\": \"add task requires title is not empty\""));
    }

    #[test]
    fn task_obligations_link_to_covering_tests() {
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
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"link_status\": \"linked\""));
        assert!(json.contains("\"link_status\": \"unlinked\""));
        assert!(json.contains("\"linked_tests\": [{\"name\": \"add task saves nonempty title\""));
        assert!(json.contains("\"modifiers\": [\"property\"]"));
        assert!(json.contains("\"covers\": \"add task ensures new task is saved\""));
    }

    #[test]
    fn section_json_includes_line_items_with_spans() {
        let source = r#"task add task(title: Text) -> Task {
  why:
    save a task
    // explain later

  does:
    return task
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"name\": \"why\", \"span\": {\"file\": \"demo.hum\", \"line\": 2"));
        assert!(json.contains("\"line_items\": [{\"text\": \"save a task\", \"span\": {\"file\": \"demo.hum\", \"line\": 3"));
        assert!(json.contains("\"meaningful\": true"));
        assert!(json.contains("\"text\": \"// explain later\""));
        assert!(json.contains("\"meaningful\": false"));
    }
}
