use crate::ast::{
    AuthenticatedCanonicalTaskSignature, CanonicalCoreOperationOwnerExpectation,
    CanonicalExpression, CanonicalExpressionKind, Item, Param, ParsedBinaryOperator,
    ParsedBodyStatement, ParsedBodyStatementKind, ParsedSourceRange, Program, Section, SectionLine,
    Task,
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
use crate::type_check::{self, CheckedReturnSummary};
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

#[allow(unexpected_cfgs)]
mod canonical_minimal_add_type_outcome_foreign_issue_compile_proof {
    #[cfg(hum_compile_fail_canonical_minimal_add_type_outcome_foreign_issue)]
    use crate::type_check::CanonicalMinimalAddTypeOutcome as O;
    #[cfg(hum_compile_fail_canonical_minimal_add_type_outcome_foreign_issue)]
    fn canonical_minimal_add_type_outcome_foreign_issue_must_not_compile() {
        let canonical_minimal_add_type_outcome_foreign_issue_must_not_compile = O::non_target();
    }
}

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
    candidate_origin: CoreItemCandidateOrigin,
}

enum CoreLowerTaskSignature {
    NotATask,
    Authenticated(Box<AuthenticatedCanonicalTaskSignature>),
    Rejected(crate::ast::CanonicalTaskSignatureRejection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CoreOperationExpectationError {
    OwnerRejected(&'static str),
    Missing(usize),
    Ambiguous(usize),
    Foreign(usize),
    Ordering(usize),
    SlotOverflow,
    SlotUnderflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CoreItemCandidateOriginFacts(
    usize,
    usize,
    crate::ast::CanonicalCoreOwnerBinding,
    usize,
    Span,
    usize,
    Span,
);

impl CoreItemCandidateOriginFacts {
    fn from_expected(owner: &CanonicalCoreOperationOwnerExpectation<'_>) -> Self {
        let (program, file, binding, ordinal) = owner.candidate_facts();
        Self(
            program,
            file,
            binding.clone(),
            ordinal,
            portable_span(owner.item().span()),
            owner.section_slot(),
            portable_span(&owner.section().span),
        )
    }

    fn matches(&self, owner: &CanonicalCoreOperationOwnerExpectation<'_>) -> bool {
        self == &Self::from_expected(owner)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CoreItemCandidateOrigin {
    Authenticated(Box<CoreItemCandidateOriginFacts>),
    Rejected(CoreOperationExpectationError),
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
    candidate_origin: CoreOperationCandidateOrigin,
    type_outcome: Option<type_check::CanonicalMinimalAddTypeOutcome>,
    type_claim: Option<CanonicalMinimalAddTypeClaim>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalMinimalAddTypeClaim([String; 4]);

#[derive(Debug, Clone, PartialEq, Eq)]
enum CoreOperationCandidateSourceFacts {
    Body(String),
    Predicate(String, String, String, RecognitionStatus),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CoreOperationCandidateOriginFacts {
    owner: CoreItemCandidateOriginFacts,
    slot: usize,
    source: CoreOperationCandidateSourceFacts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CoreOperationCandidateOrigin {
    Authenticated(Box<CoreOperationCandidateOriginFacts>),
    Rejected(CoreOperationExpectationError),
}

enum ExpectedCoreOperationSource<'program, 'invocation> {
    Body {
        line: &'program SectionLine,
        parsed: &'program ParsedBodyStatement,
        artifact: Result<&'invocation CanonicalBodyStatement, CoreOperationExpectationError>,
    },
    Predicate {
        section: &'program Section,
        line: &'program SectionLine,
        artifact: Result<&'invocation PredicateFact, CoreOperationExpectationError>,
    },
}

pub(crate) struct ExpectedCoreOperation<'program, 'invocation> {
    owner: &'invocation CanonicalCoreOperationOwnerExpectation<'program>,
    slot: usize,
    source: ExpectedCoreOperationSource<'program, 'invocation>,
}

#[allow(unexpected_cfgs)]
mod expected_core_operation_escape_compile_proof {
    #[cfg(hum_compile_fail_expected_core_operation_escape)]
    use super::{ExpectedCoreOperation, with_expected_core_operations_for_item};
    #[cfg(hum_compile_fail_expected_core_operation_escape)]
    use crate::{
        ast::CanonicalCoreOperationOwnerExpectation, core_body::CanonicalBodyGrammarReport,
        predicate::PredicateFact,
    };

    #[cfg(hum_compile_fail_expected_core_operation_escape)]
    fn expected_core_operation_artifact_escape_must_not_compile<'program>(
        owner: CanonicalCoreOperationOwnerExpectation<'program>,
        body: &CanonicalBodyGrammarReport,
        predicate_facts: &[PredicateFact],
    ) -> ExpectedCoreOperation<'program, 'program> {
        let mut expected_core_operation_artifact_escape_must_not_compile = None;
        let _result =
            with_expected_core_operations_for_item(owner, body, predicate_facts, |expected| {
                expected_core_operation_artifact_escape_must_not_compile = Some(expected)
            });
        expected_core_operation_artifact_escape_must_not_compile.expect("artifact escape")
    }

    #[cfg(hum_compile_fail_expected_core_operation_escape)]
    fn expected_core_operation_program_escape_must_not_compile()
    -> ExpectedCoreOperation<'static, 'static> {
        let parsed = crate::parser::parse_source_at_index(
            "compile-fail/program-escape.hum",
            "task escape() {\n  does:\n    return 1\n}\n",
            0,
        );
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let item = &program.files[0].items[0];
        let does = item
            .sections()
            .iter()
            .find(|section| section.name == "does")
            .expect("does");
        let owner = program
            .canonical_core_operation_owner_expectation(item, does)
            .expect("owner");
        let predicates = crate::predicate::PredicateAnalysis::build(&program);
        let mut escaped = None;
        let _result = super::with_fresh_expected_core_operations_for_item(
            &program,
            item,
            does,
            owner,
            predicates.facts(),
            |expected| escaped = Some(expected),
        );
        escaped.expect("expected_core_operation_program_escape_must_not_compile")
    }

    #[cfg(hum_compile_fail_expected_core_operation_escape)]
    fn expected_core_operation_static_escape_must_not_compile<'program>(
        owner: CanonicalCoreOperationOwnerExpectation<'program>,
        body: &CanonicalBodyGrammarReport,
        predicate_facts: &[PredicateFact],
    ) {
        let _result =
            with_expected_core_operations_for_item(owner, body, predicate_facts, |expected| {
                let _escaped: &'static ExpectedCoreOperation<'static, 'static> = &expected;
            });
    }

    #[cfg(hum_compile_fail_expected_core_operation_escape)]
    fn expected_core_operation_collection_escape_must_not_compile<'program>(
        owner: CanonicalCoreOperationOwnerExpectation<'program>,
        body: &CanonicalBodyGrammarReport,
        predicate_facts: &[PredicateFact],
    ) {
        let mut expected_core_operation_collection_escape_must_not_compile = Vec::new();
        let _result =
            with_expected_core_operations_for_item(owner, body, predicate_facts, |expected| {
                expected_core_operation_collection_escape_must_not_compile.push(expected)
            });
        drop(expected_core_operation_collection_escape_must_not_compile);
    }
}

impl ExpectedCoreOperation<'_, '_> {
    pub(crate) fn slot(&self) -> usize {
        self.slot
    }
}

pub(crate) fn expected_core_operation_source_span<'a>(
    expected: &'a ExpectedCoreOperation<'_, '_>,
) -> &'a Span {
    match &expected.source {
        ExpectedCoreOperationSource::Body { line, .. } => &line.span,
        ExpectedCoreOperationSource::Predicate { line, .. } => &line.span,
    }
}

#[cfg_attr(test, derive(Clone))]
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
    result_value_present: bool,
    pub(crate) result_value: Option<CoreLowerResultValue>,
    pub(crate) effect_status: &'static str,
    pub(crate) reason: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CoreLowerResultValue {
    pub(crate) id: String,
    pub(crate) type_id: String,
    pub(crate) type_status: &'static str,
    pub(crate) type_text: String,
    pub(crate) provenance: &'static str,
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
    ZeroBasedRange,
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
        CoreLowerTreeCorruption::ZeroBasedRange => {
            structured.source_range.start.line = 0;
            structured.children[0].source_range.start.column = 0;
        }
        CoreLowerTreeCorruption::StructuralOverclaim => structured.kind = "call",
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn swap_operation_origins_for_test(
    report: &mut CoreLowerReport,
    item_index: usize,
    left: usize,
    right: usize,
) {
    let operations = &mut report.core_items[item_index].operations;
    let origin = operations[left].candidate_origin.clone();
    operations[left].candidate_origin = operations[right].candidate_origin.clone();
    operations[right].candidate_origin = origin;
}

#[cfg(test)]
pub(crate) fn copy_item_origin_for_test(
    target: &mut CoreLowerReport,
    target_index: usize,
    source: &CoreLowerReport,
    source_index: usize,
) {
    target.core_items[target_index].candidate_origin =
        source.core_items[source_index].candidate_origin.clone();
}

#[cfg(test)]
pub(crate) fn copy_operation_origin_for_test(
    target: &mut CoreLowerReport,
    target_item: usize,
    target_operation: usize,
    source: &CoreLowerReport,
    source_item: usize,
    source_operation: usize,
) {
    target.core_items[target_item].operations[target_operation].candidate_origin =
        source.core_items[source_item].operations[source_operation]
            .candidate_origin
            .clone();
}

#[cfg(test)]
pub(crate) fn reject_operation_origin_for_test(
    report: &mut CoreLowerReport,
    item_index: usize,
    operation_index: usize,
) {
    report.core_items[item_index].operations[operation_index].candidate_origin =
        CoreOperationCandidateOrigin::Rejected(CoreOperationExpectationError::Ordering(
            operation_index,
        ));
}

#[cfg(test)]
pub(crate) fn corrupt_minimal_add_candidate_for_test(
    report: &mut CoreLowerReport,
    corruption: &str,
) {
    let operation = &mut report.core_items[0].operations[0];
    if corruption == "coherent" {
        let claim = operation.type_claim.as_mut().expect("claim");
        claim.0[1] = "hum-type:builtin:UInt".to_string();
        claim.0[2] = "UInt".to_string();
        let expression = operation.expression.as_mut().expect("expression");
        expression.type_text = Some("UInt".to_string());
        let value = expression.result_value.as_mut().expect("result value");
        value.type_id = "hum-type:builtin:UInt".to_string();
        value.type_text = "UInt".to_string();
        return;
    }
    if corruption == "claim" {
        operation.type_claim.as_mut().expect("claim").0[3] = "core-value:foreign".to_string();
        return;
    }
    let expression = operation.expression.as_mut().expect("expression");
    match corruption {
        "type-status" => expression.type_status = "unchecked_type_v0",
        "type-text" => expression.type_text = Some("UInt".to_string()),
        "type-source" => expression.type_source = Some("foreign_v0"),
        "drop-projection" => expression.structured = None,
        "drop-authority" => expression.structured_authority = None,
        "drop-both" => {
            expression.structured = None;
            expression.structured_authority = None;
        }
        corruption => {
            let value = expression.result_value.as_mut().expect("result value");
            match corruption {
                "result-id" => value.id = "core-value:foreign".to_string(),
                "result-type-id" => value.type_id = "hum-type:builtin:UInt".to_string(),
                "result-status" => value.type_status = "unchecked_type_v0",
                "result-text" => value.type_text = "UInt".to_string(),
                "result-provenance" => value.provenance = "foreign_v0",
                _ => unreachable!(),
            }
        }
    }
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
    let minimal_add_type_producer =
        type_check::CanonicalMinimalAddTypeProducer::new(program, diagnostics);
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
            &file.items,
            &checked_returns,
            errors,
            resolve_summary.resolver_errors,
            type_check_summary.type_errors,
            &failure_analysis,
            predicate_facts.facts(),
            &minimal_add_type_producer,
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

pub(crate) fn core_item_occupies_expected_slot(
    expected: &CanonicalCoreOperationOwnerExpectation<'_>,
    candidate: &CoreLowerItem,
) -> bool {
    match &candidate.candidate_origin {
        CoreItemCandidateOrigin::Authenticated(origin) => origin.matches(expected),
        CoreItemCandidateOrigin::Rejected(error) => {
            let _private_reason = error;
            false
        }
    }
}

pub(crate) fn core_operation_occupies_expected_slot(
    expected: &ExpectedCoreOperation<'_, '_>,
    candidate: &CoreLowerOperation,
) -> bool {
    match &candidate.candidate_origin {
        CoreOperationCandidateOrigin::Authenticated(origin) => {
            CoreOperationCandidateOriginFacts::from_expected(expected)
                .is_ok_and(|expected| origin.as_ref() == &expected)
        }
        CoreOperationCandidateOrigin::Rejected(error) => {
            let _private_reason = error;
            false
        }
    }
}

impl CoreOperationCandidateOriginFacts {
    fn from_expected(
        expected: &ExpectedCoreOperation<'_, '_>,
    ) -> Result<Self, CoreOperationExpectationError> {
        let source = match &expected.source {
            ExpectedCoreOperationSource::Body {
                parsed, artifact, ..
            } => {
                artifact.as_ref().map_err(|error| error.clone())?;
                CoreOperationCandidateSourceFacts::Body(parsed.source_node_id.as_str().to_string())
            }
            ExpectedCoreOperationSource::Predicate {
                section,
                line: _,
                artifact,
            } => {
                let artifact = artifact.as_ref().map_err(|error| error.clone())?;
                CoreOperationCandidateSourceFacts::Predicate(
                    artifact.semantic_task_identity().to_string(),
                    artifact.semantic_line_identity().to_string(),
                    section.name.clone(),
                    artifact.status,
                )
            }
        };
        Ok(Self {
            owner: CoreItemCandidateOriginFacts::from_expected(expected.owner),
            slot: expected.slot,
            source,
        })
    }
}

fn require_local_slot(actual: usize, expected: usize) -> Result<(), CoreOperationExpectationError> {
    match actual
        .checked_sub(expected)
        .ok_or(CoreOperationExpectationError::SlotUnderflow)?
    {
        0 => Ok(()),
        _ => Err(CoreOperationExpectationError::Ordering(expected)),
    }
}

fn unique_artifact<'a, T>(
    artifacts: impl Iterator<Item = (usize, &'a T)>,
    source_slot: usize,
    mut matches: impl FnMut(&T) -> bool,
) -> Result<(usize, &'a T), CoreOperationExpectationError> {
    let mut matched = None;
    for (index, artifact) in artifacts.filter(|(_, artifact)| matches(artifact)) {
        if matched.replace((index, artifact)).is_some() {
            return Err(CoreOperationExpectationError::Ambiguous(source_slot));
        }
    }
    matched.ok_or(CoreOperationExpectationError::Missing(source_slot))
}

fn matching_body_artifact<'invocation>(
    line: &SectionLine,
    parsed: &ParsedBodyStatement,
    body: &'invocation CanonicalBodyGrammarReport,
    source_slot: usize,
) -> Result<&'invocation CanonicalBodyStatement, CoreOperationExpectationError> {
    let (index, artifact) = unique_artifact(
        body.statements.iter().enumerate(),
        source_slot,
        |artifact| {
            let statement = artifact.statement();
            let expression = match &parsed.kind {
                ParsedBodyStatementKind::Return(expression) => Some(&expression.canonical),
                ParsedBodyStatementKind::Binding { .. } | ParsedBodyStatementKind::Other { .. } => {
                    None
                }
            };
            statement.span == portable_span(&line.span)
                && statement.text == line.text.trim()
                && (
                    statement.kind,
                    statement.status,
                    statement.expression_kind,
                    statement.reason,
                ) == (
                    parsed.core_kind,
                    parsed.core_status,
                    parsed.core_expression_kind,
                    parsed.core_reason,
                )
                && artifact.canonical_expression() == expression
        },
    )?;
    require_local_slot(index, source_slot)?;
    Ok(artifact)
}

fn predicate_local_ordinal(
    task: &Task,
    artifact: &PredicateFact,
    predicate_facts: &[PredicateFact],
) -> Result<usize, CoreOperationExpectationError> {
    let mut ordinal = 0usize;
    for candidate in predicate_facts {
        if candidate.task_span != task.span
            || candidate.status == RecognitionStatus::NonExecutableProse
        {
            continue;
        }
        if std::ptr::eq(candidate, artifact) {
            return Ok(ordinal);
        }
        ordinal = ordinal
            .checked_add(1)
            .ok_or(CoreOperationExpectationError::SlotOverflow)?;
    }
    Err(CoreOperationExpectationError::Foreign(ordinal))
}

fn matching_predicate_artifact<'invocation>(
    task: &Task,
    section: &Section,
    line: &SectionLine,
    predicate_facts: &'invocation [PredicateFact],
    predicate_slot: usize,
) -> Result<&'invocation PredicateFact, CoreOperationExpectationError> {
    let (_, artifact) = unique_artifact(
        predicate_facts.iter().enumerate(),
        predicate_slot,
        |artifact| {
            (artifact.task_span == task.span)
                && artifact.section == section.name
                && artifact.line_span == line.span
                && artifact.text == line.text.trim()
        },
    )?;
    if artifact.status != RecognitionStatus::NonExecutableProse {
        require_local_slot(
            predicate_local_ordinal(task, artifact, predicate_facts)?,
            predicate_slot,
        )?;
    }
    Ok(artifact)
}

fn with_expected_core_operation_sources<'program>(
    owner: &CanonicalCoreOperationOwnerExpectation<'program>,
    body: &CanonicalBodyGrammarReport,
    predicate_facts: &[PredicateFact],
    mut operation_slot: usize,
    mut visit: impl for<'invocation> FnMut(usize, ExpectedCoreOperationSource<'program, 'invocation>),
) -> Result<(), CoreOperationExpectationError> {
    fn retain(
        current: &mut Option<CoreOperationExpectationError>,
        error: CoreOperationExpectationError,
    ) {
        if current.is_none()
            || matches!(
                error,
                CoreOperationExpectationError::SlotOverflow
                    | CoreOperationExpectationError::SlotUnderflow
            )
        {
            *current = Some(error);
        }
    }
    let mut rejection = None;
    let mut body_slot = 0usize;
    for (line, parsed) in owner
        .section()
        .lines
        .iter()
        .zip(&owner.section().body_syntax)
    {
        let Some(parsed) = parsed.as_ref() else {
            continue;
        };
        let artifact = matching_body_artifact(line, parsed, body, body_slot);
        if let Err(error) = &artifact {
            retain(&mut rejection, error.clone());
        }
        visit(
            operation_slot,
            ExpectedCoreOperationSource::Body {
                line,
                parsed,
                artifact,
            },
        );
        body_slot = body_slot
            .checked_add(1)
            .ok_or(CoreOperationExpectationError::SlotOverflow)?;
        operation_slot = operation_slot
            .checked_add(1)
            .ok_or(CoreOperationExpectationError::SlotOverflow)?;
    }
    if body.statements.get(body_slot).is_some() {
        retain(
            &mut rejection,
            CoreOperationExpectationError::Foreign(body_slot),
        );
    }
    if let Item::Task(task) = owner.item() {
        let mut predicate_slot = 0usize;
        for section_name in ["needs", "ensures"] {
            let Some(section) = task.section(section_name) else {
                continue;
            };
            for line in &section.lines {
                if !crate::graph::is_meaningful_line_text(&line.text) {
                    continue;
                }
                let artifact = matching_predicate_artifact(
                    task,
                    section,
                    line,
                    predicate_facts,
                    predicate_slot,
                );
                if artifact
                    .as_ref()
                    .is_ok_and(|fact| fact.status == RecognitionStatus::NonExecutableProse)
                {
                    continue;
                }
                if let Err(error) = &artifact {
                    retain(&mut rejection, error.clone());
                }
                visit(
                    operation_slot,
                    ExpectedCoreOperationSource::Predicate {
                        section,
                        line,
                        artifact,
                    },
                );
                predicate_slot = predicate_slot
                    .checked_add(1)
                    .ok_or(CoreOperationExpectationError::SlotOverflow)?;
                operation_slot = operation_slot
                    .checked_add(1)
                    .ok_or(CoreOperationExpectationError::SlotOverflow)?;
            }
        }
        if predicate_facts.iter().any(|fact| {
            fact.task_span == task.span
                && fact.status != RecognitionStatus::NonExecutableProse
                && predicate_local_ordinal(task, fact, predicate_facts)
                    .is_ok_and(|slot| slot >= predicate_slot)
        }) {
            retain(
                &mut rejection,
                CoreOperationExpectationError::Foreign(predicate_slot),
            );
        }
    }
    rejection.map_or(Ok(()), Err)
}

pub(crate) fn with_expected_core_operations_for_item<'program>(
    owner: CanonicalCoreOperationOwnerExpectation<'program>,
    body: &CanonicalBodyGrammarReport,
    predicate_facts: &[PredicateFact],
    visit: impl for<'invocation> FnMut(ExpectedCoreOperation<'program, 'invocation>),
) -> Result<(), CoreOperationExpectationError> {
    stream_expected_core_operations_for_item(owner, body, predicate_facts, 0, visit)
}

fn stream_expected_core_operations_for_item<'program>(
    owner: CanonicalCoreOperationOwnerExpectation<'program>,
    body: &CanonicalBodyGrammarReport,
    predicate_facts: &[PredicateFact],
    operation_slot: usize,
    mut visit: impl for<'invocation> FnMut(ExpectedCoreOperation<'program, 'invocation>),
) -> Result<(), CoreOperationExpectationError> {
    let validation = with_expected_core_operation_sources(
        &owner,
        body,
        predicate_facts,
        operation_slot,
        |_, _| {},
    );
    let streamed = with_expected_core_operation_sources(
        &owner,
        body,
        predicate_facts,
        operation_slot,
        |slot, mut source| {
            if let Err(error) = &validation {
                match &mut source {
                    ExpectedCoreOperationSource::Body { artifact, .. } => {
                        *artifact = Err(error.clone())
                    }
                    ExpectedCoreOperationSource::Predicate { artifact, .. } => {
                        *artifact = Err(error.clone())
                    }
                }
            }
            visit(ExpectedCoreOperation {
                owner: &owner,
                slot,
                source,
            });
        },
    );
    debug_assert_eq!(streamed, validation);
    validation
}

#[cfg(test)]
pub(crate) fn with_expected_core_operations_from_slot_for_test<'program>(
    owner: CanonicalCoreOperationOwnerExpectation<'program>,
    body: &CanonicalBodyGrammarReport,
    predicate_facts: &[PredicateFact],
    operation_slot: usize,
    visit: impl for<'invocation> FnMut(ExpectedCoreOperation<'program, 'invocation>),
) -> Result<(), CoreOperationExpectationError> {
    stream_expected_core_operations_for_item(owner, body, predicate_facts, operation_slot, visit)
}

fn fresh_canonical_body_for_item<'program>(
    program: &'program Program,
    item: &'program Item,
    does: &'program Section,
) -> CanonicalBodyGrammarReport {
    core_body::analyze_does_section_for_lowering(
        program
            .canonical_core_expectation(item, does)
            .expect("live Core item must have parser authority"),
    )
}

pub(crate) fn with_fresh_expected_core_operations_for_item<'program>(
    program: &'program Program,
    item: &'program Item,
    does: &'program Section,
    owner: CanonicalCoreOperationOwnerExpectation<'program>,
    predicate_facts: &[PredicateFact],
    visit: impl for<'invocation> FnMut(ExpectedCoreOperation<'program, 'invocation>),
) -> Result<(), CoreOperationExpectationError> {
    let body = fresh_canonical_body_for_item(program, item, does);
    with_expected_core_operations_for_item(owner, &body, predicate_facts, visit)
}

#[allow(clippy::too_many_arguments)]
fn collect_items(
    program: &Program,
    items: &[Item],
    checked_returns: &[CheckedReturnSummary],
    source_errors: usize,
    resolver_errors: usize,
    type_errors: usize,
    failure_analysis: &ProgramFailureAnalysis,
    predicate_facts: &[PredicateFact],
    minimal_add_type_producer: &type_check::CanonicalMinimalAddTypeProducer,
    core_items: &mut Vec<CoreLowerItem>,
) {
    for item in items {
        if let Some(core_item) = core_item(
            program,
            item,
            checked_returns,
            source_errors,
            resolver_errors,
            type_errors,
            failure_analysis,
            predicate_facts,
            minimal_add_type_producer,
        ) {
            core_items.push(core_item);
        }
        if let Item::App(app) = item {
            collect_items(
                program,
                &app.items,
                checked_returns,
                source_errors,
                resolver_errors,
                type_errors,
                failure_analysis,
                predicate_facts,
                minimal_add_type_producer,
                core_items,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn core_item(
    program: &Program,
    item: &Item,
    checked_returns: &[CheckedReturnSummary],
    source_errors: usize,
    resolver_errors: usize,
    type_errors: usize,
    failure_analysis: &ProgramFailureAnalysis,
    predicate_facts: &[PredicateFact],
    minimal_add_type_producer: &type_check::CanonicalMinimalAddTypeProducer,
) -> Option<CoreLowerItem> {
    let does = item_sections(item)
        .iter()
        .find(|section| section.name == "does")?;
    let body = fresh_canonical_body_for_item(program, item, does);
    let failure_analysis = match item {
        Item::Task(task) => failure_analysis.task(task).cloned().unwrap_or_default(),
        _ => Default::default(),
    };
    let operation_owner = program
        .canonical_core_operation_owner_expectation(item, does)
        .map_err(CoreOperationExpectationError::OwnerRejected);
    let candidate_origin = match &operation_owner {
        Ok(owner) => CoreItemCandidateOrigin::Authenticated(Box::new(
            CoreItemCandidateOriginFacts::from_expected(owner),
        )),
        Err(error) => CoreItemCandidateOrigin::Rejected(error.clone()),
    };
    let operations = lower_operations(
        item,
        operation_owner,
        &body,
        checked_returns,
        &failure_analysis.facts,
        predicate_facts,
        minimal_add_type_producer,
        program,
    );
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
    let task_signature = match item {
        Item::Task(task) => match program.authenticate_canonical_task_signature(task) {
            Ok(authority) => CoreLowerTaskSignature::Authenticated(Box::new(authority)),
            Err(rejection) => CoreLowerTaskSignature::Rejected(rejection),
        },
        _ => CoreLowerTaskSignature::NotATask,
    };
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
        candidate_origin,
    })
}

#[allow(clippy::too_many_arguments)]
fn lower_operations<'program>(
    item: &Item,
    owner: Result<CanonicalCoreOperationOwnerExpectation<'program>, CoreOperationExpectationError>,
    body: &CanonicalBodyGrammarReport,
    checked_returns: &[CheckedReturnSummary],
    failure_facts: &std::collections::BTreeMap<usize, FailureFact>,
    predicate_facts: &[PredicateFact],
    minimal_add_type_producer: &type_check::CanonicalMinimalAddTypeProducer,
    program: &Program,
) -> Vec<CoreLowerOperation> {
    let pending = match &owner {
        Ok(_) => CoreOperationExpectationError::Ordering(0),
        Err(error) => error.clone(),
    };
    let mut operations = body
        .statements
        .iter()
        .enumerate()
        .map(|(index, statement)| {
            lower_operation(
                item,
                index,
                statement,
                checked_returns,
                failure_facts.get(&index),
                CoreOperationCandidateOrigin::Rejected(pending.clone()),
            )
        })
        .collect::<Vec<_>>();
    if let Item::Task(task) = item {
        for fact in predicate_facts.iter().filter(|fact| {
            fact.task_span == task.span && fact.status != RecognitionStatus::NonExecutableProse
        }) {
            let index = operations.len();
            operations.push(lower_predicate_operation(
                index,
                fact,
                CoreOperationCandidateOrigin::Rejected(pending.clone()),
            ));
        }
    }
    if let Ok(owner) = owner {
        let result = with_expected_core_operations_for_item(
            owner,
            body,
            predicate_facts,
            |expected| {
                if let Some(candidate) = operations.get_mut(expected.slot) {
                    if let Ok(origin) = CoreOperationCandidateOriginFacts::from_expected(&expected)
                    {
                        candidate.candidate_origin =
                            CoreOperationCandidateOrigin::Authenticated(Box::new(origin));
                    }
                    let (parsed, expression, authenticated) = match &expected.source {
                        ExpectedCoreOperationSource::Body {
                            parsed, artifact, ..
                        } => (
                            Some(*parsed),
                            artifact
                                .as_ref()
                                .ok()
                                .and_then(|artifact| artifact.canonical_expression()),
                            artifact.is_ok(),
                        ),
                        ExpectedCoreOperationSource::Predicate { .. } => (None, None, true),
                    };
                    let outcome = minimal_add_type_producer.classify(
                        program,
                        expected.owner.candidate_facts().2,
                        expected.owner.item(),
                        parsed,
                        expression,
                        authenticated,
                    );
                    candidate.type_claim = None;
                    if let Some(authority) = outcome.supported_authority() {
                        if let Some(expression) = candidate.expression.as_mut() {
                            let (_, root, type_id, type_text, value_id, _, _) =
                                authority.verification_facts();
                            expression.type_status =
                                core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_STATUS;
                            expression.type_text = Some(type_text.to_string());
                            expression.type_source =
                                Some(core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_SOURCE);
                            expression.result_value = Some(CoreLowerResultValue {
                                id: value_id.to_string(),
                                type_id: type_id.to_string(),
                                type_status:
                                    core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_STATUS,
                                type_text: type_text.to_string(),
                                provenance:
                                    core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_SOURCE,
                            });
                            expression.result_value_present = true;
                            candidate.type_claim = Some(CanonicalMinimalAddTypeClaim([
                                root.to_string(),
                                type_id.to_string(),
                                type_text.to_string(),
                                value_id.to_string(),
                            ]));
                        }
                    } else if outcome.integrity_failure_reason().is_some()
                        && let Some(expression) = candidate.expression.as_mut()
                    {
                        expression.type_status = core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_INTEGRITY_FAILURE_STATUS;
                        expression.result_value_present = true;
                    }
                    candidate.type_outcome = Some(outcome);
                }
            },
        );
        if let Err(error) = result {
            for candidate in &mut operations {
                candidate.candidate_origin = CoreOperationCandidateOrigin::Rejected(error.clone());
            }
        }
    }
    operations
}

impl CoreLowerOperation {
    pub(crate) fn minimal_add_type_outcome(
        &self,
    ) -> Option<&type_check::CanonicalMinimalAddTypeOutcome> {
        self.type_outcome.as_ref()
    }
}

pub(crate) fn canonical_minimal_add_verification_facts(
    program: &Program,
    expected: Option<&ExpectedCoreOperation<'_, '_>>,
    operation: &CoreLowerOperation,
) -> (bool, bool, bool) {
    let Some(authority) = operation
        .type_outcome
        .as_ref()
        .and_then(type_check::CanonicalMinimalAddTypeOutcome::supported_authority)
    else {
        return (false, false, false);
    };
    let retained_structure_present = operation.expression.as_ref().is_some_and(|expression| {
        expression.structured.is_some() && expression.structured_authority().is_some()
    });
    let authority_matches = retained_structure_present
        && expected.is_some_and(|expected| match &expected.source {
            ExpectedCoreOperationSource::Body {
                parsed, artifact, ..
            } => artifact
                .as_ref()
                .ok()
                .and_then(|artifact| artifact.canonical_expression())
                .is_some_and(|expression| {
                    let (_, _, owner, _) = expected.owner.candidate_facts();
                    authority.matches_source(
                        program,
                        owner,
                        expected.owner.item(),
                        parsed,
                        expression,
                    )
                }),
            ExpectedCoreOperationSource::Predicate { .. } => false,
        });
    let (_, root_node_id, type_id, type_text, result_value_id, _, _) =
        authority.verification_facts();
    let claim_matches = operation.type_claim.as_ref().is_some_and(|claim| {
        claim
            .0
            .iter()
            .map(String::as_str)
            .eq([root_node_id, type_id, type_text, result_value_id])
    });
    let public_projection_matches = operation.expression.as_ref().is_some_and(|expression| {
        expression.type_status == core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_STATUS
            && expression.type_text.as_deref() == Some(type_text)
            && expression.type_source
                == Some(core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_SOURCE)
            && expression.result_value.as_ref().is_some_and(|value| {
                value.id == result_value_id
                    && value.type_id == type_id
                    && value.type_status
                        == core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_STATUS
                    && value.type_text == type_text
                    && value.provenance
                        == core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_SOURCE
            })
    });
    (authority_matches, claim_matches, public_projection_matches)
}

fn lower_predicate_operation(
    index: usize,
    fact: &PredicateFact,
    candidate_origin: CoreOperationCandidateOrigin,
) -> CoreLowerOperation {
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
        candidate_origin,
        type_outcome: None,
        type_claim: None,
    }
}

fn lower_operation(
    item: &Item,
    index: usize,
    bound_statement: &CanonicalBodyStatement,
    checked_returns: &[CheckedReturnSummary],
    failure_fact: Option<&FailureFact>,
    candidate_origin: CoreOperationCandidateOrigin,
) -> CoreLowerOperation {
    let statement = bound_statement.statement();
    let canonical_expression = bound_statement.canonical_expression();
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
            candidate_origin,
            type_outcome: None,
            type_claim: None,
        };
    }
    let (core_operation, status, fallback_reason) = core_operation_for(statement);
    let checked_return = checked_return_for_statement(item, statement, checked_returns);
    let mut expression = expression_text_for_statement(statement)
        .map(|text| lower_expression(text, checked_return, canonical_expression));
    if statement.status == "unsupported_v0" {
        expression = None;
    }
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
        candidate_origin,
        type_outcome: None,
        type_claim: None,
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
        result_value_present: false,
        result_value: None,
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

pub(crate) fn portable_span(span: &Span) -> Span {
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
        if expression.result_value_present {
            push_result_value(out, expression.result_value.as_ref(), indent + 2, true);
        }
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

fn push_result_value(
    out: &mut String,
    value: Option<&CoreLowerResultValue>,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"result_value\": ");
    if let Some(value) = value {
        out.push_str("{\n");
        push_string_field(out, indent + 2, "id", &value.id, true);
        push_string_field(out, indent + 2, "type_id", &value.type_id, true);
        push_string_field(out, indent + 2, "type_status", value.type_status, true);
        push_string_field(out, indent + 2, "type_text", &value.type_text, true);
        push_string_field(out, indent + 2, "provenance", value.provenance, false);
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
        CoreLowerTaskSignatureVerdict, CoreOperationCandidateOrigin, CoreOperationExpectationError,
        build_core_lower_report, core_lower_json, core_lower_text, lower_operation,
    };
    use crate::ast::{CanonicalTaskSignatureCorruption, Item, ParamPermission, Program};
    use crate::parser::parse_source;

    #[test]
    fn core_operation_candidate_origin_is_attached_once() {
        fn lower(
            path: &str,
            source: &str,
        ) -> (
            Program,
            Vec<crate::diagnostic::Diagnostic>,
            super::CoreLowerReport,
        ) {
            let parsed = parse_source(path, source);
            let program = Program {
                files: vec![parsed.file],
            };
            let report = build_core_lower_report(&program, &parsed.diagnostics);
            (program, parsed.diagnostics, report)
        }
        let (program, diagnostics, report) = lower(
            "fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum",
            include_str!("../fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum"),
        );
        assert!(report.core_items.iter().all(|item| matches!(
            item.candidate_origin,
            super::CoreItemCandidateOrigin::Authenticated(_)
        )));
        let mut ops = report.core_items.iter().flat_map(|item| &item.operations);
        assert!(ops.clone().any(|op| op.status == "blocked_operation_v0"));
        assert!(ops.all(|operation| matches!(
            operation.candidate_origin,
            super::CoreOperationCandidateOrigin::Authenticated(_)
        )));
        let json = core_lower_json(&program, &diagnostics);
        assert!(!json.contains("candidate_origin"));
        let (_, _, add_report) = lower(
            "examples/core/add.hum",
            include_str!("../examples/core/add.hum"),
        );
        assert!(add_report.core_items.iter().any(|item| item.kind != "task"));
        assert!(
            add_report
                .core_items
                .iter()
                .flat_map(|item| &item.operations)
                .any(|op| op.source_kind == "contract_predicate")
        );
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
        assert!(json.contains("\"type_status\": \"checked_canonical_minimal_add_type_v0\""));
        assert!(json.contains("\"type_text\": \"Int\""));
        assert!(json.contains("\"type_source\": \"canonical_minimal_add_type_authority_v0\""));
        assert!(json.contains("\"result_value\": {"));
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
        let rejected = CoreOperationCandidateOrigin::Rejected(
            CoreOperationExpectationError::OwnerRejected("test_only_v0"),
        );
        let lowered = lower_operation(
            item,
            0,
            &body.statements[0],
            &checked_returns,
            None,
            rejected,
        );
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

    #[test]
    fn canonical_minimal_add_type_authority_is_owned_by_exact_operation() {
        let supported = |operation: &super::CoreLowerOperation| {
            operation
                .minimal_add_type_outcome()
                .is_some_and(crate::type_check::CanonicalMinimalAddTypeOutcome::is_supported)
        };
        let parsed = parse_source(
            "owned-add.hum",
            "task add(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\n",
        );
        let program = Program {
            files: vec![parsed.file],
        };
        let report = super::build_core_lower_report(&program, &parsed.diagnostics);
        let operation = &report.core_items[0].operations[0];
        assert!(supported(operation));
        assert!(operation.type_claim.is_some());
        let expression = operation.expression.as_ref().expect("expression");
        assert_eq!(
            expression.type_status,
            crate::core_expr::CORE_EXPRESSION_CANONICAL_MINIMAL_ADD_TYPE_STATUS
        );
        assert_eq!(expression.type_text.as_deref(), Some("Int"));
        assert_eq!(
            expression.result_value.as_ref().unwrap().type_id,
            crate::type_check::CANONICAL_MINIMAL_ADD_TYPE_ID
        );
        let json = core_lower_json(&program, &parsed.diagnostics);
        assert!(json.contains("\"result_value\": {"));
        assert!(!json.contains("type_claim"));
        assert!(!json.contains("CanonicalMinimalAddTypeAuthority"));

        let paired = parse_source(
            "paired-adds.hum",
            "task first(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\ntask second(a: Int, b: Int) -> Int {\n  does:\n    return a + b\n}\n",
        );
        let paired_program = Program {
            files: vec![paired.file],
        };
        let paired_report = super::build_core_lower_report(&paired_program, &paired.diagnostics);
        let first = &paired_report.core_items[0].operations[0];
        let second = &paired_report.core_items[1].operations[0];
        assert!(supported(first) && supported(second));
        assert_ne!(
            first.type_claim.as_ref().unwrap().0[3],
            second.type_claim.as_ref().unwrap().0[3]
        );
    }
}
