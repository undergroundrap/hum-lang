use crate::ast::{Item, Program, Task};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::graph::{
    TestCoverage, collect_test_coverages, coverage_key, coverage_match_kind, evidence_obligations,
};
use crate::version;

pub const EVIDENCE_REPORT_SCHEMA: &str = "hum.evidence.v0";

struct EvidenceReport<'a> {
    rows: Vec<EvidenceRow<'a>>,
    files: usize,
    tasks: usize,
    errors: usize,
    warnings: usize,
}

struct EvidenceRow<'a> {
    task_name: &'a str,
    id: String,
    kind: &'static str,
    blame: &'static str,
    source_section: &'static str,
    text: &'a str,
    span: &'a Span,
    covers: String,
    coverage_key: String,
    suggested_evidence: String,
    verification_status: &'static str,
    linked_evidence: Vec<LinkedEvidence>,
}

struct LinkedEvidence {
    test_name: String,
    modifiers: Vec<String>,
    covers: String,
    coverage_key: String,
    match_kind: &'static str,
    span: Span,
}

pub fn evidence_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let linked = report.linked_count();
    let unverified = report.unverified_count();
    let mut out = String::new();
    out.push_str(&format!("Hum evidence report ({EVIDENCE_REPORT_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!(
        "summary: files={} tasks={} evidence_obligations={} linked={} unverified={} errors={} warnings={}\n",
        report.files,
        report.tasks,
        report.rows.len(),
        linked,
        unverified,
        report.errors,
        report.warnings
    ));

    if report.rows.is_empty() {
        out.push_str("evidence_obligations: none\n");
        return out;
    }

    out.push_str("evidence_obligations:\n");
    for row in &report.rows {
        out.push_str(&format!(
            "  {}:{}:{} [{}] {}\n",
            row.span.file, row.span.line, row.span.column, row.verification_status, row.covers
        ));
        out.push_str(&format!(
            "    source: {} {} -> {}\n",
            row.task_name, row.source_section, row.text
        ));
        out.push_str(&format!(
            "    suggested_evidence: {}\n",
            row.suggested_evidence
        ));
        if row.linked_evidence.is_empty() {
            out.push_str("    linked_evidence: none\n");
        } else {
            out.push_str("    linked_evidence:\n");
            for evidence in &row.linked_evidence {
                out.push_str(&format!(
                    "      test {} ({}) at {}:{}:{} covers {}\n",
                    evidence.test_name,
                    evidence.match_kind,
                    evidence.span.file,
                    evidence.span.line,
                    evidence.span.column,
                    evidence.covers
                ));
            }
        }
    }

    out
}

pub fn evidence_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", EVIDENCE_REPORT_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_summary(&mut out, &report, 2, true);
    push_rows(&mut out, &report.rows, 2, false);
    out.push_str("}\n");
    out
}

fn build_report<'a>(program: &'a Program, diagnostics: &'a [Diagnostic]) -> EvidenceReport<'a> {
    let test_coverages = collect_test_coverages(program);
    let mut rows = Vec::new();
    let mut tasks = 0;

    for file in &program.files {
        collect_rows_from_items(&file.items, &test_coverages, &mut rows, &mut tasks);
    }

    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len().saturating_sub(errors);

    EvidenceReport {
        rows,
        files: program.files.len(),
        tasks,
        errors,
        warnings,
    }
}

fn collect_rows_from_items<'a>(
    items: &'a [Item],
    test_coverages: &[TestCoverage<'a>],
    rows: &mut Vec<EvidenceRow<'a>>,
    tasks: &mut usize,
) {
    for item in items {
        match item {
            Item::App(app) => collect_rows_from_items(&app.items, test_coverages, rows, tasks),
            Item::Task(task) => {
                *tasks += 1;
                collect_rows_from_task(task, test_coverages, rows);
            }
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn collect_rows_from_task<'a>(
    task: &'a Task,
    test_coverages: &[TestCoverage<'a>],
    rows: &mut Vec<EvidenceRow<'a>>,
) {
    for obligation in evidence_obligations(task) {
        let linked_evidence = linked_evidence_for(&obligation.covers, test_coverages);
        let verification_status = if linked_evidence.is_empty() {
            "unverified"
        } else {
            "linked"
        };
        let coverage_key = coverage_key(&obligation.covers);
        rows.push(EvidenceRow {
            task_name: &task.name,
            id: obligation.id,
            kind: obligation.kind,
            blame: obligation.blame,
            source_section: obligation.source_section,
            text: &obligation.line.text,
            span: &obligation.line.span,
            covers: obligation.covers,
            coverage_key,
            suggested_evidence: obligation.suggested_evidence,
            verification_status,
            linked_evidence,
        });
    }
}

fn linked_evidence_for<'a>(
    covers: &str,
    test_coverages: &[TestCoverage<'a>],
) -> Vec<LinkedEvidence> {
    test_coverages
        .iter()
        .filter_map(|coverage| {
            coverage_match_kind(covers, coverage).map(|match_kind| LinkedEvidence {
                test_name: coverage.test_name.to_string(),
                modifiers: coverage.modifiers.to_vec(),
                covers: coverage.covers.clone(),
                coverage_key: coverage.coverage_key.clone(),
                match_kind,
                span: coverage.line.span.clone(),
            })
        })
        .collect()
}

impl EvidenceReport<'_> {
    fn linked_count(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.verification_status == "linked")
            .count()
    }

    fn unverified_count(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.verification_status == "unverified")
            .count()
    }
}

fn push_summary(out: &mut String, report: &EvidenceReport<'_>, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"tasks\": {}, \"evidence_obligations\": {}, \"linked\": {}, \"unverified\": {}, \"errors\": {}, \"warnings\": {}",
        report.files,
        report.tasks,
        report.rows.len(),
        report.linked_count(),
        report.unverified_count(),
        report.errors,
        report.warnings
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_rows(out: &mut String, rows: &[EvidenceRow<'_>], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"evidence_obligations\": [\n");
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_row(out, row, indent + 2);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_row(out: &mut String, row: &EvidenceRow<'_>, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &row.id, true);
    push_string_field(out, indent + 2, "task", row.task_name, true);
    push_string_field(out, indent + 2, "kind", row.kind, true);
    push_string_field(out, indent + 2, "blame", row.blame, true);
    push_string_field(out, indent + 2, "source_section", row.source_section, true);
    push_string_field(out, indent + 2, "text", row.text, true);
    push_span_field(out, indent + 2, "span", row.span, true);
    push_string_field(out, indent + 2, "covers", &row.covers, true);
    push_string_field(out, indent + 2, "coverage_key", &row.coverage_key, true);
    push_string_field(
        out,
        indent + 2,
        "suggested_evidence",
        &row.suggested_evidence,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "verification_status",
        row.verification_status,
        true,
    );
    push_linked_evidence(out, &row.linked_evidence, indent + 2, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_linked_evidence(out: &mut String, evidence: &[LinkedEvidence], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"linked_evidence\": [\n");
    for (index, item) in evidence.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "kind", "test", true);
        push_string_field(out, indent + 4, "name", &item.test_name, true);
        push_string_array_field(out, indent + 4, "modifiers", &item.modifiers, true);
        push_string_field(out, indent + 4, "covers", &item.covers, true);
        push_string_field(out, indent + 4, "coverage_key", &item.coverage_key, true);
        push_string_field(out, indent + 4, "match", item.match_kind, true);
        push_span_field(out, indent + 4, "span", &item.span, false);
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_span_field(out: &mut String, indent: usize, key: &str, span: &Span, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": {");
    out.push_str("\"file\": ");
    push_json_string(out, &span.file);
    out.push_str(&format!(
        ", \"line\": {}, \"column\": {}",
        span.line, span.column
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_string_array_field(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[String],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(out, value);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_string_field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_json_string(out, value);
    push_comma_newline(out, comma);
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn push_comma_newline(out: &mut String, comma: bool) {
    if comma {
        out.push(',');
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use crate::ast::Program;
    use crate::parser::parse_source;

    use super::{evidence_json, evidence_text};

    #[test]
    fn text_report_lists_linked_and_unverified_evidence() {
        let program = demo_program();
        let text = evidence_text(&program, &[]);

        assert!(text.contains("Hum evidence report (hum.evidence.v0)"));
        assert!(text.contains("evidence_obligations=2 linked=1 unverified=1"));
        assert!(text.contains("[linked] add task protects user data remains private"));
        assert!(text.contains("[unverified] add task trusts local profile storage"));
        assert!(text.contains("linked_evidence:"));
    }

    #[test]
    fn json_report_is_machine_readable() {
        let program = demo_program();
        let json = evidence_json(&program, &[]);

        assert!(json.contains("\"schema\": \"hum.evidence.v0\""));
        assert!(json.contains("\"evidence_obligations\": 2"));
        assert!(json.contains("\"linked\": 1"));
        assert!(json.contains("\"unverified\": 1"));
        assert!(json.contains("\"verification_status\": \"linked\""));
        assert!(json.contains("\"verification_status\": \"unverified\""));
        assert!(json.contains("\"linked_evidence\": ["));
        assert!(json.contains("\"kind\": \"test\""));
    }

    fn demo_program() -> Program {
        let source = r#"task add task(title: Text) -> Task {
  why:
    save a task

  protects:
    user data remains private

  trusts:
    local profile storage

  does:
    return task
}

test add task privacy unit {
  covers:
    add task protects user data remains private

  does:
    expect privacy evidence exists
}
"#;
        let parsed = parse_source("demo.hum", source);
        Program {
            files: vec![parsed.file],
        }
    }
}
