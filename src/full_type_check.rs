use std::collections::BTreeMap;

use crate::ast::{Item, Param, Program, Section};
use crate::callable::{self, CallableAnalysis};
use crate::core_body::{self, BodyStatement};
use crate::core_contract;
use crate::core_verify;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticOccurrence, DiagnosticOccurrenceSet, Severity, Span,
};
use crate::element_place;
use crate::field_place::{self, FieldTypeMap};
use crate::predicate::{self, PredicateFact};
use crate::return_dependency;
use crate::type_check;
use crate::type_scopes::TypeScopeStack;
use crate::typed_failure::{self, FailureFact, ProgramFailureAnalysis};
use crate::version;
use crate::writable_field_alias;

pub const FULL_TYPE_CHECK_SCHEMA: &str = "hum.full_type_check.v0";
pub const FULL_TYPE_CHECK_MODE: &str = "recognized_core_body_type_gate_v0";
pub const FULL_TYPE_CHECK_STATUS: &str = "recognized_core_body_type_gate_available_v0";

const NON_CLAIMS: &[&str] = &[
    "no executable semantics",
    "no Hum IR emission",
    "no backend lowering",
    "no proof artifact",
    "no memory-safety proof",
    "no effect checking",
    "no ownership or borrow checking",
    "no generic, trait, layout, or ABI checking",
    "no optimization claim",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullTypeCheckSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub source_errors: usize,
    pub resolver_errors: usize,
    pub type_errors: usize,
    pub core_verify_errors: usize,
    pub items: usize,
    pub body_items: usize,
    pub statements: usize,
    pub checked_statements: usize,
    pub accepted_statements: usize,
    pub rejected_statements: usize,
    pub unchecked_statements: usize,
    pub unsupported_statements: usize,
    pub blocking_issues: usize,
    pub execution_ready: usize,
    pub ir_ready: usize,
}

struct FullTypeCheckReport {
    type_check_summary: type_check::TypeCheckSummary,
    core_verify_summary: core_verify::CoreVerifyReadinessSummary,
    items: Vec<FullTypeItem>,
    files: usize,
    item_count: usize,
    source_errors: usize,
    predicates: Vec<PredicateFact>,
    diagnostic_occurrences: DiagnosticOccurrenceSet,
}

pub(crate) struct FullTypeEffectReportAccess<'report> {
    program: &'report Program,
    report: &'report FullTypeCheckReport,
    core_verify: &'report core_verify::CoreVerifyFullTypeReportAccess<'report>,
}

pub(crate) struct VerifiedMinimalAddFullType<'report>(
    core_verify::VerifiedCanonicalMinimalAddTypeResult<'report>,
);

pub(crate) struct VerifiedIntegerSignFullType<'report> {
    authority: &'report type_check::CanonicalIntegerSignTypeAuthority,
    _report: &'report FullTypeCheckReport,
}

pub(crate) struct VerifiedConstantTextFullType<'report> {
    authority: &'report type_check::CanonicalConstantTextTypeAuthority,
    _report: &'report FullTypeCheckReport,
}

impl VerifiedMinimalAddFullType<'_> {
    pub(crate) fn backend_identity(&self) -> type_check::CanonicalMinimalAddBackendIdentity<'_> {
        self.0.backend_identity()
    }

    pub(crate) fn core_prerequisite_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.0.core_prerequisite_names()
    }
}

struct FullTypeItem {
    id: String,
    kind: &'static str,
    name: String,
    span: Span,
    status: &'static str,
    statements: Vec<TypedStatement>,
}

struct TypedStatement {
    id: String,
    span: Span,
    statement_kind: &'static str,
    expression_text: Option<String>,
    expected_type: Option<String>,
    actual_type: Option<String>,
    type_source: Option<&'static str>,
    status: &'static str,
    reason: Option<&'static str>,
    failure_form: Option<&'static str>,
    callee: Option<String>,
    callee_result_root: Option<String>,
    caller_result_root: Option<String>,
    wrapper_root: Option<String>,
    call_span: Option<Span>,
    callee_span: Option<Span>,
    caller_span: Option<Span>,
    diagnostic_code: Option<&'static str>,
    help: Option<String>,
    prior_blocker: Option<crate::diagnostic::PriorBlockerRef>,
    diagnostic_occurrence: Option<DiagnosticOccurrence>,
}

#[derive(Debug, Clone)]
struct TypeFact {
    type_text: String,
    source: &'static str,
}

// WO30 Item 2: H0622/H0626/H0632 retired. Their arity/type reasons moved to
// the general H0640/H0641 probe; the per-builtin probe structs below are
// removed. `text_split` keeps its value-level H0636 probe (narrowed).

struct TextSplitTypeIssue {
    call_source: String,
    call_span: Span,
    actual_type: Option<TypeFact>,
    reason: &'static str,
}

struct InvalidTextEscapeIssue {
    offending_span: Span,
    spelling: String,
}

/// Walks a statement's canonical expressions for a text literal whose escape
/// sequences failed decision-0022 decoding. The parser marks the literal
/// Unsupported with an InvalidTextEscape malformed completion; the checker
/// surfaces it here as H0638.
fn invalid_text_escape_issue(
    parsed: &crate::ast::ParsedBodyStatement,
) -> Option<InvalidTextEscapeIssue> {
    fn walk(
        expression: &crate::ast::CanonicalExpression,
    ) -> Option<&crate::ast::CanonicalMalformedEvent> {
        if let crate::ast::CanonicalCompletionEvent::Unsupported(event) = &expression.completion
            && event.cause == crate::ast::CanonicalMalformedCause::InvalidTextEscape
        {
            return Some(event);
        }
        let children: Vec<&crate::ast::CanonicalExpression> = match &expression.kind {
            crate::ast::CanonicalExpressionKind::Field { base, .. } => vec![base],
            crate::ast::CanonicalExpressionKind::ElementPlace { base, .. } => vec![base],
            crate::ast::CanonicalExpressionKind::ListLiteral(items) => items.iter().collect(),
            crate::ast::CanonicalExpressionKind::RecordLiteral { fields, .. } => {
                fields.iter().map(|(_, value)| value).collect()
            }
            crate::ast::CanonicalExpressionKind::Call { callee, arguments } => {
                std::iter::once(callee.as_ref())
                    .chain(arguments.iter())
                    .collect()
            }
            crate::ast::CanonicalExpressionKind::Permission { value, .. } => vec![value],
            crate::ast::CanonicalExpressionKind::Try { value, .. } => vec![value],
            crate::ast::CanonicalExpressionKind::Binary { left, right, .. } => vec![left, right],
            crate::ast::CanonicalExpressionKind::Group(inner) => vec![inner],
            _ => Vec::new(),
        };
        children.into_iter().find_map(walk)
    }

    let expressions: Vec<&crate::ast::ParsedExpression> = match &parsed.kind {
        crate::ast::ParsedBodyStatementKind::Return(expression) => vec![expression],
        crate::ast::ParsedBodyStatementKind::Binding { value, .. } => value.iter().collect(),
        crate::ast::ParsedBodyStatementKind::Other { expressions } => expressions.iter().collect(),
    };
    expressions.into_iter().find_map(|expression| {
        walk(&expression.canonical).map(|event| {
            let spelling = match &event.actual {
                crate::ast::CanonicalActualLexicalEvidence::Token { spelling, .. } => {
                    spelling.clone()
                }
                _ => String::new(),
            };
            InvalidTextEscapeIssue {
                offending_span: event.offending.start.clone(),
                spelling,
            }
        })
    })
}

pub fn full_type_check_has_errors(program: &Program, diagnostics: &[Diagnostic]) -> bool {
    full_type_check_summary(program, diagnostics).blocking_issues > 0
}

pub fn full_type_check_has_only_predicate_errors(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> bool {
    let report = build_report(program, diagnostics);
    report.rejected_predicates() > 0
        && report.source_errors == 0
        && report.type_check_summary.resolver_errors == 0
        && report.type_check_summary.type_errors == 0
        && report.core_verify_summary.failed_checks == 0
        && report.rejected_statements() == 0
        && report.unchecked_statements() == 0
        && report.unsupported_statements() == 0
}

pub fn full_type_check_summary(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> FullTypeCheckSummary {
    let report = build_report(program, diagnostics);
    summary_from_report(&report)
}

fn summary_from_report(report: &FullTypeCheckReport) -> FullTypeCheckSummary {
    FullTypeCheckSummary {
        schema: FULL_TYPE_CHECK_SCHEMA,
        status: report.status(),
        mode: FULL_TYPE_CHECK_MODE,
        source_errors: report.source_errors,
        resolver_errors: report.type_check_summary.resolver_errors,
        type_errors: report.type_check_summary.type_errors,
        core_verify_errors: report.core_verify_summary.failed_checks,
        items: report.item_count(),
        body_items: report.items.len(),
        statements: report.statement_count(),
        checked_statements: report.checked_statements(),
        accepted_statements: report.accepted_statements(),
        rejected_statements: report.rejected_statements(),
        unchecked_statements: report.unchecked_statements(),
        unsupported_statements: report.unsupported_statements(),
        blocking_issues: report.blocking_issues(),
        execution_ready: 0,
        ir_ready: 0,
    }
}

pub(crate) fn validate_typed_failure_prior_blockers(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> Result<(), crate::diagnostic::DiagnosticInvariantError> {
    let analysis = typed_failure::analyze_program(program);
    let mut collector = crate::diagnostic::DiagnosticOccurrenceCollector::default();
    for occurrence in analysis.occurrences() {
        collector.insert(occurrence)?;
    }
    let report = build_report(program, diagnostics);
    for prior in report
        .items
        .iter()
        .flat_map(|item| item.statements.iter())
        .filter_map(|statement| statement.prior_blocker.as_ref())
    {
        collector.validate_prior(prior)?;
    }
    Ok(())
}

pub fn full_type_check_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum full type check ({FULL_TYPE_CHECK_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {FULL_TYPE_CHECK_MODE}\n"));
    out.push_str(&format!("status: {}\n", report.status()));
    out.push_str(&format!(
        "dependencies: core_contract={} type_check={} core_verify={}\n",
        core_contract::CORE_CONTRACT_SCHEMA,
        type_check::TYPE_CHECK_SCHEMA,
        core_verify::CORE_VERIFY_SCHEMA
    ));
    out.push_str(&format!(
        "summary: files={} items={} body_items={} statements={} checked_statements={} accepted_statements={} rejected_statements={} unchecked_statements={} unsupported_statements={} blocking_issues={} source_errors={} resolver_errors={} type_errors={} core_verify_errors={} execution_ready=0 ir_ready=0\n",
        report.files(),
        report.item_count(),
        report.items.len(),
        report.statement_count(),
        report.checked_statements(),
        report.accepted_statements(),
        report.rejected_statements(),
        report.unchecked_statements(),
        report.unsupported_statements(),
        report.blocking_issues(),
        report.source_errors,
        report.type_check_summary.resolver_errors,
        report.type_check_summary.type_errors,
        report.core_verify_summary.failed_checks
    ));

    if report.items.is_empty() {
        out.push_str("typed_items: none\n");
    } else {
        out.push_str("typed_items:\n");
        for item in &report.items {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} `{}` statements={}\n",
                item.span.file,
                item.span.line,
                item.span.column,
                item.status,
                item.kind,
                item.name,
                item.statements.len()
            ));
            for statement in &item.statements {
                out.push_str(&format!(
                    "    {}:{}:{} [{}] {}",
                    statement.span.file,
                    statement.span.line,
                    statement.span.column,
                    statement.status,
                    statement.statement_kind
                ));
                if let Some(expression) = &statement.expression_text {
                    out.push_str(&format!(" `{expression}`"));
                }
                out.push_str(&format!(
                    " expected={} actual={}",
                    statement.expected_type.as_deref().unwrap_or("none"),
                    statement.actual_type.as_deref().unwrap_or("unknown")
                ));
                if let Some(reason) = statement.reason {
                    out.push_str(&format!(" reason={reason}"));
                }
                if let Some(code) = statement.diagnostic_code {
                    out.push_str(&format!(" diagnostic={code}"));
                }
                if let Some(form) = statement.failure_form {
                    out.push_str(&format!(" failure_form={form}"));
                }
                if let Some(root) = &statement.callee_result_root {
                    out.push_str(&format!(" callee_root={root}"));
                }
                if let Some(root) = &statement.caller_result_root {
                    out.push_str(&format!(" caller_root={root}"));
                }
                if let Some(help) = &statement.help {
                    out.push_str(&format!(" help={help}"));
                }
                out.push('\n');
            }
        }
    }

    if report.predicates.is_empty() {
        out.push_str("predicate_facts: none\n");
    } else {
        out.push_str("predicate_facts:\n");
        for fact in &report.predicates {
            out.push_str(&format!(
                "  {}:{}:{} [{}] task=`{}` section={} reason={} text=`{}`",
                fact.line_span.file.replace('\\', "/"),
                fact.line_span.line,
                fact.line_span.column,
                fact.status.as_str(),
                fact.task,
                fact.section,
                fact.reason,
                fact.text
            ));
            for place in &fact.places {
                out.push_str(&format!(
                    " place=`{}` place_span={}:{}:{} scope={} definition={} root_definition={} resolution={} eligibility={} type={}",
                    place.text,
                    place.span.file.replace('\\', "/"),
                    place.span.line,
                    place.span.column,
                    place.scope_id,
                    place.definition_id.as_deref().unwrap_or("none"),
                    place.root_definition_id.as_deref().unwrap_or("none"),
                    place.resolution,
                    place.eligibility,
                    place.type_text.as_deref().unwrap_or("unknown")
                ));
            }
            if let Some(expected) = &fact.expected {
                out.push_str(&format!(" expected={expected}"));
            }
            if let Some(actual) = &fact.actual {
                out.push_str(&format!(" actual={actual}"));
            }
            if let Some(operator) = fact.comparison {
                out.push_str(&format!(" operator={operator}"));
            }
            if let (Some(left), Some(right)) = (&fact.left_type, &fact.right_type) {
                out.push_str(&format!(" left_type={left} right_type={right}"));
            }
            if let Some(diagnostic) = fact.diagnostic() {
                out.push_str(&format!(
                    " diagnostic={} help={}",
                    diagnostic.code.as_str(),
                    fact.repair()
                ));
            }
            if let Some(span) = &fact.intent_span {
                out.push_str(&format!(
                    " intent_span={}:{}:{}",
                    span.file, span.line, span.column
                ));
            }
            if let Some(span) = &fact.offending_span {
                out.push_str(&format!(
                    " offending_span={}:{}:{}",
                    span.file, span.line, span.column
                ));
            }
            out.push('\n');
        }
    }

    out.push_str("non_claims:\n");
    for non_claim in NON_CLAIMS {
        out.push_str(&format!("  - {non_claim}\n"));
    }

    out
}

pub fn full_type_check_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", FULL_TYPE_CHECK_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", FULL_TYPE_CHECK_MODE, true);
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
        "type_check_schema",
        type_check::TYPE_CHECK_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "core_verify_schema",
        core_verify::CORE_VERIFY_SCHEMA,
        true,
    );
    push_dependency_summaries(&mut out, &report, 2, true);
    push_summary(&mut out, &report, 2, true);
    push_items(&mut out, &report.items, 2, true);
    push_predicates(&mut out, &report.predicates, 2, true);
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> FullTypeCheckReport {
    build_report_with(program, diagnostics, |_| ()).0
}

fn build_report_with<R>(
    program: &Program,
    diagnostics: &[Diagnostic],
    consume: impl for<'report> FnOnce(FullTypeEffectReportAccess<'report>) -> R,
) -> (FullTypeCheckReport, R) {
    let type_check_summary = type_check::type_check_summary(program, diagnostics);
    let callables = callable::analyze_program(program);
    let source_errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let task_returns = task_return_types(program);
    let task_signatures = task_signatures(program);
    let failure_analysis = typed_failure::analyze_program(program);
    let field_types = field_place::collect_field_types(program);
    let predicates = predicate::analyze_program(program);
    #[cfg(test)]
    let mut core_verify_report_identity = 0;
    let (handoff, (report, result)) =
        core_verify::with_core_verify_for_full_type(program, diagnostics, |core_verify_access| {
            #[cfg(test)]
            {
                core_verify_report_identity = core_verify_access.report_identity_for_test();
                assert_eq!(
                    core_verify_report_identity,
                    core_verify_access
                        .diagnostic_occurrences()
                        .report_identity_for_test()
                );
            }
            let core_verify_summary = core_verify_access.readiness_summary();
            let blocked = source_errors > 0
                || type_check_summary.resolver_errors > 0
                || type_check_summary.type_errors > 0
                || core_verify_summary.failed_checks > 0;
            let diagnostic_access = core_verify_access.diagnostic_occurrences();
            let _ = diagnostic_access.occurrences().count();
            let mut items = Vec::new();
            let context = FullTypeCollectionContext {
                program,
                blocked,
                failure_analysis: &failure_analysis,
                field_types: &field_types,
                callables: &callables,
                core_verify_access: &core_verify_access,
            };
            for file in &program.files {
                collect_items(
                    &context,
                    &file.items,
                    &task_returns,
                    &task_signatures,
                    &mut items,
                );
            }
            let mut diagnostic_occurrences = core_verify_access.diagnostic_occurrence_set().clone();
            extend_full_type_occurrences(&failure_analysis, &items, &mut diagnostic_occurrences);
            #[allow(unused_mut)]
            let mut report = FullTypeCheckReport {
                type_check_summary,
                core_verify_summary: core_verify_summary.clone(),
                items,
                files: program.files.len(),
                item_count: count_items(program),
                source_errors,
                predicates: predicates.facts().to_vec(),
                diagnostic_occurrences,
            };
            #[cfg(test)]
            corrupt_wo19_report_for_test(&mut report);
            let result = consume(FullTypeEffectReportAccess {
                program,
                report: &report,
                core_verify: &core_verify_access,
            });
            (report, result)
        });
    #[cfg(test)]
    assert_eq!(
        core_verify_report_identity,
        handoff.report_identity_for_test()
    );
    let (core_verify_summary, _core_diagnostics) = handoff.into_parts();
    debug_assert_eq!(report.core_verify_summary, core_verify_summary);
    (report, result)
}

#[cfg(test)]
fn corrupt_wo19_report_for_test(report: &mut FullTypeCheckReport) {
    let Some(kind) = type_check::take_wo19_stage_corruption("full_type_check") else {
        return;
    };
    match kind {
        "missing" => {
            if let Some(item) = report.items.first_mut() {
                item.statements.clear()
            }
        }
        "global" => report.source_errors = 1,
        kind => {
            let Some(row) = report
                .items
                .first_mut()
                .and_then(|item| item.statements.first_mut())
            else {
                return;
            };
            match kind {
                "rejected" => row.status = "rejected_statement_type_v0",
                "unchecked" => row.status = "unchecked_statement_type_v0",
                "foreign" => row.span.file = "foreign.hum".to_string(),
                "fabricated" => row.type_source = Some("fabricated_public_type_v0"),
                _ => report.items.clear(),
            }
        }
    }
}

pub(crate) fn with_full_type_for_effect<R>(
    program: &Program,
    diagnostics: &[Diagnostic],
    consume: impl for<'report> FnOnce(FullTypeEffectReportAccess<'report>) -> R,
) -> R {
    build_report_with(program, diagnostics, consume).1
}

impl<'report> FullTypeEffectReportAccess<'report> {
    pub(crate) fn report_parts(&self) -> (FullTypeCheckSummary, &'report DiagnosticOccurrenceSet) {
        (
            summary_from_report(self.report),
            &self.report.diagnostic_occurrences,
        )
    }

    pub(crate) fn canonical_minimal_add_for(
        &self,
        item: &Item,
        statement: &crate::ast::ParsedBodyStatement,
    ) -> Option<VerifiedMinimalAddFullType<'report>> {
        let core_verify::CanonicalMinimalAddTypeLookup::Delivered(verified_type) = self
            .core_verify
            .canonical_minimal_add_type_for(item, statement)
        else {
            return None;
        };
        (self.report.blocking_issues() == 0
            && std::ptr::from_ref(self.program).addr()
                == verified_type.backend_identity().program_identity)
            .then_some(())?;
        let item_row = unique(self.report.items.iter().filter(|row| {
            row.kind == item.kind()
                && row.name == item.name()
                && row.span == portable_span(item.span())
        }))?;
        let statement_row = unique(
            item_row
                .statements
                .iter()
                .filter(|row| row.span == portable_span(&statement.span)),
        )?;
        (statement_row.statement_kind == "return"
            && statement_row.actual_type.as_deref() == Some("Int")
            && statement_row.type_source == Some("verified_canonical_minimal_add_type_v0")
            && statement_row.status == "accepted_statement_type_v0"
            && statement_row.reason.is_none())
        .then_some(VerifiedMinimalAddFullType(verified_type))
    }

    pub(crate) fn canonical_integer_sign_for(
        &self,
        layout: &crate::app_entry::CanonicalNativeLayout<'_>,
        authority: &'report type_check::CanonicalIntegerSignTypeAuthority,
    ) -> Option<VerifiedIntegerSignFullType<'report>> {
        (self.report.blocking_issues() == 0 && authority.matches(self.program, layout)).then_some(
            VerifiedIntegerSignFullType {
                authority,
                _report: self.report,
            },
        )
    }

    pub(crate) fn canonical_constant_text_for(
        &self,
        layout: &crate::app_entry::CanonicalNativeLayout<'_>,
        authority: &'report type_check::CanonicalConstantTextTypeAuthority,
    ) -> Option<VerifiedConstantTextFullType<'report>> {
        (self.report.blocking_issues() == 0 && authority.matches(self.program, layout)).then_some(
            VerifiedConstantTextFullType {
                authority,
                _report: self.report,
            },
        )
    }
}

impl VerifiedIntegerSignFullType<'_> {
    pub(crate) fn authority(&self) -> &type_check::CanonicalIntegerSignTypeAuthority {
        self.authority
    }
}

impl VerifiedConstantTextFullType<'_> {
    pub(crate) fn authority(&self) -> &type_check::CanonicalConstantTextTypeAuthority {
        self.authority
    }
}

fn unique<I: Iterator>(mut values: I) -> Option<I::Item> {
    let value = values.next()?;
    values.next().is_none().then_some(value)
}

fn extend_full_type_occurrences(
    failure_analysis: &typed_failure::ProgramFailureAnalysis,
    items: &[FullTypeItem],
    diagnostic_occurrences: &mut DiagnosticOccurrenceSet,
) {
    for occurrence in failure_analysis
        .occurrences()
        .into_iter()
        .filter(|occurrence| occurrence.owning_stage() == "full_type_check")
    {
        diagnostic_occurrences
            .insert_owned(occurrence)
            .expect("typed-failure occurrences must remain unique at full type");
    }
    for occurrence in items
        .iter()
        .flat_map(|item| &item.statements)
        .filter_map(|statement| statement.diagnostic_occurrence.clone())
    {
        diagnostic_occurrences
            .insert_owned(occurrence)
            .expect("built-in call occurrences must be unique");
    }
}

#[allow(dead_code)]
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
) -> Result<DiagnosticOccurrenceSet, crate::diagnostic::DiagnosticInvariantError> {
    let report = build_report(program, diagnostics);
    let mut occurrences = core_verify::validate_diagnostic_projection_from_source(
        program,
        diagnostics,
        source_occurrences,
    )?;
    let failure_analysis = typed_failure::analyze_program(program);
    extend_full_type_occurrences(&failure_analysis, &report.items, &mut occurrences);
    Ok(occurrences)
}

struct FullTypeCollectionContext<'a, 'report> {
    program: &'a Program,
    blocked: bool,
    failure_analysis: &'a ProgramFailureAnalysis,
    field_types: &'a FieldTypeMap,
    callables: &'a CallableAnalysis,
    core_verify_access: &'a core_verify::CoreVerifyFullTypeReportAccess<'report>,
}

fn collect_items(
    context: &FullTypeCollectionContext<'_, '_>,
    items: &[Item],
    task_returns: &BTreeMap<String, TypeFact>,
    task_signatures: &BTreeMap<String, TaskSignature>,
    out: &mut Vec<FullTypeItem>,
) {
    for item in items {
        if let Some(typed_item) = type_item(
            context.program,
            item,
            context.blocked,
            task_returns,
            task_signatures,
            context.failure_analysis,
            context.field_types,
            context.callables,
            context.core_verify_access,
        ) {
            out.push(typed_item);
        }
        if let Item::App(app) = item {
            let app_task_returns = task_return_types_from_items(&app.items);
            let app_task_signatures = task_signatures_from_items(&app.items);
            collect_items(
                context,
                &app.items,
                &app_task_returns,
                &app_task_signatures,
                out,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn type_item(
    program: &Program,
    item: &Item,
    blocked: bool,
    task_returns: &BTreeMap<String, TypeFact>,
    task_signatures: &BTreeMap<String, TaskSignature>,
    failure_analysis: &ProgramFailureAnalysis,
    field_types: &FieldTypeMap,
    callables: &CallableAnalysis,
    core_verify_access: &core_verify::CoreVerifyFullTypeReportAccess<'_>,
) -> Option<FullTypeItem> {
    let item_identity = crate::resolve::semantic_item_identity_for(program, item);
    let does = item_sections(item)
        .iter()
        .find(|section| section.name == "does")?;
    let body = core_body::analyze_does_section(
        program
            .canonical_core_expectation(item, does)
            .expect("live typed item must have parser authority"),
    );
    let mut scopes = TypeScopeStack::new(initial_environment(item_params(item)));
    let mut statements = Vec::new();
    for (index, (statement, parsed)) in body
        .statements
        .iter()
        .zip(does.body_syntax.iter().flatten())
        .enumerate()
    {
        // WO28 #16: block scoping must match the resolver. Push a scope for
        // block openers before typing the statement, so the for-each binder
        // and block-local lets land in the pushed scope; pop on block_close.
        scopes.handle_block_boundary(statement.kind);
        let typed = type_statement(
            &item_identity,
            item,
            index,
            statement,
            &mut scopes,
            task_returns,
            task_signatures,
            field_types,
            blocked,
            match item {
                Item::Task(task) => failure_analysis.fact(task, index),
                _ => None,
            },
            callables,
            core_verify_access,
            parsed,
        );
        statements.push(typed);
    }
    let status = item_status(&statements, blocked);
    Some(FullTypeItem {
        id: prefixed_id(
            "hum_full_type_item",
            &format!("{}_{}_{}", item.kind(), item.name(), item.span().line),
        ),
        kind: item.kind(),
        name: item.name().to_string(),
        span: portable_span(item.span()),
        status,
        statements,
    })
}

#[allow(clippy::too_many_arguments)]
fn type_statement(
    item_identity: &str,
    item: &Item,
    index: usize,
    statement: &BodyStatement,
    scopes: &mut TypeScopeStack<TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    task_signatures: &BTreeMap<String, TaskSignature>,
    field_types: &FieldTypeMap,
    blocked: bool,
    failure_fact: Option<&FailureFact>,
    callables: &CallableAnalysis,
    core_verify_access: &core_verify::CoreVerifyFullTypeReportAccess<'_>,
    parsed: &crate::ast::ParsedBodyStatement,
) -> TypedStatement {
    if blocked {
        return typed_statement(
            statement,
            index,
            None,
            None,
            None,
            "not_checked_blocked_by_prior_errors_v0",
            Some("source_resolver_type_or_core_verify_errors"),
        );
    }

    if statement.status == "unsupported_v0" {
        return typed_statement(
            statement,
            index,
            None,
            None,
            None,
            "blocked_unsupported_statement_v0",
            statement
                .reason
                .or(Some("statement_not_in_core_body_grammar_v0")),
        );
    }

    if let Some(issue) = invalid_text_escape_issue(parsed) {
        let mut typed = typed_statement(
            statement,
            index,
            None,
            Some("Text".to_string()),
            None,
            "rejected_invalid_text_escape_v0",
            Some("text_literal_has_invalid_escape_sequence_v0"),
        );
        typed.call_span = Some(issue.offending_span);
        typed.caller_span = Some(item.span().clone());
        typed.diagnostic_code = Some(DiagnosticCode::INVALID_TEXT_ESCAPE.as_str());
        typed.help = Some(format!(
            "Replace `{}` with one of the accepted escapes (`\\n`, `\\t`, `\\\\`, `\\\"`).",
            issue.spelling
        ));
        attach_builtin_occurrence(
            &mut typed,
            item_identity,
            index,
            DiagnosticCode::INVALID_TEXT_ESCAPE,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(185),
            "text_literal_escape_shape",
        );
        return typed;
    }

    // WO30 Item 4: the general call-shape probe runs before the per-builtin
    // shape probes so migrated codes cannot double-fire. A statement the
    // probe rejects never reaches the narrower probes below.
    if let Some(issue) = call_shape_issue(parsed, task_signatures) {
        let expected = issue.signature.params.len();
        // WO30 Item 2 (Claude review): `expected` is the type the statement
        // expects, never the callee's return type. The signature already
        // appears in the help text.
        let statement_expected = expected_type_for_statement(item, statement, scopes, field_types);
        let mut typed = typed_statement(
            statement,
            index,
            expression_text_for_statement(statement).map(str::to_string),
            statement_expected,
            None,
            "rejected_invalid_call_arity_v0",
            Some("call_argument_count_mismatch_v0"),
        );
        if issue.is_builtin {
            // WO30 Item 2: the migrated builtins keep the failure forms
            // their retired per-builtin diagnostics carried.
            typed.failure_form = match issue.callee.as_str() {
                "text_split" => Some("text_split_builtin"),
                "stdout_write" => Some("bounded_output_builtin"),
                "clock_replay_tick" => Some("runner_replay_builtin"),
                "files_read_text" => Some("hardened_exact_file_read_builtin"),
                _ => None,
            };
        }
        typed.call_span = Some(issue.call_span);
        typed.caller_span = Some(item.span().clone());
        typed.diagnostic_code = Some(DiagnosticCode::INVALID_CALL_ARITY.as_str());
        typed.help = Some(format!(
            "Call `{}` with exactly {} argument{} ({}); found {}.",
            issue.callee,
            expected,
            if expected == 1 { "" } else { "s" },
            signature_help_text(&issue.callee, &issue.signature),
            issue.actual_args,
        ));
        attach_builtin_occurrence(
            &mut typed,
            item_identity,
            index,
            DiagnosticCode::INVALID_CALL_ARITY,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(188),
            "call_shape",
        );
        return typed;
    }

    // WO30 Item 2: the argument-type probe runs after the arity probe, so
    // arity mismatches keep H0640 precedence. The first call with a
    // statically known argument type that mismatches its parameter type is
    // H0641; unknown argument types stay silent.
    if let Some(issue) =
        call_argument_type_issue(parsed, task_signatures, scopes, task_returns, field_types)
    {
        let actual_name = canonical_argument_type_name(&issue.actual_type);
        let mut typed = typed_statement(
            statement,
            index,
            expression_text_for_statement(statement).map(str::to_string),
            Some(issue.expected_type.clone()),
            Some(type_fact(&actual_name, "call_argument_type_v0")),
            "rejected_invalid_call_argument_type_v0",
            Some("call_argument_type_mismatch_v0"),
        );
        if issue.is_builtin {
            typed.failure_form = match issue.callee.as_str() {
                "text_split" => Some("text_split_builtin"),
                "stdout_write" => Some("bounded_output_builtin"),
                "clock_replay_tick" => Some("runner_replay_builtin"),
                "files_read_text" => Some("hardened_exact_file_read_builtin"),
                _ => None,
            };
        }
        typed.call_span = Some(issue.call_span);
        typed.caller_span = Some(item.span().clone());
        typed.diagnostic_code = Some(DiagnosticCode::INVALID_CALL_ARGUMENT_TYPE.as_str());
        typed.help = Some(format!(
            "Call `{}` with `{}` for argument {} ({}); found `{}`.",
            issue.callee,
            issue.expected_type,
            issue.argument_index + 1,
            signature_help_text(&issue.callee, &issue.signature),
            actual_name,
        ));
        attach_builtin_occurrence(
            &mut typed,
            item_identity,
            index,
            DiagnosticCode::INVALID_CALL_ARGUMENT_TYPE,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(191),
            "call_argument_type",
        );
        return typed;
    }

    // WO30 Item 2: narrowed to value-level H0636 reasons only (stray empty
    // argument, directly written empty separator); arity/type reasons moved
    // to the general H0640/H0641 probe above.
    if let Some(issue) = text_split_type_issue(statement) {
        let mut typed = typed_statement(
            statement,
            index,
            Some(issue.call_source),
            Some("List Text".to_string()),
            issue.actual_type,
            "rejected_invalid_text_split_call_v0",
            Some(issue.reason),
        );
        typed.failure_form = Some("text_split_builtin");
        typed.call_span = Some(issue.call_span);
        typed.caller_span = Some(item.span().clone());
        typed.diagnostic_code = Some(DiagnosticCode::INVALID_TEXT_SPLIT_CALL.as_str());
        typed.help = Some(
            "Pass no stray empty arguments and a non-empty separator to `text_split`, then handle its `TextSplitError` explicitly unless the separator is a directly-written non-empty literal."
                .to_string(),
        );
        attach_builtin_occurrence(
            &mut typed,
            item_identity,
            index,
            DiagnosticCode::INVALID_TEXT_SPLIT_CALL,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(183),
            "text_split_call_shape",
        );
        return typed;
    }

    if let Some(binding_name) = constant_text_stdout_write_binding(parsed) {
        let actual = type_fact("Unit", "constant_text_stdout_write_success_v0");
        scopes.insert(binding_name, actual.clone());
        let mut typed = typed_statement(
            statement,
            index,
            expression_text_for_statement(statement).map(str::to_string),
            None,
            Some(actual),
            "accepted_constant_text_stdout_write_v0",
            None,
        );
        typed.failure_form = Some("bounded_output_builtin");
        typed.caller_span = Some(item.span().clone());
        return typed;
    }

    if let Some(fact) = failure_fact {
        let effect_owned_missing_declaration = fact.diagnostic_code
            == Some(crate::diagnostic::DiagnosticCode::MISSING_FAILURE_DECLARATION);
        let actual = fact
            .success_type
            .as_ref()
            .map(|type_text| type_fact(type_text, "typed_failure_success_v0"));
        let expected_type = if statement.kind == "fail" {
            fact.caller_result_root.clone()
        } else {
            binding_annotation(statement)
        };
        let mut typed = typed_statement(
            statement,
            index,
            expression_text_for_statement(statement).map(str::to_string),
            expected_type,
            actual.clone(),
            if effect_owned_missing_declaration {
                "accepted_typed_failure_deferred_to_effect_v0"
            } else {
                fact.status
            },
            if effect_owned_missing_declaration {
                Some("fails_when_declaration_deferred_to_effect_v0")
            } else {
                fact.reason
            },
        );
        apply_failure_fact(&mut typed, fact);
        if effect_owned_missing_declaration {
            typed.diagnostic_code = None;
            typed.help = None;
        }
        if matches!(statement.kind, "let_binding" | "mutable_binding")
            && (fact.diagnostic_code.is_none() || effect_owned_missing_declaration)
            && let Some((name, type_fact)) = binding_type_fact(statement, actual.as_ref())
        {
            scopes.insert(&name, type_fact);
        }
        return typed;
    }

    // WO28 #13: a `for each` header over `List T` binds its loop variable
    // as `T`. Every unprovable shape fails closed into the generic path,
    // which keeps the current unchecked status.
    if statement.kind == "for_each_header"
        && let Some((binder, element_type)) =
            for_each_binding(statement, scopes, task_returns, field_types)
    {
        let actual = type_fact(element_type.clone(), "for_each_binding_v0");
        scopes.insert(&binder, actual.clone());
        return typed_statement(
            statement,
            index,
            expression_text_for_statement(statement).map(str::to_string),
            None,
            Some(actual),
            "accepted_for_each_binding_v0",
            None,
        );
    }

    // WO28 #13: a test expectation proves when its call targets a known
    // task and the expected value's type is compatible with the task's
    // return type. Unprovable or mismatched shapes fail closed into the
    // generic path, which keeps the current unchecked status.
    if statement.kind == "test_expectation"
        && let Some(fact) = test_expectation_fact(statement, task_returns, scopes, field_types)
    {
        return typed_statement(
            statement,
            index,
            expression_text_for_statement(statement).map(str::to_string),
            None,
            Some(fact),
            "accepted_test_expectation_v0",
            None,
        );
    }

    let expression_text = expression_text_for_statement(statement).map(str::to_string);
    let expected_type = expected_type_for_statement(item, statement, scopes, field_types);
    let verified_actual = if let core_verify::CanonicalMinimalAddTypeLookup::Delivered(result) =
        core_verify_access.canonical_minimal_add_type_for(item, parsed)
    {
        let (_, _, type_text, _, provenance, declared_result_compatible) = result.facts();
        debug_assert_eq!(
            declared_result_compatible,
            expected_type
                .as_deref()
                .map(|expected| expected == type_text)
        );
        Some(type_fact(type_text, provenance))
    } else {
        None
    };
    let callable_actual = match item {
        Item::Task(task) => callables
            .indirect_application(task, &statement.span)
            .map(|fact| {
                type_fact(
                    &fact.result_type,
                    "checked_indirect_callable_application_v0",
                )
            }),
        _ => None,
    };
    let actual = verified_actual.or(callable_actual).or_else(|| {
        expression_text.as_deref().and_then(|expression| {
            infer_expression_type(expression, scopes, task_returns, field_types)
        })
    });
    let (status, reason) = statement_status(statement, expected_type.as_deref(), actual.as_ref());

    if matches!(statement.kind, "let_binding" | "mutable_binding")
        && let Some((name, fact)) = binding_type_fact(statement, actual.as_ref())
    {
        scopes.insert(&name, fact);
    }

    typed_statement(
        statement,
        index,
        expression_text,
        expected_type,
        actual,
        status,
        reason,
    )
}

fn for_each_binding(
    statement: &BodyStatement,
    scopes: &TypeScopeStack<TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<(String, String)> {
    let header = header_body(&statement.text, "for each")?;
    let (binder, iterated) = header.split_once(" in ")?;
    let binder = binder.trim();
    let iterated = iterated.trim();
    if !element_place::is_value_ident(binder) || iterated.is_empty() {
        return None;
    }
    let iterated_fact = infer_expression_type(iterated, scopes, task_returns, field_types)?;
    let element = element_place::list_element_type(&iterated_fact.type_text)?;
    Some((binder.to_string(), element.to_string()))
}

fn test_expectation_fact(
    statement: &BodyStatement,
    task_returns: &BTreeMap<String, TypeFact>,
    scopes: &TypeScopeStack<TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<TypeFact> {
    let body = strip_keyword(&statement.text, "expect")?;
    let (call, expected) = body.split_once(" returns ")?;
    let (callee, _args) = split_call(call)?;
    let return_fact = task_returns.get(&name_key(callee))?;
    let expected_fact = infer_expression_type(expected.trim(), scopes, task_returns, field_types)?;
    if types_compatible(&return_fact.type_text, &expected_fact.type_text) {
        Some(type_fact("Bool", "test_expectation_v0"))
    } else {
        None
    }
}

fn constant_text_stdout_write_binding(statement: &crate::ast::ParsedBodyStatement) -> Option<&str> {
    let crate::ast::ParsedBodyStatementKind::Binding {
        mutable: false,
        name: Some(name),
        value: Some(value),
    } = &statement.kind
    else {
        return None;
    };
    let crate::ast::CanonicalExpressionKind::Try { value, .. } = &value.canonical.kind else {
        return None;
    };
    let crate::ast::CanonicalExpressionKind::Call { callee, arguments } = &value.kind else {
        return None;
    };
    let [argument] = arguments.as_slice() else {
        return None;
    };
    (matches!(&callee.kind, crate::ast::CanonicalExpressionKind::Identifier(callee) if callee == "stdout_write")
        && matches!(&argument.kind, crate::ast::CanonicalExpressionKind::TextLiteral(_)))
    .then_some(name.name.as_str())
}

/// WO30 Item 2: narrowed to value-level H0636 reasons only. Arity and
/// argument-type reasons moved to the general H0640/H0641 probe, which runs
/// before this probe — so a `text_split` call seen here already has exactly
/// two arguments. Retained: stray empty argument, directly written empty
/// separator literal.
fn text_split_type_issue(statement: &BodyStatement) -> Option<TextSplitTypeIssue> {
    let expression = expression_text_for_statement(statement)?;
    let expression_offset = statement.text.find(expression).unwrap_or(0);
    let call = typed_failure::calls_in_expression(expression)
        .into_iter()
        .find(|call| call.callee == "text_split")?;
    let call_span = Span {
        file: statement.span.file.clone(),
        line: statement.span.line,
        column: statement.span.column
            + statement.text[..expression_offset + call.source_offset]
                .chars()
                .count(),
    };
    let args = call.source.strip_prefix("text_split(")?.strip_suffix(')')?;
    let arguments = typed_failure::split_call_arguments(args);
    // Decision 0022: a stray empty argument (e.g. `text_split(line,, ",")`)
    // is a checker error, not a silently-dropped segment.
    if typed_failure::has_stray_empty_argument(args) {
        return Some(TextSplitTypeIssue {
            call_source: call.source,
            call_span,
            actual_type: None,
            reason: "text_split_rejects_stray_empty_argument_v0",
        });
    }
    // A literal empty separator is a checker error (decision 0021). Only a
    // directly-written `""` is caught here; a runtime-computed empty
    // separator raises `TextSplitError.SepEmpty` through `try`/`fail`.
    // The index is guarded: the H0640 probe runs before this probe, so a
    // call seen here has exactly two arguments — but never index blindly.
    if arguments.len() == 2 && arguments[1].trim() == "\"\"" {
        return Some(TextSplitTypeIssue {
            call_source: call.source,
            call_span,
            actual_type: None,
            reason: "text_split_separator_must_not_be_empty_literal_v0",
        });
    }
    None
}

// WO27 Part 1b: the thin `split_call_arguments` wrapper is removed; call
// sites use `typed_failure::split_call_arguments` directly.

fn expected_type_for_statement(
    item: &Item,
    statement: &BodyStatement,
    scopes: &TypeScopeStack<TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<String> {
    match statement.kind {
        "return" => item_result(item).map(expected_return_value_type),
        "fail" => item_result(item).and_then(expected_error_value_type),
        "if_header" | "while_header" => Some("Bool".to_string()),
        "let_binding" | "mutable_binding" => binding_annotation(statement),
        "set_place" => set_place_name(statement)
            .and_then(|name| place_type_fact(name, scopes, field_types))
            .map(|fact| fact.type_text),
        _ => None,
    }
}

fn statement_status(
    statement: &BodyStatement,
    expected_type: Option<&str>,
    actual: Option<&TypeFact>,
) -> (&'static str, Option<&'static str>) {
    match statement.kind {
        "block_close" | "loop_header" => ("accepted_no_expression_type_obligation_v0", None),
        "nested_intent_header" => (
            "blocked_unsupported_statement_v0",
            Some("nested_intent_lowering_not_implemented"),
        ),
        "test_expectation" => (
            "unchecked_statement_type_v0",
            Some("test_expectation_typing_not_implemented"),
        ),
        "for_each_header" | "for_index_header" => (
            "unchecked_statement_type_v0",
            Some("iterator_type_checking_not_implemented"),
        ),
        "record_field_initializer" => (
            "unchecked_statement_type_v0",
            Some("record_field_context_not_tracked_v0"),
        ),
        "return" | "fail" | "if_header" | "while_header" | "set_place" => {
            typed_expression_status(expected_type, actual)
        }
        "let_binding" | "mutable_binding" => {
            let writable_alias_candidate =
                writable_field_alias::candidate_name(statement).is_some();
            let unsupported_or_untyped_alias_candidate = writable_alias_candidate
                && (writable_field_alias::exact_binding(statement).is_none() || actual.is_none());
            if unsupported_or_untyped_alias_candidate {
                (
                    "accepted_writable_field_alias_candidate_deferred_to_ownership_v0",
                    Some("writable_field_alias_shape_deferred_to_ownership_v0"),
                )
            } else if expected_type.is_some() {
                typed_expression_status(expected_type, actual)
            } else if actual.is_some() {
                ("accepted_inferred_binding_type_v0", None)
            } else {
                (
                    "unchecked_statement_type_v0",
                    Some("binding_initializer_type_unknown_v0"),
                )
            }
        }
        _ => (
            "blocked_unsupported_statement_v0",
            statement
                .reason
                .or(Some("statement_type_rule_not_implemented")),
        ),
    }
}

fn typed_expression_status(
    expected_type: Option<&str>,
    actual: Option<&TypeFact>,
) -> (&'static str, Option<&'static str>) {
    let Some(expected_type) = expected_type else {
        return (
            "unchecked_statement_type_v0",
            Some("expected_type_context_missing_v0"),
        );
    };
    let Some(actual) = actual else {
        return (
            "unchecked_statement_type_v0",
            Some("expression_type_unknown_v0"),
        );
    };
    if types_compatible(expected_type, &actual.type_text) {
        ("accepted_statement_type_v0", None)
    } else {
        (
            "rejected_statement_type_mismatch_v0",
            Some("statement_expression_type_mismatch"),
        )
    }
}

fn typed_statement(
    statement: &BodyStatement,
    index: usize,
    expression_text: Option<String>,
    expected_type: Option<String>,
    actual: Option<TypeFact>,
    status: &'static str,
    reason: Option<&'static str>,
) -> TypedStatement {
    TypedStatement {
        id: prefixed_id(
            "hum_full_type_stmt",
            &format!("{}_{}_{}", statement.kind, statement.span.line, index),
        ),
        span: portable_span(&statement.span),
        statement_kind: statement.kind,
        expression_text,
        expected_type,
        actual_type: actual.as_ref().map(|fact| fact.type_text.clone()),
        type_source: actual.map(|fact| fact.source),
        status,
        reason,
        failure_form: None,
        callee: None,
        callee_result_root: None,
        caller_result_root: None,
        wrapper_root: None,
        call_span: None,
        callee_span: None,
        caller_span: None,
        diagnostic_code: None,
        help: None,
        prior_blocker: None,
        diagnostic_occurrence: None,
    }
}

fn attach_builtin_occurrence(
    statement: &mut TypedStatement,
    item_identity: &str,
    statement_index: usize,
    code: DiagnosticCode,
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    semantic_kind: &'static str,
) {
    let cause = crate::diagnostic_catalog::diagnostic_cause_for_key(cause_key)
        .expect("built-in semantic producer must name one registered cause");
    let semantic_origin =
        format!("full-type:{item_identity}:statement-{statement_index}:builtin-call-0");
    let identity = DiagnosticOccurrence::semantic_relationship_identity(
        cause.origin_kind,
        cause.route_kind,
        semantic_origin,
        vec![
            format!("item_identity={item_identity}"),
            format!("statement_index={statement_index}"),
            format!("semantic_call_kind={semantic_kind}"),
        ],
    );
    let diagnostic = Diagnostic::error(
        code,
        statement
            .reason
            .expect("built-in semantic producer has a detail reason"),
        statement
            .call_span
            .clone()
            .or_else(|| Some(statement.span.clone())),
    )
    .with_help(
        statement
            .help
            .clone()
            .expect("built-in semantic producer has repair text"),
    );
    statement.diagnostic_occurrence = Some(
        DiagnosticOccurrence::registered(cause, identity, diagnostic)
            .expect("built-in semantic occurrence must validate at production"),
    );
}

fn apply_failure_fact(statement: &mut TypedStatement, fact: &FailureFact) {
    statement.failure_form = Some(fact.form);
    statement.callee = fact.callee.clone();
    statement.callee_result_root = fact.callee_result_root.clone();
    statement.caller_result_root = fact.caller_result_root.clone();
    statement.wrapper_root = fact.wrapper_root.clone();
    statement.call_span = Some(fact.call_span.clone());
    statement.callee_span = fact.callee_span.clone();
    statement.caller_span = Some(fact.caller_span.clone());
    statement.diagnostic_code = fact.diagnostic_code.map(|code| code.as_str());
    statement.help = fact.help.clone();
    statement.prior_blocker = fact
        .occurrence
        .as_ref()
        .map(|occurrence| occurrence.prior_blocker());
}

fn task_return_types(program: &Program) -> BTreeMap<String, TypeFact> {
    let mut returns = session_z_builtin_return_types();
    for file in &program.files {
        collect_task_return_types(&file.items, &mut returns);
    }
    returns
}

fn task_return_types_from_items(items: &[Item]) -> BTreeMap<String, TypeFact> {
    let mut returns = session_z_builtin_return_types();
    collect_task_return_types(items, &mut returns);
    returns
}

fn session_z_builtin_return_types() -> BTreeMap<String, TypeFact> {
    BTreeMap::from([
        (
            name_key("stdout_write"),
            type_fact("Unit", "stdout_write_builtin_v0"),
        ),
        (
            name_key("clock_replay_tick"),
            type_fact("UInt", "runner_replay_builtin_v0"),
        ),
        (
            name_key("files_read_text"),
            type_fact("Text", "hardened_exact_file_read_builtin_v0"),
        ),
        (
            name_key("text_split"),
            type_fact("List Text", "text_split_builtin_v0"),
        ),
        // Decision 0028: infallible integer rendering, one builtin per
        // integer type. Both render to Text; no failure form, no `try`.
        (
            name_key("uint_to_text"),
            type_fact("Text", "uint_to_text_builtin_v0"),
        ),
        (
            name_key("int_to_text"),
            type_fact("Text", "int_to_text_builtin_v0"),
        ),
        // WO28 #13: list lengths are UInt (wordfreq_count declares -> UInt;
        // counts are UInt like clock_replay_tick above).
        (
            name_key("list_len"),
            type_fact("UInt", "list_len_builtin_v0"),
        ),
    ])
}

fn collect_task_return_types(items: &[Item], returns: &mut BTreeMap<String, TypeFact>) {
    for item in items {
        match item {
            Item::Task(task) => {
                if let Some(result) = task.result.as_deref() {
                    returns.insert(
                        name_key(&task.name),
                        type_fact(expected_return_value_type(result), "task_call_result_v0"),
                    );
                }
            }
            Item::App(app) => collect_task_return_types(&app.items, returns),
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

/// WO30 Items 1+4: a declared call signature — ordered parameter types plus
/// the declared return type. Builtins resolve before user tasks
/// (builtin-table-first precedence matches runtime dispatch, ledger #19);
/// user tasks follow the same scope traversal as `collect_task_return_types`
/// so module scope sees every file's items (including app-nested tasks)
/// while app-nested checking sees only that app's items.
#[derive(Debug, Clone)]
struct TaskSignature {
    params: Vec<String>,
    return_type: Option<String>,
}

fn builtin_task_signatures() -> BTreeMap<String, TaskSignature> {
    // WO30 Item 4: the single builtin signature table. These builtins are
    // checked exactly like user tasks with the same signatures. WO30 Item 2:
    // `stdout_write`, `clock_replay_tick`, and `files_read_text` carry their
    // parameter signatures here so their arity/type reasons are owned by the
    // general H0640/H0641 probe (H0622/H0626/H0632 retired).
    BTreeMap::from([
        (
            name_key("clock_replay_tick"),
            TaskSignature {
                params: Vec::new(),
                return_type: Some("UInt".to_string()),
            },
        ),
        (
            name_key("files_read_text"),
            TaskSignature {
                params: vec!["Path".to_string()],
                return_type: Some("Text".to_string()),
            },
        ),
        (
            name_key("uint_to_text"),
            TaskSignature {
                params: vec!["UInt".to_string()],
                return_type: Some("Text".to_string()),
            },
        ),
        (
            name_key("int_to_text"),
            TaskSignature {
                params: vec!["Int".to_string()],
                return_type: Some("Text".to_string()),
            },
        ),
        (
            name_key("text_split"),
            TaskSignature {
                params: vec!["Text".to_string(), "Text".to_string()],
                return_type: Some("List Text".to_string()),
            },
        ),
        (
            name_key("list_len"),
            TaskSignature {
                params: vec!["List".to_string()],
                return_type: Some("UInt".to_string()),
            },
        ),
        (
            name_key("stdout_write"),
            TaskSignature {
                params: vec!["Text".to_string()],
                return_type: Some("Unit".to_string()),
            },
        ),
    ])
}

fn task_signatures(program: &Program) -> BTreeMap<String, TaskSignature> {
    let mut signatures = BTreeMap::new();
    for file in &program.files {
        collect_task_signatures(&file.items, &mut signatures);
    }
    signatures
}

fn task_signatures_from_items(items: &[Item]) -> BTreeMap<String, TaskSignature> {
    let mut signatures = BTreeMap::new();
    collect_task_signatures(items, &mut signatures);
    signatures
}

fn collect_task_signatures(items: &[Item], signatures: &mut BTreeMap<String, TaskSignature>) {
    for item in items {
        match item {
            Item::Task(task) => {
                signatures.insert(
                    name_key(&task.name),
                    TaskSignature {
                        params: task.params.iter().map(|param| param.ty.clone()).collect(),
                        return_type: task.result.as_deref().map(expected_return_value_type),
                    },
                );
            }
            Item::App(app) => collect_task_signatures(&app.items, signatures),
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn signature_help_text(callee: &str, signature: &TaskSignature) -> String {
    let params = signature.params.join(", ");
    match &signature.return_type {
        Some(return_type) => format!("`{callee}({params}) -> {return_type}`"),
        None => format!("`{callee}({params})`"),
    }
}

struct CallShapeIssue {
    callee: String,
    is_builtin: bool,
    signature: TaskSignature,
    actual_args: usize,
    call_span: Span,
}

/// WO30 Item 4: the AST-driven general call-shape probe. Walks the
/// statement's canonical expressions in pre-order; the first call whose
/// callee resolves to a declared signature with a mismatched argument count
/// is H0640. Unresolved callees never reach this probe: resolver errors set
/// the item's `blocked` flag and `type_statement` returns before it runs, so
/// H0640 cannot stack on H0601.
fn call_shape_issue(
    parsed: &crate::ast::ParsedBodyStatement,
    task_signatures: &BTreeMap<String, TaskSignature>,
) -> Option<CallShapeIssue> {
    let builtins = builtin_task_signatures();
    fn walk(
        expression: &crate::ast::CanonicalExpression,
        builtins: &BTreeMap<String, TaskSignature>,
        task_signatures: &BTreeMap<String, TaskSignature>,
    ) -> Option<CallShapeIssue> {
        if let crate::ast::CanonicalExpressionKind::Call { callee, arguments } = &expression.kind
            && let crate::ast::CanonicalExpressionKind::Identifier(name) = &callee.kind
        {
            // Builtin-table-first: a builtin name wins over a same-named user
            // task, matching runtime dispatch (ledger #19). An unknown callee
            // is not a shape issue — the resolver owns it (H0601) — so keep
            // walking for nested calls.
            let resolved = builtins
                .get(&name_key(name))
                .map(|signature| (true, signature))
                .or_else(|| {
                    task_signatures
                        .get(&name_key(name))
                        .map(|signature| (false, signature))
                });
            if let Some((is_builtin, signature)) = resolved
                && arguments.len() != signature.params.len()
            {
                return Some(CallShapeIssue {
                    callee: name.clone(),
                    is_builtin,
                    signature: signature.clone(),
                    actual_args: arguments.len(),
                    call_span: expression.range.start.clone(),
                });
            }
        }
        let children = canonical_child_expressions(expression);
        children
            .into_iter()
            .find_map(|child| walk(child, builtins, task_signatures))
    }

    let expressions: Vec<&crate::ast::ParsedExpression> = match &parsed.kind {
        crate::ast::ParsedBodyStatementKind::Return(expression) => vec![expression],
        crate::ast::ParsedBodyStatementKind::Binding { value, .. } => value.iter().collect(),
        crate::ast::ParsedBodyStatementKind::Other { expressions } => expressions.iter().collect(),
    };
    expressions
        .into_iter()
        .find_map(|expression| walk(&expression.canonical, &builtins, task_signatures))
}

/// The canonical child expressions of a canonical expression, shared by the
/// call-shape (H0640) and call-argument-type (H0641) walks so both probes see
/// the same tree through `Try`/`Group` wrappers.
fn canonical_child_expressions(
    expression: &crate::ast::CanonicalExpression,
) -> Vec<&crate::ast::CanonicalExpression> {
    match &expression.kind {
        crate::ast::CanonicalExpressionKind::Field { base, .. } => vec![base],
        crate::ast::CanonicalExpressionKind::ElementPlace { base, .. } => vec![base],
        crate::ast::CanonicalExpressionKind::ListLiteral(items) => items.iter().collect(),
        crate::ast::CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            fields.iter().map(|(_, value)| value).collect()
        }
        crate::ast::CanonicalExpressionKind::Call { callee, arguments } => {
            std::iter::once(callee.as_ref())
                .chain(arguments.iter())
                .collect()
        }
        crate::ast::CanonicalExpressionKind::Permission { value, .. } => vec![value],
        crate::ast::CanonicalExpressionKind::Try { value, .. } => vec![value],
        crate::ast::CanonicalExpressionKind::Binary { left, right, .. } => vec![left, right],
        crate::ast::CanonicalExpressionKind::Group(inner) => vec![inner],
        _ => Vec::new(),
    }
}

/// WO30 Item 2: the statically known type of a call argument, classified
/// from the canonical AST — never from source text. Literals classify by
/// their canonical form (the parser files every non-negative digit run as
/// `UIntLiteral`, so a negative literal always arrives as `IntLiteral` with
/// a negative value); identifiers and places use their lexical scope facts;
/// calls use the callee's resolved return type. Anything else is unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CanonicalArgumentType {
    Unknown,
    TextLiteral,
    BoolLiteral,
    NonNegativeIntLiteral,
    NegativeIntLiteral,
    ListLiteral,
    Named(String),
}

fn classify_argument_type(
    argument: &crate::ast::CanonicalExpression,
    scopes: &TypeScopeStack<TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> CanonicalArgumentType {
    match &argument.kind {
        crate::ast::CanonicalExpressionKind::TextLiteral(_) => CanonicalArgumentType::TextLiteral,
        crate::ast::CanonicalExpressionKind::BoolLiteral(_) => CanonicalArgumentType::BoolLiteral,
        crate::ast::CanonicalExpressionKind::UIntLiteral(_) => {
            CanonicalArgumentType::NonNegativeIntLiteral
        }
        crate::ast::CanonicalExpressionKind::IntLiteral(value) => {
            if *value < 0 {
                CanonicalArgumentType::NegativeIntLiteral
            } else {
                CanonicalArgumentType::NonNegativeIntLiteral
            }
        }
        crate::ast::CanonicalExpressionKind::ListLiteral(_) => CanonicalArgumentType::ListLiteral,
        crate::ast::CanonicalExpressionKind::Identifier(name) => scopes
            .lookup(name)
            .map_or(CanonicalArgumentType::Unknown, |fact| {
                CanonicalArgumentType::Named(fact.type_text)
            }),
        crate::ast::CanonicalExpressionKind::Field { base, field } => {
            if let crate::ast::CanonicalExpressionKind::Identifier(root) = &base.kind
                && let Some(root_fact) = scopes.lookup(root)
                && let Some(field_type) =
                    field_place::field_type(field_types, &root_fact.type_text, field)
            {
                CanonicalArgumentType::Named(field_type.to_string())
            } else {
                CanonicalArgumentType::Unknown
            }
        }
        crate::ast::CanonicalExpressionKind::ElementPlace { base, .. } => {
            if let crate::ast::CanonicalExpressionKind::Identifier(root) = &base.kind
                && let Some(root_fact) = scopes.lookup(root)
                && let Some(element_type) = element_place::list_element_type(&root_fact.type_text)
            {
                CanonicalArgumentType::Named(element_type.to_string())
            } else {
                CanonicalArgumentType::Unknown
            }
        }
        crate::ast::CanonicalExpressionKind::Call { callee, .. } => {
            if let crate::ast::CanonicalExpressionKind::Identifier(name) = &callee.kind {
                // Builtin-table-first, matching the call-shape probe and
                // runtime dispatch (ledger #19): the builtin return type
                // wins over a same-named user task's declared return.
                if let Some(signature) = builtin_task_signatures().get(&name_key(name))
                    && let Some(return_type) = &signature.return_type
                {
                    return CanonicalArgumentType::Named(return_type.clone());
                }
                if let Some(fact) = task_returns.get(&name_key(name)) {
                    return CanonicalArgumentType::Named(fact.type_text.clone());
                }
            }
            CanonicalArgumentType::Unknown
        }
        // Transparent wrappers: the argument's type is the inner type.
        crate::ast::CanonicalExpressionKind::Group(inner)
        | crate::ast::CanonicalExpressionKind::Permission { value: inner, .. }
        | crate::ast::CanonicalExpressionKind::Try { value: inner, .. } => {
            classify_argument_type(inner, scopes, task_returns, field_types)
        }
        _ => CanonicalArgumentType::Unknown,
    }
}

/// WO30 Item 2: exact type-compatibility — no inference beyond this table.
/// Unknown argument types are compatible (they stay silent per decision
/// 0014 honesty). A negative integer literal is *not* rejected for `UInt`
/// here; that rejection is WO30 Item 3 (H0642).
fn argument_type_compatible(expected: &str, actual: &CanonicalArgumentType) -> bool {
    match actual {
        CanonicalArgumentType::Unknown => true,
        CanonicalArgumentType::TextLiteral => expected == "Text",
        CanonicalArgumentType::BoolLiteral => expected == "Bool",
        CanonicalArgumentType::NonNegativeIntLiteral => expected == "Int" || expected == "UInt",
        // WO30 Item 3 owns the negative-literal-to-UInt rejection (H0642);
        // Item 2 treats the negative literal as compatible with Int and
        // silent for UInt.
        CanonicalArgumentType::NegativeIntLiteral => expected == "Int" || expected == "UInt",
        CanonicalArgumentType::ListLiteral => expected == "List" || expected.starts_with("List "),
        CanonicalArgumentType::Named(name) => {
            if expected == "List" {
                // The only width rule: a bare `List` parameter accepts any
                // list-typed argument. It exists so `list_len` accepts every
                // list the runtime accepts.
                name == "List" || name.starts_with("List ")
            } else {
                name == expected
            }
        }
    }
}

/// The machine-readable argument type name for the H0641 diagnostic, using
/// the historical `infer_expression_type` vocabulary (`integer_literal`,
/// `list_literal`) for literals whose language type is contextual.
fn canonical_argument_type_name(actual: &CanonicalArgumentType) -> String {
    match actual {
        CanonicalArgumentType::Unknown => "unknown".to_string(),
        CanonicalArgumentType::TextLiteral => "Text".to_string(),
        CanonicalArgumentType::BoolLiteral => "Bool".to_string(),
        CanonicalArgumentType::NonNegativeIntLiteral
        | CanonicalArgumentType::NegativeIntLiteral => "integer_literal".to_string(),
        CanonicalArgumentType::ListLiteral => "list_literal".to_string(),
        CanonicalArgumentType::Named(name) => name.clone(),
    }
}

struct CallArgumentTypeIssue {
    callee: String,
    is_builtin: bool,
    signature: TaskSignature,
    argument_index: usize,
    expected_type: String,
    actual_type: CanonicalArgumentType,
    call_span: Span,
}

/// WO30 Item 2: the AST-driven argument-type probe (H0641). Runs after the
/// arity probe so arity mismatches keep H0640 precedence: walks the
/// statement's canonical expressions in pre-order; the first call whose
/// callee resolves to a declared signature with a matching argument count,
/// but which has a statically known argument type that mismatches its
/// parameter type, is H0641. Unknown argument types produce no diagnostic.
fn call_argument_type_issue(
    parsed: &crate::ast::ParsedBodyStatement,
    task_signatures: &BTreeMap<String, TaskSignature>,
    scopes: &TypeScopeStack<TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<CallArgumentTypeIssue> {
    let builtins = builtin_task_signatures();
    fn walk(
        expression: &crate::ast::CanonicalExpression,
        builtins: &BTreeMap<String, TaskSignature>,
        task_signatures: &BTreeMap<String, TaskSignature>,
        scopes: &TypeScopeStack<TypeFact>,
        task_returns: &BTreeMap<String, TypeFact>,
        field_types: &FieldTypeMap,
    ) -> Option<CallArgumentTypeIssue> {
        if let crate::ast::CanonicalExpressionKind::Call { callee, arguments } = &expression.kind
            && let crate::ast::CanonicalExpressionKind::Identifier(name) = &callee.kind
        {
            // Builtin-table-first, matching the call-shape probe and runtime
            // dispatch (ledger #19). An unknown callee is not a type issue —
            // the resolver owns it (H0601) — so keep walking for nested calls.
            let resolved = builtins
                .get(&name_key(name))
                .map(|signature| (true, signature))
                .or_else(|| {
                    task_signatures
                        .get(&name_key(name))
                        .map(|signature| (false, signature))
                });
            // Arity mismatches belong to the H0640 probe, which runs first;
            // only arity-clean calls are type-checked here.
            if let Some((is_builtin, signature)) = resolved
                && arguments.len() == signature.params.len()
            {
                for (argument_index, (argument, expected)) in
                    arguments.iter().zip(signature.params.iter()).enumerate()
                {
                    let actual =
                        classify_argument_type(argument, scopes, task_returns, field_types);
                    if !argument_type_compatible(expected, &actual) {
                        return Some(CallArgumentTypeIssue {
                            callee: name.clone(),
                            is_builtin,
                            signature: signature.clone(),
                            argument_index,
                            expected_type: expected.clone(),
                            actual_type: actual,
                            call_span: expression.range.start.clone(),
                        });
                    }
                }
            }
        }
        canonical_child_expressions(expression)
            .into_iter()
            .find_map(|child| {
                walk(
                    child,
                    builtins,
                    task_signatures,
                    scopes,
                    task_returns,
                    field_types,
                )
            })
    }

    let expressions: Vec<&crate::ast::ParsedExpression> = match &parsed.kind {
        crate::ast::ParsedBodyStatementKind::Return(expression) => vec![expression],
        crate::ast::ParsedBodyStatementKind::Binding { value, .. } => value.iter().collect(),
        crate::ast::ParsedBodyStatementKind::Other { expressions } => expressions.iter().collect(),
    };
    expressions.into_iter().find_map(|expression| {
        walk(
            &expression.canonical,
            &builtins,
            task_signatures,
            scopes,
            task_returns,
            field_types,
        )
    })
}

fn initial_environment(params: &[Param]) -> BTreeMap<String, TypeFact> {
    let mut environment = BTreeMap::new();
    for param in params {
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

fn binding_type_fact(
    statement: &BodyStatement,
    actual: Option<&TypeFact>,
) -> Option<(String, TypeFact)> {
    let annotation = binding_annotation(statement);
    let name = binding_name(statement)?;
    if let Some(type_text) = annotation {
        return Some((
            name,
            TypeFact {
                type_text,
                source: "binding_annotation_v0",
            },
        ));
    }
    let actual = actual?;
    Some((name, actual.clone()))
}

fn binding_annotation(statement: &BodyStatement) -> Option<String> {
    let left = binding_left(statement)?;
    let (_name, type_text) = left.split_once(':')?;
    let type_text = type_text.trim();
    if type_text.is_empty() {
        None
    } else {
        Some(type_text.to_string())
    }
}

fn binding_name(statement: &BodyStatement) -> Option<String> {
    let left = binding_left(statement)?;
    let name = left.split_once(':').map_or(left, |(name, _type_text)| name);
    let name = name.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn binding_left(statement: &BodyStatement) -> Option<&str> {
    if !matches!(statement.kind, "let_binding" | "mutable_binding") {
        return None;
    }
    let keyword = if statement.kind == "let_binding" {
        "let"
    } else {
        "change"
    };
    let rest = strip_keyword(&statement.text, keyword)?;
    rest.split_once('=').map(|(left, _value)| left.trim())
}

fn set_place_name(statement: &BodyStatement) -> Option<&str> {
    let rest = strip_keyword(&statement.text, "set")?;
    let (place, _value) = rest.split_once('=')?;
    let place = place.trim();
    if place.is_empty() { None } else { Some(place) }
}

fn place_type_fact(
    name: &str,
    scopes: &TypeScopeStack<TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<TypeFact> {
    if let Some((root, _index)) = element_place::split_element_place(name) {
        let root_fact = scopes.lookup(root)?;
        let type_text = element_place::list_element_type(&root_fact.type_text)?;
        return Some(type_fact(type_text, "list_element_place_v0"));
    }
    if let Some((root, field)) = field_place::split_field_place(name) {
        let root_fact = scopes.lookup(root)?;
        let type_text = field_place::field_type(field_types, &root_fact.type_text, field)?;
        return Some(type_fact(type_text, "record_field_place_v0"));
    }
    // The whole-text fallback is for plain name references only. Without
    // the guard, any expression whose snake-normalized form collides with
    // a bound name (e.g. `piece != ""` normalizing to `piece`) would take
    // the bound name's type instead of its own inferred type (WO28 #13).
    if is_plain_name(name) {
        return scopes.lookup(name);
    }
    None
}

fn is_plain_name(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty()
        && text
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn infer_expression_type(
    expression_text: &str,
    scopes: &TypeScopeStack<TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<TypeFact> {
    let text = expression_text.trim();
    if text.is_empty() {
        return Some(type_fact("Unit", "unit_expression_v0"));
    }
    if let Some(argument) = strip_permission_expression(text) {
        return infer_expression_type(argument, scopes, task_returns, field_types);
    }
    if text == "true" || text == "false" {
        return Some(type_fact("Bool", "bool_literal_v0"));
    }
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return Some(type_fact("Text", "text_literal_v0"));
    }
    if return_dependency::is_closed_view_derivation_expression(text) {
        return Some(type_fact("Text", "closed_view_derivation_slice_until_v0"));
    }
    if is_list_literal(text) {
        return Some(type_fact("list_literal", "list_literal_v0"));
    }
    if text.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(type_fact("integer_literal", "integer_literal_v0"));
    }
    if let Some(fact) = place_type_fact(text, scopes, field_types) {
        return Some(fact);
    }
    if is_condition_expression(text) {
        return Some(type_fact("Bool", "condition_expression_v0"));
    }
    if let Some(type_name) = record_literal_type_name(text) {
        return Some(type_fact(type_name, "record_literal_constructor_v0"));
    }
    if let Some(root) = path_root_type_name(text) {
        return Some(type_fact(root, "path_root_type_v0"));
    }
    if let Some(fact) = infer_additive_expression_type(text, scopes, task_returns, field_types) {
        return Some(fact);
    }
    if let Some(fact) =
        infer_multiplicative_expression_type(text, scopes, task_returns, field_types)
    {
        return Some(fact);
    }
    if let Some((callee, _args)) = split_call(text) {
        if callee == "list_append" {
            return Some(type_fact("Unit", "list_append_builtin_v0"));
        }
        return task_returns.get(&name_key(callee)).cloned();
    }
    place_type_fact(text, scopes, field_types)
}

fn infer_additive_expression_type(
    text: &str,
    scopes: &TypeScopeStack<TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<TypeFact> {
    let (left, right) = text.split_once(" + ")?;
    let left = infer_expression_type(left, scopes, task_returns, field_types)?;
    let right = infer_expression_type(right, scopes, task_returns, field_types)?;
    if right.type_text == "integer_literal" || left.type_text == right.type_text {
        Some(TypeFact {
            type_text: left.type_text,
            source: "additive_expression_v0",
        })
    } else {
        None
    }
}

fn infer_multiplicative_expression_type(
    text: &str,
    scopes: &TypeScopeStack<TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<TypeFact> {
    let (left, right) = text.split_once(" * ")?;
    let left = infer_expression_type(left, scopes, task_returns, field_types)?;
    let right = infer_expression_type(right, scopes, task_returns, field_types)?;
    if right.type_text == "integer_literal" || left.type_text == right.type_text {
        Some(TypeFact {
            type_text: left.type_text,
            source: "multiplicative_expression_v0",
        })
    } else {
        None
    }
}

fn is_list_literal(text: &str) -> bool {
    text.starts_with('[') && text.ends_with(']')
}

fn strip_permission_expression(text: &str) -> Option<&str> {
    ["borrow", "change", "consume"]
        .iter()
        .find_map(|keyword| strip_keyword(text.trim(), keyword))
}

fn split_call(text: &str) -> Option<(&str, &str)> {
    let text = text.trim();
    let inside = text.strip_suffix(')')?;
    let (callee, args) = inside.split_once('(')?;
    let callee = callee.trim();
    if callee.is_empty() {
        None
    } else {
        Some((callee, args))
    }
}

fn is_condition_expression(text: &str) -> bool {
    [
        " == ", " != ", " <= ", " >= ", " < ", " > ", " is ", " does ", " and ", " or ",
    ]
    .iter()
    .any(|operator| text.contains(operator))
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

fn expected_error_value_type(expected_type: &str) -> Option<String> {
    let expected_type = return_dependency::result_type_without_return_dependency(expected_type);
    let tokens = type_tokens(&expected_type);
    if matches!(tokens.first().map(String::as_str), Some("Result")) && tokens.len() >= 3 {
        Some(tokens[2].clone())
    } else {
        None
    }
}

fn types_compatible(expected_type: &str, actual_type: &str) -> bool {
    let expected_type = return_dependency::result_type_without_return_dependency(expected_type);
    let actual_key = name_key(actual_type);
    if actual_key.is_empty() {
        return false;
    }
    if actual_key == name_key(&expected_type) {
        return true;
    }
    if actual_key == "integer_literal" {
        return matches!(name_key(&expected_type).as_str(), "int" | "uint" | "float");
    }
    actual_key == "list_literal" && name_key(&expected_type).starts_with("list")
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

fn type_fact(type_text: impl Into<String>, source: &'static str) -> TypeFact {
    TypeFact {
        type_text: type_text.into(),
        source,
    }
}

fn expression_text_for_statement(statement: &BodyStatement) -> Option<&str> {
    match statement.kind {
        "return" => strip_keyword(&statement.text, "return"),
        "fail" => strip_keyword(&statement.text, "fail"),
        "let_binding" | "mutable_binding" | "set_place" => statement
            .text
            .split_once('=')
            .map(|(_left, value)| value.trim()),
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

fn item_sections(item: &Item) -> &[Section] {
    match item {
        Item::App(app) => &app.sections,
        Item::Type(type_def) => &type_def.sections,
        Item::Store(store) => &store.sections,
        Item::Task(task) => &task.sections,
        Item::Test(test) => &test.sections,
    }
}

fn item_params(item: &Item) -> &[Param] {
    match item {
        Item::Task(task) => &task.params,
        Item::Test(test) => &test.params,
        _ => &[],
    }
}

fn item_result(item: &Item) -> Option<&str> {
    match item {
        Item::Task(task) => task.result.as_deref(),
        _ => None,
    }
}

fn item_status(statements: &[TypedStatement], blocked: bool) -> &'static str {
    if blocked {
        "blocked_by_prior_errors"
    } else if statements
        .iter()
        .any(|statement| statement.status.starts_with("rejected_"))
    {
        "full_type_errors_v0"
    } else if statements.iter().any(is_blocking_statement) {
        "blocked_by_unchecked_body_types_v0"
    } else {
        "recognized_core_body_types_checked_v0"
    }
}

fn is_blocking_statement(statement: &TypedStatement) -> bool {
    matches!(
        statement.status,
        "rejected_statement_type_mismatch_v0"
            | "rejected_typed_failure_relationship_v0"
            | "rejected_invalid_stdout_write_call_v0"
            | "rejected_invalid_clock_replay_call_v0"
            | "rejected_invalid_files_read_text_call_v0"
            | "rejected_invalid_text_escape_v0"
            | "unchecked_statement_type_v0"
            | "blocked_unsupported_statement_v0"
            | "not_checked_blocked_by_prior_errors_v0"
    )
}

impl FullTypeCheckReport {
    fn status(&self) -> &'static str {
        if self.source_errors > 0 {
            "blocked_by_source_errors"
        } else if self.type_check_summary.resolver_errors > 0 {
            "blocked_by_resolver_errors"
        } else if self.type_check_summary.type_errors > 0 {
            "blocked_by_type_errors"
        } else if self.core_verify_summary.failed_checks > 0 {
            "blocked_by_core_verify_errors"
        } else if self.rejected_statements() > 0 || self.rejected_predicates() > 0 {
            "full_type_errors_v0"
        } else if self.unchecked_statements() > 0 || self.unsupported_statements() > 0 {
            "blocked_by_unchecked_body_types_v0"
        } else {
            "recognized_core_body_types_checked_v0"
        }
    }

    fn files(&self) -> usize {
        self.files
    }

    fn item_count(&self) -> usize {
        self.item_count
    }

    fn statement_count(&self) -> usize {
        self.items.iter().map(|item| item.statements.len()).sum()
    }

    fn checked_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                matches!(
                    statement.status,
                    "accepted_statement_type_v0"
                        | "accepted_inferred_binding_type_v0"
                        | "accepted_writable_field_alias_candidate_deferred_to_ownership_v0"
                        | "accepted_no_expression_type_obligation_v0"
                        | "accepted_same_root_failure_propagation_v0"
                        | "accepted_for_each_binding_v0"
                        | "accepted_test_expectation_v0"
                        | "accepted_causal_failure_wrap_v0"
                        | "accepted_nominal_direct_failure_v0"
                        | "accepted_typed_failure_deferred_to_effect_v0"
                        | "rejected_typed_failure_relationship_v0"
                        | "rejected_statement_type_mismatch_v0"
                        | "rejected_invalid_stdout_write_call_v0"
                        | "rejected_invalid_clock_replay_call_v0"
                        | "rejected_invalid_files_read_text_call_v0"
                        | "rejected_invalid_text_escape_v0"
                )
            })
            .count()
    }

    fn accepted_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                matches!(
                    statement.status,
                    "accepted_statement_type_v0"
                        | "accepted_inferred_binding_type_v0"
                        | "accepted_writable_field_alias_candidate_deferred_to_ownership_v0"
                        | "accepted_no_expression_type_obligation_v0"
                        | "accepted_same_root_failure_propagation_v0"
                        | "accepted_for_each_binding_v0"
                        | "accepted_test_expectation_v0"
                        | "accepted_causal_failure_wrap_v0"
                        | "accepted_nominal_direct_failure_v0"
                        | "accepted_typed_failure_deferred_to_effect_v0"
                )
            })
            .count()
    }

    fn rejected_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                matches!(
                    statement.status,
                    "rejected_statement_type_mismatch_v0"
                        | "rejected_typed_failure_relationship_v0"
                        | "rejected_invalid_stdout_write_call_v0"
                        | "rejected_invalid_clock_replay_call_v0"
                        | "rejected_invalid_files_read_text_call_v0"
                        | "rejected_invalid_text_escape_v0"
                        | "rejected_invalid_call_arity_v0"
                        | "rejected_invalid_call_argument_type_v0"
                )
            })
            .count()
    }

    fn unchecked_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| statement.status == "unchecked_statement_type_v0")
            .count()
    }

    fn unsupported_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                matches!(
                    statement.status,
                    "blocked_unsupported_statement_v0" | "not_checked_blocked_by_prior_errors_v0"
                )
            })
            .count()
    }

    fn blocking_issues(&self) -> usize {
        debug_assert!(
            self.items
                .iter()
                .flat_map(|item| item.statements.iter())
                .filter_map(|statement| statement.prior_blocker.as_ref())
                .all(|prior| !prior.occurrence_id.as_str().is_empty())
        );
        self.source_errors
            + self.type_check_summary.resolver_errors
            + self.type_check_summary.type_errors
            + self.core_verify_summary.failed_checks
            + self.rejected_statements()
            + self.unchecked_statements()
            + self.unsupported_statements()
            + self.rejected_predicates()
    }

    fn rejected_predicates(&self) -> usize {
        self.predicates.iter().filter(|fact| fact.blocks()).count()
    }
}

fn push_predicates(out: &mut String, facts: &[PredicateFact], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "predicate_facts");
    out.push_str(": [\n");
    for (index, fact) in facts.iter().enumerate() {
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "task", &fact.task, true);
        push_span_field(out, indent + 4, "task_span", &fact.task_span, true);
        push_string_field(out, indent + 4, "section", &fact.section, true);
        push_string_field(out, indent + 4, "text", &fact.text, true);
        push_string_field(
            out,
            indent + 4,
            "predicate_recognition_status",
            fact.status.as_str(),
            true,
        );
        push_string_field(out, indent + 4, "reason", fact.reason, true);
        push_string_field(
            out,
            indent + 4,
            "diagnostic_code",
            fact.diagnostic()
                .map(|diagnostic| diagnostic.code.as_str())
                .unwrap_or("none"),
            true,
        );
        push_indent(out, indent + 4);
        push_json_string(out, "places");
        out.push_str(": [");
        for (place_index, place) in fact.places.iter().enumerate() {
            if place_index > 0 {
                out.push_str(", ");
            }
            out.push('{');
            push_json_string(out, "text");
            out.push_str(": ");
            push_json_string(out, &place.text);
            out.push_str(", ");
            push_json_string(out, "span");
            out.push_str(": {");
            push_json_string(out, "file");
            out.push_str(": ");
            push_json_string(out, &place.span.file.replace('\\', "/"));
            out.push_str(&format!(
                ", \"line\": {}, \"column\": {}",
                place.span.line, place.span.column
            ));
            out.push_str("}, ");
            for (name, value) in [
                ("scope_id", Some(place.scope_id.as_str())),
                ("root_definition_id", place.root_definition_id.as_deref()),
                ("definition_id", place.definition_id.as_deref()),
                ("resolution", Some(place.resolution)),
                ("eligibility", Some(place.eligibility)),
                ("type", place.type_text.as_deref()),
            ] {
                push_json_string(out, name);
                out.push_str(": ");
                push_json_string(out, value.unwrap_or("none"));
                if name != "type" {
                    out.push_str(", ");
                }
            }
            out.push('}');
        }
        out.push_str("],\n");
        push_string_field(
            out,
            indent + 4,
            "expected",
            fact.expected.as_deref().unwrap_or("none"),
            true,
        );
        push_string_field(
            out,
            indent + 4,
            "actual",
            fact.actual.as_deref().unwrap_or("none"),
            true,
        );
        push_string_field(
            out,
            indent + 4,
            "comparison_operator",
            fact.comparison.unwrap_or("none"),
            true,
        );
        push_string_field(
            out,
            indent + 4,
            "left_type",
            fact.left_type.as_deref().unwrap_or("unknown"),
            true,
        );
        push_string_field(
            out,
            indent + 4,
            "right_type",
            fact.right_type.as_deref().unwrap_or("unknown"),
            true,
        );
        push_string_field(out, indent + 4, "repair", &fact.repair(), true);
        push_optional_span_field(
            out,
            indent + 4,
            "intent_span",
            fact.intent_span.as_ref(),
            true,
        );
        push_optional_span_field(
            out,
            indent + 4,
            "offending_span",
            fact.offending_span.as_ref(),
            true,
        );
        push_usize_field(
            out,
            indent + 4,
            "delimiter_depth",
            fact.delimiter_depth,
            true,
        );
        push_span_field(out, indent + 4, "line_span", &fact.line_span, false);
        push_indent(out, indent + 2);
        out.push('}');
        push_comma_newline(out, index + 1 < facts.len());
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
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

fn push_dependency_summaries(
    out: &mut String,
    report: &FullTypeCheckReport,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "dependencies");
    out.push_str(": {\n");
    push_indent(out, indent + 2);
    push_json_string(out, "type_check");
    out.push_str(": {\n");
    push_string_field(
        out,
        indent + 4,
        "schema",
        report.type_check_summary.schema,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "status",
        report.type_check_summary.status,
        true,
    );
    push_usize_field(
        out,
        indent + 4,
        "type_errors",
        report.type_check_summary.type_errors,
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("},\n");
    push_indent(out, indent + 2);
    push_json_string(out, "core_verify");
    out.push_str(": {\n");
    push_string_field(
        out,
        indent + 4,
        "schema",
        report.core_verify_summary.schema,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "status",
        report.core_verify_summary.status,
        true,
    );
    push_usize_field(
        out,
        indent + 4,
        "failed_checks",
        report.core_verify_summary.failed_checks,
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("}\n");
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_summary(out: &mut String, report: &FullTypeCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "summary");
    out.push_str(": {\n");
    push_usize_field(out, indent + 2, "files", report.files(), true);
    push_usize_field(out, indent + 2, "items", report.item_count(), true);
    push_usize_field(out, indent + 2, "body_items", report.items.len(), true);
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
        "unsupported_statements",
        report.unsupported_statements(),
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
        report.type_check_summary.resolver_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_errors",
        report.type_check_summary.type_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "core_verify_errors",
        report.core_verify_summary.failed_checks,
        true,
    );
    push_usize_field(out, indent + 2, "execution_ready", 0, true);
    push_usize_field(out, indent + 2, "ir_ready", 0, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_items(out: &mut String, items: &[FullTypeItem], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "typed_items");
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

fn push_item(out: &mut String, item: &FullTypeItem, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &item.id, true);
    push_string_field(out, indent + 2, "kind", item.kind, true);
    push_string_field(out, indent + 2, "name", &item.name, true);
    push_span_field(out, indent + 2, "source_span", &item.span, true);
    push_string_field(out, indent + 2, "status", item.status, true);
    push_statements(out, &item.statements, indent + 2, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_statements(out: &mut String, statements: &[TypedStatement], indent: usize, comma: bool) {
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

fn push_statement(out: &mut String, statement: &TypedStatement, indent: usize) {
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
    push_optional_string_field(
        out,
        indent + 2,
        "expression_text",
        statement.expression_text.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "expected_type",
        statement.expected_type.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "actual_type",
        statement.actual_type.as_deref(),
        true,
    );
    push_optional_string_field(out, indent + 2, "type_source", statement.type_source, true);
    push_string_field(out, indent + 2, "status", statement.status, true);
    push_optional_string_field(out, indent + 2, "reason", statement.reason, true);
    push_optional_string_field(
        out,
        indent + 2,
        "failure_form",
        statement.failure_form,
        true,
    );
    push_optional_string_field(out, indent + 2, "callee", statement.callee.as_deref(), true);
    push_optional_string_field(
        out,
        indent + 2,
        "callee_result_root",
        statement.callee_result_root.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "caller_result_root",
        statement.caller_result_root.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "wrapper_root",
        statement.wrapper_root.as_deref(),
        true,
    );
    push_optional_span_field(
        out,
        indent + 2,
        "call_span",
        statement.call_span.as_ref(),
        true,
    );
    push_optional_span_field(
        out,
        indent + 2,
        "callee_span",
        statement.callee_span.as_ref(),
        true,
    );
    push_optional_span_field(
        out,
        indent + 2,
        "caller_span",
        statement.caller_span.as_ref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "diagnostic_code",
        statement.diagnostic_code,
        true,
    );
    push_optional_string_field(out, indent + 2, "help", statement.help.as_deref(), false);
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
    use crate::core_verify::{
        core_verify_report_build_count_for_test as verify_builds,
        reset_core_verify_report_build_count_for_test as reset_verify_builds,
    };
    use crate::parser::parse_source;

    use super::{
        build_report, full_type_check_has_errors, full_type_check_json, full_type_check_text,
    };
    use crate::diagnostic::{Diagnostic, DiagnosticCode};

    #[test]
    fn json_accepts_recognized_task_body_types_without_execution_claims() {
        let program = typed_demo_program();
        let json = full_type_check_json(&program, &[]);

        assert!(!full_type_check_has_errors(&program, &[]));
        assert!(json.contains("\"schema\": \"hum.full_type_check.v0\""));
        assert!(json.contains("\"status\": \"recognized_core_body_types_checked_v0\""));
        assert!(json.contains("\"accepted_statement_type_v0\""));
        assert!(json.contains("\"accepted_inferred_binding_type_v0\""));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"ir_ready\": 0"));
        assert!(json.contains("\"no executable semantics\""));
    }

    #[test]
    fn json_blocks_unknown_and_unsupported_body_types() {
        let program = reference_like_program();
        let json = full_type_check_json(&program, &[]);

        assert!(full_type_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"blocked_by_unchecked_body_types_v0\""));
        assert!(json.contains("\"blocked_unsupported_statement_v0\""));
        assert!(json.contains("\"record_field_context_not_tracked_v0\""));
        assert!(json.contains("\"surface_save_requires_store_lowering\""));
    }

    #[test]
    fn unknown_writable_alias_candidates_defer_without_weakening_other_unknown_bindings() {
        let alias_program = Program {
            files: vec![
                parse_source(
                    "alias_defer.hum",
                    r#"type Point {
  x: UInt
  y: UInt
}

task alias_defer(change point: Point) -> Point {
  does:
    let first = change point.x
    let second = change first.y
    let nested = change point.x.deep
    return point
}
"#,
                )
                .file,
            ],
        };
        let alias_json = full_type_check_json(&alias_program, &[]);
        assert!(!full_type_check_has_errors(&alias_program, &[]));
        assert!(alias_json.contains("\"status\": \"recognized_core_body_types_checked_v0\""));
        assert!(alias_json.contains(
            "\"status\": \"accepted_writable_field_alias_candidate_deferred_to_ownership_v0\""
        ));
        assert!(
            alias_json
                .contains("\"reason\": \"writable_field_alias_shape_deferred_to_ownership_v0\"")
        );
        assert!(alias_json.contains("\"unchecked_statements\": 0"));

        let ordinary_program = Program {
            files: vec![
                parse_source(
                    "ordinary_unknown.hum",
                    r#"type Point {
  x: UInt
}

task ordinary_unknown(change point: Point) -> Point {
  does:
    let ordinary = point.x.deep
    return point
}
"#,
                )
                .file,
            ],
        };
        let ordinary_json = full_type_check_json(&ordinary_program, &[]);
        assert!(full_type_check_has_errors(&ordinary_program, &[]));
        assert!(ordinary_json.contains("\"status\": \"blocked_by_unchecked_body_types_v0\""));
        assert!(ordinary_json.contains("\"reason\": \"binding_initializer_type_unknown_v0\""));
    }

    #[test]
    fn text_reports_full_type_gate_without_safety_claims() {
        let program = typed_demo_program();
        let text = full_type_check_text(&program, &[]);

        assert!(text.contains("Hum full type check (hum.full_type_check.v0)"));
        assert!(text.contains("status: recognized_core_body_types_checked_v0"));
        assert!(text.contains("no memory-safety proof"));
    }

    #[test]
    fn ao_full_type_consumes_exact_typed_failure_occurrences_and_defers_h0907() {
        let h0901 = typed_failure_program(false);
        let report = build_report(&h0901, &[]);
        let statement = report
            .items
            .iter()
            .flat_map(|item| item.statements.iter())
            .find(|statement| statement.diagnostic_code == Some("H0901"))
            .expect("H0901 statement");
        let occurrence = crate::typed_failure::analyze_program(&h0901)
            .occurrences()
            .into_iter()
            .find(|occurrence| occurrence.code == DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY)
            .expect("H0901 occurrence");
        statement
            .prior_blocker
            .as_ref()
            .expect("full type prior")
            .validate_against(&occurrence)
            .expect("exact H0901 prior");

        let h0907 = typed_failure_program(true);
        let report = build_report(&h0907, &[]);
        let statement = report
            .items
            .iter()
            .flat_map(|item| item.statements.iter())
            .find(|statement| statement.status == "accepted_typed_failure_deferred_to_effect_v0")
            .expect("deferred H0907 statement");
        assert_eq!(statement.diagnostic_code, None);
        let occurrence = crate::typed_failure::analyze_program(&h0907)
            .occurrences()
            .into_iter()
            .find(|occurrence| occurrence.code == DiagnosticCode::MISSING_FAILURE_DECLARATION)
            .expect("H0907 occurrence");
        statement
            .prior_blocker
            .as_ref()
            .expect("deferred H0907 prior")
            .validate_against(&occurrence)
            .expect("exact H0907 prior");
    }

    fn typed_failure_program(explicit_try_without_declaration: bool) -> Program {
        let call = if explicit_try_without_declaration {
            "try source()"
        } else {
            "source()"
        };
        let caller_failure = if explicit_try_without_declaration {
            ""
        } else {
            "  fails when:\n    the caller source fails\n"
        };
        let source = format!(
            r#"module tests.ao.full_type

type SourceError {{
  code: Text
}}

task source() -> Result UInt, SourceError {{
  fails when:
    the source fails
  does:
    fail SourceError.origin
}}

task caller() -> Result UInt, SourceError {{
{caller_failure}  does:
    let value = {call}
    return value
}}
"#
        );
        Program {
            files: vec![parse_source("session_ao_full_type.hum", &source).file],
        }
    }

    fn typed_demo_program() -> crate::ast::Program {
        Program {
            files: vec![
                parse_source(
                    "typed_demo.hum",
                    r#"type WorkItem {
  title: Text
}

type WorkError {
  code: Text
}

task remember(item: WorkItem) -> Result WorkItem, WorkError {
  why:
    keep a typed body small

  needs:
    item exists

  ensures:
    item is returned

  fails when:
    item is missing

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    if item is missing {
      fail WorkError.empty_title
    }

    let checked = item
    return checked
}
"#,
                )
                .file,
            ],
        }
    }

    fn reference_like_program() -> crate::ast::Program {
        Program {
            files: vec![
                parse_source(
                    "reference_like.hum",
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
  why:
    keep a typed body small

  needs:
    title exists

  changes:
    work_items

  ensures:
    item is returned

  fails when:
    title is empty

  cost:
    time: O(1)
    space: O(1)
    check: warn

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
                .file,
            ],
        }
    }

    #[test]
    fn minimal_add_consumes_only_verified_canonical_type() {
        fn subject(path: &str, source: &str) -> (Program, Vec<Diagnostic>) {
            let parsed = parse_source(path, source);
            let mut diagnostics = crate::check::check_file(&parsed);
            diagnostics.extend(parsed.diagnostics);
            let program = Program {
                files: vec![parsed.file],
            };
            (program, diagnostics)
        }
        fn one_verify_report<R>(build: impl FnOnce() -> R) -> R {
            reset_verify_builds();
            let result = build();
            assert_eq!(verify_builds(), 1);
            result
        }
        fn assert_verified(
            report: &super::FullTypeCheckReport,
            expected: Option<&str>,
            status: &str,
        ) {
            let statement = &report.items[0].statements[0];
            assert_eq!(statement.expected_type.as_deref(), expected);
            assert_eq!(statement.actual_type.as_deref(), Some("Int"));
            assert_eq!(
                statement.type_source,
                Some("verified_canonical_minimal_add_type_v0")
            );
            assert_eq!(statement.status, status);
        }
        let (program, diagnostics) = subject(
            "verified-full-type.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\n",
        );
        let report = one_verify_report(|| build_report(&program, &diagnostics));
        assert_verified(&report, Some("Int"), "accepted_statement_type_v0");

        crate::core_verify::set_core_verify_corruption_for_test("type-text");
        let corrupted = one_verify_report(|| build_report(&program, &diagnostics));
        let statement = &corrupted.items[0].statements[0];
        assert!(statement.actual_type.is_none() && statement.type_source.is_none());
        assert_eq!(statement.status, "not_checked_blocked_by_prior_errors_v0");

        let (wrapped_program, wrapped_diagnostics) = subject(
            "verified-result-full-type.hum",
            "task add(a: Int, b: Int) -> Result Int, Text {\n  does:\n    return a + b\n}\n",
        );
        let wrapped_report =
            one_verify_report(|| build_report(&wrapped_program, &wrapped_diagnostics));
        assert_verified(&wrapped_report, Some("Int"), "accepted_statement_type_v0");
        assert_eq!(
            [
                crate::type_check::type_check_has_errors(&wrapped_program, &wrapped_diagnostics),
                crate::core_verify::core_verify_has_errors(&wrapped_program, &wrapped_diagnostics),
                super::full_type_check_has_errors(&wrapped_program, &wrapped_diagnostics),
            ],
            [false; 3]
        );
        for rendered in [
            crate::type_check::type_check_text(&wrapped_program, &wrapped_diagnostics),
            crate::type_check::type_check_json(&wrapped_program, &wrapped_diagnostics),
            crate::core_lower::core_lower_text(&wrapped_program, &wrapped_diagnostics),
            crate::core_lower::core_lower_json(&wrapped_program, &wrapped_diagnostics),
            crate::core_verify::core_verify_text(&wrapped_program, &wrapped_diagnostics),
            crate::core_verify::core_verify_json(&wrapped_program, &wrapped_diagnostics),
            super::full_type_check_text(&wrapped_program, &wrapped_diagnostics),
            super::full_type_check_json(&wrapped_program, &wrapped_diagnostics),
        ] {
            assert!(!rendered.is_empty());
        }

        for (path, result, expected, status) in [
            (
                "verified-no-result.hum",
                "",
                None,
                "unchecked_statement_type_v0",
            ),
            (
                "verified-mismatch.hum",
                " -> UInt",
                Some("UInt"),
                "rejected_statement_type_mismatch_v0",
            ),
        ] {
            let source =
                format!("task add(a: Int, b: Int){result} {{\n  does:\n    return a + b\n}}\n");
            let (program, diagnostics) = subject(path, &source);
            let report = one_verify_report(|| build_report(&program, &diagnostics));
            assert_verified(&report, expected, status);
            if expected == Some("UInt") {
                assert_eq!(
                    report.items[0].statements[0].reason,
                    Some("statement_expression_type_mismatch")
                );
            }
        }

        let (uint_program, uint_diagnostics) = subject(
            "legacy-full-type.hum",
            "task add(a: UInt) -> UInt {\n  does:\n    return a + 1\n}\n",
        );
        let uint_report = one_verify_report(|| build_report(&uint_program, &uint_diagnostics));
        assert_eq!(
            uint_report.items[0].statements[0].type_source,
            Some("additive_expression_v0")
        );

        let (blocked_program, blocked_diagnostics) = subject(
            "verified-blocked.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\ntask unsupported(a: Int) -> Int {\n  does:\n    return 1 + a\n}\n",
        );
        let blocked_report =
            one_verify_report(|| build_report(&blocked_program, &blocked_diagnostics));
        let blocked_statement = &blocked_report.items[0].statements[0];
        assert_eq!(
            (
                blocked_statement.actual_type.as_deref(),
                blocked_statement.type_source,
                blocked_statement.status
            ),
            (None, None, "not_checked_blocked_by_prior_errors_v0")
        );

        assert!(!one_verify_report(|| super::full_type_check_has_errors(
            &program,
            &diagnostics
        )));
        one_verify_report(|| super::full_type_check_text(&program, &diagnostics));
        one_verify_report(|| super::full_type_check_json(&program, &diagnostics));
    }

    #[test]
    fn minimal_add_backend_fact_handoff_is_exact_and_borrowed() {
        let parsed = parse_source(
            "examples/core/minimal_add.hum",
            include_str!("../examples/core/minimal_add.hum"),
        );
        let diagnostics = parsed.diagnostics;
        let program = Program {
            files: vec![parsed.file],
        };
        let item = &program.files[0].items[0];
        let crate::ast::Item::Task(task) = item else {
            panic!("minimal add task")
        };
        let statement = &task.body_syntax[0];
        reset_verify_builds();
        let delivered = super::with_full_type_for_effect(&program, &diagnostics, |access| {
            let result = access
                .canonical_minimal_add_for(item, statement)
                .expect("same-report verified full type");
            let identity = result.backend_identity();
            assert_eq!(
                identity.program_identity,
                std::ptr::from_ref(&program).addr()
            );
            assert_eq!(
                identity.owner.file.source_revision.as_ref(),
                include_bytes!("../examples/core/minimal_add.hum")
            );
            assert_eq!(identity.owner.file.semantic_file_index, 0);
            assert_eq!(
                identity.owner.file.normalized_path.as_ref(),
                "examples/core/minimal_add.hum"
            );
            assert_eq!(identity.source_module, Some("examples.core.minimal_add"));
            assert_eq!(identity.owner.item_path.as_ref(), [0]);
            assert_eq!(identity.owner.item_kind, "task");
            assert_eq!(
                identity
                    .owner
                    .section_slots
                    .iter()
                    .map(|slot| slot.as_ref())
                    .collect::<Vec<_>>(),
                ["allocates", "does"]
            );
            assert_eq!(result.core_prerequisite_names().count(), 7);
            assert_eq!(identity.operand(0).unwrap().4, "Int");
            assert_eq!(identity.operand(1).unwrap().4, "Int");
            true
        });
        assert!(delivered);
        assert_eq!(verify_builds(), 1);
    }

    // WO27 Part 1a: `text_split` checker-shape tests (decision 0021).
    fn text_split_probe_program(source: &str) -> Program {
        Program {
            files: vec![parse_source("text_split_probe.hum", source).file],
        }
    }

    fn text_split_probe_json(source: &str) -> String {
        let program = text_split_probe_program(source);
        full_type_check_json(&program, &[])
    }

    fn count_diagnostic_code(json: &str, code: &str) -> usize {
        json.matches(&format!("\"diagnostic_code\": \"{code}\""))
            .count()
    }

    #[test]
    fn text_split_literal_empty_separator_is_h0636() {
        let source =
            include_str!("../fixtures/diagnostics/text_split_literal_empty_separator_fail.hum");
        let program = text_split_probe_program(source);
        let json = full_type_check_json(&program, &[]);
        assert_eq!(count_diagnostic_code(&json, "H0636"), 1);
        assert!(full_type_check_has_errors(&program, &[]));
    }

    // WO30 Item 4: the arity reason migrates from H0636 to the general
    // call-shape probe (H0640), which runs before the per-builtin probes so
    // the two codes cannot double-fire.
    #[test]
    fn text_split_wrong_arity_is_h0640() {
        let json = text_split_probe_json(
            r#"task split_arity() -> List Text {
  does:
    let pieces = text_split("a,b,c")
    return pieces
}
"#,
        );
        assert_eq!(count_diagnostic_code(&json, "H0640"), 1);
        assert_eq!(count_diagnostic_code(&json, "H0636"), 0);
    }

    // WO30 Item 2: argument-type mismatches moved from H0636 to H0641.
    // H0636 is narrowed to value-level reasons only (stray empty argument,
    // directly-written empty separator).
    #[test]
    fn text_split_non_text_argument_is_h0641() {
        let json = text_split_probe_json(
            r#"task split_types(count: UInt) -> List Text {
  does:
    let pieces = text_split("a,b,c", count)
    return pieces
}
"#,
        );
        assert_eq!(count_diagnostic_code(&json, "H0641"), 1);
        assert_eq!(count_diagnostic_code(&json, "H0636"), 0);
        assert_eq!(count_diagnostic_code(&json, "H0640"), 0);
    }

    #[test]
    fn text_split_valid_calls_have_no_h0636() {
        let json = text_split_probe_json(include_str!("../examples/probes/text_split.hum"));
        assert_eq!(count_diagnostic_code(&json, "H0636"), 0);
        let args_json =
            text_split_probe_json(include_str!("../examples/probes/text_split_args.hum"));
        assert_eq!(count_diagnostic_code(&args_json, "H0636"), 0);
    }

    // Decision 0028 (rework): the builtins behave exactly like user tasks
    // with the same signature. User-task calls carry no static diagnostics
    // for wrong argument type or `-` literals to `UInt` parameters (ledger
    // #21), so neither do these builtins. Both remain infallible at the
    // checker level: no `try`, no H0901.
    // WO30 Item 4: the arity shape now fires H0640 via the general
    // call-shape probe, so it leaves this checker-accepted set.
    // WO30 Item 2: wrong-type and cross-type argument shapes now fire H0641
    // via the general argument-type probe. The negative-literal-to-UInt
    // shape stays checker-accepted here; WO30 Item 3 (H0642) owns it.
    #[test]
    fn uint_to_text_wrong_type_shapes_are_h0641() {
        for source in [
            r#"task render_type() -> Text {
  does:
    return uint_to_text("3")
}
"#,
            r#"task render_cross(n: Int) -> Text {
  does:
    return uint_to_text(n)
}
"#,
        ] {
            let program = text_split_probe_program(source);
            let json = full_type_check_json(&program, &[]);
            assert_eq!(count_diagnostic_code(&json, "H0641"), 1, "{source}");
            assert_eq!(count_diagnostic_code(&json, "H0640"), 0, "{source}");
            assert_eq!(count_diagnostic_code(&json, "H0901"), 0, "{source}");
            assert!(
                full_type_check_has_errors(&program, &[]),
                "wrong-type shape must be rejected: {source}"
            );
        }
    }

    // WO30 Item 2 leaves the negative-literal-to-UInt shape checker-accepted;
    // WO30 Item 3 (H0642) owns its rejection.
    #[test]
    fn uint_to_text_negative_literal_stays_checker_accepted() {
        let source = r#"task render_neg() -> Text {
  does:
    return uint_to_text(-5)
}
"#;
        let program = text_split_probe_program(source);
        let json = full_type_check_json(&program, &[]);
        assert_eq!(count_diagnostic_code(&json, "H0641"), 0);
        assert_eq!(count_diagnostic_code(&json, "H0642"), 0);
        assert!(!full_type_check_has_errors(&program, &[]));
    }

    #[test]
    fn uint_to_text_valid_calls_have_no_h0901() {
        let program = text_split_probe_program(
            r#"task render_valid(n: UInt) -> Text {
  does:
    let a = uint_to_text(42)
    let b = uint_to_text(n)
    let c = uint_to_text(0)
    return c
}
"#,
        );
        let json = full_type_check_json(&program, &[]);
        assert_eq!(count_diagnostic_code(&json, "H0901"), 0);
        assert!(!full_type_check_has_errors(&program, &[]));
    }

    // WO30 Item 2: wrong-type and cross-type argument shapes now fire H0641
    // via the general argument-type probe.
    #[test]
    fn int_to_text_wrong_type_shapes_are_h0641() {
        for source in [
            r#"task render_type() -> Text {
  does:
    return int_to_text("3")
}
"#,
            r#"task render_cross(n: UInt) -> Text {
  does:
    return int_to_text(n)
}
"#,
        ] {
            let program = text_split_probe_program(source);
            let json = full_type_check_json(&program, &[]);
            assert_eq!(count_diagnostic_code(&json, "H0641"), 1, "{source}");
            assert_eq!(count_diagnostic_code(&json, "H0640"), 0, "{source}");
            assert_eq!(count_diagnostic_code(&json, "H0901"), 0, "{source}");
            assert!(
                full_type_check_has_errors(&program, &[]),
                "wrong-type shape must be rejected: {source}"
            );
        }
    }

    // WO30 Item 2: focused H0641 coverage — exactly one diagnostic for the
    // first mismatch, nested calls, module/app scope, builtin precedence,
    // unknown-type silence, list width rules, H0640 masking, user-task
    // mismatches, field/element places, and migrated builtin failure forms.
    #[test]
    fn h0641_reports_only_the_first_mismatch() {
        let json = text_split_probe_json(
            r#"task f() -> Text {
  does:
    return text_split(42, true)
}
"#,
        );
        assert_eq!(count_diagnostic_code(&json, "H0641"), 1);
        assert_eq!(count_diagnostic_code(&json, "H0640"), 0);
    }

    #[test]
    fn h0641_fires_for_nested_call_mismatch() {
        let json = text_split_probe_json(
            r#"task f() -> Text {
  does:
    return text_split(uint_to_text(1), uint_to_text("x"))
}
"#,
        );
        // The outer text_split args are both Text (builtin return); the
        // inner uint_to_text("x") is the Text-vs-UInt mismatch.
        assert_eq!(count_diagnostic_code(&json, "H0641"), 1);
        assert!(full_type_check_has_errors(
            &text_split_probe_program(
                r#"task f() -> Text {
  does:
    return text_split(uint_to_text(1), uint_to_text("x"))
}
"#,
            ),
            &[]
        ));
    }

    #[test]
    fn h0641_fires_in_module_and_app_scope() {
        let json = text_split_probe_json(
            r#"module tests.wo30

task helper(x: UInt) -> UInt {
  does:
    return helper("s")
}

app probe {
  task start() -> Text {
    does:
      return uint_to_text("s")
  }
}
"#,
        );
        assert_eq!(count_diagnostic_code(&json, "H0641"), 2);
    }

    #[test]
    fn h0641_builtin_signature_wins_over_same_named_user_task() {
        let json = text_split_probe_json(
            r#"task uint_to_text(x: Text) -> Text {
  does:
    return x
}

task f() -> Text {
  does:
    return uint_to_text("s")
}
"#,
        );
        // Builtin-table-first (ledger #19): the builtin uint_to_text takes
        // UInt, so the Text literal is H0641 despite the user task's Text
        // parameter.
        assert_eq!(count_diagnostic_code(&json, "H0641"), 1);
    }

    #[test]
    fn h0641_unknown_argument_type_stays_silent() {
        // An element place on a non-list type has no static element type;
        // the call stays silent per decision 0014 honesty.
        let source = r#"task f(n: UInt) -> Text {
  does:
    return uint_to_text(n[0])
}
"#;
        let program = text_split_probe_program(source);
        let json = full_type_check_json(&program, &[]);
        assert_eq!(count_diagnostic_code(&json, "H0641"), 0);
        assert!(!full_type_check_has_errors(&program, &[]));
    }

    #[test]
    fn h0641_list_width_rules_are_accepted() {
        let source = r#"task f(xs: List Text) -> UInt {
  does:
    let a = list_len([1, 2, 3])
    let b = list_len(xs)
    return a
}
"#;
        let program = text_split_probe_program(source);
        let json = full_type_check_json(&program, &[]);
        // A list literal is compatible with `List` and `List T`; a bare
        // `List` parameter accepts any list-typed argument.
        assert_eq!(count_diagnostic_code(&json, "H0641"), 0);
        assert!(!full_type_check_has_errors(&program, &[]));
    }

    #[test]
    fn h0641_list_literal_accepts_list_t_parameter() {
        let source = r#"task g(xs: List Text) -> UInt {
  does:
    return list_len(xs)
}

task f() -> UInt {
  does:
    return g([1, 2])
}
"#;
        let program = text_split_probe_program(source);
        let json = full_type_check_json(&program, &[]);
        assert_eq!(count_diagnostic_code(&json, "H0641"), 0);
        assert!(!full_type_check_has_errors(&program, &[]));
    }

    #[test]
    fn h0640_masks_h0641_on_bad_arity() {
        let json = text_split_probe_json(
            r#"task f() -> Text {
  does:
    return uint_to_text("a", "b")
}
"#,
        );
        assert_eq!(count_diagnostic_code(&json, "H0640"), 1);
        assert_eq!(count_diagnostic_code(&json, "H0641"), 0);
    }

    #[test]
    fn h0641_fires_for_user_task_mismatch() {
        let json = text_split_probe_json(
            r#"task g(x: UInt, y: Text) -> Text {
  does:
    return y
}

task f() -> Text {
  does:
    return g("s", "t")
}
"#,
        );
        assert_eq!(count_diagnostic_code(&json, "H0641"), 1);
        assert_eq!(count_diagnostic_code(&json, "H0640"), 0);
    }

    #[test]
    fn h0641_fires_for_field_and_element_places() {
        let json = text_split_probe_json(
            r#"type Item {
  name: Text
  count: UInt
}

task f(item: Item, xs: List UInt) -> Text {
  does:
    let a = uint_to_text(item.name)
    let b = uint_to_text(xs[0])
    return b
}
"#,
        );
        // item.name is Text (mismatch); xs[0] is UInt (accepted).
        assert_eq!(count_diagnostic_code(&json, "H0641"), 1);
    }

    #[test]
    fn h0641_preserves_migrated_builtin_failure_forms() {
        for (source, form) in [
            (
                r#"task f() -> List Text {
  does:
    return text_split(42, "b")
}
"#,
                "text_split_builtin",
            ),
            (
                r#"app probe {
  uses:
    stdout.write
  starts with:
    run_tool
  task run_tool -> Result Unit, OutputError {
    uses:
      stdout.write
    fails when:
      the output operation fails
    allocates:
      callee-defined allocation behavior
    does:
      let written = try stdout_write(42)
      return written
  }
}
"#,
                "bounded_output_builtin",
            ),
            (
                r#"app probe {
  why:
    reject Text where the hardened reader requires opaque Path
  uses:
    files.read
  starts with:
    run_tool
  task run_tool(input: Text) -> Result Unit, FileReadError {
    uses:
      files.read
    fails when:
      the exact file operation fails
    allocates:
      one bounded file buffer
    does:
      let text = try files_read_text(input)
      return
  }
}
"#,
                "hardened_exact_file_read_builtin",
            ),
        ] {
            let json = text_split_probe_json(source);
            assert_eq!(count_diagnostic_code(&json, "H0641"), 1, "{source}");
            assert!(
                json.contains(&format!("\"failure_form\": \"{form}\"")),
                "missing failure form {form}: {source}"
            );
        }
    }

    #[test]
    fn h0636_keeps_value_reasons_only() {
        // Stray empty argument and directly-written empty separator stay
        // H0636; arity and type reasons moved to H0640/H0641.
        let empty_sep = text_split_probe_json(
            r#"task f() -> List Text {
  does:
    return text_split("a,b", "")
}
"#,
        );
        assert_eq!(count_diagnostic_code(&empty_sep, "H0636"), 1);
        let stray_empty = text_split_probe_json(
            r#"task f() -> List Text {
  does:
    return text_split("a,b", , "c")
}
"#,
        );
        assert_eq!(count_diagnostic_code(&stray_empty, "H0636"), 1);
    }

    #[test]
    fn int_to_text_valid_calls_have_no_h0901() {
        let program = text_split_probe_program(
            r#"task render_valid(n: Int) -> Text {
  does:
    let a = int_to_text(-7)
    let b = int_to_text(n)
    let c = int_to_text(0)
    return c
}
"#,
        );
        let json = full_type_check_json(&program, &[]);
        assert_eq!(count_diagnostic_code(&json, "H0901"), 0);
        assert!(!full_type_check_has_errors(&program, &[]));
    }

    // WO30 Item 4 (PR A): wrong-arity calls to the four builtins and to user
    // tasks fire exactly one H0640 each. The general probe runs before the
    // per-builtin probes, so no migrated code double-fires.
    fn count_non_null_diagnostic_codes(json: &str) -> usize {
        let mut count = 0;
        let mut idx = 0;
        while let Some(found) = json[idx..].find("\"diagnostic_code\":") {
            let rest = json[idx + found + "\"diagnostic_code\":".len()..].trim_start();
            if rest.starts_with("\"H") {
                count += 1;
            }
            idx = idx + found + 1;
        }
        count
    }

    #[test]
    fn wrong_arity_calls_are_exactly_one_h0640() {
        for source in [
            r#"task render_arity() -> Text {
  does:
    return uint_to_text(1, 2)
}
"#,
            r#"task render_arity() -> Text {
  does:
    return int_to_text(1, 2)
}
"#,
            r#"task split_arity() -> List Text {
  does:
    let pieces = text_split("a,b,c")
    return pieces
}
"#,
            r#"task split_arity_extra() -> List Text {
  does:
    let pieces = text_split("a", ",", "!")
    return pieces
}
"#,
            r#"task len_arity(xs: List Text) -> UInt {
  does:
    return list_len()
}
"#,
            r#"task len_arity_extra(xs: List Text) -> UInt {
  does:
    return list_len(xs, xs)
}
"#,
            r#"task helper(a: UInt, b: Text) -> Text {
  does:
    return b
}

task caller() -> Text {
  does:
    return helper(1)
}
"#,
            r#"task helper(a: UInt) -> UInt {
  does:
    return a
}

task caller() -> Text {
  does:
    return uint_to_text(helper(1, 2))
}
"#,
        ] {
            let program = text_split_probe_program(source);
            let json = full_type_check_json(&program, &[]);
            assert_eq!(count_diagnostic_code(&json, "H0640"), 1, "{source}");
            assert_eq!(
                count_non_null_diagnostic_codes(&json),
                1,
                "exactly one diagnostic per arity shape: {source}"
            );
            assert!(full_type_check_has_errors(&program, &[]));
        }
    }

    // WO30 Item 1 (PR A): app-nested checking uses that app's task
    // signatures, so a wrong-arity call to an app-local task fires H0640.
    #[test]
    fn app_scope_wrong_arity_is_h0640() {
        let source = include_str!("../fixtures/diagnostics/call_arity_mismatch_app_fail.hum");
        let program = text_split_probe_program(source);
        let json = full_type_check_json(&program, &[]);
        assert_eq!(count_diagnostic_code(&json, "H0640"), 1);
        assert_eq!(count_non_null_diagnostic_codes(&json), 1);
        assert!(full_type_check_has_errors(&program, &[]));
    }

    // WO30 Item 1 (PR A): module tasks are not visible to app-nested
    // checking — the signature map mirrors `task_return_types`, which only
    // collects the app's own items there — so the probe must not fire on a
    // wrong-arity call to a module task from app scope.
    #[test]
    fn app_scope_does_not_see_module_task_signatures() {
        let json = text_split_probe_json(
            r#"task helper(a: UInt, b: Text) -> Text {
  does:
    return b
}

app probe {
  task start() -> Text {
    does:
      return helper(1)
  }
}
"#,
        );
        assert_eq!(count_diagnostic_code(&json, "H0640"), 0);
    }

    // WO30 Item 4 (PR A): an unresolved callee is owned by the resolver
    // (H0601); the blocked item never reaches the call-shape probe, so no
    // H0640 stacks on it.
    #[test]
    fn unresolved_callee_has_no_h0640() {
        let json = text_split_probe_json(
            r#"task caller() -> Text {
  does:
    return nosuchfn(1, 2, 3)
}
"#,
        );
        assert_eq!(count_diagnostic_code(&json, "H0640"), 0);
    }

    // WO30 Item 4 (PR A): well-shaped calls stay clean, including nested
    // calls and the existing probe fixtures.
    #[test]
    fn valid_call_shapes_have_no_h0640() {
        for source in [
            include_str!("../fixtures/diagnostics/call_arity_mismatch_ok.hum"),
            include_str!("../examples/probes/text_split.hum"),
        ] {
            let program = text_split_probe_program(source);
            let json = full_type_check_json(&program, &[]);
            assert_eq!(count_diagnostic_code(&json, "H0640"), 0, "{source}");
            assert!(
                !full_type_check_has_errors(&program, &[]),
                "valid shape must stay checker-accepted"
            );
        }
    }

    // WO27 Part 1a: a variable separator requires `try` even when bound to a
    // literal — the direct-literal exemption is literal-only.
    #[test]
    fn text_split_variable_separator_requires_try() {
        let json = text_split_probe_json(include_str!(
            "../fixtures/diagnostics/text_split_variable_separator_needs_try_fail.hum"
        ));
        assert_eq!(count_diagnostic_code(&json, "H0901"), 1);
        assert_eq!(count_diagnostic_code(&json, "H0636"), 0);
    }

    // WO27 Part 1a: an exempt literal-separator call is skipped by H0901
    // analysis but never masks a later fallible call in the same body.
    #[test]
    fn text_split_literal_exemption_does_not_mask_later_fallible_call() {
        let masked = text_split_probe_json(
            r#"type ProbeError {
  code: Text
}

task probe(line: Text, sep: Text) -> Result Unit, ProbeError {
  why:
    exempt text_split must not mask a later fallible text_split

  fails when:
    splitting fails

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    let first = text_split(line, ", ")
    let second = text_split(line, sep)
    fail ProbeError.done
}
"#,
        );
        assert_eq!(count_diagnostic_code(&masked, "H0901"), 1);
        let unmasked = text_split_probe_json(
            r#"type ProbeError {
  code: Text
}

task probe(line: Text) -> Result Unit, ProbeError {
  why:
    the exempt call alone raises no H0901

  fails when:
    splitting fails

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    let first = text_split(line, ", ")
    fail ProbeError.done
}
"#,
        );
        assert_eq!(count_diagnostic_code(&unmasked, "H0901"), 0);
    }

    #[test]
    fn h0638_span_marks_the_bad_escape_itself() {
        // Decision 0022: the diagnostic's span marks the bad escape itself
        // (the backslash and its following character), not the whole literal.
        // Backslash inputs built without double-backslash literals
        // (public-readiness).
        let bs = char::from(92).to_string();
        let source = format!(
            "task bad() -> Text {{\n  does:\n    return {}bad{}qescape{}\n}}\n",
            '"', bs, '"'
        );
        let program = Program {
            files: vec![parse_source("h0638_span.hum", &source).file],
        };
        let report = build_report(&program, &[]);
        assert!(full_type_check_has_errors(&program, &[]));
        let statement = &report.items[0].statements[0];
        assert_eq!(statement.status, "rejected_invalid_text_escape_v0");
        assert_eq!(
            statement.diagnostic_code,
            Some(DiagnosticCode::INVALID_TEXT_ESCAPE.as_str())
        );
        // `return "bad\qescape"`: the backslash is at column 16 (1-based).
        let span = statement.call_span.as_ref().expect("H0638 has a span");
        assert_eq!(span.line, 3);
        assert_eq!(span.column, 16);
    }

    #[test]
    fn h0638_trailing_backslash_is_a_checker_error() {
        // Decision 0022: a trailing backslash before the closing quote is a
        // checker error (H0638, unterminated escape), not a silent accept.
        let bs = char::from(92).to_string();
        let source = format!(
            "task trail() -> Text {{\n  does:\n    return {}ab{}{}\n}}\n",
            '"', bs, '"'
        );
        let program = Program {
            files: vec![parse_source("h0638_trailing.hum", &source).file],
        };
        let report = build_report(&program, &[]);
        assert!(full_type_check_has_errors(&program, &[]));
        let statement = &report.items[0].statements[0];
        assert_eq!(statement.status, "rejected_invalid_text_escape_v0");
        assert_eq!(
            statement.diagnostic_code,
            Some(DiagnosticCode::INVALID_TEXT_ESCAPE.as_str())
        );
        // `return "ab\"`: the backslash is at column 15 (1-based).
        let span = statement.call_span.as_ref().expect("H0638 has a span");
        assert_eq!(span.line, 3);
        assert_eq!(span.column, 15);
    }

    #[test]
    fn for_each_over_list_binds_element_type() {
        let program = Program {
            files: vec![
                parse_source(
                    "for_each_list.hum",
                    r#"task each_positive(words: List Text) -> List Text {
  does:
    change out: List Text = []
    for each word in words {
      let added = list_append(change out, word)
    }
    return out
}
"#,
                )
                .file,
            ],
        };
        let report = build_report(&program, &[]);
        assert!(!full_type_check_has_errors(&program, &[]));
        let header = report.items[0]
            .statements
            .iter()
            .find(|s| s.statement_kind == "for_each_header")
            .expect("for_each_header statement");
        assert_eq!(header.status, "accepted_for_each_binding_v0");
        assert_eq!(header.actual_type.as_deref(), Some("Text"));
    }

    #[test]
    fn for_each_over_non_list_stays_unchecked() {
        let program = Program {
            files: vec![
                parse_source(
                    "for_each_non_list.hum",
                    r#"task each_non_list(line: Text) -> List Text {
  does:
    change out: List Text = []
    for each ch in line {
      let added = list_append(change out, ch)
    }
    return out
}
"#,
                )
                .file,
            ],
        };
        let report = build_report(&program, &[]);
        assert!(full_type_check_has_errors(&program, &[]));
        let header = report.items[0]
            .statements
            .iter()
            .find(|s| s.statement_kind == "for_each_header")
            .expect("for_each_header statement");
        assert_eq!(header.status, "unchecked_statement_type_v0");
        assert_eq!(
            header.reason,
            Some("iterator_type_checking_not_implemented")
        );
    }

    #[test]
    fn condition_text_comparison_does_not_collapse_to_name_fact() {
        // Regression: place_type_fact used to snake-normalize the whole
        // `piece != ""` condition to `piece` and return the bound Text fact
        // before condition inference could return Bool, rejecting the `if`.
        let program = Program {
            files: vec![
                parse_source(
                    "condition_guard.hum",
                    r#"task cond_probe(piece: Text) -> Text {
  does:
    if piece != "" {
      return piece
    }
    return "empty"
}
"#,
                )
                .file,
            ],
        };
        assert!(!full_type_check_has_errors(&program, &[]));
        let json = full_type_check_json(&program, &[]);
        assert!(json.contains("\"status\": \"recognized_core_body_types_checked_v0\""));
    }

    #[test]
    fn test_expectation_matching_return_type_accepted() {
        let program = Program {
            files: vec![
                parse_source(
                    "expectation_match.hum",
                    r#"app probeapp {
  why:
    probe

  starts with:
    start_here

  task start_here() -> Unit {
    does:
      let n = probe_target()
  }

  task probe_target() -> UInt {
    does:
      return 7
  }

  test good_shape unit {
    covers:
      probe_target returns its value
    does:
      expect probe_target() returns 7
  }
}
"#,
                )
                .file,
            ],
        };
        let report = build_report(&program, &[]);
        assert!(!full_type_check_has_errors(&program, &[]));
        let expectation = report
            .items
            .iter()
            .flat_map(|item| item.statements.iter())
            .find(|s| s.statement_kind == "test_expectation")
            .expect("test_expectation statement");
        assert_eq!(expectation.status, "accepted_test_expectation_v0");
        assert_eq!(expectation.actual_type.as_deref(), Some("Bool"));
    }

    #[test]
    fn test_expectation_type_mismatch_stays_unchecked() {
        let program = Program {
            files: vec![
                parse_source(
                    "expectation_mismatch.hum",
                    r#"app probeapp {
  why:
    probe

  starts with:
    start_here

  task start_here() -> Unit {
    does:
      let n = probe_target()
  }

  task probe_target() -> UInt {
    does:
      return 7
  }

  test wrong_type unit {
    covers:
      probe_target type mismatch stays unchecked
    does:
      expect probe_target() returns "not a number"
  }
}
"#,
                )
                .file,
            ],
        };
        let report = build_report(&program, &[]);
        assert!(full_type_check_has_errors(&program, &[]));
        let expectation = report
            .items
            .iter()
            .flat_map(|item| item.statements.iter())
            .find(|s| s.statement_kind == "test_expectation")
            .expect("test_expectation statement");
        assert_eq!(expectation.status, "unchecked_statement_type_v0");
        assert_eq!(
            expectation.reason,
            Some("test_expectation_typing_not_implemented")
        );
    }
}
