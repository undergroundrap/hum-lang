use crate::ast::{Item, Program, Section, Task};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::graph::is_meaningful_line_text;
use crate::node_id;
use crate::version;

pub const RESOURCE_REPORT_SCHEMA: &str = "hum.resource_report.v0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceReportSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub files: usize,
    pub tasks: usize,
    pub resource_claims: usize,
    pub allocation_claims: usize,
    pub allocation_free_claims: usize,
    pub errors: usize,
    pub warnings: usize,
    pub proof_ready: usize,
    pub benchmark_ready: usize,
}

struct ResourceReport {
    files: usize,
    tasks: usize,
    errors: usize,
    warnings: usize,
    claims: Vec<ResourceClaim>,
}

struct ResourceClaim {
    id: String,
    task_name: String,
    graph_node_id: String,
    source_section: String,
    claim_kind: &'static str,
    resource_dimension: &'static str,
    text: String,
    span: Span,
    end_line: usize,
    end_column: usize,
    normalized_claim: String,
    related_math_obligation_kind: Option<&'static str>,
}

pub fn resource_report_summary(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> ResourceReportSummary {
    let report = build_report(program, diagnostics);
    ResourceReportSummary {
        schema: RESOURCE_REPORT_SCHEMA,
        status: if report.errors > 0 {
            "source_errors_v0"
        } else {
            "declared_resource_claims_reported_v0"
        },
        files: report.files,
        tasks: report.tasks,
        resource_claims: report.claims.len(),
        allocation_claims: report
            .claims
            .iter()
            .filter(|claim| claim.claim_kind == "allocation_behavior")
            .count(),
        allocation_free_claims: report
            .claims
            .iter()
            .filter(|claim| claim.related_math_obligation_kind == Some("allocation_freedom"))
            .count(),
        errors: report.errors,
        warnings: report.warnings,
        proof_ready: 0,
        benchmark_ready: 0,
    }
}

pub fn resource_report_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum resource report ({RESOURCE_REPORT_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!(
        "summary: files={} tasks={} resource_claims={} errors={} warnings={}\n",
        report.files,
        report.tasks,
        report.claims.len(),
        report.errors,
        report.warnings
    ));

    if report.claims.is_empty() {
        out.push_str("resource_claims: none\n");
        return out;
    }

    out.push_str("resource_claims:\n");
    for claim in &report.claims {
        out.push_str(&format!(
            "  {}:{}:{} [{}/{}] task `{}` {}: {}\n",
            claim.span.file,
            claim.span.line,
            claim.span.column,
            claim.claim_kind,
            claim.resource_dimension,
            claim.task_name,
            claim.source_section,
            claim.text
        ));
        out.push_str("    verification_status: declared\n");
        out.push_str("    proof_status: not_proven\n");
        out.push_str("    benchmark_status: not_measured\n");
        if let Some(kind) = claim.related_math_obligation_kind {
            out.push_str(&format!("    related_math_obligation_kind: {kind}\n"));
        }
    }

    out
}

pub fn resource_report_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", RESOURCE_REPORT_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_summary(&mut out, &report, 2, true);
    push_claims(&mut out, &report.claims, 2, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> ResourceReport {
    let mut claims = Vec::new();
    let mut tasks = 0;

    for file in &program.files {
        collect_claims_from_items(&file.items, &mut claims, &mut tasks);
    }

    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len().saturating_sub(errors);

    ResourceReport {
        files: program.files.len(),
        tasks,
        errors,
        warnings,
        claims,
    }
}

fn collect_claims_from_items(items: &[Item], claims: &mut Vec<ResourceClaim>, tasks: &mut usize) {
    for item in items {
        match item {
            Item::App(app) => collect_claims_from_items(&app.items, claims, tasks),
            Item::Task(task) => {
                *tasks += 1;
                collect_claims_from_task(task, claims);
            }
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn collect_claims_from_task(task: &Task, claims: &mut Vec<ResourceClaim>) {
    for section in &task.sections {
        if is_resource_section(&section.name) {
            for line in meaningful_lines(section) {
                claims.push(resource_claim(task, section, line));
            }
        }
    }
}

fn is_resource_section(name: &str) -> bool {
    matches!(
        name,
        "cost" | "allocates" | "avoids" | "tradeoffs" | "optimizes"
    )
}

fn resource_claim(task: &Task, section: &Section, line: &crate::ast::SectionLine) -> ResourceClaim {
    let classification = classify_claim(&section.name, &line.text);
    let graph_node_id = task_graph_node_id(task);
    let end_column = line
        .span
        .column
        .saturating_add(line.text.chars().count().max(1));
    let id = claim_id(
        classification.claim_kind,
        &task.name,
        &section.name,
        line.span.line,
        line.span.column,
    );

    ResourceClaim {
        id,
        task_name: task.name.clone(),
        graph_node_id,
        source_section: section.name.clone(),
        claim_kind: classification.claim_kind,
        resource_dimension: classification.resource_dimension,
        text: line.text.clone(),
        span: portable_span(&line.span),
        end_line: line.span.line,
        end_column,
        normalized_claim: normalized_claim(&section.name, &line.text),
        related_math_obligation_kind: classification.related_math_obligation_kind,
    }
}

struct ClaimClassification {
    claim_kind: &'static str,
    resource_dimension: &'static str,
    related_math_obligation_kind: Option<&'static str>,
}

fn classify_claim(section_name: &str, text: &str) -> ClaimClassification {
    let lower = text.trim().to_ascii_lowercase();
    let (claim_kind, resource_dimension) = match section_name {
        "allocates" => ("allocation_behavior", "allocation"),
        "avoids" => ("avoided_shape", "code_shape"),
        "tradeoffs" => ("accepted_tradeoff", "tradeoff"),
        "optimizes" => ("optimization_priority", "optimization"),
        "cost" if lower.starts_with("time:") => ("time_complexity", "time"),
        "cost" if lower.starts_with("space:") || lower.starts_with("memory:") => {
            ("space_complexity", "space")
        }
        "cost" if lower.starts_with("allocates:") => ("allocation_behavior", "allocation"),
        "cost" if lower.starts_with("check:") => ("check_strategy", "compile_time"),
        "cost" => ("cost_claim", "resource"),
        _ => ("resource_claim", "resource"),
    };

    ClaimClassification {
        claim_kind,
        resource_dimension,
        related_math_obligation_kind: related_math_obligation_kind(section_name, text),
    }
}

fn related_math_obligation_kind(section_name: &str, text: &str) -> Option<&'static str> {
    let candidate = if section_name == "cost" {
        text.trim().strip_prefix("allocates:").map(str::trim)
    } else if section_name == "allocates" {
        Some(text.trim())
    } else {
        None
    };

    candidate
        .filter(|value| is_allocation_free_text(value))
        .map(|_| "allocation_freedom")
}

fn normalized_claim(section_name: &str, text: &str) -> String {
    let trimmed = text.trim();
    if section_name == "cost"
        && let Some((key, value)) = trimmed.split_once(':')
    {
        return format!(
            "{}:{}",
            key.trim().to_ascii_lowercase(),
            value.split_whitespace().collect::<Vec<_>>().join(" ")
        );
    }

    trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
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

fn portable_span(span: &Span) -> Span {
    Span {
        file: span.file.replace('\\', "/"),
        line: span.line,
        column: span.column,
    }
}

fn task_graph_node_id(task: &Task) -> String {
    node_id::span("item", &task.span, &format!("{} {}", "task", task.name))
}

fn claim_id(kind: &str, task_name: &str, section_name: &str, line: usize, column: usize) -> String {
    prefixed_id(
        "hum_res",
        &format!("{kind}_{task_name}_{section_name}_{line}_{column}"),
    )
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

fn push_summary(out: &mut String, report: &ResourceReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"tasks\": {}, \"resource_claims\": {}, \"errors\": {}, \"warnings\": {}",
        report.files,
        report.tasks,
        report.claims.len(),
        report.errors,
        report.warnings
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_claims(out: &mut String, claims: &[ResourceClaim], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"resource_claims\": [\n");
    for (index, claim) in claims.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_claim(out, claim, indent + 2);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_claim(out: &mut String, claim: &ResourceClaim, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &claim.id, true);
    push_string_field(out, indent + 2, "task", &claim.task_name, true);
    push_string_field(out, indent + 2, "graph_node_id", &claim.graph_node_id, true);
    push_string_field(
        out,
        indent + 2,
        "source_section",
        &claim.source_section,
        true,
    );
    push_string_field(out, indent + 2, "claim_kind", claim.claim_kind, true);
    push_string_field(
        out,
        indent + 2,
        "resource_dimension",
        claim.resource_dimension,
        true,
    );
    push_string_field(out, indent + 2, "text", &claim.text, true);
    push_source_span(out, claim, indent + 2, true);
    push_normalized_claim(out, claim, indent + 2, true);
    push_string_field(out, indent + 2, "verification_status", "declared", true);
    push_string_field(out, indent + 2, "proof_status", "not_proven", true);
    push_string_field(out, indent + 2, "benchmark_status", "not_measured", true);
    push_optional_string_field(
        out,
        indent + 2,
        "related_math_obligation_kind",
        claim.related_math_obligation_kind,
        false,
    );
    push_indent(out, indent);
    out.push('}');
}

fn push_source_span(out: &mut String, claim: &ResourceClaim, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"source_span\": {");
    out.push_str("\"file\": ");
    push_json_string(out, &claim.span.file);
    out.push_str(&format!(
        ", \"start_line\": {}, \"start_column\": {}, \"end_line\": {}, \"end_column\": {}",
        claim.span.line, claim.span.column, claim.end_line, claim.end_column
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_normalized_claim(out: &mut String, claim: &ResourceClaim, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"normalized_claim\": {\n");
    push_string_field(
        out,
        indent + 2,
        "representation",
        "hum_resource_claim_v0",
        true,
    );
    push_string_field(out, indent + 2, "dimension", claim.resource_dimension, true);
    push_string_field(out, indent + 2, "claim", &claim.normalized_claim, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_optional_string_field(
    out: &mut String,
    indent: usize,
    key: &str,
    value: Option<&str>,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    match value {
        Some(value) => push_json_string(out, value),
        None => out.push_str("null"),
    }
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

    use super::{resource_report_json, resource_report_summary, resource_report_text};

    #[test]
    fn text_report_lists_resource_claims_without_proof_claims() {
        let program = demo_program();
        let text = resource_report_text(&program, &[]);

        assert!(text.contains("Hum resource report (hum.resource_report.v0)"));
        assert!(text.contains("resource_claims=6"));
        assert!(text.contains("[time_complexity/time]"));
        assert!(text.contains("verification_status: declared"));
        assert!(text.contains("proof_status: not_proven"));
        assert!(text.contains("benchmark_status: not_measured"));
        assert!(text.contains("related_math_obligation_kind: allocation_freedom"));
    }

    #[test]
    fn json_report_classifies_resource_sections() {
        let program = demo_program();
        let json = resource_report_json(&program, &[]);
        let summary = resource_report_summary(&program, &[]);

        assert_eq!(summary.schema, "hum.resource_report.v0");
        assert_eq!(summary.status, "declared_resource_claims_reported_v0");
        assert_eq!(summary.resource_claims, 6);
        assert_eq!(summary.allocation_claims, 1);
        assert_eq!(summary.allocation_free_claims, 1);
        assert_eq!(summary.proof_ready, 0);
        assert_eq!(summary.benchmark_ready, 0);
        assert!(json.contains("\"schema\": \"hum.resource_report.v0\""));
        assert!(json.contains("\"resource_claims\": 6"));
        assert!(json.contains("\"claim_kind\": \"time_complexity\""));
        assert!(json.contains("\"claim_kind\": \"space_complexity\""));
        assert!(json.contains("\"claim_kind\": \"allocation_behavior\""));
        assert!(json.contains("\"claim_kind\": \"optimization_priority\""));
        assert!(json.contains("\"claim_kind\": \"avoided_shape\""));
        assert!(json.contains("\"claim_kind\": \"accepted_tradeoff\""));
        assert!(json.contains("\"verification_status\": \"declared\""));
        assert!(json.contains("\"proof_status\": \"not_proven\""));
        assert!(json.contains("\"benchmark_status\": \"not_measured\""));
        assert!(json.contains("\"related_math_obligation_kind\": \"allocation_freedom\""));
        assert!(json.contains("\"graph_node_id\": \"item:demo.hum:1:1:task-find-active-session\""));
    }

    fn demo_program() -> Program {
        let source = r#"task find active session(sessions: Sessions) -> Session {
  why:
    find the live session

  cost:
    time: O(sessions)
    space: O(1)
    allocates: nothing

  optimizes:
    p99 lookup latency

  avoids:
    unbounded heap growth

  tradeoffs:
    recompute stale session status instead of caching it

  does:
    for each session in sessions {
      return session
    }
}
"#;
        let parsed = parse_source("demo.hum", source);
        Program {
            files: vec![parsed.file],
        }
    }
}
