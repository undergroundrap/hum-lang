use crate::ast::{
    CanonicalExpression, CanonicalExpressionKind, Item, ParsedBinaryOperator, ParsedBodyStatement,
    ParsedSourceRange, Program,
};
use crate::callable;
use crate::core_contract;
use crate::core_expr;
use crate::core_lower::{
    self, CoreLowerExpression, CoreLowerItem, CoreLowerOperation, CoreLowerReport,
    CoreLowerSourceRange, CoreLowerStructuredExpression,
};
use crate::core_preview;
use crate::diagnostic::{Diagnostic, DiagnosticOccurrenceSet, Span};
use crate::ir_contract;
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    backend_passes: Vec<CoreBackendPassAuthority>,
}

struct CoreBackendPassAuthority {
    operation_id: String,
    context: (usize, Vec<u8>, String, String, String),
    conclusions: Vec<(&'static str, bool)>,
}

pub(crate) struct CoreVerifyFullTypeHandoff {
    readiness_summary: CoreVerifyReadinessSummary,
    diagnostic_occurrences: DiagnosticOccurrenceSet,
    _report_identity: usize,
}

pub(crate) struct CoreVerifyFullTypeReportAccess<'report> {
    program: &'report Program,
    report: &'report CoreVerifyReport,
    readiness_summary: &'report CoreVerifyReadinessSummary,
}

pub(crate) struct CoreVerifyDiagnosticOccurrenceAccess<'report> {
    occurrences: &'report DiagnosticOccurrenceSet,
    _report_identity: usize,
}

pub(crate) struct VerifiedCanonicalMinimalAddTypeResult<'report> {
    authority: &'report type_check::CanonicalMinimalAddTypeAuthority,
    backend_passes: &'report CoreBackendPassAuthority,
}

pub(crate) enum CanonicalMinimalAddTypeLookup<'report> {
    Delivered(VerifiedCanonicalMinimalAddTypeResult<'report>),
    MissingOperation,
    DuplicateOperation,
    AmbiguousOperation,
    ForeignOperation,
    NonSupportedDisposition,
    LocallyIneligible,
    ReportBlocked,
}

#[allow(unexpected_cfgs)]
mod verified_canonical_minimal_add_type_escape_compile_proof {
    #[cfg(hum_compile_fail_verified_canonical_minimal_add_type_escape)]
    mod enabled {
        use super::super::{
            CanonicalMinimalAddTypeLookup, CoreVerifyDiagnosticOccurrenceAccess,
            CoreVerifyFullTypeReportAccess, VerifiedCanonicalMinimalAddTypeResult,
            with_core_verify_for_full_type,
        };
        use crate::{
            ast::{Item, ParsedBodyStatement, Program},
            diagnostic::{Diagnostic, DiagnosticOccurrence},
        };
        type StaticAccess = CoreVerifyFullTypeReportAccess<'static>;
        type StaticResult = VerifiedCanonicalMinimalAddTypeResult<'static>;
        type StaticOccurrences = (
            &'static DiagnosticOccurrence,
            CoreVerifyDiagnosticOccurrenceAccess<'static>,
        );

        macro_rules! access_escape {
            ($name:ident) => {
                fn $name(program: &Program, diagnostics: &[Diagnostic]) -> StaticAccess {
                    let mut $name = None;
                    let _ = with_core_verify_for_full_type(program, diagnostics, |access| {
                        $name = Some(access)
                    });
                    $name.unwrap()
                }
            };
        }
        access_escape!(verified_canonical_minimal_add_access_cannot_outlive_verify_artifact);
        access_escape!(core_verify_full_type_report_access_cannot_escape);

        fn verified_canonical_minimal_add_result_cannot_be_collected(
            program: &Program,
            diagnostics: &[Diagnostic],
            item: &Item,
            statement: &ParsedBodyStatement,
        ) {
            let mut verified_canonical_minimal_add_result_cannot_be_collected: Vec<StaticResult> =
                vec![];
            let _ = with_core_verify_for_full_type(program, diagnostics, |access| {
                if let CanonicalMinimalAddTypeLookup::Delivered(result) =
                    access.canonical_minimal_add_type_for(item, statement)
                {
                    verified_canonical_minimal_add_result_cannot_be_collected.push(result);
                }
            });
        }

        fn verified_canonical_minimal_add_result_cannot_become_static(
            program: &Program,
            diagnostics: &[Diagnostic],
            item: &Item,
            statement: &ParsedBodyStatement,
        ) -> StaticResult {
            let mut verified_canonical_minimal_add_result_cannot_become_static = None;
            let _ = with_core_verify_for_full_type(program, diagnostics, |access| {
                let CanonicalMinimalAddTypeLookup::Delivered(result) =
                    access.canonical_minimal_add_type_for(item, statement)
                else {
                    panic!("missing verified result")
                };
                verified_canonical_minimal_add_result_cannot_become_static = Some(result);
            });
            verified_canonical_minimal_add_result_cannot_become_static.unwrap()
        }

        fn core_verify_diagnostic_occurrence_access_cannot_escape(
            program: &Program,
            diagnostics: &[Diagnostic],
        ) -> StaticOccurrences {
            with_core_verify_for_full_type(program, diagnostics, |access| {
                let core_verify_diagnostic_occurrence_access_cannot_escape =
                    access.diagnostic_occurrences();
                let occurrence = core_verify_diagnostic_occurrence_access_cannot_escape
                    .occurrences()
                    .next()
                    .expect("diagnostic occurrence");
                (
                    occurrence,
                    core_verify_diagnostic_occurrence_access_cannot_escape,
                )
            })
            .1
        }
    }
}

pub(crate) fn with_core_verify_for_full_type<R>(
    program: &Program,
    diagnostics: &[Diagnostic],
    consume: impl for<'report> FnOnce(CoreVerifyFullTypeReportAccess<'report>) -> R,
) -> (CoreVerifyFullTypeHandoff, R) {
    let report = build_report(program, diagnostics);
    let readiness_summary = readiness_summary_from_report(&report);
    let result = consume(CoreVerifyFullTypeReportAccess {
        program,
        report: &report,
        readiness_summary: &readiness_summary,
    });
    (
        CoreVerifyFullTypeHandoff {
            readiness_summary,
            _report_identity: std::ptr::from_ref(&report).addr(),
            diagnostic_occurrences: report.lower.diagnostic_occurrences,
        },
        result,
    )
}

impl CoreVerifyFullTypeHandoff {
    pub(crate) fn into_parts(self) -> (CoreVerifyReadinessSummary, DiagnosticOccurrenceSet) {
        (self.readiness_summary, self.diagnostic_occurrences)
    }
}

impl<'report> CoreVerifyFullTypeReportAccess<'report> {
    pub(crate) fn readiness_summary(&self) -> &'report CoreVerifyReadinessSummary {
        self.readiness_summary
    }

    pub(crate) fn diagnostic_occurrences(&self) -> CoreVerifyDiagnosticOccurrenceAccess<'report> {
        CoreVerifyDiagnosticOccurrenceAccess {
            occurrences: &self.report.lower.diagnostic_occurrences,
            _report_identity: std::ptr::from_ref(self.report).addr(),
        }
    }

    pub(crate) fn diagnostic_occurrence_set(&self) -> &'report DiagnosticOccurrenceSet {
        &self.report.lower.diagnostic_occurrences
    }

    pub(crate) fn canonical_minimal_add_type_for(
        &self,
        item: &Item,
        statement: &ParsedBodyStatement,
    ) -> CanonicalMinimalAddTypeLookup<'report> {
        let matching_items = self.report.lower.core_items.iter().filter(|candidate| {
            candidate.kind == item.kind()
                && candidate.name == item.name()
                && candidate.span == core_lower::portable_span(item.span())
        });
        let candidate_item = match unique(matching_items) {
            Some(Ok(item)) => item,
            Some(Err(())) => return CanonicalMinimalAddTypeLookup::AmbiguousOperation,
            None => return CanonicalMinimalAddTypeLookup::MissingOperation,
        };
        let matching_operations = candidate_item
            .operations
            .iter()
            .filter(|operation| operation.span == core_lower::portable_span(&statement.span));
        let operation = match unique(matching_operations) {
            Some(Ok(operation)) => operation,
            Some(Err(())) => return CanonicalMinimalAddTypeLookup::DuplicateOperation,
            None => return CanonicalMinimalAddTypeLookup::MissingOperation,
        };
        let has_check = |rule, status| {
            self.report.checks.iter().any(|check| {
                check.scope_id == operation.id && check.rule == rule && check.status == status
            })
        };
        if has_check("operation_index_consistent", "failed_v0") {
            return CanonicalMinimalAddTypeLookup::ForeignOperation;
        }
        let Some(authority) = operation
            .minimal_add_type_outcome()
            .and_then(type_check::CanonicalMinimalAddTypeOutcome::supported_authority)
        else {
            return CanonicalMinimalAddTypeLookup::NonSupportedDisposition;
        };
        if authority.verification_facts().5 != statement.source_node_id.as_str()
            || authority.verification_facts().0 != std::ptr::from_ref(self.program).addr()
        {
            return CanonicalMinimalAddTypeLookup::ForeignOperation;
        }
        if !has_check(
            "canonical_minimal_add_type_access_locally_eligible",
            "passed_v0",
        ) {
            return CanonicalMinimalAddTypeLookup::LocallyIneligible;
        }
        if self.report.failed_checks() > 0 {
            return CanonicalMinimalAddTypeLookup::ReportBlocked;
        }
        let backend_passes = match unique(
            self.report
                .backend_passes
                .iter()
                .filter(|candidate| candidate.operation_id == operation.id),
        ) {
            Some(Ok(authority)) => authority,
            Some(Err(())) => return CanonicalMinimalAddTypeLookup::DuplicateOperation,
            None => return CanonicalMinimalAddTypeLookup::LocallyIneligible,
        };
        let identity = authority.backend_identity();
        const REQUIRED_CORE_PASSES: [&str; 7] = [
            "parse",
            "semantic_graph_build",
            "resolve",
            "body_grammar",
            "core_preview",
            "core_lowering",
            "core_verify",
        ];
        if backend_passes.context.0 != identity.program_identity
            || backend_passes.context.1 != identity.owner.file.source_revision.as_ref()
            || backend_passes.context.2 != identity.source_identities[0]
            || backend_passes.context.3 != identity.source_identities[1]
            || backend_passes.context.4 != identity.source_identities[2]
            || backend_passes.conclusions.len() != REQUIRED_CORE_PASSES.len()
            || backend_passes
                .conclusions
                .iter()
                .zip(REQUIRED_CORE_PASSES)
                .any(|((actual, accepted), expected)| *actual != expected || !accepted)
        {
            return CanonicalMinimalAddTypeLookup::LocallyIneligible;
        }
        CanonicalMinimalAddTypeLookup::Delivered(VerifiedCanonicalMinimalAddTypeResult {
            authority,
            backend_passes,
        })
    }
}

fn unique<I: Iterator>(mut values: I) -> Option<Result<I::Item, ()>> {
    let value = values.next()?;
    Some(values.next().is_none().then_some(value).ok_or(()))
}

impl<'report> CoreVerifyDiagnosticOccurrenceAccess<'report> {
    pub(crate) fn occurrences(
        &self,
    ) -> impl Iterator<Item = &'report crate::diagnostic::DiagnosticOccurrence> + 'report {
        self.occurrences.occurrences()
    }
}

impl VerifiedCanonicalMinimalAddTypeResult<'_> {
    pub(crate) fn facts(&self) -> (&str, &str, &str, &str, &'static str, Option<bool>) {
        let (_, _, type_id, type_text, result_value_id, statement, compatible) =
            self.authority.verification_facts();
        (
            result_value_id,
            type_id,
            type_text,
            statement,
            "verified_canonical_minimal_add_type_v0",
            compatible,
        )
    }

    pub(crate) fn backend_identity(&self) -> type_check::CanonicalMinimalAddBackendIdentity<'_> {
        self.authority.backend_identity()
    }

    pub(crate) fn core_prerequisite_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.backend_passes
            .conclusions
            .iter()
            .map(|(name, _)| *name)
    }
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
    readiness_summary_from_report(&report)
}

fn readiness_summary_from_report(report: &CoreVerifyReport) -> CoreVerifyReadinessSummary {
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
    #[cfg(test)]
    let mut lower = lower;
    #[cfg(test)]
    CORE_VERIFY_TEST_CORRUPTION.with(|corruption| {
        if let Some(corruption) = corruption.take() {
            core_lower::corrupt_minimal_add_candidate_for_test(&mut lower, corruption);
        }
    });
    let report = finish_report(program, lower);
    #[cfg(test)]
    let mut report = report;
    #[cfg(test)]
    CORE_VERIFY_BACKEND_PASS_CORRUPTION.with(|corruption| {
        if let Some(corruption) = corruption.take() {
            corrupt_backend_passes_for_test(&mut report, corruption);
        }
    });
    report
}

fn finish_report(program: &Program, lower: CoreLowerReport) -> CoreVerifyReport {
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
    #[cfg(test)]
    CORE_VERIFY_REPORT_BUILD_COUNT.with(|count| count.set(count.get().checked_add(1).unwrap()));
    let backend_passes = collect_core_backend_passes(program, &lower);
    CoreVerifyReport {
        lower,
        checks,
        backend_passes,
    }
}

fn collect_core_backend_passes(
    program: &Program,
    lower: &CoreLowerReport,
) -> Vec<CoreBackendPassAuthority> {
    lower
        .core_items
        .iter()
        .flat_map(|item| &item.operations)
        .filter_map(|operation| {
            let authority = operation
                .minimal_add_type_outcome()?
                .supported_authority()?;
            let identity = authority.backend_identity();
            (identity.program_identity == std::ptr::from_ref(program).addr()).then(|| {
                CoreBackendPassAuthority {
                    operation_id: operation.id.clone(),
                    context: (
                        identity.program_identity,
                        identity.owner.file.source_revision.to_vec(),
                        identity.source_identities[0].clone(),
                        identity.source_identities[1].clone(),
                        identity.source_identities[2].clone(),
                    ),
                    conclusions: vec![
                        ("parse", true),
                        ("semantic_graph_build", true),
                        ("resolve", true),
                        ("body_grammar", true),
                        ("core_preview", true),
                        ("core_lowering", true),
                        ("core_verify", true),
                    ],
                }
            })
        })
        .collect()
}

#[cfg(test)]
thread_local! {
    static CORE_VERIFY_REPORT_BUILD_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static CORE_VERIFY_TEST_CORRUPTION: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
    static CORE_VERIFY_BACKEND_PASS_CORRUPTION: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(crate) fn reset_core_verify_report_build_count_for_test() {
    CORE_VERIFY_REPORT_BUILD_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn core_verify_report_build_count_for_test() -> usize {
    CORE_VERIFY_REPORT_BUILD_COUNT.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(crate) fn set_core_verify_corruption_for_test(corruption: &'static str) {
    CORE_VERIFY_TEST_CORRUPTION.with(|active| assert_eq!(active.replace(Some(corruption)), None));
}

#[cfg(test)]
pub(crate) fn set_backend_pass_corruption_for_test(corruption: &'static str) {
    CORE_VERIFY_BACKEND_PASS_CORRUPTION
        .with(|active| assert_eq!(active.replace(Some(corruption)), None));
}

#[cfg(test)]
fn corrupt_backend_passes_for_test(report: &mut CoreVerifyReport, corruption: &'static str) {
    let authority = report
        .backend_passes
        .first_mut()
        .expect("backend-pass corruption requires one supported operation");
    let preview = authority
        .conclusions
        .iter()
        .position(|(name, _)| *name == "core_preview")
        .expect("core preview pass");
    match corruption {
        "missing_core_preview" => {
            authority.conclusions.remove(preview);
        }
        "blocked_core_preview" => authority.conclusions[preview].1 = false,
        "duplicate_core_preview" => {
            if let Some(next) = preview.checked_add(1) {
                authority.conclusions.insert(next, ("core_preview", true));
            } else {
                authority.conclusions.clear();
            }
        }
        "foreign_core_preview" => authority.context.0 ^= 1,
        "reordered_core_preview" => {
            if let Some(prior) = preview.checked_sub(1) {
                authority.conclusions.swap(prior, preview);
            } else {
                authority.conclusions.clear();
            }
        }
        _ => panic!("unknown backend-pass corruption: {corruption}"),
    }
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
impl CoreVerifyFullTypeHandoff {
    pub(crate) fn report_identity_for_test(&self) -> usize {
        self._report_identity
    }
}

#[cfg(test)]
impl CoreVerifyFullTypeReportAccess<'_> {
    pub(crate) fn report_identity_for_test(&self) -> usize {
        std::ptr::from_ref(self.report).addr()
    }
}

#[cfg(test)]
impl CoreVerifyDiagnosticOccurrenceAccess<'_> {
    pub(crate) fn report_identity_for_test(&self) -> usize {
        self._report_identity
    }
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

    let predicate_analysis = predicate::PredicateAnalysis::build(program);
    let mut candidate_item_cursor = 0usize;
    for file in &program.files {
        verify_program_items(
            program,
            &file.items,
            lower,
            predicate_analysis.facts(),
            &mut candidate_item_cursor,
            &mut checks,
        );
    }
    while let Some(item) = lower.core_items.get(candidate_item_cursor) {
        verify_item(
            item,
            false,
            |checks| {
                for (index, operation) in item.operations.iter().enumerate() {
                    verify_operation(program, item, operation, index, None, checks);
                }
            },
            &mut checks,
        );
        let Some(next) = candidate_item_cursor.checked_add(1) else {
            break;
        };
        candidate_item_cursor = next;
    }

    checks
}

fn verify_program_items(
    program: &Program,
    items: &[Item],
    lower: &CoreLowerReport,
    predicate_facts: &[crate::predicate::PredicateFact],
    candidate_item_cursor: &mut usize,
    checks: &mut Vec<CoreVerifyCheck>,
) {
    for item in items {
        if let Some(does) = item
            .sections()
            .iter()
            .find(|section| section.name == "does")
        {
            let expected = program.canonical_core_operation_owner_expectation(item, does);
            let candidate = lower.core_items.get(*candidate_item_cursor);
            match (expected, candidate) {
                (Ok(expected), Some(candidate))
                    if core_lower::core_item_occupies_expected_slot(&expected, candidate) =>
                {
                    verify_item(
                        candidate,
                        true,
                        |checks| {
                            let mut expected_slots = Some(0usize);
                            let _streamed =
                                core_lower::with_fresh_expected_core_operations_for_item(
                                    program,
                                    item,
                                    does,
                                    expected,
                                    predicate_facts,
                                    |slot| {
                                        verify_expected_operation(
                                            program,
                                            candidate,
                                            &mut expected_slots,
                                            slot,
                                            checks,
                                        )
                                    },
                                );
                            verify_remaining_operations(program, candidate, expected_slots, checks);
                        },
                        checks,
                    );
                    if let Some(next) = candidate_item_cursor.checked_add(1) {
                        *candidate_item_cursor = next;
                    } else {
                        push_check(
                            checks,
                            "core_item",
                            &candidate.id,
                            Some(&candidate.span),
                            false,
                            "row_identity",
                            "core item has no exact Program-owned source-slot association",
                        );
                    }
                }
                _ => push_missing_item_check(item, checks),
            }
        }
        verify_program_items(
            program,
            item.nested_items(),
            lower,
            predicate_facts,
            candidate_item_cursor,
            checks,
        );
    }
}

fn push_missing_item_check(item: &Item, checks: &mut Vec<CoreVerifyCheck>) {
    let id = crate::node_id::span(
        "core-item",
        item.span(),
        &format!("{} {}", item.kind(), item.name()),
    );
    push_check(
        checks,
        "core_item",
        &id,
        Some(item.span()),
        false,
        "expected_core_item_present",
        "parser-owned Core item has one exact lowered candidate",
    );
}

fn verify_item(
    item: &CoreLowerItem,
    associated: bool,
    operations: impl FnOnce(&mut Vec<CoreVerifyCheck>),
    checks: &mut Vec<CoreVerifyCheck>,
) {
    push_span_check(checks, "core_item", &item.id, &item.span);
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        associated && !item.id.trim().is_empty(),
        "row_identity",
        if associated {
            "core item id is present"
        } else {
            "core item has no exact Program-owned source-slot association"
        },
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
    operations(checks);
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

#[cfg(test)]
fn verify_expected_operations(
    program: &Program,
    expected_owner: crate::ast::CanonicalCoreOperationOwnerExpectation<'_>,
    item: &CoreLowerItem,
    body: &crate::core_body::CanonicalBodyGrammarReport,
    predicate_facts: &[crate::predicate::PredicateFact],
    operation_slot: usize,
    checks: &mut Vec<CoreVerifyCheck>,
) -> Result<(), crate::core_lower::CoreOperationExpectationError> {
    let mut expected_slots = Some(operation_slot);
    let streamed = core_lower::with_expected_core_operations_from_slot_for_test(
        expected_owner,
        body,
        predicate_facts,
        operation_slot,
        |expected| verify_expected_operation(program, item, &mut expected_slots, expected, checks),
    );
    verify_remaining_operations(program, item, expected_slots, checks);
    streamed
}

fn verify_expected_operation(
    program: &Program,
    item: &CoreLowerItem,
    expected_slots: &mut Option<usize>,
    expected: crate::core_lower::ExpectedCoreOperation<'_, '_>,
    checks: &mut Vec<CoreVerifyCheck>,
) {
    let slot = expected.slot();
    *expected_slots = slot.checked_add(1);
    if let Some(operation) = item.operations.get(slot) {
        verify_operation(program, item, operation, slot, Some(&expected), checks);
    } else {
        push_check(
            checks,
            "core_item",
            &item.id,
            Some(core_lower::expected_core_operation_source_span(&expected)),
            false,
            "expected_core_operation_present",
            "parser-owned Core operation slot has one lowered candidate",
        );
    }
}

fn verify_remaining_operations(
    program: &Program,
    item: &CoreLowerItem,
    expected_slots: Option<usize>,
    checks: &mut Vec<CoreVerifyCheck>,
) {
    let Some(expected_slots) = expected_slots else {
        for (index, operation) in item.operations.iter().enumerate() {
            verify_operation(program, item, operation, index, None, checks);
        }
        return;
    };
    for (index, operation) in item.operations.iter().enumerate().skip(expected_slots) {
        verify_operation(program, item, operation, index, None, checks);
    }
}

fn verify_operation(
    program: &Program,
    item: &CoreLowerItem,
    operation: &CoreLowerOperation,
    expected_index: usize,
    expected: Option<&crate::core_lower::ExpectedCoreOperation<'_, '_>>,
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
        operation.index == expected_index
            && expected.is_some_and(|expected| {
                core_lower::core_operation_occupies_expected_slot(expected, operation)
            }),
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

    let expression_checks_start = checks.len();
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
                    let emit_outer_unchecked = !operation
                        .minimal_add_type_outcome()
                        .is_some_and(type_check::CanonicalMinimalAddTypeOutcome::is_supported);
                    verify_structured_expression(
                        operation,
                        expression,
                        structured,
                        emit_outer_unchecked,
                        checks,
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
    push_minimal_add_disposition_checks(
        program,
        operation,
        expected,
        expression_checks_start,
        checks,
    );
}

fn push_minimal_add_disposition_checks(
    program: &Program,
    operation: &CoreLowerOperation,
    expected: Option<&crate::core_lower::ExpectedCoreOperation<'_, '_>>,
    expression_checks_start: usize,
    checks: &mut Vec<CoreVerifyCheck>,
) {
    let Some(outcome) = operation.minimal_add_type_outcome() else {
        return;
    };
    let structural_checks_pass = checks[expression_checks_start..]
        .iter()
        .all(|check| check.status == "passed_v0");
    let mut push = |passed, rule, detail| {
        push_check(
            checks,
            "operation_expression",
            &operation.id,
            Some(&operation.span),
            passed,
            rule,
            detail,
        )
    };
    if outcome.is_supported() {
        let (authority_matches, claim_matches, public_projection_matches) =
            core_lower::canonical_minimal_add_verification_facts(program, expected, operation);
        push(
            authority_matches,
            "canonical_minimal_add_type_authority_matches_exact_operation",
            "canonical minimal-add type authority matches parser, resolver, declarations, and exact Core operation",
        );
        push(
            authority_matches && claim_matches,
            "canonical_minimal_add_type_claim_matches_untouched_authority",
            "canonical minimal-add private type claim matches untouched checked authority",
        );
        let public_pass = authority_matches && claim_matches && public_projection_matches;
        push(
            public_pass,
            "canonical_minimal_add_result_value_matches_checked_type",
            "canonical minimal-add public type and result value match untouched checked authority",
        );
        push(
            structural_checks_pass && public_pass,
            "canonical_minimal_add_type_access_locally_eligible",
            "canonical minimal-add target is locally eligible for report-bound verified type access",
        );
    } else {
        let (rule, detail) = if outcome.integrity_failure_reason().is_some() {
            (
                "canonical_minimal_add_integrity_failure_rejected",
                "recognized canonical minimal-add target has incomplete or inconsistent checked type authority",
            )
        } else if outcome.is_unsupported_target_like() {
            (
                "canonical_minimal_add_unsupported_target_like_rejected",
                "unsupported additive task-return shape has no checked canonical type authority",
            )
        } else {
            return;
        };
        push(false, rule, detail);
    }
}

fn verify_structured_expression(
    operation: &CoreLowerOperation,
    expression: &CoreLowerExpression,
    structured: &CoreLowerStructuredExpression,
    emit_outer_unchecked: bool,
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

    if emit_outer_unchecked {
        let outer_type_unchecked = expression.type_status == core_expr::CORE_EXPRESSION_TYPE_STATUS
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
            | core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_STATUS
            | core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_INTEGRITY_FAILURE_STATUS
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
    use crate::core_body::CanonicalBodyGrammarReport;
    use crate::core_lower::{self, CoreLowerReport, CoreOperationExpectationError};
    use crate::diagnostic::Diagnostic;
    use crate::parser::parse_source;
    use crate::predicate::{PredicateAnalysis, PredicateFact};

    use super::{
        build_report, core_verify_json, core_verify_text,
        validate_diagnostic_projection_from_source, verify_lower_report,
    };

    #[rustfmt::skip]
    #[test]
    fn borrowed_core_operation_expectation_is_load_bearing() {
        const SOURCE: &str = "task first(value: Int) -> Int {\n  needs:\n    value < 100\n  ensures:\n    value < 100\n  does:\n    let next = value + 1\n    return next\n}\ntask empty() {\n  does:\n}\ntask last() -> Int {\n  does:\n    return 1\n}\ntest order remains stable unit {\n  does:\n    expect first(1) returns 2\n}\n";
        fn subject(source: &str) -> (Program, Vec<Diagnostic>) {
            let parsed = parse_source("order-bound.hum", source);
            (Program { files: vec![parsed.file] }, parsed.diagnostics)
        }
        fn lower(p: &Program, d: &[Diagnostic]) -> CoreLowerReport { core_lower::build_core_lower_report(p, d) }
        fn failures(p: &Program, r: &CoreLowerReport, rule: &str) -> usize {
            verify_lower_report(p, r).iter().filter(|c| c.rule == rule && c.status == "failed_v0").count()
        }
        fn reindex(r: &mut CoreLowerReport, item: usize) {
            for (index, operation) in r.core_items[item].operations.iter_mut().enumerate() { operation.index = index; }
        }
        fn artifacts(p: &Program) -> (CanonicalBodyGrammarReport, Vec<PredicateFact>) {
            let item = &p.files[0].items[0];
            let does = item.sections().iter().find(|s| s.name == "does").unwrap();
            let body = crate::core_body::analyze_does_section_for_lowering(p.canonical_core_expectation(item, does).unwrap());
            (body, PredicateAnalysis::build(p).facts().to_vec())
        }
        fn injected(p: &Program, r: &CoreLowerReport, body: &CanonicalBodyGrammarReport, facts: &[PredicateFact])
            -> (Result<(), CoreOperationExpectationError>, Vec<super::CoreVerifyCheck>) {
            let item = &p.files[0].items[0];
            let does = item.sections().iter().find(|s| s.name == "does").unwrap();
            let owner = p.canonical_core_operation_owner_expectation(item, does).unwrap();
            let mut checks = Vec::new();
            let result = super::verify_expected_operations(p, owner, &r.core_items[0], body, facts, 0, &mut checks);
            (result, checks)
        }
        fn fails(p: &Program, r: &CoreLowerReport, rule: &str, count: usize) { assert_eq!(failures(p, r, rule), count, "{rule}"); }
        fn rejected_local(p: &Program, r: &CoreLowerReport, body: &CanonicalBodyGrammarReport, facts: &[PredicateFact]) -> bool {
            let (result, checks) = injected(p, r, body, facts);
            result.is_err() && checks.iter().any(|c| c.rule == "operation_index_consistent" && c.status == "failed_v0")
        }
        fn fixture(path: &str, source: &str, checks: usize) -> super::CoreVerifyReport {
            let parsed = parse_source(path, source);
            let program = Program { files: vec![parsed.file] };
            let report = super::build_report(&program, &parsed.diagnostics);
            assert_eq!((report.checks.len(), report.failed_checks()), (checks, 0));
            report
        }
        let (program, diagnostics) = subject(SOURCE);
        let clean = lower(&program, &diagnostics);
        assert!(verify_lower_report(&program, &clean).iter().all(|c| c.status == "passed_v0"));
        assert_eq!((clean.core_items.len(), clean.core_items[0].operations.len()), (4, 4));
        assert!(clean.core_items[1].operations.is_empty());
        assert!(clean.core_items[0].operations[2..].iter().all(|op| op.source_kind == "contract_predicate"));
        assert_ne!(clean.core_items[3].kind, "task");
        let (foreign_program, foreign_diagnostics) = subject(SOURCE);
        let foreign = lower(&foreign_program, &foreign_diagnostics);
        let (revised_program, revised_diagnostics) = subject(&format!("{SOURCE}\n"));
        let revised = lower(&revised_program, &revised_diagnostics);
        let (sole_program, sole_diagnostics) =
            subject("task only() -> Int {\n  does:\n    return 1\n}\n");
        let mut sole = lower(&sole_program, &sole_diagnostics); sole.core_items[0].operations.clear();
        fails(&sole_program, &sole, "expected_core_operation_present", 1);
        let mut sole_item = lower(&sole_program, &sole_diagnostics); sole_item.core_items.clear();
        fails(&sole_program, &sole_item, "expected_core_item_present", 1);
        for (kind, a, b, rule, expected) in [
            ("di", 0, 0, "expected_core_item_present", 1), ("di", 1, 0, "expected_core_item_present", 1),
            ("di", 3, 0, "expected_core_item_present", 1),
            ("ii", 1, 0, "row_identity", 0), ("ii", 4, 0, "row_identity", 0),
            ("si", 0, 2, "expected_core_item_present", 0),
            ("do", 1, 0, "expected_core_operation_present", 1), ("do", 3, 0, "expected_core_operation_present", 1),
            ("io", 1, 0, "operation_index_consistent", 0), ("io", 4, 0, "operation_index_consistent", 0),
            ("so", 0, 1, "operation_index_consistent", 2), ("so", 0, 2, "operation_index_consistent", 2),
            ("so", 2, 3, "operation_index_consistent", 2),
            ("sr", 0, 1, "operation_index_consistent", 2), ("sr", 0, 2, "operation_index_consistent", 2),
            ("sr", 2, 3, "operation_index_consistent", 2),
            ("po", 2, 3, "operation_index_consistent", 2),
        ] {
            let mut changed = lower(&program, &diagnostics);
            match kind {
                "di" => { changed.core_items.remove(a); }
                "ii" => { let extra = lower(&program, &diagnostics).core_items.remove(0); changed.core_items.insert(a, extra); }
                "si" => changed.core_items.swap(a, b),
                "do" => { changed.core_items[0].operations.remove(a); reindex(&mut changed, 0); }
                "io" => { let mut issued = lower(&program, &diagnostics); let extra = issued.core_items[0].operations.remove(0); changed.core_items[0].operations.insert(a, extra); reindex(&mut changed, 0); }
                "so" | "sr" => { changed.core_items[0].operations.swap(a, b); if kind == "sr" { reindex(&mut changed, 0); } }
                "po" => core_lower::swap_operation_origins_for_test(&mut changed, 0, a, b),
                _ => unreachable!(),
            }
            let count = failures(&program, &changed, rule);
            assert!(if expected == 0 { count > 0 } else { count == expected });
            if kind == "sr" {
                assert!(verify_lower_report(&program, &changed).iter().filter(|c| c.status == "failed_v0").all(|c| c.rule == rule));
            }
        }
        let mut middle = lower(&program, &diagnostics); middle.core_items[0].operations.remove(1);
        reindex(&mut middle, 0);
        assert!(failures(&program, &middle, "operation_index_consistent") > 0);
        for (source, source_item) in [(&foreign, 0usize), (&foreign, 2), (&revised, 0)] {
            let mut changed = lower(&program, &diagnostics);
            core_lower::copy_item_origin_for_test(&mut changed, 0, source, source_item);
            assert!(failures(&program, &changed, "expected_core_item_present") > 0);
            assert!(failures(&program, &changed, "row_identity") > 0);
        }
        for source in [&foreign, &revised] {
            let mut changed = lower(&program, &diagnostics);
            core_lower::copy_operation_origin_for_test(&mut changed, 0, 0, source, 0, 0);
            fails(&program, &changed, "operation_index_consistent", 1);
        }
        let mut changed = lower(&program, &diagnostics);
        core_lower::reject_operation_origin_for_test(&mut changed, 0, 0);
        fails(&program, &changed, "operation_index_consistent", 1);
        let (body, facts) = artifacts(&program);
        for duplicate in [false, true] {
            let mut changed = body.clone();
            if duplicate { changed.statements.insert(0, body.statements[0].clone()); } else { changed.statements.remove(0); }
            assert!(rejected_local(&program, &clean, &changed, &facts));
        }
        for duplicate in [false, true] {
            let mut changed = facts.clone();
            if duplicate { changed.insert(0, facts[0].clone()); } else { changed.remove(0); }
            assert!(rejected_local(&program, &clean, &body, &changed));
        }
        let (mut sole_body, sole_facts) = artifacts(&sole_program); sole_body.statements.clear();
        let (result, sole_checks) = injected(&sole_program, &sole, &sole_body, &sole_facts);
        assert_eq!(result, Err(CoreOperationExpectationError::Missing(0)));
        let missing = sole_checks.first().expect("sole missing row");
        assert_eq!(missing.scope, "core_item");
        assert_eq!(missing.scope_id, sole.core_items[0].id);
        assert_eq!(missing.status, "failed_v0");
        assert_eq!(missing.rule, "expected_core_operation_present");
        assert_eq!(missing.detail, "parser-owned Core operation slot has one lowered candidate");
        assert_eq!(missing.span.as_ref(), Some(&sole_program.files[0].items[0].sections()[0].lines[0].span));
        let report = super::CoreVerifyReport {
            lower: sole,
            checks: sole_checks,
            backend_passes: Vec::new(),
        };
        assert_eq!(report.verification_status(), super::CORE_VERIFY_FAILED_STATUS);
        assert_eq!(report.verified_operations(), 0);

        let mut middle_lower = lower(&program, &diagnostics); middle_lower.core_items[0].operations.remove(1);
        reindex(&mut middle_lower, 0);
        let mut middle_body = body.clone(); middle_body.statements.remove(1);
        let (_, middle_checks) = injected(&program, &middle_lower, &middle_body, &facts);
        assert_eq!(middle_checks.iter().filter(|c| c.rule == "expected_core_operation_present").count(), 1);
        let mut final_lower = lower(&program, &diagnostics); final_lower.core_items[0].operations.pop();
        let mut final_facts = facts.clone(); final_facts.pop();
        let (_, final_checks) = injected(&program, &final_lower, &body, &final_facts);
        assert_eq!(final_checks.last().unwrap().rule, "expected_core_operation_present");

        let mut underflow_body = body.clone();
        underflow_body.statements.remove(0);
        assert_eq!(injected(&program, &clean, &underflow_body, &facts).0, Err(CoreOperationExpectationError::SlotUnderflow));
        let mut impossible_body = body.clone();
        let mut foreign_artifact = body.statements[0].clone();
        foreign_artifact.statement_mut_for_test().span.file = "foreign/order.hum".to_string();
        impossible_body.statements.insert(0, foreign_artifact);
        assert_eq!(injected(&program, &clean, &impossible_body, &facts).0, Err(CoreOperationExpectationError::Ordering(0)));
        let item = &program.files[0].items[0];
        let does = item.sections().iter().find(|s| s.name == "does").unwrap();
        let owner = program.canonical_core_operation_owner_expectation(item, does).unwrap();
        let mut overflow_checks = Vec::new();
        let overflow = super::verify_expected_operations(&program, owner, &clean.core_items[0], &body, &facts, usize::MAX, &mut overflow_checks);
        assert_eq!(overflow, Err(CoreOperationExpectationError::SlotOverflow));
        assert!(overflow_checks.iter().any(|c| c.status == "failed_v0"));

        fixture("examples/core/minimal_add.hum", include_str!("../examples/core/minimal_add.hum"), 38);
        let uint_report = fixture("fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum", include_str!("../fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum"), 766);
        assert!(uint_report.lower.core_items.iter().flat_map(|i| &i.operations).any(|op| op.core_operation == "blocked_unsupported_try_expression"));
    }

    #[test]
    fn task_signature_authority_is_load_bearing() {
        const SOURCE: &str =
            "task identity(left: Int, right: Int) -> Int {\n  does:\n    return left\n}\n";

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
            assert_eq!(
                checks.len(),
                clean_count.checked_add(1).unwrap(),
                "{corruption:?}"
            );
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
                == "canonical_minimal_add_type_access_locally_eligible"
                && check.status == "passed_v0"),
            "the production verifier must consume the checked structured tree"
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

    #[test]
    fn canonical_minimal_add_type_verification_withholds_invalid_access() {
        fn subject(path: &str, source: &str) -> (Program, Vec<Diagnostic>) {
            let parsed = parse_source(path, source);
            let program = Program {
                files: vec![parsed.file],
            };
            (program, parsed.diagnostics)
        }
        fn disposition_fixture(
            source: &str,
            checks: usize,
            failed: usize,
            failure_rule: Option<&str>,
        ) {
            let (program, diagnostics) = subject("disposition.hum", source);
            let report = build_report(&program, &diagnostics);
            let matching_failures = report
                .checks
                .iter()
                .filter(|check| Some(check.rule) == failure_rule && check.status == "failed_v0")
                .count();
            assert_eq!(report.checks.len(), checks);
            assert_eq!(report.failed_checks(), failed);
            assert_eq!(matching_failures, usize::from(failure_rule.is_some()));
        }
        fn supported_corruption(
            program: &Program,
            diagnostics: &[Diagnostic],
            mutate: impl FnOnce(&mut CoreLowerReport),
        ) -> usize {
            let mut lower = crate::core_lower::build_core_lower_report(program, diagnostics);
            let before = snapshot(&lower.core_items[0].operations[0]);
            mutate(&mut lower);
            let after = snapshot(&lower.core_items[0].operations[0]);
            assert_eq!(after, before);
            let report = super::finish_report(program, lower);
            let failed = |rule| {
                report
                    .checks
                    .iter()
                    .any(|check| check.rule == rule && check.status == "failed_v0")
            };
            assert!(failed("canonical_minimal_add_type_access_locally_eligible"));
            assert!(!failed("canonical_minimal_add_integrity_failure_rejected"));
            assert_eq!(lookup_name(program, &report), "LocallyIneligible");
            report.checks.len()
        }
        fn snapshot(operation: &crate::core_lower::CoreLowerOperation) -> (usize, String) {
            operation
                .minimal_add_type_outcome()
                .and_then(
                    crate::type_check::CanonicalMinimalAddTypeOutcome::authority_snapshot_for_test,
                )
                .expect("issued Supported authority")
        }
        fn lookup_name(program: &Program, report: &super::CoreVerifyReport) -> &'static str {
            use super::CanonicalMinimalAddTypeLookup::*;
            let item = &program.files[0].items[0];
            let does = item
                .sections()
                .iter()
                .find(|section| section.name == "does")
                .unwrap();
            let statement = does.body_syntax[0].as_ref().unwrap();
            let summary = super::readiness_summary_from_report(report);
            let access = super::CoreVerifyFullTypeReportAccess {
                program,
                report,
                readiness_summary: &summary,
            };
            match access.canonical_minimal_add_type_for(item, statement) {
                Delivered(_) => "Delivered",
                MissingOperation => "MissingOperation",
                DuplicateOperation => "DuplicateOperation",
                AmbiguousOperation => "AmbiguousOperation",
                ForeignOperation => "ForeignOperation",
                NonSupportedDisposition => "NonSupportedDisposition",
                LocallyIneligible => "LocallyIneligible",
                ReportBlocked => "ReportBlocked",
            }
        }
        let (program, diagnostics) = subject(
            "verified-add.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\n",
        );
        macro_rules! corrupt {
            ($mutation:expr) => {
                supported_corruption(&program, &diagnostics, $mutation)
            };
        }
        let clean = build_report(&program, &diagnostics);
        assert_eq!((clean.checks.len(), clean.failed_checks()), (38, 0));
        let rules = clean
            .checks
            .iter()
            .filter(|check| check.scope == "operation_expression")
            .map(|check| check.rule)
            .collect::<Vec<_>>();
        assert!(rules.ends_with(&[
            "canonical_minimal_add_type_authority_matches_exact_operation",
            "canonical_minimal_add_type_claim_matches_untouched_authority",
            "canonical_minimal_add_result_value_matches_checked_type",
            "canonical_minimal_add_type_access_locally_eligible",
        ]));
        assert_eq!(lookup_name(&program, &clean), "Delivered");
        let (foreign_program, foreign_diagnostics) = subject(
            "verified-add.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\n",
        );
        assert_eq!(lookup_name(&foreign_program, &clean), "ForeignOperation");
        let foreign_lower =
            crate::core_lower::build_core_lower_report(&foreign_program, &foreign_diagnostics);

        for corruption in [
            "claim",
            "type-status",
            "type-text",
            "type-source",
            "result-id",
            "result-type-id",
            "result-status",
            "result-text",
            "result-provenance",
        ] {
            corrupt!(|lower| core_lower::corrupt_minimal_add_candidate_for_test(lower, corruption));
        }
        use core_lower::CoreLowerTreeCorruption as Tree;
        for corruption in [
            Tree::ReorderChildren,
            Tree::DuplicateChildIdentity,
            Tree::IncorrectIdentifierSpelling("foreign".to_string()),
            Tree::OverflowSizedRange,
            Tree::ZeroBasedRange,
        ] {
            corrupt!(|lower| {
                core_lower::corrupt_first_structured_expression_for_test(lower, corruption)
                    .expect("structured corruption")
            });
        }
        corrupt!(|lower| core_lower::corrupt_minimal_add_candidate_for_test(lower, "coherent"));
        for (corruption, total) in [
            ("drop-authority", 38usize),
            ("drop-projection", 26usize),
            ("drop-both", 25usize),
        ] {
            let observed = corrupt!(|lower| core_lower::corrupt_minimal_add_candidate_for_test(
                lower, corruption
            ));
            assert_eq!(observed, total);
        }
        for (case, expected) in [
            ("missing", "MissingOperation"),
            ("duplicate", "DuplicateOperation"),
            ("ambiguous", "AmbiguousOperation"),
            ("foreign", "ForeignOperation"),
            ("rejected", "ForeignOperation"),
        ] {
            let mut lower = core_lower::build_core_lower_report(&program, &diagnostics);
            let before = snapshot(&lower.core_items[0].operations[0]);
            let mut issued = core_lower::build_core_lower_report(&program, &diagnostics);
            let removed = match case {
                "missing" => Some(lower.core_items[0].operations.remove(0)),
                "duplicate" => {
                    let duplicate = issued.core_items[0].operations.remove(0);
                    lower.core_items[0].operations.push(duplicate);
                    None
                }
                "ambiguous" => {
                    lower.core_items.push(issued.core_items.remove(0));
                    None
                }
                "foreign" => {
                    core_lower::copy_operation_origin_for_test(
                        &mut lower,
                        0,
                        0,
                        &foreign_lower,
                        0,
                        0,
                    );
                    None
                }
                "rejected" => {
                    core_lower::reject_operation_origin_for_test(&mut lower, 0, 0);
                    None
                }
                _ => unreachable!(),
            };
            let preserved = removed
                .as_ref()
                .unwrap_or_else(|| &lower.core_items[0].operations[0]);
            assert_eq!(snapshot(preserved), before);
            assert_eq!(
                lookup_name(&program, &super::finish_report(&program, lower)),
                expected
            );
        }
        let (reordered_program, reordered_diagnostics) = subject(
            "reordered-add.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n    return a + b\n}\n",
        );
        let mut lower =
            crate::core_lower::build_core_lower_report(&reordered_program, &reordered_diagnostics);
        let before = snapshot(&lower.core_items[0].operations[0]);
        lower.core_items[0].operations.swap(0, 1);
        for (index, operation) in lower.core_items[0].operations.iter_mut().enumerate() {
            operation.index = index;
        }
        assert_eq!(snapshot(&lower.core_items[0].operations[1]), before);
        assert_eq!(
            lookup_name(
                &reordered_program,
                &super::finish_report(&reordered_program, lower)
            ),
            "ForeignOperation"
        );

        let (blocked_program, blocked_diagnostics) = subject(
            "report-blocked.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\ntask unsupported(a: Int) -> Int {\n  does:\n    return 1 + a\n}\n",
        );
        let blocked_report = build_report(&blocked_program, &blocked_diagnostics);
        assert_eq!(
            lookup_name(&blocked_program, &blocked_report),
            "ReportBlocked"
        );
        let add = |params: &str, result: &str, expression: &str| {
            format!("task add({params}){result} {{\n  does:\n    return {expression}\n}}\n")
        };
        for (params, result, expression, checks, failed, kind) in [
            ("a: UInt, b: UInt", " -> UInt", "a + b", 35, 0, 'n'),
            ("a: UInt", " -> UInt", "a + 1", 21, 0, 'n'),
            ("a: Int, b: UInt", " -> Int", "a + b", 36, 1, 'u'),
            ("a: Int", " -> Int", "1 + a", 22, 1, 'u'),
            ("", " -> Int", "1 + 2", 22, 1, 'u'),
            ("a: Int, b: Int", " -> Int", "a + missing", 38, 2, 'i'),
            ("b: Int", " -> Int", "1 + missing", 24, 1, 'i'),
            ("a: Int", " -> Int", "a", 21, 0, 'n'),
        ] {
            let rule = match kind {
                'u' => Some("canonical_minimal_add_unsupported_target_like_rejected"),
                'i' => Some("canonical_minimal_add_integrity_failure_rejected"),
                _ => None,
            };
            disposition_fixture(&add(params, result, expression), checks, failed, rule);
        }
    }
}
