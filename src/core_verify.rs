use crate::ast::{
    CanonicalExpression, CanonicalExpressionKind, Item, ParsedBinaryOperator, ParsedSourceRange,
    Program,
};
use crate::callable;
use crate::core_body::{self, CanonicalBodyStatement};
use crate::core_contract;
use crate::core_expr;
use crate::core_lower::{
    self, CanonicalMinimalAddOperationIdentity, CoreLowerCanonicalMinimalAddType,
    CoreLowerExpression, CoreLowerItem, CoreLowerOperation, CoreLowerReport, CoreLowerSourceRange,
    CoreLowerStructuredExpression,
};
use crate::core_preview;
use crate::diagnostic::{Diagnostic, DiagnosticOccurrenceSet, Span};
use crate::ir_contract;
use crate::node_id;
use crate::predicate;
use crate::resolve;
use crate::type_check;
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
    lower: CoreLowerReport,
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

pub(crate) struct CanonicalMinimalAddVerification<'report> {
    program: &'report Program,
    report: &'report CoreVerifyReport,
}

pub(crate) enum CanonicalMinimalAddVerificationOutcome<'verified> {
    Supported(VerifiedCanonicalMinimalAddType<'verified>),
    AuthenticatedOutOfScope,
    LegacyCompatibleAdditive,
    UnsupportedTargetLike,
    IntegrityFailure,
    NonTarget,
}

pub(crate) struct VerifiedCanonicalMinimalAddType<'verified> {
    authority: &'verified crate::type_check::CanonicalMinimalAddTypeAuthority,
    operation_id: &'verified str,
    _report: &'verified CoreVerifyReport,
}

impl VerifiedCanonicalMinimalAddType<'_> {
    pub(crate) fn actual_type(&self) -> &'static str {
        self.authority.produced_type()
    }

    pub(crate) fn provenance(&self) -> &'static str {
        "verified_canonical_minimal_add_type_v0"
    }

    pub(crate) fn operation_id(&self) -> &str {
        self.operation_id
    }
}

impl CanonicalMinimalAddVerification<'_> {
    pub(crate) fn readiness_summary(&self) -> CoreVerifyReadinessSummary {
        readiness_summary(self.report)
    }

    pub(crate) fn diagnostic_occurrence_set(&self) -> DiagnosticOccurrenceSet {
        self.report.lower.diagnostic_occurrences.clone()
    }
}

pub(crate) fn with_canonical_minimal_add_verification<R>(
    program: &Program,
    diagnostics: &[Diagnostic],
    consume: impl for<'report> FnOnce(CanonicalMinimalAddVerification<'report>) -> R,
) -> R {
    let report = build_report(program, diagnostics);
    consume(CanonicalMinimalAddVerification {
        program,
        report: &report,
    })
}

pub(crate) fn with_canonical_minimal_add_operation<'report, 'call, R>(
    verification: &'call CanonicalMinimalAddVerification<'report>,
    item: &Item,
    statement_index: usize,
    statement: &CanonicalBodyStatement,
    consume: impl FnOnce(CanonicalMinimalAddVerificationOutcome<'call>) -> R,
) -> R
where
    'report: 'call,
{
    let Some(root) = statement.canonical_expression() else {
        return consume(CanonicalMinimalAddVerificationOutcome::NonTarget);
    };
    if !matches!(
        root.kind,
        CanonicalExpressionKind::Binary {
            operator: ParsedBinaryOperator::Add,
            ..
        }
    ) || item.kind() != "task"
        || statement.statement().kind != "return"
    {
        return consume(CanonicalMinimalAddVerificationOutcome::NonTarget);
    }
    let Item::Task(task) = item else {
        return consume(CanonicalMinimalAddVerificationOutcome::IntegrityFailure);
    };
    let Ok(task_signature) = verification
        .program
        .authenticate_canonical_task_signature(task)
    else {
        return consume(CanonicalMinimalAddVerificationOutcome::IntegrityFailure);
    };
    let Some(expected) = core_lower::canonical_minimal_add_operation_identity(
        &task_signature,
        statement_index,
        statement,
    ) else {
        return consume(CanonicalMinimalAddVerificationOutcome::NonTarget);
    };
    let lower_item_id = node_id::span(
        "core-item",
        item.span(),
        &format!("{} {}", item.kind(), item.name()),
    );
    let Some(lower_item) = verification
        .report
        .lower
        .core_items
        .iter()
        .find(|candidate| candidate.id == lower_item_id)
    else {
        return consume(CanonicalMinimalAddVerificationOutcome::IntegrityFailure);
    };
    let DirectCanonicalMinimalAddLookup::One(operation) =
        lookup_direct_canonical_minimal_add_operation(lower_item, &expected)
    else {
        return consume(CanonicalMinimalAddVerificationOutcome::IntegrityFailure);
    };
    let target_checks_pass = required_canonical_minimal_add_checks_pass(
        &verification.report.checks,
        lower_item,
        operation,
    );
    let eligibility = canonical_minimal_add_supported_eligibility(
        verification.program,
        lower_item,
        operation,
        target_checks_pass,
    );
    let view_row_passes =
        canonical_minimal_add_view_row_passes(&verification.report.checks, operation);
    let outcome = match operation.canonical_minimal_add_type() {
        CoreLowerCanonicalMinimalAddType::Supported { .. }
            if eligibility.view_issued && view_row_passes =>
        {
            let authority = eligibility
                .authority
                .expect("supported eligibility retains operation-owned authority");
            CanonicalMinimalAddVerificationOutcome::Supported(VerifiedCanonicalMinimalAddType {
                authority,
                operation_id: &operation.id,
                _report: verification.report,
            })
        }
        CoreLowerCanonicalMinimalAddType::Supported { .. }
        | CoreLowerCanonicalMinimalAddType::IntegrityFailure => {
            CanonicalMinimalAddVerificationOutcome::IntegrityFailure
        }
        CoreLowerCanonicalMinimalAddType::AuthenticatedOutOfScope => {
            CanonicalMinimalAddVerificationOutcome::AuthenticatedOutOfScope
        }
        CoreLowerCanonicalMinimalAddType::LegacyCompatibleAdditive => {
            CanonicalMinimalAddVerificationOutcome::LegacyCompatibleAdditive
        }
        CoreLowerCanonicalMinimalAddType::UnsupportedTargetLike => {
            CanonicalMinimalAddVerificationOutcome::UnsupportedTargetLike
        }
        CoreLowerCanonicalMinimalAddType::Noncanonical => {
            CanonicalMinimalAddVerificationOutcome::IntegrityFailure
        }
    };
    consume(outcome)
}

#[allow(unexpected_cfgs, unused_imports)]
mod verified_canonical_minimal_add_direct_escape_compile_proof {
    use super::*;

    #[cfg(hum_compile_fail_verified_canonical_minimal_add_direct_escape)]
    fn verified_canonical_minimal_add_artifact_escape_must_not_compile<'a>(
        program: &'a Program,
        diagnostics: &'a [Diagnostic],
        item: &'a Item,
        statement_index: usize,
        statement: &'a CanonicalBodyStatement,
    ) -> VerifiedCanonicalMinimalAddType<'a> {
        with_canonical_minimal_add_verification(program, diagnostics, |verification| {
            with_canonical_minimal_add_operation(
                &verification,
                item,
                statement_index,
                statement,
                |outcome| match outcome {
                    CanonicalMinimalAddVerificationOutcome::Supported(view) => view,
                    _ => panic!("compile-fail probe requires a supported view"),
                },
            )
        })
    }

    #[cfg(hum_compile_fail_verified_canonical_minimal_add_direct_escape)]
    fn verified_canonical_minimal_add_report_escape_must_not_compile<'a>(
        program: &'a Program,
        diagnostics: &'a [Diagnostic],
    ) -> CanonicalMinimalAddVerification<'a> {
        with_canonical_minimal_add_verification(program, diagnostics, |verification| verification)
    }

    #[cfg(hum_compile_fail_verified_canonical_minimal_add_direct_escape)]
    fn verified_canonical_minimal_add_static_escape_must_not_compile(
        program: &Program,
        diagnostics: &[Diagnostic],
        item: &Item,
        statement_index: usize,
        statement: &CanonicalBodyStatement,
    ) -> VerifiedCanonicalMinimalAddType<'static> {
        with_canonical_minimal_add_verification(program, diagnostics, |verification| {
            with_canonical_minimal_add_operation(
                &verification,
                item,
                statement_index,
                statement,
                |outcome| match outcome {
                    CanonicalMinimalAddVerificationOutcome::Supported(view) => view,
                    _ => panic!("compile-fail probe requires a supported view"),
                },
            )
        })
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
    readiness_summary(&report)
}

fn readiness_summary(report: &CoreVerifyReport) -> CoreVerifyReadinessSummary {
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
    let lower =
        core_lower::build_core_lower_report_from_preview(program, diagnostics, &preview_authority)
            .expect("Core lower must preserve one sealed preview occurrence projection");
    lower
        .diagnostic_projection
        .validate_against("core_lower", &preview_authority)
        .expect("Core verify must compare lower projection with preview authority");
    let mut checks = verify_lower_report(program, &lower);
    let callable_failures = callable::analyze_program(program).verify();
    if callable_failures.is_empty() {
        push_check(
            &mut checks,
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
                &mut checks,
                "callable_semantic_spine",
                "session-al-callable-facts",
                None,
                false,
                failure,
                format!("callable fact verification failed: {failure}"),
            );
        }
    }
    CoreVerifyReport { lower, checks }
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

fn verify_lower_report(program: &Program, lower: &CoreLowerReport) -> Vec<CoreVerifyCheck> {
    let mut checks = Vec::new();
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
        verify_item(program, item, &mut checks);
    }

    checks
}

fn verify_item(program: &Program, item: &CoreLowerItem, checks: &mut Vec<CoreVerifyCheck>) {
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
    match item.task_signature_verdict() {
        crate::core_lower::CoreLowerTaskSignatureVerdict::Failed => push_check(
            checks,
            "core_item",
            &item.id,
            Some(&item.span),
            false,
            "task_signature_authority_matches_parser_owner",
            "task signature does not match retained parser authority",
        ),
        crate::core_lower::CoreLowerTaskSignatureVerdict::NotATask
        | crate::core_lower::CoreLowerTaskSignatureVerdict::Passed => push_check(
            checks,
            "core_item",
            &item.id,
            Some(&item.span),
            item.grammar_status == crate::core_body::CORE_BODY_GRAMMAR_STATUS,
            "body_grammar_consistency",
            "item keeps partial body grammar provenance",
        ),
    }
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

    verify_direct_canonical_minimal_add_operations(program, item, checks);
    for (expected_index, operation) in item.operations.iter().enumerate() {
        verify_operation(program, item, operation, expected_index, checks);
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

enum DirectCanonicalMinimalAddLookup<'a> {
    NoMatch,
    One(&'a CoreLowerOperation),
    Ambiguous,
}

fn verify_direct_canonical_minimal_add_operations(
    program: &Program,
    lower_item: &CoreLowerItem,
    checks: &mut Vec<CoreVerifyCheck>,
) {
    let Some(item) = find_program_item_for_lower(program, lower_item) else {
        return;
    };
    let Item::Task(task) = item else {
        return;
    };
    let Ok(task_signature) = program.authenticate_canonical_task_signature(task) else {
        return;
    };
    let Some(does) = task.section("does") else {
        return;
    };
    let body = core_body::analyze_does_section_for_lowering(
        program
            .canonical_core_expectation(item, does)
            .expect("live verifier item must retain parser authority"),
    );
    for (statement_index, statement) in body.statements.iter().enumerate() {
        let Some(expected) = core_lower::canonical_minimal_add_operation_identity(
            &task_signature,
            statement_index,
            statement,
        ) else {
            continue;
        };
        if !matches!(
            lookup_direct_canonical_minimal_add_operation(lower_item, &expected),
            DirectCanonicalMinimalAddLookup::One(_)
        ) {
            push_check(
                checks,
                "core_item",
                &lower_item.id,
                Some(&statement.statement().span),
                false,
                "canonical_minimal_add_direct_operation_identity_unique",
                "one Core operation matches the parser-owned additive task-return identity",
            );
        }
    }
}

fn lookup_direct_canonical_minimal_add_operation<'a>(
    item: &'a CoreLowerItem,
    expected: &CanonicalMinimalAddOperationIdentity,
) -> DirectCanonicalMinimalAddLookup<'a> {
    let mut state = DirectCanonicalMinimalAddLookup::NoMatch;
    for operation in &item.operations {
        if !operation
            .canonical_minimal_add_identity()
            .is_some_and(|candidate| candidate.matches(expected))
        {
            continue;
        }
        state = match state {
            DirectCanonicalMinimalAddLookup::NoMatch => {
                DirectCanonicalMinimalAddLookup::One(operation)
            }
            DirectCanonicalMinimalAddLookup::One(_)
            | DirectCanonicalMinimalAddLookup::Ambiguous => {
                DirectCanonicalMinimalAddLookup::Ambiguous
            }
        };
    }
    state
}

struct CanonicalMinimalAddSupportedEligibility<'a> {
    authority: Option<&'a crate::type_check::CanonicalMinimalAddTypeAuthority>,
    state_consistent: bool,
    public_matches: bool,
    claim_matches: bool,
    view_issued: bool,
}

fn canonical_minimal_add_supported_eligibility<'a>(
    program: &Program,
    lower_item: &CoreLowerItem,
    operation: &'a CoreLowerOperation,
    required_checks_pass: bool,
) -> CanonicalMinimalAddSupportedEligibility<'a> {
    let CoreLowerCanonicalMinimalAddType::Supported { authority, claim } =
        operation.canonical_minimal_add_type()
    else {
        return CanonicalMinimalAddSupportedEligibility {
            authority: None,
            state_consistent: false,
            public_matches: false,
            claim_matches: false,
            view_issued: false,
        };
    };
    let program_operation_matches = (|| {
        let item = find_program_item_for_lower(program, lower_item)?;
        let Item::Task(task) = item else {
            return None;
        };
        let task_signature = program.authenticate_canonical_task_signature(task).ok()?;
        let does = task.section("does")?;
        let body = core_body::analyze_does_section_for_lowering(
            program.canonical_core_expectation(item, does).ok()?,
        );
        let candidate_identity = operation.canonical_minimal_add_identity()?;
        let matching_statements = body
            .statements
            .iter()
            .enumerate()
            .filter(|(statement_index, statement)| {
                core_lower::canonical_minimal_add_operation_identity(
                    &task_signature,
                    *statement_index,
                    statement,
                )
                .is_some_and(|expected| candidate_identity.matches(&expected))
            })
            .collect::<Vec<_>>();
        let [(statement_index, statement)] = matching_statements.as_slice() else {
            return None;
        };
        let expected = core_lower::canonical_minimal_add_operation_identity(
            &task_signature,
            *statement_index,
            statement,
        )?;
        let DirectCanonicalMinimalAddLookup::One(matched_operation) =
            lookup_direct_canonical_minimal_add_operation(lower_item, &expected)
        else {
            return None;
        };
        (std::ptr::eq(matched_operation, operation)
            && authority.matches_operation(&task_signature, item, *statement_index, statement))
        .then_some(())
    })()
    .is_some();
    let state_consistent = program_operation_matches
        && authority.semantic_facts_are_complete()
        && authority.matches_claim(
            authority.statement_index(),
            authority.root_node_id(),
            authority.produced_type(),
        );
    let public_matches = operation.expression.as_ref().is_some_and(|expression| {
        authority.matches_public_projection(
            expression.type_status,
            expression.type_text.as_deref(),
            expression.type_source,
        )
    });
    let claim_matches = claim.matches_authority(authority);
    CanonicalMinimalAddSupportedEligibility {
        authority: Some(authority),
        state_consistent,
        public_matches,
        claim_matches,
        view_issued: required_checks_pass && state_consistent && public_matches && claim_matches,
    }
}

fn canonical_minimal_add_type_rule(rule: &str) -> bool {
    matches!(
        rule,
        "canonical_minimal_add_type_state_consistent"
            | "canonical_minimal_add_public_projection_matches_authority"
            | "canonical_minimal_add_private_claim_matches_authority"
            | "canonical_minimal_add_verified_view_issued"
    )
}

fn required_canonical_minimal_add_checks_pass(
    checks: &[CoreVerifyCheck],
    lower_item: &CoreLowerItem,
    operation: &CoreLowerOperation,
) -> bool {
    checks.iter().all(|check| {
        let relevant_item_check = check.scope_id == lower_item.id
            && check.scope == "core_item"
            && (check.rule != "canonical_minimal_add_direct_operation_identity_unique"
                || check.span.as_ref() == Some(&operation.span));
        let relevant = relevant_item_check
            || (check.scope_id == operation.id
                && matches!(
                    check.scope,
                    "operation" | "operation_expression" | "structured_expression"
                ));
        !relevant || canonical_minimal_add_type_rule(check.rule) || check.status == "passed_v0"
    })
}

fn canonical_minimal_add_view_row_passes(
    checks: &[CoreVerifyCheck],
    operation: &CoreLowerOperation,
) -> bool {
    let matching = checks
        .iter()
        .filter(|check| {
            check.scope_id == operation.id
                && check.rule == "canonical_minimal_add_verified_view_issued"
        })
        .collect::<Vec<_>>();
    matches!(matching.as_slice(), [check] if check.status == "passed_v0")
}

fn find_program_item_for_lower<'a>(
    program: &'a Program,
    lower_item: &CoreLowerItem,
) -> Option<&'a Item> {
    fn collect<'a>(items: &'a [Item], lower_item: &CoreLowerItem, matches: &mut Vec<&'a Item>) {
        for item in items {
            let expected_id = node_id::span(
                "core-item",
                item.span(),
                &format!("{} {}", item.kind(), item.name()),
            );
            if expected_id == lower_item.id {
                matches.push(item);
            }
            if let Item::App(app) = item {
                collect(&app.items, lower_item, matches);
            }
        }
    }
    let mut matches = Vec::new();
    for file in &program.files {
        collect(&file.items, lower_item, &mut matches);
    }
    let [item] = matches.as_slice() else {
        return None;
    };
    Some(*item)
}

fn verify_operation(
    program: &Program,
    item: &CoreLowerItem,
    operation: &CoreLowerOperation,
    expected_index: usize,
    checks: &mut Vec<CoreVerifyCheck>,
) {
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
            if matches!(
                operation.canonical_minimal_add_type(),
                CoreLowerCanonicalMinimalAddType::UnsupportedTargetLike
            ) {
                push_check(
                    checks,
                    "operation_expression",
                    &operation.id,
                    Some(&operation.span),
                    false,
                    "canonical_minimal_add_unsupported_target_like_rejected",
                    "unsupported additive task-return shape has no canonical type authority",
                );
            } else {
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
            }
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
                        program, item, operation, expression, structured, checks,
                    );
                }
                (None, Some(_)) => push_check(
                    checks,
                    "structured_expression",
                    &operation.id,
                    Some(&operation.span),
                    false,
                    "structured_expression_projection_present",
                    "retained parser authority requires its structured projection",
                ),
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
    program: &Program,
    item: &CoreLowerItem,
    operation: &CoreLowerOperation,
    expression: &CoreLowerExpression,
    structured: &CoreLowerStructuredExpression,
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

    match operation.canonical_minimal_add_type() {
        CoreLowerCanonicalMinimalAddType::Supported { authority, claim } => {
            let prior_required_checks_pass =
                required_canonical_minimal_add_checks_pass(checks, item, operation);
            let eligibility = canonical_minimal_add_supported_eligibility(
                program,
                item,
                operation,
                prior_required_checks_pass,
            );
            debug_assert!(std::ptr::eq(
                eligibility.authority.expect("supported authority"),
                authority.as_ref(),
            ));
            debug_assert_eq!(
                eligibility.public_matches,
                authority.matches_public_projection(
                    expression.type_status,
                    expression.type_text.as_deref(),
                    expression.type_source,
                ),
            );
            debug_assert_eq!(
                eligibility.claim_matches,
                claim.matches_authority(authority)
            );
            push_canonical_minimal_add_type_checks(
                checks,
                operation,
                eligibility.state_consistent,
                eligibility.public_matches,
                eligibility.claim_matches,
                eligibility.view_issued,
            );
        }
        CoreLowerCanonicalMinimalAddType::IntegrityFailure => {
            let state_consistent = expression.type_status
                == core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_UNAVAILABLE_STATUS
                && expression.type_text.is_none()
                && expression.type_source.is_none();
            push_canonical_minimal_add_type_checks(
                checks,
                operation,
                state_consistent,
                false,
                false,
                false,
            );
        }
        CoreLowerCanonicalMinimalAddType::AuthenticatedOutOfScope
        | CoreLowerCanonicalMinimalAddType::LegacyCompatibleAdditive
        | CoreLowerCanonicalMinimalAddType::UnsupportedTargetLike
        | CoreLowerCanonicalMinimalAddType::Noncanonical => {
            let outer_type_unchecked = expression.type_status
                == core_expr::CORE_EXPRESSION_TYPE_STATUS
                && expression.type_text.is_none()
                && expression.type_source.is_none();
            push_check(
                checks,
                scope,
                scope_id,
                span,
                outer_type_unchecked,
                "structured_expression_outer_type_unchecked",
                "structured add preserves the authoritative unchecked outer type state",
            );
        }
    }
}

fn push_canonical_minimal_add_type_checks(
    checks: &mut Vec<CoreVerifyCheck>,
    operation: &CoreLowerOperation,
    state_consistent: bool,
    public_matches: bool,
    claim_matches: bool,
    view_issued: bool,
) {
    let rows = [
        (
            state_consistent,
            "canonical_minimal_add_type_state_consistent",
            "canonical minimal-add type state is complete for its closed disposition",
        ),
        (
            public_matches,
            "canonical_minimal_add_public_projection_matches_authority",
            "canonical minimal-add public projection matches untouched producer authority",
        ),
        (
            claim_matches,
            "canonical_minimal_add_private_claim_matches_authority",
            "canonical minimal-add private claim matches untouched producer authority",
        ),
        (
            view_issued,
            "canonical_minimal_add_verified_view_issued",
            "verified canonical minimal-add type access is gated by every required check",
        ),
    ];
    for (passed, rule, detail) in rows {
        push_check(
            checks,
            "structured_expression",
            &operation.id,
            Some(&operation.span),
            passed,
            rule,
            detail,
        );
    }
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
    use crate::ast::{CanonicalTaskSignatureCorruption, ParamPermission, Program};
    use crate::parser::parse_source;

    use super::{
        CanonicalMinimalAddVerification, CanonicalMinimalAddVerificationOutcome, CoreVerifyReport,
        build_report, core_verify_json, core_verify_text,
        validate_diagnostic_projection_from_source, verify_lower_report,
        with_canonical_minimal_add_operation,
    };

    #[test]
    fn canonical_minimal_add_type_verification_withholds_invalid_view() {
        fn parsed_program(
            path: &str,
            source: &str,
        ) -> (Program, Vec<crate::diagnostic::Diagnostic>) {
            let parsed = parse_source(path, source);
            (
                Program {
                    files: vec![parsed.file],
                },
                parsed.diagnostics,
            )
        }

        fn outcome_name(program: &Program, report: &CoreVerifyReport) -> &'static str {
            let item = &program.files[0].items[0];
            let crate::ast::Item::Task(task) = item else {
                panic!("task")
            };
            let body = crate::core_body::analyze_does_section_for_lowering(
                program
                    .canonical_core_expectation(item, task.section("does").expect("does"))
                    .expect("expectation"),
            );
            let verification = CanonicalMinimalAddVerification { program, report };
            with_canonical_minimal_add_operation(
                &verification,
                item,
                0,
                &body.statements[0],
                |outcome| match outcome {
                    CanonicalMinimalAddVerificationOutcome::Supported(_) => "Supported",
                    CanonicalMinimalAddVerificationOutcome::AuthenticatedOutOfScope => {
                        "AuthenticatedOutOfScope"
                    }
                    CanonicalMinimalAddVerificationOutcome::LegacyCompatibleAdditive => {
                        "LegacyCompatibleAdditive"
                    }
                    CanonicalMinimalAddVerificationOutcome::UnsupportedTargetLike => {
                        "UnsupportedTargetLike"
                    }
                    CanonicalMinimalAddVerificationOutcome::IntegrityFailure => "IntegrityFailure",
                    CanonicalMinimalAddVerificationOutcome::NonTarget => "NonTarget",
                },
            )
        }

        fn outcome_is_supported(program: &Program, report: &CoreVerifyReport) -> bool {
            outcome_name(program, report) == "Supported"
        }

        fn report_from_lower(
            program: &Program,
            lower: crate::core_lower::CoreLowerReport,
        ) -> CoreVerifyReport {
            let checks = verify_lower_report(program, &lower);
            CoreVerifyReport { lower, checks }
        }

        fn assert_no_view(program: &Program, report: &CoreVerifyReport, case: &str) {
            assert_eq!(
                outcome_name(program, report),
                "IntegrityFailure",
                "{case}: a corrupted supported target must not downgrade"
            );
            assert!(
                !report.checks.iter().any(|check| {
                    check.rule == "canonical_minimal_add_verified_view_issued"
                        && check.status == "passed_v0"
                }),
                "{case}: a zero-view outcome cannot publish a passed view row"
            );
        }

        let source =
            "task add(left: Int, right: Int) -> Int {\n  does:\n    return left + right\n}\n";
        let (program, diagnostics) = parsed_program("verify/direct-minimal-add.hum", source);
        let clean = build_report(&program, &diagnostics);
        assert!(outcome_is_supported(&program, &clean));
        for rule in [
            "canonical_minimal_add_type_state_consistent",
            "canonical_minimal_add_public_projection_matches_authority",
            "canonical_minimal_add_private_claim_matches_authority",
            "canonical_minimal_add_verified_view_issued",
        ] {
            assert!(
                clean
                    .checks
                    .iter()
                    .any(|check| check.rule == rule && check.status == "passed_v0")
            );
        }

        let (mixed_program, mixed_diagnostics) = parsed_program(
            "verify/target-local-view.hum",
            "task add(left: Int, right: Int) -> Int {\n  does:\n    return left + right\n}\n\ntask unsupported(value: UInt) -> UInt {\n  does:\n    return 1 + value\n}\n",
        );
        let mixed = build_report(&mixed_program, &mixed_diagnostics);
        assert!(
            mixed.failed_checks() > 0,
            "unrelated target fails the report"
        );
        assert_eq!(
            outcome_name(&mixed_program, &mixed),
            "Supported",
            "view issuance remains target-local despite an unrelated report failure"
        );
        assert!(mixed.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_verified_view_issued"
                && check.status == "passed_v0"
        }));

        let mut public = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        let expression = public.core_items[0].operations[0]
            .expression
            .as_mut()
            .expect("expression");
        expression.type_text = Some("UInt".to_string());
        let public = report_from_lower(&program, public);
        assert_no_view(&program, &public, "public-only corruption");
        assert!(public.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_public_projection_matches_authority"
                && check.status == "failed_v0"
        }));

        let mut claim = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_canonical_minimal_add_claim_for_test(&mut claim)
            .expect("claim corruption");
        let claim = report_from_lower(&program, claim);
        assert_no_view(&program, &claim, "claim-only corruption");
        assert!(claim.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_private_claim_matches_authority"
                && check.status == "failed_v0"
        }));

        let mut coherent = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_canonical_minimal_add_public_and_claim_for_test(
            &mut coherent,
        )
        .expect("coherent public/private corruption");
        let coherent = report_from_lower(&program, coherent);
        assert_no_view(&program, &coherent, "coherent public/private corruption");
        assert!(coherent.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_public_projection_matches_authority"
                && check.status == "failed_v0"
        }));
        assert!(coherent.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_private_claim_matches_authority"
                && check.status == "failed_v0"
        }));

        let (foreign_program, foreign_diagnostics) =
            parsed_program("verify/foreign-direct-minimal-add.hum", source);
        let mut foreign_state_target =
            crate::core_lower::build_core_lower_report(&program, &diagnostics);
        let mut foreign_state =
            crate::core_lower::build_core_lower_report(&foreign_program, &foreign_diagnostics);
        crate::core_lower::substitute_first_canonical_minimal_add_state_for_test(
            &mut foreign_state_target,
            &mut foreign_state,
        )
        .expect("foreign operation-owned authority substitution");
        let foreign_state = report_from_lower(&program, foreign_state_target);
        assert_no_view(
            &program,
            &foreign_state,
            "public projection plus claim plus foreign authority",
        );
        assert!(foreign_state.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_type_state_consistent"
                && check.status == "failed_v0"
        }));
        assert!(foreign_state.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_verified_view_issued"
                && check.status == "failed_v0"
        }));

        let mut structural = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut structural,
            crate::core_lower::CoreLowerTreeCorruption::ReorderChildren,
        )
        .expect("structural corruption");
        let structural = report_from_lower(&program, structural);
        assert_no_view(
            &program,
            &structural,
            "structural corruption with valid type facts",
        );
        assert!(structural.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_verified_view_issued"
                && check.status == "failed_v0"
        }));

        let mut missing = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        missing.core_items[0].operations.clear();
        let missing = report_from_lower(&program, missing);
        assert_no_view(&program, &missing, "sole expected operation deletion");
        assert!(missing.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_direct_operation_identity_unique"
                && check.status == "failed_v0"
        }));

        let mut duplicate = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        let mut second = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        duplicate.core_items[0]
            .operations
            .push(second.core_items[0].operations.remove(0));
        let duplicate = report_from_lower(&program, duplicate);
        assert_no_view(&program, &duplicate, "duplicate operation identity");
        assert!(duplicate.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_direct_operation_identity_unique"
                && check.status == "failed_v0"
        }));

        let mut foreign_identity_target =
            crate::core_lower::build_core_lower_report(&program, &diagnostics);
        let mut foreign_identity =
            crate::core_lower::build_core_lower_report(&foreign_program, &foreign_diagnostics);
        crate::core_lower::substitute_first_canonical_minimal_add_identity_for_test(
            &mut foreign_identity_target,
            &mut foreign_identity,
        )
        .expect("foreign identity substitution");
        let foreign_identity = report_from_lower(&program, foreign_identity_target);
        assert_no_view(&program, &foreign_identity, "foreign operation identity");
        assert!(foreign_identity.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_direct_operation_identity_unique"
                && check.status == "failed_v0"
        }));

        let mut same_visible_id =
            crate::core_lower::build_core_lower_report(&program, &diagnostics);
        let expected_public_id = same_visible_id.core_items[0].operations[0].id.clone();
        let mut foreign_operation =
            crate::core_lower::build_core_lower_report(&foreign_program, &foreign_diagnostics)
                .core_items[0]
                .operations
                .remove(0);
        foreign_operation.id = expected_public_id;
        same_visible_id.core_items[0].operations[0] = foreign_operation;
        let same_visible_id = report_from_lower(&program, same_visible_id);
        assert_no_view(
            &program,
            &same_visible_id,
            "same-visible-ID operation from a foreign revision",
        );
        assert!(same_visible_id.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_direct_operation_identity_unique"
                && check.status == "failed_v0"
        }));

        let (noncanonical_program, noncanonical_diagnostics) = parsed_program(
            "verify/noncanonical-operation.hum",
            "task noop() -> Int {\n  does:\n    return 1\n}\n",
        );
        let mut final_deleted = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        final_deleted.core_items[0].operations.clear();
        let mut unrelated = crate::core_lower::build_core_lower_report(
            &noncanonical_program,
            &noncanonical_diagnostics,
        );
        final_deleted.core_items[0]
            .operations
            .push(unrelated.core_items[0].operations.remove(0));
        let final_deleted = report_from_lower(&program, final_deleted);
        assert_no_view(
            &program,
            &final_deleted,
            "final expected operation deleted while unrelated operation remains",
        );
        assert!(final_deleted.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_direct_operation_identity_unique"
                && check.status == "failed_v0"
        }));

        let (unsupported_program, unsupported_diagnostics) = parsed_program(
            "verify/unsupported-add.hum",
            "task add(value: UInt) -> UInt {\n  does:\n    return 1 + value\n}\n",
        );
        let unsupported = build_report(&unsupported_program, &unsupported_diagnostics);
        assert!(unsupported.checks.iter().any(|check| {
            check.rule == "canonical_minimal_add_unsupported_target_like_rejected"
                && check.status == "failed_v0"
        }));

        let (uint_program, uint_diagnostics) = parsed_program(
            "verify/uint-add.hum",
            "task add(left: UInt, right: UInt) -> UInt {\n  does:\n    return left + right\n}\n",
        );
        let uint = build_report(&uint_program, &uint_diagnostics);
        assert_eq!(uint.failed_checks(), 0);
        assert!(uint.checks.iter().any(|check| {
            check.rule == "structured_expression_outer_type_unchecked"
                && check.status == "passed_v0"
        }));
    }

    #[test]
    fn task_signature_authority_is_load_bearing() {
        const SOURCE: &str =
            "task add(left: Int, right: Int) -> Int {\n  does:\n    return left + right\n}\n";

        fn test_program() -> (Program, Vec<crate::diagnostic::Diagnostic>) {
            let parsed = parse_source("authority/load-bearing.hum", SOURCE);
            (
                Program {
                    files: vec![parsed.file],
                },
                parsed.diagnostics,
            )
        }

        fn signature_check(checks: &[super::CoreVerifyCheck]) -> Option<&super::CoreVerifyCheck> {
            checks.iter().find(|check| {
                check.rule == "body_grammar_consistency"
                    || check.rule == "task_signature_authority_matches_parser_owner"
            })
        }

        let (program, diagnostics) = test_program();
        let clean_lower = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        let clean_checks = verify_lower_report(&program, &clean_lower);
        let clean = signature_check(&clean_checks).expect("existing item check");
        assert_eq!(clean.status, "passed_v0");
        assert_eq!(clean.rule, "body_grammar_consistency");
        assert_eq!(clean.detail, "item keeps partial body grammar provenance");
        let clean_count = clean_checks.len();

        for corruption in [
            CanonicalTaskSignatureCorruption::Missing,
            CanonicalTaskSignatureCorruption::ResultRangeRelocated,
            CanonicalTaskSignatureCorruption::ForeignTask,
            CanonicalTaskSignatureCorruption::ForeignRevision,
            CanonicalTaskSignatureCorruption::Overflow,
            CanonicalTaskSignatureCorruption::Underflow,
        ] {
            let (mut corrupted_program, diagnostics) = test_program();
            corrupted_program.files[0].items[0].corrupt_canonical_task_signature(corruption);
            let lower =
                crate::core_lower::build_core_lower_report(&corrupted_program, &diagnostics);
            let checks = verify_lower_report(&corrupted_program, &lower);
            assert_eq!(checks.len(), clean_count, "{corruption:?}");
            let rejected = signature_check(&checks).expect("replacement item check");
            assert_eq!(rejected.status, "failed_v0", "{corruption:?}");
            assert_eq!(
                rejected.rule,
                "task_signature_authority_matches_parser_owner"
            );
            assert_eq!(
                rejected.detail,
                "task signature does not match retained parser authority"
            );
            assert_eq!(rejected.scope, "core_item");
            assert!(rejected.span.is_some());
        }

        let mut public = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        let task = &mut public.core_items[0];
        task.name = "foreign".to_string();
        task.params[0].name = "other_left".to_string();
        task.params[0].permission = ParamPermission::Change;
        task.params[0].ty = "UInt".to_string();
        task.params[1].name = "other_right".to_string();
        task.params[1].ty = "UInt".to_string();
        task.result = Some("UInt".to_string());
        let checks = verify_lower_report(&program, &public);
        let rejected = signature_check(&checks).expect("public substitution check");
        assert_eq!(rejected.status, "failed_v0");
        assert_eq!(
            rejected.rule,
            "task_signature_authority_matches_parser_owner"
        );

        let mut precedence = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        precedence.core_items[0].grammar_status = "corrupted_body_grammar_v0";
        precedence.core_items[0].name = "foreign".to_string();
        let checks = verify_lower_report(&program, &precedence);
        let rejected = signature_check(&checks).expect("precedence check");
        assert_eq!(
            rejected.rule, "task_signature_authority_matches_parser_owner",
            "signature rejection owns the existing ordinal"
        );
        assert_eq!(checks.len(), clean_count);
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

        fn failed_rule(
            program: &Program,
            report: &crate::core_lower::CoreLowerReport,
            rule: &str,
        ) -> bool {
            verify_lower_report(program, report)
                .iter()
                .any(|check| check.status == "failed_v0" && check.rule == rule)
        }

        let (program, diagnostics) = minimal_add();
        let clean = build_report(&program, &diagnostics);
        assert_eq!(clean.failed_checks(), 0, "clean artifact must verify");
        assert!(
            clean.checks.iter().any(|check| check.rule
                == "canonical_minimal_add_verified_view_issued"
                && check.status == "passed_v0"),
            "the production verifier must consume the structured tree"
        );

        let mut reordered = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut reordered,
            crate::core_lower::CoreLowerTreeCorruption::ReorderChildren,
        )
        .expect("reorder seam");
        assert!(failed_rule(
            &program,
            &reordered,
            "structured_expression_child_order"
        ));

        let mut duplicate = crate::core_lower::build_core_lower_report(&program, &diagnostics);
        crate::core_lower::corrupt_first_structured_expression_for_test(
            &mut duplicate,
            crate::core_lower::CoreLowerTreeCorruption::DuplicateChildIdentity,
        )
        .expect("duplicate seam");
        assert!(failed_rule(
            &program,
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
            &program,
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
            &program,
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
            &program,
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
            &program,
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
            &program,
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
            &program,
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
            &program,
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
