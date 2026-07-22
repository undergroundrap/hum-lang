use std::collections::BTreeMap;

use crate::ast::{Item, Program, Task, TypeSyntaxKind};
use crate::callable;
use crate::core_body::{self, BodyStatement};
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticOccurrence, DiagnosticOccurrenceSet, Severity, Span,
};
use crate::predicate;
use crate::return_dependency;
use crate::type_env::{self, TypeDeclaration, TypeEnvReport};
use crate::version;

pub const TYPE_CHECK_SCHEMA: &str = "hum.type_check.v0";
pub const TYPE_CHECK_MODE: &str = "declaration_annotation_and_trivial_return_check_v0";

const NON_CLAIMS: &[&str] = &[
    "no full expression type inference",
    "no broad body statement type checking",
    "no call, overload, field, or operator type checking",
    "no generic arity validation",
    "no trait or interface checking",
    "no layout or ABI proof",
    "no borrow checking",
    "no effect checking",
    "no executable semantics",
];

#[derive(Debug, Clone)]
struct TypeCheckReport {
    type_env: TypeEnvReport,
    callable_blockers: usize,
    checked_declarations: Vec<CheckedDeclaration>,
    checked_returns: Vec<CheckedReturn>,
    diagnostics: Vec<TypeCheckDiagnostic>,
    diagnostic_occurrences: DiagnosticOccurrenceSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeCheckSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub source_errors: usize,
    pub source_warnings: usize,
    pub resolver_errors: usize,
    pub checked_declarations: usize,
    pub accepted_declarations: usize,
    pub rejected_declarations: usize,
    pub checked_type_references: usize,
    pub unknown_type_references: usize,
    pub checked_returns: usize,
    pub accepted_returns: usize,
    pub rejected_returns: usize,
    pub unchecked_returns: usize,
    pub type_errors: usize,
    pub type_warnings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedReturnSummary {
    pub id: String,
    pub owner_kind: &'static str,
    pub owner_name: String,
    pub source_span: Span,
    pub expression_text: String,
    pub expected_type: Option<String>,
    pub expected_value_type: Option<String>,
    pub actual_type: Option<String>,
    pub type_source: Option<&'static str>,
    pub status: &'static str,
    pub reason: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct CheckedDeclaration {
    id: String,
    declaration_id: String,
    declaration_kind: &'static str,
    owner_kind: &'static str,
    owner_name: String,
    name: String,
    resolver_definition_id: Option<String>,
    source_span: Span,
    type_text: String,
    type_references: Vec<CheckedTypeReference>,
    status: &'static str,
}

#[derive(Debug, Clone)]
struct CheckedTypeReference {
    text: String,
    normalized_name: String,
    role: &'static str,
    type_env_status: &'static str,
    check_status: &'static str,
}

#[derive(Debug, Clone)]
struct CheckedReturn {
    id: String,
    owner_kind: &'static str,
    owner_name: String,
    source_span: Span,
    expression_text: String,
    expected_type: Option<String>,
    expected_value_type: Option<String>,
    actual_type: Option<String>,
    type_source: Option<&'static str>,
    status: &'static str,
    reason: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct TypeFact {
    type_text: String,
    source: &'static str,
}

#[derive(Debug, Clone)]
struct TypeCheckDiagnostic {
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    code: DiagnosticCode,
    severity: Severity,
    title: &'static str,
    message: String,
    source_span: Span,
    help: &'static str,
    declaration_id: Option<String>,
    resolver_definition_id: Option<String>,
    type_name: Option<String>,
    return_id: Option<String>,
    expression_text: Option<String>,
    expected_type: Option<String>,
    actual_type: Option<String>,
}

pub fn type_check_has_errors(program: &Program, diagnostics: &[Diagnostic]) -> bool {
    let summary = type_check_summary(program, diagnostics);
    summary.source_errors > 0 || summary.resolver_errors > 0 || summary.type_errors > 0
}

pub fn type_check_summary(program: &Program, diagnostics: &[Diagnostic]) -> TypeCheckSummary {
    let report = build_report(program, diagnostics);
    TypeCheckSummary {
        schema: TYPE_CHECK_SCHEMA,
        status: report.status(),
        mode: TYPE_CHECK_MODE,
        source_errors: report.source_errors(),
        source_warnings: report.type_env.source_warnings,
        resolver_errors: report.resolver_errors(),
        checked_declarations: report.checked_declarations.len(),
        accepted_declarations: report.accepted_declarations(),
        rejected_declarations: report.rejected_declarations(),
        checked_type_references: report.checked_type_references(),
        unknown_type_references: report.type_env.unknown_type_references(),
        checked_returns: report.checked_returns.len(),
        accepted_returns: report.accepted_returns(),
        rejected_returns: report.rejected_returns(),
        unchecked_returns: report.unchecked_returns(),
        type_errors: report.type_error_count(),
        type_warnings: report.type_warning_count(),
    }
}

pub(crate) fn unknown_type_diagnostics(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> Vec<Diagnostic> {
    build_report(program, diagnostics)
        .diagnostics
        .into_iter()
        .filter(|diagnostic| {
            diagnostic.code == DiagnosticCode::UNKNOWN_TYPE_NAME
                && !callable_parameter_owns_span(program, &diagnostic.source_span)
        })
        .map(|diagnostic| public_type_diagnostic(&diagnostic))
        .collect()
}

pub(crate) fn diagnostic_occurrence_set(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> DiagnosticOccurrenceSet {
    build_report(program, diagnostics).diagnostic_occurrences
}

pub(crate) fn diagnostic_occurrence_set_from_source(
    program: &Program,
    diagnostics: &[Diagnostic],
    source_occurrences: &DiagnosticOccurrenceSet,
) -> DiagnosticOccurrenceSet {
    let report = build_report(program, diagnostics);
    let mut occurrences =
        type_env::type_env_report_from_source(program, diagnostics, source_occurrences)
            .diagnostic_occurrences;
    for diagnostic in &report.diagnostics {
        occurrences
            .insert_owned(type_diagnostic_occurrence(diagnostic))
            .expect("type diagnostic occurrences must be unique");
    }
    let suppressed_type_occurrences = type_diagnostics(&report.type_env.declarations)
        .iter()
        .map(type_diagnostic_occurrence)
        .collect::<Vec<_>>();
    let relationships =
        resolver_precedence_relationships(&occurrences, &suppressed_type_occurrences);
    let consumptions =
        consume_resolver_precedence(&occurrences, &relationships, relationships.len())
            .expect("resolver/type precedence must consume exact producer relationships");
    assert_eq!(consumptions.len(), relationships.len());
    occurrences
}

pub(crate) fn resolver_precedence_relationships(
    occurrences: &DiagnosticOccurrenceSet,
    suppressed_type_occurrences: &[DiagnosticOccurrence],
) -> Vec<crate::diagnostic::DiagnosticPrecedenceRelationship> {
    let mut relationships = Vec::new();
    for dominant in occurrences.occurrences() {
        let Some(definition_id) =
            route_value(dominant.relationship_route(), "resolver_definition=")
        else {
            continue;
        };
        for suppressed in suppressed_type_occurrences {
            if route_value(suppressed.relationship_route(), "resolver_definition=")
                != Some(definition_id)
                || crate::diagnostic_catalog::exact_precedence_spec(
                    dominant.cause_key(),
                    suppressed.cause_key(),
                    "resolver_blocks_type_relationship_v0",
                    "type_check",
                    "type_check",
                )
                .is_none()
            {
                continue;
            }
            let relationship_id = format!(
                "resolver-type-precedence:{definition_id}:{}:{}",
                dominant.cause_key().ordinal(),
                suppressed.cause_key().ordinal()
            );
            relationships.push(
                crate::diagnostic::DiagnosticPrecedenceRelationship::producer_owned(
                    "resolver_over_type_v0",
                    "type_check",
                    "type_check",
                    relationship_id.clone(),
                    dominant,
                    suppressed,
                    [
                        dominant.semantic_origin().to_string(),
                        suppressed.semantic_origin().to_string(),
                    ],
                    vec![
                        relationship_id,
                        format!("resolver_definition={definition_id}"),
                    ],
                )
                .expect("type checker must seal exact resolver precedence"),
            );
        }
    }
    relationships
}

fn route_value<'a>(route: &'a [String], prefix: &str) -> Option<&'a str> {
    route.iter().find_map(|entry| entry.strip_prefix(prefix))
}

pub(crate) fn consume_resolver_precedence(
    occurrences: &DiagnosticOccurrenceSet,
    relationships: &[crate::diagnostic::DiagnosticPrecedenceRelationship],
    expected_count: usize,
) -> Result<
    Vec<crate::diagnostic::DiagnosticPrecedenceConsumption>,
    crate::diagnostic::DiagnosticInvariantError,
> {
    occurrences.consume_precedence_relationships("type_check", relationships, expected_count)
}

pub(crate) fn unknown_type_diagnostics_for_tasks(
    program: &Program,
    diagnostics: &[Diagnostic],
    tasks: &[&Task],
) -> Vec<Diagnostic> {
    unknown_type_diagnostics(program, diagnostics)
        .into_iter()
        .filter(|diagnostic| {
            diagnostic.span.as_ref().is_some_and(|span| {
                tasks.iter().any(|task| {
                    task.params.iter().any(|param| same_span(&param.span, span))
                        || task
                            .result_syntax
                            .as_ref()
                            .is_some_and(|result| same_span(&result.span, span))
                })
            })
        })
        .collect()
}

fn same_span(left: &Span, right: &Span) -> bool {
    left.file.replace('\\', "/") == right.file.replace('\\', "/")
        && left.line == right.line
        && left.column == right.column
}

fn callable_parameter_owns_span(program: &Program, span: &Span) -> bool {
    fn visit(items: &[Item], span: &Span) -> bool {
        items.iter().any(|item| match item {
            Item::App(app) => visit(&app.items, span),
            Item::Task(task) => task.params.iter().any(|param| {
                matches!(
                    param.type_syntax.kind,
                    TypeSyntaxKind::Callable(_) | TypeSyntaxKind::CallableCandidate { .. }
                ) && param.span.file == span.file
                    && param.span.line == span.line
                    && param.span.column == span.column
            }),
            _ => false,
        })
    }
    program.files.iter().any(|file| visit(&file.items, span))
}

pub fn checked_return_summaries(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> Vec<CheckedReturnSummary> {
    build_report(program, diagnostics)
        .checked_returns
        .iter()
        .map(CheckedReturnSummary::from)
        .collect()
}

pub fn type_check_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum type check ({TYPE_CHECK_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {TYPE_CHECK_MODE}\n"));
    out.push_str(&format!("status: {}\n", report.status()));
    out.push_str(&format!(
        "type_environment: schema={} status={} mode={} type_names={} declarations={} unknown_type_references={} resolver_errors={}\n",
        type_env::TYPE_ENV_SCHEMA,
        report.type_env.status(),
        type_env::TYPE_ENV_MODE,
        report.type_env.type_names.len(),
        report.type_env.declarations.len(),
        report.type_env.unknown_type_references(),
        report.resolver_errors()
    ));
    out.push_str(&format!(
        "summary: files={} items={} checked_declarations={} accepted_declarations={} rejected_declarations={} checked_type_references={} checked_returns={} accepted_returns={} rejected_returns={} unchecked_returns={} type_errors={} source_errors={} resolver_errors={}\n",
        report.type_env.files,
        report.type_env.items,
        report.checked_declarations.len(),
        report.accepted_declarations(),
        report.rejected_declarations(),
        report.checked_type_references(),
        report.checked_returns.len(),
        report.accepted_returns(),
        report.rejected_returns(),
        report.unchecked_returns(),
        report.type_error_count(),
        report.source_errors(),
        report.resolver_errors()
    ));

    if report.diagnostics.is_empty() {
        out.push_str("diagnostics: none\n");
    } else {
        out.push_str("diagnostics:\n");
        for diagnostic in &report.diagnostics {
            out.push_str(&format!(
                "  {}:{}:{}: {}[{}]: {}\n",
                diagnostic.source_span.file,
                diagnostic.source_span.line,
                diagnostic.source_span.column,
                diagnostic.severity.as_str(),
                diagnostic.code.as_str(),
                diagnostic.message
            ));
            out.push_str(&format!("    help: {}\n", diagnostic.help));
        }
    }

    if report.checked_declarations.is_empty() {
        out.push_str("checked_declarations: none\n");
    } else {
        out.push_str("checked_declarations:\n");
        for declaration in &report.checked_declarations {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} {} `{}`: {}\n",
                declaration.source_span.file,
                declaration.source_span.line,
                declaration.source_span.column,
                declaration.status,
                declaration.owner_kind,
                declaration.declaration_kind,
                declaration.name,
                declaration.type_text
            ));
            if !declaration.type_references.is_empty() {
                out.push_str("    type_references:");
                for reference in &declaration.type_references {
                    out.push_str(&format!(" {}[{}]", reference.text, reference.check_status));
                }
                out.push('\n');
            }
        }
    }

    if report.checked_returns.is_empty() {
        out.push_str("checked_returns: none\n");
    } else {
        out.push_str("checked_returns:\n");
        for checked_return in &report.checked_returns {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} `{}` return `{}` expected={} value={} actual={}\n",
                checked_return.source_span.file,
                checked_return.source_span.line,
                checked_return.source_span.column,
                checked_return.status,
                checked_return.owner_kind,
                checked_return.owner_name,
                checked_return.expression_text,
                checked_return.expected_type.as_deref().unwrap_or("none"),
                checked_return
                    .expected_value_type
                    .as_deref()
                    .unwrap_or("none"),
                checked_return.actual_type.as_deref().unwrap_or("unknown")
            ));
        }
    }

    out.push_str(&predicate::analyze_program(program).place_facts_text());
    out.push_str("non_claims:\n");
    for non_claim in NON_CLAIMS {
        out.push_str(&format!("  - {non_claim}\n"));
    }

    out
}

pub fn type_check_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", TYPE_CHECK_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", TYPE_CHECK_MODE, true);
    push_type_environment(&mut out, &report, 2, true);
    push_summary(&mut out, &report, 2, true);
    push_checked_declarations(&mut out, &report.checked_declarations, 2, true);
    push_checked_returns(&mut out, &report.checked_returns, 2, true);
    push_diagnostics(&mut out, &report.diagnostics, 2, true);
    push_indent(&mut out, 2);
    push_json_string(&mut out, "predicate_place_facts");
    out.push_str(": ");
    out.push_str(&predicate::analyze_program(program).place_facts_json());
    out.push_str(",\n");
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> TypeCheckReport {
    let type_env_report = type_env::type_env_report(program, diagnostics);
    let callable_blockers = callable::stage_blockers(program, "type_check");
    let blocked =
        type_env_report.source_errors > 0 || type_env_report.resolver_summary.resolver_errors > 0;
    let checked_declarations = type_env_report
        .declarations
        .iter()
        .map(|declaration| checked_declaration(declaration, blocked))
        .collect::<Vec<_>>();
    let mut diagnostics = if blocked {
        Vec::new()
    } else {
        type_diagnostics(&type_env_report.declarations)
    };
    let returns_blocked = blocked || !diagnostics.is_empty();
    let checked_returns = collect_checked_returns(program, returns_blocked);
    if !returns_blocked {
        diagnostics.extend(return_diagnostics(&checked_returns));
    }

    let mut diagnostic_occurrences = type_env_report.diagnostic_occurrences.inherited();
    for diagnostic in &diagnostics {
        let occurrence = type_diagnostic_occurrence(diagnostic);
        diagnostic_occurrences
            .insert_owned(occurrence)
            .expect("type diagnostic occurrences must be unique");
    }

    TypeCheckReport {
        type_env: type_env_report,
        callable_blockers,
        checked_declarations,
        checked_returns,
        diagnostics,
        diagnostic_occurrences,
    }
}

fn public_type_diagnostic(diagnostic: &TypeCheckDiagnostic) -> Diagnostic {
    let public_diagnostic = match diagnostic.severity {
        Severity::Error => Diagnostic::error(
            diagnostic.code,
            diagnostic.message.clone(),
            Some(diagnostic.source_span.clone()),
        ),
        Severity::Warning => Diagnostic::warning(
            diagnostic.code,
            diagnostic.message.clone(),
            Some(diagnostic.source_span.clone()),
        ),
    };
    public_diagnostic.with_help(diagnostic.help)
}

fn type_diagnostic_occurrence(diagnostic: &TypeCheckDiagnostic) -> DiagnosticOccurrence {
    let mut route = Vec::new();
    if let Some(declaration_id) = &diagnostic.declaration_id {
        route.push(format!("type_declaration={declaration_id}"));
    }
    if let Some(resolver_definition_id) = &diagnostic.resolver_definition_id {
        route.push(format!("resolver_definition={resolver_definition_id}"));
    }
    if let Some(type_name) = &diagnostic.type_name {
        route.push(format!("type_name={type_name}"));
    }
    if let Some(return_id) = &diagnostic.return_id {
        route.push(format!("return_relationship={return_id}"));
    }
    if let Some(expected) = &diagnostic.expected_type {
        route.push(format!("expected_type={expected}"));
    }
    if let Some(actual) = &diagnostic.actual_type {
        route.push(format!("actual_type={actual}"));
    }
    let mut semantic_origin = diagnostic
        .declaration_id
        .as_ref()
        .or(diagnostic.return_id.as_ref())
        .cloned()
        .unwrap_or_else(|| panic!("type diagnostic lacks semantic declaration/return identity"));
    if let Some(type_name) = &diagnostic.type_name {
        semantic_origin.push_str(":type-name=");
        semantic_origin.push_str(type_name);
    }
    route.insert(
        0,
        format!("type_cause_key={}", diagnostic.cause_key.ordinal()),
    );
    DiagnosticOccurrence::registered_cause(
        diagnostic.cause_key,
        public_type_diagnostic(diagnostic),
        semantic_origin,
        route,
    )
    .expect("type diagnostic cause must be registered")
}

fn checked_declaration(declaration: &TypeDeclaration, blocked: bool) -> CheckedDeclaration {
    let type_references = declaration
        .type_references
        .iter()
        .map(|reference| CheckedTypeReference {
            text: reference.text.clone(),
            normalized_name: reference.normalized_name.clone(),
            role: reference.role,
            type_env_status: reference.status,
            check_status: reference_check_status(reference.status, blocked),
        })
        .collect::<Vec<_>>();
    let status = if blocked {
        "not_checked_blocked_by_prior_errors_v0"
    } else if declaration
        .type_references
        .iter()
        .any(|reference| reference.status == "unknown_type_name_v0")
    {
        "rejected_unknown_type_name_v0"
    } else if declaration.status == "missing_type_annotation_v0" {
        "skipped_missing_type_annotation_v0"
    } else {
        "accepted_declaration_annotation_v0"
    };

    CheckedDeclaration {
        id: prefixed_id("hum_type_check_decl", &declaration.id),
        declaration_id: declaration.id.clone(),
        declaration_kind: declaration.declaration_kind,
        owner_kind: declaration.owner_kind,
        owner_name: declaration.owner_name.clone(),
        name: declaration.name.clone(),
        resolver_definition_id: declaration.resolver_definition_id.clone(),
        source_span: declaration.source_span.clone(),
        type_text: declaration.type_text.clone(),
        type_references,
        status,
    }
}

fn reference_check_status(type_env_status: &'static str, blocked: bool) -> &'static str {
    if blocked {
        "not_checked_prior_errors_v0"
    } else if type_env_status == "unknown_type_name_v0" {
        "rejected_unknown_type_name_v0"
    } else {
        "accepted_type_reference_v0"
    }
}

fn type_diagnostics(declarations: &[TypeDeclaration]) -> Vec<TypeCheckDiagnostic> {
    let mut diagnostics = Vec::new();
    for declaration in declarations {
        for reference in &declaration.type_references {
            if reference.status != "unknown_type_name_v0" {
                continue;
            }
            diagnostics.push(TypeCheckDiagnostic {
                cause_key: crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(82),
                code: DiagnosticCode::UNKNOWN_TYPE_NAME,
                severity: Severity::Error,
                title: DiagnosticCode::UNKNOWN_TYPE_NAME.title(),
                message: format!(
                    "unknown type `{}` in {} {} `{}` annotation",
                    reference.text,
                    declaration.owner_kind,
                    declaration.declaration_kind,
                    declaration.name
                ),
                source_span: declaration.source_span.clone(),
                help: "Declare the type, use a reserved type root, or wait for imports/packages before relying on external type names.",
                declaration_id: Some(declaration.id.clone()),
                resolver_definition_id: declaration.resolver_definition_id.clone(),
                type_name: Some(reference.text.clone()),
                return_id: None,
                expression_text: None,
                expected_type: None,
                actual_type: None,
            });
        }
    }
    diagnostics
}

fn collect_checked_returns(program: &Program, blocked: bool) -> Vec<CheckedReturn> {
    let mut checked_returns = Vec::new();
    for file in &program.files {
        collect_checked_returns_from_items(program, &file.items, blocked, &mut checked_returns);
    }
    checked_returns
}

fn collect_checked_returns_from_items(
    program: &Program,
    items: &[Item],
    blocked: bool,
    checked_returns: &mut Vec<CheckedReturn>,
) {
    for item in items {
        match item {
            Item::App(app) => {
                collect_checked_returns_from_items(program, &app.items, blocked, checked_returns)
            }
            Item::Task(task) => {
                checked_returns.extend(checked_returns_for_task(program, item, task, blocked))
            }
            _ => {}
        }
    }
}

fn checked_returns_for_task(
    program: &Program,
    item: &Item,
    task: &Task,
    blocked: bool,
) -> Vec<CheckedReturn> {
    let Some(section) = task.section("does") else {
        return Vec::new();
    };
    let body = core_body::analyze_does_section(
        program
            .canonical_core_expectation(item, section)
            .expect("live type-check task must have parser authority"),
    );
    let mut environment = initial_task_type_environment(task);
    let mut checked_returns = Vec::new();

    for statement in &body.statements {
        if statement.kind == "return" {
            checked_returns.push(checked_return(task, statement, &environment, blocked));
        }
        if let Some((name, fact)) = binding_type_fact(statement, &environment) {
            environment.insert(name_key(&name), fact);
        }
    }

    checked_returns
}

fn initial_task_type_environment(task: &Task) -> BTreeMap<String, TypeFact> {
    let mut environment = BTreeMap::new();
    for param in &task.params {
        let type_text = param.ty.trim();
        if !type_text.is_empty() {
            environment.insert(
                name_key(&param.name),
                TypeFact {
                    type_text: type_text.to_string(),
                    source: "parameter_annotation_v0",
                },
            );
        }
    }
    environment
}

fn checked_return(
    task: &Task,
    statement: &BodyStatement,
    environment: &BTreeMap<String, TypeFact>,
    blocked: bool,
) -> CheckedReturn {
    let expression_text = strip_keyword(&statement.text, "return")
        .unwrap_or("")
        .trim()
        .to_string();
    let expected_type = task.result.as_ref().map(|result| result.trim().to_string());
    let expected_value_type = expected_type.as_deref().map(expected_return_value_type);
    let actual = infer_expression_type(&expression_text, environment);
    let (status, reason) = if blocked {
        (
            "not_checked_blocked_by_prior_errors_v0",
            Some("prior_source_resolver_or_declaration_type_errors"),
        )
    } else if expected_type.is_none() {
        (
            "skipped_no_result_annotation_v0",
            Some("task_has_no_result_annotation"),
        )
    } else if actual.is_none() {
        (
            "unchecked_return_expression_v0",
            Some("return_expression_type_unknown_v0"),
        )
    } else if return_types_compatible(
        expected_type.as_deref().unwrap_or_default(),
        expected_value_type.as_deref().unwrap_or_default(),
        actual
            .as_ref()
            .map(|fact| fact.type_text.as_str())
            .unwrap_or_default(),
    ) {
        ("accepted_return_expression_v0", None)
    } else {
        (
            "rejected_return_type_mismatch_v0",
            Some("return_expression_type_mismatch"),
        )
    };

    CheckedReturn {
        id: prefixed_id(
            "hum_type_check_return",
            &format!("{}_{}", task.name, statement.span.line),
        ),
        owner_kind: "task",
        owner_name: task.name.clone(),
        source_span: statement.span.clone(),
        expression_text,
        expected_type,
        expected_value_type,
        actual_type: actual.as_ref().map(|fact| fact.type_text.clone()),
        type_source: actual.map(|fact| fact.source),
        status,
        reason,
    }
}

impl From<&CheckedReturn> for CheckedReturnSummary {
    fn from(checked_return: &CheckedReturn) -> Self {
        Self {
            id: checked_return.id.clone(),
            owner_kind: checked_return.owner_kind,
            owner_name: checked_return.owner_name.clone(),
            source_span: checked_return.source_span.clone(),
            expression_text: checked_return.expression_text.clone(),
            expected_type: checked_return.expected_type.clone(),
            expected_value_type: checked_return.expected_value_type.clone(),
            actual_type: checked_return.actual_type.clone(),
            type_source: checked_return.type_source,
            status: checked_return.status,
            reason: checked_return.reason,
        }
    }
}

fn binding_type_fact(
    statement: &BodyStatement,
    environment: &BTreeMap<String, TypeFact>,
) -> Option<(String, TypeFact)> {
    let keyword = match statement.kind {
        "let_binding" => "let",
        "mutable_binding" => "change",
        _ => return None,
    };
    let rest = strip_keyword(&statement.text, keyword)?;
    let (left, value) = rest.split_once('=')?;
    let (name, explicit_type) = binding_left_parts(left)?;
    let fact = if let Some(type_text) = explicit_type {
        TypeFact {
            type_text,
            source: "binding_annotation_v0",
        }
    } else {
        infer_expression_type(value.trim(), environment)?
    };
    Some((name, fact))
}

fn binding_left_parts(left: &str) -> Option<(String, Option<String>)> {
    let left = left.trim();
    if left.is_empty() {
        return None;
    }
    if let Some((name, type_text)) = left.split_once(':') {
        let name = name.trim();
        let type_text = type_text.trim();
        if name.is_empty() || type_text.is_empty() {
            return None;
        }
        Some((name.to_string(), Some(type_text.to_string())))
    } else {
        Some((left.to_string(), None))
    }
}

fn infer_expression_type(
    expression_text: &str,
    environment: &BTreeMap<String, TypeFact>,
) -> Option<TypeFact> {
    let text = expression_text.trim();
    if text.is_empty() {
        return Some(TypeFact {
            type_text: "Unit".to_string(),
            source: "unit_expression_v0",
        });
    }
    if text == "true" || text == "false" {
        return Some(TypeFact {
            type_text: "Bool".to_string(),
            source: "bool_literal_v0",
        });
    }
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return Some(TypeFact {
            type_text: "Text".to_string(),
            source: "text_literal_v0",
        });
    }
    if return_dependency::is_closed_view_derivation_expression(text) {
        return Some(TypeFact {
            type_text: "Text".to_string(),
            source: "closed_view_derivation_slice_until_v0",
        });
    }
    if text.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(TypeFact {
            type_text: "integer_literal".to_string(),
            source: "integer_literal_v0",
        });
    }
    if let Some(type_name) = record_literal_type_name(text) {
        return Some(TypeFact {
            type_text: type_name,
            source: "record_literal_constructor_v0",
        });
    }
    if let Some(root) = path_root_type_name(text) {
        return Some(TypeFact {
            type_text: root,
            source: "path_root_type_v0",
        });
    }
    environment.get(&name_key(text)).cloned()
}

fn record_literal_type_name(text: &str) -> Option<String> {
    let constructor = text.trim().strip_suffix('{')?.trim();
    if is_type_like_name(constructor) {
        Some(constructor.to_string())
    } else {
        None
    }
}

fn path_root_type_name(text: &str) -> Option<String> {
    let (root, _field) = text.split_once('.')?;
    let root = root.trim();
    if is_type_like_name(root) {
        Some(root.to_string())
    } else {
        None
    }
}

fn is_type_like_name(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
        && text
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == ' ')
}

fn expected_return_value_type(expected_type: &str) -> String {
    let expected_type = return_dependency::result_type_without_return_dependency(expected_type);
    let tokens = type_tokens(&expected_type);
    if matches!(
        tokens.first().map(String::as_str),
        Some("Result" | "Option" | "Maybe" | "Slice" | "Span")
    ) && tokens.len() >= 2
    {
        tokens[1].clone()
    } else {
        expected_type
    }
}

fn return_types_compatible(
    expected_type: &str,
    expected_value_type: &str,
    actual_type: &str,
) -> bool {
    let expected_type = return_dependency::result_type_without_return_dependency(expected_type);
    let actual_key = name_key(actual_type);
    if actual_key.is_empty() {
        return false;
    }
    if actual_key == name_key(&expected_type) || actual_key == name_key(expected_value_type) {
        return true;
    }
    actual_key == "integer_literal"
        && matches!(
            name_key(expected_value_type).as_str(),
            "int" | "uint" | "float"
        )
}

fn return_diagnostics(checked_returns: &[CheckedReturn]) -> Vec<TypeCheckDiagnostic> {
    checked_returns
        .iter()
        .filter(|checked_return| checked_return.status == "rejected_return_type_mismatch_v0")
        .map(|checked_return| TypeCheckDiagnostic {
            cause_key: crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(83),
            code: DiagnosticCode::RETURN_TYPE_MISMATCH,
            severity: Severity::Error,
            title: DiagnosticCode::RETURN_TYPE_MISMATCH.title(),
            message: format!(
                "return expression `{}` has type `{}` but task `{}` returns `{}`",
                checked_return.expression_text,
                checked_return.actual_type.as_deref().unwrap_or("unknown"),
                checked_return.owner_name,
                checked_return.expected_type.as_deref().unwrap_or("none")
            ),
            source_span: checked_return.source_span.clone(),
            help: "Return a value that matches the task result type, change the task result annotation, or leave complex cases unchecked until full expression typing exists.",
            declaration_id: None,
            resolver_definition_id: None,
            type_name: None,
            return_id: Some(checked_return.id.clone()),
            expression_text: Some(checked_return.expression_text.clone()),
            expected_type: checked_return.expected_type.clone(),
            actual_type: checked_return.actual_type.clone(),
        })
        .collect()
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

fn strip_keyword<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    if text == keyword {
        return Some("");
    }
    text.strip_prefix(keyword)
        .and_then(|rest| rest.strip_prefix(char::is_whitespace))
        .map(str::trim)
}
impl TypeCheckReport {
    fn status(&self) -> &'static str {
        if self.source_errors() > 0 {
            "blocked_by_source_errors"
        } else if self.resolver_errors() > 0 {
            "blocked_by_resolver_errors"
        } else if self.type_error_count() > 0 {
            "type_errors_v0"
        } else if self.callable_blockers > 0 {
            "blocked_by_callable_errors_v0"
        } else {
            "declaration_annotations_and_trivial_returns_checked_v0"
        }
    }

    fn source_errors(&self) -> usize {
        self.type_env.source_errors
    }

    fn resolver_errors(&self) -> usize {
        self.type_env.resolver_summary.resolver_errors
    }

    fn type_error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .count()
    }

    fn type_warning_count(&self) -> usize {
        self.diagnostics
            .len()
            .saturating_sub(self.type_error_count())
    }

    fn accepted_declarations(&self) -> usize {
        self.checked_declarations
            .iter()
            .filter(|declaration| declaration.status == "accepted_declaration_annotation_v0")
            .count()
    }

    fn rejected_declarations(&self) -> usize {
        self.checked_declarations
            .iter()
            .filter(|declaration| declaration.status == "rejected_unknown_type_name_v0")
            .count()
    }

    fn checked_type_references(&self) -> usize {
        self.checked_declarations
            .iter()
            .map(|declaration| declaration.type_references.len())
            .sum()
    }

    fn accepted_returns(&self) -> usize {
        self.checked_returns
            .iter()
            .filter(|checked_return| checked_return.status == "accepted_return_expression_v0")
            .count()
    }

    fn rejected_returns(&self) -> usize {
        self.checked_returns
            .iter()
            .filter(|checked_return| checked_return.status == "rejected_return_type_mismatch_v0")
            .count()
    }

    fn unchecked_returns(&self) -> usize {
        self.checked_returns
            .iter()
            .filter(|checked_return| {
                matches!(
                    checked_return.status,
                    "unchecked_return_expression_v0"
                        | "skipped_no_result_annotation_v0"
                        | "not_checked_blocked_by_prior_errors_v0"
                )
            })
            .count()
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

fn name_key(name: &str) -> String {
    snake_identifier(name)
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

fn push_type_environment(out: &mut String, report: &TypeCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "type_environment");
    out.push_str(": {\n");
    push_string_field(out, indent + 2, "schema", type_env::TYPE_ENV_SCHEMA, true);
    push_string_field(out, indent + 2, "status", report.type_env.status(), true);
    push_string_field(out, indent + 2, "mode", type_env::TYPE_ENV_MODE, true);
    push_usize_field(
        out,
        indent + 2,
        "type_names",
        report.type_env.type_names.len(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "declarations",
        report.type_env.declarations.len(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_references",
        report.type_env.type_reference_count(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unknown_type_references",
        report.type_env.unknown_type_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        report.resolver_errors(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_summary(out: &mut String, report: &TypeCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "summary");
    out.push_str(": {\n");
    push_usize_field(out, indent + 2, "files", report.type_env.files, true);
    push_usize_field(out, indent + 2, "items", report.type_env.items, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        report.source_errors(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "source_warnings",
        report.type_env.source_warnings,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        report.resolver_errors(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_declarations",
        report.checked_declarations.len(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_declarations",
        report.accepted_declarations(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_declarations",
        report.rejected_declarations(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_type_references",
        report.checked_type_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_returns",
        report.checked_returns.len(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_returns",
        report.accepted_returns(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_returns",
        report.rejected_returns(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_returns",
        report.unchecked_returns(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unknown_type_references",
        report.type_env.unknown_type_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_errors",
        report.type_error_count(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_warnings",
        report.type_warning_count(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_checked_declarations(
    out: &mut String,
    declarations: &[CheckedDeclaration],
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "checked_declarations");
    out.push_str(": [");
    if !declarations.is_empty() {
        out.push('\n');
        for (index, declaration) in declarations.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_checked_declaration(out, declaration, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_checked_declaration(out: &mut String, declaration: &CheckedDeclaration, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &declaration.id, true);
    push_string_field(
        out,
        indent + 2,
        "declaration_id",
        &declaration.declaration_id,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "declaration_kind",
        declaration.declaration_kind,
        true,
    );
    push_string_field(out, indent + 2, "owner_kind", declaration.owner_kind, true);
    push_string_field(out, indent + 2, "owner_name", &declaration.owner_name, true);
    push_string_field(out, indent + 2, "name", &declaration.name, true);
    push_optional_string_field(
        out,
        indent + 2,
        "resolver_definition_id",
        declaration.resolver_definition_id.as_deref(),
        true,
    );
    push_span_field(
        out,
        indent + 2,
        "source_span",
        &declaration.source_span,
        true,
    );
    push_string_field(out, indent + 2, "type_text", &declaration.type_text, true);
    push_checked_type_references(out, &declaration.type_references, indent + 2, true);
    push_string_field(out, indent + 2, "status", declaration.status, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_checked_type_references(
    out: &mut String,
    references: &[CheckedTypeReference],
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "type_references");
    out.push_str(": [");
    if !references.is_empty() {
        out.push('\n');
        for (index, reference) in references.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            out.push_str("{\n");
            push_string_field(out, indent + 4, "text", &reference.text, true);
            push_string_field(
                out,
                indent + 4,
                "normalized_name",
                &reference.normalized_name,
                true,
            );
            push_string_field(out, indent + 4, "role", reference.role, true);
            push_string_field(
                out,
                indent + 4,
                "type_env_status",
                reference.type_env_status,
                true,
            );
            push_string_field(
                out,
                indent + 4,
                "check_status",
                reference.check_status,
                false,
            );
            push_indent(out, indent + 2);
            out.push('}');
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_checked_returns(out: &mut String, returns: &[CheckedReturn], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "checked_returns");
    out.push_str(": [");
    if !returns.is_empty() {
        out.push('\n');
        for (index, checked_return) in returns.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_checked_return(out, checked_return, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_checked_return(out: &mut String, checked_return: &CheckedReturn, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &checked_return.id, true);
    push_string_field(
        out,
        indent + 2,
        "owner_kind",
        checked_return.owner_kind,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "owner_name",
        &checked_return.owner_name,
        true,
    );
    push_span_field(
        out,
        indent + 2,
        "source_span",
        &checked_return.source_span,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "expression_text",
        &checked_return.expression_text,
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "expected_type",
        checked_return.expected_type.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "expected_value_type",
        checked_return.expected_value_type.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "actual_type",
        checked_return.actual_type.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "type_source",
        checked_return.type_source,
        true,
    );
    push_string_field(out, indent + 2, "status", checked_return.status, true);
    push_optional_string_field(out, indent + 2, "reason", checked_return.reason, false);
    push_indent(out, indent);
    out.push('}');
}
fn push_diagnostics(
    out: &mut String,
    diagnostics: &[TypeCheckDiagnostic],
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "diagnostics");
    out.push_str(": [");
    if !diagnostics.is_empty() {
        out.push('\n');
        for (index, diagnostic) in diagnostics.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_diagnostic(out, diagnostic, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_diagnostic(out: &mut String, diagnostic: &TypeCheckDiagnostic, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "code", diagnostic.code.as_str(), true);
    push_string_field(out, indent + 2, "title", diagnostic.title, true);
    push_string_field(
        out,
        indent + 2,
        "severity",
        diagnostic.severity.as_str(),
        true,
    );
    push_string_field(out, indent + 2, "message", &diagnostic.message, true);
    push_span_field(
        out,
        indent + 2,
        "source_span",
        &diagnostic.source_span,
        true,
    );
    push_string_field(out, indent + 2, "help", diagnostic.help, true);
    push_optional_string_field(
        out,
        indent + 2,
        "declaration_id",
        diagnostic.declaration_id.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "type_name",
        diagnostic.type_name.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "return_id",
        diagnostic.return_id.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "expression_text",
        diagnostic.expression_text.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "expected_type",
        diagnostic.expected_type.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "actual_type",
        diagnostic.actual_type.as_deref(),
        false,
    );
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

    use super::{
        build_report, diagnostic_occurrence_set_from_source, resolver_precedence_relationships,
        type_check_has_errors, type_check_json, type_check_text, type_diagnostic_occurrence,
        type_diagnostics,
    };

    #[test]
    fn resolver_precedence_is_consumed_for_a_genuine_blocked_type_relationship() {
        let parsed = parse_source(
            "resolver-type-precedence.hum",
            r#"task duplicate(value: MissingType, value: MissingType) -> UInt {
  does:
    return 1
}
"#,
        );
        let checked = crate::check::check_file_with_occurrences(&parsed);
        let mut source_occurrences = parsed.diagnostic_occurrences.clone();
        source_occurrences
            .extend_owned(&checked.diagnostic_occurrences)
            .expect("source occurrences");
        let mut diagnostics = parsed.diagnostics;
        diagnostics.extend(checked.diagnostics);
        let program = Program {
            files: vec![parsed.file],
        };
        let occurrences =
            diagnostic_occurrence_set_from_source(&program, &diagnostics, &source_occurrences);
        let report = build_report(&program, &diagnostics);
        let suppressed = type_diagnostics(&report.type_env.declarations)
            .iter()
            .map(type_diagnostic_occurrence)
            .collect::<Vec<_>>();
        let relationships = resolver_precedence_relationships(&occurrences, &suppressed);
        assert_eq!(relationships.len(), 1);
        let application = relationships[0].application();
        assert_eq!(application.rule_id, "resolver_over_type_v0");
        assert_eq!(application.applying_stage, "type_check");
        assert!(
            application
                .relationship_route
                .iter()
                .any(|entry| entry.starts_with("resolver_definition=def_"))
        );
    }

    #[test]
    fn json_rejects_unknown_declared_annotation_names() {
        let program = demo_program_with_unknown();
        let json = type_check_json(&program, &[]);

        assert!(type_check_has_errors(&program, &[]));
        assert!(json.contains("\"schema\": \"hum.type_check.v0\""));
        assert!(json.contains("\"mode\": \"declaration_annotation_and_trivial_return_check_v0\""));
        assert!(json.contains("\"schema\": \"hum.type_env.v0\""));
        assert!(json.contains("\"status\": \"type_errors_v0\""));
        assert!(json.contains("\"code\": \"H0605\""));
        assert!(json.contains("\"type_name\": \"WorkError\""));
        assert!(json.contains("\"check_status\": \"rejected_unknown_type_name_v0\""));
        assert!(json.contains("\"no full expression type inference\""));
    }

    #[test]
    fn json_accepts_declarations_and_trivial_result_return() {
        let program = demo_program_without_unknown();
        let json = type_check_json(&program, &[]);

        assert!(!type_check_has_errors(&program, &[]));
        assert!(
            json.contains("\"status\": \"declaration_annotations_and_trivial_returns_checked_v0\"")
        );
        assert!(json.contains("\"type_errors\": 0"));
        assert!(json.contains("\"accepted_declaration_annotation_v0\""));
        assert!(json.contains("\"accepted_type_reference_v0\""));
        assert!(json.contains("\"checked_returns\""));
        assert!(json.contains("\"expected_type\": \"Result WorkItem, WorkError\""));
        assert!(json.contains("\"expected_value_type\": \"WorkItem\""));
        assert!(json.contains("\"actual_type\": \"WorkItem\""));
        assert!(json.contains("\"status\": \"accepted_return_expression_v0\""));
        assert!(json.contains("\"type_source\": \"record_literal_constructor_v0\""));
    }

    #[test]
    fn json_rejects_trivial_return_type_mismatch() {
        let program = parse_program(
            r#"task bad_return(title: Text) -> UInt {
  does:
    return title
}
"#,
        );
        let json = type_check_json(&program, &[]);

        assert!(type_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"type_errors_v0\""));
        assert!(json.contains("\"code\": \"H0606\""));
        assert!(json.contains("\"title\": \"return type mismatch\""));
        assert!(json.contains("\"expected_type\": \"UInt\""));
        assert!(json.contains("\"actual_type\": \"Text\""));
        assert!(json.contains("\"status\": \"rejected_return_type_mismatch_v0\""));
        assert!(json.contains("\"rejected_returns\": 1"));
    }

    #[test]
    fn summary_reports_type_check_gate_counts() {
        let program = demo_program_with_unknown();
        let summary = super::type_check_summary(&program, &[]);

        assert_eq!(summary.schema, "hum.type_check.v0");
        assert_eq!(summary.status, "type_errors_v0");
        assert_eq!(
            summary.mode,
            "declaration_annotation_and_trivial_return_check_v0"
        );
        assert_eq!(summary.source_errors, 0);
        assert_eq!(summary.resolver_errors, 0);
        assert_eq!(summary.type_errors, 1);
        assert_eq!(summary.unknown_type_references, 1);
        assert_eq!(summary.rejected_declarations, 1);
        assert_eq!(summary.checked_returns, 1);
        assert_eq!(summary.unchecked_returns, 1);
    }

    #[test]
    fn resolver_errors_block_type_check_authority() {
        let source = r#"task bad_names() -> UInt {
  does:
    return missing
}
"#;
        let parsed = parse_source("bad.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = type_check_json(&program, &[]);

        assert!(type_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"blocked_by_resolver_errors\""));
        assert!(json.contains("\"type_errors\": 0"));
        assert!(json.contains("\"diagnostics\": []"));
    }

    #[test]
    fn text_report_names_unknown_type_diagnostics() {
        let program = demo_program_with_unknown();
        let text = type_check_text(&program, &[]);

        assert!(text.contains("Hum type check (hum.type_check.v0)"));
        assert!(text.contains("status: type_errors_v0"));
        assert!(text.contains("error[H0605]"));
        assert!(text.contains("WorkError[rejected_unknown_type_name_v0]"));
    }

    fn demo_program_with_unknown() -> Program {
        parse_program(
            r#"type WorkItem {
  title: Text
  done: Bool
}

task remember_work_item(title: Text) -> Result WorkItem, WorkError {
  changes:
    work_items

  does:
    return title
}
"#,
        )
    }

    fn demo_program_without_unknown() -> Program {
        parse_program(
            r#"type WorkItem {
  title: Text
  done: Bool
}

type WorkError {
  code: Text
}

task remember_work_item(title: Text) -> Result WorkItem, WorkError {
  changes:
    work_items

  does:
    let item = WorkItem {
      title: title
      done: false
    }
    return item
}
"#,
        )
    }

    fn parse_program(source: &str) -> Program {
        let parsed = parse_source("types.hum", source);
        Program {
            files: vec![parsed.file],
        }
    }
}
