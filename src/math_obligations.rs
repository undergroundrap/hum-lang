use crate::ast::{Item, Program, Section, Task};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::graph::is_meaningful_line_text;
use crate::node_id;
use crate::version;

pub const MATH_OBLIGATIONS_REPORT_SCHEMA: &str = "hum.math_obligations.v0";
pub const MATH_OBLIGATION_SCHEMA: &str = "hum.math_obligation.v0";

pub struct ObligationFile {
    pub file_name: String,
    pub json: String,
}

struct MathObligationsReport {
    files: usize,
    tasks: usize,
    errors: usize,
    warnings: usize,
    obligations: Vec<MathObligation>,
}

struct MathObligation {
    obligation_id: String,
    obligation_kind: &'static str,
    graph_node_id: String,
    source_span: Span,
    end_line: usize,
    end_column: usize,
    claim_text: String,
    formal_expression: String,
    formal_variable_name: &'static str,
    expected_bound: &'static str,
    assumption_id: String,
    assumption_text: String,
    allowed_effects: Vec<&'static str>,
    program_shape: ProgramShape,
}

struct ProgramShape {
    primary: &'static str,
    access_pattern: &'static str,
    mutation: &'static str,
    io: &'static str,
    concurrency: &'static str,
    hardware: &'static str,
}

pub fn math_obligations_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!(
        "Hum math obligations ({MATH_OBLIGATIONS_REPORT_SCHEMA})\n"
    ));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!(
        "summary: files={} tasks={} obligations={} errors={} warnings={}\n",
        report.files,
        report.tasks,
        report.obligations.len(),
        report.errors,
        report.warnings
    ));
    out.push_str("emitted_schema: ");
    out.push_str(MATH_OBLIGATION_SCHEMA);
    out.push('\n');

    if report.obligations.is_empty() {
        out.push_str("math_obligations: none\n");
        return out;
    }

    out.push_str("math_obligations:\n");
    for obligation in &report.obligations {
        out.push_str(&format!(
            "  {}:{}:{} [{}] {}\n",
            obligation.source_span.file,
            obligation.source_span.line,
            obligation.source_span.column,
            obligation.obligation_kind,
            obligation.claim_text
        ));
        out.push_str(&format!(
            "    id: {}\n    graph_node_id: {}\n    confidence_requested: evidence_only\n",
            obligation.obligation_id, obligation.graph_node_id
        ));
    }

    out
}

pub fn math_obligations_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", MATH_OBLIGATIONS_REPORT_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_summary(&mut out, &report, 2, true);
    push_indent(&mut out, 2);
    out.push_str("\"obligations\": [\n");
    for (index, obligation) in report.obligations.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_obligation(&mut out, obligation, 4);
    }
    out.push('\n');
    push_indent(&mut out, 2);
    out.push_str("]\n");
    out.push_str("}\n");
    out
}

pub fn obligation_files(program: &Program) -> Vec<ObligationFile> {
    let report = build_report(program, &[]);
    report
        .obligations
        .iter()
        .map(|obligation| ObligationFile {
            file_name: format!("{}.json", obligation.obligation_id),
            json: individual_obligation_json(obligation),
        })
        .collect()
}

fn individual_obligation_json(obligation: &MathObligation) -> String {
    let mut out = String::new();
    push_obligation(&mut out, obligation, 0);
    out.push('\n');
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> MathObligationsReport {
    let mut obligations = Vec::new();
    let mut tasks = 0;

    for file in &program.files {
        collect_obligations_from_items(&file.items, &mut obligations, &mut tasks);
    }

    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len().saturating_sub(errors);

    MathObligationsReport {
        files: program.files.len(),
        tasks,
        errors,
        warnings,
        obligations,
    }
}

fn collect_obligations_from_items(
    items: &[Item],
    obligations: &mut Vec<MathObligation>,
    tasks: &mut usize,
) {
    for item in items {
        match item {
            Item::App(app) => collect_obligations_from_items(&app.items, obligations, tasks),
            Item::Task(task) => {
                *tasks += 1;
                obligations.extend(task_math_obligations(task));
            }
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn task_math_obligations(task: &Task) -> Vec<MathObligation> {
    let mut obligations = Vec::new();
    for line in allocation_free_claim_lines(task) {
        obligations.push(allocation_free_obligation(task, line));
    }
    obligations
}

fn allocation_free_claim_lines(task: &Task) -> Vec<&crate::ast::SectionLine> {
    let mut lines = Vec::new();

    if let Some(allocates) = task.section("allocates") {
        for line in meaningful_lines(allocates) {
            if is_allocation_free_text(&line.text) {
                lines.push(line);
            }
        }
    }

    if let Some(cost) = task.section("cost") {
        for line in meaningful_lines(cost) {
            if line
                .text
                .strip_prefix("allocates:")
                .is_some_and(is_allocation_free_text)
            {
                lines.push(line);
            }
        }
    }

    lines
}

fn allocation_free_obligation(task: &Task, line: &crate::ast::SectionLine) -> MathObligation {
    let graph_node_id = task_graph_node_id(task);
    let obligation_id = obligation_id(
        "allocation_freedom",
        &task.name,
        line.span.line,
        line.span.column,
    );
    let assumption_id = assumption_id("declared_no_allocation", &task.name, line.span.line);
    let allowed_effects = allowed_effects(task);
    let program_shape = program_shape(task, &allowed_effects);
    let end_column = line
        .span
        .column
        .saturating_add(line.text.chars().count().max(1));

    MathObligation {
        obligation_id,
        obligation_kind: "allocation_freedom",
        graph_node_id,
        source_span: portable_span(&line.span),
        end_line: line.span.line,
        end_column,
        claim_text: format!(
            "task `{}` declares no heap allocation: {}",
            task.name, line.text
        ),
        formal_expression: format!("heap_allocations({}) == 0", symbolic_task_name(&task.name)),
        formal_variable_name: "heap_allocations",
        expected_bound: "0",
        assumption_id,
        assumption_text:
            "Hum source declares no allocation; Milestone 0 exports this as evidence, not proof."
                .to_string(),
        allowed_effects,
        program_shape,
    }
}

fn meaningful_lines(section: &Section) -> impl Iterator<Item = &crate::ast::SectionLine> {
    section
        .lines
        .iter()
        .filter(|line| is_meaningful_line_text(&line.text))
}

fn is_allocation_free_text(text: &str) -> bool {
    let normalized = text
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    matches!(
        normalized.as_str(),
        "nothing" | "none" | "no heap allocation" | "no allocation" | "zero allocations"
    )
}

fn allowed_effects(task: &Task) -> Vec<&'static str> {
    let mut effects = Vec::new();
    let uses_lines = task.section("uses").map(section_text).unwrap_or_default();
    let changes_lines = task
        .section("changes")
        .map(section_text)
        .unwrap_or_default();
    let does_lines = task.section("does").map(section_text).unwrap_or_default();
    let combined = format!("{uses_lines}\n{changes_lines}\n{does_lines}").to_ascii_lowercase();

    if !uses_lines.is_empty() || !does_lines.is_empty() {
        push_unique_effect(&mut effects, "read_memory");
    }
    if !changes_lines.is_empty() || combined.contains("change ") || combined.contains("set ") {
        push_unique_effect(&mut effects, "write_local_memory");
    }
    if has_any_token(&combined, &["network", "socket", "http"]) {
        push_unique_effect(&mut effects, "network");
    }
    if has_any_token(&combined, &["random"]) {
        push_unique_effect(&mut effects, "randomness");
    }
    if has_any_token(&combined, &["clock", "time"]) {
        push_unique_effect(&mut effects, "time");
    }
    if has_any_token(&combined, &["screen", "file", "io"]) {
        push_unique_effect(&mut effects, "io");
    }
    if has_any_token(&combined, &["concurrent", "thread", "async"]) {
        push_unique_effect(&mut effects, "concurrency");
    }
    if has_any_token(&combined, &["unsafe", "pointer"]) {
        push_unique_effect(&mut effects, "unsafe_pointer");
    }
    if has_any_token(&combined, &["volatile", "hardware"]) {
        push_unique_effect(&mut effects, "volatile_hardware");
    }

    if effects.is_empty() {
        effects.push("none");
    }

    effects
}

fn portable_span(span: &Span) -> Span {
    Span {
        file: span.file.replace('\\', "/"),
        line: span.line,
        column: span.column,
    }
}

fn has_any_token(text: &str, needles: &[&str]) -> bool {
    text.split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|token| !token.is_empty() && needles.contains(&token))
}
fn push_unique_effect(effects: &mut Vec<&'static str>, effect: &'static str) {
    if !effects.contains(&effect) {
        effects.push(effect);
    }
}

fn section_text(section: &Section) -> String {
    meaningful_lines(section)
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn program_shape(task: &Task, allowed_effects: &[&str]) -> ProgramShape {
    let does_text = task
        .section("does")
        .map(section_text)
        .unwrap_or_default()
        .to_ascii_lowercase();

    let has_loop = does_text.contains("for each ") || does_text.contains("while ");
    let has_pointer = allowed_effects.contains(&"unsafe_pointer");
    let has_hardware = allowed_effects.contains(&"volatile_hardware");
    let has_concurrency = allowed_effects.contains(&"concurrency");
    let has_io = allowed_effects
        .iter()
        .any(|effect| matches!(*effect, "io" | "network" | "randomness" | "time"));
    let has_mutation = allowed_effects.contains(&"write_local_memory");

    let primary = if has_hardware {
        "hardware_specific"
    } else if has_concurrency {
        "concurrent"
    } else if has_pointer {
        "pointer_mutation_heavy"
    } else if has_io {
        "io_effectful"
    } else {
        "streaming_sequential"
    };

    ProgramShape {
        primary,
        access_pattern: if has_loop { "sequential" } else { "none" },
        mutation: if has_pointer {
            "pointer_aliasing_unknown"
        } else if has_mutation {
            "local"
        } else {
            "none"
        },
        io: if has_io { "declared" } else { "none" },
        concurrency: if has_concurrency {
            "structured"
        } else {
            "single_threaded"
        },
        hardware: if has_hardware {
            "volatile_or_device_bound"
        } else {
            "abstract"
        },
    }
}

fn task_graph_node_id(task: &Task) -> String {
    node_id::span("item", &task.span, &format!("{} {}", "task", task.name))
}

fn obligation_id(kind: &str, task_name: &str, line: usize, column: usize) -> String {
    prefixed_id("hum_obl", &format!("{kind}_{task_name}_{line}_{column}"))
}

fn assumption_id(kind: &str, task_name: &str, line: usize) -> String {
    prefixed_id("assume", &format!("{kind}_{task_name}_{line}"))
}

fn prefixed_id(prefix: &str, text: &str) -> String {
    let mut body = snake_identifier(text);
    if body.len() < 4 {
        body.push_str("_task");
    }
    if body.len() > 96 {
        body.truncate(96);
        body = body.trim_matches('_').to_string();
    }
    format!("{prefix}_{body}")
}

fn symbolic_task_name(name: &str) -> String {
    let symbol = snake_identifier(name);
    if symbol.is_empty() {
        "task".to_string()
    } else {
        symbol
    }
}

fn snake_identifier(text: &str) -> String {
    let mut out = String::new();
    let mut previous_was_separator = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !out.is_empty() {
            out.push('_');
            previous_was_separator = true;
        }
    }
    out.trim_matches('_').to_string()
}

fn push_summary(out: &mut String, report: &MathObligationsReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"tasks\": {}, \"obligations\": {}, \"errors\": {}, \"warnings\": {}, \"emitted_schema\": ",
        report.files,
        report.tasks,
        report.obligations.len(),
        report.errors,
        report.warnings
    ));
    push_json_string(out, MATH_OBLIGATION_SCHEMA);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_obligation(out: &mut String, obligation: &MathObligation, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(
        out,
        indent + 2,
        "schema_version",
        MATH_OBLIGATION_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "obligation_id",
        &obligation.obligation_id,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "obligation_kind",
        obligation.obligation_kind,
        true,
    );
    push_source_span(out, obligation, indent + 2, true);
    push_string_field(
        out,
        indent + 2,
        "graph_node_id",
        &obligation.graph_node_id,
        true,
    );
    push_string_field(out, indent + 2, "claim_text", &obligation.claim_text, true);
    push_normalized_formal_claim(out, obligation, indent + 2, true);
    push_assumptions(out, obligation, indent + 2, true);
    push_string_array_field(
        out,
        indent + 2,
        "allowed_effects",
        &obligation.allowed_effects,
        true,
    );
    push_program_shape(out, &obligation.program_shape, indent + 2, true);
    push_resource_model(out, indent + 2, true);
    push_string_field(
        out,
        indent + 2,
        "confidence_requested",
        "evidence_only",
        true,
    );
    push_timeout_budget(out, indent + 2, true);
    push_privacy(out, indent + 2, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_source_span(out: &mut String, obligation: &MathObligation, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"source_span\": {");
    out.push_str("\"file\": ");
    push_json_string(out, &obligation.source_span.file);
    out.push_str(&format!(
        ", \"start_line\": {}, \"start_column\": {}, \"end_line\": {}, \"end_column\": {}",
        obligation.source_span.line,
        obligation.source_span.column,
        obligation.end_line,
        obligation.end_column
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_normalized_formal_claim(
    out: &mut String,
    obligation: &MathObligation,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"normalized_formal_claim\": {\n");
    push_string_field(
        out,
        indent + 2,
        "representation",
        "hum_static_claim_v0",
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "expression",
        &obligation.formal_expression,
        true,
    );
    push_indent(out, indent + 2);
    out.push_str("\"variables\": [\n");
    push_indent(out, indent + 4);
    out.push_str("{\n");
    push_string_field(
        out,
        indent + 6,
        "name",
        obligation.formal_variable_name,
        true,
    );
    push_string_field(out, indent + 6, "sort", "nat", true);
    push_string_field(out, indent + 6, "unit", "allocations", false);
    push_indent(out, indent + 4);
    out.push_str("}\n");
    push_indent(out, indent + 2);
    out.push_str("],\n");
    push_string_field(
        out,
        indent + 2,
        "expected_bound",
        obligation.expected_bound,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_assumptions(out: &mut String, obligation: &MathObligation, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"assumptions\": [\n");
    push_indent(out, indent + 2);
    out.push_str("{\n");
    push_string_field(
        out,
        indent + 4,
        "assumption_id",
        &obligation.assumption_id,
        true,
    );
    push_string_field(out, indent + 4, "evidence_class", "compiler_fact", true);
    push_string_field(out, indent + 4, "text", &obligation.assumption_text, true);
    push_string_field(
        out,
        indent + 4,
        "source_ref",
        &format!("hum_graph:{}", obligation.graph_node_id),
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("}\n");
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_program_shape(out: &mut String, shape: &ProgramShape, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"program_shape\": {\n");
    push_string_field(out, indent + 2, "primary", shape.primary, true);
    push_string_field(
        out,
        indent + 2,
        "access_pattern",
        shape.access_pattern,
        true,
    );
    push_string_field(out, indent + 2, "mutation", shape.mutation, true);
    push_string_field(out, indent + 2, "io", shape.io, true);
    push_string_field(out, indent + 2, "concurrency", shape.concurrency, true);
    push_string_field(out, indent + 2, "hardware", shape.hardware, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_resource_model(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"resource_model\": {\n");
    push_string_field(
        out,
        indent + 2,
        "machine_model",
        "hum_abstract_machine_v0",
        true,
    );
    push_indent(out, indent + 2);
    out.push_str("\"word_bits\": 64,\n");
    push_string_field(out, indent + 2, "integer_overflow", "checked", true);
    push_string_field(out, indent + 2, "allocation_model", "none", true);
    push_string_field(out, indent + 2, "peak_memory_unit", "bytes", false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_timeout_budget(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"timeout_budget\": {");
    out.push_str(
        "\"timeout_ms\": 1000, \"max_solver_memory_bytes\": 67108864, \"max_steps\": 10000",
    );
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_privacy(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"privacy\": {");
    out.push_str("\"local_first\": true, ");
    out.push_str("\"network_access\": \"none\", ");
    out.push_str("\"cloud_access\": \"none\", ");
    out.push_str("\"telemetry\": \"none\"");
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_string_array_field(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[&str],
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

    use super::{math_obligations_json, obligation_files};

    #[test]
    fn exports_allocation_freedom_obligation_for_allocates_nothing() {
        let program = demo_program("allocates: nothing");
        let json = math_obligations_json(&program, &[]);

        assert!(json.contains("\"schema\": \"hum.math_obligations.v0\""));
        assert!(json.contains("\"schema_version\": \"hum.math_obligation.v0\""));
        assert!(json.contains("\"obligation_kind\": \"allocation_freedom\""));
        assert!(json.contains("\"confidence_requested\": \"evidence_only\""));
        assert!(json.contains("\"allowed_effects\": [\"read_memory\"]"));
        assert!(json.contains("\"local_first\": true"));
        assert!(json.contains("\"source_ref\": \"hum_graph:item:demo.hum:1:1:task-show-tasks\""));
    }

    #[test]
    fn writes_validation_shaped_individual_obligation_files() {
        let program = demo_program("allocates: nothing");
        let files = obligation_files(&program);

        assert_eq!(files.len(), 1);
        assert!(
            files[0]
                .file_name
                .starts_with("hum_obl_allocation_freedom_show_tasks")
        );
        assert!(
            files[0]
                .json
                .starts_with("{\n  \"schema_version\": \"hum.math_obligation.v0\"")
        );
        assert!(
            !files[0]
                .json
                .contains("\"schema\": \"hum.math_obligations.v0\"")
        );
    }

    #[test]
    fn does_not_export_weaker_allocation_claims_as_math_obligations() {
        let program = demo_program("allocates: one task");
        let json = math_obligations_json(&program, &[]);

        assert!(json.contains("\"obligations\": [\n\n  ]"));
    }

    fn demo_program(allocation_line: &str) -> Program {
        let source = format!(
            r#"task show tasks(tasks: Tasks) {{
  why:
    list visible tasks

  uses:
    tasks

  needs:
    task store can be read

  cost:
    time: O(tasks)
    space: O(1)
    {allocation_line}
    check: compile

  does:
    for each task in tasks {{
      show task
    }}
}}
"#
        );
        let parsed = parse_source("demo.hum", &source);
        Program {
            files: vec![parsed.file],
        }
    }
}
