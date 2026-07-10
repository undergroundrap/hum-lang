use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{Item, ParamPermission, Program, Section};
use crate::core_body::{self, BodyStatement};
use crate::core_contract;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity, Span};
use crate::effect_check;
use crate::element_place;
use crate::field_place;
use crate::graph::is_meaningful_line_text;
use crate::return_dependency::{self, ReturnDependency};
use crate::version;
use crate::writable_field_alias::{self, AliasAnalysis, AliasBinding, AliasIssueKind};

pub const OWNERSHIP_CHECK_SCHEMA: &str = "hum.ownership_check.v0";
pub const OWNERSHIP_CHECK_MODE: &str = "recognized_core_ownership_gate_v0";
pub const OWNERSHIP_CHECK_STATUS: &str = "recognized_core_ownership_gate_available_v0";

const NON_CLAIMS: &[&str] = &[
    "no executable semantics",
    "no Hum IR emission",
    "no backend lowering",
    "no proof artifact",
    "no memory-safety proof",
    "no complete ownership system",
    "no complete borrow checker",
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
pub struct OwnershipCheckSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub source_errors: usize,
    pub resolver_errors: usize,
    pub type_errors: usize,
    pub core_verify_errors: usize,
    pub full_type_check_errors: usize,
    pub effect_check_errors: usize,
    pub items: usize,
    pub ownership_items: usize,
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

struct OwnershipCheckReport {
    effect_check_summary: effect_check::EffectCheckSummary,
    items: Vec<OwnershipItem>,
    files: usize,
    item_count: usize,
    source_errors: usize,
}

struct OwnershipItem {
    id: String,
    kind: &'static str,
    name: String,
    span: Span,
    status: &'static str,
    declarations: OwnershipDeclarations,
    statements: Vec<OwnershipStatement>,
    return_dependencies: Vec<OwnershipReturnDependency>,
    boundary_checks: Vec<OwnershipBoundaryCheck>,
}

#[derive(Default)]
struct OwnershipDeclarations {
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

#[derive(Default)]
struct LocalOwnershipFacts {
    parameters: BTreeMap<String, ParamPermission>,
    immutable_locals: BTreeSet<String>,
    mutable_locals: BTreeSet<String>,
    writable_aliases: BTreeMap<String, AliasBinding>,
    duplicate_locals: BTreeSet<String>,
}

#[derive(Default)]
struct MoveTracker {
    moved: BTreeMap<String, MoveSite>,
}

#[derive(Clone)]
struct MoveSite {
    span: Span,
    kind: String,
}

#[derive(Clone)]
struct LinearResourceSite {
    span: Span,
    type_text: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewKind {
    Field,
    Element,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewInvalidationKind {
    FieldWrite,
    ListAppend,
}

#[derive(Clone)]
struct ViewInvalidation {
    span: Span,
    kind: ViewInvalidationKind,
}

#[derive(Clone)]
struct FieldViewSite {
    kind: ViewKind,
    source_place: String,
    bound_span: Span,
    invalidated_by: Option<ViewInvalidation>,
}

#[derive(Clone, Default)]
struct PathState {
    moved: BTreeMap<String, MoveSite>,
    linear_resources: BTreeMap<String, LinearResourceSite>,
    open_linear_roots: BTreeSet<String>,
    field_views: BTreeMap<String, FieldViewSite>,
    path: Vec<String>,
}

struct PathDiagnostic {
    index: usize,
    ownership_kind: &'static str,
    target: Option<String>,
    status: &'static str,
    reason: &'static str,
    diagnostic_code: &'static str,
    help: String,
}

struct OwnershipStatement {
    id: String,
    span: Span,
    statement_kind: &'static str,
    ownership_kind: &'static str,
    target: Option<String>,
    declaration: Option<String>,
    status: &'static str,
    reason: Option<&'static str>,
    diagnostic_code: Option<&'static str>,
    help: Option<String>,
    alias: Option<String>,
    place: Option<String>,
    binding_span: Option<Span>,
    last_use_span: Option<Span>,
    conflict_place: Option<String>,
    conflict_span: Option<Span>,
}

struct OwnershipReturnDependency {
    id: String,
    span: Span,
    result_type: String,
    source: String,
    source_kind: &'static str,
    status: &'static str,
    reason: Option<&'static str>,
    diagnostic_code: Option<&'static str>,
    help: Option<String>,
}

struct OwnershipBoundaryCheck {
    id: String,
    span: Span,
    check: &'static str,
    status: &'static str,
    reason: Option<&'static str>,
}

pub fn ownership_check_has_errors(program: &Program, diagnostics: &[Diagnostic]) -> bool {
    ownership_check_summary(program, diagnostics).blocking_issues > 0
}

pub fn ownership_check_summary(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> OwnershipCheckSummary {
    let report = build_report(program, diagnostics);
    OwnershipCheckSummary {
        schema: OWNERSHIP_CHECK_SCHEMA,
        status: report.status(),
        mode: OWNERSHIP_CHECK_MODE,
        source_errors: report.source_errors,
        resolver_errors: report.effect_check_summary.resolver_errors,
        type_errors: report.effect_check_summary.type_errors,
        core_verify_errors: report.effect_check_summary.core_verify_errors,
        full_type_check_errors: report.effect_check_summary.full_type_check_errors,
        effect_check_errors: report.effect_check_errors(),
        items: report.item_count(),
        ownership_items: report.items.len(),
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

pub fn ownership_check_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum ownership check ({OWNERSHIP_CHECK_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {OWNERSHIP_CHECK_MODE}\n"));
    out.push_str(&format!("status: {}\n", report.status()));
    out.push_str(&format!(
        "dependencies: core_contract={} effect_check={}\n",
        core_contract::CORE_CONTRACT_SCHEMA,
        effect_check::EFFECT_CHECK_SCHEMA
    ));
    out.push_str(&format!(
        "summary: files={} items={} ownership_items={} statements={} checked_statements={} accepted_statements={} rejected_statements={} unchecked_statements={} boundary_checks={} rejected_boundary_checks={} declared_uses={} declared_changes={} declared_failures={} declared_allocations={} declared_avoids={} declared_protects={} declared_trusts={} inferred_reads={} inferred_changes={} inferred_failures={} blocking_issues={} source_errors={} resolver_errors={} type_errors={} core_verify_errors={} full_type_check_errors={} effect_check_errors={} execution_ready=0 ir_ready=0\n",
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
        report.effect_check_summary.resolver_errors,
        report.effect_check_summary.type_errors,
        report.effect_check_summary.core_verify_errors,
        report.effect_check_summary.full_type_check_errors,
        report.effect_check_errors()
    ));

    if report.items.is_empty() {
        out.push_str("ownership_items: none\n");
    } else {
        out.push_str("ownership_items:\n");
        for item in &report.items {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} `{}` statements={} return_dependencies={} boundary_checks={}\n",
                item.span.file,
                item.span.line,
                item.span.column,
                item.status,
                item.kind,
                item.name,
                item.statements.len(),
                item.return_dependencies.len(),
                item.boundary_checks.len()
            ));
            for statement in &item.statements {
                out.push_str(&format!(
                    "    {}:{}:{} [{}] {} ownership={} target={}",
                    statement.span.file,
                    statement.span.line,
                    statement.span.column,
                    statement.status,
                    statement.statement_kind,
                    statement.ownership_kind,
                    statement.target.as_deref().unwrap_or("none")
                ));
                if let Some(declaration) = &statement.declaration {
                    out.push_str(&format!(" declaration={declaration}"));
                }
                if let Some(reason) = statement.reason {
                    out.push_str(&format!(" reason={reason}"));
                }
                if let Some(diagnostic_code) = statement.diagnostic_code {
                    out.push_str(&format!(" diagnostic={diagnostic_code}"));
                }
                if let Some(alias) = &statement.alias {
                    out.push_str(&format!(" alias={alias}"));
                }
                if let Some(place) = &statement.place {
                    out.push_str(&format!(" place={place}"));
                }
                if let Some(span) = &statement.binding_span {
                    out.push_str(&format!(
                        " binding_span={}:{}:{}",
                        span.file, span.line, span.column
                    ));
                }
                if let Some(span) = &statement.last_use_span {
                    out.push_str(&format!(
                        " last_use_span={}:{}:{}",
                        span.file, span.line, span.column
                    ));
                }
                if let Some(place) = &statement.conflict_place {
                    out.push_str(&format!(" conflict_place={place}"));
                }
                if let Some(span) = &statement.conflict_span {
                    out.push_str(&format!(
                        " conflict_span={}:{}:{}",
                        span.file, span.line, span.column
                    ));
                }
                if let Some(help) = &statement.help {
                    out.push_str(&format!(" help={help}"));
                }
                out.push('\n');
            }
            for dependency in &item.return_dependencies {
                out.push_str(&format!(
                    "    {}:{}:{} [{}] return_dependency result_type={} source={} source_kind={} reason={}",
                    dependency.span.file,
                    dependency.span.line,
                    dependency.span.column,
                    dependency.status,
                    dependency.result_type,
                    dependency.source,
                    dependency.source_kind,
                    dependency.reason.unwrap_or("none")
                ));
                if let Some(diagnostic_code) = dependency.diagnostic_code {
                    out.push_str(&format!(" diagnostic={diagnostic_code}"));
                }
                if let Some(help) = &dependency.help {
                    out.push_str(&format!(" help={help}"));
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

pub fn ownership_check_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", OWNERSHIP_CHECK_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", OWNERSHIP_CHECK_MODE, true);
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
        "effect_check_schema",
        effect_check::EFFECT_CHECK_SCHEMA,
        true,
    );
    push_dependency_summary(&mut out, &report, 2, true);
    push_summary(&mut out, &report, 2, true);
    push_items(&mut out, &report.items, 2, true);
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> OwnershipCheckReport {
    let effect_check_summary = effect_check::effect_check_summary(program, diagnostics);
    let source_errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let blocked = source_errors > 0 || effect_check_summary.blocking_issues > 0;
    let mut items = Vec::new();
    for file in &program.files {
        collect_items(&file.items, blocked, &mut items);
    }

    OwnershipCheckReport {
        effect_check_summary,
        items,
        files: program.files.len(),
        item_count: count_items(program),
        source_errors,
    }
}

fn collect_items(items: &[Item], blocked: bool, out: &mut Vec<OwnershipItem>) {
    for item in items {
        if let Some(ownership_item) = check_item(item, blocked) {
            out.push(ownership_item);
        }
        if let Item::App(app) = item {
            collect_items(&app.items, blocked, out);
        }
    }
}

fn check_item(item: &Item, blocked: bool) -> Option<OwnershipItem> {
    let does = item_sections(item)
        .iter()
        .find(|section| section.name == "does")?;
    let body = core_body::analyze_does_section(does);
    let declarations = collect_declarations(item_sections(item));
    let mut existing_names = item_parameters(item)
        .into_iter()
        .map(|parameter| first_resource(&parameter.name))
        .collect::<BTreeSet<_>>();
    existing_names.extend(
        declarations
            .uses
            .iter()
            .chain(&declarations.changes)
            .map(|fact| fact.resource.clone()),
    );
    let alias_analysis = writable_field_alias::analyze_item(
        &body.statements,
        matches!(item, Item::Task(_)),
        &existing_names,
    );
    let ownership_facts = local_ownership_facts(item, &body.statements, &alias_analysis);
    let mut statements = Vec::new();
    for (index, statement) in body.statements.iter().enumerate() {
        let mut move_tracker = MoveTracker::default();
        statements.push(check_statement_ownership(
            item.name(),
            statement,
            index,
            &declarations,
            &ownership_facts,
            &mut move_tracker,
            blocked,
        ));
    }
    if !blocked {
        apply_path_diagnostics(
            &mut statements,
            ownership_path_diagnostics(item.name(), &body.statements, &ownership_facts),
        );
        apply_alias_diagnostics(
            &mut statements,
            item.name(),
            &alias_analysis,
            &ownership_facts,
        );
    }
    let return_dependencies = return_dependency_facts(item, &ownership_facts, &body.statements);
    let boundary_checks = boundary_checks(
        item,
        &declarations,
        &ownership_facts,
        &body.statements,
        &statements,
    );
    let status = item_status(&statements, &return_dependencies, &boundary_checks, blocked);
    Some(OwnershipItem {
        id: prefixed_id(
            "hum_ownership_item",
            &format!("{}_{}_{}", item.kind(), item.name(), item.span().line),
        ),
        kind: item.kind(),
        name: item.name().to_string(),
        span: portable_span(item.span()),
        status,
        declarations,
        statements,
        return_dependencies,
        boundary_checks,
    })
}

fn check_statement_ownership(
    item_name: &str,
    statement: &BodyStatement,
    index: usize,
    declarations: &OwnershipDeclarations,
    ownership_facts: &LocalOwnershipFacts,
    move_tracker: &mut MoveTracker,
    blocked: bool,
) -> OwnershipStatement {
    if blocked {
        return ownership_statement(
            statement,
            index,
            "prior_blocker",
            None,
            None,
            "not_checked_blocked_by_prior_errors_v0",
            Some("source_or_effect_check_errors"),
        );
    }

    if statement.status == "unsupported_v0" {
        return ownership_statement(
            statement,
            index,
            "unsupported_statement",
            None,
            None,
            "unchecked_statement_ownership_v0",
            statement
                .reason
                .or(Some("statement_not_in_core_body_grammar_v0")),
        );
    }

    if let Some((target, move_site)) = moved_value_use(statement, move_tracker) {
        return ownership_statement_with_diagnostic(
            ownership_statement(
                statement,
                index,
                "use_after_move",
                Some(target.clone()),
                None,
                "rejected_use_after_move_v0",
                Some("value_used_after_move_v0"),
            ),
            Some(DiagnosticCode::USE_AFTER_MOVE.as_str()),
            Some(move_help(item_name, &target, move_site)),
        );
    }

    let ownership = match statement.kind {
        "fail" => {
            if declarations.failures.is_empty() {
                ownership_statement(
                    statement,
                    index,
                    "no_ownership_transfer",
                    expression_text_for_statement(statement).map(str::to_string),
                    None,
                    "rejected_missing_fails_when_declaration_v0",
                    Some("fail_statement_requires_fails_when_section"),
                )
            } else {
                ownership_statement(
                    statement,
                    index,
                    "no_ownership_transfer",
                    expression_text_for_statement(statement).map(str::to_string),
                    Some("fails when".to_string()),
                    "accepted_no_ownership_transfer_v0",
                    None,
                )
            }
        }
        "let_binding" => {
            check_binding_statement(item_name, statement, index, ownership_facts, false)
        }
        "mutable_binding" => {
            check_binding_statement(item_name, statement, index, ownership_facts, true)
        }
        "set_place" => {
            check_set_statement(item_name, statement, index, declarations, ownership_facts)
        }
        "save_in_store" => check_save_statement(statement, index, declarations),
        "test_expectation" => ownership_statement(
            statement,
            index,
            "test_expectation",
            None,
            None,
            "unchecked_statement_ownership_v0",
            Some("test_expectation_ownerships_not_checked_v0"),
        ),
        "for_each_header" | "for_index_header" => ownership_statement(
            statement,
            index,
            "iteration",
            expression_text_for_statement(statement).map(str::to_string),
            None,
            "unchecked_statement_ownership_v0",
            Some("iterator_ownerships_not_checked_v0"),
        ),
        "nested_intent_header" => ownership_statement(
            statement,
            index,
            "nested_intent",
            None,
            None,
            "unchecked_statement_ownership_v0",
            Some("nested_intent_ownerships_not_checked_v0"),
        ),
        _ => expression_or_pure_ownership(statement, index, declarations, ownership_facts),
    };

    if !is_rejected_statement(&ownership) && !is_unchecked_statement(&ownership) {
        record_statement_moves(statement, ownership_facts, move_tracker);
    }

    ownership
}

fn check_binding_statement(
    item_name: &str,
    statement: &BodyStatement,
    index: usize,
    ownership_facts: &LocalOwnershipFacts,
    mutable: bool,
) -> OwnershipStatement {
    let Some(target) = binding_name(statement).map(|name| first_resource(&name)) else {
        return ownership_statement(
            statement,
            index,
            "local_binding",
            None,
            None,
            "unchecked_statement_ownership_v0",
            Some("binding_name_unknown_v0"),
        );
    };
    if ownership_facts.duplicate_locals.contains(&target) {
        return ownership_statement(
            statement,
            index,
            "duplicate_local_place",
            Some(target),
            None,
            "rejected_duplicate_local_place_v0",
            Some("local_place_names_must_be_unique_in_v0"),
        );
    }
    if let Some(binding) = ownership_facts.writable_aliases.get(&target) {
        let authority = writable_alias_authority(ownership_facts, binding);
        let statement = match authority {
            WritableAliasAuthority::Accepted(declaration) => ownership_statement(
                statement,
                index,
                "writable_field_alias",
                Some(binding.name.clone()),
                Some(format!("{declaration} {}", binding.source_place)),
                "accepted_writable_field_alias_v0",
                None,
            ),
            WritableAliasAuthority::Borrow => ownership_statement_with_diagnostic(
                ownership_statement(
                    statement,
                    index,
                    "writable_field_alias",
                    Some(binding.name.clone()),
                    Some(format!("change {}", binding.source_place)),
                    "rejected_writable_field_alias_without_mutation_authority_v0",
                    Some("borrow_owner_cannot_supply_writable_field_alias_v0"),
                ),
                Some(DiagnosticCode::BORROW_PARAMETER_MUTATION.as_str()),
                Some(writable_field_alias::authority_help(
                    item_name, binding, "borrow",
                )),
            ),
            WritableAliasAuthority::Immutable => ownership_statement_with_diagnostic(
                ownership_statement(
                    statement,
                    index,
                    "writable_field_alias",
                    Some(binding.name.clone()),
                    Some(format!("change {}", binding.source_place)),
                    "rejected_unsupported_writable_field_alias_v0",
                    Some("immutable_owner_cannot_supply_writable_field_alias_v0"),
                ),
                Some(DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS.as_str()),
                Some(writable_field_alias::authority_help(
                    item_name,
                    binding,
                    "immutable let",
                )),
            ),
            WritableAliasAuthority::Unknown => ownership_statement_with_diagnostic(
                ownership_statement(
                    statement,
                    index,
                    "writable_field_alias",
                    Some(binding.name.clone()),
                    Some(format!("change {}", binding.source_place)),
                    "rejected_unsupported_writable_field_alias_v0",
                    Some("writable_alias_owner_authority_unknown_v0"),
                ),
                Some(DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS.as_str()),
                Some(writable_field_alias::authority_help(
                    item_name, binding, "unknown",
                )),
            ),
        };
        return ownership_statement_with_alias_facts(statement, binding);
    }
    if mutable {
        ownership_statement(
            statement,
            index,
            "mutable_local_owner",
            Some(target),
            Some("change".to_string()),
            "accepted_mutable_local_owner_v0",
            None,
        )
    } else if let Some((_view_name, source_place)) = field_view_binding(statement) {
        ownership_statement(
            statement,
            index,
            "field_view_borrow",
            Some(target),
            Some(format!("borrow {source_place}")),
            "accepted_field_view_borrow_v0",
            None,
        )
    } else if let Some((_view_name, source_place)) = element_view_binding(statement) {
        ownership_statement(
            statement,
            index,
            "element_view_borrow",
            Some(target),
            Some(format!("borrow {source_place}")),
            "accepted_element_view_borrow_v0",
            None,
        )
    } else {
        ownership_statement(
            statement,
            index,
            "immutable_local_owner",
            Some(target),
            Some("let".to_string()),
            "accepted_immutable_local_owner_v0",
            None,
        )
    }
}

fn check_set_statement(
    item_name: &str,
    statement: &BodyStatement,
    index: usize,
    declarations: &OwnershipDeclarations,
    ownership_facts: &LocalOwnershipFacts,
) -> OwnershipStatement {
    let Some(target) = set_place_name(statement) else {
        return ownership_statement(
            statement,
            index,
            "mutation",
            None,
            None,
            "unchecked_statement_ownership_v0",
            Some("set_target_unknown_v0"),
        );
    };
    let resource = first_resource(&target);
    if let Some(binding) = ownership_facts.writable_aliases.get(&resource) {
        ownership_statement_with_alias_facts(
            ownership_statement(
                statement,
                index,
                "writable_field_alias_write_through",
                Some(binding.source_place.clone()),
                Some(format!("change {}", binding.source_place)),
                "accepted_writable_field_alias_write_through_v0",
                None,
            ),
            binding,
        )
    } else if ownership_facts.mutable_locals.contains(&resource) {
        accepted_set_statement(
            statement,
            index,
            target,
            resource,
            "change".to_string(),
            "local_mutation",
            "accepted_exclusive_local_mutation_v0",
        )
    } else if ownership_facts.immutable_locals.contains(&resource) {
        let display_target = if field_place::split_field_place(&target).is_some() {
            target
        } else {
            resource
        };
        ownership_statement(
            statement,
            index,
            "immutable_local_mutation",
            Some(display_target),
            None,
            "rejected_mutating_immutable_local_v0",
            Some("set_requires_change_local_not_let_binding"),
        )
    } else if let Some(permission) = ownership_facts.parameters.get(&resource).copied() {
        match permission {
            ParamPermission::Borrow => ownership_statement_with_diagnostic(
                ownership_statement(
                    statement,
                    index,
                    "parameter_mutation",
                    Some(target.clone()),
                    Some(permission.as_str().to_string()),
                    "rejected_mutating_borrowed_parameter_v0",
                    Some("borrow_parameter_requires_change_permission_for_set_v0"),
                ),
                Some(DiagnosticCode::BORROW_PARAMETER_MUTATION.as_str()),
                Some(borrow_mutation_help(
                    item_name, &target, &resource, statement,
                )),
            ),
            ParamPermission::Change | ParamPermission::Consume => accepted_set_statement(
                statement,
                index,
                target,
                resource,
                permission.as_str().to_string(),
                "parameter_mutation",
                "accepted_parameter_mutation_v0",
            ),
        }
    } else if declares_resource(&declarations.changes, &resource) {
        ownership_statement(
            statement,
            index,
            "external_change_deferred",
            Some(resource),
            Some("changes".to_string()),
            "accepted_external_change_deferred_to_resource_check_v0",
            None,
        )
    } else {
        ownership_statement(
            statement,
            index,
            "external_change_deferred",
            Some(target),
            None,
            "rejected_missing_mutation_authority_v0",
            Some("set_statement_requires_change_binding_or_changes_section"),
        )
    }
}

fn accepted_set_statement(
    statement: &BodyStatement,
    index: usize,
    target: String,
    resource: String,
    declaration: String,
    root_kind: &'static str,
    root_status: &'static str,
) -> OwnershipStatement {
    if field_place::split_field_place(&target).is_some() {
        ownership_statement(
            statement,
            index,
            "field_mutation",
            Some(target),
            Some(declaration),
            "accepted_disjoint_field_mutation_v0",
            None,
        )
    } else {
        ownership_statement(
            statement,
            index,
            root_kind,
            Some(resource),
            Some(declaration),
            root_status,
            None,
        )
    }
}

fn check_save_statement(
    statement: &BodyStatement,
    index: usize,
    declarations: &OwnershipDeclarations,
) -> OwnershipStatement {
    let Some(target) = save_target(&statement.text) else {
        return ownership_statement(
            statement,
            index,
            "store_change",
            None,
            None,
            "unchecked_statement_ownership_v0",
            Some("save_target_unknown_v0"),
        );
    };
    let resource = first_resource(target);
    if declares_resource(&declarations.changes, &resource) {
        ownership_statement(
            statement,
            index,
            "store_change",
            Some(resource),
            Some("changes".to_string()),
            "accepted_external_change_deferred_to_resource_check_v0",
            None,
        )
    } else {
        ownership_statement(
            statement,
            index,
            "store_change",
            Some(resource),
            None,
            "rejected_missing_mutation_authority_v0",
            Some("save_statement_requires_changes_section"),
        )
    }
}

fn expression_or_pure_ownership(
    statement: &BodyStatement,
    index: usize,
    declarations: &OwnershipDeclarations,
    ownership_facts: &LocalOwnershipFacts,
) -> OwnershipStatement {
    let expression = expression_text_for_statement(statement);
    if let Some(resource) = expression
        .and_then(first_consume_move_root)
        .filter(|resource| ownership_facts.is_movable_root(resource))
    {
        return ownership_statement(
            statement,
            index,
            "consume_argument_move",
            Some(resource),
            Some("consume".to_string()),
            "accepted_consume_argument_move_v0",
            None,
        );
    }
    if let Some(resource) = returned_move_root(statement, ownership_facts) {
        return ownership_statement(
            statement,
            index,
            "return_move",
            Some(resource),
            Some("return".to_string()),
            "accepted_return_move_v0",
            None,
        );
    }
    if let Some(resource) = expression.and_then(first_ambient_resource) {
        if declares_resource(&declarations.uses, &resource) {
            ownership_statement(
                statement,
                index,
                "ambient_read",
                Some(resource),
                Some("uses".to_string()),
                "accepted_no_ownership_transfer_v0",
                None,
            )
        } else {
            ownership_statement(
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
        ownership_statement(
            statement,
            index,
            "pure_or_local",
            expression.map(str::to_string),
            None,
            "accepted_no_ownership_transfer_v0",
            None,
        )
    }
}

fn ownership_statement(
    statement: &BodyStatement,
    index: usize,
    ownership_kind: &'static str,
    target: Option<String>,
    declaration: Option<String>,
    status: &'static str,
    reason: Option<&'static str>,
) -> OwnershipStatement {
    OwnershipStatement {
        id: prefixed_id(
            "hum_ownership_stmt",
            &format!("{}_{}_{}", statement.kind, statement.span.line, index),
        ),
        span: portable_span(&statement.span),
        statement_kind: statement.kind,
        ownership_kind,
        target,
        declaration,
        status,
        reason,
        diagnostic_code: None,
        help: None,
        alias: None,
        place: None,
        binding_span: None,
        last_use_span: None,
        conflict_place: None,
        conflict_span: None,
    }
}

fn ownership_statement_with_alias_facts(
    mut statement: OwnershipStatement,
    binding: &AliasBinding,
) -> OwnershipStatement {
    statement.alias = Some(binding.name.clone());
    statement.place = Some(binding.source_place.clone());
    statement.binding_span = Some(portable_span(&binding.binding_span));
    statement.last_use_span = Some(portable_span(&binding.last_use_span));
    statement
}

fn ownership_statement_with_diagnostic(
    mut statement: OwnershipStatement,
    diagnostic_code: Option<&'static str>,
    help: Option<String>,
) -> OwnershipStatement {
    statement.diagnostic_code = diagnostic_code;
    statement.help = help;
    statement
}

fn apply_path_diagnostics(
    statements: &mut [OwnershipStatement],
    diagnostics: BTreeMap<usize, PathDiagnostic>,
) {
    for (_index, diagnostic) in diagnostics {
        if let Some(statement) = statements.get_mut(diagnostic.index) {
            statement.ownership_kind = diagnostic.ownership_kind;
            statement.target = diagnostic.target;
            statement.declaration = None;
            statement.status = diagnostic.status;
            statement.reason = Some(diagnostic.reason);
            statement.diagnostic_code = Some(diagnostic.diagnostic_code);
            statement.help = Some(diagnostic.help);
        }
    }
}

fn apply_alias_diagnostics(
    statements: &mut [OwnershipStatement],
    item_name: &str,
    analysis: &AliasAnalysis,
    ownership_facts: &LocalOwnershipFacts,
) {
    for issue in &analysis.issues {
        if let Some(binding) = ownership_facts.writable_aliases.get(&issue.alias_name)
            && !matches!(
                writable_alias_authority(ownership_facts, binding),
                WritableAliasAuthority::Accepted(_)
            )
            && issue.index != binding.binding_index
        {
            continue;
        }
        let Some(statement) = statements.get_mut(issue.index) else {
            continue;
        };
        if statement.diagnostic_code == Some(DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS.as_str())
            && issue.kind == AliasIssueKind::Overlap
        {
            continue;
        }
        let (ownership_kind, status, code, message) = match issue.kind {
            AliasIssueKind::Overlap => (
                "writable_field_alias_overlap",
                "rejected_writable_field_alias_overlap_v0",
                DiagnosticCode::WRITABLE_ALIAS_OVERLAP.as_str(),
                writable_field_alias::overlap_message(issue),
            ),
            AliasIssueKind::Unsupported => (
                "unsupported_writable_field_alias",
                "rejected_unsupported_writable_field_alias_v0",
                DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS.as_str(),
                writable_field_alias::unsupported_message(issue),
            ),
        };
        statement.ownership_kind = ownership_kind;
        statement.target = Some(issue.alias_name.clone());
        statement.declaration = Some(format!("change {}", issue.source_place));
        statement.status = status;
        statement.reason = Some(issue.reason);
        statement.diagnostic_code = Some(code);
        statement.help = Some(format!(
            "{}. {}",
            message,
            writable_field_alias::issue_help(item_name, issue)
        ));
        statement.alias = Some(issue.alias_name.clone());
        statement.place = Some(issue.source_place.clone());
        statement.binding_span = Some(portable_span(&issue.binding_span));
        statement.last_use_span = Some(portable_span(&issue.last_use_span));
        statement.conflict_place = Some(issue.conflict_place.clone());
        statement.conflict_span = Some(portable_span(&issue.conflict_span));
    }
}

fn ownership_path_diagnostics(
    item_name: &str,
    statements: &[BodyStatement],
    ownership_facts: &LocalOwnershipFacts,
) -> BTreeMap<usize, PathDiagnostic> {
    let mut diagnostics = BTreeMap::new();
    let states = analyze_statement_range(
        item_name,
        statements,
        0,
        statements.len(),
        vec![PathState::default()],
        ownership_facts,
        &mut diagnostics,
    );
    if let Some(exit_index) = last_non_close_statement(statements) {
        for state in states {
            record_missing_linear_diagnostics(
                item_name,
                statements,
                exit_index,
                "fallthrough",
                &state,
                &mut diagnostics,
            );
        }
    }
    diagnostics
}

fn analyze_statement_range(
    item_name: &str,
    statements: &[BodyStatement],
    start: usize,
    end: usize,
    mut states: Vec<PathState>,
    ownership_facts: &LocalOwnershipFacts,
    diagnostics: &mut BTreeMap<usize, PathDiagnostic>,
) -> Vec<PathState> {
    let mut index = start;
    while index < end {
        let statement = &statements[index];
        if statement.kind == "block_close" {
            index += 1;
            continue;
        }
        if statement.kind == "if_header"
            && let Some(close) = matching_statement_close(statements, index)
            && close <= end
        {
            let true_states = states
                .iter()
                .cloned()
                .map(|mut state| {
                    state
                        .path
                        .push(format!("if line {} true", statement.span.line));
                    state
                })
                .collect::<Vec<_>>();
            let false_states = states
                .into_iter()
                .map(|mut state| {
                    state
                        .path
                        .push(format!("if line {} false", statement.span.line));
                    state
                })
                .collect::<Vec<_>>();
            let mut next_states = analyze_statement_range(
                item_name,
                statements,
                index + 1,
                close,
                true_states,
                ownership_facts,
                diagnostics,
            );
            next_states.extend(false_states);
            states = next_states;
            index = close + 1;
            continue;
        }

        states = analyze_single_statement(
            item_name,
            statements,
            index,
            states,
            ownership_facts,
            diagnostics,
        );
        if states.is_empty() {
            break;
        }
        index += 1;
    }
    states
}

fn analyze_single_statement(
    item_name: &str,
    statements: &[BodyStatement],
    index: usize,
    states: Vec<PathState>,
    ownership_facts: &LocalOwnershipFacts,
    diagnostics: &mut BTreeMap<usize, PathDiagnostic>,
) -> Vec<PathState> {
    let statement = &statements[index];
    let mut next_states = Vec::new();
    for mut state in states {
        if statement.status == "unsupported_v0" {
            next_states.push(state);
            continue;
        }

        let consume_roots = consume_move_roots(statement);
        let mut rejected = false;
        for root in &consume_roots {
            if let Some(move_site) = state.moved.get(root) {
                record_path_diagnostic(
                    diagnostics,
                    if state.linear_resources.contains_key(root) {
                        linear_double_consume_diagnostic(
                            item_name, statements, index, root, move_site, &state,
                        )
                    } else {
                        use_after_move_diagnostic(item_name, index, root, move_site, &state)
                    },
                );
                rejected = true;
                break;
            }
        }
        if rejected {
            continue;
        }

        for root in statement_roots(statement) {
            if let Some(move_site) = state.moved.get(&root) {
                record_path_diagnostic(
                    diagnostics,
                    use_after_move_diagnostic(item_name, index, &root, move_site, &state),
                );
                rejected = true;
                break;
            }
        }
        if rejected {
            continue;
        }

        if let Some((view_name, site)) = stale_field_view_use(statement, &state) {
            record_path_diagnostic(
                diagnostics,
                stale_field_view_diagnostic(item_name, statements, index, &view_name, site, &state),
            );
            continue;
        }

        let consume_action = consume_action(statement);
        for root in consume_roots {
            if ownership_facts.is_movable_root(&root) {
                state.moved.entry(root.clone()).or_insert_with(|| MoveSite {
                    span: portable_span(&statement.span),
                    kind: consume_action.clone(),
                });
                state.open_linear_roots.remove(&root);
            }
        }

        if let Some((root, type_text)) = linear_binding(statement) {
            state.linear_resources.insert(
                root.clone(),
                LinearResourceSite {
                    span: portable_span(&statement.span),
                    type_text,
                },
            );
            state.open_linear_roots.insert(root);
        }

        if let Some((view_name, source_place)) = field_view_binding(statement) {
            state.field_views.insert(
                view_name,
                FieldViewSite {
                    kind: ViewKind::Field,
                    source_place,
                    bound_span: portable_span(&statement.span),
                    invalidated_by: None,
                },
            );
        }

        if let Some((view_name, source_place)) = element_view_binding(statement) {
            state.field_views.insert(
                view_name,
                FieldViewSite {
                    kind: ViewKind::Element,
                    source_place,
                    bound_span: portable_span(&statement.span),
                    invalidated_by: None,
                },
            );
        }

        if let Some(target) = set_place_name(statement) {
            let target_root = first_resource(&target);
            let effective_target = ownership_facts
                .writable_aliases
                .get(&target_root)
                .map_or(target.as_str(), |binding| binding.source_place.as_str());
            if field_place::split_field_place(effective_target).is_some() {
                invalidate_path_field_views(&mut state, effective_target, &statement.span);
            }
        }
        if let Some(root) = list_append_change_root(statement) {
            invalidate_path_element_views(&mut state, &root, &statement.span);
        }

        if let Some(root) = returned_move_root(statement, ownership_facts) {
            state.moved.entry(root).or_insert_with(|| MoveSite {
                span: portable_span(&statement.span),
                kind: "return".to_string(),
            });
        }

        if matches!(statement.kind, "return" | "fail") {
            record_missing_linear_diagnostics(
                item_name,
                statements,
                index,
                statement.kind,
                &state,
                diagnostics,
            );
            continue;
        }

        next_states.push(state);
    }
    next_states
}

fn record_path_diagnostic(
    diagnostics: &mut BTreeMap<usize, PathDiagnostic>,
    diagnostic: PathDiagnostic,
) {
    diagnostics.entry(diagnostic.index).or_insert(diagnostic);
}

fn record_missing_linear_diagnostics(
    item_name: &str,
    statements: &[BodyStatement],
    index: usize,
    exit_kind: &str,
    state: &PathState,
    diagnostics: &mut BTreeMap<usize, PathDiagnostic>,
) {
    for root in &state.open_linear_roots {
        let Some(site) = state.linear_resources.get(root) else {
            continue;
        };
        record_path_diagnostic(
            diagnostics,
            PathDiagnostic {
                index,
                ownership_kind: "linear_resource_missing_consume",
                target: Some(root.clone()),
                status: "rejected_linear_resource_not_consumed_v0",
                reason: "linear_resource_must_be_consumed_exactly_once_on_every_path_v0",
                diagnostic_code: DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED.as_str(),
                help: linear_missing_help(
                    item_name,
                    root,
                    site,
                    &statements[index],
                    exit_kind,
                    state,
                ),
            },
        );
    }
}

fn use_after_move_diagnostic(
    item_name: &str,
    index: usize,
    target: &str,
    move_site: &MoveSite,
    state: &PathState,
) -> PathDiagnostic {
    PathDiagnostic {
        index,
        ownership_kind: "use_after_move",
        target: Some(target.to_string()),
        status: "rejected_use_after_move_v0",
        reason: "value_used_after_move_v0",
        diagnostic_code: DiagnosticCode::USE_AFTER_MOVE.as_str(),
        help: path_move_help(item_name, target, move_site, state),
    }
}

fn stale_field_view_diagnostic(
    item_name: &str,
    statements: &[BodyStatement],
    index: usize,
    view_name: &str,
    site: &FieldViewSite,
    state: &PathState,
) -> PathDiagnostic {
    let (ownership_kind, status, reason) = match site.kind {
        ViewKind::Field => (
            "stale_field_view_use",
            "rejected_stale_field_view_use_v0",
            "field_view_invalidated_by_exact_field_write_v0",
        ),
        ViewKind::Element => (
            "stale_element_view_use",
            "rejected_stale_element_view_use_v0",
            "element_view_invalidated_by_list_growth_v0",
        ),
    };
    PathDiagnostic {
        index,
        ownership_kind,
        target: Some(view_name.to_string()),
        status,
        reason,
        diagnostic_code: DiagnosticCode::STALE_FIELD_VIEW.as_str(),
        help: stale_field_view_help(item_name, view_name, site, &statements[index], state),
    }
}

fn linear_double_consume_diagnostic(
    item_name: &str,
    statements: &[BodyStatement],
    index: usize,
    target: &str,
    move_site: &MoveSite,
    state: &PathState,
) -> PathDiagnostic {
    PathDiagnostic {
        index,
        ownership_kind: "linear_resource_double_consume",
        target: Some(target.to_string()),
        status: "rejected_linear_resource_consumed_twice_v0",
        reason: "linear_resource_consumed_more_than_once_v0",
        diagnostic_code: DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE.as_str(),
        help: linear_double_consume_help(item_name, target, move_site, &statements[index], state),
    }
}

fn linear_binding(statement: &BodyStatement) -> Option<(String, String)> {
    if !matches!(statement.kind, "let_binding" | "mutable_binding") {
        return None;
    }
    let type_text = binding_annotation(statement)?;
    if !is_linear_resource_type(&type_text) {
        return None;
    }
    let root = binding_name(statement).map(|name| first_resource(&name))?;
    Some((root, type_text))
}

fn field_view_binding(statement: &BodyStatement) -> Option<(String, String)> {
    if statement.kind != "let_binding" {
        return None;
    }
    let view_name = binding_name(statement).map(|name| first_resource(&name))?;
    let source_place = field_view_source(binding_initializer(statement)?)?;
    Some((view_name, source_place.to_string()))
}

fn field_view_source(text: &str) -> Option<&str> {
    let source = strip_keyword(text.trim(), "borrow")?;
    field_place::split_field_place(source)
        .is_some()
        .then_some(source)
}

fn element_view_binding(statement: &BodyStatement) -> Option<(String, String)> {
    if statement.kind != "let_binding" {
        return None;
    }
    let view_name = binding_name(statement).map(|name| first_resource(&name))?;
    let source_place = element_view_source(binding_initializer(statement)?)?;
    Some((view_name, source_place.to_string()))
}

fn element_view_source(text: &str) -> Option<&str> {
    let source = strip_keyword(text.trim(), "borrow")?;
    element_place::split_element_place(source)
        .is_some()
        .then_some(source)
}

fn binding_annotation(statement: &BodyStatement) -> Option<String> {
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
    let (_name, type_text) = left.split_once(':')?;
    let type_text = type_text.trim();
    if type_text.is_empty() {
        None
    } else {
        Some(type_text.to_string())
    }
}

fn is_linear_resource_type(type_text: &str) -> bool {
    type_tokens(type_text)
        .into_iter()
        .any(|token| token.eq_ignore_ascii_case("Transaction") || token.ends_with("Transaction"))
}

fn type_tokens(type_text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in type_text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn consume_action(statement: &BodyStatement) -> String {
    expression_text_for_statement(statement)
        .and_then(call_callee_name)
        .unwrap_or_else(|| "consume_argument".to_string())
}

fn call_callee_name(text: &str) -> Option<String> {
    let (callee, _args) = text.trim().split_once('(')?;
    let callee = callee.trim();
    if callee.is_empty()
        || !callee
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
    {
        None
    } else {
        Some(first_resource(callee))
    }
}

fn matching_statement_close(statements: &[BodyStatement], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, statement) in statements.iter().enumerate().skip(open) {
        if statement.text.ends_with('{') {
            depth = depth.saturating_add(1);
        }
        if statement.kind == "block_close" {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn last_non_close_statement(statements: &[BodyStatement]) -> Option<usize> {
    statements
        .iter()
        .enumerate()
        .rev()
        .find(|(_index, statement)| statement.kind != "block_close")
        .map(|(index, _statement)| index)
}

fn format_path(path: &[String]) -> String {
    if path.is_empty() {
        "straight-line path".to_string()
    } else {
        path.join(" -> ")
    }
}

fn stale_field_view_help(
    item_name: &str,
    view_name: &str,
    site: &FieldViewSite,
    statement: &BodyStatement,
    state: &PathState,
) -> String {
    let Some(invalidation) = &site.invalidated_by else {
        return format!(
            "Fix task {item_name} on {}: view {view_name} has no recorded invalidation; re-borrow after changes or bind a value copy before the change.",
            format_path(&state.path)
        );
    };
    match (site.kind, invalidation.kind) {
        (ViewKind::Field, ViewInvalidationKind::FieldWrite) => format!(
            "Fix task {item_name} on {}: {view_name} borrowed {} at {}:{}:{}, but {} was written at {}:{}:{} before the use at {}:{}:{}; re-borrow after the write or copy the field value before the write.",
            format_path(&state.path),
            site.source_place,
            site.bound_span.file,
            site.bound_span.line,
            site.bound_span.column,
            site.source_place,
            invalidation.span.file,
            invalidation.span.line,
            invalidation.span.column,
            statement.span.file,
            statement.span.line,
            statement.span.column
        ),
        (ViewKind::Element, ViewInvalidationKind::ListAppend) => {
            let root = first_resource(&site.source_place);
            format!(
                "Fix task {item_name} on {}: {view_name} borrowed {} at {}:{}:{}, but list_append grew {root} at {}:{}:{} before the use at {}:{}:{}; re-borrow after the append or copy the element value before the append.",
                format_path(&state.path),
                site.source_place,
                site.bound_span.file,
                site.bound_span.line,
                site.bound_span.column,
                invalidation.span.file,
                invalidation.span.line,
                invalidation.span.column,
                statement.span.file,
                statement.span.line,
                statement.span.column
            )
        }
        _ => format!(
            "Fix task {item_name} on {}: {view_name} borrowed {} at {}:{}:{}, but the source changed at {}:{}:{} before the use at {}:{}:{}; re-borrow after the change or copy the value first.",
            format_path(&state.path),
            site.source_place,
            site.bound_span.file,
            site.bound_span.line,
            site.bound_span.column,
            invalidation.span.file,
            invalidation.span.line,
            invalidation.span.column,
            statement.span.file,
            statement.span.line,
            statement.span.column
        ),
    }
}
fn path_move_help(
    item_name: &str,
    target: &str,
    move_site: &MoveSite,
    state: &PathState,
) -> String {
    format!(
        "Fix task {item_name} on {}: {target} was moved by {} at {}:{}:{}; use it before that move or create a fresh owned value.",
        format_path(&state.path),
        move_site.kind,
        move_site.span.file,
        move_site.span.line,
        move_site.span.column
    )
}

fn linear_missing_help(
    item_name: &str,
    target: &str,
    site: &LinearResourceSite,
    exit_statement: &BodyStatement,
    exit_kind: &str,
    state: &PathState,
) -> String {
    format!(
        "Fix task {item_name} on {}: linear resource {target} declared as {} at {}:{}:{} reaches {exit_kind} at {}:{}:{} without commit, rollback, close, or transfer; consume it exactly once before leaving this path.",
        format_path(&state.path),
        site.type_text,
        site.span.file,
        site.span.line,
        site.span.column,
        exit_statement.span.file,
        exit_statement.span.line,
        exit_statement.span.column
    )
}

fn linear_double_consume_help(
    item_name: &str,
    target: &str,
    move_site: &MoveSite,
    statement: &BodyStatement,
    state: &PathState,
) -> String {
    format!(
        "Fix task {item_name} on {}: linear resource {target} was already consumed by {} at {}:{}:{} before the second consume at {}:{}:{}; keep exactly one commit, rollback, close, or transfer on each path.",
        format_path(&state.path),
        move_site.kind,
        move_site.span.file,
        move_site.span.line,
        move_site.span.column,
        statement.span.file,
        statement.span.line,
        statement.span.column
    )
}

fn boundary_checks(
    item: &Item,
    declarations: &OwnershipDeclarations,
    ownership_facts: &LocalOwnershipFacts,
    body_statements: &[BodyStatement],
    ownership_statements: &[OwnershipStatement],
) -> Vec<OwnershipBoundaryCheck> {
    let mut checks = Vec::new();
    let item_span = portable_span(item.span());
    for duplicate in &ownership_facts.duplicate_locals {
        checks.push(boundary_check(
            "local_place_names_unique",
            &item_span,
            "rejected_duplicate_local_place_v0",
            Some("duplicate_local_place_would_hide_ownership_identity_v0"),
        ));
        let _ = duplicate;
    }

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

    if has_security_sensitive_ownership(declarations, body_statements) {
        checks.push(boundary_check(
            "security_ownership_requires_protects",
            &item_span,
            if declarations.protects.is_empty() {
                "rejected_security_ownership_without_protects_v0"
            } else {
                "accepted_security_ownership_has_protects_v0"
            },
            if declarations.protects.is_empty() {
                Some("security_sensitive_ownership_requires_protects_section")
            } else {
                None
            },
        ));
    }

    for avoid in &declarations.avoids {
        if let Some(reason) = avoid_contradiction(&avoid.text, ownership_statements) {
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

fn return_dependency_facts(
    item: &Item,
    ownership_facts: &LocalOwnershipFacts,
    body_statements: &[BodyStatement],
) -> Vec<OwnershipReturnDependency> {
    let Some((item_name, result, item_span)) = item_return_annotation(item) else {
        return Vec::new();
    };
    let Some(dependency) = return_dependency::parse_return_dependency(result) else {
        return Vec::new();
    };

    vec![return_dependency_fact(
        item_name,
        &dependency,
        item_span,
        ownership_facts,
        body_statements,
    )]
}

fn return_dependency_fact(
    item_name: &str,
    dependency: &ReturnDependency,
    item_span: &Span,
    ownership_facts: &LocalOwnershipFacts,
    body_statements: &[BodyStatement],
) -> OwnershipReturnDependency {
    let returned_roots = returned_roots(body_statements);
    let (source_kind, status, reason, span, diagnostic_code, help) =
        if !return_dependency::is_bare_source_name(&dependency.source) {
            (
                "internal_reference",
                "rejected_return_dependency_internal_reference_v0",
                Some("internal_reference_return_dependency_not_implemented_v0"),
                portable_span(item_span),
                Some(DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER.as_str()),
                Some(return_dependency_help(item_name, &dependency.source)),
            )
        } else if ownership_facts.parameters.contains_key(&dependency.source) {
            return_dependency_parameter_status(
                item_name,
                dependency,
                item_span,
                ownership_facts,
                &returned_roots,
            )
        } else if ownership_facts.is_local_root(&dependency.source) {
            (
                "local",
                "rejected_return_dependency_local_v0",
                Some("returned_view_cannot_depend_on_local_place_v0"),
                portable_span(item_span),
                Some(DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER.as_str()),
                Some(return_dependency_help(item_name, &dependency.source)),
            )
        } else {
            (
                "unknown",
                "rejected_return_dependency_unknown_source_v0",
                Some("return_dependency_source_must_name_parameter_v0"),
                portable_span(item_span),
                Some(DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER.as_str()),
                Some(return_dependency_help(item_name, &dependency.source)),
            )
        };

    OwnershipReturnDependency {
        id: prefixed_id(
            "hum_ownership_return_dependency",
            &format!("{item_name}_{}_{}", dependency.source, span.line),
        ),
        span,
        result_type: dependency.result_type.clone(),
        source: dependency.source.clone(),
        source_kind,
        status,
        reason,
        diagnostic_code,
        help,
    }
}

fn return_dependency_parameter_status(
    item_name: &str,
    dependency: &ReturnDependency,
    item_span: &Span,
    ownership_facts: &LocalOwnershipFacts,
    returned_roots: &[ReturnedRoot],
) -> (
    &'static str,
    &'static str,
    Option<&'static str>,
    Span,
    Option<&'static str>,
    Option<String>,
) {
    let mut accepted_status = "accepted_return_dependency_parameter_v0";
    for returned in returned_roots {
        match returned.root.as_deref() {
            Some(root) if root == dependency.source => {
                if returned.derivation == "closed_view_derivation" {
                    accepted_status = "accepted_return_dependency_closed_view_derivation_v0";
                }
            }
            Some(root) if ownership_facts.is_local_root(root) => {
                return (
                    "parameter",
                    "rejected_return_dependency_returned_local_v0",
                    Some("returned_view_expression_uses_local_place_v0"),
                    portable_span(&returned.span),
                    Some(DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER.as_str()),
                    Some(return_dependency_help(item_name, &dependency.source)),
                );
            }
            Some(_) => {
                return (
                    "parameter",
                    "rejected_return_dependency_source_mismatch_v0",
                    Some("returned_view_expression_does_not_match_from_source_v0"),
                    portable_span(&returned.span),
                    Some(DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER.as_str()),
                    Some(return_dependency_help(item_name, &dependency.source)),
                );
            }
            None => {
                return (
                    "parameter",
                    "rejected_return_dependency_complex_expression_v0",
                    Some(returned.reason),
                    portable_span(&returned.span),
                    Some(DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER.as_str()),
                    Some(return_dependency_help(item_name, &dependency.source)),
                );
            }
        }
    }

    (
        "parameter",
        accepted_status,
        None,
        portable_span(item_span),
        None,
        None,
    )
}

struct ReturnedRoot {
    root: Option<String>,
    span: Span,
    derivation: &'static str,
    reason: &'static str,
}

fn returned_roots(body_statements: &[BodyStatement]) -> Vec<ReturnedRoot> {
    body_statements
        .iter()
        .filter(|statement| statement.kind == "return")
        .map(|statement| {
            let expression = expression_text_for_statement(statement).unwrap_or("");
            if let Some(root) = bare_place_root(expression) {
                return ReturnedRoot {
                    root: Some(root),
                    span: statement.span.clone(),
                    derivation: "bare_place",
                    reason: "returned_view_expression_not_bare_place_v0",
                };
            }
            if let Some(root) = return_dependency::closed_view_derivation_source(expression) {
                return ReturnedRoot {
                    root: Some(root),
                    span: statement.span.clone(),
                    derivation: "closed_view_derivation",
                    reason: "returned_view_expression_not_closed_view_derivation_v0",
                };
            }
            ReturnedRoot {
                root: None,
                span: statement.span.clone(),
                derivation: "unknown",
                reason: if return_dependency::is_closed_view_derivation_expression(expression) {
                    "returned_view_expression_has_no_visible_source_root_v0"
                } else {
                    "returned_view_expression_not_closed_view_derivation_v0"
                },
            }
        })
        .collect()
}

fn item_return_annotation(item: &Item) -> Option<(&str, &str, &Span)> {
    match item {
        Item::Task(task) => task
            .result
            .as_deref()
            .map(|result| (task.name.as_str(), result, &task.span)),
        _ => None,
    }
}

fn return_dependency_help(item_name: &str, source: &str) -> String {
    format!(
        "Fix task `{item_name}`: returned-view `from` source `{source}` must name a task parameter, and returns must visibly return that parameter or a closed-set view derivation such as `slice_until(source, separator)`; locals, internal references, and non-closed derivation chains remain rejected."
    )
}

fn boundary_check(
    check: &'static str,
    span: &Span,
    status: &'static str,
    reason: Option<&'static str>,
) -> OwnershipBoundaryCheck {
    OwnershipBoundaryCheck {
        id: prefixed_id(
            "hum_ownership_boundary",
            &format!("{}_{}_{}", check, span.line, span.column),
        ),
        span: portable_span(span),
        check,
        status,
        reason,
    }
}

fn collect_declarations(sections: &[Section]) -> OwnershipDeclarations {
    OwnershipDeclarations {
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

impl LocalOwnershipFacts {
    fn is_local_root(&self, root: &str) -> bool {
        self.immutable_locals.contains(root)
            || self.mutable_locals.contains(root)
            || self.writable_aliases.contains_key(root)
    }

    fn is_movable_root(&self, root: &str) -> bool {
        self.immutable_locals.contains(root)
            || self.mutable_locals.contains(root)
            || self
                .parameters
                .get(root)
                .is_some_and(|permission| *permission == ParamPermission::Consume)
    }
}

enum WritableAliasAuthority {
    Accepted(String),
    Borrow,
    Immutable,
    Unknown,
}

fn writable_alias_authority(
    facts: &LocalOwnershipFacts,
    binding: &AliasBinding,
) -> WritableAliasAuthority {
    if let Some(permission) = facts.parameters.get(&binding.owner_root) {
        return match permission {
            ParamPermission::Borrow => WritableAliasAuthority::Borrow,
            ParamPermission::Change | ParamPermission::Consume => {
                WritableAliasAuthority::Accepted(permission.as_str().to_string())
            }
        };
    }
    if facts.mutable_locals.contains(&binding.owner_root) {
        return WritableAliasAuthority::Accepted("change".to_string());
    }
    if facts.immutable_locals.contains(&binding.owner_root) {
        return WritableAliasAuthority::Immutable;
    }
    WritableAliasAuthority::Unknown
}

fn local_ownership_facts(
    item: &Item,
    statements: &[BodyStatement],
    alias_analysis: &AliasAnalysis,
) -> LocalOwnershipFacts {
    let mut facts = LocalOwnershipFacts::default();
    for parameter in item_parameters(item) {
        facts
            .parameters
            .insert(first_resource(&parameter.name), parameter.permission);
    }

    let mut seen = BTreeSet::new();
    for statement in statements {
        if !matches!(statement.kind, "let_binding" | "mutable_binding") {
            continue;
        }
        let Some(name) = binding_name(statement) else {
            continue;
        };
        let name = first_resource(&name);
        if !seen.insert(name.clone()) {
            facts.duplicate_locals.insert(name.clone());
        }
        if let Some(binding) = alias_analysis
            .bindings
            .iter()
            .find(|binding| binding.name == name)
        {
            facts.writable_aliases.insert(name, binding.clone());
        } else if statement.kind == "mutable_binding" {
            facts.mutable_locals.insert(name);
        } else {
            facts.immutable_locals.insert(name);
        }
    }

    facts
}

fn item_parameters(item: &Item) -> Vec<&crate::ast::Param> {
    match item {
        Item::Task(task) => task.params.iter().collect(),
        Item::Test(test) => test.params.iter().collect(),
        _ => Vec::new(),
    }
}

fn moved_value_use<'a>(
    statement: &BodyStatement,
    move_tracker: &'a MoveTracker,
) -> Option<(String, &'a MoveSite)> {
    for root in statement_roots(statement) {
        if let Some(move_site) = move_tracker.moved.get(&root) {
            return Some((root, move_site));
        }
    }
    None
}

fn stale_field_view_use<'a>(
    statement: &BodyStatement,
    state: &'a PathState,
) -> Option<(String, &'a FieldViewSite)> {
    let expression = expression_text_for_statement(statement)?;
    for token in identifier_tokens(expression) {
        let root = first_resource(&token);
        if let Some(site) = state.field_views.get(&root)
            && site.invalidated_by.is_some()
        {
            return Some((root, site));
        }
    }
    None
}

fn invalidate_path_field_views(state: &mut PathState, target: &str, span: &Span) {
    for site in state.field_views.values_mut() {
        if site.kind == ViewKind::Field
            && site.source_place == target
            && site.invalidated_by.is_none()
        {
            site.invalidated_by = Some(ViewInvalidation {
                span: portable_span(span),
                kind: ViewInvalidationKind::FieldWrite,
            });
        }
    }
}

fn invalidate_path_element_views(state: &mut PathState, root: &str, span: &Span) {
    for site in state.field_views.values_mut() {
        if site.kind == ViewKind::Element
            && first_resource(&site.source_place) == root
            && site.invalidated_by.is_none()
        {
            site.invalidated_by = Some(ViewInvalidation {
                span: portable_span(span),
                kind: ViewInvalidationKind::ListAppend,
            });
        }
    }
}

fn list_append_change_root(statement: &BodyStatement) -> Option<String> {
    let expression = expression_text_for_statement(statement)?.trim();
    let args = expression.strip_prefix("list_append(")?.strip_suffix(')')?;
    let (first, _second) = args.split_once(',')?;
    let list_place = strip_keyword(first.trim(), "change")?;
    Some(first_resource(list_place))
}
fn record_statement_moves(
    statement: &BodyStatement,
    ownership_facts: &LocalOwnershipFacts,
    move_tracker: &mut MoveTracker,
) {
    for root in consume_move_roots(statement) {
        if ownership_facts.is_movable_root(&root) {
            move_tracker.moved.entry(root).or_insert_with(|| MoveSite {
                span: portable_span(&statement.span),
                kind: "consume_argument".to_string(),
            });
        }
    }

    if let Some(root) = returned_move_root(statement, ownership_facts) {
        move_tracker.moved.entry(root).or_insert_with(|| MoveSite {
            span: portable_span(&statement.span),
            kind: "return".to_string(),
        });
    }
}

fn statement_roots(statement: &BodyStatement) -> Vec<String> {
    let mut roots = Vec::new();
    if let Some(expression) = expression_text_for_statement(statement) {
        roots.extend(identifier_roots(expression));
    }
    if statement.kind == "set_place"
        && let Some(target) = set_place_name(statement)
    {
        roots.push(first_resource(&target));
    }
    roots
}

fn consume_move_roots(statement: &BodyStatement) -> Vec<String> {
    expression_text_for_statement(statement)
        .map(consume_roots)
        .unwrap_or_default()
}

fn first_consume_move_root(expression: &str) -> Option<String> {
    consume_roots(expression).into_iter().next()
}

fn consume_roots(expression: &str) -> Vec<String> {
    let tokens = identifier_tokens(expression);
    let mut roots = Vec::new();
    for index in 0..tokens.len() {
        if tokens[index] == "consume"
            && let Some(next) = tokens.get(index + 1)
        {
            roots.push(first_resource(next));
        }
    }
    roots
}

fn returned_move_root(
    statement: &BodyStatement,
    ownership_facts: &LocalOwnershipFacts,
) -> Option<String> {
    if statement.kind != "return" {
        return None;
    }
    let expression = expression_text_for_statement(statement)?;
    let root = bare_place_root(expression)?;
    ownership_facts.is_movable_root(&root).then_some(root)
}

fn bare_place_root(expression: &str) -> Option<String> {
    let expression = expression.trim();
    if expression.is_empty()
        || expression.contains('(')
        || expression.contains(' ')
        || expression.starts_with('"')
    {
        return None;
    }
    let root = first_resource(expression);
    if root
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase() || ch == '_')
    {
        Some(root)
    } else {
        None
    }
}

fn identifier_roots(expression: &str) -> Vec<String> {
    identifier_tokens(expression)
        .into_iter()
        .filter(|token| {
            !matches!(
                token.as_str(),
                "borrow" | "change" | "consume" | "true" | "false"
            )
        })
        .filter(|token| {
            token
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_lowercase() || ch == '_')
        })
        .map(|token| first_resource(&token))
        .collect()
}

fn identifier_tokens(expression: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    for ch in expression.chars() {
        if ch == '"' {
            in_string = !in_string;
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            continue;
        }
        if in_string {
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn move_help(item_name: &str, target: &str, move_site: &MoveSite) -> String {
    format!(
        "Fix task `{item_name}`: `{target}` was moved by {} at {}:{}:{}; use it before that move or create a fresh owned value.",
        move_site.kind, move_site.span.file, move_site.span.line, move_site.span.column
    )
}

fn borrow_mutation_help(
    item_name: &str,
    target: &str,
    root: &str,
    statement: &BodyStatement,
) -> String {
    format!(
        "Fix task `{item_name}`: `{target}` writes through borrowed parameter `{root}` at {}:{}:{}; mark the parameter `change`, copy into a `change` local, or remove the `set`.",
        statement.span.file, statement.span.line, statement.span.column
    )
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

fn has_security_sensitive_ownership(
    declarations: &OwnershipDeclarations,
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
    ownership_statements: &[OwnershipStatement],
) -> Option<&'static str> {
    let lowered = avoid_text.to_ascii_lowercase();
    for statement in ownership_statements {
        if statement.ownership_kind == "no_ownership_transfer" && lowered.contains("fail") {
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
    let token = text
        .split_whitespace()
        .next()
        .unwrap_or(text)
        .trim_matches(|ch: char| ch == ',' || ch == '.');
    token
        .split(['.', '['])
        .next()
        .unwrap_or(token)
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
    statements: &[OwnershipStatement],
    return_dependencies: &[OwnershipReturnDependency],
    boundary_checks: &[OwnershipBoundaryCheck],
    blocked: bool,
) -> &'static str {
    if blocked {
        "blocked_by_prior_errors"
    } else if statements.iter().any(is_rejected_statement)
        || return_dependencies
            .iter()
            .any(is_rejected_return_dependency)
        || boundary_checks.iter().any(is_rejected_boundary_check)
    {
        "ownership_errors_v0"
    } else if statements.iter().any(is_unchecked_statement) {
        "blocked_by_unchecked_ownership_facts_v0"
    } else {
        "recognized_core_ownership_facts_checked_v0"
    }
}

fn is_rejected_statement(statement: &OwnershipStatement) -> bool {
    statement.status.starts_with("rejected_")
}

fn is_unchecked_statement(statement: &OwnershipStatement) -> bool {
    statement.status.starts_with("unchecked_") || statement.status.starts_with("not_checked_")
}

fn is_rejected_return_dependency(dependency: &OwnershipReturnDependency) -> bool {
    dependency.status.starts_with("rejected_")
}

fn is_rejected_boundary_check(check: &OwnershipBoundaryCheck) -> bool {
    check.status.starts_with("rejected_")
}

impl OwnershipCheckReport {
    fn status(&self) -> &'static str {
        if self.source_errors > 0 {
            "blocked_by_source_errors"
        } else if self.effect_check_summary.resolver_errors > 0 {
            "blocked_by_resolver_errors"
        } else if self.effect_check_summary.type_errors > 0 {
            "blocked_by_type_errors"
        } else if self.effect_check_summary.core_verify_errors > 0 {
            "blocked_by_core_verify_errors"
        } else if self.effect_check_summary.full_type_check_errors > 0 {
            "blocked_by_full_type_check_errors"
        } else if self.effect_check_errors() > 0 {
            "blocked_by_effect_check_errors"
        } else if self.rejected_statements() > 0
            || self.rejected_return_dependencies() > 0
            || self.rejected_boundary_checks() > 0
        {
            "ownership_errors_v0"
        } else if self.unchecked_statements() > 0 {
            "blocked_by_unchecked_ownership_facts_v0"
        } else {
            "recognized_core_ownership_facts_checked_v0"
        }
    }

    fn files(&self) -> usize {
        self.files
    }

    fn item_count(&self) -> usize {
        self.item_count
    }

    fn effect_check_errors(&self) -> usize {
        self.effect_check_summary
            .blocking_issues
            .saturating_sub(self.source_errors)
            .saturating_sub(self.effect_check_summary.resolver_errors)
            .saturating_sub(self.effect_check_summary.type_errors)
            .saturating_sub(self.effect_check_summary.core_verify_errors)
            .saturating_sub(self.effect_check_summary.full_type_check_errors)
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

    fn rejected_return_dependencies(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.return_dependencies.iter())
            .filter(|dependency| is_rejected_return_dependency(dependency))
            .count()
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
            .filter(|statement| statement.ownership_kind == "ambient_read")
            .count()
    }

    fn inferred_changes(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                matches!(
                    statement.ownership_kind,
                    "external_change_deferred"
                        | "store_change"
                        | "local_mutation"
                        | "parameter_mutation"
                        | "field_mutation"
                        | "writable_field_alias_write_through"
                )
            })
            .count()
    }

    fn inferred_failures(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| statement.ownership_kind == "no_ownership_transfer")
            .count()
    }

    fn blocking_issues(&self) -> usize {
        self.source_errors
            + self.effect_check_summary.resolver_errors
            + self.effect_check_summary.type_errors
            + self.effect_check_summary.core_verify_errors
            + self.effect_check_summary.full_type_check_errors
            + self.effect_check_errors()
            + self.rejected_statements()
            + self.unchecked_statements()
            + self.rejected_return_dependencies()
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
    report: &OwnershipCheckReport,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "dependencies");
    out.push_str(": {\n");
    push_indent(out, indent + 2);
    push_json_string(out, "effect_check");
    out.push_str(": {\n");
    push_string_field(
        out,
        indent + 4,
        "schema",
        report.effect_check_summary.schema,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "status",
        report.effect_check_summary.status,
        true,
    );
    push_usize_field(
        out,
        indent + 4,
        "blocking_issues",
        report.effect_check_summary.blocking_issues,
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("}\n");
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_summary(out: &mut String, report: &OwnershipCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "summary");
    out.push_str(": {\n");
    push_usize_field(out, indent + 2, "files", report.files(), true);
    push_usize_field(out, indent + 2, "items", report.item_count(), true);
    push_usize_field(out, indent + 2, "ownership_items", report.items.len(), true);
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
        report.effect_check_summary.resolver_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_errors",
        report.effect_check_summary.type_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "core_verify_errors",
        report.effect_check_summary.core_verify_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "full_type_check_errors",
        report.effect_check_summary.full_type_check_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "effect_check_errors",
        report.effect_check_errors(),
        true,
    );
    push_usize_field(out, indent + 2, "execution_ready", 0, true);
    push_usize_field(out, indent + 2, "ir_ready", 0, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_items(out: &mut String, items: &[OwnershipItem], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "ownership_items");
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

fn push_item(out: &mut String, item: &OwnershipItem, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &item.id, true);
    push_string_field(out, indent + 2, "kind", item.kind, true);
    push_string_field(out, indent + 2, "name", &item.name, true);
    push_span_field(out, indent + 2, "source_span", &item.span, true);
    push_string_field(out, indent + 2, "status", item.status, true);
    push_declarations(out, &item.declarations, indent + 2, true);
    push_statements(out, &item.statements, indent + 2, true);
    push_return_dependencies(out, &item.return_dependencies, indent + 2, true);
    push_boundary_checks(out, &item.boundary_checks, indent + 2, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_declarations(
    out: &mut String,
    declarations: &OwnershipDeclarations,
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

fn push_statements(
    out: &mut String,
    statements: &[OwnershipStatement],
    indent: usize,
    comma: bool,
) {
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

fn push_statement(out: &mut String, statement: &OwnershipStatement, indent: usize) {
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
    push_string_field(
        out,
        indent + 2,
        "ownership_kind",
        statement.ownership_kind,
        true,
    );
    push_optional_string_field(out, indent + 2, "target", statement.target.as_deref(), true);
    push_optional_string_field(
        out,
        indent + 2,
        "declaration",
        statement.declaration.as_deref(),
        true,
    );
    push_string_field(out, indent + 2, "status", statement.status, true);
    push_optional_string_field(out, indent + 2, "reason", statement.reason, true);
    push_optional_string_field(
        out,
        indent + 2,
        "diagnostic_code",
        statement.diagnostic_code,
        true,
    );
    push_optional_string_field(out, indent + 2, "alias", statement.alias.as_deref(), true);
    push_optional_string_field(out, indent + 2, "place", statement.place.as_deref(), true);
    push_optional_span_field(
        out,
        indent + 2,
        "binding_span",
        statement.binding_span.as_ref(),
        true,
    );
    push_optional_span_field(
        out,
        indent + 2,
        "last_use_span",
        statement.last_use_span.as_ref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "conflict_place",
        statement.conflict_place.as_deref(),
        true,
    );
    push_optional_span_field(
        out,
        indent + 2,
        "conflict_span",
        statement.conflict_span.as_ref(),
        true,
    );
    push_optional_string_field(out, indent + 2, "help", statement.help.as_deref(), false);
    push_indent(out, indent);
    out.push('}');
}

fn push_return_dependencies(
    out: &mut String,
    dependencies: &[OwnershipReturnDependency],
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "return_dependencies");
    out.push_str(": [");
    if !dependencies.is_empty() {
        out.push('\n');
        for (index, dependency) in dependencies.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_return_dependency(out, dependency, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_return_dependency(out: &mut String, dependency: &OwnershipReturnDependency, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &dependency.id, true);
    push_span_field(out, indent + 2, "source_span", &dependency.span, true);
    push_string_field(
        out,
        indent + 2,
        "result_type",
        &dependency.result_type,
        true,
    );
    push_string_field(out, indent + 2, "source", &dependency.source, true);
    push_string_field(out, indent + 2, "source_kind", dependency.source_kind, true);
    push_string_field(out, indent + 2, "status", dependency.status, true);
    push_optional_string_field(out, indent + 2, "reason", dependency.reason, true);
    push_optional_string_field(
        out,
        indent + 2,
        "diagnostic_code",
        dependency.diagnostic_code,
        true,
    );
    push_optional_string_field(out, indent + 2, "help", dependency.help.as_deref(), false);
    push_indent(out, indent);
    out.push('}');
}

fn push_boundary_checks(
    out: &mut String,
    checks: &[OwnershipBoundaryCheck],
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

fn push_boundary_check(out: &mut String, check: &OwnershipBoundaryCheck, indent: usize) {
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

fn push_optional_span_field(
    out: &mut String,
    indent: usize,
    key: &str,
    span: Option<&Span>,
    comma: bool,
) {
    match span {
        Some(span) => push_span_field(out, indent, key, span, comma),
        None => {
            push_indent(out, indent);
            push_json_string(out, key);
            out.push_str(": null");
            push_comma_newline(out, comma);
        }
    }
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

    use super::{
        ownership_check_has_errors, ownership_check_json, ownership_check_summary,
        ownership_check_text,
    };

    #[test]
    fn json_accepts_local_ownership_and_mutation_facts() {
        let program = ownership_demo_program();
        let json = ownership_check_json(&program, &[]);

        assert!(!ownership_check_has_errors(&program, &[]));
        assert!(json.contains("\"schema\": \"hum.ownership_check.v0\""));
        assert!(json.contains("\"status\": \"recognized_core_ownership_facts_checked_v0\""));
        assert!(json.contains("\"accepted_mutable_local_owner_v0\""));
        assert!(json.contains("\"accepted_exclusive_local_mutation_v0\""));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"ir_ready\": 0"));
        assert!(json.contains("\"no complete ownership system\""));
    }

    #[test]
    fn json_blocks_when_effect_check_has_errors() {
        let program = missing_failure_program();
        let json = ownership_check_json(&program, &[]);

        assert!(ownership_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"blocked_by_effect_check_errors\""));
        assert!(json.contains("\"effect_check_errors\":"));
    }

    #[test]
    fn summary_counts_full_type_check_prior_blockers() {
        let program = unchecked_full_type_program();
        let summary = ownership_check_summary(&program, &[]);

        assert_eq!(summary.status, "blocked_by_full_type_check_errors");
        assert!(summary.full_type_check_errors > 0);
        assert!(summary.blocking_issues >= summary.full_type_check_errors);
    }

    #[test]
    fn json_rejects_duplicate_local_place() {
        let program = duplicate_local_program();
        let json = ownership_check_json(&program, &[]);

        assert!(ownership_check_has_errors(&program, &[]));
        assert!(json.contains("\"rejected_duplicate_local_place_v0\""));
    }

    #[test]
    fn text_reports_ownership_gate_without_safety_claims() {
        let program = ownership_demo_program();
        let text = ownership_check_text(&program, &[]);

        assert!(text.contains("Hum ownership check (hum.ownership_check.v0)"));
        assert!(text.contains("status: recognized_core_ownership_facts_checked_v0"));
        assert!(text.contains("no memory-safety proof"));
    }

    #[test]
    fn json_exposes_writable_alias_place_and_last_use_facts() {
        let program = parse_program(
            "examples/probes/writable_field_aliases.hum",
            include_str!("../examples/probes/writable_field_aliases.hum"),
        );
        let json = ownership_check_json(&program, &[]);

        assert!(!ownership_check_has_errors(&program, &[]), "{json}");
        assert!(json.contains("\"accepted_writable_field_alias_v0\""));
        assert!(json.contains("\"accepted_writable_field_alias_write_through_v0\""));
        assert!(json.contains("\"alias\": \"alias_to_x\""));
        assert!(json.contains("\"place\": \"point.x\""));
        assert!(json.contains("\"binding_span\": {"));
        assert!(json.contains("\"last_use_span\": {"));
        assert!(json.contains("\"accepted_disjoint_field_mutation_v0\""));
    }

    #[test]
    fn human_and_json_reject_pinned_overlap_with_structured_h0808_blame() {
        let program = parse_program(
            "fixtures/ownership_check/session_v_program8_overlap_write_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_program8_overlap_write_fail.hum"),
        );
        let text = ownership_check_text(&program, &[]);
        let json = ownership_check_json(&program, &[]);

        assert!(ownership_check_has_errors(&program, &[]));
        for expected in [
            "H0808",
            "alias_to_x",
            "point.x",
            "not known independent",
            "definitely distinct direct field",
        ] {
            assert!(text.contains(expected), "missing {expected}: {text}");
            assert!(json.contains(expected), "missing {expected}: {json}");
        }
        assert!(json.contains("\"conflict_place\": \"point.x\""));
        assert!(json.contains("\"conflict_span\": {"));
        assert!(json.contains("\"line\": 13"));
        assert!(json.contains("\"line\": 14"));
        assert!(json.contains("\"line\": 15"));
    }

    #[test]
    fn ownership_rejects_writable_alias_escape_with_h0809() {
        let program = parse_program(
            "fixtures/ownership_check/session_v_alias_escape_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_escape_fail.hum"),
        );
        let json = ownership_check_json(&program, &[]);
        assert!(json.contains("\"diagnostic_code\": \"H0809\""));
        assert!(json.contains("\"rejected_unsupported_writable_field_alias_v0\""));
        assert!(json.contains("writable_alias_escape_v0"));
    }

    #[test]
    fn ownership_keeps_borrowed_alias_authority_ahead_of_overlap() {
        let program = parse_program(
            "fixtures/ownership_check/session_v_borrowed_owner_overlap_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_borrowed_owner_overlap_fail.hum"),
        );
        let json = ownership_check_json(&program, &[]);
        assert!(json.contains("\"diagnostic_code\": \"H0802\""));
        assert!(!json.contains("\"diagnostic_code\": \"H0808\""));
    }

    fn ownership_demo_program() -> Program {
        parse_program(
            "ownership_demo.hum",
            r#"type WorkError {
  code: Text
}

task retry(flag: Bool) -> Result UInt, WorkError {
  why:
    keep the ownership gate small
  fails when:
    flag is false

  does:
    change attempts: UInt = 0
    if flag == false {
      fail WorkError.no_flag
    }

    set attempts = attempts + 1
    return attempts
}
"#,
        )
    }

    fn missing_failure_program() -> Program {
        parse_program(
            "missing_failure.hum",
            r#"type WorkError {
  code: Text
}

task bad(flag: Bool) -> Result UInt, WorkError {
  does:
    if flag == false {
      fail WorkError.no_flag
    }
    return 0
}
"#,
        )
    }

    fn duplicate_local_program() -> Program {
        parse_program(
            "duplicate_local.hum",
            r#"task duplicate() -> UInt {
  does:
    let count: UInt = 0
    let count: UInt = 1
    return count
}
"#,
        )
    }

    fn unchecked_full_type_program() -> Program {
        parse_program(
            "unchecked_full_type.hum",
            r#"type WorkItem {
  title: Text
  done: Bool
}

type WorkError {
  code: Text
}

store work_items: list WorkItem {
  why:
    keep work_items
}

task remember(title: Text) -> Result WorkItem, WorkError {
  changes:
    work_items

  fails when:
    title is empty

  does:
    let item = WorkItem {
      title: title
      done: false
    }

    save item in work_items
    return item
}
"#,
        )
    }

    fn parse_program(path: &str, source: &str) -> Program {
        Program {
            files: vec![parse_source(path, source).file],
        }
    }
}
