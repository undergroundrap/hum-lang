use crate::ast::{
    AuthenticatedCanonicalTaskSignature, CanonicalExpression, CanonicalExpressionKind,
    CanonicalTaskSignatureJoinKey, Item, Param, ParsedBinaryOperator, ParsedSourceRange, Program,
    Section, Task,
};
use crate::callable;
use crate::core_body::{self, BodyStatement, CanonicalBodyGrammarReport, CanonicalBodyStatement};
use crate::core_contract;
use crate::core_expr::{self, CoreExpressionPreview};
use crate::core_preview;
use crate::diagnostic::{
    Diagnostic, DiagnosticOccurrenceSet, DiagnosticProjection, Severity, Span,
};
use crate::ir_contract;
use crate::node_id;
use crate::predicate::{self, PredicateFact, RecognitionStatus};
use crate::resolve;
use crate::type_check::{
    self, CanonicalMinimalAddTypeAuthority, CanonicalMinimalAddTypeDecision, CheckedReturnSummary,
};
use crate::typed_failure::{self, FailureFact, ProgramFailureAnalysis};
use crate::version;

pub const CORE_LOWER_SCHEMA: &str = "hum.core_lower.v0";
pub const CORE_LOWER_STATUS: &str = "unverified_core_artifact_v0";

const NON_GOALS: &[&str] = &[
    "no executable semantics",
    "no interpreter",
    "no Hum IR emission",
    "no backend lowering",
    "no independent type checking",
    "no effect checking",
    "no ownership checking",
    "no optimization",
    "no safety proof",
];

pub struct CoreLowerReadinessSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub files: usize,
    pub items: usize,
    pub tasks: usize,
    pub tests: usize,
    pub core_items: usize,
    pub lowered_items: usize,
    pub blocked_items: usize,
    pub lowered_operations: usize,
    pub blocked_operations: usize,
    pub execution_ready: usize,
    pub ir_ready: usize,
    pub errors: usize,
    pub warnings: usize,
    pub resolver_errors: usize,
    pub type_errors: usize,
    pub preview_blocked_statements: usize,
}

pub(crate) struct CoreLowerReport {
    pub(crate) files: usize,
    pub(crate) items: usize,
    pub(crate) tasks: usize,
    pub(crate) tests: usize,
    pub(crate) execution_ready: usize,
    pub(crate) ir_ready: usize,
    pub(crate) errors: usize,
    pub(crate) warnings: usize,
    pub(crate) resolver_errors: usize,
    pub(crate) type_errors: usize,
    pub(crate) preview_blocked_statements: usize,
    pub(crate) core_items: Vec<CoreLowerItem>,
    pub(crate) diagnostic_occurrences: DiagnosticOccurrenceSet,
    pub(crate) diagnostic_projection: DiagnosticProjection,
}

pub(crate) struct CoreLowerItem {
    pub(crate) id: String,
    pub(crate) kind: &'static str,
    pub(crate) name: String,
    pub(crate) span: Span,
    pub(crate) status: &'static str,
    pub(crate) verification_status: &'static str,
    pub(crate) body_status: &'static str,
    pub(crate) grammar_status: &'static str,
    pub(crate) params: Vec<Param>,
    pub(crate) result: Option<String>,
    pub(crate) source_sections: Vec<String>,
    pub(crate) operations: Vec<CoreLowerOperation>,
    pub(crate) blockers: Vec<CoreLowerBlocker>,
    task_signature: CoreLowerTaskSignature,
}

enum CoreLowerTaskSignature {
    NotATask,
    Authenticated(Box<AuthenticatedCanonicalTaskSignature>),
    Rejected(crate::ast::CanonicalTaskSignatureRejection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoreLowerTaskSignatureVerdict {
    NotATask,
    Passed,
    Failed,
}

impl CoreLowerItem {
    pub(crate) fn task_signature_verdict(&self) -> CoreLowerTaskSignatureVerdict {
        match &self.task_signature {
            CoreLowerTaskSignature::NotATask => CoreLowerTaskSignatureVerdict::NotATask,
            CoreLowerTaskSignature::Authenticated(authority) => {
                if authority.matches_lowered_candidate(
                    self.kind,
                    &self.name,
                    &self.span,
                    &self.params,
                    self.result.as_deref(),
                ) {
                    CoreLowerTaskSignatureVerdict::Passed
                } else {
                    CoreLowerTaskSignatureVerdict::Failed
                }
            }
            CoreLowerTaskSignature::Rejected(rejection) => {
                let _private_reason = rejection.reason();
                CoreLowerTaskSignatureVerdict::Failed
            }
        }
    }
}

pub(crate) struct CoreLowerOperation {
    pub(crate) id: String,
    pub(crate) index: usize,
    pub(crate) span: Span,
    pub(crate) surface_text: String,
    pub(crate) source_kind: &'static str,
    pub(crate) source_status: &'static str,
    pub(crate) core_operation: &'static str,
    pub(crate) status: &'static str,
    pub(crate) expression: Option<CoreLowerExpression>,
    pub(crate) reason: Option<&'static str>,
    canonical_minimal_add_type: CoreLowerCanonicalMinimalAddType,
    canonical_minimal_add_identity: Option<CanonicalMinimalAddOperationIdentity>,
}

pub(crate) enum CoreLowerCanonicalMinimalAddType {
    Noncanonical,
    AuthenticatedOutOfScope,
    LegacyCompatibleAdditive,
    UnsupportedTargetLike,
    IntegrityFailure,
    Supported {
        authority: Box<CanonicalMinimalAddTypeAuthority>,
        claim: CanonicalMinimalAddTypeClaim,
    },
}

pub(crate) struct CanonicalMinimalAddTypeClaim {
    statement_index: usize,
    root_node_id: String,
    type_text: &'static str,
}

pub(crate) struct CanonicalMinimalAddOperationIdentity {
    task_join_key: CanonicalTaskSignatureJoinKey,
    statement_index: usize,
    statement_span: Span,
    parser_root_node_id: String,
    parser_root_range: ParsedSourceRange,
    operation_id: String,
}

impl CanonicalMinimalAddOperationIdentity {
    pub(crate) fn matches(&self, expected: &Self) -> bool {
        self.task_join_key.matches(&expected.task_join_key)
            && self.statement_index == expected.statement_index
            && self.statement_span == expected.statement_span
            && self.parser_root_node_id == expected.parser_root_node_id
            && self.parser_root_range == expected.parser_root_range
            && self.operation_id == expected.operation_id
    }
}

impl CoreLowerOperation {
    pub(crate) fn canonical_minimal_add_type(&self) -> &CoreLowerCanonicalMinimalAddType {
        &self.canonical_minimal_add_type
    }

    pub(crate) fn canonical_minimal_add_identity(
        &self,
    ) -> Option<&CanonicalMinimalAddOperationIdentity> {
        self.canonical_minimal_add_identity.as_ref()
    }
}

impl CanonicalMinimalAddTypeClaim {
    pub(crate) fn matches_authority(&self, authority: &CanonicalMinimalAddTypeAuthority) -> bool {
        authority.matches_claim(self.statement_index, &self.root_node_id, self.type_text)
    }
}

pub(crate) struct CoreLowerExpression {
    pub(crate) text: String,
    pub(crate) kind: &'static str,
    pub(crate) status: &'static str,
    pub(crate) ast_status: &'static str,
    pub(crate) root_form: &'static str,
    pub(crate) operator: Option<&'static str>,
    pub(crate) node_count: usize,
    pub(crate) structured: Option<CoreLowerStructuredExpression>,
    structured_authority: Option<CanonicalExpression>,
    pub(crate) type_status: &'static str,
    pub(crate) type_text: Option<String>,
    pub(crate) type_source: Option<&'static str>,
    pub(crate) effect_status: &'static str,
    pub(crate) reason: Option<&'static str>,
}

impl CoreLowerExpression {
    pub(crate) fn structured_authority(&self) -> Option<&CanonicalExpression> {
        self.structured_authority.as_ref()
    }
}

#[derive(Clone)]
pub(crate) struct CoreLowerStructuredExpression {
    pub(crate) provenance: &'static str,
    pub(crate) parser_node_id: String,
    pub(crate) source_range: CoreLowerSourceRange,
    pub(crate) kind: &'static str,
    pub(crate) operator: &'static str,
    pub(crate) children: Vec<CoreLowerStructuredChild>,
}

#[derive(Clone)]
pub(crate) struct CoreLowerStructuredChild {
    pub(crate) index: usize,
    pub(crate) role: &'static str,
    pub(crate) parser_node_id: String,
    pub(crate) source_range: CoreLowerSourceRange,
    pub(crate) kind: &'static str,
    pub(crate) identifier: String,
}

#[derive(Clone)]
pub(crate) struct CoreLowerSourceRange {
    pub(crate) start: Span,
    pub(crate) byte_len: usize,
}

pub(crate) struct CoreLowerBlocker {
    pub(crate) span: Span,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

pub fn core_lower_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum Core lower ({CORE_LOWER_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!(
        "status: {CORE_LOWER_STATUS}\ncore_contract_schema: {}\ncore_preview_schema: {}\nir_contract_schema: {}\n",
        core_contract::CORE_CONTRACT_SCHEMA,
        core_preview::CORE_PREVIEW_SCHEMA,
        ir_contract::IR_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "summary: files={} items={} tasks={} tests={} core_items={} lowered_items={} blocked_items={} lowered_operations={} blocked_operations={} execution_ready=0 ir_ready=0 errors={} warnings={} resolver_errors={} type_errors={} preview_blocked_statements={}\n",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.core_items.len(),
        report.lowered_items(),
        report.blocked_items(),
        report.lowered_operations(),
        report.blocked_operations(),
        report.errors,
        report.warnings,
        report.resolver_errors,
        report.type_errors,
        report.preview_blocked_statements
    ));

    if report.core_items.is_empty() {
        out.push_str("core_items: none\n");
        out.push_str(&predicate::analyze_program(program).place_facts_text());
        return out;
    }

    out.push_str("core_items:\n");
    for item in &report.core_items {
        out.push_str(&format!(
            "  {}:{}:{} [{}] {} `{}` verification={} execution_ready=0\n",
            item.span.file,
            item.span.line,
            item.span.column,
            item.status,
            item.kind,
            item.name,
            item.verification_status
        ));
        out.push_str(&format!(
            "    body: {} grammar={} operations={} blockers={}\n",
            item.body_status,
            item.grammar_status,
            item.operations.len(),
            item.blockers.len()
        ));
        for operation in &item.operations {
            out.push_str(&format!(
                "    {}:{}:{} [{}] {} -> {}\n",
                operation.span.file,
                operation.span.line,
                operation.span.column,
                operation.status,
                operation.source_kind,
                operation.core_operation
            ));
        }
        for blocker in &item.blockers {
            out.push_str(&format!(
                "    blocker {}:{}:{} [{}] {}\n",
                blocker.span.file,
                blocker.span.line,
                blocker.span.column,
                blocker.status,
                blocker.reason
            ));
        }
    }

    out.push_str(&predicate::analyze_program(program).place_facts_text());
    out
}

pub fn core_lower_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", CORE_LOWER_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "lowering_status", CORE_LOWER_STATUS, true);
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
    push_items(&mut out, &report.core_items, 2, true);
    push_indent(&mut out, 2);
    push_json_string(&mut out, "predicate_place_facts");
    out.push_str(": ");
    out.push_str(&predicate::analyze_program(program).place_facts_json());
    out.push_str(",\n");
    push_string_array(&mut out, 2, "non_goals_v0", NON_GOALS, false);
    out.push_str("}\n");
    out
}

pub fn core_lower_readiness_summary(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> CoreLowerReadinessSummary {
    let report = build_report(program, diagnostics);
    CoreLowerReadinessSummary {
        schema: CORE_LOWER_SCHEMA,
        status: CORE_LOWER_STATUS,
        files: report.files,
        items: report.items,
        tasks: report.tasks,
        tests: report.tests,
        core_items: report.core_items.len(),
        lowered_items: report.lowered_items(),
        blocked_items: report.blocked_items(),
        lowered_operations: report.lowered_operations(),
        blocked_operations: report.blocked_operations(),
        execution_ready: 0,
        ir_ready: 0,
        errors: report.errors,
        warnings: report.warnings,
        resolver_errors: report.resolver_errors,
        type_errors: report.type_errors,
        preview_blocked_statements: report.preview_blocked_statements,
    }
}
pub(crate) fn build_core_lower_report(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> CoreLowerReport {
    let preview_authority = core_preview::diagnostic_occurrence_set(program, diagnostics);
    build_core_lower_report_from_preview(program, diagnostics, &preview_authority)
        .expect("Core lower must carry one producer-supplied preview projection")
}

#[cfg(test)]
pub(crate) enum CoreLowerTreeCorruption {
    ReorderChildren,
    DuplicateChildIdentity,
    ForeignChildIdentity(String),
    ForeignChildRange(CoreLowerSourceRange),
    IncorrectIdentifierSpelling(String),
    CoherentForeignProjection(CoreLowerStructuredExpression),
    CoherentRangeRelocation {
        file: String,
        line_offset: usize,
        column_offset: usize,
    },
    OverflowSizedRange,
    StructuralOverclaim,
}

#[cfg(test)]
pub(crate) fn corrupt_first_structured_expression_for_test(
    report: &mut CoreLowerReport,
    corruption: CoreLowerTreeCorruption,
) -> Result<(), &'static str> {
    let structured = report
        .core_items
        .iter_mut()
        .flat_map(|item| &mut item.operations)
        .filter_map(|operation| operation.expression.as_mut())
        .find_map(|expression| expression.structured.as_mut())
        .ok_or("structured_expression_absent_v0")?;
    if structured.children.len() != 2 {
        return Err("structured_expression_child_count_unexpected_v0");
    }
    match corruption {
        CoreLowerTreeCorruption::ReorderChildren => structured.children.swap(0, 1),
        CoreLowerTreeCorruption::DuplicateChildIdentity => {
            structured.children[1].parser_node_id = structured.children[0].parser_node_id.clone();
        }
        CoreLowerTreeCorruption::ForeignChildIdentity(identity) => {
            structured.children[1].parser_node_id = identity;
        }
        CoreLowerTreeCorruption::ForeignChildRange(range) => {
            structured.children[1].source_range = range;
        }
        CoreLowerTreeCorruption::IncorrectIdentifierSpelling(identifier) => {
            structured.children[0].identifier = identifier;
        }
        CoreLowerTreeCorruption::CoherentForeignProjection(foreign) => {
            *structured = foreign;
        }
        CoreLowerTreeCorruption::CoherentRangeRelocation {
            file,
            line_offset,
            column_offset,
        } => {
            let ranges = std::iter::once(&mut structured.source_range).chain(
                structured
                    .children
                    .iter_mut()
                    .map(|child| &mut child.source_range),
            );
            for range in ranges {
                range.start.file = file.clone();
                range.start.line = range
                    .start
                    .line
                    .checked_add(line_offset)
                    .ok_or("structured_expression_relocation_overflow_v0")?;
                range.start.column = range
                    .start
                    .column
                    .checked_add(column_offset)
                    .ok_or("structured_expression_relocation_overflow_v0")?;
            }
        }
        CoreLowerTreeCorruption::OverflowSizedRange => {
            structured.source_range.start.column = usize::MAX;
            structured.source_range.byte_len = usize::MAX;
        }
        CoreLowerTreeCorruption::StructuralOverclaim => structured.kind = "call",
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn corrupt_first_canonical_minimal_add_claim_for_test(
    report: &mut CoreLowerReport,
) -> Result<(), &'static str> {
    let claim = report
        .core_items
        .iter_mut()
        .flat_map(|item| &mut item.operations)
        .find_map(
            |operation| match &mut operation.canonical_minimal_add_type {
                CoreLowerCanonicalMinimalAddType::Supported { claim, .. } => Some(claim),
                _ => None,
            },
        )
        .ok_or("canonical_minimal_add_claim_absent_v0")?;
    claim.type_text = "UInt";
    Ok(())
}

#[cfg(test)]
pub(crate) fn corrupt_first_canonical_minimal_add_public_and_claim_for_test(
    report: &mut CoreLowerReport,
) -> Result<(), &'static str> {
    let operation = report
        .core_items
        .iter_mut()
        .flat_map(|item| &mut item.operations)
        .find(|operation| {
            matches!(
                operation.canonical_minimal_add_type,
                CoreLowerCanonicalMinimalAddType::Supported { .. }
            )
        })
        .ok_or("canonical_minimal_add_supported_operation_absent_v0")?;
    let expression = operation
        .expression
        .as_mut()
        .ok_or("canonical_minimal_add_expression_absent_v0")?;
    expression.type_text = Some("UInt".to_string());
    let CoreLowerCanonicalMinimalAddType::Supported { claim, .. } =
        &mut operation.canonical_minimal_add_type
    else {
        unreachable!("selected supported operation")
    };
    claim.type_text = "UInt";
    Ok(())
}

#[cfg(test)]
pub(crate) fn substitute_first_canonical_minimal_add_state_for_test(
    target: &mut CoreLowerReport,
    donor: &mut CoreLowerReport,
) -> Result<(), &'static str> {
    fn supported_state(
        report: &mut CoreLowerReport,
    ) -> Option<&mut CoreLowerCanonicalMinimalAddType> {
        report
            .core_items
            .iter_mut()
            .flat_map(|item| &mut item.operations)
            .find_map(|operation| {
                matches!(
                    operation.canonical_minimal_add_type,
                    CoreLowerCanonicalMinimalAddType::Supported { .. }
                )
                .then_some(&mut operation.canonical_minimal_add_type)
            })
    }
    let donor = supported_state(donor).ok_or("canonical_minimal_add_donor_state_absent_v0")?;
    let donor = std::mem::replace(donor, CoreLowerCanonicalMinimalAddType::IntegrityFailure);
    let target = supported_state(target).ok_or("canonical_minimal_add_target_state_absent_v0")?;
    *target = donor;
    Ok(())
}

#[cfg(test)]
pub(crate) fn substitute_first_canonical_minimal_add_identity_for_test(
    target: &mut CoreLowerReport,
    donor: &mut CoreLowerReport,
) -> Result<(), &'static str> {
    fn supported_operation(report: &mut CoreLowerReport) -> Option<&mut CoreLowerOperation> {
        report
            .core_items
            .iter_mut()
            .flat_map(|item| &mut item.operations)
            .find(|operation| {
                matches!(
                    operation.canonical_minimal_add_type,
                    CoreLowerCanonicalMinimalAddType::Supported { .. }
                )
            })
    }
    let donor = supported_operation(donor)
        .and_then(|operation| operation.canonical_minimal_add_identity.take())
        .ok_or("canonical_minimal_add_donor_identity_absent_v0")?;
    let target =
        supported_operation(target).ok_or("canonical_minimal_add_target_identity_absent_v0")?;
    target.canonical_minimal_add_identity = Some(donor);
    Ok(())
}

pub(crate) fn build_core_lower_report_from_preview(
    program: &Program,
    diagnostics: &[Diagnostic],
    preview_authority: &DiagnosticOccurrenceSet,
) -> Result<CoreLowerReport, crate::diagnostic::DiagnosticInvariantError> {
    let resolve_summary = resolve::resolve_readiness_summary(program, diagnostics);
    let type_check_summary = type_check::type_check_summary(program, diagnostics);
    let core_preview_summary = core_preview::core_preview_readiness_summary(program, diagnostics);
    let checked_returns = type_check::checked_return_summaries(program, diagnostics);
    let failure_analysis = typed_failure::analyze_program(program);
    let predicate_facts = predicate::analyze_program(program);
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count()
        + callable::stage_blockers(program, "core_lower");
    let warnings = diagnostics.len().saturating_sub(errors);
    let mut core_items = Vec::new();
    for file in &program.files {
        collect_items(
            program,
            diagnostics,
            &file.items,
            &checked_returns,
            errors,
            resolve_summary.resolver_errors,
            type_check_summary.type_errors,
            &failure_analysis,
            predicate_facts.facts(),
            &mut core_items,
        );
    }

    Ok(CoreLowerReport {
        files: program.files.len(),
        items: count_items(program),
        tasks: count_kind(program, "task"),
        tests: count_kind(program, "test"),
        execution_ready: 0,
        ir_ready: 0,
        errors,
        warnings,
        resolver_errors: resolve_summary.resolver_errors,
        type_errors: type_check_summary.type_errors,
        preview_blocked_statements: core_preview_summary.blocked_statements,
        core_items,
        diagnostic_occurrences: preview_authority.clone(),
        diagnostic_projection: DiagnosticProjection::from_upstream(
            "core_lower",
            preview_authority,
        )?,
    })
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> CoreLowerReport {
    build_core_lower_report(program, diagnostics)
}

#[cfg(test)]
pub(crate) fn diagnostic_projection_from_preview(
    preview_authority: &DiagnosticOccurrenceSet,
) -> Result<DiagnosticProjection, crate::diagnostic::DiagnosticInvariantError> {
    DiagnosticProjection::from_upstream("core_lower", preview_authority)
}

#[allow(clippy::too_many_arguments)]
fn collect_items(
    program: &Program,
    diagnostics: &[Diagnostic],
    items: &[Item],
    checked_returns: &[CheckedReturnSummary],
    source_errors: usize,
    resolver_errors: usize,
    type_errors: usize,
    failure_analysis: &ProgramFailureAnalysis,
    predicate_facts: &[PredicateFact],
    core_items: &mut Vec<CoreLowerItem>,
) {
    for item in items {
        if let Some(core_item) = core_item(
            program,
            diagnostics,
            item,
            checked_returns,
            source_errors,
            resolver_errors,
            type_errors,
            failure_analysis,
            predicate_facts,
        ) {
            core_items.push(core_item);
        }
        if let Item::App(app) = item {
            collect_items(
                program,
                diagnostics,
                &app.items,
                checked_returns,
                source_errors,
                resolver_errors,
                type_errors,
                failure_analysis,
                predicate_facts,
                core_items,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn core_item(
    program: &Program,
    diagnostics: &[Diagnostic],
    item: &Item,
    checked_returns: &[CheckedReturnSummary],
    source_errors: usize,
    resolver_errors: usize,
    type_errors: usize,
    failure_analysis: &ProgramFailureAnalysis,
    predicate_facts: &[PredicateFact],
) -> Option<CoreLowerItem> {
    let does = item_sections(item)
        .iter()
        .find(|section| section.name == "does")?;
    let body = core_body::analyze_does_section_for_lowering(
        program
            .canonical_core_expectation(item, does)
            .expect("live Core item must have parser authority"),
    );
    let failure_analysis = match item {
        Item::Task(task) => failure_analysis.task(task).cloned().unwrap_or_default(),
        _ => Default::default(),
    };
    let task_signature = match item {
        Item::Task(task) => match program.authenticate_canonical_task_signature(task) {
            Ok(authority) => CoreLowerTaskSignature::Authenticated(Box::new(authority)),
            Err(rejection) => CoreLowerTaskSignature::Rejected(rejection),
        },
        _ => CoreLowerTaskSignature::NotATask,
    };
    let authenticated_task_signature = match &task_signature {
        CoreLowerTaskSignature::Authenticated(authority) => Some(authority.as_ref()),
        CoreLowerTaskSignature::NotATask | CoreLowerTaskSignature::Rejected(_) => None,
    };
    let operation_context = CoreLowerOperationContext {
        program,
        diagnostics,
        item,
        task_signature: authenticated_task_signature,
        checked_returns,
        failure_facts: &failure_analysis.facts,
    };
    let operations = lower_operations(&operation_context, &body, predicate_facts);
    let mut blockers = item_blockers(
        item,
        &body,
        &operations,
        source_errors,
        resolver_errors,
        type_errors,
    );
    add_brace_blockers(item, &operations, &mut blockers);
    let status = item_status(
        &body,
        source_errors,
        resolver_errors,
        type_errors,
        &blockers,
    );
    Some(CoreLowerItem {
        id: node_id::span(
            "core-item",
            item.span(),
            &format!("{} {}", item.kind(), item.name()),
        ),
        kind: item.kind(),
        name: item.name().to_string(),
        span: portable_span(item.span()),
        status,
        verification_status: "unverified_v0",
        body_status: body.status,
        grammar_status: body.grammar_status,
        params: item_params(item).to_vec(),
        result: item_result(item).map(str::to_string),
        source_sections: item_sections(item)
            .iter()
            .map(|section| section.name.clone())
            .collect(),
        operations,
        blockers,
        task_signature,
    })
}

struct CoreLowerOperationContext<'a> {
    program: &'a Program,
    diagnostics: &'a [Diagnostic],
    item: &'a Item,
    task_signature: Option<&'a AuthenticatedCanonicalTaskSignature>,
    checked_returns: &'a [CheckedReturnSummary],
    failure_facts: &'a std::collections::BTreeMap<usize, FailureFact>,
}

fn lower_operations(
    context: &CoreLowerOperationContext<'_>,
    body: &CanonicalBodyGrammarReport,
    predicate_facts: &[PredicateFact],
) -> Vec<CoreLowerOperation> {
    let mut operations = body
        .statements
        .iter()
        .enumerate()
        .map(|(index, statement)| lower_operation(context, index, statement))
        .collect::<Vec<_>>();
    if let Item::Task(task) = context.item {
        let first_predicate_index = operations.len();
        operations.extend(
            predicate_facts
                .iter()
                .filter(|fact| fact.task_span == task.span)
                .filter(|fact| fact.status != RecognitionStatus::NonExecutableProse)
                .enumerate()
                .map(|(offset, fact)| {
                    lower_predicate_operation(first_predicate_index + offset, fact)
                }),
        );
    }
    operations
}

fn lower_predicate_operation(index: usize, fact: &PredicateFact) -> CoreLowerOperation {
    let accepted = fact.status == RecognitionStatus::RecognizedTyped;
    let expression = accepted.then(|| {
        let preview = core_preview::predicate_expression_preview_for_lowering(fact);
        expression_from_preview(&preview)
    });
    CoreLowerOperation {
        id: node_id::span(
            "core-op",
            &fact.line_span,
            &format!("{index} predicate {}", fact.status.as_str()),
        ),
        index,
        span: portable_span(&fact.line_span),
        surface_text: fact.text.clone(),
        source_kind: "contract_predicate",
        source_status: fact.status.as_str(),
        core_operation: if accepted {
            "checked_contract_predicate_v2"
        } else {
            "blocked_contract_predicate_v2"
        },
        status: if accepted {
            "lowered_unverified_operation_v0"
        } else {
            "blocked_operation_v0"
        },
        expression,
        reason: Some(fact.reason),
        canonical_minimal_add_type: CoreLowerCanonicalMinimalAddType::Noncanonical,
        canonical_minimal_add_identity: None,
    }
}

fn lower_operation(
    context: &CoreLowerOperationContext<'_>,
    index: usize,
    bound_statement: &CanonicalBodyStatement,
) -> CoreLowerOperation {
    let program = context.program;
    let diagnostics = context.diagnostics;
    let item = context.item;
    let task_signature = context.task_signature;
    let checked_returns = context.checked_returns;
    let failure_fact = context.failure_facts.get(&index);
    let statement = bound_statement.statement();
    let canonical_expression = bound_statement.canonical_expression();
    let type_decision = type_check::canonical_minimal_add_type_for_operation(
        program,
        diagnostics,
        item,
        task_signature,
        index,
        bound_statement,
    );
    let canonical_minimal_add_identity = task_signature.and_then(|authority| {
        canonical_minimal_add_operation_identity(authority, index, bound_statement)
    });
    if let Some(fact) = failure_fact
        && fact.diagnostic_code
            == Some(crate::diagnostic::DiagnosticCode::UNSUPPORTED_TRY_EXPRESSION)
    {
        return CoreLowerOperation {
            id: node_id::span(
                "core-op",
                &statement.span,
                &format!("{index} blocked_unsupported_try"),
            ),
            index,
            span: portable_span(&statement.span),
            surface_text: statement.text.clone(),
            source_kind: statement.kind,
            source_status: statement.status,
            core_operation: "blocked_unsupported_try_expression",
            status: "blocked_operation_v0",
            expression: None,
            reason: fact.reason.or(Some("unsupported_try_expression_shape_v0")),
            canonical_minimal_add_type: lower_canonical_minimal_add_type(type_decision, None),
            canonical_minimal_add_identity,
        };
    }
    let (core_operation, status, fallback_reason) = core_operation_for(statement);
    let checked_return = checked_return_for_statement(item, statement, checked_returns);
    let mut expression = expression_text_for_statement(statement)
        .map(|text| lower_expression(text, checked_return, canonical_expression));
    if statement.status == "unsupported_v0" {
        expression = None;
    }
    let canonical_minimal_add_type =
        lower_canonical_minimal_add_type(type_decision, expression.as_mut());
    CoreLowerOperation {
        id: node_id::span(
            "core-op",
            &statement.span,
            &format!("{} {}", index, core_operation),
        ),
        index,
        span: portable_span(&statement.span),
        surface_text: statement.text.clone(),
        source_kind: statement.kind,
        source_status: statement.status,
        core_operation,
        status,
        expression,
        reason: statement.reason.or(fallback_reason),
        canonical_minimal_add_type,
        canonical_minimal_add_identity,
    }
}

fn lower_canonical_minimal_add_type(
    decision: CanonicalMinimalAddTypeDecision,
    expression: Option<&mut CoreLowerExpression>,
) -> CoreLowerCanonicalMinimalAddType {
    match decision {
        CanonicalMinimalAddTypeDecision::Supported(authority) => {
            if let Some(expression) = expression {
                expression.type_status =
                    core_expr::CORE_EXPRESSION_CHECKED_CANONICAL_MINIMAL_ADD_TYPE_STATUS;
                expression.type_text = Some(authority.produced_type().to_string());
                expression.type_source =
                    Some(core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_SOURCE);
            }
            let claim = CanonicalMinimalAddTypeClaim {
                statement_index: authority.statement_index(),
                root_node_id: authority.root_node_id().to_string(),
                type_text: authority.produced_type(),
            };
            CoreLowerCanonicalMinimalAddType::Supported { authority, claim }
        }
        CanonicalMinimalAddTypeDecision::AuthenticatedOutOfScope => {
            CoreLowerCanonicalMinimalAddType::AuthenticatedOutOfScope
        }
        CanonicalMinimalAddTypeDecision::LegacyCompatibleAdditive => {
            CoreLowerCanonicalMinimalAddType::LegacyCompatibleAdditive
        }
        CanonicalMinimalAddTypeDecision::UnsupportedTargetLike => {
            if let Some(expression) = expression {
                expression.type_status =
                    core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_UNAVAILABLE_STATUS;
                expression.type_text = None;
                expression.type_source = None;
            }
            CoreLowerCanonicalMinimalAddType::UnsupportedTargetLike
        }
        CanonicalMinimalAddTypeDecision::IntegrityFailure => {
            if let Some(expression) = expression {
                expression.type_status =
                    core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_UNAVAILABLE_STATUS;
                expression.type_text = None;
                expression.type_source = None;
            }
            CoreLowerCanonicalMinimalAddType::IntegrityFailure
        }
        CanonicalMinimalAddTypeDecision::Noncanonical => {
            CoreLowerCanonicalMinimalAddType::Noncanonical
        }
    }
}

fn core_operation_for(
    statement: &BodyStatement,
) -> (&'static str, &'static str, Option<&'static str>) {
    if statement.status == "unsupported_v0" {
        return (
            "blocked_surface_statement",
            "blocked_operation_v0",
            statement.reason.or(Some("not_in_core_lower_v0")),
        );
    }

    match statement.kind {
        "return" => ("return", "lowered_unverified_operation_v0", None),
        "fail" => ("fail", "lowered_unverified_operation_v0", None),
        "let_binding" => ("let_binding", "lowered_unverified_operation_v0", None),
        "mutable_binding" => ("mutable_binding", "lowered_unverified_operation_v0", None),
        "set_place" => ("set_place", "lowered_unverified_operation_v0", None),
        "if_header" => ("if_statement", "lowered_unverified_operation_v0", None),
        "while_header" => ("while_loop", "lowered_unverified_operation_v0", None),
        "for_each_header" => ("for_each", "lowered_unverified_operation_v0", None),
        "for_index_header" => ("for_index", "lowered_unverified_operation_v0", None),
        "loop_header" => ("loop", "lowered_unverified_operation_v0", None),
        "block_close" => ("block_close", "lowered_unverified_operation_v0", None),
        "record_field_initializer" => (
            "record_construction_field",
            "blocked_operation_v0",
            Some("record_literal_lowering_not_implemented"),
        ),
        "nested_intent_header" => (
            "contract_context",
            "blocked_operation_v0",
            Some("nested_intent_lowering_not_implemented"),
        ),
        "test_expectation" => (
            "test_expectation",
            "blocked_operation_v0",
            Some("test_body_not_core_runtime"),
        ),
        _ => (
            "blocked_surface_statement",
            "blocked_operation_v0",
            Some("not_in_core_lower_v0"),
        ),
    }
}

fn lower_expression(
    text: &str,
    checked_return: Option<&CheckedReturnSummary>,
    canonical_expression: Option<&CanonicalExpression>,
) -> CoreLowerExpression {
    let mut preview = core_expr::analyze_expression(text);
    if let Some(checked_return) = checked_return {
        let type_status = if checked_return.status == "accepted_return_expression_v0" {
            core_expr::CORE_EXPRESSION_CHECKED_TRIVIAL_RETURN_TYPE_STATUS
        } else {
            core_expr::CORE_EXPRESSION_CHECKED_TRIVIAL_RETURN_MISMATCH_STATUS
        };
        core_expr::annotate_expression_type(
            &mut preview,
            type_status,
            checked_return.actual_type.as_deref(),
            checked_return.type_source,
        );
    }
    let mut expression = expression_from_preview(&preview);
    if let Some(canonical_expression) = canonical_expression
        && let Some(structured) = structured_minimal_add_expression(canonical_expression)
    {
        expression.structured = Some(structured);
        expression.structured_authority = Some(canonical_expression.clone());
    }
    expression
}

fn expression_from_preview(preview: &CoreExpressionPreview) -> CoreLowerExpression {
    CoreLowerExpression {
        text: preview.text.clone(),
        kind: preview.kind,
        status: preview.status,
        ast_status: preview.ast.status,
        root_form: preview.ast.root.form,
        operator: preview.ast.root.operator,
        node_count: preview.ast.node_count,
        structured: None,
        structured_authority: None,
        type_status: preview.ast.type_status,
        type_text: preview.ast.type_text.clone(),
        type_source: preview.ast.type_source,
        effect_status: preview.ast.effect_status,
        reason: preview.reason.or(preview.ast.root.reason),
    }
}

fn structured_minimal_add_expression(
    expression: &CanonicalExpression,
) -> Option<CoreLowerStructuredExpression> {
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

    Some(CoreLowerStructuredExpression {
        provenance: "parser_owned_canonical_expression_v0",
        parser_node_id: expression.node_id.as_str().to_string(),
        source_range: lower_source_range(&expression.range),
        kind: "binary",
        operator: "add",
        children: vec![
            CoreLowerStructuredChild {
                index: 0,
                role: "left",
                parser_node_id: left.node_id.as_str().to_string(),
                source_range: lower_source_range(&left.range),
                kind: "identifier",
                identifier: left_name.clone(),
            },
            CoreLowerStructuredChild {
                index: 1,
                role: "right",
                parser_node_id: right.node_id.as_str().to_string(),
                source_range: lower_source_range(&right.range),
                kind: "identifier",
                identifier: right_name.clone(),
            },
        ],
    })
}

fn lower_source_range(range: &ParsedSourceRange) -> CoreLowerSourceRange {
    CoreLowerSourceRange {
        start: portable_span(&range.start),
        byte_len: range.byte_len,
    }
}

pub(crate) fn canonical_minimal_add_operation_identity(
    task_signature: &AuthenticatedCanonicalTaskSignature,
    statement_index: usize,
    statement: &CanonicalBodyStatement,
) -> Option<CanonicalMinimalAddOperationIdentity> {
    let root = statement.canonical_expression()?;
    if !matches!(
        root.kind,
        CanonicalExpressionKind::Binary {
            operator: ParsedBinaryOperator::Add,
            ..
        }
    ) {
        return None;
    }
    let (core_operation, _, _) = core_operation_for(statement.statement());
    let operation_id = node_id::span(
        "core-op",
        &statement.statement().span,
        &format!("{} {}", statement_index, core_operation),
    );
    Some(CanonicalMinimalAddOperationIdentity {
        task_join_key: task_signature.join_key(),
        statement_index,
        statement_span: statement.statement().span.clone(),
        parser_root_node_id: root.node_id.as_str().to_string(),
        parser_root_range: root.range.clone(),
        operation_id,
    })
}

fn checked_return_for_statement<'a>(
    item: &Item,
    statement: &BodyStatement,
    checked_returns: &'a [CheckedReturnSummary],
) -> Option<&'a CheckedReturnSummary> {
    if item.kind() != "task" || statement.kind != "return" {
        return None;
    }
    let expression_text = strip_keyword(&statement.text, "return")?.trim();
    let span = portable_span(&statement.span);
    checked_returns.iter().find(|checked_return| {
        checked_return.owner_kind == "task"
            && checked_return.owner_name == item.name()
            && checked_return.source_span == span
            && checked_return.expression_text == expression_text
            && checked_return.actual_type.is_some()
    })
}

fn item_blockers(
    item: &Item,
    body: &CanonicalBodyGrammarReport,
    operations: &[CoreLowerOperation],
    source_errors: usize,
    resolver_errors: usize,
    type_errors: usize,
) -> Vec<CoreLowerBlocker> {
    let mut blockers = Vec::new();
    if source_errors > 0 {
        blockers.push(blocker(
            item.span(),
            "blocked_by_source_errors",
            "source_errors_must_be_fixed_before_core_lowering",
        ));
    }
    if resolver_errors > 0 {
        blockers.push(blocker(
            item.span(),
            "blocked_by_resolver_errors",
            "checked_resolver_errors_must_be_fixed_before_core_lowering",
        ));
    }
    if type_errors > 0 {
        blockers.push(blocker(
            item.span(),
            "blocked_by_type_errors",
            "type_errors_must_be_fixed_before_core_lowering",
        ));
    }
    if body.meaningful_lines == 0 {
        blockers.push(blocker(
            item.span(),
            "empty_body",
            "no_core_operations_to_lower",
        ));
    }
    for operation in operations {
        if operation.status == "blocked_operation_v0" {
            blockers.push(CoreLowerBlocker {
                span: operation.span.clone(),
                status: "blocked_operation_v0",
                reason: operation.reason.unwrap_or("operation_not_lowerable_v0"),
            });
        }
    }
    blockers
}

fn add_brace_blockers(
    item: &Item,
    operations: &[CoreLowerOperation],
    blockers: &mut Vec<CoreLowerBlocker>,
) {
    let mut depth = 0usize;
    for operation in operations {
        if operation.core_operation == "block_close" {
            if depth == 0 {
                blockers.push(CoreLowerBlocker {
                    span: operation.span.clone(),
                    status: "blocked_operation_v0",
                    reason: "unmatched_block_close",
                });
            } else {
                depth -= 1;
            }
        } else if opens_block(operation.core_operation) {
            depth += 1;
        }
    }
    if depth > 0 {
        blockers.push(blocker(
            item.span(),
            "blocked_operation_v0",
            "unclosed_core_block",
        ));
    }
}

fn opens_block(core_operation: &str) -> bool {
    matches!(
        core_operation,
        "if_statement" | "while_loop" | "for_each" | "for_index" | "loop"
    )
}

fn item_status(
    body: &CanonicalBodyGrammarReport,
    source_errors: usize,
    resolver_errors: usize,
    type_errors: usize,
    blockers: &[CoreLowerBlocker],
) -> &'static str {
    if source_errors > 0 {
        "blocked_by_source_errors"
    } else if resolver_errors > 0 {
        "blocked_by_resolver_errors"
    } else if type_errors > 0 {
        "blocked_by_type_errors"
    } else if body.meaningful_lines == 0 {
        "empty_body"
    } else if blockers.is_empty() {
        "lowered_unverified_core_v0"
    } else {
        "blocked_before_core_execution"
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

fn blocker(span: &Span, status: &'static str, reason: &'static str) -> CoreLowerBlocker {
    CoreLowerBlocker {
        span: portable_span(span),
        status,
        reason,
    }
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
        Item::Task(Task { result, .. }) => result.as_deref(),
        _ => None,
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

fn count_kind(program: &Program, kind: &str) -> usize {
    program
        .files
        .iter()
        .map(|file| count_kind_in(&file.items, kind))
        .sum()
}

fn count_kind_in(items: &[Item], kind: &str) -> usize {
    items
        .iter()
        .map(|item| {
            usize::from(item.kind() == kind)
                + match item {
                    Item::App(app) => count_kind_in(&app.items, kind),
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

impl CoreLowerReport {
    pub(crate) fn lowered_items(&self) -> usize {
        self.core_items
            .iter()
            .filter(|item| item.status == "lowered_unverified_core_v0")
            .count()
    }

    pub(crate) fn blocked_items(&self) -> usize {
        self.core_items
            .iter()
            .filter(|item| item.status != "lowered_unverified_core_v0")
            .count()
    }

    pub(crate) fn lowered_operations(&self) -> usize {
        self.core_items
            .iter()
            .flat_map(|item| &item.operations)
            .filter(|operation| operation.status == "lowered_unverified_operation_v0")
            .count()
    }

    pub(crate) fn blocked_operations(&self) -> usize {
        self.core_items
            .iter()
            .flat_map(|item| &item.operations)
            .filter(|operation| operation.status == "blocked_operation_v0")
            .count()
    }
}

fn push_summary(out: &mut String, report: &CoreLowerReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"core_items\": {}, \"lowered_items\": {}, \"blocked_items\": {}, \"lowered_operations\": {}, \"blocked_operations\": {}, \"execution_ready\": 0, \"ir_ready\": 0, \"errors\": {}, \"warnings\": {}, \"resolver_errors\": {}, \"type_errors\": {}, \"preview_blocked_statements\": {}",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.core_items.len(),
        report.lowered_items(),
        report.blocked_items(),
        report.lowered_operations(),
        report.blocked_operations(),
        report.errors,
        report.warnings,
        report.resolver_errors,
        report.type_errors,
        report.preview_blocked_statements
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_items(out: &mut String, items: &[CoreLowerItem], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"core_items\": [\n");
    for (index, item) in items.iter().enumerate() {
        push_item(out, item, indent + 2, index + 1 < items.len());
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_item(out: &mut String, item: &CoreLowerItem, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &item.id, true);
    push_string_field(out, indent + 2, "kind", item.kind, true);
    push_string_field(out, indent + 2, "name", &item.name, true);
    push_span_field(out, indent + 2, "source_span", &item.span, true);
    push_string_field(out, indent + 2, "status", item.status, true);
    push_string_field(
        out,
        indent + 2,
        "verification_status",
        item.verification_status,
        true,
    );
    push_usize_field(out, indent + 2, "execution_ready", 0, true);
    push_string_field(out, indent + 2, "body_status", item.body_status, true);
    push_string_field(out, indent + 2, "grammar_status", item.grammar_status, true);
    push_params(out, &item.params, indent + 2, true);
    push_optional_string_field(out, indent + 2, "result", item.result.as_deref(), true);
    push_string_array_refs(
        out,
        indent + 2,
        "source_sections",
        &item.source_sections,
        true,
    );
    push_operations(out, &item.operations, indent + 2, true);
    push_blockers(out, &item.blockers, indent + 2, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_params(out: &mut String, params: &[Param], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"params\": [");
    for (index, param) in params.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str("{\"name\": ");
        push_json_string(out, &param.name);
        out.push_str(", \"type\": ");
        push_json_string(out, &param.ty);
        out.push('}');
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_operations(
    out: &mut String,
    operations: &[CoreLowerOperation],
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"operations\": [\n");
    for (index, operation) in operations.iter().enumerate() {
        push_operation(out, operation, indent + 2, index + 1 < operations.len());
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_operation(out: &mut String, operation: &CoreLowerOperation, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &operation.id, true);
    push_usize_field(out, indent + 2, "index", operation.index, true);
    push_span_field(out, indent + 2, "source_span", &operation.span, true);
    push_string_field(
        out,
        indent + 2,
        "surface_text",
        &operation.surface_text,
        true,
    );
    push_string_field(out, indent + 2, "source_kind", operation.source_kind, true);
    push_string_field(
        out,
        indent + 2,
        "source_status",
        operation.source_status,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "core_operation",
        operation.core_operation,
        true,
    );
    push_string_field(out, indent + 2, "status", operation.status, true);
    push_expression(out, operation.expression.as_ref(), indent + 2, true);
    push_optional_string_field(out, indent + 2, "reason", operation.reason, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_expression(
    out: &mut String,
    expression: Option<&CoreLowerExpression>,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"expression\": ");
    if let Some(expression) = expression {
        out.push_str("{\n");
        push_string_field(out, indent + 2, "text", &expression.text, true);
        push_string_field(out, indent + 2, "kind", expression.kind, true);
        push_string_field(out, indent + 2, "status", expression.status, true);
        push_string_field(out, indent + 2, "ast_status", expression.ast_status, true);
        push_string_field(out, indent + 2, "root_form", expression.root_form, true);
        push_optional_string_field(out, indent + 2, "operator", expression.operator, true);
        push_usize_field(out, indent + 2, "node_count", expression.node_count, true);
        push_structured_expression(out, expression.structured.as_ref(), indent + 2, true);
        push_string_field(out, indent + 2, "type_status", expression.type_status, true);
        push_optional_string_field(
            out,
            indent + 2,
            "type_text",
            expression.type_text.as_deref(),
            true,
        );
        push_optional_string_field(out, indent + 2, "type_source", expression.type_source, true);
        push_string_field(
            out,
            indent + 2,
            "effect_status",
            expression.effect_status,
            true,
        );
        push_optional_string_field(out, indent + 2, "reason", expression.reason, false);
        push_indent(out, indent);
        out.push('}');
    } else {
        out.push_str("null");
    }
    push_comma_newline(out, comma);
}

fn push_structured_expression(
    out: &mut String,
    expression: Option<&CoreLowerStructuredExpression>,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"structured_expression\": ");
    if let Some(expression) = expression {
        out.push_str("{\n");
        push_string_field(out, indent + 2, "provenance", expression.provenance, true);
        push_string_field(
            out,
            indent + 2,
            "parser_node_id",
            &expression.parser_node_id,
            true,
        );
        push_source_range_field(
            out,
            indent + 2,
            "source_range",
            &expression.source_range,
            true,
        );
        push_string_field(out, indent + 2, "kind", expression.kind, true);
        push_string_field(out, indent + 2, "operator", expression.operator, true);
        push_indent(out, indent + 2);
        out.push_str("\"children\": [\n");
        for (index, child) in expression.children.iter().enumerate() {
            push_indent(out, indent + 4);
            out.push_str("{\n");
            push_usize_field(out, indent + 6, "index", child.index, true);
            push_string_field(out, indent + 6, "role", child.role, true);
            push_string_field(
                out,
                indent + 6,
                "parser_node_id",
                &child.parser_node_id,
                true,
            );
            push_source_range_field(out, indent + 6, "source_range", &child.source_range, true);
            push_string_field(out, indent + 6, "kind", child.kind, true);
            push_string_field(out, indent + 6, "identifier", &child.identifier, false);
            push_indent(out, indent + 4);
            out.push('}');
            push_comma_newline(out, index + 1 < expression.children.len());
        }
        push_indent(out, indent + 2);
        out.push_str("]\n");
        push_indent(out, indent);
        out.push('}');
    } else {
        out.push_str("null");
    }
    push_comma_newline(out, comma);
}

fn push_source_range_field(
    out: &mut String,
    indent: usize,
    key: &str,
    range: &CoreLowerSourceRange,
    comma: bool,
) {
    push_indent(out, indent);
    out.push('"');
    out.push_str(key);
    out.push_str("\": {\"file\": ");
    push_json_string(out, &range.start.file);
    out.push_str(&format!(
        ", \"line\": {}, \"column\": {}, \"byte_length\": {}",
        range.start.line, range.start.column, range.byte_len
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_blockers(out: &mut String, blockers: &[CoreLowerBlocker], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"blockers\": [\n");
    for (index, blocker) in blockers.iter().enumerate() {
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_span_field(out, indent + 4, "source_span", &blocker.span, true);
        push_string_field(out, indent + 4, "status", blocker.status, true);
        push_string_field(out, indent + 4, "reason", blocker.reason, false);
        push_indent(out, indent + 2);
        out.push('}');
        push_comma_newline(out, index + 1 < blockers.len());
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_span_field(out: &mut String, indent: usize, key: &str, span: &Span, comma: bool) {
    push_indent(out, indent);
    out.push('"');
    out.push_str(key);
    out.push_str("\": {\"file\": ");
    push_json_string(out, &span.file);
    out.push_str(&format!(
        ", \"line\": {}, \"column\": {}",
        span.line, span.column
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_string_field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    push_indent(out, indent);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    push_json_string(out, value);
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
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    if let Some(value) = value {
        push_json_string(out, value);
    } else {
        out.push_str("null");
    }
    push_comma_newline(out, comma);
}

fn push_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    out.push_str(&value.to_string());
    push_comma_newline(out, comma);
}

fn push_string_array(out: &mut String, indent: usize, key: &str, values: &[&str], comma: bool) {
    push_indent(out, indent);
    out.push('"');
    out.push_str(key);
    out.push_str("\": [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(out, value);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_string_array_refs(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[String],
    comma: bool,
) {
    push_indent(out, indent);
    out.push('"');
    out.push_str(key);
    out.push_str("\": [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(out, value);
    }
    out.push(']');
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
            _ => out.push(ch),
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
    use super::{
        CoreLowerCanonicalMinimalAddType, CoreLowerOperationContext, CoreLowerTaskSignatureVerdict,
        build_core_lower_report, canonical_minimal_add_operation_identity, core_lower_json,
        core_lower_text, corrupt_first_canonical_minimal_add_public_and_claim_for_test,
        lower_operation, substitute_first_canonical_minimal_add_identity_for_test,
        substitute_first_canonical_minimal_add_state_for_test,
    };
    use crate::ast::{CanonicalTaskSignatureCorruption, Item, ParamPermission, Program};
    use crate::parser::parse_source;

    #[test]
    fn canonical_minimal_add_type_authority_is_owned_by_exact_operation() {
        let parsed = parse_source(
            "owned/direct-minimal-add.hum",
            "task add(left: Int, right: Int) -> Int {\n  does:\n    return left + right\n}\n",
        );
        let diagnostics = parsed.diagnostics;
        let program = Program {
            files: vec![parsed.file],
        };
        let item = &program.files[0].items[0];
        let Item::Task(task) = item else {
            panic!("task")
        };
        let signature = match program.authenticate_canonical_task_signature(task) {
            Ok(authority) => authority,
            Err(_) => panic!("signature"),
        };
        let body = crate::core_body::analyze_does_section_for_lowering(
            program
                .canonical_core_expectation(item, task.section("does").expect("does"))
                .expect("expectation"),
        );
        let expected = canonical_minimal_add_operation_identity(&signature, 0, &body.statements[0])
            .expect("expected direct identity");
        let report = build_core_lower_report(&program, &diagnostics);
        let operation = &report.core_items[0].operations[0];
        assert!(
            operation
                .canonical_minimal_add_identity()
                .is_some_and(|identity| identity.matches(&expected))
        );
        let CoreLowerCanonicalMinimalAddType::Supported { authority, claim } =
            operation.canonical_minimal_add_type()
        else {
            panic!("supported operation-owned authority")
        };
        assert!(authority.matches_operation(&signature, item, 0, &body.statements[0]));
        assert!(claim.matches_authority(authority));
        let expression = operation.expression.as_ref().expect("expression");
        assert_eq!(
            expression.type_status,
            crate::core_expr::CORE_EXPRESSION_CHECKED_CANONICAL_MINIMAL_ADD_TYPE_STATUS
        );
        assert_eq!(expression.type_text.as_deref(), Some("Int"));
        assert_eq!(
            expression.type_source,
            Some(crate::core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_SOURCE)
        );

        let foreign = parse_source(
            "foreign/direct-minimal-add.hum",
            "task add(left: Int, right: Int) -> Int {\n  does:\n    return left + right\n}\n",
        );
        let foreign_program = Program {
            files: vec![foreign.file],
        };
        let mut substituted_state = build_core_lower_report(&program, &diagnostics);
        let mut foreign_state = build_core_lower_report(&foreign_program, &foreign.diagnostics);
        substitute_first_canonical_minimal_add_state_for_test(
            &mut substituted_state,
            &mut foreign_state,
        )
        .expect("foreign state substitution");
        let substituted_operation = &substituted_state.core_items[0].operations[0];
        let CoreLowerCanonicalMinimalAddType::Supported { authority, claim } =
            substituted_operation.canonical_minimal_add_type()
        else {
            panic!("substituted supported state")
        };
        assert!(authority.semantic_facts_are_complete());
        assert!(claim.matches_authority(authority));
        assert!(
            authority.matches_public_projection(
                substituted_operation
                    .expression
                    .as_ref()
                    .expect("expression")
                    .type_status,
                substituted_operation
                    .expression
                    .as_ref()
                    .and_then(|expression| expression.type_text.as_deref()),
                substituted_operation
                    .expression
                    .as_ref()
                    .and_then(|expression| expression.type_source),
            )
        );
        assert!(
            !authority.matches_operation(&signature, item, 0, &body.statements[0]),
            "internally complete foreign authority cannot own the original operation"
        );

        let mut substituted_identity = build_core_lower_report(&program, &diagnostics);
        let mut foreign_identity = build_core_lower_report(&foreign_program, &foreign.diagnostics);
        substitute_first_canonical_minimal_add_identity_for_test(
            &mut substituted_identity,
            &mut foreign_identity,
        )
        .expect("foreign identity substitution");
        assert!(
            !substituted_identity.core_items[0].operations[0]
                .canonical_minimal_add_identity()
                .is_some_and(|identity| identity.matches(&expected))
        );

        let mut coherent_candidate = build_core_lower_report(&program, &diagnostics);
        corrupt_first_canonical_minimal_add_public_and_claim_for_test(&mut coherent_candidate)
            .expect("coherent candidate corruption");
        let operation = &coherent_candidate.core_items[0].operations[0];
        let CoreLowerCanonicalMinimalAddType::Supported { authority, claim } =
            operation.canonical_minimal_add_type()
        else {
            panic!("supported coherent candidate")
        };
        assert!(!claim.matches_authority(authority));
        assert!(
            !authority.matches_public_projection(
                operation
                    .expression
                    .as_ref()
                    .expect("expression")
                    .type_status,
                operation
                    .expression
                    .as_ref()
                    .and_then(|expression| expression.type_text.as_deref()),
                operation
                    .expression
                    .as_ref()
                    .and_then(|expression| expression.type_source),
            )
        );

        let mut sabotaged_body = body.clone();
        sabotaged_body.statements[0].statement_mut_for_test().text =
            "return fabricated + spellings".to_string();
        let checked_returns = crate::type_check::checked_return_summaries(&program, &diagnostics);
        let failure_facts = std::collections::BTreeMap::new();
        let context = CoreLowerOperationContext {
            program: &program,
            diagnostics: &diagnostics,
            item,
            task_signature: Some(&signature),
            checked_returns: &checked_returns,
            failure_facts: &failure_facts,
        };
        let sabotaged = lower_operation(&context, 0, &sabotaged_body.statements[0]);
        let CoreLowerCanonicalMinimalAddType::Supported { authority, claim } =
            sabotaged.canonical_minimal_add_type()
        else {
            panic!("statement text cannot change parser-owned classification")
        };
        assert!(authority.matches_operation(&signature, item, 0, &sabotaged_body.statements[0]));
        assert!(claim.matches_authority(authority));
        let structured = sabotaged
            .expression
            .as_ref()
            .and_then(|expression| expression.structured.as_ref())
            .expect("parser-owned structured expression");
        assert_eq!(structured.children[0].identifier, "left");
        assert_eq!(structured.children[1].identifier, "right");
    }

    #[test]
    fn task_signature_authority_is_owned_one_to_one() {
        const SOURCE: &str = "task add(left: Int, right: Int) -> Int {\n  does:\n    return left + right\n}\n\ntest add is stable unit {\n  does:\n    expect add(1, 2) returns 3\n}\n";

        fn program(source: &str) -> (Program, Vec<crate::diagnostic::Diagnostic>) {
            let parsed = parse_source("owned/signature.hum", source);
            (
                Program {
                    files: vec![parsed.file],
                },
                parsed.diagnostics,
            )
        }

        let (program, diagnostics) = program(SOURCE);
        let report = build_core_lower_report(&program, &diagnostics);
        assert_eq!(report.core_items.len(), 2);
        assert_eq!(
            report.core_items[0].task_signature_verdict(),
            CoreLowerTaskSignatureVerdict::Passed
        );
        assert_eq!(
            report.core_items[1].task_signature_verdict(),
            CoreLowerTaskSignatureVerdict::NotATask
        );

        let uint = parse_source(
            "arbitrary\\uint.hum",
            "task add(left: UInt, right: UInt) -> UInt {\n  does:\n    return left\n}\n",
        );
        let uint_program = Program {
            files: vec![uint.file],
        };
        let uint_report = build_core_lower_report(&uint_program, &uint.diagnostics);
        assert_eq!(
            uint_report.core_items[0].task_signature_verdict(),
            CoreLowerTaskSignatureVerdict::Passed,
            "signature authority is type-agnostic"
        );

        let mut missing_program = program.clone();
        missing_program.files[0].items[0]
            .corrupt_canonical_task_signature(CanonicalTaskSignatureCorruption::Missing);
        let missing = build_core_lower_report(&missing_program, &diagnostics);
        assert_eq!(
            missing.core_items[0].task_signature_verdict(),
            CoreLowerTaskSignatureVerdict::Failed
        );

        let mut substituted = build_core_lower_report(&program, &diagnostics);
        let task = &mut substituted.core_items[0];
        task.name = "foreign".to_string();
        task.span.file = "foreign/signature.hum".to_string();
        task.params[0].name = "foreign_left".to_string();
        task.params[0].permission = ParamPermission::Consume;
        task.params[0].ty = "UInt".to_string();
        task.params[1].name = "foreign_right".to_string();
        task.params[1].ty = "UInt".to_string();
        task.result = Some("UInt".to_string());
        assert_eq!(
            task.task_signature_verdict(),
            CoreLowerTaskSignatureVerdict::Failed,
            "coherent public substitution cannot alter untouched parser authority"
        );

        let Item::Task(live_task) = &program.files[0].items[0] else {
            panic!("task")
        };
        assert_eq!(live_task.name, "add");
        let source = include_str!("core_lower.rs");
        let owned_field = ["task_signature:", " CoreLowerTaskSignature"].concat();
        assert_eq!(source.matches(&owned_field).count(), 1);
        let batch = ["Vec", "<CanonicalTaskSignature"].concat();
        assert!(!source.contains(&batch));
    }

    #[test]
    fn json_emits_ordered_parser_owned_minimal_add_tree() {
        let source = include_str!("../examples/core/minimal_add.hum");
        let parsed = parse_source("examples/core/minimal_add.hum", source);
        let diagnostics = parsed.diagnostics;
        let program = Program {
            files: vec![parsed.file],
        };
        let json = core_lower_json(&program, &diagnostics);

        assert!(json.contains("\"structured_expression\": {"));
        assert!(json.contains("\"provenance\": \"parser_owned_canonical_expression_v0\""));
        assert!(json.contains("\"kind\": \"binary\""));
        assert!(json.contains("\"operator\": \"add\""));
        assert!(json.contains("\"type_status\": \"checked_canonical_minimal_add_v0\""));
        assert!(json.contains("\"type_text\": \"Int\""));
        assert!(json.contains("\"type_source\": \"canonical_minimal_add_type_authority_v0\""));
        assert!(!json.contains("\"checked_type_status\""));
        assert!(!json.contains("\"checked_type\""));
        assert!(json.contains("\"index\": 0"));
        assert!(json.contains("\"role\": \"left\""));
        assert!(json.contains("\"identifier\": \"a\""));
        assert!(json.contains("\"index\": 1"));
        assert!(json.contains("\"role\": \"right\""));
        assert!(json.contains("\"identifier\": \"b\""));
        assert!(json.contains("\"byte_length\": 5"));

        let item = &program.files[0].items[0];
        let crate::ast::Item::Task(task) = item else {
            panic!("task")
        };
        let does = task.section("does").expect("does");
        let mut body = crate::core_body::analyze_does_section_for_lowering(
            program
                .canonical_core_expectation(item, does)
                .expect("canonical expectation"),
        );
        let original = body.statements[0]
            .canonical_expression()
            .expect("canonical expression");
        let crate::ast::CanonicalExpressionKind::Binary { left, right, .. } = &original.kind else {
            panic!("binary")
        };
        let expected_left = left.node_id.as_str().to_string();
        let expected_right = right.node_id.as_str().to_string();

        body.statements[0].statement_mut_for_test().text = "return fabricated + names".to_string();
        let checked_returns = crate::type_check::checked_return_summaries(&program, &diagnostics);
        let task_signature = match program.authenticate_canonical_task_signature(task) {
            Ok(authority) => authority,
            Err(_) => panic!("authenticated task signature"),
        };
        let failure_facts = std::collections::BTreeMap::new();
        let context = CoreLowerOperationContext {
            program: &program,
            diagnostics: &diagnostics,
            item,
            task_signature: Some(&task_signature),
            checked_returns: &checked_returns,
            failure_facts: &failure_facts,
        };
        let lowered = lower_operation(&context, 0, &body.statements[0]);
        let structured = lowered
            .expression
            .expect("expression")
            .structured
            .expect("structured expression survives text sabotage");
        assert_eq!(structured.children[0].parser_node_id, expected_left);
        assert_eq!(structured.children[1].parser_node_id, expected_right);
        assert_eq!(structured.children[0].identifier, "a");
        assert_eq!(structured.children[1].identifier, "b");
    }

    #[test]
    fn json_lowers_tiny_task_without_execution_claims() {
        let source = r#"task add(a: Int, b: Int) -> Int {
  does:
    return a + b
}
"#;
        let parsed = parse_source("add.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = core_lower_json(&program, &parsed.diagnostics);

        assert!(json.contains("\"schema\": \"hum.core_lower.v0\""));
        assert!(json.contains("\"core_preview_schema\": \"hum.core_preview.v0\""));
        assert!(json.contains("\"status\": \"lowered_unverified_core_v0\""));
        assert!(json.contains("\"verification_status\": \"unverified_v0\""));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"ir_ready\": 0"));
        assert!(json.contains("\"core_operation\": \"return\""));
        assert!(json.contains("\"root_form\": \"binary_operation_candidate\""));
        assert!(json.contains("\"operator\": \"add\""));
        assert!(json.contains("\"no executable semantics\""));
        assert!(json.contains("\"no Hum IR emission\""));
    }

    #[test]
    fn text_and_json_block_store_write_before_core_execution() {
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
        let text = core_lower_text(&program, &parsed.diagnostics);
        let json = core_lower_json(&program, &parsed.diagnostics);

        assert!(text.contains("[blocked_before_core_execution] task `remember`"));
        assert!(text.contains("surface_save_requires_store_lowering"));
        assert!(json.contains("\"status\": \"blocked_before_core_execution\""));
        assert!(json.contains("\"reason\": \"surface_save_requires_store_lowering\""));
        assert!(json.contains("\"blocked_operations\": 1"));
        assert!(json.contains("\"core_operation\": \"blocked_surface_statement\""));
    }
}
