use std::collections::BTreeSet;

use crate::ast::{Item, Program, Section};
use crate::core_body::{self, BodyStatement};
use crate::core_contract;
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::full_type_check;
use crate::graph::is_meaningful_line_text;
use crate::version;

pub const EFFECT_CHECK_SCHEMA: &str = "hum.effect_check.v0";
pub const EFFECT_CHECK_MODE: &str = "recognized_core_effect_gate_v0";
pub const EFFECT_CHECK_STATUS: &str = "recognized_core_effect_gate_available_v0";

const NON_CLAIMS: &[&str] = &[
    "no executable semantics",
    "no Hum IR emission",
    "no backend lowering",
    "no proof artifact",
    "no memory-safety proof",
    "no complete effect system",
    "no ownership or borrow checking",
    "no profile enforcement",
    "no allocation safety proof",
    "no optimization claim",
];

const AMBIENT_READ_ROOTS: &[&str] = &[
    "clock", "time", "random", "crypto", "file", "network", "socket", "env", "process", "os",
    "registry", "device", "sensor", "storage", "database", "http",
];

const SECURITY_SENSITIVE_ROOTS: &[&str] =
    &["random", "crypto", "password", "token", "network", "socket"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectCheckSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub source_errors: usize,
    pub resolver_errors: usize,
    pub type_errors: usize,
    pub core_verify_errors: usize,
    pub full_type_check_errors: usize,
    pub items: usize,
    pub effect_items: usize,
    pub statements: usize,
    pub checked_statements: usize,
    pub accepted_statements: usize,
    pub rejected_statements: usize,
    pub unchecked_statements: usize,
    pub boundary_checks: usize,
    pub rejected_boundary_checks: usize,
    pub declared_uses: usize,
    pub declared_changes: usize,
    pub declared_failures: usize,
    pub declared_allocations: usize,
    pub declared_avoids: usize,
    pub declared_protects: usize,
    pub declared_trusts: usize,
    pub inferred_reads: usize,
    pub inferred_changes: usize,
    pub inferred_failures: usize,
    pub blocking_issues: usize,
    pub execution_ready: usize,
    pub ir_ready: usize,
}

struct EffectCheckReport {
    full_type_check_summary: full_type_check::FullTypeCheckSummary,
    items: Vec<EffectItem>,
    files: usize,
    item_count: usize,
    source_errors: usize,
}

struct EffectItem {
    id: String,
    kind: &'static str,
    name: String,
    span: Span,
    status: &'static str,
    declarations: EffectDeclarations,
    statements: Vec<EffectStatement>,
    boundary_checks: Vec<EffectBoundaryCheck>,
}

#[derive(Default)]
struct EffectDeclarations {
    uses: Vec<DeclaredFact>,
    changes: Vec<DeclaredFact>,
    failures: Vec<DeclaredFact>,
    allocations: Vec<DeclaredFact>,
    avoids: Vec<DeclaredFact>,
    protects: Vec<DeclaredFact>,
    trusts: Vec<DeclaredFact>,
}

struct DeclaredFact {
    section: &'static str,
    text: String,
    resource: String,
    span: Span,
}

struct EffectStatement {
    id: String,
    span: Span,
    statement_kind: &'static str,
    effect_kind: &'static str,
    target: Option<String>,
    declaration: Option<String>,
    status: &'static str,
    reason: Option<&'static str>,
}

struct EffectBoundaryCheck {
    id: String,
    span: Span,
    check: &'static str,
    status: &'static str,
    reason: Option<&'static str>,
}

pub fn effect_check_has_errors(program: &Program, diagnostics: &[Diagnostic]) -> bool {
    effect_check_summary(program, diagnostics).blocking_issues > 0
}

pub fn effect_check_summary(program: &Program, diagnostics: &[Diagnostic]) -> EffectCheckSummary {
    let report = build_report(program, diagnostics);
    EffectCheckSummary {
        schema: EFFECT_CHECK_SCHEMA,
        status: report.status(),
        mode: EFFECT_CHECK_MODE,
        source_errors: report.source_errors,
        resolver_errors: report.full_type_check_summary.resolver_errors,
        type_errors: report.full_type_check_summary.type_errors,
        core_verify_errors: report.full_type_check_summary.core_verify_errors,
        full_type_check_errors: report.full_type_check_errors(),
        items: report.item_count(),
        effect_items: report.items.len(),
        statements: report.statement_count(),
        checked_statements: report.checked_statements(),
        accepted_statements: report.accepted_statements(),
        rejected_statements: report.rejected_statements(),
        unchecked_statements: report.unchecked_statements(),
        boundary_checks: report.boundary_checks(),
        rejected_boundary_checks: report.rejected_boundary_checks(),
        declared_uses: report.declared_uses(),
        declared_changes: report.declared_changes(),
        declared_failures: report.declared_failures(),
        declared_allocations: report.declared_allocations(),
        declared_avoids: report.declared_avoids(),
        declared_protects: report.declared_protects(),
        declared_trusts: report.declared_trusts(),
        inferred_reads: report.inferred_reads(),
        inferred_changes: report.inferred_changes(),
        inferred_failures: report.inferred_failures(),
        blocking_issues: report.blocking_issues(),
        execution_ready: 0,
        ir_ready: 0,
    }
}

pub fn effect_check_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum effect check ({EFFECT_CHECK_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {EFFECT_CHECK_MODE}\n"));
    out.push_str(&format!("status: {}\n", report.status()));
    out.push_str(&format!(
        "dependencies: core_contract={} full_type_check={}\n",
        core_contract::CORE_CONTRACT_SCHEMA,
        full_type_check::FULL_TYPE_CHECK_SCHEMA
    ));
    out.push_str(&format!(
        "summary: files={} items={} effect_items={} statements={} checked_statements={} accepted_statements={} rejected_statements={} unchecked_statements={} boundary_checks={} rejected_boundary_checks={} declared_uses={} declared_changes={} declared_failures={} declared_allocations={} declared_avoids={} declared_protects={} declared_trusts={} inferred_reads={} inferred_changes={} inferred_failures={} blocking_issues={} source_errors={} resolver_errors={} type_errors={} core_verify_errors={} full_type_check_errors={} execution_ready=0 ir_ready=0\n",
        report.files(),
        report.item_count(),
        report.items.len(),
        report.statement_count(),
        report.checked_statements(),
        report.accepted_statements(),
        report.rejected_statements(),
        report.unchecked_statements(),
        report.boundary_checks(),
        report.rejected_boundary_checks(),
        report.declared_uses(),
        report.declared_changes(),
        report.declared_failures(),
        report.declared_allocations(),
        report.declared_avoids(),
        report.declared_protects(),
        report.declared_trusts(),
        report.inferred_reads(),
        report.inferred_changes(),
        report.inferred_failures(),
        report.blocking_issues(),
        report.source_errors,
        report.full_type_check_summary.resolver_errors,
        report.full_type_check_summary.type_errors,
        report.full_type_check_summary.core_verify_errors,
        report.full_type_check_errors()
    ));

    if report.items.is_empty() {
        out.push_str("effect_items: none\n");
    } else {
        out.push_str("effect_items:\n");
        for item in &report.items {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} `{}` statements={} boundary_checks={}\n",
                item.span.file,
                item.span.line,
                item.span.column,
                item.status,
                item.kind,
                item.name,
                item.statements.len(),
                item.boundary_checks.len()
            ));
            for statement in &item.statements {
                out.push_str(&format!(
                    "    {}:{}:{} [{}] {} effect={} target={}",
                    statement.span.file,
                    statement.span.line,
                    statement.span.column,
                    statement.status,
                    statement.statement_kind,
                    statement.effect_kind,
                    statement.target.as_deref().unwrap_or("none")
                ));
                if let Some(declaration) = &statement.declaration {
                    out.push_str(&format!(" declaration={declaration}"));
                }
                if let Some(reason) = statement.reason {
                    out.push_str(&format!(" reason={reason}"));
                }
                out.push('\n');
            }
            for check in &item.boundary_checks {
                out.push_str(&format!(
                    "    {}:{}:{} [{}] boundary={} reason={}\n",
                    check.span.file,
                    check.span.line,
                    check.span.column,
                    check.status,
                    check.check,
                    check.reason.unwrap_or("none")
                ));
            }
        }
    }

    out.push_str("non_claims:\n");
    for non_claim in NON_CLAIMS {
        out.push_str(&format!("  - {non_claim}\n"));
    }

    out
}

pub fn effect_check_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", EFFECT_CHECK_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", EFFECT_CHECK_MODE, true);
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
        "full_type_check_schema",
        full_type_check::FULL_TYPE_CHECK_SCHEMA,
        true,
    );
    push_dependency_summary(&mut out, &report, 2, true);
    push_summary(&mut out, &report, 2, true);
    push_items(&mut out, &report.items, 2, true);
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> EffectCheckReport {
    let full_type_check_summary = full_type_check::full_type_check_summary(program, diagnostics);
    let source_errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let blocked = source_errors > 0 || full_type_check_summary.blocking_issues > 0;
    let mut items = Vec::new();
    for file in &program.files {
        collect_items(&file.items, blocked, &mut items);
    }

    EffectCheckReport {
        full_type_check_summary,
        items,
        files: program.files.len(),
        item_count: count_items(program),
        source_errors,
    }
}

fn collect_items(items: &[Item], blocked: bool, out: &mut Vec<EffectItem>) {
    for item in items {
        if let Some(effect_item) = check_item(item, blocked) {
            out.push(effect_item);
        }
        if let Item::App(app) = item {
            collect_items(&app.items, blocked, out);
        }
    }
}

fn check_item(item: &Item, blocked: bool) -> Option<EffectItem> {
    let does = item_sections(item)
        .iter()
        .find(|section| section.name == "does")?;
    let body = core_body::analyze_does_section(does);
    let declarations = collect_declarations(item_sections(item));
    let local_mutables = local_mutables(&body.statements);
    let mut statements = Vec::new();
    for (index, statement) in body.statements.iter().enumerate() {
        statements.push(check_statement_effect(
            statement,
            index,
            &declarations,
            &local_mutables,
            blocked,
        ));
    }
    let boundary_checks = boundary_checks(item, &declarations, &body.statements, &statements);
    let status = item_status(&statements, &boundary_checks, blocked);
    Some(EffectItem {
        id: prefixed_id(
            "hum_effect_item",
            &format!("{}_{}_{}", item.kind(), item.name(), item.span().line),
        ),
        kind: item.kind(),
        name: item.name().to_string(),
        span: portable_span(item.span()),
        status,
        declarations,
        statements,
        boundary_checks,
    })
}

fn check_statement_effect(
    statement: &BodyStatement,
    index: usize,
    declarations: &EffectDeclarations,
    local_mutables: &BTreeSet<String>,
    blocked: bool,
) -> EffectStatement {
    if blocked {
        return effect_statement(
            statement,
            index,
            "prior_blocker",
            None,
            None,
            "not_checked_blocked_by_prior_errors_v0",
            Some("source_or_full_type_check_errors"),
        );
    }

    if statement.status == "unsupported_v0" {
        return effect_statement(
            statement,
            index,
            "unsupported_statement",
            None,
            None,
            "unchecked_statement_effect_v0",
            statement
                .reason
                .or(Some("statement_not_in_core_body_grammar_v0")),
        );
    }

    match statement.kind {
        "fail" => {
            if declarations.failures.is_empty() {
                effect_statement(
                    statement,
                    index,
                    "typed_failure",
                    expression_text_for_statement(statement).map(str::to_string),
                    None,
                    "rejected_missing_fails_when_declaration_v0",
                    Some("fail_statement_requires_fails_when_section"),
                )
            } else {
                effect_statement(
                    statement,
                    index,
                    "typed_failure",
                    expression_text_for_statement(statement).map(str::to_string),
                    Some("fails when".to_string()),
                    "accepted_declared_failure_v0",
                    None,
                )
            }
        }
        "mutable_binding" => effect_statement(
            statement,
            index,
            "local_mutation_permission",
            binding_name(statement),
            Some("change".to_string()),
            "accepted_local_mutation_permission_v0",
            None,
        ),
        "set_place" => check_set_statement(statement, index, declarations, local_mutables),
        "save_in_store" => check_save_statement(statement, index, declarations),
        "test_expectation" => effect_statement(
            statement,
            index,
            "test_expectation",
            None,
            None,
            "unchecked_statement_effect_v0",
            Some("test_expectation_effects_not_checked_v0"),
        ),
        "for_each_header" | "for_index_header" => effect_statement(
            statement,
            index,
            "iteration",
            expression_text_for_statement(statement).map(str::to_string),
            None,
            "unchecked_statement_effect_v0",
            Some("iterator_effects_not_checked_v0"),
        ),
        "nested_intent_header" => effect_statement(
            statement,
            index,
            "nested_intent",
            None,
            None,
            "unchecked_statement_effect_v0",
            Some("nested_intent_effects_not_checked_v0"),
        ),
        _ => expression_or_pure_effect(statement, index, declarations),
    }
}

fn check_set_statement(
    statement: &BodyStatement,
    index: usize,
    declarations: &EffectDeclarations,
    local_mutables: &BTreeSet<String>,
) -> EffectStatement {
    let Some(target) = set_place_name(statement) else {
        return effect_statement(
            statement,
            index,
            "mutation",
            None,
            None,
            "unchecked_statement_effect_v0",
            Some("set_target_unknown_v0"),
        );
    };
    let resource = first_resource(&target);
    if local_mutables.contains(&resource) {
        effect_statement(
            statement,
            index,
            "local_mutation",
            Some(resource),
            Some("change".to_string()),
            "accepted_local_mutation_v0",
            None,
        )
    } else if declares_resource(&declarations.changes, &resource) {
        effect_statement(
            statement,
            index,
            "declared_change",
            Some(resource),
            Some("changes".to_string()),
            "accepted_declared_change_v0",
            None,
        )
    } else {
        effect_statement(
            statement,
            index,
            "declared_change",
            Some(resource),
            None,
            "rejected_missing_changes_declaration_v0",
            Some("set_statement_requires_change_binding_or_changes_section"),
        )
    }
}

fn check_save_statement(
    statement: &BodyStatement,
    index: usize,
    declarations: &EffectDeclarations,
) -> EffectStatement {
    let Some(target) = save_target(&statement.text) else {
        return effect_statement(
            statement,
            index,
            "store_change",
            None,
            None,
            "unchecked_statement_effect_v0",
            Some("save_target_unknown_v0"),
        );
    };
    let resource = first_resource(target);
    if declares_resource(&declarations.changes, &resource) {
        effect_statement(
            statement,
            index,
            "store_change",
            Some(resource),
            Some("changes".to_string()),
            "accepted_declared_change_v0",
            None,
        )
    } else {
        effect_statement(
            statement,
            index,
            "store_change",
            Some(resource),
            None,
            "rejected_missing_changes_declaration_v0",
            Some("save_statement_requires_changes_section"),
        )
    }
}

fn expression_or_pure_effect(
    statement: &BodyStatement,
    index: usize,
    declarations: &EffectDeclarations,
) -> EffectStatement {
    let expression = expression_text_for_statement(statement);
    if let Some(resource) = expression.and_then(first_ambient_resource) {
        if declares_resource(&declarations.uses, &resource) {
            effect_statement(
                statement,
                index,
                "ambient_read",
                Some(resource),
                Some("uses".to_string()),
                "accepted_declared_use_v0",
                None,
            )
        } else {
            effect_statement(
                statement,
                index,
                "ambient_read",
                Some(resource),
                None,
                "rejected_missing_uses_declaration_v0",
                Some("ambient_read_requires_uses_section"),
            )
        }
    } else {
        effect_statement(
            statement,
            index,
            "pure_or_local",
            expression.map(str::to_string),
            None,
            "accepted_no_external_effect_v0",
            None,
        )
    }
}

fn effect_statement(
    statement: &BodyStatement,
    index: usize,
    effect_kind: &'static str,
    target: Option<String>,
    declaration: Option<String>,
    status: &'static str,
    reason: Option<&'static str>,
) -> EffectStatement {
    EffectStatement {
        id: prefixed_id(
            "hum_effect_stmt",
            &format!("{}_{}_{}", statement.kind, statement.span.line, index),
        ),
        span: portable_span(&statement.span),
        statement_kind: statement.kind,
        effect_kind,
        target,
        declaration,
        status,
        reason,
    }
}

fn boundary_checks(
    item: &Item,
    declarations: &EffectDeclarations,
    body_statements: &[BodyStatement],
    effect_statements: &[EffectStatement],
) -> Vec<EffectBoundaryCheck> {
    let mut checks = Vec::new();
    let item_span = portable_span(item.span());
    if !declarations.trusts.is_empty() {
        checks.push(boundary_check(
            "trust_requires_protects",
            &item_span,
            if declarations.protects.is_empty() {
                "rejected_trust_without_protects_v0"
            } else {
                "accepted_trust_boundary_has_protects_v0"
            },
            if declarations.protects.is_empty() {
                Some("trusts_section_requires_visible_protects_section")
            } else {
                None
            },
        ));
    }

    if has_security_sensitive_effect(declarations, body_statements) {
        checks.push(boundary_check(
            "security_effect_requires_protects",
            &item_span,
            if declarations.protects.is_empty() {
                "rejected_security_effect_without_protects_v0"
            } else {
                "accepted_security_effect_has_protects_v0"
            },
            if declarations.protects.is_empty() {
                Some("security_sensitive_effect_requires_protects_section")
            } else {
                None
            },
        ));
    }

    for avoid in &declarations.avoids {
        if let Some(reason) = avoid_contradiction(&avoid.text, effect_statements) {
            checks.push(boundary_check(
                "avoids_not_contradicted",
                &avoid.span,
                "rejected_avoids_contradicted_v0",
                Some(reason),
            ));
        }
    }

    checks
}

fn boundary_check(
    check: &'static str,
    span: &Span,
    status: &'static str,
    reason: Option<&'static str>,
) -> EffectBoundaryCheck {
    EffectBoundaryCheck {
        id: prefixed_id(
            "hum_effect_boundary",
            &format!("{}_{}_{}", check, span.line, span.column),
        ),
        span: portable_span(span),
        check,
        status,
        reason,
    }
}

fn collect_declarations(sections: &[Section]) -> EffectDeclarations {
    EffectDeclarations {
        uses: declared_facts(sections, "uses"),
        changes: declared_facts(sections, "changes"),
        failures: declared_facts(sections, "fails when"),
        allocations: declared_facts(sections, "allocates"),
        avoids: declared_facts(sections, "avoids"),
        protects: declared_facts(sections, "protects"),
        trusts: declared_facts(sections, "trusts"),
    }
}

fn declared_facts(sections: &[Section], name: &'static str) -> Vec<DeclaredFact> {
    sections
        .iter()
        .filter(|section| section.name == name)
        .flat_map(|section| {
            section.lines.iter().filter_map(move |line| {
                let text = line.text.trim();
                if !is_meaningful_line_text(text) {
                    return None;
                }
                Some(DeclaredFact {
                    section: name,
                    text: text.to_string(),
                    resource: first_resource(text),
                    span: portable_span(&line.span),
                })
            })
        })
        .collect()
}

fn local_mutables(statements: &[BodyStatement]) -> BTreeSet<String> {
    statements
        .iter()
        .filter(|statement| statement.kind == "mutable_binding")
        .filter_map(binding_name)
        .map(|name| first_resource(&name))
        .collect()
}

fn expression_text_for_statement(statement: &BodyStatement) -> Option<&str> {
    match statement.kind {
        "return" => strip_keyword(&statement.text, "return"),
        "fail" => strip_keyword(&statement.text, "fail"),
        "let_binding" => binding_initializer(statement),
        "mutable_binding" => binding_initializer(statement),
        "set_place" => statement
            .text
            .split_once('=')
            .map(|(_place, value)| value.trim()),
        "if_header" => header_body(&statement.text, "if"),
        "while_header" => header_body(&statement.text, "while"),
        "for_each_header" => header_body(&statement.text, "for each"),
        "for_index_header" => header_body(&statement.text, "for index"),
        "record_field_initializer" => statement
            .text
            .split_once(':')
            .map(|(_field, value)| value.trim()),
        "test_expectation" => strip_keyword(&statement.text, "expect"),
        _ => None,
    }
}

fn binding_initializer(statement: &BodyStatement) -> Option<&str> {
    if !matches!(statement.kind, "let_binding" | "mutable_binding") {
        return None;
    }
    statement
        .text
        .split_once('=')
        .map(|(_left, value)| value.trim())
}

fn binding_name(statement: &BodyStatement) -> Option<String> {
    if !matches!(statement.kind, "let_binding" | "mutable_binding") {
        return None;
    }
    let keyword = if statement.kind == "let_binding" {
        "let"
    } else {
        "change"
    };
    let rest = strip_keyword(&statement.text, keyword)?;
    let left = rest.split_once('=').map(|(left, _value)| left.trim())?;
    let name = left.split_once(':').map_or(left, |(name, _type_text)| name);
    let name = name.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn set_place_name(statement: &BodyStatement) -> Option<String> {
    let rest = strip_keyword(&statement.text, "set")?;
    let (place, _value) = rest.split_once('=')?;
    let place = place.trim();
    if place.is_empty() {
        None
    } else {
        Some(place.to_string())
    }
}

fn save_target(text: &str) -> Option<&str> {
    let rest = strip_keyword(text, "save")?;
    let (_value, target) = rest.split_once(" in ")?;
    let target = target.trim();
    if target.is_empty() {
        None
    } else {
        Some(target)
    }
}

fn first_ambient_resource(text: &str) -> Option<String> {
    let lowered = text.to_ascii_lowercase();
    AMBIENT_READ_ROOTS
        .iter()
        .find(|root| contains_word_or_path(&lowered, root))
        .map(|root| root.to_string())
}

fn has_security_sensitive_effect(
    declarations: &EffectDeclarations,
    body_statements: &[BodyStatement],
) -> bool {
    declarations.uses.iter().any(|fact| {
        SECURITY_SENSITIVE_ROOTS
            .iter()
            .any(|root| fact.resource == *root || fact.text.to_ascii_lowercase().contains(root))
    }) || body_statements.iter().any(|statement| {
        let lowered = statement.text.to_ascii_lowercase();
        SECURITY_SENSITIVE_ROOTS
            .iter()
            .any(|root| contains_word_or_path(&lowered, root))
    })
}

fn avoid_contradiction(
    avoid_text: &str,
    effect_statements: &[EffectStatement],
) -> Option<&'static str> {
    let lowered = avoid_text.to_ascii_lowercase();
    for statement in effect_statements {
        if statement.effect_kind == "typed_failure" && lowered.contains("fail") {
            return Some("avoids_failure_but_body_can_fail_v0");
        }
        if let Some(target) = &statement.target {
            let target = target.to_ascii_lowercase();
            if !target.is_empty() && lowered.contains(&target) {
                return Some("avoids_named_resource_but_body_uses_it_v0");
            }
        }
    }
    None
}

fn declares_resource(facts: &[DeclaredFact], target: &str) -> bool {
    let target = target.to_ascii_lowercase();
    facts.iter().any(|fact| {
        fact.resource == target || fact.text.to_ascii_lowercase().contains(target.as_str())
    })
}

fn contains_word_or_path(text: &str, needle: &str) -> bool {
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '.'))
        .any(|part| {
            part == needle
                || part
                    .split_once('.')
                    .is_some_and(|(root, _rest)| root == needle)
        })
}

fn first_resource(text: &str) -> String {
    text.split_whitespace()
        .next()
        .unwrap_or(text)
        .trim_matches(|ch: char| ch == ',' || ch == '.')
        .to_ascii_lowercase()
}

fn header_body<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = strip_keyword(text, keyword)?;
    rest.strip_suffix('{').map(str::trim)
}

fn strip_keyword<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    if text == keyword {
        return Some("");
    }
    text.strip_prefix(keyword)
        .and_then(|rest| rest.strip_prefix(char::is_whitespace))
        .map(str::trim)
}

fn item_status(
    statements: &[EffectStatement],
    boundary_checks: &[EffectBoundaryCheck],
    blocked: bool,
) -> &'static str {
    if blocked {
        "blocked_by_prior_errors"
    } else if statements.iter().any(is_rejected_statement)
        || boundary_checks.iter().any(is_rejected_boundary_check)
    {
        "effect_errors_v0"
    } else if statements.iter().any(is_unchecked_statement) {
        "blocked_by_unchecked_effects_v0"
    } else {
        "recognized_core_effects_checked_v0"
    }
}

fn is_rejected_statement(statement: &EffectStatement) -> bool {
    statement.status.starts_with("rejected_")
}

fn is_unchecked_statement(statement: &EffectStatement) -> bool {
    statement.status.starts_with("unchecked_") || statement.status.starts_with("not_checked_")
}

fn is_rejected_boundary_check(check: &EffectBoundaryCheck) -> bool {
    check.status.starts_with("rejected_")
}

impl EffectCheckReport {
    fn status(&self) -> &'static str {
        if self.source_errors > 0 {
            "blocked_by_source_errors"
        } else if self.full_type_check_summary.resolver_errors > 0 {
            "blocked_by_resolver_errors"
        } else if self.full_type_check_summary.type_errors > 0 {
            "blocked_by_type_errors"
        } else if self.full_type_check_summary.core_verify_errors > 0 {
            "blocked_by_core_verify_errors"
        } else if self.full_type_check_errors() > 0 {
            "blocked_by_full_type_check_errors"
        } else if self.rejected_statements() > 0 || self.rejected_boundary_checks() > 0 {
            "effect_errors_v0"
        } else if self.unchecked_statements() > 0 {
            "blocked_by_unchecked_effects_v0"
        } else {
            "recognized_core_effects_checked_v0"
        }
    }

    fn files(&self) -> usize {
        self.files
    }

    fn item_count(&self) -> usize {
        self.item_count
    }

    fn full_type_check_errors(&self) -> usize {
        self.full_type_check_summary
            .blocking_issues
            .saturating_sub(self.source_errors)
            .saturating_sub(self.full_type_check_summary.resolver_errors)
            .saturating_sub(self.full_type_check_summary.type_errors)
            .saturating_sub(self.full_type_check_summary.core_verify_errors)
    }

    fn statement_count(&self) -> usize {
        self.items.iter().map(|item| item.statements.len()).sum()
    }

    fn checked_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                !is_unchecked_statement(statement)
                    && statement.status != "not_checked_blocked_by_prior_errors_v0"
            })
            .count()
    }

    fn accepted_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| statement.status.starts_with("accepted_"))
            .count()
    }

    fn rejected_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| is_rejected_statement(statement))
            .count()
    }

    fn unchecked_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| is_unchecked_statement(statement))
            .count()
    }

    fn boundary_checks(&self) -> usize {
        self.items
            .iter()
            .map(|item| item.boundary_checks.len())
            .sum()
    }

    fn rejected_boundary_checks(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.boundary_checks.iter())
            .filter(|check| is_rejected_boundary_check(check))
            .count()
    }

    fn declared_uses(&self) -> usize {
        self.items
            .iter()
            .map(|item| item.declarations.uses.len())
            .sum()
    }

    fn declared_changes(&self) -> usize {
        self.items
            .iter()
            .map(|item| item.declarations.changes.len())
            .sum()
    }

    fn declared_failures(&self) -> usize {
        self.items
            .iter()
            .map(|item| item.declarations.failures.len())
            .sum()
    }

    fn declared_allocations(&self) -> usize {
        self.items
            .iter()
            .map(|item| item.declarations.allocations.len())
            .sum()
    }

    fn declared_avoids(&self) -> usize {
        self.items
            .iter()
            .map(|item| item.declarations.avoids.len())
            .sum()
    }

    fn declared_protects(&self) -> usize {
        self.items
            .iter()
            .map(|item| item.declarations.protects.len())
            .sum()
    }

    fn declared_trusts(&self) -> usize {
        self.items
            .iter()
            .map(|item| item.declarations.trusts.len())
            .sum()
    }

    fn inferred_reads(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| statement.effect_kind == "ambient_read")
            .count()
    }

    fn inferred_changes(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                matches!(
                    statement.effect_kind,
                    "declared_change" | "store_change" | "local_mutation"
                )
            })
            .count()
    }

    fn inferred_failures(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| statement.effect_kind == "typed_failure")
            .count()
    }

    fn blocking_issues(&self) -> usize {
        self.source_errors
            + self.full_type_check_summary.resolver_errors
            + self.full_type_check_summary.type_errors
            + self.full_type_check_summary.core_verify_errors
            + self.full_type_check_errors()
            + self.rejected_statements()
            + self.unchecked_statements()
            + self.rejected_boundary_checks()
    }
}

fn count_items(program: &Program) -> usize {
    program
        .files
        .iter()
        .map(|file| count_items_in(&file.items))
        .sum()
}

fn count_items_in(items: &[Item]) -> usize {
    items
        .iter()
        .map(|item| {
            1 + match item {
                Item::App(app) => count_items_in(&app.items),
                _ => 0,
            }
        })
        .sum()
}

fn item_sections(item: &Item) -> &[Section] {
    match item {
        Item::App(app) => &app.sections,
        Item::Type(type_def) => &type_def.sections,
        Item::Store(store) => &store.sections,
        Item::Task(task) => &task.sections,
        Item::Test(test) => &test.sections,
    }
}

fn portable_span(span: &Span) -> Span {
    Span {
        file: span.file.replace('\\', "/"),
        line: span.line,
        column: span.column,
    }
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

fn push_dependency_summary(
    out: &mut String,
    report: &EffectCheckReport,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "dependencies");
    out.push_str(": {\n");
    push_indent(out, indent + 2);
    push_json_string(out, "full_type_check");
    out.push_str(": {\n");
    push_string_field(
        out,
        indent + 4,
        "schema",
        report.full_type_check_summary.schema,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "status",
        report.full_type_check_summary.status,
        true,
    );
    push_usize_field(
        out,
        indent + 4,
        "blocking_issues",
        report.full_type_check_summary.blocking_issues,
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("}\n");
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_summary(out: &mut String, report: &EffectCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "summary");
    out.push_str(": {\n");
    push_usize_field(out, indent + 2, "files", report.files(), true);
    push_usize_field(out, indent + 2, "items", report.item_count(), true);
    push_usize_field(out, indent + 2, "effect_items", report.items.len(), true);
    push_usize_field(
        out,
        indent + 2,
        "statements",
        report.statement_count(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_statements",
        report.checked_statements(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_statements",
        report.accepted_statements(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_statements",
        report.rejected_statements(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_statements",
        report.unchecked_statements(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "boundary_checks",
        report.boundary_checks(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_boundary_checks",
        report.rejected_boundary_checks(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "declared_uses",
        report.declared_uses(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "declared_changes",
        report.declared_changes(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "declared_failures",
        report.declared_failures(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "declared_allocations",
        report.declared_allocations(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "declared_avoids",
        report.declared_avoids(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "declared_protects",
        report.declared_protects(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "declared_trusts",
        report.declared_trusts(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "inferred_reads",
        report.inferred_reads(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "inferred_changes",
        report.inferred_changes(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "inferred_failures",
        report.inferred_failures(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocking_issues",
        report.blocking_issues(),
        true,
    );
    push_usize_field(out, indent + 2, "source_errors", report.source_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        report.full_type_check_summary.resolver_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_errors",
        report.full_type_check_summary.type_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "core_verify_errors",
        report.full_type_check_summary.core_verify_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "full_type_check_errors",
        report.full_type_check_errors(),
        true,
    );
    push_usize_field(out, indent + 2, "execution_ready", 0, true);
    push_usize_field(out, indent + 2, "ir_ready", 0, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_items(out: &mut String, items: &[EffectItem], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "effect_items");
    out.push_str(": [");
    if !items.is_empty() {
        out.push('\n');
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_item(out, item, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_item(out: &mut String, item: &EffectItem, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &item.id, true);
    push_string_field(out, indent + 2, "kind", item.kind, true);
    push_string_field(out, indent + 2, "name", &item.name, true);
    push_span_field(out, indent + 2, "source_span", &item.span, true);
    push_string_field(out, indent + 2, "status", item.status, true);
    push_declarations(out, &item.declarations, indent + 2, true);
    push_statements(out, &item.statements, indent + 2, true);
    push_boundary_checks(out, &item.boundary_checks, indent + 2, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_declarations(
    out: &mut String,
    declarations: &EffectDeclarations,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "declarations");
    out.push_str(": {\n");
    push_declared_facts(out, indent + 2, "uses", &declarations.uses, true);
    push_declared_facts(out, indent + 2, "changes", &declarations.changes, true);
    push_declared_facts(out, indent + 2, "failures", &declarations.failures, true);
    push_declared_facts(
        out,
        indent + 2,
        "allocations",
        &declarations.allocations,
        true,
    );
    push_declared_facts(out, indent + 2, "avoids", &declarations.avoids, true);
    push_declared_facts(out, indent + 2, "protects", &declarations.protects, true);
    push_declared_facts(out, indent + 2, "trusts", &declarations.trusts, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_declared_facts(
    out: &mut String,
    indent: usize,
    key: &str,
    facts: &[DeclaredFact],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    if !facts.is_empty() {
        out.push('\n');
        for (index, fact) in facts.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_declared_fact(out, fact, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_declared_fact(out: &mut String, fact: &DeclaredFact, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "section", fact.section, true);
    push_string_field(out, indent + 2, "text", &fact.text, true);
    push_string_field(out, indent + 2, "resource", &fact.resource, true);
    push_span_field(out, indent + 2, "source_span", &fact.span, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_statements(out: &mut String, statements: &[EffectStatement], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "statements");
    out.push_str(": [");
    if !statements.is_empty() {
        out.push('\n');
        for (index, statement) in statements.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_statement(out, statement, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_statement(out: &mut String, statement: &EffectStatement, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &statement.id, true);
    push_span_field(out, indent + 2, "source_span", &statement.span, true);
    push_string_field(
        out,
        indent + 2,
        "statement_kind",
        statement.statement_kind,
        true,
    );
    push_string_field(out, indent + 2, "effect_kind", statement.effect_kind, true);
    push_optional_string_field(out, indent + 2, "target", statement.target.as_deref(), true);
    push_optional_string_field(
        out,
        indent + 2,
        "declaration",
        statement.declaration.as_deref(),
        true,
    );
    push_string_field(out, indent + 2, "status", statement.status, true);
    push_optional_string_field(out, indent + 2, "reason", statement.reason, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_boundary_checks(
    out: &mut String,
    checks: &[EffectBoundaryCheck],
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "boundary_checks");
    out.push_str(": [");
    if !checks.is_empty() {
        out.push('\n');
        for (index, check) in checks.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_boundary_check(out, check, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_boundary_check(out: &mut String, check: &EffectBoundaryCheck, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &check.id, true);
    push_span_field(out, indent + 2, "source_span", &check.span, true);
    push_string_field(out, indent + 2, "check", check.check, true);
    push_string_field(out, indent + 2, "status", check.status, true);
    push_optional_string_field(out, indent + 2, "reason", check.reason, false);
    push_indent(out, indent);
    out.push('}');
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

fn push_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    out.push_str(&value.to_string());
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

    use super::{effect_check_has_errors, effect_check_json, effect_check_text};

    #[test]
    fn json_accepts_declared_local_mutation_and_failure_effects() {
        let program = effect_demo_program(true, true);
        let json = effect_check_json(&program, &[]);

        assert!(!effect_check_has_errors(&program, &[]));
        assert!(json.contains("\"schema\": \"hum.effect_check.v0\""));
        assert!(json.contains("\"status\": \"recognized_core_effects_checked_v0\""));
        assert!(json.contains("\"accepted_declared_failure_v0\""));
        assert!(json.contains("\"accepted_local_mutation_v0\""));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"ir_ready\": 0"));
        assert!(json.contains("\"no complete effect system\""));
    }

    #[test]
    fn json_rejects_missing_failure_declaration() {
        let program = effect_demo_program(false, true);
        let json = effect_check_json(&program, &[]);

        assert!(effect_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"effect_errors_v0\""));
        assert!(json.contains("\"rejected_missing_fails_when_declaration_v0\""));
    }

    #[test]
    fn json_rejects_trust_without_protection() {
        let program = effect_demo_program(true, false);
        let json = effect_check_json(&program, &[]);

        assert!(effect_check_has_errors(&program, &[]));
        assert!(json.contains("\"rejected_trust_without_protects_v0\""));
    }

    #[test]
    fn text_reports_effect_gate_without_safety_claims() {
        let program = effect_demo_program(true, true);
        let text = effect_check_text(&program, &[]);

        assert!(text.contains("Hum effect check (hum.effect_check.v0)"));
        assert!(text.contains("status: recognized_core_effects_checked_v0"));
        assert!(text.contains("no memory-safety proof"));
    }

    fn effect_demo_program(declare_failure: bool, protect_trust: bool) -> Program {
        let failure = if declare_failure {
            r#"
  fails when:
    flag is false
"#
        } else {
            ""
        };
        let protection = if protect_trust {
            r#"
  protects:
    retry state stays bounded
"#
        } else {
            ""
        };
        let source = format!(
            r#"type WorkError {{
  code: Text
}}

task retry(flag: Bool) -> Result UInt, WorkError {{
  why:
    keep the effect gate small
{failure}
{protection}
  trusts:
    caller gives a stable flag

  does:
    change attempts: UInt = 0
    if flag == false {{
      fail WorkError.no_flag
    }}

    set attempts = attempts + 1
    return attempts
}}
"#
        );
        Program {
            files: vec![parse_source("effect_demo.hum", &source).file],
        }
    }
}
