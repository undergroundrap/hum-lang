use crate::ast::{App, Item, Program, Section, Store, Task, Test, TypeDef};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::graph::is_meaningful_line_text;
use crate::ir_contract;
use crate::node_id;
use crate::version;

pub const IR_READINESS_SCHEMA: &str = "hum.ir_readiness.v0";

struct IrReadinessReport {
    files: usize,
    items: usize,
    tasks: usize,
    tests: usize,
    errors: usize,
    warnings: usize,
    candidates: Vec<LoweringCandidate>,
}

struct LoweringCandidate {
    id: String,
    kind: &'static str,
    name: String,
    graph_node_id: String,
    span: Span,
    status: &'static str,
    current_layer: &'static str,
    target_layer: &'static str,
    facts_available: Vec<&'static str>,
    missing_passes: Vec<&'static str>,
    blocking_reasons: Vec<&'static str>,
    section_names: Vec<String>,
}

struct PassStatus {
    name: &'static str,
    status: &'static str,
    source: &'static str,
}

const CURRENT_LAYER: &str = "surface_hum_and_semantic_graph";
const TARGET_LAYER: &str = "core_hum_then_hum_ir";

const PASS_STATUSES: &[PassStatus] = &[
    PassStatus {
        name: "parse",
        status: "current",
        source: "hum parser",
    },
    PassStatus {
        name: "semantic_graph_build",
        status: "current",
        source: "hum graph",
    },
    PassStatus {
        name: "core_lowering",
        status: "not_implemented",
        source: "hum.ir_contract.v0",
    },
    PassStatus {
        name: "type_check",
        status: "not_implemented",
        source: "hum.ir_contract.v0",
    },
    PassStatus {
        name: "effect_check",
        status: "not_implemented",
        source: "hum.ir_contract.v0",
    },
    PassStatus {
        name: "ownership_alias_check",
        status: "not_implemented",
        source: "hum.ir_contract.v0",
    },
    PassStatus {
        name: "allocation_resource_check",
        status: "not_implemented",
        source: "hum.ir_contract.v0",
    },
    PassStatus {
        name: "contract_evidence_linking",
        status: "report_available_not_ir_pass",
        source: "hum evidence",
    },
    PassStatus {
        name: "profile_check",
        status: "not_implemented",
        source: "hum.ir_contract.v0",
    },
    PassStatus {
        name: "ir_verify",
        status: "not_implemented",
        source: "hum.ir_contract.v0",
    },
];

const MISSING_IR_PASSES: &[&str] = &[
    "core_lowering",
    "type_check",
    "effect_check",
    "ownership_alias_check",
    "allocation_resource_check",
    "profile_check",
    "ir_verify",
];

pub fn ir_readiness_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let blocked = report.blocked_count();
    let mut out = String::new();
    out.push_str(&format!("Hum IR readiness ({IR_READINESS_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!(
        "summary: files={} items={} tasks={} tests={} lowering_candidates={} ready_for_ir=0 blocked={} errors={} warnings={}\n",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.candidates.len(),
        blocked,
        report.errors,
        report.warnings
    ));
    out.push_str(&format!(
        "ir_contract_schema: {}\n",
        ir_contract::IR_CONTRACT_SCHEMA
    ));
    out.push_str("pass_status:\n");
    for pass in PASS_STATUSES {
        out.push_str(&format!(
            "  {} [{}]: {}\n",
            pass.name, pass.status, pass.source
        ));
    }

    if report.candidates.is_empty() {
        out.push_str("lowering_candidates: none\n");
        return out;
    }

    out.push_str("lowering_candidates:\n");
    for candidate in &report.candidates {
        out.push_str(&format!(
            "  {}:{}:{} [{}] {} `{}` -> {}\n",
            candidate.span.file,
            candidate.span.line,
            candidate.span.column,
            candidate.status,
            candidate.kind,
            candidate.name,
            candidate.target_layer
        ));
        out.push_str(&format!("    graph_node_id: {}\n", candidate.graph_node_id));
        out.push_str(&format!(
            "    facts_available: {}\n",
            candidate.facts_available.join(", ")
        ));
        out.push_str(&format!(
            "    missing_passes: {}\n",
            candidate.missing_passes.join(", ")
        ));
        out.push_str(&format!(
            "    blocking_reasons: {}\n",
            candidate.blocking_reasons.join(", ")
        ));
    }

    out
}

pub fn ir_readiness_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", IR_READINESS_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(
        &mut out,
        2,
        "ir_contract_schema",
        ir_contract::IR_CONTRACT_SCHEMA,
        true,
    );
    push_summary(&mut out, &report, 2, true);
    push_pass_status(&mut out, 2, true);
    push_candidates(&mut out, &report.candidates, 2, true);
    push_string_array(
        &mut out,
        2,
        "non_goals_v0",
        &[
            "no IR emission",
            "no executable semantics",
            "no backend lowering",
            "no optimizer claim",
            "no proof of type or memory safety",
        ],
        false,
    );
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> IrReadinessReport {
    let mut candidates = Vec::new();
    for file in &program.files {
        collect_candidates_from_items(&file.items, diagnostics, &mut candidates);
    }

    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len().saturating_sub(errors);
    let tasks = candidates
        .iter()
        .filter(|candidate| candidate.kind == "task")
        .count();
    let tests = candidates
        .iter()
        .filter(|candidate| candidate.kind == "test")
        .count();

    IrReadinessReport {
        files: program.files.len(),
        items: candidates.len(),
        tasks,
        tests,
        errors,
        warnings,
        candidates,
    }
}

fn collect_candidates_from_items(
    items: &[Item],
    diagnostics: &[Diagnostic],
    candidates: &mut Vec<LoweringCandidate>,
) {
    for item in items {
        candidates.push(lowering_candidate(item, diagnostics));
        if let Item::App(app) = item {
            collect_candidates_from_items(&app.items, diagnostics, candidates);
        }
    }
}

fn lowering_candidate(item: &Item, diagnostics: &[Diagnostic]) -> LoweringCandidate {
    let graph_node_id = node_id::span(
        "item",
        item.span(),
        &format!("{} {}", item.kind(), item.name()),
    );
    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);
    let blocking_reasons = blocking_reasons(has_errors);
    let section_names = item_sections(item)
        .iter()
        .map(|section| section.name.clone())
        .collect::<Vec<_>>();

    LoweringCandidate {
        id: readiness_id(item),
        kind: item.kind(),
        name: item.name().to_string(),
        graph_node_id,
        span: portable_span(item.span()),
        status: if has_errors {
            "blocked_by_source_errors"
        } else {
            "blocked_before_core_lowering"
        },
        current_layer: CURRENT_LAYER,
        target_layer: TARGET_LAYER,
        facts_available: facts_available(item),
        missing_passes: MISSING_IR_PASSES.to_vec(),
        blocking_reasons,
        section_names,
    }
}

fn facts_available(item: &Item) -> Vec<&'static str> {
    let mut facts = vec![
        "source_span",
        "semantic_graph_node_id",
        "item_kind",
        "item_name",
    ];

    let sections = item_sections(item);
    if !sections.is_empty() {
        facts.push("source_sections");
    }
    if sections.iter().any(has_meaningful_lines) {
        facts.push("section_line_spans");
    }

    match item {
        Item::App(app) => add_app_facts(app, &mut facts),
        Item::Type(type_def) => add_type_facts(type_def, &mut facts),
        Item::Store(store) => add_store_facts(store, &mut facts),
        Item::Task(task) => add_task_facts(task, &mut facts),
        Item::Test(test) => add_test_facts(test, &mut facts),
    }

    facts
}

fn add_app_facts(app: &App, facts: &mut Vec<&'static str>) {
    if !app.items.is_empty() {
        facts.push("nested_item_scope");
    }
}

fn add_type_facts(type_def: &TypeDef, facts: &mut Vec<&'static str>) {
    if !type_def.fields.is_empty() {
        facts.push("field_shapes");
    }
}

fn add_store_facts(store: &Store, facts: &mut Vec<&'static str>) {
    if !store.ty.trim().is_empty() {
        facts.push("store_type_annotation");
    }
}

fn add_task_facts(task: &Task, facts: &mut Vec<&'static str>) {
    if !task.params.is_empty() {
        facts.push("signature_params");
    }
    if task.result.is_some() {
        facts.push("signature_result");
    }
    add_section_family_facts(&task.sections, facts);
}

fn add_test_facts(test: &Test, facts: &mut Vec<&'static str>) {
    if !test.params.is_empty() {
        facts.push("signature_params");
    }
    if !test.modifiers.is_empty() {
        facts.push("test_modifiers");
    }
    if test.section("covers").is_some() {
        facts.push("test_coverage_hints");
    }
    add_section_family_facts(&test.sections, facts);
}

fn add_section_family_facts(sections: &[Section], facts: &mut Vec<&'static str>) {
    if has_any_section(sections, &["uses", "changes"]) {
        facts.push("effect_hints");
    }
    if has_any_section(
        sections,
        &[
            "needs",
            "ensures",
            "keeps",
            "protects",
            "trusts",
            "watch for",
        ],
    ) {
        facts.push("contract_hints");
    }
    if has_any_section(
        sections,
        &["cost", "allocates", "avoids", "tradeoffs", "optimizes"],
    ) {
        facts.push("resource_hints");
    }
    if has_any_section(sections, &["does"]) {
        facts.push("body_text_captured");
    }
}

fn has_any_section(sections: &[Section], names: &[&str]) -> bool {
    sections
        .iter()
        .any(|section| names.contains(&section.name.as_str()) && has_meaningful_lines(section))
}

fn has_meaningful_lines(section: &Section) -> bool {
    section
        .lines
        .iter()
        .any(|line| is_meaningful_line_text(&line.text))
}

fn item_sections(item: &Item) -> &[Section] {
    match item {
        Item::App(item) => &item.sections,
        Item::Type(item) => &item.sections,
        Item::Store(item) => &item.sections,
        Item::Task(item) => &item.sections,
        Item::Test(item) => &item.sections,
    }
}

fn blocking_reasons(has_errors: bool) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if has_errors {
        reasons.push("source_diagnostics_include_errors");
    }
    reasons.push("core_lowering_not_implemented");
    reasons.push("type_check_not_implemented");
    reasons.push("effect_check_not_implemented");
    reasons.push("ir_verify_not_implemented");
    reasons
}

impl IrReadinessReport {
    fn blocked_count(&self) -> usize {
        self.candidates
            .iter()
            .filter(|candidate| candidate.status.starts_with("blocked"))
            .count()
    }
}

fn readiness_id(item: &Item) -> String {
    prefixed_id(
        "hum_ir_ready",
        &format!(
            "{}_{}_{}_{}",
            item.kind(),
            item.name(),
            item.span().line,
            item.span().column
        ),
    )
}

fn prefixed_id(prefix: &str, text: &str) -> String {
    let mut body = snake_identifier(text);
    if body.len() < 4 {
        body.push_str("_item");
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

fn portable_span(span: &Span) -> Span {
    Span {
        file: span.file.replace('\\', "/"),
        line: span.line,
        column: span.column,
    }
}

fn push_summary(out: &mut String, report: &IrReadinessReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"lowering_candidates\": {}, \"ready_for_ir\": 0, \"blocked\": {}, \"errors\": {}, \"warnings\": {}",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.candidates.len(),
        report.blocked_count(),
        report.errors,
        report.warnings
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_pass_status(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"pass_status\": [\n");
    for (index, pass) in PASS_STATUSES.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "name", pass.name, true);
        push_string_field(out, indent + 4, "status", pass.status, true);
        push_string_field(out, indent + 4, "source", pass.source, false);
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_candidates(out: &mut String, candidates: &[LoweringCandidate], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"lowering_candidates\": [\n");
    for (index, candidate) in candidates.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_candidate(out, candidate, indent + 2);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_candidate(out: &mut String, candidate: &LoweringCandidate, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &candidate.id, true);
    push_string_field(out, indent + 2, "kind", candidate.kind, true);
    push_string_field(out, indent + 2, "name", &candidate.name, true);
    push_string_field(
        out,
        indent + 2,
        "graph_node_id",
        &candidate.graph_node_id,
        true,
    );
    push_span_field(out, indent + 2, "source_span", &candidate.span, true);
    push_string_field(out, indent + 2, "status", candidate.status, true);
    push_string_field(
        out,
        indent + 2,
        "current_layer",
        candidate.current_layer,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "target_layer",
        candidate.target_layer,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "facts_available",
        &candidate.facts_available,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "missing_passes",
        &candidate.missing_passes,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "blocking_reasons",
        &candidate.blocking_reasons,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "source_sections",
        &candidate.section_names,
        false,
    );
    push_indent(out, indent);
    out.push('}');
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

fn push_string_array(out: &mut String, indent: usize, key: &str, values: &[&str], comma: bool) {
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

fn push_owned_string_array(
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

    use super::{ir_readiness_json, ir_readiness_text};

    #[test]
    fn text_report_lists_lowering_candidates_without_emitting_ir() {
        let program = demo_program();
        let text = ir_readiness_text(&program, &[]);

        assert!(text.contains("Hum IR readiness (hum.ir_readiness.v0)"));
        assert!(text.contains("lowering_candidates=3 ready_for_ir=0 blocked=3"));
        assert!(text.contains("pass_status:"));
        assert!(text.contains("core_lowering [not_implemented]"));
        assert!(text.contains("task `add task`"));
        assert!(text.contains("missing_passes: core_lowering"));
    }

    #[test]
    fn json_report_lists_facts_and_blockers() {
        let program = demo_program();
        let json = ir_readiness_json(&program, &[]);

        assert!(json.contains("\"schema\": \"hum.ir_readiness.v0\""));
        assert!(json.contains("\"ir_contract_schema\": \"hum.ir_contract.v0\""));
        assert!(json.contains("\"ready_for_ir\": 0"));
        assert!(json.contains("\"status\": \"blocked_before_core_lowering\""));
        assert!(json.contains("\"name\": \"semantic_graph_build\""));
        assert!(json.contains("\"status\": \"report_available_not_ir_pass\""));
        assert!(json.contains("\"effect_hints\""));
        assert!(json.contains("\"contract_hints\""));
        assert!(json.contains("\"body_text_captured\""));
        assert!(json.contains("\"no IR emission\""));
    }

    fn demo_program() -> Program {
        let source = r#"type Task {
  title: Text
}

task add task(title: Text) -> Task {
  why:
    save a task

  uses:
    tasks

  changes:
    tasks

  ensures:
    task is visible

  does:
    save task
    return task
}

test add task unit {
  covers:
    add task ensures task is visible

  does:
    expect task is visible
}
"#;
        let parsed = parse_source("demo.hum", source);
        Program {
            files: vec![parsed.file],
        }
    }
}
