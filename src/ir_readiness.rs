use crate::ast::{App, Item, Program, Section, Store, Task, Test, TypeDef};
use crate::core_body::{self, BodyGrammarReport, BodyStatement};
use crate::core_contract;
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::graph::is_meaningful_line_text;
use crate::ir_contract;
use crate::node_id;
use crate::resolve;
use crate::type_check;
use crate::version;

pub const IR_READINESS_SCHEMA: &str = "hum.ir_readiness.v0";

struct IrReadinessReport {
    files: usize,
    items: usize,
    tasks: usize,
    tests: usize,
    errors: usize,
    warnings: usize,
    resolve_summary: resolve::ResolveReadinessSummary,
    type_check_summary: type_check::TypeCheckSummary,
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
    body_grammar: Option<BodyGrammarReport>,
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
        name: "resolve",
        status: "checked_report_available",
        source: resolve::RESOLVE_REPORT_SCHEMA,
    },
    PassStatus {
        name: "body_grammar",
        status: core_body::CORE_BODY_GRAMMAR_STATUS,
        source: core_contract::CORE_CONTRACT_SCHEMA,
    },
    PassStatus {
        name: "core_lowering",
        status: "not_implemented",
        source: core_contract::CORE_CONTRACT_SCHEMA,
    },
    PassStatus {
        name: "type_check",
        status: "declaration_and_trivial_return_check_available",
        source: type_check::TYPE_CHECK_SCHEMA,
    },
    PassStatus {
        name: "effect_check",
        status: "not_implemented",
        source: core_contract::CORE_CONTRACT_SCHEMA,
    },
    PassStatus {
        name: "ownership_alias_check",
        status: "not_implemented",
        source: ir_contract::IR_CONTRACT_SCHEMA,
    },
    PassStatus {
        name: "allocation_resource_check",
        status: "not_implemented",
        source: core_contract::CORE_CONTRACT_SCHEMA,
    },
    PassStatus {
        name: "contract_evidence_linking",
        status: "report_available_not_ir_pass",
        source: "hum evidence",
    },
    PassStatus {
        name: "profile_check",
        status: "not_implemented",
        source: core_contract::CORE_CONTRACT_SCHEMA,
    },
    PassStatus {
        name: "ir_verify",
        status: "not_implemented",
        source: ir_contract::IR_CONTRACT_SCHEMA,
    },
];

const MISSING_IR_PASSES: &[&str] = &[
    "core_lowering",
    "full_type_check",
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
        "summary: files={} items={} tasks={} tests={} lowering_candidates={} ready_for_ir=0 blocked={} errors={} warnings={} body_grammar_candidates={} body_grammar_recognized_lines={} body_grammar_unsupported_lines={} resolver_status={} resolver_errors={} unresolved_references={} type_check_status={} type_errors={} unknown_type_references={} checked_returns={} rejected_returns={} unchecked_returns={}\n",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.candidates.len(),
        blocked,
        report.errors,
        report.warnings,
        report.body_grammar_candidates(),
        report.body_grammar_recognized_lines(),
        report.body_grammar_unsupported_lines(),
        report.resolve_summary.status,
        report.resolve_summary.resolver_errors,
        report.resolve_summary.unresolved_references,
        report.type_check_summary.status,
        report.type_check_summary.type_errors,
        report.type_check_summary.unknown_type_references,
        report.type_check_summary.checked_returns,
        report.type_check_summary.rejected_returns,
        report.type_check_summary.unchecked_returns
    ));
    out.push_str(&format!(
        "core_contract_schema: {}\n",
        core_contract::CORE_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "ir_contract_schema: {}\n",
        ir_contract::IR_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "resolver: schema={} status={} mode={} scopes={} definitions={} references={} resolved={} unresolved={} external={} duplicate_definitions={} mutable_place_errors={} resolver_errors={} resolver_warnings={}\n",
        report.resolve_summary.schema,
        report.resolve_summary.status,
        report.resolve_summary.mode,
        report.resolve_summary.scopes,
        report.resolve_summary.definitions,
        report.resolve_summary.references,
        report.resolve_summary.resolved_references,
        report.resolve_summary.unresolved_references,
        report.resolve_summary.external_references,
        report.resolve_summary.duplicate_definitions,
        report.resolve_summary.mutable_place_errors,
        report.resolve_summary.resolver_errors,
        report.resolve_summary.resolver_warnings
    ));
    out.push_str(&format!(
        "type_check: schema={} status={} mode={} checked_declarations={} rejected_declarations={} checked_returns={} rejected_returns={} unchecked_returns={} type_errors={} unknown_type_references={}\n",
        report.type_check_summary.schema,
        report.type_check_summary.status,
        report.type_check_summary.mode,
        report.type_check_summary.checked_declarations,
        report.type_check_summary.rejected_declarations,
        report.type_check_summary.checked_returns,
        report.type_check_summary.rejected_returns,
        report.type_check_summary.unchecked_returns,
        report.type_check_summary.type_errors,
        report.type_check_summary.unknown_type_references
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
        if let Some(body_grammar) = &candidate.body_grammar {
            out.push_str(&format!(
                "    body_grammar: {} meaningful_lines={} recognized_lines={} unsupported_lines={}\n",
                body_grammar.status,
                body_grammar.meaningful_lines,
                body_grammar.recognized_lines,
                body_grammar.unsupported_lines
            ));
        }
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
        "core_contract_schema",
        core_contract::CORE_CONTRACT_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "ir_contract_schema",
        ir_contract::IR_CONTRACT_SCHEMA,
        true,
    );
    push_resolver_summary(&mut out, &report.resolve_summary, 2, true);
    push_type_check_summary(&mut out, &report.type_check_summary, 2, true);
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
    let resolve_summary = resolve::resolve_readiness_summary(program, diagnostics);
    let type_check_summary = type_check::type_check_summary(program, diagnostics);
    let mut candidates = Vec::new();
    for file in &program.files {
        collect_candidates_from_items(
            &file.items,
            diagnostics,
            &resolve_summary,
            &type_check_summary,
            &mut candidates,
        );
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
        resolve_summary,
        type_check_summary,
        candidates,
    }
}

fn collect_candidates_from_items(
    items: &[Item],
    diagnostics: &[Diagnostic],
    resolve_summary: &resolve::ResolveReadinessSummary,
    type_check_summary: &type_check::TypeCheckSummary,
    candidates: &mut Vec<LoweringCandidate>,
) {
    for item in items {
        candidates.push(lowering_candidate(
            item,
            diagnostics,
            resolve_summary,
            type_check_summary,
        ));
        if let Item::App(app) = item {
            collect_candidates_from_items(
                &app.items,
                diagnostics,
                resolve_summary,
                type_check_summary,
                candidates,
            );
        }
    }
}

fn lowering_candidate(
    item: &Item,
    diagnostics: &[Diagnostic],
    resolve_summary: &resolve::ResolveReadinessSummary,
    type_check_summary: &type_check::TypeCheckSummary,
) -> LoweringCandidate {
    let graph_node_id = node_id::span(
        "item",
        item.span(),
        &format!("{} {}", item.kind(), item.name()),
    );
    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);
    let has_resolver_errors = resolve_summary.resolver_errors > 0;
    let has_type_errors = type_check_summary.type_errors > 0;
    let blocking_reasons = blocking_reasons(has_errors, has_resolver_errors, has_type_errors);
    let section_names = item_sections(item)
        .iter()
        .map(|section| section.name.clone())
        .collect::<Vec<_>>();
    let body_grammar = body_grammar_for_item(item);

    LoweringCandidate {
        id: readiness_id(item),
        kind: item.kind(),
        name: item.name().to_string(),
        graph_node_id,
        span: portable_span(item.span()),
        status: if has_errors {
            "blocked_by_source_errors"
        } else if has_resolver_errors {
            "blocked_by_resolver_errors"
        } else if has_type_errors {
            "blocked_by_type_errors"
        } else {
            "blocked_before_core_lowering"
        },
        current_layer: CURRENT_LAYER,
        target_layer: TARGET_LAYER,
        facts_available: facts_available(item, resolve_summary, type_check_summary),
        missing_passes: MISSING_IR_PASSES.to_vec(),
        blocking_reasons,
        section_names,
        body_grammar,
    }
}

fn facts_available(
    item: &Item,
    resolve_summary: &resolve::ResolveReadinessSummary,
    type_check_summary: &type_check::TypeCheckSummary,
) -> Vec<&'static str> {
    let mut facts = vec![
        "source_span",
        "semantic_graph_node_id",
        "item_kind",
        "item_name",
        "resolver_summary_v0",
        resolve_summary.status,
        "type_check_summary_v0",
        type_check_summary.status,
        "trivial_return_checks_v0",
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
    if sections
        .iter()
        .find(|section| section.name == "does")
        .map(core_body::analyze_does_section)
        .is_some_and(|report| report.recognized_lines > 0)
    {
        facts.push("body_grammar_partial_v0");
    }
}

fn body_grammar_for_item(item: &Item) -> Option<BodyGrammarReport> {
    item_sections(item)
        .iter()
        .find(|section| section.name == "does")
        .map(core_body::analyze_does_section)
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

fn blocking_reasons(
    has_errors: bool,
    has_resolver_errors: bool,
    has_type_errors: bool,
) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if has_errors {
        reasons.push("source_diagnostics_include_errors");
    }
    if has_resolver_errors {
        reasons.push("checked_resolver_errors");
    }
    if has_type_errors {
        reasons.push("type_check_errors");
    }
    reasons.push("core_lowering_not_implemented");
    reasons.push("full_type_check_not_implemented");
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

    fn body_grammar_candidates(&self) -> usize {
        self.candidates
            .iter()
            .filter(|candidate| candidate.body_grammar.is_some())
            .count()
    }

    fn body_grammar_recognized_lines(&self) -> usize {
        self.candidates
            .iter()
            .filter_map(|candidate| candidate.body_grammar.as_ref())
            .map(|report| report.recognized_lines)
            .sum()
    }

    fn body_grammar_unsupported_lines(&self) -> usize {
        self.candidates
            .iter()
            .filter_map(|candidate| candidate.body_grammar.as_ref())
            .map(|report| report.unsupported_lines)
            .sum()
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

fn push_resolver_summary(
    out: &mut String,
    summary: &resolve::ResolveReadinessSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"resolver\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(out, indent + 2, "files", summary.files, true);
    push_usize_field(out, indent + 2, "items", summary.items, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        summary.source_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "source_warnings",
        summary.source_warnings,
        true,
    );
    push_usize_field(out, indent + 2, "scopes", summary.scopes, true);
    push_usize_field(out, indent + 2, "definitions", summary.definitions, true);
    push_usize_field(out, indent + 2, "references", summary.references, true);
    push_usize_field(
        out,
        indent + 2,
        "resolved_references",
        summary.resolved_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unresolved_references",
        summary.unresolved_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "external_references",
        summary.external_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "duplicate_definitions",
        summary.duplicate_definitions,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "mutable_place_errors",
        summary.mutable_place_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        summary.resolver_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_warnings",
        summary.resolver_warnings,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_type_check_summary(
    out: &mut String,
    summary: &type_check::TypeCheckSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"type_check\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        summary.source_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "source_warnings",
        summary.source_warnings,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        summary.resolver_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_declarations",
        summary.checked_declarations,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_declarations",
        summary.accepted_declarations,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_declarations",
        summary.rejected_declarations,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_type_references",
        summary.checked_type_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unknown_type_references",
        summary.unknown_type_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_returns",
        summary.checked_returns,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_returns",
        summary.accepted_returns,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_returns",
        summary.rejected_returns,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_returns",
        summary.unchecked_returns,
        true,
    );
    push_usize_field(out, indent + 2, "type_errors", summary.type_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "type_warnings",
        summary.type_warnings,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_summary(out: &mut String, report: &IrReadinessReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"lowering_candidates\": {}, \"ready_for_ir\": 0, \"blocked\": {}, \"errors\": {}, \"warnings\": {}, \"type_errors\": {}, \"unknown_type_references\": {}, \"checked_returns\": {}, \"rejected_returns\": {}, \"unchecked_returns\": {}, \"body_grammar_candidates\": {}, \"body_grammar_recognized_lines\": {}, \"body_grammar_unsupported_lines\": {}",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.candidates.len(),
        report.blocked_count(),
        report.errors,
        report.warnings,
        report.type_check_summary.type_errors,
        report.type_check_summary.unknown_type_references,
        report.type_check_summary.checked_returns,
        report.type_check_summary.rejected_returns,
        report.type_check_summary.unchecked_returns,
        report.body_grammar_candidates(),
        report.body_grammar_recognized_lines(),
        report.body_grammar_unsupported_lines()
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
    if let Some(body_grammar) = &candidate.body_grammar {
        push_body_grammar(out, indent + 2, body_grammar, true);
    }
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

fn push_body_grammar(out: &mut String, indent: usize, report: &BodyGrammarReport, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"body_grammar\": {\n");
    push_string_field(out, indent + 2, "status", report.status, true);
    push_string_field(
        out,
        indent + 2,
        "grammar_status",
        report.grammar_status,
        true,
    );
    push_usize_field(out, indent + 2, "total_lines", report.total_lines, true);
    push_usize_field(
        out,
        indent + 2,
        "meaningful_lines",
        report.meaningful_lines,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "recognized_lines",
        report.recognized_lines,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unsupported_lines",
        report.unsupported_lines,
        true,
    );
    push_body_statements(out, indent + 2, &report.statements, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_body_statements(
    out: &mut String,
    indent: usize,
    statements: &[BodyStatement],
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"statements\": [\n");
    for (index, statement) in statements.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_body_statement(out, indent + 2, statement);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_body_statement(out: &mut String, indent: usize, statement: &BodyStatement) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_span_field(out, indent + 2, "source_span", &statement.span, true);
    push_string_field(out, indent + 2, "text", &statement.text, true);
    push_string_field(out, indent + 2, "kind", statement.kind, true);
    push_string_field(out, indent + 2, "status", statement.status, true);
    push_optional_string_field(
        out,
        indent + 2,
        "expression_kind",
        statement.expression_kind,
        true,
    );
    push_optional_string_field(out, indent + 2, "reason", statement.reason, false);
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

fn push_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(&format!(": {value}"));
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

    use super::{ir_readiness_json, ir_readiness_text};

    #[test]
    fn text_report_lists_lowering_candidates_without_emitting_ir() {
        let program = demo_program();
        let text = ir_readiness_text(&program, &[]);

        assert!(text.contains("Hum IR readiness (hum.ir_readiness.v0)"));
        assert!(text.contains("core_contract_schema: hum.core_contract.v0"));
        assert!(text.contains("resolver: schema=hum.resolve.v0 status=checked_resolver_v0"));
        assert!(text.contains("lowering_candidates=4 ready_for_ir=0 blocked=4"));
        assert!(text.contains("body_grammar_candidates=2"));
        assert!(
            text.contains(
                "type_check_status=declaration_annotations_and_trivial_returns_checked_v0"
            )
        );
        assert!(text.contains("type_errors=0 unknown_type_references=0"));
        assert!(text.contains("type_check: schema=hum.type_check.v0"));
        assert!(text.contains("pass_status:"));
        assert!(text.contains("body_grammar [partial_v0]"));
        assert!(text.contains("type_check [declaration_and_trivial_return_check_available]"));
        assert!(text.contains("core_lowering [not_implemented]"));
        assert!(text.contains("task `add task`"));
        assert!(text.contains("missing_passes: core_lowering"));
        assert!(text.contains("full_type_check"));
    }

    #[test]
    fn json_report_lists_facts_and_blockers() {
        let program = demo_program();
        let json = ir_readiness_json(&program, &[]);

        assert!(json.contains("\"schema\": \"hum.ir_readiness.v0\""));
        assert!(json.contains("\"core_contract_schema\": \"hum.core_contract.v0\""));
        assert!(json.contains("\"ir_contract_schema\": \"hum.ir_contract.v0\""));
        assert!(json.contains("\"resolver\""));
        assert!(json.contains("\"schema\": \"hum.resolve.v0\""));
        assert!(json.contains("\"status\": \"checked_resolver_v0\""));
        assert!(json.contains("\"mode\": \"source_analysis_only_no_type_or_borrow_check\""));
        assert!(json.contains("\"resolver_errors\": 0"));
        assert!(json.contains("\"type_check\""));
        assert!(json.contains("\"schema\": \"hum.type_check.v0\""));
        assert!(
            json.contains("\"status\": \"declaration_annotations_and_trivial_returns_checked_v0\"")
        );
        assert!(json.contains("\"type_errors\": 0"));
        assert!(json.contains("\"unknown_type_references\": 0"));
        assert!(json.contains("\"checked_resolver_v0\""));
        assert!(json.contains("\"type_check_summary_v0\""));
        assert!(json.contains("\"declaration_annotations_and_trivial_returns_checked_v0\""));
        assert!(json.contains("\"ready_for_ir\": 0"));
        assert!(json.contains("\"body_grammar_candidates\": 2"));
        assert!(json.contains("\"body_grammar_unsupported_lines\": 1"));
        assert!(json.contains("\"status\": \"blocked_before_core_lowering\""));
        assert!(json.contains("\"name\": \"body_grammar\""));
        assert!(json.contains("\"status\": \"partial_v0\""));
        assert!(json.contains("\"body_grammar\""));
        assert!(json.contains("\"kind\": \"return\""));
        assert!(json.contains("\"reason\": \"surface_save_requires_store_lowering\""));
        assert!(json.contains("\"body_grammar_partial_v0\""));
        assert!(json.contains("\"name\": \"semantic_graph_build\""));
        assert!(json.contains("\"name\": \"type_check\""));
        assert!(json.contains("\"status\": \"declaration_and_trivial_return_check_available\""));
        assert!(json.contains("\"full_type_check\""));
        assert!(json.contains("\"full_type_check_not_implemented\""));
        assert!(json.contains("\"status\": \"report_available_not_ir_pass\""));
        assert!(json.contains("\"effect_hints\""));
        assert!(json.contains("\"contract_hints\""));
        assert!(json.contains("\"body_text_captured\""));
        assert!(json.contains("\"no IR emission\""));
    }

    #[test]
    fn json_blocks_on_type_errors_before_lowering() {
        let source = r#"type Box {
  value: MissingType
}

task pass box(item: Box) -> Box {
  does:
    return item
}
"#;
        let parsed = parse_source("bad_type.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = ir_readiness_json(&program, &[]);

        assert!(json.contains("\"type_check\""));
        assert!(json.contains("\"schema\": \"hum.type_check.v0\""));
        assert!(json.contains("\"status\": \"type_errors_v0\""));
        assert!(json.contains("\"type_errors\": 1"));
        assert!(json.contains("\"unknown_type_references\": 1"));
        assert!(json.contains("\"status\": \"blocked_by_type_errors\""));
        assert!(json.contains("\"type_check_errors\""));
        assert!(json.contains("\"full_type_check_not_implemented\""));
        assert!(json.contains("\"ready_for_ir\": 0"));
    }
    #[test]
    fn json_blocks_on_resolver_errors_before_lowering() {
        let source = r#"task bad names() -> UInt {
  does:
    return missing
}
"#;
        let parsed = parse_source("bad.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = ir_readiness_json(&program, &[]);

        assert!(json.contains("\"status\": \"checked_resolver_with_errors_v0\""));
        assert!(json.contains("\"resolver_errors\": 1"));
        assert!(json.contains("\"status\": \"blocked_by_resolver_errors\""));
        assert!(json.contains("\"checked_resolver_errors\""));
        assert!(json.contains("\"ready_for_ir\": 0"));
    }

    fn demo_program() -> Program {
        let source = r#"type Task {
  title: Text
}

store tasks: list Task {
  why:
    remember tasks
}

task add task(title: Text) -> Task {
  why:
    save a task

  changes:
    tasks

  ensures:
    task is visible

  does:
    let task = Task {
      title: title
    }
    save task in tasks
    return task
}

test add task is visible {
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
