#![allow(unexpected_cfgs)]

use crate::ast::{
    CanonicalExpression, CanonicalExpressionKind, ParsedBinaryOperator, ParsedSourceRange, Program,
};
use crate::callable;
use crate::core_contract;
use crate::core_expr;
use crate::core_lower::{
    self, CanonicalMinimalAddLowering, CoreLowerExpression, CoreLowerItem, CoreLowerOperation,
    CoreLowerReport, CoreLowerSourceRange, CoreLowerStructuredExpression,
};
use crate::core_preview;
use crate::diagnostic::{Diagnostic, DiagnosticOccurrenceSet, Span};
use crate::ir_contract;
use crate::predicate;
use crate::resolve;
use crate::type_check::{
    self, CanonicalMinimalAddTypeAuthority, CanonicalMinimalAddTypeClassification,
    CanonicalMinimalAddTypeClassifications, CanonicalMinimalAddTypeRecord,
};
use crate::version;

pub const CORE_VERIFY_SCHEMA: &str = "hum.core_verify.v0";
pub const CORE_VERIFY_STATUS: &str = "verified_non_executing_core_artifact_v0";
pub const CORE_VERIFY_FAILED_STATUS: &str = "core_artifact_verification_failed_v0";
pub const CORE_VERIFY_MODE: &str = "non_executing_artifact_invariant_check_v0";

const NON_GOALS: &[&str] = &[
    "no executable semantics",
    "no Hum IR emission",
    "no backend lowering",
    "no proof artifact",
    "no memory-safety proof",
    "no optimization claim",
    "no full type checking",
    "no effect checking",
    "no ownership checking",
    "no profile enforcement",
];

pub struct CoreVerifyReadinessSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub files: usize,
    pub items: usize,
    pub tasks: usize,
    pub tests: usize,
    pub core_items: usize,
    pub verified_items: usize,
    pub lower_blocked_items: usize,
    pub operations: usize,
    pub verified_operations: usize,
    pub lower_blocked_operations: usize,
    pub checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub execution_ready: usize,
    pub ir_ready: usize,
    pub errors: usize,
    pub warnings: usize,
    pub resolver_errors: usize,
    pub type_errors: usize,
    pub preview_blocked_statements: usize,
}

struct CoreVerifyReport {
    lower: CanonicalMinimalAddLowering,
    checks: Vec<CoreVerifyCheck>,
}

struct CoreVerifyCheck {
    id: String,
    scope: &'static str,
    scope_id: String,
    span: Option<Span>,
    status: &'static str,
    rule: &'static str,
    detail: String,
}

pub(crate) struct VerifiedCanonicalMinimalAddType<'artifact, 'authority> {
    item: &'artifact CoreLowerItem,
    operation: &'artifact CoreLowerOperation,
    authority: &'authority CanonicalMinimalAddTypeAuthority,
}

impl VerifiedCanonicalMinimalAddType<'_, '_> {
    pub(crate) fn item_identity(&self) -> &str {
        &self.item.semantic_identity
    }

    pub(crate) fn statement_index(&self) -> usize {
        self.operation.index
    }

    pub(crate) fn root_node_id(&self) -> &str {
        self.authority
            .key()
            .root_node_id()
            .expect("verified canonical minimal-add has a root identity")
    }

    pub(crate) fn expected_type(&self) -> &str {
        self.authority.declared_result_type()
    }

    pub(crate) fn actual_type(&self) -> &'static str {
        self.authority.expression_type()
    }
}

pub fn core_verify_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum Core verify ({CORE_VERIFY_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!(
        "verification_status: {}\nmode: {CORE_VERIFY_MODE}\ncore_contract_schema: {}\ncore_lower_schema: {}\ncore_preview_schema: {}\nir_contract_schema: {}\n",
        report.verification_status(),
        core_contract::CORE_CONTRACT_SCHEMA,
        core_lower::CORE_LOWER_SCHEMA,
        core_preview::CORE_PREVIEW_SCHEMA,
        ir_contract::IR_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "summary: files={} items={} tasks={} tests={} core_items={} verified_items={} lower_blocked_items={} operations={} verified_operations={} lower_blocked_operations={} checks={} passed_checks={} failed_checks={} execution_ready=0 ir_ready=0 errors={} warnings={} resolver_errors={} type_errors={} preview_blocked_statements={}\n",
        report.lower.files,
        report.lower.items,
        report.lower.tasks,
        report.lower.tests,
        report.lower.core_items.len(),
        report.verified_items(),
        report.lower.blocked_items(),
        report.operations(),
        report.verified_operations(),
        report.lower.blocked_operations(),
        report.checks.len(),
        report.passed_checks(),
        report.failed_checks(),
        report.lower.errors,
        report.lower.warnings,
        report.lower.resolver_errors,
        report.lower.type_errors,
        report.lower.preview_blocked_statements
    ));
    out.push_str(&format!(
        "core_lower: schema={} status={} lowered_items={} blocked_items={} lowered_operations={} blocked_operations={} execution_ready=0 ir_ready=0\n",
        core_lower::CORE_LOWER_SCHEMA,
        core_lower::CORE_LOWER_STATUS,
        report.lower.lowered_items(),
        report.lower.blocked_items(),
        report.lower.lowered_operations(),
        report.lower.blocked_operations()
    ));

    if report.lower.core_items.is_empty() {
        out.push_str("core_items: none\n");
    } else {
        out.push_str("core_items:\n");
        for item in &report.lower.core_items {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} `{}` lower_status={} operations={} blockers={}\n",
                item.span.file,
                item.span.line,
                item.span.column,
                report.item_status(item),
                item.kind,
                item.name,
                item.status,
                item.operations.len(),
                item.blockers.len()
            ));
        }
    }

    if report.failed_checks() == 0 {
        out.push_str("verification_failures: none\n");
    } else {
        out.push_str("verification_failures:\n");
        for check in report
            .checks
            .iter()
            .filter(|check| check.status == "failed_v0")
        {
            if let Some(span) = &check.span {
                out.push_str(&format!(
                    "  {}:{}:{} [{}] {}: {}\n",
                    span.file, span.line, span.column, check.rule, check.scope_id, check.detail
                ));
            } else {
                out.push_str(&format!(
                    "  [{}] {}: {}\n",
                    check.rule, check.scope_id, check.detail
                ));
            }
        }
    }

    out.push_str(&predicate::analyze_program(program).place_facts_text());
    out
}

pub fn core_verify_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", CORE_VERIFY_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(
        &mut out,
        2,
        "verification_status",
        report.verification_status(),
        true,
    );
    push_string_field(&mut out, 2, "mode", CORE_VERIFY_MODE, true);
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
        "core_lower_schema",
        core_lower::CORE_LOWER_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "core_preview_schema",
        core_preview::CORE_PREVIEW_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "resolve_schema",
        resolve::RESOLVE_REPORT_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "type_check_schema",
        type_check::TYPE_CHECK_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "ir_contract_schema",
        ir_contract::IR_CONTRACT_SCHEMA,
        true,
    );
    push_summary(&mut out, &report, 2, true);
    push_core_lower_summary(&mut out, &report, 2, true);
    push_items(&mut out, &report, 2, true);
    push_checks(&mut out, &report.checks, 2, true);
    push_indent(&mut out, 2);
    push_json_string(&mut out, "predicate_place_facts");
    out.push_str(": ");
    out.push_str(&predicate::analyze_program(program).place_facts_json());
    out.push_str(",\n");
    push_string_array(&mut out, 2, "non_goals_v0", NON_GOALS, false);
    out.push_str("}\n");
    out
}

pub fn core_verify_has_errors(program: &Program, diagnostics: &[Diagnostic]) -> bool {
    build_report(program, diagnostics).failed_checks() > 0
}

pub fn core_verify_readiness_summary(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> CoreVerifyReadinessSummary {
    let report = build_report(program, diagnostics);
    CoreVerifyReadinessSummary {
        schema: CORE_VERIFY_SCHEMA,
        status: report.verification_status(),
        mode: CORE_VERIFY_MODE,
        files: report.lower.files,
        items: report.lower.items,
        tasks: report.lower.tasks,
        tests: report.lower.tests,
        core_items: report.lower.core_items.len(),
        verified_items: report.verified_items(),
        lower_blocked_items: report.lower.blocked_items(),
        operations: report.operations(),
        verified_operations: report.verified_operations(),
        lower_blocked_operations: report.lower.blocked_operations(),
        checks: report.checks.len(),
        passed_checks: report.passed_checks(),
        failed_checks: report.failed_checks(),
        execution_ready: 0,
        ir_ready: 0,
        errors: report.lower.errors,
        warnings: report.lower.warnings,
        resolver_errors: report.lower.resolver_errors,
        type_errors: report.lower.type_errors,
        preview_blocked_statements: report.lower.preview_blocked_statements,
    }
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> CoreVerifyReport {
    let preview_authority = core_preview::diagnostic_occurrence_set(program, diagnostics);
    let lower = core_lower::build_canonical_minimal_add_lowering_from_preview(
        program,
        diagnostics,
        &preview_authority,
    )
    .expect("Core lower must preserve one sealed preview occurrence projection");
    lower
        .diagnostic_projection
        .validate_against("core_lower", &preview_authority)
        .expect("Core verify must compare lower projection with preview authority");
    let (mut checks, _) = verify_canonical_minimal_add_lowering(&lower);
    append_callable_checks(program, &mut checks);
    CoreVerifyReport { lower, checks }
}

fn append_callable_checks(program: &Program, checks: &mut Vec<CoreVerifyCheck>) {
    let callable_failures = callable::analyze_program(program).verify();
    if callable_failures.is_empty() {
        push_check(
            checks,
            "callable_semantic_spine",
            "session-al-callable-facts",
            None,
            true,
            "callable_closed_fact_consistency",
            "callable definition, type, row, value, and application facts are internally consistent",
        );
    } else {
        for failure in callable_failures {
            push_check(
                checks,
                "callable_semantic_spine",
                "session-al-callable-facts",
                None,
                false,
                failure,
                format!("callable fact verification failed: {failure}"),
            );
        }
    }
}

pub(crate) fn with_verified_canonical_minimal_add_types<R>(
    program: &Program,
    diagnostics: &[Diagnostic],
    consume: impl for<'artifact, 'authority> FnOnce(
        &[VerifiedCanonicalMinimalAddType<'artifact, 'authority>],
    ) -> R,
) -> (CoreVerifyReadinessSummary, R) {
    let preview_authority = core_preview::diagnostic_occurrence_set(program, diagnostics);
    let lower = core_lower::build_canonical_minimal_add_lowering_from_preview(
        program,
        diagnostics,
        &preview_authority,
    )
    .expect("Core lower must preserve one sealed preview occurrence projection");
    lower
        .diagnostic_projection
        .validate_against("core_lower", &preview_authority)
        .expect("Core verify must compare lower projection with preview authority");
    let (mut checks, views) = verify_canonical_minimal_add_lowering(&lower);
    append_callable_checks(program, &mut checks);
    let summary = readiness_summary(&lower, &checks);
    let consumed = consume(&views);
    (summary, consumed)
}

#[cfg(test)]
pub(crate) fn with_verified_canonical_minimal_add_types_from_owner_for_test<R>(
    lower: &CanonicalMinimalAddLowering,
    consume: impl for<'artifact, 'authority> FnOnce(
        &[VerifiedCanonicalMinimalAddType<'artifact, 'authority>],
    ) -> R,
) -> R {
    let (_, views) = verify_canonical_minimal_add_lowering(lower);
    consume(&views)
}

fn readiness_summary(
    lower: &CoreLowerReport,
    checks: &[CoreVerifyCheck],
) -> CoreVerifyReadinessSummary {
    let failed_checks = checks
        .iter()
        .filter(|check| check.status == "failed_v0")
        .count();
    let passed_checks = checks.len().saturating_sub(failed_checks);
    let item_failed = |item: &CoreLowerItem| {
        checks.iter().any(|check| {
            check.status == "failed_v0"
                && (check.scope_id == item.id
                    || item
                        .operations
                        .iter()
                        .any(|operation| operation.id == check.scope_id))
        })
    };
    let verified_items = lower
        .core_items
        .iter()
        .filter(|item| !item_failed(item))
        .count();
    let verified_operations = lower
        .core_items
        .iter()
        .flat_map(|item| &item.operations)
        .filter(|operation| {
            !checks
                .iter()
                .any(|check| check.status == "failed_v0" && check.scope_id == operation.id)
        })
        .count();
    CoreVerifyReadinessSummary {
        schema: CORE_VERIFY_SCHEMA,
        status: if failed_checks == 0 {
            CORE_VERIFY_STATUS
        } else {
            CORE_VERIFY_FAILED_STATUS
        },
        mode: CORE_VERIFY_MODE,
        files: lower.files,
        items: lower.items,
        tasks: lower.tasks,
        tests: lower.tests,
        core_items: lower.core_items.len(),
        verified_items,
        lower_blocked_items: lower.blocked_items(),
        operations: lower.lowered_operations() + lower.blocked_operations(),
        verified_operations,
        lower_blocked_operations: lower.blocked_operations(),
        checks: checks.len(),
        passed_checks,
        failed_checks,
        execution_ready: 0,
        ir_ready: 0,
        errors: lower.errors,
        warnings: lower.warnings,
        resolver_errors: lower.resolver_errors,
        type_errors: lower.type_errors,
        preview_blocked_statements: lower.preview_blocked_statements,
    }
}

pub(crate) fn diagnostic_occurrence_set(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> DiagnosticOccurrenceSet {
    build_report(program, diagnostics)
        .lower
        .diagnostic_occurrences
        .clone()
}

pub(crate) fn validate_diagnostic_projection_from_source(
    program: &Program,
    diagnostics: &[Diagnostic],
    source_occurrences: &DiagnosticOccurrenceSet,
) -> Result<DiagnosticOccurrenceSet, crate::diagnostic::DiagnosticInvariantError> {
    let authoritative = core_preview::diagnostic_occurrence_set_from_source(
        program,
        diagnostics,
        source_occurrences,
    );
    let lower =
        core_lower::build_core_lower_report_from_preview(program, diagnostics, &authoritative)?;
    lower
        .diagnostic_projection
        .validate_against("core_lower", &authoritative)?;
    DiagnosticOccurrenceSet::validate_projection_from(
        &authoritative,
        &lower.diagnostic_occurrences,
    )?;
    Ok(lower.diagnostic_occurrences)
}

#[cfg(test)]
fn verify_lower_report(lower: &CoreLowerReport) -> Vec<CoreVerifyCheck> {
    let mut views = Vec::new();
    verify_lower_report_with_classifications(lower, None, &mut views)
}

fn verify_canonical_minimal_add_lowering<'lowering>(
    lower: &'lowering CanonicalMinimalAddLowering,
) -> (
    Vec<CoreVerifyCheck>,
    Vec<VerifiedCanonicalMinimalAddType<'lowering, 'lowering>>,
) {
    let mut views = Vec::new();
    let checks = verify_lower_report_with_classifications(
        lower.report(),
        Some(lower.classifications()),
        &mut views,
    );
    (checks, views)
}

fn verify_lower_report_with_classifications<'artifact, 'authority>(
    lower: &'artifact CoreLowerReport,
    classifications: Option<&'authority CanonicalMinimalAddTypeClassifications>,
    views: &mut Vec<VerifiedCanonicalMinimalAddType<'artifact, 'authority>>,
) -> Vec<CoreVerifyCheck> {
    let mut checks = Vec::new();
    let classification_batch_valid = classifications
        .is_none_or(|classifications| canonical_minimal_add_batch_matches(lower, classifications));
    push_check(
        &mut checks,
        "summary",
        "core-lower-summary",
        None,
        lower.execution_ready == 0,
        "claim_honesty",
        "execution_ready remains 0",
    );
    push_check(
        &mut checks,
        "summary",
        "core-lower-summary",
        None,
        lower.ir_ready == 0,
        "claim_honesty",
        "ir_ready remains 0",
    );
    push_check(
        &mut checks,
        "summary",
        "core-lower-summary",
        None,
        core_lower::CORE_LOWER_STATUS == "unverified_core_artifact_v0",
        "claim_honesty",
        "core-lower status is explicitly unverified",
    );

    for item in &lower.core_items {
        verify_item(
            item,
            classifications,
            classification_batch_valid,
            &mut checks,
            views,
        );
    }
    if !classification_batch_valid
        && !checks.iter().any(|check| {
            check.status == "failed_v0" && check.rule.starts_with("canonical_minimal_add_")
        })
    {
        push_canonical_minimal_add_batch_failure(
            &mut checks,
            lower
                .core_items
                .iter()
                .flat_map(|item| &item.operations)
                .next(),
        );
    }

    checks
}

fn push_canonical_minimal_add_batch_failure(
    checks: &mut Vec<CoreVerifyCheck>,
    operation: Option<&CoreLowerOperation>,
) {
    let (scope, scope_id, span) = operation
        .map_or(("summary", "core-lower-summary", None), |operation| {
            ("operation", operation.id.as_str(), Some(&operation.span))
        });
    for (rule, detail) in [
        (
            "canonical_minimal_add_public_projection_matches_authority",
            "canonical minimal-add public projection does not match untouched producer authority",
        ),
        (
            "canonical_minimal_add_private_claim_matches_authority",
            "canonical minimal-add private claim does not match untouched producer authority",
        ),
        (
            "canonical_minimal_add_verified_view_issued",
            "canonical minimal-add verified view withheld after failed or missing authority check",
        ),
    ] {
        push_check(checks, scope, scope_id, span, false, rule, detail);
    }
}

fn canonical_minimal_add_batch_matches(
    lower: &CoreLowerReport,
    classifications: &CanonicalMinimalAddTypeClassifications,
) -> bool {
    let operations = lower
        .core_items
        .iter()
        .flat_map(|item| {
            item.operations.iter().filter_map(move |operation| {
                operation
                    .canonical_minimal_add_classification_ordinal()
                    .map(|ordinal| (item, operation, ordinal))
            })
        })
        .collect::<Vec<_>>();
    let records = classifications.records();
    if operations.len() != records.len() {
        return false;
    }
    let mut identities = std::collections::BTreeSet::new();
    operations.iter().zip(records).enumerate().all(
        |(expected_ordinal, ((item, operation, ordinal), record))| {
            let key = record.key();
            let root_node_id = operation
                .expression
                .as_ref()
                .and_then(CoreLowerExpression::structured_authority)
                .map(|authority| authority.node_id.as_str());
            let root_identity_matches = match record.classification() {
                CanonicalMinimalAddTypeClassification::Noncanonical => root_node_id.is_none(),
                _ => key.root_node_id() == root_node_id,
            };
            let identity = (
                key.source_revision().map(<[u8]>::to_vec),
                key.item_path().map(<[usize]>::to_vec),
                key.task_name().to_string(),
                key.item_identity().to_string(),
                key.statement_index(),
                crate::node_id::source_path_identity(&key.statement_span().file),
                key.statement_span().line,
                key.statement_span().column,
                key.root_node_id().map(str::to_string),
            );
            let ordinal_matches = *ordinal == expected_ordinal;
            let task_matches = key.task_name() == item.name;
            let item_path_matches = key.item_path().map_or_else(
                || {
                    matches!(
                        record.classification(),
                        CanonicalMinimalAddTypeClassification::Noncanonical
                    )
                },
                |path| {
                    item.semantic_identity.ends_with(&format!(
                        "path-{}",
                        path.iter()
                            .map(usize::to_string)
                            .collect::<Vec<_>>()
                            .join(".")
                    ))
                },
            );
            let item_identity_matches = key.item_identity() == item.semantic_identity;
            let statement_index_matches = key.statement_index() == operation.index;
            let statement_path_matches =
                crate::node_id::source_path_identity(&key.statement_span().file)
                    == crate::node_id::source_path_identity(&operation.span.file);
            let statement_line_matches = key.statement_span().line == operation.span.line;
            let statement_column_matches = key.statement_span().column == operation.span.column;
            let classification_key_matches = record
                .classification()
                .key()
                .is_none_or(|classification_key| classification_key == key);
            let identity_unique = identities.insert(identity);
            ordinal_matches
                && task_matches
                && item_path_matches
                && item_identity_matches
                && statement_index_matches
                && statement_path_matches
                && statement_line_matches
                && statement_column_matches
                && root_identity_matches
                && classification_key_matches
                && identity_unique
        },
    )
}

fn relevant_structural_checks_passed(
    checks: &[CoreVerifyCheck],
    item_check_start: usize,
    operation_check_start: usize,
    item_id: &str,
    operation_id: &str,
) -> bool {
    checks[item_check_start..operation_check_start]
        .iter()
        .filter(|check| check.scope_id == item_id)
        .chain(
            checks[operation_check_start..]
                .iter()
                .filter(|check| check.scope_id == operation_id),
        )
        .all(|check| check.status == "passed_v0")
}

fn verify_item<'artifact, 'authority>(
    item: &'artifact CoreLowerItem,
    classifications: Option<&'authority CanonicalMinimalAddTypeClassifications>,
    classification_batch_valid: bool,
    checks: &mut Vec<CoreVerifyCheck>,
    views: &mut Vec<VerifiedCanonicalMinimalAddType<'artifact, 'authority>>,
) {
    let item_check_start = checks.len();
    push_span_check(checks, "core_item", &item.id, &item.span);
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        !item.id.trim().is_empty(),
        "row_identity",
        "core item id is present",
    );
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        item.verification_status == "unverified_v0",
        "claim_honesty",
        "core-lower item remains unverified before core-verify",
    );
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        item.grammar_status == crate::core_body::CORE_BODY_GRAMMAR_STATUS,
        "body_grammar_consistency",
        "item keeps partial body grammar provenance",
    );
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        valid_item_status(item.status),
        "item_status_known",
        format!("item status `{}` is known to core-verify", item.status),
    );
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        item_status_consistent(item),
        "item_status_consistent",
        format!(
            "item lower status `{}` agrees with operations and blockers",
            item.status
        ),
    );

    for (expected_index, operation) in item.operations.iter().enumerate() {
        verify_operation(
            item,
            operation,
            expected_index,
            classifications,
            classification_batch_valid,
            item_check_start,
            checks,
            views,
        );
    }
    for blocker in &item.blockers {
        push_span_check(checks, "blocker", &item.id, &blocker.span);
        push_check(
            checks,
            "blocker",
            &item.id,
            Some(&blocker.span),
            valid_blocker_status(blocker.status),
            "blocker_status_known",
            format!(
                "blocker `{}` has known status `{}`",
                blocker.reason, blocker.status
            ),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_operation<'artifact, 'authority>(
    item: &'artifact CoreLowerItem,
    operation: &'artifact CoreLowerOperation,
    expected_index: usize,
    classifications: Option<&'authority CanonicalMinimalAddTypeClassifications>,
    classification_batch_valid: bool,
    item_check_start: usize,
    checks: &mut Vec<CoreVerifyCheck>,
    views: &mut Vec<VerifiedCanonicalMinimalAddType<'artifact, 'authority>>,
) {
    let operation_check_start = checks.len();
    push_span_check(checks, "operation", &operation.id, &operation.span);
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        !operation.id.trim().is_empty(),
        "row_identity",
        "operation id is present",
    );
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        operation.index == expected_index,
        "operation_index_consistent",
        format!("operation index is {}", operation.index),
    );
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        operation_status_consistent(operation),
        "operation_family_status_consistent",
        format!(
            "{} uses status {}",
            operation.core_operation, operation.status
        ),
    );
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        source_status_consistent(operation),
        "source_status_consistent",
        format!(
            "source kind {} with source status {} maps to {}",
            operation.source_kind, operation.source_status, operation.status
        ),
    );

    if operation.status == "blocked_operation_v0" {
        let detail = operation.reason.unwrap_or("missing_blocker_reason");
        push_check(
            checks,
            "operation",
            &operation.id,
            Some(&operation.span),
            operation.reason.is_some(),
            "blocked_operation_has_reason",
            format!("blocked operation reason: {detail}"),
        );
        push_check(
            checks,
            "operation",
            &operation.id,
            Some(&operation.span),
            has_matching_blocker(item, operation),
            "blocked_operation_has_matching_blocker",
            format!("blocked operation has matching blocker: {detail}"),
        );
    }

    match &operation.expression {
        Some(expression) => {
            let classification = classifications.and_then(|classifications| {
                operation
                    .canonical_minimal_add_classification_ordinal()
                    .and_then(|ordinal| classifications.record_at(ordinal))
            });
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                operation.source_status != "unsupported_v0",
                "expression_source_status_consistent",
                "unsupported source rows do not carry expression previews",
            );
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                valid_expression_status(expression.status),
                "expression_status_known",
                format!("expression status `{}` is known", expression.status),
            );
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                valid_expression_ast_status(expression.ast_status),
                "expression_ast_status_known",
                format!("expression AST status `{}` is known", expression.ast_status),
            );
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                expression.node_count > 0,
                "expression_ast_present",
                "expression AST root is present",
            );
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                valid_type_status(expression.type_status),
                "type_claim_honesty",
                format!(
                    "type status `{}` is provenance-limited",
                    expression.type_status
                ),
            );
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                matches!(
                    expression.effect_status,
                    core_expr::CORE_EXPRESSION_EFFECT_STATUS
                        | core_expr::CORE_PREDICATE_EFFECT_STATUS
                ),
                "effect_claim_honesty",
                "expression effects remain not checked",
            );
            match (&expression.structured, expression.structured_authority()) {
                (Some(structured), _) => {
                    verify_structured_expression(
                        operation,
                        expression,
                        structured,
                        classification,
                        checks,
                    );
                    verify_canonical_minimal_add_classification(
                        item,
                        operation,
                        classification,
                        classification_batch_valid,
                        relevant_structural_checks_passed(
                            checks,
                            item_check_start,
                            operation_check_start,
                            &item.id,
                            &operation.id,
                        ),
                        checks,
                        views,
                    );
                }
                (None, Some(_)) => {
                    push_check(
                        checks,
                        "structured_expression",
                        &operation.id,
                        Some(&operation.span),
                        false,
                        "structured_expression_projection_present",
                        "retained parser authority requires its structured projection",
                    );
                    verify_canonical_minimal_add_classification(
                        item,
                        operation,
                        classification,
                        classification_batch_valid,
                        false,
                        checks,
                        views,
                    );
                }
                (None, None) => {}
            }
        }
        None => {
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                operation.source_status == "unsupported_v0"
                    || blocked_operation_family(operation.core_operation)
                    || !operation_kind_requires_expression(operation.source_kind),
                "expression_absence_consistent",
                "operation expression absence is consistent with source kind",
            );
        }
    }
}

fn verify_structured_expression(
    operation: &CoreLowerOperation,
    expression: &CoreLowerExpression,
    structured: &CoreLowerStructuredExpression,
    classification: Option<&CanonicalMinimalAddTypeRecord>,
    checks: &mut Vec<CoreVerifyCheck>,
) {
    let scope = "structured_expression";
    let scope_id = &operation.id;
    let span = Some(&operation.span);
    push_check(
        checks,
        scope,
        scope_id,
        span,
        structured.provenance == "parser_owned_canonical_expression_v0",
        "structured_expression_parser_provenance",
        "structured expression provenance is parser-owned",
    );

    let authority = expression
        .structured_authority()
        .and_then(minimal_add_authority);
    push_check(
        checks,
        scope,
        scope_id,
        span,
        authority.is_some(),
        "structured_expression_parser_authority_present",
        "structured expression retains the bounded parser-owned add authority",
    );
    push_check(
        checks,
        scope,
        scope_id,
        span,
        !structured.parser_node_id.trim().is_empty()
            && structured
                .children
                .iter()
                .all(|child| !child.parser_node_id.trim().is_empty()),
        "structured_expression_identity_present",
        "root and child parser identities are present",
    );
    push_check(
        checks,
        scope,
        scope_id,
        span,
        structured.kind == "binary"
            && structured.operator == "add"
            && expression.operator == Some("add"),
        "structured_expression_binary_add_shape",
        "structured expression is the bounded binary add shape",
    );
    push_check(
        checks,
        scope,
        scope_id,
        span,
        structured.children.len() == 2,
        "structured_expression_child_count",
        format!(
            "structured binary expression has {} children",
            structured.children.len()
        ),
    );

    let child_order_valid = structured
        .children
        .iter()
        .enumerate()
        .all(|(expected, child)| child.index == expected);
    push_check(
        checks,
        scope,
        scope_id,
        span,
        child_order_valid,
        "structured_expression_child_order",
        "structured child indexes are exactly 0 then 1",
    );
    let child_roles_valid = structured.children.len() == 2
        && structured.children[0].role == "left"
        && structured.children[1].role == "right";
    push_check(
        checks,
        scope,
        scope_id,
        span,
        child_roles_valid,
        "structured_expression_child_roles",
        "structured child roles are exactly left then right",
    );
    push_check(
        checks,
        scope,
        scope_id,
        span,
        structured.children.len() == 2
            && structured
                .children
                .iter()
                .all(|child| child.kind == "identifier" && !child.identifier.is_empty()),
        "structured_expression_identifier_children",
        "structured children are named identifier nodes",
    );

    let mut identities = std::collections::BTreeSet::new();
    let identities_distinct = identities.insert(structured.parser_node_id.as_str())
        && structured
            .children
            .iter()
            .all(|child| identities.insert(child.parser_node_id.as_str()));
    push_check(
        checks,
        scope,
        scope_id,
        span,
        identities_distinct,
        "structured_expression_identity_distinct",
        "root and child parser identities are pairwise distinct",
    );

    let root_authority_matches = authority.as_ref().is_some_and(|authority| {
        structured.parser_node_id == authority.root.node_id.as_str()
            && structured.kind == "binary"
            && structured.operator == "add"
    });
    push_check(
        checks,
        scope,
        scope_id,
        span,
        root_authority_matches,
        "structured_expression_root_authority",
        "structured root identity, kind, and operator match retained parser authority",
    );

    let child_authority_matches = authority.as_ref().is_some_and(|authority| {
        structured.children.len() == 2
            && structured.children[0].index == 0
            && structured.children[0].role == "left"
            && structured.children[0].parser_node_id == authority.left.node_id.as_str()
            && structured.children[0].kind == "identifier"
            && structured.children[0].identifier == authority.left_name
            && structured.children[1].index == 1
            && structured.children[1].role == "right"
            && structured.children[1].parser_node_id == authority.right.node_id.as_str()
            && structured.children[1].kind == "identifier"
            && structured.children[1].identifier == authority.right_name
    });
    push_check(
        checks,
        scope,
        scope_id,
        span,
        child_authority_matches,
        "structured_expression_child_authority",
        "ordered child identities, roles, kinds, and spellings match retained parser authority",
    );

    let range_authority_matches = authority.as_ref().is_some_and(|authority| {
        source_range_matches_authority(&structured.source_range, &authority.root.range)
            && structured.children.len() == 2
            && source_range_matches_authority(
                &structured.children[0].source_range,
                &authority.left.range,
            )
            && source_range_matches_authority(
                &structured.children[1].source_range,
                &authority.right.range,
            )
    });
    push_check(
        checks,
        scope,
        scope_id,
        span,
        range_authority_matches,
        "structured_expression_range_authority",
        "root and child ranges match retained parser authority exactly",
    );

    let ranges_valid = source_range_is_sane(&structured.source_range)
        && structured.children.len() == 2
        && structured.children.iter().all(|child| {
            source_range_is_sane(&child.source_range)
                && source_range_contains(&structured.source_range, &child.source_range)
        })
        && source_range_precedes(
            &structured.children[0].source_range,
            &structured.children[1].source_range,
        );
    push_check(
        checks,
        scope,
        scope_id,
        span,
        ranges_valid,
        "structured_expression_source_ranges",
        "child ranges are sane, same-file, ordered, and contained by the root",
    );

    match classification.map(CanonicalMinimalAddTypeRecord::classification) {
        Some(CanonicalMinimalAddTypeClassification::Supported(_)) => push_check(
            checks,
            scope,
            scope_id,
            span,
            expression.type_status
                == core_expr::CORE_EXPRESSION_CHECKED_CANONICAL_MINIMAL_ADD_TYPE_STATUS
                && expression.type_text.as_deref() == Some("Int")
                && expression.type_source
                    == Some(core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_SOURCE),
            "structured_expression_outer_type_matches_canonical_minimal_add_classification",
            "structured expression outer type state matches canonical minimal-add classification",
        ),
        Some(CanonicalMinimalAddTypeClassification::IntegrityFailure(_)) => push_check(
            checks,
            scope,
            scope_id,
            span,
            expression.type_status
                == core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_UNAVAILABLE_STATUS
                && expression.type_text.is_none()
                && expression.type_source.is_none(),
            "structured_expression_outer_type_matches_canonical_minimal_add_classification",
            "structured expression outer type state matches canonical minimal-add classification",
        ),
        _ => push_check(
            checks,
            scope,
            scope_id,
            span,
            expression.type_status == core_expr::CORE_EXPRESSION_TYPE_STATUS
                && expression.type_text.is_none()
                && expression.type_source.is_none(),
            "structured_expression_outer_type_unchecked",
            "structured add preserves the authoritative unchecked outer type state",
        ),
    }
}

fn verify_canonical_minimal_add_classification<'artifact, 'authority>(
    item: &'artifact CoreLowerItem,
    operation: &'artifact CoreLowerOperation,
    classification: Option<&'authority CanonicalMinimalAddTypeRecord>,
    classification_batch_valid: bool,
    structural_checks_passed: bool,
    checks: &mut Vec<CoreVerifyCheck>,
    views: &mut Vec<VerifiedCanonicalMinimalAddType<'artifact, 'authority>>,
) {
    match classification.map(CanonicalMinimalAddTypeRecord::classification) {
        Some(CanonicalMinimalAddTypeClassification::Supported(authority)) => {
            if let Some(view) = verify_canonical_minimal_add_type(
                item,
                operation,
                authority,
                classification_batch_valid,
                structural_checks_passed,
                checks,
            ) {
                views.push(view);
            }
        }
        Some(CanonicalMinimalAddTypeClassification::IntegrityFailure(_)) => {
            push_check(
                checks,
                "operation",
                &operation.id,
                Some(&operation.span),
                false,
                "canonical_minimal_add_public_projection_matches_authority",
                "canonical minimal-add public projection does not match untouched producer authority",
            );
            push_check(
                checks,
                "operation",
                &operation.id,
                Some(&operation.span),
                false,
                "canonical_minimal_add_private_claim_matches_authority",
                "canonical minimal-add private claim does not match untouched producer authority",
            );
            push_check(
                checks,
                "operation",
                &operation.id,
                Some(&operation.span),
                false,
                "canonical_minimal_add_verified_view_issued",
                "canonical minimal-add verified view withheld after failed or missing authority check",
            );
        }
        Some(CanonicalMinimalAddTypeClassification::AuthenticatedOutOfScope(_))
        | Some(CanonicalMinimalAddTypeClassification::Noncanonical)
        | None => {}
    }
}

fn verify_canonical_minimal_add_type<'artifact, 'authority>(
    item: &'artifact CoreLowerItem,
    operation: &'artifact CoreLowerOperation,
    authority: &'authority CanonicalMinimalAddTypeAuthority,
    classification_batch_valid: bool,
    structural_checks_passed: bool,
    checks: &mut Vec<CoreVerifyCheck>,
) -> Option<VerifiedCanonicalMinimalAddType<'artifact, 'authority>> {
    let public_matches = classification_batch_valid
        && canonical_minimal_add_public_projection_matches(item, operation, authority);
    let private_matches = classification_batch_valid
        && operation
            .canonical_minimal_add_claim()
            .is_some_and(|claim| {
                Some(claim.root_node_id()) == authority.key().root_node_id()
                    && claim.matches(authority)
            });
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        public_matches,
        "canonical_minimal_add_public_projection_matches_authority",
        if public_matches {
            "canonical minimal-add public projection matches untouched producer authority"
        } else {
            "canonical minimal-add public projection does not match untouched producer authority"
        },
    );
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        private_matches,
        "canonical_minimal_add_private_claim_matches_authority",
        if private_matches {
            "canonical minimal-add private claim matches untouched producer authority"
        } else {
            "canonical minimal-add private claim does not match untouched producer authority"
        },
    );
    let issued = public_matches && private_matches && structural_checks_passed;
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        issued,
        "canonical_minimal_add_verified_view_issued",
        if issued {
            "canonical minimal-add verified view issued from successful checks"
        } else {
            "canonical minimal-add verified view withheld after failed or missing authority check"
        },
    );
    issued.then_some(VerifiedCanonicalMinimalAddType {
        item,
        operation,
        authority,
    })
}

fn canonical_minimal_add_public_projection_matches(
    item: &CoreLowerItem,
    operation: &CoreLowerOperation,
    authority: &CanonicalMinimalAddTypeAuthority,
) -> bool {
    let Some(expression) = operation.expression.as_ref() else {
        return false;
    };
    let Some(structured) = expression.structured.as_ref() else {
        return false;
    };
    let item_path = authority
        .item_path()
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".");
    if !authority.public_identity_matches(
        &item.semantic_identity,
        operation.index,
        &structured.parser_node_id,
    ) || !item
        .semantic_identity
        .ends_with(&format!("path-{item_path}"))
        || item.name != authority.task_name()
        || item.span != *authority.task_span()
        || operation.span != *authority.key().statement_span()
        || structured.children.len() != 2
        || !source_range_matches_authority(&structured.source_range, authority.root_range())
        || expression.type_status
            != core_expr::CORE_EXPRESSION_CHECKED_CANONICAL_MINIMAL_ADD_TYPE_STATUS
        || expression.type_text.as_deref() != Some(authority.expression_type())
        || expression.type_source
            != Some(core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_SOURCE)
    {
        return false;
    }
    structured
        .children
        .iter()
        .enumerate()
        .all(|(index, child)| {
            let Some((expected_index, role, node_id, range, identifier)) =
                authority.operand_identity(index)
            else {
                return false;
            };
            let Some((reference, definition, declaration, checked_type)) =
                authority.operand_projection(index)
            else {
                return false;
            };
            let Some(projected) = child.canonical_type.as_ref() else {
                return false;
            };
            child.index == expected_index
                && child.role == role
                && child.parser_node_id == node_id
                && source_range_matches_authority(&child.source_range, range)
                && child.identifier == identifier
                && projected.resolver_reference_id.as_deref() == Some(reference)
                && projected.resolved_definition_id.as_deref() == Some(definition)
                && projected.checked_declaration_id.as_deref() == Some(declaration)
                && projected.checked_type.as_deref() == Some(checked_type)
        })
}

#[allow(unexpected_cfgs)]
#[cfg(hum_compile_fail_verified_canonical_minimal_add_escape)]
fn verified_canonical_minimal_add_cannot_outlive_lower_artifact<'artifact, 'authority>(
    view: VerifiedCanonicalMinimalAddType<'artifact, 'authority>,
) -> VerifiedCanonicalMinimalAddType<'static, 'authority> {
    view
}

#[allow(unexpected_cfgs)]
#[cfg(hum_compile_fail_verified_canonical_minimal_add_escape)]
fn verified_canonical_minimal_add_cannot_outlive_producer_authority<'artifact, 'authority>(
    view: VerifiedCanonicalMinimalAddType<'artifact, 'authority>,
) -> VerifiedCanonicalMinimalAddType<'artifact, 'static> {
    view
}

#[allow(unexpected_cfgs)]
#[cfg(hum_compile_fail_verified_canonical_minimal_add_escape)]
fn verified_canonical_minimal_add_cannot_become_owned_static_authority<'artifact, 'authority>(
    view: VerifiedCanonicalMinimalAddType<'artifact, 'authority>,
) -> VerifiedCanonicalMinimalAddType<'static, 'static> {
    view
}

struct MinimalAddAuthority<'a> {
    root: &'a CanonicalExpression,
    left: &'a CanonicalExpression,
    right: &'a CanonicalExpression,
    left_name: &'a str,
    right_name: &'a str,
}

fn minimal_add_authority(expression: &CanonicalExpression) -> Option<MinimalAddAuthority<'_>> {
    let CanonicalExpressionKind::Binary {
        operator: ParsedBinaryOperator::Add,
        left,
        right,
    } = &expression.kind
    else {
        return None;
    };
    let CanonicalExpressionKind::Identifier(left_name) = &left.kind else {
        return None;
    };
    let CanonicalExpressionKind::Identifier(right_name) = &right.kind else {
        return None;
    };
    Some(MinimalAddAuthority {
        root: expression,
        left,
        right,
        left_name,
        right_name,
    })
}

fn source_range_is_sane(range: &CoreLowerSourceRange) -> bool {
    span_is_sane(&range.start)
        && range.byte_len > 0
        && range.start.column.checked_add(range.byte_len).is_some()
}

fn source_range_contains(parent: &CoreLowerSourceRange, child: &CoreLowerSourceRange) -> bool {
    let Some(parent_end) = parent.start.column.checked_add(parent.byte_len) else {
        return false;
    };
    let Some(child_end) = child.start.column.checked_add(child.byte_len) else {
        return false;
    };
    parent.start.file == child.start.file
        && parent.start.line == child.start.line
        && child.start.column >= parent.start.column
        && child_end <= parent_end
}

fn source_range_precedes(left: &CoreLowerSourceRange, right: &CoreLowerSourceRange) -> bool {
    left.start
        .column
        .checked_add(left.byte_len)
        .is_some_and(|left_end| left_end <= right.start.column)
}

fn source_range_matches_authority(
    projected: &CoreLowerSourceRange,
    authority: &ParsedSourceRange,
) -> bool {
    projected.start.file == authority.start.file.replace('\\', "/")
        && projected.start.line == authority.start.line
        && projected.start.column == authority.start.column
        && projected.byte_len == authority.byte_len
}

fn push_span_check(
    checks: &mut Vec<CoreVerifyCheck>,
    scope: &'static str,
    scope_id: &str,
    span: &Span,
) {
    push_check(
        checks,
        scope,
        scope_id,
        Some(span),
        span_is_sane(span),
        "source_span_sane",
        "source span has file, line, and column",
    );
}

fn push_check(
    checks: &mut Vec<CoreVerifyCheck>,
    scope: &'static str,
    scope_id: &str,
    span: Option<&Span>,
    passed: bool,
    rule: &'static str,
    detail: impl Into<String>,
) {
    checks.push(CoreVerifyCheck {
        id: format!("core-verify-check-{}", checks.len() + 1),
        scope,
        scope_id: scope_id.to_string(),
        span: span.cloned(),
        status: if passed { "passed_v0" } else { "failed_v0" },
        rule,
        detail: detail.into(),
    });
}

fn span_is_sane(span: &Span) -> bool {
    !span.file.trim().is_empty() && span.line > 0 && span.column > 0
}

fn valid_item_status(status: &str) -> bool {
    matches!(
        status,
        "lowered_unverified_core_v0"
            | "blocked_by_source_errors"
            | "blocked_by_resolver_errors"
            | "blocked_by_type_errors"
            | "blocked_before_core_execution"
            | "empty_body"
    )
}

fn item_status_consistent(item: &CoreLowerItem) -> bool {
    let blocked_operations = item
        .operations
        .iter()
        .filter(|operation| operation.status == "blocked_operation_v0")
        .count();
    match item.status {
        "lowered_unverified_core_v0" => item.blockers.is_empty() && blocked_operations == 0,
        "blocked_by_source_errors" => item
            .blockers
            .iter()
            .any(|blocker| blocker.status == "blocked_by_source_errors"),
        "blocked_by_resolver_errors" => item
            .blockers
            .iter()
            .any(|blocker| blocker.status == "blocked_by_resolver_errors"),
        "blocked_by_type_errors" => item
            .blockers
            .iter()
            .any(|blocker| blocker.status == "blocked_by_type_errors"),
        "blocked_before_core_execution" => !item.blockers.is_empty(),
        "empty_body" => item
            .blockers
            .iter()
            .any(|blocker| blocker.status == "empty_body"),
        _ => false,
    }
}

fn operation_status_consistent(operation: &CoreLowerOperation) -> bool {
    match operation.status {
        "lowered_unverified_operation_v0" => lowered_operation_family(operation.core_operation),
        "blocked_operation_v0" => blocked_operation_family(operation.core_operation),
        _ => false,
    }
}

fn source_status_consistent(operation: &CoreLowerOperation) -> bool {
    match operation.source_status {
        "recognized_v0" => true,
        "recognized_typed_executable_predicate_v2" => {
            operation.core_operation == "checked_contract_predicate_v2"
                && operation.status == "lowered_unverified_operation_v0"
        }
        "malformed_executable_predicate_v2" | "rejected_executable_predicate_semantics_v2" => {
            operation.core_operation == "blocked_contract_predicate_v2"
                && operation.status == "blocked_operation_v0"
        }
        "unsupported_v0" => {
            operation.status == "blocked_operation_v0" && operation.expression.is_none()
        }
        _ => false,
    }
}

fn lowered_operation_family(core_operation: &str) -> bool {
    matches!(
        core_operation,
        "return"
            | "fail"
            | "let_binding"
            | "mutable_binding"
            | "set_place"
            | "if_statement"
            | "while_loop"
            | "for_each"
            | "for_index"
            | "loop"
            | "block_close"
            | "checked_contract_predicate_v2"
    )
}

fn blocked_operation_family(core_operation: &str) -> bool {
    matches!(
        core_operation,
        "blocked_surface_statement"
            | "blocked_unsupported_try_expression"
            | "record_construction_field"
            | "contract_context"
            | "test_expectation"
            | "blocked_contract_predicate_v2"
    )
}

fn valid_blocker_status(status: &str) -> bool {
    matches!(
        status,
        "blocked_by_source_errors"
            | "blocked_by_resolver_errors"
            | "blocked_by_type_errors"
            | "blocked_operation_v0"
            | "empty_body"
    )
}

fn has_matching_blocker(item: &CoreLowerItem, operation: &CoreLowerOperation) -> bool {
    item.blockers.iter().any(|blocker| {
        blocker.status == "blocked_operation_v0"
            && blocker.span == operation.span
            && operation.reason == Some(blocker.reason)
    })
}

fn valid_expression_status(status: &str) -> bool {
    matches!(
        status,
        "atom_preview_v0"
            | "compound_preview_v0"
            | "contextual_preview_v0"
            | "surface_phrase_preview_v0"
            | core_expr::CORE_PREDICATE_EXPRESSION_STATUS
    )
}

fn valid_expression_ast_status(status: &str) -> bool {
    matches!(
        status,
        core_expr::CORE_EXPRESSION_AST_STATUS
            | core_expr::CORE_EXPRESSION_CONTEXTUAL_AST_STATUS
            | core_expr::CORE_EXPRESSION_SURFACE_AST_STATUS
            | core_expr::CORE_PREDICATE_AST_STATUS
    )
}

fn valid_type_status(status: &str) -> bool {
    matches!(
        status,
        core_expr::CORE_EXPRESSION_TYPE_STATUS
            | core_expr::CORE_EXPRESSION_CHECKED_TRIVIAL_RETURN_TYPE_STATUS
            | core_expr::CORE_EXPRESSION_CHECKED_TRIVIAL_RETURN_MISMATCH_STATUS
            | core_expr::CORE_EXPRESSION_CHECKED_CANONICAL_MINIMAL_ADD_TYPE_STATUS
            | core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_UNAVAILABLE_STATUS
            | core_expr::CORE_PREDICATE_TYPE_STATUS
    )
}

fn operation_kind_requires_expression(source_kind: &str) -> bool {
    matches!(
        source_kind,
        "return"
            | "fail"
            | "let_binding"
            | "mutable_binding"
            | "set_place"
            | "if_header"
            | "while_header"
            | "for_each_header"
            | "for_index_header"
            | "record_field_initializer"
            | "test_expectation"
    )
}

impl CoreVerifyReport {
    fn verification_status(&self) -> &'static str {
        if self.failed_checks() == 0 {
            CORE_VERIFY_STATUS
        } else {
            CORE_VERIFY_FAILED_STATUS
        }
    }

    fn passed_checks(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| check.status == "passed_v0")
            .count()
    }

    fn failed_checks(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| check.status == "failed_v0")
            .count()
    }

    fn operations(&self) -> usize {
        self.lower
            .core_items
            .iter()
            .map(|item| item.operations.len())
            .sum()
    }

    fn verified_items(&self) -> usize {
        if self.failed_checks() == 0 {
            self.lower.core_items.len()
        } else {
            0
        }
    }

    fn verified_operations(&self) -> usize {
        if self.failed_checks() == 0 {
            self.operations()
        } else {
            0
        }
    }

    fn item_status(&self, item: &CoreLowerItem) -> &'static str {
        if self.failed_checks() == 0 || !self.item_has_failed_check(item) {
            "verified_core_artifact_item_v0"
        } else {
            "core_artifact_item_verification_failed_v0"
        }
    }

    fn item_has_failed_check(&self, item: &CoreLowerItem) -> bool {
        self.checks.iter().any(|check| {
            check.status == "failed_v0"
                && (check.scope_id == item.id
                    || item
                        .operations
                        .iter()
                        .any(|operation| operation.id == check.scope_id))
        })
    }
}

fn push_summary(out: &mut String, report: &CoreVerifyReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"core_items\": {}, \"verified_items\": {}, \"lower_blocked_items\": {}, \"operations\": {}, \"verified_operations\": {}, \"lower_blocked_operations\": {}, \"checks\": {}, \"passed_checks\": {}, \"failed_checks\": {}, \"execution_ready\": 0, \"ir_ready\": 0, \"errors\": {}, \"warnings\": {}, \"resolver_errors\": {}, \"type_errors\": {}, \"preview_blocked_statements\": {}",
        report.lower.files,
        report.lower.items,
        report.lower.tasks,
        report.lower.tests,
        report.lower.core_items.len(),
        report.verified_items(),
        report.lower.blocked_items(),
        report.operations(),
        report.verified_operations(),
        report.lower.blocked_operations(),
        report.checks.len(),
        report.passed_checks(),
        report.failed_checks(),
        report.lower.errors,
        report.lower.warnings,
        report.lower.resolver_errors,
        report.lower.type_errors,
        report.lower.preview_blocked_statements
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_core_lower_summary(
    out: &mut String,
    report: &CoreVerifyReport,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"core_lower\": {\n");
    push_string_field(
        out,
        indent + 2,
        "schema",
        core_lower::CORE_LOWER_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "status",
        core_lower::CORE_LOWER_STATUS,
        true,
    );
    push_usize_field(out, indent + 2, "files", report.lower.files, true);
    push_usize_field(out, indent + 2, "items", report.lower.items, true);
    push_usize_field(out, indent + 2, "tasks", report.lower.tasks, true);
    push_usize_field(out, indent + 2, "tests", report.lower.tests, true);
    push_usize_field(
        out,
        indent + 2,
        "core_items",
        report.lower.core_items.len(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "lowered_items",
        report.lower.lowered_items(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocked_items",
        report.lower.blocked_items(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "lowered_operations",
        report.lower.lowered_operations(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocked_operations",
        report.lower.blocked_operations(),
        true,
    );
    push_usize_field(out, indent + 2, "execution_ready", 0, true);
    push_usize_field(out, indent + 2, "ir_ready", 0, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_items(out: &mut String, report: &CoreVerifyReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"core_items\": [\n");
    for (index, item) in report.lower.core_items.iter().enumerate() {
        push_item(
            out,
            report,
            item,
            indent + 2,
            index + 1 < report.lower.core_items.len(),
        );
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_item(
    out: &mut String,
    report: &CoreVerifyReport,
    item: &CoreLowerItem,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &item.id, true);
    push_string_field(out, indent + 2, "kind", item.kind, true);
    push_string_field(out, indent + 2, "name", &item.name, true);
    push_span_field(out, indent + 2, "source_span", &item.span, true);
    push_string_field(out, indent + 2, "lower_status", item.status, true);
    push_string_field(
        out,
        indent + 2,
        "verification_status",
        report.item_status(item),
        true,
    );
    push_usize_field(out, indent + 2, "operations", item.operations.len(), true);
    push_usize_field(out, indent + 2, "blockers", item.blockers.len(), false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_checks(out: &mut String, checks: &[CoreVerifyCheck], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"checks\": [\n");
    for (index, check) in checks.iter().enumerate() {
        push_check_json(out, check, indent + 2, index + 1 < checks.len());
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_check_json(out: &mut String, check: &CoreVerifyCheck, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &check.id, true);
    push_string_field(out, indent + 2, "scope", check.scope, true);
    push_string_field(out, indent + 2, "scope_id", &check.scope_id, true);
    push_optional_span_field(out, indent + 2, "source_span", check.span.as_ref(), true);
    push_string_field(out, indent + 2, "status", check.status, true);
    push_string_field(out, indent + 2, "rule", check.rule, true);
    push_string_field(out, indent + 2, "detail", &check.detail, false);
    push_indent(out, indent);
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
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    if let Some(span) = span {
        push_span(out, span);
    } else {
        out.push_str("null");
    }
    push_comma_newline(out, comma);
}

fn push_span_field(out: &mut String, indent: usize, key: &str, span: &Span, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_span(out, span);
    push_comma_newline(out, comma);
}

fn push_span(out: &mut String, span: &Span) {
    out.push('{');
    out.push_str("\"file\": ");
    push_json_string(out, &span.file);
    out.push_str(&format!(
        ", \"line\": {}, \"column\": {}",
        span.line, span.column
    ));
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

fn push_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    out.push_str(&value.to_string());
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

    use super::{
        build_report, canonical_minimal_add_batch_matches, core_verify_json, core_verify_text,
        validate_diagnostic_projection_from_source, verify_canonical_minimal_add_lowering,
        verify_lower_report, verify_lower_report_with_classifications,
    };
    use crate::type_check::CanonicalMinimalAddTypeClassification;

    #[test]
    fn typed_minimal_add_verifier_rejects_authority_corruption() {
        fn owner_from(path: &str, source: &str) -> crate::core_lower::CanonicalMinimalAddLowering {
            let parsed = parse_source(path, source);
            let diagnostics = parsed.diagnostics;
            let program = Program {
                files: vec![parsed.file],
            };
            crate::core_lower::build_canonical_minimal_add_lowering(&program, &diagnostics)
        }

        fn owner() -> crate::core_lower::CanonicalMinimalAddLowering {
            owner_from(
                "minimal-add-verifier.hum",
                "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\n",
            )
        }

        fn results(
            owner: &crate::core_lower::CanonicalMinimalAddLowering,
        ) -> (Vec<(&'static str, &'static str)>, usize) {
            let (checks, views) = verify_canonical_minimal_add_lowering(owner);
            (
                checks
                    .into_iter()
                    .filter(|check| check.rule.starts_with("canonical_minimal_add_"))
                    .map(|check| (check.rule, check.status))
                    .collect(),
                views.len(),
            )
        }

        fn assert_clean_claimless(owner: &crate::core_lower::CanonicalMinimalAddLowering) {
            let operation_ordinals = owner
                .report()
                .core_items
                .iter()
                .flat_map(|item| &item.operations)
                .map(|operation| operation.canonical_minimal_add_classification_ordinal())
                .collect::<Vec<_>>();
            let classification_kinds = owner
                .classifications()
                .records()
                .iter()
                .map(|record| match record.classification() {
                    CanonicalMinimalAddTypeClassification::Supported(_) => "supported",
                    CanonicalMinimalAddTypeClassification::AuthenticatedOutOfScope(_) => {
                        "out_of_scope"
                    }
                    CanonicalMinimalAddTypeClassification::IntegrityFailure(_) => "integrity",
                    CanonicalMinimalAddTypeClassification::Noncanonical => "noncanonical",
                })
                .collect::<Vec<_>>();
            assert!(
                canonical_minimal_add_batch_matches(owner.report(), owner.classifications()),
                "valid claimless batch mismatch: ordinals={operation_ordinals:?} classifications={classification_kinds:?}",
            );
            assert!(
                owner
                    .report()
                    .core_items
                    .iter()
                    .flat_map(|item| &item.operations)
                    .all(|operation| operation.canonical_minimal_add_claim().is_none()),
                "claimless classifications must not fabricate candidate claims",
            );
            let (checks, views) = verify_canonical_minimal_add_lowering(owner);
            assert!(views.is_empty());
            let failed = checks
                .iter()
                .filter(|check| check.status == "failed_v0")
                .map(|check| (check.rule, check.detail.as_str()))
                .collect::<Vec<_>>();
            assert!(failed.is_empty(), "valid claimless failures: {failed:?}");
            assert!(
                checks
                    .iter()
                    .all(|check| !check.rule.starts_with("canonical_minimal_add_")),
                "a valid claimless batch must receive no canonical authority rows",
            );
        }

        fn assert_claimless_batch_failure(owner: crate::core_lower::CanonicalMinimalAddLowering) {
            assert!(
                owner
                    .report()
                    .core_items
                    .iter()
                    .flat_map(|item| &item.operations)
                    .all(|operation| operation.canonical_minimal_add_claim().is_none()),
                "batch corruption must remain independent of candidate claims",
            );
            let (checks, views) = verify_canonical_minimal_add_lowering(&owner);
            assert!(views.is_empty(), "an invalid batch must issue no view");
            assert!(checks.iter().any(|check| {
                check.status == "failed_v0" && check.rule.starts_with("canonical_minimal_add_")
            }));
            drop(views);
            let report = super::CoreVerifyReport {
                lower: owner,
                checks,
            };
            assert!(
                report.failed_checks() > 0,
                "the existing CLI error predicate must reject the malformed batch",
            );
            assert_eq!(
                report.verification_status(),
                super::CORE_VERIFY_FAILED_STATUS,
            );
        }

        let clean = owner();
        assert_eq!(
            results(&clean),
            (
                vec![
                    (
                        "canonical_minimal_add_public_projection_matches_authority",
                        "passed_v0",
                    ),
                    (
                        "canonical_minimal_add_private_claim_matches_authority",
                        "passed_v0",
                    ),
                    ("canonical_minimal_add_verified_view_issued", "passed_v0"),
                ],
                1,
            )
        );

        let mut public = owner();
        public.report_mut_for_test().core_items[0].operations[0]
            .expression
            .as_mut()
            .unwrap()
            .structured
            .as_mut()
            .unwrap()
            .children[0]
            .canonical_type
            .as_mut()
            .unwrap()
            .checked_type = Some("UInt".to_string());
        assert_eq!(results(&public).1, 0);
        assert!(results(&public).0.contains(&(
            "canonical_minimal_add_public_projection_matches_authority",
            "failed_v0"
        )));

        let mut private = owner();
        private.report_mut_for_test().core_items[0].operations[0]
            .canonical_minimal_add_claim_mut_for_test()
            .unwrap()
            .corrupt_checked_type_for_test("UInt");
        assert_eq!(results(&private).1, 0);
        assert!(results(&private).0.contains(&(
            "canonical_minimal_add_private_claim_matches_authority",
            "failed_v0"
        )));

        let mut coherent = owner();
        let operation = &mut coherent.report_mut_for_test().core_items[0].operations[0];
        operation
            .expression
            .as_mut()
            .unwrap()
            .structured
            .as_mut()
            .unwrap()
            .children[0]
            .canonical_type
            .as_mut()
            .unwrap()
            .checked_type = Some("UInt".to_string());
        operation
            .canonical_minimal_add_claim_mut_for_test()
            .unwrap()
            .corrupt_checked_type_for_test("UInt");
        let (checks, views) = results(&coherent);
        assert_eq!(views, 0);
        assert!(checks.iter().all(|(_, status)| *status == "failed_v0"));

        let mut missing = owner();
        missing.report_mut_for_test().core_items[0].operations[0]
            .remove_canonical_minimal_add_claim_for_test();
        let (checks, views) = results(&missing);
        assert_eq!(views, 0);
        assert!(checks.contains(&(
            "canonical_minimal_add_private_claim_matches_authority",
            "failed_v0"
        )));

        let mut partial_public = owner();
        partial_public.report_mut_for_test().core_items[0].operations[0]
            .expression
            .as_mut()
            .unwrap()
            .structured
            .as_mut()
            .unwrap()
            .children[1]
            .canonical_type
            .as_mut()
            .unwrap()
            .resolved_definition_id = None;
        assert_eq!(results(&partial_public).1, 0);

        let mut relocated_identity = owner();
        relocated_identity.report_mut_for_test().core_items[0].operations[0]
            .expression
            .as_mut()
            .unwrap()
            .structured
            .as_mut()
            .unwrap()
            .parser_node_id
            .push_str("-foreign");
        let (checks, views) = results(&relocated_identity);
        assert_eq!(views, 0);
        assert_eq!(
            checks.len(),
            3,
            "dispatch must not trust the public root id"
        );
        assert_eq!(
            checks,
            vec![
                (
                    "canonical_minimal_add_public_projection_matches_authority",
                    "failed_v0",
                ),
                (
                    "canonical_minimal_add_private_claim_matches_authority",
                    "passed_v0",
                ),
                ("canonical_minimal_add_verified_view_issued", "failed_v0"),
            ]
        );

        let candidate = owner();
        let parsed = parse_source(
            "minimal-add-verifier.hum",
            "task foreign(x: Int, y: Int) -> Int {\n  does:\n    return x + y\n}\n",
        );
        let foreign_diagnostics = parsed.diagnostics;
        let foreign_program = Program {
            files: vec![parsed.file],
        };
        let foreign = crate::core_lower::build_canonical_minimal_add_lowering(
            &foreign_program,
            &foreign_diagnostics,
        );
        let mut foreign_views = Vec::new();
        let foreign_checks = verify_lower_report_with_classifications(
            candidate.report(),
            Some(foreign.classifications()),
            &mut foreign_views,
        );
        assert!(foreign_views.is_empty());
        let foreign_authority_checks = foreign_checks
            .iter()
            .filter(|check| check.rule.starts_with("canonical_minimal_add_"))
            .collect::<Vec<_>>();
        assert_eq!(foreign_authority_checks.len(), 3);
        assert!(
            foreign_authority_checks
                .iter()
                .all(|check| check.status == "failed_v0")
        );

        let mut duplicate = owner();
        duplicate.append_classifications_for_test(owner());
        let (checks, views) = results(&duplicate);
        assert_eq!(
            views, 0,
            "duplicate classification batch must issue no view"
        );
        assert!(checks.iter().all(|(_, status)| *status == "failed_v0"));

        let mut absent = owner();
        absent.remove_classification_for_test(0);
        let (checks, views) = results(&absent);
        assert_eq!(views, 0, "missing classification must issue no view");
        assert!(checks.iter().all(|(_, status)| *status == "failed_v0"));

        let mut duplicate_integrity = owner();
        duplicate_integrity.force_first_integrity_failure_for_test();
        let mut second_integrity = owner();
        second_integrity.force_first_integrity_failure_for_test();
        duplicate_integrity.append_classifications_for_test(second_integrity);
        let (checks, views) = results(&duplicate_integrity);
        assert_eq!(views, 0, "duplicate integrity rows must issue no view");
        assert!(checks.iter().all(|(_, status)| *status == "failed_v0"));

        let out_of_scope_source =
            "task add(a: UInt, b: UInt) -> UInt {\n  does:\n    return a + b\n}\n";
        let out_of_scope = owner_from("claimless-out-of-scope.hum", out_of_scope_source);
        assert_clean_claimless(&out_of_scope);

        let mut duplicate_out_of_scope =
            owner_from("claimless-out-of-scope.hum", out_of_scope_source);
        duplicate_out_of_scope.append_classifications_for_test(owner_from(
            "claimless-out-of-scope.hum",
            out_of_scope_source,
        ));
        assert_claimless_batch_failure(duplicate_out_of_scope);

        let mut missing_out_of_scope =
            owner_from("claimless-out-of-scope.hum", out_of_scope_source);
        missing_out_of_scope.remove_classification_for_test(0);
        assert_claimless_batch_failure(missing_out_of_scope);

        let noncanonical_source =
            "task identity(value: Int) -> Int {\n  does:\n    return value\n}\n";
        let noncanonical = owner_from("claimless-noncanonical.hum", noncanonical_source);
        assert_clean_claimless(&noncanonical);

        let mut duplicate_noncanonical =
            owner_from("claimless-noncanonical.hum", noncanonical_source);
        duplicate_noncanonical.append_classifications_for_test(owner_from(
            "claimless-noncanonical.hum",
            noncanonical_source,
        ));
        assert_claimless_batch_failure(duplicate_noncanonical);

        let mut missing_noncanonical =
            owner_from("claimless-noncanonical.hum", noncanonical_source);
        missing_noncanonical.remove_classification_for_test(0);
        assert_claimless_batch_failure(missing_noncanonical);

        let mut extra_foreign_claimless =
            owner_from("claimless-out-of-scope.hum", out_of_scope_source);
        extra_foreign_claimless.append_classifications_for_test(owner_from(
            "foreign-claimless-noncanonical.hum",
            noncanonical_source,
        ));
        assert_claimless_batch_failure(extra_foreign_claimless);

        let mut reordered = owner_from(
            "two-minimal-adds.hum",
            "task first(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\n\ntask second(x: Int, y: Int) -> Int {\n  does:\n    return x + y\n}\n",
        );
        reordered.swap_classifications_for_test(0, 1);
        let (_, views) = results(&reordered);
        assert_eq!(views, 0, "reordered batch must issue no view");

        let same_visible_foreign = owner_from(
            "minimal-add-verifier.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b   \n}\n",
        );
        let mut same_visible_views = Vec::new();
        let same_visible_checks = verify_lower_report_with_classifications(
            candidate.report(),
            Some(same_visible_foreign.classifications()),
            &mut same_visible_views,
        );
        assert!(same_visible_views.is_empty());
        let same_visible_authority_checks = same_visible_checks
            .iter()
            .filter(|check| check.rule.starts_with("canonical_minimal_add_"))
            .collect::<Vec<_>>();
        assert!(
            same_visible_authority_checks
                .iter()
                .any(|check| check.status == "failed_v0")
        );
        assert!(same_visible_authority_checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_verified_view_issued"
                && check.status == "failed_v0"
        }));

        for corruption in [
            crate::core_lower::CoreLowerTreeCorruption::MissingStructuredTree,
            crate::core_lower::CoreLowerTreeCorruption::WrongOperationKind,
            crate::core_lower::CoreLowerTreeCorruption::WrongOperator,
            crate::core_lower::CoreLowerTreeCorruption::ForeignRootIdentity(
                "foreign-root".to_string(),
            ),
            crate::core_lower::CoreLowerTreeCorruption::WrongChildMetadata,
            crate::core_lower::CoreLowerTreeCorruption::MissingChild,
            crate::core_lower::CoreLowerTreeCorruption::ReorderChildren,
            crate::core_lower::CoreLowerTreeCorruption::DuplicateChildIdentity,
            crate::core_lower::CoreLowerTreeCorruption::IncorrectIdentifierSpelling(
                "foreign".to_string(),
            ),
            crate::core_lower::CoreLowerTreeCorruption::CoherentRangeRelocation {
                file: "foreign/minimal-add.hum".to_string(),
                line_offset: 1,
                column_offset: 1,
            },
            crate::core_lower::CoreLowerTreeCorruption::OverflowSizedRange,
            crate::core_lower::CoreLowerTreeCorruption::StructuralOverclaim,
        ] {
            let mut structurally_corrupt = owner();
            crate::core_lower::corrupt_first_structured_expression_for_test(
                structurally_corrupt.report_mut_for_test(),
                corruption,
            )
            .expect("structural corruption seam");
            let (checks, views) = results(&structurally_corrupt);
            assert_eq!(views, 0, "failed structure must issue no view");
            assert!(checks.contains(&("canonical_minimal_add_verified_view_issued", "failed_v0",)));
        }
    }

    #[test]
    fn verifier_rejects_minimal_add_tree_corruption() {
        fn minimal_add() -> (Program, Vec<crate::diagnostic::Diagnostic>) {
            let parsed = parse_source(
                "examples/core/minimal_add.hum",
                include_str!("../examples/core/minimal_add.hum"),
            );
            (
                Program {
                    files: vec![parsed.file],
                },
                parsed.diagnostics,
            )
        }

        fn failed_rule(report: &crate::core_lower::CoreLowerReport, rule: &str) -> bool {
            verify_lower_report(report)
                .iter()
                .any(|check| check.status == "failed_v0" && check.rule == rule)
        }

        let (program, diagnostics) = minimal_add();
        let clean = build_report(&program, &diagnostics);
        assert_eq!(clean.failed_checks(), 0, "clean artifact must verify");
        assert!(clean.checks.iter().any(|check| {
            check.rule
                == "structured_expression_outer_type_matches_canonical_minimal_add_classification"
                && check.status == "passed_v0"
        }), "the production verifier must consume the typed structured tree");

        let mut reordered = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut reordered,
            crate::core_lower::CoreLowerTreeCorruption::ReorderChildren,
        )
        .expect("reorder seam");
        assert!(failed_rule(&reordered, "structured_expression_child_order"));

        let mut duplicate = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut duplicate,
            crate::core_lower::CoreLowerTreeCorruption::DuplicateChildIdentity,
        )
        .expect("duplicate seam");
        assert!(failed_rule(
            &duplicate,
            "structured_expression_identity_distinct"
        ));

        let foreign_source = r#"task foreign_add(x: Int, y: Int) -> Int {
  does:
    return x
    return x + y
}
"#;
        let foreign_parsed = parse_source("foreign/minimal-add.hum", foreign_source);
        let foreign_diagnostics = foreign_parsed.diagnostics;
        let foreign_program = Program {
            files: vec![foreign_parsed.file],
        };
        let foreign =
            crate::core_lower::build_core_lower_report(&foreign_program, &foreign_diagnostics);
        let foreign_projection = foreign.core_items[0]
            .operations
            .iter()
            .filter_map(|operation| operation.expression.as_ref())
            .find_map(|expression| expression.structured.as_ref())
            .expect("foreign parser-owned tree")
            .clone();
        let foreign_identity = foreign_projection.children[1].parser_node_id.clone();
        let foreign_range = foreign_projection.children[1].source_range.clone();

        let mut substituted = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut substituted,
            crate::core_lower::CoreLowerTreeCorruption::ForeignChildIdentity(foreign_identity),
        )
        .expect("foreign seam");
        assert!(failed_rule(
            &substituted,
            "structured_expression_child_authority"
        ));

        let mut foreign_ranged = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut foreign_ranged,
            crate::core_lower::CoreLowerTreeCorruption::ForeignChildRange(foreign_range),
        )
        .expect("foreign range seam");
        assert!(failed_rule(
            &foreign_ranged,
            "structured_expression_range_authority"
        ));

        let mut misspelled = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut misspelled,
            crate::core_lower::CoreLowerTreeCorruption::IncorrectIdentifierSpelling(
                "same_shape_wrong_name".to_string(),
            ),
        )
        .expect("identifier spelling seam");
        assert!(failed_rule(
            &misspelled,
            "structured_expression_child_authority"
        ));

        let mut foreign_tree = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut foreign_tree,
            crate::core_lower::CoreLowerTreeCorruption::CoherentForeignProjection(
                foreign_projection,
            ),
        )
        .expect("coherent foreign projection seam");
        assert!(failed_rule(
            &foreign_tree,
            "structured_expression_root_authority"
        ));

        let mut relocated = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut relocated,
            crate::core_lower::CoreLowerTreeCorruption::CoherentRangeRelocation {
                file: "relocated/minimal-add.hum".to_string(),
                line_offset: 10,
                column_offset: 40,
            },
        )
        .expect("coherent range relocation seam");
        assert!(failed_rule(
            &relocated,
            "structured_expression_range_authority"
        ));

        let mut overflowing = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut overflowing,
            crate::core_lower::CoreLowerTreeCorruption::OverflowSizedRange,
        )
        .expect("overflow-sized range seam");
        assert!(failed_rule(
            &overflowing,
            "structured_expression_source_ranges"
        ));

        let mut overclaimed = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut overclaimed,
            crate::core_lower::CoreLowerTreeCorruption::StructuralOverclaim,
        )
        .expect("structural overclaim seam");
        assert!(failed_rule(
            &overclaimed,
            "structured_expression_binary_add_shape"
        ));
    }

    #[test]
    fn core_transport_rejects_projection_regenerated_or_corrupted_downstream() {
        let source = include_str!(
            "../fixtures/diagnostics/session_ap_same_line_independent_causes_fail.hum"
        );
        let parsed = parse_source(
            "fixtures/diagnostics/session_ap_same_line_independent_causes_fail.hum",
            source,
        );
        let checked = crate::check::check_file_with_occurrences(&parsed);
        let mut source_occurrences = parsed.diagnostic_occurrences.clone();
        source_occurrences
            .extend_owned(&checked.diagnostic_occurrences)
            .expect("source authority");
        let mut diagnostics = parsed.diagnostics;
        diagnostics.extend(checked.diagnostics);
        let program = Program {
            files: vec![parsed.file],
        };
        let authoritative = crate::core_preview::diagnostic_occurrence_set_from_source(
            &program,
            &diagnostics,
            &source_occurrences,
        );
        let projected =
            validate_diagnostic_projection_from_source(&program, &diagnostics, &source_occurrences)
                .expect("canonical Core projection");
        assert_eq!(projected, authoritative);

        let lower = crate::core_lower::build_core_lower_report_from_preview(
            &program,
            &diagnostics,
            &authoritative,
        )
        .expect("lower projection transport");
        let canonical_lower_projection = lower.diagnostic_projection;
        canonical_lower_projection
            .validate_against("core_lower", &authoritative)
            .expect("lower validates against separate preview authority");

        let mut missing_reference = canonical_lower_projection.clone();
        missing_reference.prior_blockers_mut_for_test().pop();
        assert!(
            missing_reference
                .validate_against("core_lower", &authoritative)
                .is_err()
        );

        let mut substituted_reference = canonical_lower_projection;
        substituted_reference.prior_blockers_mut_for_test()[0]
            .semantic_origin
            .push_str(":substituted");
        assert!(
            substituted_reference
                .validate_against("core_lower", &authoritative)
                .is_err()
        );

        let mut missing = projected.clone();
        missing.remove_first_for_test();
        assert!(
            crate::diagnostic::DiagnosticOccurrenceSet::validate_projection_from(
                &authoritative,
                &missing,
            )
            .is_err()
        );
        let mut internal_corruption = projected;
        internal_corruption.corrupt_first_diagnostic_for_test();
        assert!(
            crate::diagnostic::DiagnosticOccurrenceSet::validate_projection_from(
                &authoritative,
                &internal_corruption,
            )
            .is_err()
        );
    }

    #[test]
    fn json_verifies_tiny_core_artifact_without_execution_claims() {
        let source = r#"task add(a: Int, b: Int) -> Int {
  does:
    return a + b
}
"#;
        let parsed = parse_source("add.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = core_verify_json(&program, &parsed.diagnostics);

        assert!(json.contains("\"schema\": \"hum.core_verify.v0\""));
        assert!(json.contains("\"core_lower_schema\": \"hum.core_lower.v0\""));
        assert!(
            json.contains("\"verification_status\": \"verified_non_executing_core_artifact_v0\"")
        );
        assert!(json.contains("\"mode\": \"non_executing_artifact_invariant_check_v0\""));
        assert!(json.contains("\"rule\": \"source_span_sane\""));
        assert!(json.contains("\"rule\": \"operation_family_status_consistent\""));
        assert!(json.contains("\"rule\": \"claim_honesty\""));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"ir_ready\": 0"));
        assert!(json.contains("\"failed_checks\": 0"));
        assert!(json.contains("\"no Hum IR emission\""));
        assert!(json.contains("\"no memory-safety proof\""));
        assert!(json.contains("\"no optimization claim\""));
    }

    #[test]
    fn text_and_json_verify_blocked_lowering_rows_as_honest_blockers() {
        let source = r#"type WorkItem {
  id: Text
}

store work: list WorkItem {
  why:
    keep work
}

task remember(item: WorkItem) -> WorkItem {
  changes:
    work

  does:
    save item in work
    return item
}
"#;
        let parsed = parse_source("blocked.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let text = core_verify_text(&program, &parsed.diagnostics);
        let json = core_verify_json(&program, &parsed.diagnostics);

        assert!(text.contains("Hum Core verify (hum.core_verify.v0)"));
        assert!(text.contains("verification_failures: none"));
        assert!(
            json.contains("\"verification_status\": \"verified_non_executing_core_artifact_v0\"")
        );
        assert!(json.contains("\"lower_blocked_items\": 1"));
        assert!(json.contains("\"blocked_operations\": 1"));
        assert!(json.contains("surface_save_requires_store_lowering"));
        assert!(json.contains("blocked_operation_has_matching_blocker"));
        assert!(json.contains("\"failed_checks\": 0"));
    }
}
