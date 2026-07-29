use crate::ast::{
    CanonicalCompletionEvent, CanonicalExpressionKind, Item, Param, ParsedBinaryOperator,
    ParsedBlockRelationship, Program, Section, Task,
};
use crate::callable;
use crate::core_body::{self, BodyGrammarReport, BodyStatement};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalCheckedAddCoreView {
    pub(crate) source_revision: std::sync::Arc<[u8]>,
    pub(crate) normalized_path: String,
    pub(crate) semantic_file_index: usize,
    pub(crate) module_token_identity: String,
    pub(crate) module_identity: String,
    pub(crate) module_display_name: String,
    pub(crate) module_range: crate::ast::ParsedSourceRange,
    pub(crate) item_path: Vec<usize>,
    pub(crate) item_kind: &'static str,
    pub(crate) function_identity: String,
    pub(crate) function_display_name: String,
    pub(crate) function_range: crate::ast::ParsedSourceRange,
    pub(crate) linkage_identity: String,
    pub(crate) parameter_ordinals: [usize; 2],
    pub(crate) parameter_identities: [String; 2],
    pub(crate) parameter_names: [String; 2],
    pub(crate) parameter_ranges: [crate::ast::ParsedSourceRange; 2],
    pub(crate) parameter_type_token_identities: [String; 2],
    pub(crate) parameter_type_names: [String; 2],
    pub(crate) parameter_type_ranges: [crate::ast::ParsedSourceRange; 2],
    pub(crate) parameter_permissions: [&'static str; 2],
    pub(crate) result_type_token_identity: String,
    pub(crate) result_type_name: String,
    pub(crate) result_type_range: crate::ast::ParsedSourceRange,
    pub(crate) result_type_explicit: bool,
    pub(crate) does_section_slot: usize,
    pub(crate) does_section_identity: String,
    pub(crate) does_section_name: String,
    pub(crate) does_section_range: crate::ast::ParsedSourceRange,
    pub(crate) statement_count: usize,
    pub(crate) statement_node_identity: String,
    pub(crate) statement_kind: &'static str,
    pub(crate) block_relationship: &'static str,
    pub(crate) block_depth_before: usize,
    pub(crate) block_depth_after: usize,
    pub(crate) block_identity: String,
    pub(crate) operation_kinds: [&'static str; 2],
    pub(crate) return_operation_identity: String,
    pub(crate) add_node_identity: String,
    pub(crate) add_kind: &'static str,
    pub(crate) add_operator: &'static str,
    pub(crate) add_completion: &'static str,
    pub(crate) left_node_identity: String,
    pub(crate) left_kind: &'static str,
    pub(crate) left_completion: &'static str,
    pub(crate) right_node_identity: String,
    pub(crate) right_kind: &'static str,
    pub(crate) right_completion: &'static str,
    pub(crate) ordered_child_relationship: String,
    pub(crate) left_value_identity: String,
    pub(crate) right_value_identity: String,
    pub(crate) result_value_identity: String,
    pub(crate) overflow_edge_identity: String,
    pub(crate) overflow_status: &'static str,
}

pub(crate) fn canonical_checked_add_core_view(
    body: &core_body::CanonicalBackendBody<'_>,
) -> Result<CanonicalCheckedAddCoreView, &'static str> {
    let expression = core_expr::canonical_checked_add_expression(body.statement())?;
    let signature = body.function().signature();
    let [left_parameter, right_parameter] = signature.parameters.as_ref() else {
        return Err("canonical_backend_parameter_count_unsupported_v0");
    };
    if expression.left_name != left_parameter.binder.spelling.as_ref()
        || expression.right_name != right_parameter.binder.spelling.as_ref()
    {
        return Err("canonical_backend_operand_parameter_order_mismatch_v0");
    }
    let add_node_identity = expression.add.node_id.as_str().to_string();
    let mut view = CanonicalCheckedAddCoreView {
        source_revision: signature.file.source_revision.clone(),
        normalized_path: signature.file.normalized_path.to_string(),
        semantic_file_index: signature.file.semantic_file_index,
        module_token_identity: signature.module.identity.to_string(),
        module_identity: signature.module.identity.to_string(),
        module_display_name: signature.module.spelling.to_string(),
        module_range: signature.module.range.clone(),
        item_path: signature.item_path.to_vec(),
        item_kind: signature.item_kind,
        function_identity: signature.function.identity.to_string(),
        function_display_name: signature.function.spelling.to_string(),
        function_range: signature.function.range.clone(),
        linkage_identity: signature.export_linkage_identity.to_string(),
        parameter_ordinals: [left_parameter.ordinal, right_parameter.ordinal],
        parameter_identities: [
            left_parameter.binder.identity.to_string(),
            right_parameter.binder.identity.to_string(),
        ],
        parameter_names: [
            left_parameter.binder.spelling.to_string(),
            right_parameter.binder.spelling.to_string(),
        ],
        parameter_ranges: [
            left_parameter.binder.range.clone(),
            right_parameter.binder.range.clone(),
        ],
        parameter_type_token_identities: [
            left_parameter.declared_type.identity.to_string(),
            right_parameter.declared_type.identity.to_string(),
        ],
        parameter_type_names: [
            left_parameter.declared_type.spelling.to_string(),
            right_parameter.declared_type.spelling.to_string(),
        ],
        parameter_type_ranges: [
            left_parameter.declared_type.range.clone(),
            right_parameter.declared_type.range.clone(),
        ],
        parameter_permissions: [
            left_parameter.permission.as_str(),
            right_parameter.permission.as_str(),
        ],
        result_type_token_identity: signature.result_type.identity.to_string(),
        result_type_name: signature.result_type.spelling.to_string(),
        result_type_range: signature.result_type.range.clone(),
        result_type_explicit: signature.result_type_explicit,
        does_section_slot: signature.does_section_slot,
        does_section_identity: signature.does_section.identity.to_string(),
        does_section_name: signature.does_section.spelling.to_string(),
        does_section_range: signature.does_section.range.clone(),
        statement_count: 1,
        statement_node_identity: expression.statement.source_node_id.as_str().to_string(),
        statement_kind: "return",
        block_relationship: "none",
        block_depth_before: expression.statement.block_depth_before,
        block_depth_after: expression.statement.block_depth_after,
        block_identity: canonical_backend_block_identity(signature),
        operation_kinds: ["return", "checked_add"],
        return_operation_identity: expression.statement.source_node_id.as_str().to_string(),
        add_node_identity: add_node_identity.clone(),
        add_kind: "binary",
        add_operator: "add",
        add_completion: "complete",
        left_node_identity: expression.left.node_id.as_str().to_string(),
        left_kind: "identifier",
        left_completion: "complete",
        right_node_identity: expression.right.node_id.as_str().to_string(),
        right_kind: "identifier",
        right_completion: "complete",
        ordered_child_relationship: format!(
            "ordered-children:left={}:right={}",
            expression.left.node_id.as_str(),
            expression.right.node_id.as_str()
        ),
        left_value_identity: format!("canonical-value:{}", expression.left.node_id.as_str()),
        right_value_identity: format!("canonical-value:{}", expression.right.node_id.as_str()),
        result_value_identity: format!("canonical-value:{add_node_identity}"),
        overflow_edge_identity: format!("canonical-overflow-edge:{add_node_identity}"),
        overflow_status: "checked_add_runtime_trap_exit_2_v0",
    };
    crate::ir_readiness::apply_c1_core_producer_corruption(&mut view);
    validate_canonical_checked_add_core_view(&view, signature, &expression)?;
    Ok(view)
}

fn validate_canonical_checked_add_core_view(
    view: &CanonicalCheckedAddCoreView,
    signature: &crate::parser::CanonicalBackendSignatureBinding,
    expression: &crate::core_expr::CanonicalCheckedAddExpression<'_>,
) -> Result<(), &'static str> {
    let [left_parameter, right_parameter] = signature.parameters.as_ref() else {
        return Err("canonical_backend_parameter_count_unsupported_v0");
    };
    let add = expression.add.node_id.as_str();
    if view.source_revision.as_ref() != signature.file.source_revision.as_ref()
        || view.normalized_path != signature.file.normalized_path.as_ref()
        || view.semantic_file_index != signature.file.semantic_file_index
        || view.module_token_identity != signature.module.identity.as_ref()
        || view.module_identity != signature.module.identity.as_ref()
        || view.module_display_name != signature.module.spelling.as_ref()
        || view.module_range != signature.module.range
        || view.item_path != signature.item_path.as_ref()
        || view.item_kind != signature.item_kind
        || view.function_identity != signature.function.identity.as_ref()
        || view.function_display_name != signature.function.spelling.as_ref()
        || view.function_range != signature.function.range
        || view.linkage_identity != signature.export_linkage_identity.as_ref()
        || view.parameter_ordinals != [left_parameter.ordinal, right_parameter.ordinal]
        || view.parameter_identities
            != [
                left_parameter.binder.identity.to_string(),
                right_parameter.binder.identity.to_string(),
            ]
        || view.parameter_names
            != [
                left_parameter.binder.spelling.to_string(),
                right_parameter.binder.spelling.to_string(),
            ]
        || view.parameter_ranges
            != [
                left_parameter.binder.range.clone(),
                right_parameter.binder.range.clone(),
            ]
        || view.parameter_type_token_identities
            != [
                left_parameter.declared_type.identity.to_string(),
                right_parameter.declared_type.identity.to_string(),
            ]
        || view.parameter_type_names
            != [
                left_parameter.declared_type.spelling.to_string(),
                right_parameter.declared_type.spelling.to_string(),
            ]
        || view.parameter_type_ranges
            != [
                left_parameter.declared_type.range.clone(),
                right_parameter.declared_type.range.clone(),
            ]
        || view.parameter_permissions
            != [
                left_parameter.permission.as_str(),
                right_parameter.permission.as_str(),
            ]
        || view.result_type_token_identity != signature.result_type.identity.as_ref()
        || view.result_type_name != signature.result_type.spelling.as_ref()
        || view.result_type_range != signature.result_type.range
        || view.result_type_explicit != signature.result_type_explicit
        || view.does_section_slot != signature.does_section_slot
        || view.does_section_identity != signature.does_section.identity.as_ref()
        || view.does_section_name != signature.does_section.spelling.as_ref()
        || view.does_section_range != signature.does_section.range
        || view.statement_count != 1
        || view.statement_node_identity != expression.statement.source_node_id.as_str()
        || view.statement_kind != "return"
        || view.block_relationship != "none"
        || expression.statement.block_relationship != ParsedBlockRelationship::None
        || view.block_depth_before != expression.statement.block_depth_before
        || view.block_depth_after != expression.statement.block_depth_after
        || view.block_depth_before != 0
        || view.block_depth_after != 0
        || view.block_identity != canonical_backend_block_identity(signature)
        || view.operation_kinds != ["return", "checked_add"]
        || view.return_operation_identity != expression.statement.source_node_id.as_str()
        || view.add_node_identity != add
        || view.add_kind != "binary"
        || !matches!(
            expression.add.kind,
            CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Add,
                ..
            }
        )
        || view.add_operator != "add"
        || view.add_completion != "complete"
        || !matches!(
            expression.add.completion,
            CanonicalCompletionEvent::Complete
        )
        || view.left_node_identity != expression.left.node_id.as_str()
        || view.left_kind != "identifier"
        || !matches!(expression.left.kind, CanonicalExpressionKind::Identifier(_))
        || view.left_completion != "complete"
        || !matches!(
            expression.left.completion,
            CanonicalCompletionEvent::Complete
        )
        || view.right_node_identity != expression.right.node_id.as_str()
        || view.right_kind != "identifier"
        || !matches!(
            expression.right.kind,
            CanonicalExpressionKind::Identifier(_)
        )
        || view.right_completion != "complete"
        || !matches!(
            expression.right.completion,
            CanonicalCompletionEvent::Complete
        )
        || view.ordered_child_relationship
            != format!(
                "ordered-children:left={}:right={}",
                expression.left.node_id.as_str(),
                expression.right.node_id.as_str()
            )
        || view.left_value_identity
            != format!("canonical-value:{}", expression.left.node_id.as_str())
        || view.right_value_identity
            != format!("canonical-value:{}", expression.right.node_id.as_str())
        || view.result_value_identity != format!("canonical-value:{add}")
        || view.overflow_edge_identity != format!("canonical-overflow-edge:{add}")
        || view.overflow_status != "checked_add_runtime_trap_exit_2_v0"
    {
        return Err("canonical_backend_core_producer_corruption_v0");
    }
    Ok(())
}

fn canonical_backend_block_identity(
    signature: &crate::parser::CanonicalBackendSignatureBinding,
) -> String {
    use std::fmt::Write;

    let mut revision = String::with_capacity(signature.file.source_revision.len() * 2);
    for byte in signature.file.source_revision.iter() {
        write!(&mut revision, "{byte:02x}").expect("writing to String cannot fail");
    }
    let item_path = signature
        .item_path
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".");
    format!(
        "canonical-block:revision-{revision}:file-{}:item-{item_path}:section-{}:token-{}:range-{}-{}-{}",
        signature.file.semantic_file_index,
        signature.does_section_slot,
        signature.does_section.identity,
        signature.does_section.range.start.line,
        signature.does_section.range.start.column,
        signature.does_section.range.byte_len,
    )
}

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
}

pub(crate) struct CoreLowerExpression {
    pub(crate) text: String,
    pub(crate) kind: &'static str,
    pub(crate) status: &'static str,
    pub(crate) ast_status: &'static str,
    pub(crate) root_form: &'static str,
    pub(crate) operator: Option<&'static str>,
    pub(crate) node_count: usize,
    pub(crate) type_status: &'static str,
    pub(crate) type_text: Option<String>,
    pub(crate) type_source: Option<&'static str>,
    pub(crate) effect_status: &'static str,
    pub(crate) reason: Option<&'static str>,
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
    let body = core_body::analyze_does_section(
        program
            .canonical_core_expectation(item, does)
            .expect("live Core item must have parser authority"),
    );
    let failure_analysis = match item {
        Item::Task(task) => failure_analysis.task(task).cloned().unwrap_or_default(),
        _ => Default::default(),
    };
    let operations = lower_operations(
        item,
        &body,
        checked_returns,
        &failure_analysis.facts,
        predicate_facts,
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
    })
}

fn lower_operations(
    item: &Item,
    body: &BodyGrammarReport,
    checked_returns: &[CheckedReturnSummary],
    failure_facts: &std::collections::BTreeMap<usize, FailureFact>,
    predicate_facts: &[PredicateFact],
) -> Vec<CoreLowerOperation> {
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
            )
        })
        .collect::<Vec<_>>();
    if let Item::Task(task) = item {
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
    }
}

fn lower_operation(
    item: &Item,
    index: usize,
    statement: &BodyStatement,
    checked_returns: &[CheckedReturnSummary],
    failure_fact: Option<&FailureFact>,
) -> CoreLowerOperation {
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
        };
    }
    let (core_operation, status, fallback_reason) = core_operation_for(statement);
    let mut expression = expression_text_for_statement(statement).map(|text| {
        lower_expression(
            text,
            checked_return_for_statement(item, statement, checked_returns),
        )
    });
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
    expression_from_preview(&preview)
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
        type_status: preview.ast.type_status,
        type_text: preview.ast.type_text.clone(),
        type_source: preview.ast.type_source,
        effect_status: preview.ast.effect_status,
        reason: preview.reason.or(preview.ast.root.reason),
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
    body: &BodyGrammarReport,
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
    body: &BodyGrammarReport,
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
    use super::{core_lower_json, core_lower_text};
    use crate::ast::Program;
    use crate::parser::parse_source;

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
