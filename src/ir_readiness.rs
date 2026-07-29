use crate::ast::{App, Item, Program, Section, Store, Task, Test, TypeDef};
use crate::core_body::{self, BodyGrammarReport, BodyStatement};
use crate::core_contract;
use crate::core_lower;
use crate::core_preview;
use crate::core_verify;
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::effect_check;
use crate::full_type_check;
use crate::graph::is_meaningful_line_text;
use crate::ir_contract;
use crate::node_id;
use crate::ownership_check;
use crate::profile_check;
use crate::resolve;
use crate::resource_check;
use crate::type_check;
use crate::version;

pub const IR_READINESS_SCHEMA: &str = "hum.ir_readiness.v0";

struct IrReadinessReport {
    files: usize,
    items: usize,
    tasks: usize,
    tests: usize,
    errors: usize,
    warnings: usize,
    resolve_summary: resolve::ResolveReadinessSummary,
    type_check_summary: type_check::TypeCheckSummary,
    core_preview_summary: core_preview::CorePreviewReadinessSummary,
    core_lower_summary: core_lower::CoreLowerReadinessSummary,
    core_verify_summary: core_verify::CoreVerifyReadinessSummary,
    full_type_check_summary: full_type_check::FullTypeCheckSummary,
    effect_check_summary: effect_check::EffectCheckSummary,
    ownership_check_summary: ownership_check::OwnershipCheckSummary,
    resource_check_summary: resource_check::ResourceCheckSummary,
    profile_check_summary: profile_check::ProfileCheckSummary,
    candidates: Vec<LoweringCandidate>,
}

struct LoweringCandidate {
    id: String,
    kind: &'static str,
    name: String,
    graph_node_id: String,
    span: Span,
    status: &'static str,
    current_layer: &'static str,
    target_layer: &'static str,
    facts_available: Vec<&'static str>,
    missing_passes: Vec<&'static str>,
    blocking_reasons: Vec<&'static str>,
    section_names: Vec<String>,
    body_grammar: Option<BodyGrammarReport>,
    checked_add_producer_closure: Option<CanonicalCheckedAddProducerClosure>,
    checked_add_producer_blocker: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalCheckedAddProducerCandidate {
    source_revision: String,
    normalized_path: String,
    semantic_file_index: usize,
    module_token_identity: String,
    module_identity: String,
    module_display_name: String,
    module_range: crate::ast::ParsedSourceRange,
    item_path: Vec<usize>,
    item_kind: &'static str,
    function_identity: String,
    function_display_name: String,
    function_range: crate::ast::ParsedSourceRange,
    linkage_identity: String,
    parameter_ordinals: [usize; 2],
    parameter_identities: [String; 2],
    parameter_names: [String; 2],
    parameter_ranges: [crate::ast::ParsedSourceRange; 2],
    parameter_type_token_identities: [String; 2],
    parameter_type_names: [String; 2],
    parameter_type_ranges: [crate::ast::ParsedSourceRange; 2],
    parameter_permissions: [&'static str; 2],
    result_type_token_identity: String,
    result_type_name: String,
    result_type_range: crate::ast::ParsedSourceRange,
    result_type_explicit: bool,
    does_section_slot: usize,
    does_section_identity: String,
    does_section_name: String,
    does_section_range: crate::ast::ParsedSourceRange,
    statement_count: usize,
    statement_node_identity: String,
    statement_kind: &'static str,
    block_relationship: &'static str,
    block_depth_before: usize,
    block_depth_after: usize,
    block_identity: String,
    operation_identities: [String; 2],
    operation_kinds: [&'static str; 2],
    child_node_identities: [String; 2],
    add_kind: &'static str,
    add_operator: &'static str,
    add_completion: &'static str,
    child_kinds: [&'static str; 2],
    child_completions: [&'static str; 2],
    ordered_child_relationship: String,
    operand_value_identities: [String; 2],
    result_value_identity: String,
    resolver_use_identities: [String; 2],
    resolver_definition_identities: [String; 2],
    resolver_use_order_identity: String,
    resolver_definition_order_identity: String,
    resolver_distinct_binding_status: &'static str,
    parameter_type_identities: [String; 2],
    operand_type_identities: [String; 2],
    add_result_type_identity: String,
    function_result_type_identity: String,
    effect_identity: String,
    authority_identity: String,
    ownership_identity: String,
    resource_identity: String,
    profile_identity: String,
    overflow_edge_identity: String,
    overflow_status: &'static str,
    accepted_passes: Vec<&'static str>,
    unsupported_facts: Vec<&'static str>,
    missing_artifact: &'static str,
    missing_verifier: &'static str,
    status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedCanonicalCheckedAddProducerCandidate(CanonicalCheckedAddProducerCandidate);

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalCheckedAddProducerClosure {
    validated: CanonicalCheckedAddProducerCandidate,
}

impl CanonicalCheckedAddProducerClosure {
    fn from_validated(candidate: ValidatedCanonicalCheckedAddProducerCandidate) -> Self {
        Self {
            validated: candidate.0,
        }
    }
}

impl std::ops::Deref for CanonicalCheckedAddProducerClosure {
    type Target = CanonicalCheckedAddProducerCandidate;

    fn deref(&self) -> &Self::Target {
        &self.validated
    }
}

struct PassStatus {
    name: &'static str,
    status: &'static str,
    source: &'static str,
}

struct CandidateContext<'a> {
    program: &'a Program,
    diagnostics: &'a [Diagnostic],
    resolve_summary: &'a resolve::ResolveReadinessSummary,
    type_check_summary: &'a type_check::TypeCheckSummary,
    core_preview_summary: &'a core_preview::CorePreviewReadinessSummary,
    core_lower_summary: &'a core_lower::CoreLowerReadinessSummary,
    core_verify_summary: &'a core_verify::CoreVerifyReadinessSummary,
    full_type_check_summary: &'a full_type_check::FullTypeCheckSummary,
    effect_check_summary: &'a effect_check::EffectCheckSummary,
    ownership_check_summary: &'a ownership_check::OwnershipCheckSummary,
    resource_check_summary: &'a resource_check::ResourceCheckSummary,
    profile_check_summary: &'a profile_check::ProfileCheckSummary,
    checked_returns: &'a [type_check::CheckedReturnSummary],
}

#[derive(Debug, Clone, Copy)]
struct CandidateBlockers {
    has_errors: bool,
    has_resolver_errors: bool,
    has_type_errors: bool,
    has_core_verify_errors: bool,
    has_full_type_check_errors: bool,
    has_effect_check_errors: bool,
    has_ownership_check_errors: bool,
    has_resource_check_errors: bool,
    has_profile_check_errors: bool,
}

const CURRENT_LAYER: &str = "surface_hum_and_semantic_graph";
const TARGET_LAYER: &str = "core_hum_then_hum_ir";

const PASS_STATUSES: &[PassStatus] = &[
    PassStatus {
        name: "parse",
        status: "current",
        source: "hum parser",
    },
    PassStatus {
        name: "semantic_graph_build",
        status: "current",
        source: "hum graph",
    },
    PassStatus {
        name: "resolve",
        status: "checked_report_available",
        source: resolve::RESOLVE_REPORT_SCHEMA,
    },
    PassStatus {
        name: "body_grammar",
        status: core_body::CORE_BODY_GRAMMAR_STATUS,
        source: core_contract::CORE_CONTRACT_SCHEMA,
    },
    PassStatus {
        name: "core_preview",
        status: core_preview::CORE_PREVIEW_STATUS,
        source: core_preview::CORE_PREVIEW_SCHEMA,
    },
    PassStatus {
        name: "core_lowering",
        status: core_lower::CORE_LOWER_STATUS,
        source: core_lower::CORE_LOWER_SCHEMA,
    },
    PassStatus {
        name: "core_verify",
        status: core_verify::CORE_VERIFY_STATUS,
        source: core_verify::CORE_VERIFY_SCHEMA,
    },
    PassStatus {
        name: "type_check",
        status: "declaration_and_trivial_return_check_available",
        source: type_check::TYPE_CHECK_SCHEMA,
    },
    PassStatus {
        name: "full_type_check",
        status: full_type_check::FULL_TYPE_CHECK_STATUS,
        source: full_type_check::FULL_TYPE_CHECK_SCHEMA,
    },
    PassStatus {
        name: "effect_check",
        status: effect_check::EFFECT_CHECK_STATUS,
        source: effect_check::EFFECT_CHECK_SCHEMA,
    },
    PassStatus {
        name: "ownership_alias_check",
        status: ownership_check::OWNERSHIP_CHECK_STATUS,
        source: ownership_check::OWNERSHIP_CHECK_SCHEMA,
    },
    PassStatus {
        name: "allocation_resource_check",
        status: resource_check::RESOURCE_CHECK_STATUS,
        source: resource_check::RESOURCE_CHECK_SCHEMA,
    },
    PassStatus {
        name: "contract_evidence_linking",
        status: "report_available_not_ir_pass",
        source: "hum evidence",
    },
    PassStatus {
        name: "profile_check",
        status: profile_check::PROFILE_CHECK_STATUS,
        source: profile_check::PROFILE_CHECK_SCHEMA,
    },
    PassStatus {
        name: "ir_verify",
        status: "not_implemented",
        source: ir_contract::IR_CONTRACT_SCHEMA,
    },
];

const MISSING_IR_PASSES: &[&str] = &[
    "full_type_check",
    "effect_check",
    "ownership_alias_check",
    "allocation_resource_check",
    "profile_check",
    "ir_verify",
];

const MISSING_AFTER_FULL_TYPE_PASSES: &[&str] = &[
    "effect_check",
    "ownership_alias_check",
    "allocation_resource_check",
    "profile_check",
    "ir_verify",
];

const MISSING_AFTER_EFFECT_PASSES: &[&str] = &[
    "ownership_alias_check",
    "allocation_resource_check",
    "profile_check",
    "ir_verify",
];

const MISSING_AFTER_OWNERSHIP_PASSES: &[&str] =
    &["allocation_resource_check", "profile_check", "ir_verify"];

const MISSING_AFTER_RESOURCE_PASSES: &[&str] = &["profile_check", "ir_verify"];
const MISSING_AFTER_PROFILE_PASSES: &[&str] = &["hum.backend_input.v0", "ir_verify"];

fn canonical_checked_add_producer_closure(
    program: &Program,
    diagnostics: &[Diagnostic],
    item: &Item,
    task: &Task,
) -> Result<CanonicalCheckedAddProducerClosure, &'static str> {
    let expectation = program.canonical_backend_function_expectation(task)?;
    let body = core_body::try_analyze_backend_function(expectation)?;
    if !std::ptr::eq(body.function().item(), item)
        || !program
            .files
            .iter()
            .any(|file| std::ptr::eq(file, body.function().file()))
    {
        return Err("canonical_backend_live_container_join_mismatch_v0");
    }
    let core = core_lower::canonical_checked_add_core_view(&body)?;
    let resolver = resolve::canonical_backend_resolver_view(program, diagnostics, task, &core)?;
    let types =
        full_type_check::canonical_backend_checked_add_types(program, diagnostics, task, &core)?;
    let effect =
        effect_check::canonical_backend_checked_add_effect(program, diagnostics, task, &core)?;
    let ownership = ownership_check::canonical_backend_checked_add_ownership(
        program,
        diagnostics,
        task,
        &core,
    )?;
    let resource =
        resource_check::canonical_backend_checked_add_resource(program, diagnostics, task, &core)?;
    let profile = profile_check::canonical_backend_checked_add_profile(
        program,
        diagnostics,
        task,
        &resource,
    )?;
    let core_authority = core.clone();
    let expected_resolver_identity = resolve::semantic_item_identity_for(program, item);
    let expected_resolver_definition_identity =
        resolve::semantic_task_definition_identity(program, task);
    let expected_type = full_type_check::CANONICAL_BACKEND_INT_TYPE_ID;
    let (unsupported_facts, missing_artifact, missing_verifier) = c1_readiness_producer_facts();
    let mut candidate = CanonicalCheckedAddProducerCandidate {
        source_revision: hex_bytes(&core.source_revision),
        normalized_path: core.normalized_path.clone(),
        semantic_file_index: core.semantic_file_index,
        module_token_identity: core.module_token_identity.clone(),
        module_identity: core.module_identity,
        module_display_name: core.module_display_name,
        module_range: core.module_range.clone(),
        item_path: core.item_path.clone(),
        item_kind: core.item_kind,
        function_identity: core.function_identity,
        function_display_name: core.function_display_name,
        function_range: core.function_range.clone(),
        linkage_identity: core.linkage_identity,
        parameter_ordinals: core.parameter_ordinals,
        parameter_identities: core.parameter_identities,
        parameter_names: core.parameter_names,
        parameter_ranges: core.parameter_ranges.clone(),
        parameter_type_token_identities: core.parameter_type_token_identities,
        parameter_type_names: core.parameter_type_names,
        parameter_type_ranges: core.parameter_type_ranges.clone(),
        parameter_permissions: core.parameter_permissions,
        result_type_token_identity: core.result_type_token_identity,
        result_type_name: core.result_type_name,
        result_type_range: core.result_type_range.clone(),
        result_type_explicit: core.result_type_explicit,
        does_section_slot: core.does_section_slot,
        does_section_identity: core.does_section_identity.clone(),
        does_section_name: core.does_section_name.clone(),
        does_section_range: core.does_section_range.clone(),
        statement_count: core.statement_count,
        statement_node_identity: core.statement_node_identity.clone(),
        statement_kind: core.statement_kind,
        block_relationship: core.block_relationship,
        block_depth_before: core.block_depth_before,
        block_depth_after: core.block_depth_after,
        block_identity: core.block_identity,
        operation_identities: [core.return_operation_identity, core.add_node_identity],
        operation_kinds: core.operation_kinds,
        child_node_identities: [core.left_node_identity, core.right_node_identity],
        add_kind: core.add_kind,
        add_operator: core.add_operator,
        add_completion: core.add_completion,
        child_kinds: [core.left_kind, core.right_kind],
        child_completions: [core.left_completion, core.right_completion],
        ordered_child_relationship: core.ordered_child_relationship.clone(),
        operand_value_identities: [core.left_value_identity, core.right_value_identity],
        result_value_identity: core.result_value_identity,
        resolver_use_identities: [
            resolver.left_use_identity.clone(),
            resolver.right_use_identity.clone(),
        ],
        resolver_definition_identities: [
            resolver.left_definition_identity.clone(),
            resolver.right_definition_identity.clone(),
        ],
        resolver_use_order_identity: resolver.use_order_identity.clone(),
        resolver_definition_order_identity: resolver.definition_order_identity.clone(),
        resolver_distinct_binding_status: resolver.distinct_binding_status,
        parameter_type_identities: types.parameter_type_ids.clone(),
        operand_type_identities: types.operand_type_ids.clone(),
        add_result_type_identity: types.add_result_type_id.clone(),
        function_result_type_identity: types.function_result_type_id.clone(),
        effect_identity: effect.effect_identity,
        authority_identity: effect.authority_identity,
        ownership_identity: ownership.ownership_identity,
        resource_identity: resource.resource_identity.clone(),
        profile_identity: profile.profile_identity.to_string(),
        overflow_edge_identity: core.overflow_edge_identity,
        overflow_status: core.overflow_status,
        accepted_passes: vec![
            "canonical_backend_c1_selector_selected_v0",
            "canonical_backend_source_owner_checked_v0",
            "canonical_backend_signature_checked_v0",
            "canonical_backend_body_checked_v0",
            "canonical_backend_core_checked_add_checked_v0",
            "canonical_backend_resolver_checked_v0",
            types.status,
            effect.status,
            ownership.status,
            resource.status,
            resource.allocation_status,
            profile.status,
            profile.profile_status,
        ],
        unsupported_facts,
        missing_artifact,
        missing_verifier,
        status: "canonical_checked_add_producer_closure_available_v0",
    };
    apply_c1_join_producer_corruption(&mut candidate);
    validate_canonical_checked_add_producer_closure(
        &candidate,
        &core_authority,
        &resolver.function_definition_identity,
        &resolver,
        &types,
        &effect.function_identity,
        &ownership.function_identity,
        &resource.function_identity,
        &resource.resource_identity,
        &profile.function_identity,
        &expected_resolver_identity,
        &expected_resolver_definition_identity,
        expected_type,
    )?;
    Ok(CanonicalCheckedAddProducerClosure::from_validated(
        ValidatedCanonicalCheckedAddProducerCandidate(candidate),
    ))
}

fn c1_readiness_producer_facts() -> (Vec<&'static str>, &'static str, &'static str) {
    let mut facts = (
        Vec::new(),
        "hum.backend_input.v0_absent_v0",
        "ir_verify_unimplemented_v0",
    );
    apply_c1_readiness_producer_corruption(&mut facts);
    facts
}

#[allow(clippy::too_many_arguments)]
fn validate_canonical_checked_add_producer_closure(
    closure: &CanonicalCheckedAddProducerCandidate,
    core: &core_lower::CanonicalCheckedAddCoreView,
    resolver_function_identity: &str,
    resolver: &resolve::CanonicalBackendResolverView,
    types: &full_type_check::CanonicalBackendTypeView,
    effect_function_identity: &str,
    ownership_function_identity: &str,
    resource_function_identity: &str,
    resource_identity: &str,
    profile_function_identity: &str,
    expected_resolver_identity: &str,
    expected_resolver_definition_identity: &str,
    expected_type: &str,
) -> Result<(), &'static str> {
    if closure.source_revision.is_empty()
        || closure.normalized_path.is_empty()
        || closure.module_identity.is_empty()
        || closure.function_identity.is_empty()
        || closure.linkage_identity.is_empty()
        || closure.block_identity.is_empty()
    {
        return Err("canonical_backend_owner_or_signature_fact_invalid_v0");
    }
    if closure.source_revision != hex_bytes(&core.source_revision)
        || closure.normalized_path != core.normalized_path
        || closure.semantic_file_index != core.semantic_file_index
        || closure.module_token_identity != core.module_token_identity
        || closure.module_identity != core.module_identity
        || closure.module_display_name != core.module_display_name
        || closure.module_range != core.module_range
        || closure.item_path != core.item_path
        || closure.item_kind != core.item_kind
        || closure.function_identity != core.function_identity
        || closure.function_display_name != core.function_display_name
        || closure.function_range != core.function_range
        || closure.linkage_identity != core.linkage_identity
        || closure.parameter_ordinals != core.parameter_ordinals
        || closure.parameter_identities != core.parameter_identities
        || closure.parameter_names != core.parameter_names
        || closure.parameter_ranges != core.parameter_ranges
        || closure.parameter_type_token_identities != core.parameter_type_token_identities
        || closure.parameter_type_names != core.parameter_type_names
        || closure.parameter_type_ranges != core.parameter_type_ranges
        || closure.parameter_permissions != core.parameter_permissions
        || closure.result_type_token_identity != core.result_type_token_identity
        || closure.result_type_name != core.result_type_name
        || closure.result_type_range != core.result_type_range
        || closure.result_type_explicit != core.result_type_explicit
        || closure.does_section_slot != core.does_section_slot
        || closure.does_section_identity != core.does_section_identity
        || closure.does_section_name != core.does_section_name
        || closure.does_section_range != core.does_section_range
        || closure.statement_count != core.statement_count
        || closure.statement_node_identity != core.statement_node_identity
        || closure.statement_kind != core.statement_kind
        || closure.block_relationship != core.block_relationship
        || closure.block_depth_before != core.block_depth_before
        || closure.block_depth_after != core.block_depth_after
        || closure.block_identity != core.block_identity
        || closure.operation_identities
            != [
                core.return_operation_identity.clone(),
                core.add_node_identity.clone(),
            ]
        || closure.child_node_identities
            != [
                core.left_node_identity.clone(),
                core.right_node_identity.clone(),
            ]
        || closure.add_kind != core.add_kind
        || closure.add_operator != core.add_operator
        || closure.add_completion != core.add_completion
        || closure.child_kinds != [core.left_kind, core.right_kind]
        || closure.child_completions != [core.left_completion, core.right_completion]
        || closure.ordered_child_relationship != core.ordered_child_relationship
        || closure.operand_value_identities
            != [
                core.left_value_identity.clone(),
                core.right_value_identity.clone(),
            ]
        || closure.result_value_identity != core.result_value_identity
        || closure.overflow_edge_identity != core.overflow_edge_identity
        || closure.overflow_status != core.overflow_status
    {
        return Err("canonical_backend_core_authority_join_mismatch_v0");
    }
    if closure.operation_kinds != ["return", "checked_add"] {
        return Err("canonical_backend_operation_kind_join_mismatch_v0");
    }
    if closure.parameter_identities[0] == closure.parameter_identities[1]
        || closure.parameter_names[0] == closure.parameter_names[1]
        || closure.child_node_identities[0] == closure.child_node_identities[1]
        || closure.operand_value_identities[0] == closure.operand_value_identities[1]
        || closure.resolver_use_identities[0] == closure.resolver_use_identities[1]
        || closure.resolver_definition_identities[0] == closure.resolver_definition_identities[1]
    {
        return Err("canonical_backend_ordered_identity_not_distinct_v0");
    }
    if resolver_function_identity != expected_resolver_definition_identity
        || effect_function_identity != expected_resolver_identity
        || ownership_function_identity != expected_resolver_identity
        || resource_function_identity != expected_resolver_identity
        || profile_function_identity != expected_resolver_identity
    {
        return Err("canonical_backend_function_join_mismatch_v0");
    }
    if closure.resolver_use_identities
        != [
            resolver.left_use_identity.clone(),
            resolver.right_use_identity.clone(),
        ]
        || closure.resolver_definition_identities
            != [
                resolver.left_definition_identity.clone(),
                resolver.right_definition_identity.clone(),
            ]
        || closure.resolver_use_order_identity != resolver.use_order_identity
        || closure.resolver_definition_order_identity != resolver.definition_order_identity
        || closure.resolver_distinct_binding_status != resolver.distinct_binding_status
    {
        return Err("canonical_backend_resolver_join_mismatch_v0");
    }
    if types.function_identity != closure.function_identity
        || types
            .parameter_type_ids
            .iter()
            .any(|value| value != expected_type)
        || types
            .operand_type_ids
            .iter()
            .any(|value| value != expected_type)
        || types.add_result_type_id != expected_type
        || types.function_result_type_id != expected_type
        || closure.parameter_type_identities != types.parameter_type_ids
        || closure.operand_type_identities != types.operand_type_ids
        || closure.add_result_type_identity != types.add_result_type_id
        || closure.function_result_type_identity != types.function_result_type_id
    {
        return Err("canonical_backend_type_join_mismatch_v0");
    }
    if closure.effect_identity != "hum.effect.pure.v0"
        || closure.authority_identity != "hum.authority.none.v0"
        || closure.ownership_identity != "hum.ownership.accepted.no_transfer.v0"
        || closure.resource_identity != resource_identity
        || closure.profile_identity != "normal"
        || closure.overflow_status != "checked_add_runtime_trap_exit_2_v0"
    {
        return Err("canonical_backend_policy_or_failure_join_mismatch_v0");
    }
    if closure.accepted_passes
        != [
            "canonical_backend_c1_selector_selected_v0",
            "canonical_backend_source_owner_checked_v0",
            "canonical_backend_signature_checked_v0",
            "canonical_backend_body_checked_v0",
            "canonical_backend_core_checked_add_checked_v0",
            "canonical_backend_resolver_checked_v0",
            types.status,
            "accepted_canonical_pure_effect_v0",
            "accepted_canonical_ownership_v0",
            "canonical_backend_resource_checked_v0",
            "accepted_conservative_allocation_free_claim_v0",
            "canonical_backend_profile_checked_v0",
            "accepted_normal_profile_policy_v0",
        ]
        || !closure.unsupported_facts.is_empty()
        || closure.missing_artifact != "hum.backend_input.v0_absent_v0"
        || closure.missing_verifier != "ir_verify_unimplemented_v0"
        || closure.status != "canonical_checked_add_producer_closure_available_v0"
    {
        return Err("canonical_backend_pass_or_blocker_join_mismatch_v0");
    }
    Ok(())
}

fn validate_diagnostic_transport_for_ir(program: &Program, diagnostics: &[Diagnostic]) {
    let diagnostic_transport = profile_check::diagnostic_transport(program, diagnostics)
        .expect("profile check must supply IR and graph occurrence projections");
    let diagnostic_occurrences = diagnostic_transport.authoritative();
    profile_check::validate_prior_blocker_projection(program, diagnostics)
        .expect("static prior-blocker projection must preserve exact occurrence identity");
    diagnostic_transport
        .ir_projection()
        .validate_against("ir_readiness", diagnostic_occurrences)
        .expect("IR readiness must consume the separately carried profile projection");
}

fn hex_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
    }
    out
}

#[cfg(test)]
fn apply_c1_readiness_producer_corruption(
    facts: &mut (Vec<&'static str>, &'static str, &'static str),
) {
    match current_c1_corruption() {
        Some(C1ProducerCorruption::UnsupportedFact) => {
            facts.0.push("foreign_unsupported_v0");
        }
        Some(C1ProducerCorruption::MissingArtifact) => {
            facts.1 = "artifact_falsely_present_v0";
        }
        Some(C1ProducerCorruption::MissingVerifier) => {
            facts.2 = "verifier_falsely_present_v0";
        }
        _ => {}
    }
}

#[cfg(not(test))]
fn apply_c1_readiness_producer_corruption(_: &mut (Vec<&'static str>, &'static str, &'static str)) {
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum C1ProducerCorruption {
    SelectorDecision,
    SourceRevision,
    NormalizedPath,
    SemanticFileOrdinal,
    ModuleTokenIdentity,
    ModuleTokenSpelling,
    ModuleTokenRange,
    ItemPath,
    ItemKind,
    FunctionTokenIdentity,
    FunctionTokenSpelling,
    FunctionTokenRange,
    LinkageIdentity,
    ParameterOrdinal(usize),
    ParameterBinderIdentity(usize),
    ParameterBinderSpelling(usize),
    ParameterBinderRange(usize),
    ParameterTypeTokenIdentity(usize),
    ParameterTypeSpelling(usize),
    ParameterTypeRange(usize),
    ParameterPermission(usize),
    ResultTypeIdentity,
    ResultTypeSpelling,
    ResultTypeRange,
    ResultTypeExplicitness,
    DoesSectionSlot,
    DoesSectionIdentity,
    DoesSectionSpelling,
    DoesSectionRange,
    StatementCount,
    StatementNodeIdentity,
    StatementKind,
    BlockRelationship,
    BlockDepthBefore,
    BlockDepthAfter,
    AddNodeIdentity,
    AddKind,
    AddOperator,
    AddCompletion,
    LeftNodeIdentity,
    LeftKind,
    LeftCompletion,
    RightNodeIdentity,
    RightKind,
    RightCompletion,
    OrderedChildRelationship,
    BlockIdentity,
    ReturnOperationIdentity,
    OperationKind(usize),
    OperandValueIdentity(usize),
    ResultValueIdentity,
    OverflowEdgeIdentity,
    OverflowStatus,
    ResolverFunctionIdentity,
    ResolverUseIdentity(usize),
    ResolverUseOrder,
    ResolverDefinitionIdentity(usize),
    ResolverDefinitionOrder,
    ResolverDistinctBinding,
    TypeFunctionIdentity,
    ParameterTypeIdentity(usize),
    OperandTypeIdentity(usize),
    AddResultTypeIdentity,
    FunctionResultTypeIdentity,
    TypeAcceptedStatus,
    EffectFunctionIdentity,
    EffectIdentity,
    AuthorityIdentity,
    EffectAcceptedStatus,
    OwnershipFunctionIdentity,
    OwnershipIdentity,
    OwnershipAcceptedStatus,
    ResourceFunctionIdentity,
    ResourceIdentity,
    ResourceAllocationStatus,
    ResourceProducerStatus,
    ProfileFunctionIdentity,
    ProfileIdentity,
    ProfilePolicyStatus,
    ProfileProducerStatus,
    AcceptedPass(usize),
    UnsupportedFact,
    MissingArtifact,
    MissingVerifier,
    ClosureStatus,
    CoherentOwnerSubstitution,
    CoherentOperandSubstitution,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum C1CorruptionSubcase {
    Primary,
    RangeFile,
    RangeLine,
    RangeColumn,
    RangeByteLength,
    OrderedMemberSwap,
    PostProducerJoin,
}

#[cfg(test)]
thread_local! {
    static C1_PRODUCER_CORRUPTION:
        std::cell::RefCell<Option<(
            C1ProducerCorruption,
            Option<CanonicalCheckedAddProducerCandidate>,
            C1CorruptionSubcase,
        )>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn with_c1_producer_corruption<T>(
    corruption: C1ProducerCorruption,
    foreign: Option<CanonicalCheckedAddProducerCandidate>,
    action: impl FnOnce() -> T,
) -> T {
    with_c1_producer_corruption_subcase(corruption, foreign, C1CorruptionSubcase::Primary, action)
}

#[cfg(test)]
fn with_c1_producer_corruption_subcase<T>(
    corruption: C1ProducerCorruption,
    foreign: Option<CanonicalCheckedAddProducerCandidate>,
    subcase: C1CorruptionSubcase,
    action: impl FnOnce() -> T,
) -> T {
    C1_PRODUCER_CORRUPTION.with(|slot| {
        assert!(
            slot.borrow_mut()
                .replace((corruption, foreign, subcase))
                .is_none()
        );
        let result = action();
        assert!(slot.borrow_mut().take().is_some());
        result
    })
}

#[cfg(test)]
fn current_c1_corruption() -> Option<C1ProducerCorruption> {
    C1_PRODUCER_CORRUPTION.with(|slot| slot.borrow().as_ref().map(|entry| entry.0))
}

#[cfg(test)]
fn current_c1_corruption_subcase() -> Option<C1CorruptionSubcase> {
    C1_PRODUCER_CORRUPTION.with(|slot| slot.borrow().as_ref().map(|entry| entry.2))
}

#[cfg(test)]
fn foreign_c1_candidate() -> Option<CanonicalCheckedAddProducerCandidate> {
    C1_PRODUCER_CORRUPTION.with(|slot| {
        slot.borrow()
            .as_ref()
            .and_then(|entry| entry.1.as_ref().cloned())
    })
}

#[cfg(test)]
fn apply_c1_join_producer_corruption(candidate: &mut CanonicalCheckedAddProducerCandidate) {
    match current_c1_corruption() {
        Some(C1ProducerCorruption::AcceptedPass(index)) => {
            if current_c1_corruption_subcase() == Some(C1CorruptionSubcase::OrderedMemberSwap) {
                let other = if index == 0 { 1 } else { index - 1 };
                candidate.accepted_passes.swap(index, other);
            } else {
                candidate.accepted_passes[index] = "foreign_accepted_pass_v0";
            }
        }
        Some(C1ProducerCorruption::ClosureStatus) => {
            candidate.status = "foreign_closure_status_v0";
        }
        Some(
            corruption @ (C1ProducerCorruption::CoherentOwnerSubstitution
            | C1ProducerCorruption::CoherentOperandSubstitution),
        ) if current_c1_corruption_subcase() == Some(C1CorruptionSubcase::PostProducerJoin) => {
            let foreign =
                foreign_c1_candidate().expect("post-producer substitution needs foreign facts");
            match corruption {
                C1ProducerCorruption::CoherentOwnerSubstitution => {
                    candidate.source_revision = foreign.source_revision;
                    candidate.normalized_path = foreign.normalized_path;
                    candidate.semantic_file_index = foreign.semantic_file_index;
                    candidate.module_token_identity = foreign.module_token_identity;
                    candidate.module_identity = foreign.module_identity;
                    candidate.module_display_name = foreign.module_display_name;
                    candidate.module_range = foreign.module_range;
                    candidate.item_path = foreign.item_path;
                    candidate.item_kind = foreign.item_kind;
                    candidate.function_identity = foreign.function_identity;
                    candidate.function_display_name = foreign.function_display_name;
                    candidate.function_range = foreign.function_range;
                    candidate.linkage_identity = foreign.linkage_identity;
                }
                C1ProducerCorruption::CoherentOperandSubstitution => {
                    candidate.child_node_identities[0] = foreign.child_node_identities[0].clone();
                    candidate.operand_value_identities[0] =
                        foreign.operand_value_identities[0].clone();
                    candidate.resolver_use_identities[0] =
                        foreign.resolver_use_identities[0].clone();
                    candidate.resolver_definition_identities[0] =
                        foreign.resolver_definition_identities[0].clone();
                }
                _ => unreachable!("matched coherent substitution"),
            }
        }
        _ => {}
    }
}

#[cfg(not(test))]
fn apply_c1_join_producer_corruption(_: &mut CanonicalCheckedAddProducerCandidate) {}

#[cfg(test)]
pub(crate) fn apply_c1_parser_producer_corruption(
    binding: &mut crate::parser::CanonicalBackendSignatureBinding,
) -> bool {
    let Some(corruption) = current_c1_corruption() else {
        return true;
    };
    let subcase = current_c1_corruption_subcase().unwrap_or(C1CorruptionSubcase::Primary);
    let corrupt_range = |range: &mut crate::ast::ParsedSourceRange| match subcase {
        C1CorruptionSubcase::RangeFile => {
            range.start.file = "foreign/parser-authority.hum".to_string();
        }
        C1CorruptionSubcase::RangeLine => {
            range.start.line = range.start.line.saturating_add(1);
        }
        C1CorruptionSubcase::RangeColumn => {
            range.start.column = range.start.column.saturating_add(1);
        }
        C1CorruptionSubcase::RangeByteLength => {
            range.byte_len = range.byte_len.saturating_add(1);
        }
        C1CorruptionSubcase::Primary => {
            range.start.file = "foreign/parser-authority.hum".to_string();
            range.start.line = range.start.line.saturating_add(1);
            range.start.column = range.start.column.saturating_add(1);
            range.byte_len = range.byte_len.saturating_add(1);
        }
        C1CorruptionSubcase::OrderedMemberSwap | C1CorruptionSubcase::PostProducerJoin => {
            panic!("range corruption requires a range subcase")
        }
    };
    if subcase == C1CorruptionSubcase::OrderedMemberSwap
        && matches!(corruption, C1ProducerCorruption::ParameterOrdinal(0))
    {
        std::sync::Arc::make_mut(&mut binding.parameters).swap(0, 1);
        return true;
    }
    match corruption {
        C1ProducerCorruption::SelectorDecision => return false,
        C1ProducerCorruption::SourceRevision => binding.file.source_revision = vec![0].into(),
        C1ProducerCorruption::NormalizedPath => {
            binding.file.normalized_path = "foreign/parser-authority.hum".into();
        }
        C1ProducerCorruption::SemanticFileOrdinal => {
            binding.file.semantic_file_index = binding.file.semantic_file_index.saturating_add(1);
        }
        C1ProducerCorruption::ModuleTokenIdentity => {
            binding.module.identity = "foreign-module-token".into();
        }
        C1ProducerCorruption::ModuleTokenSpelling => {
            binding.module.spelling = "foreign.module".into();
        }
        C1ProducerCorruption::ModuleTokenRange => corrupt_range(&mut binding.module.range),
        C1ProducerCorruption::ItemPath => binding.item_path = std::sync::Arc::from([usize::MAX]),
        C1ProducerCorruption::ItemKind => binding.item_kind = "test",
        C1ProducerCorruption::FunctionTokenIdentity => {
            binding.function.identity = "foreign-function-token".into();
        }
        C1ProducerCorruption::FunctionTokenSpelling => {
            binding.function.spelling = "foreign_function".into();
        }
        C1ProducerCorruption::FunctionTokenRange => corrupt_range(&mut binding.function.range),
        C1ProducerCorruption::LinkageIdentity => {
            binding.export_linkage_identity = "foreign-linkage".into();
        }
        C1ProducerCorruption::ParameterOrdinal(index) => {
            std::sync::Arc::make_mut(&mut binding.parameters)[index].ordinal = usize::MAX;
        }
        C1ProducerCorruption::ParameterBinderIdentity(index) => {
            std::sync::Arc::make_mut(&mut binding.parameters)[index]
                .binder
                .identity = "foreign-parameter-token".into();
        }
        C1ProducerCorruption::ParameterBinderSpelling(index) => {
            std::sync::Arc::make_mut(&mut binding.parameters)[index]
                .binder
                .spelling = "foreign_parameter".into();
        }
        C1ProducerCorruption::ParameterBinderRange(index) => {
            corrupt_range(
                &mut std::sync::Arc::make_mut(&mut binding.parameters)[index]
                    .binder
                    .range,
            );
        }
        C1ProducerCorruption::ParameterTypeTokenIdentity(index) => {
            std::sync::Arc::make_mut(&mut binding.parameters)[index]
                .declared_type
                .identity = "foreign-type-token".into();
        }
        C1ProducerCorruption::ParameterTypeSpelling(index) => {
            std::sync::Arc::make_mut(&mut binding.parameters)[index]
                .declared_type
                .spelling = "ForeignType".into();
        }
        C1ProducerCorruption::ParameterTypeRange(index) => {
            corrupt_range(
                &mut std::sync::Arc::make_mut(&mut binding.parameters)[index]
                    .declared_type
                    .range,
            );
        }
        C1ProducerCorruption::ParameterPermission(index) => {
            std::sync::Arc::make_mut(&mut binding.parameters)[index].permission =
                crate::ast::ParamPermission::Change;
        }
        C1ProducerCorruption::ResultTypeIdentity => {
            binding.result_type.identity = "foreign-result-token".into();
        }
        C1ProducerCorruption::ResultTypeSpelling => {
            binding.result_type.spelling = "ForeignType".into();
        }
        C1ProducerCorruption::ResultTypeRange => corrupt_range(&mut binding.result_type.range),
        C1ProducerCorruption::ResultTypeExplicitness => {
            binding.result_type_explicit = !binding.result_type_explicit;
        }
        C1ProducerCorruption::DoesSectionSlot => {
            binding.does_section_slot = binding.does_section_slot.saturating_add(1);
        }
        C1ProducerCorruption::DoesSectionIdentity => {
            binding.does_section.identity = "foreign-does-token".into();
        }
        C1ProducerCorruption::DoesSectionSpelling => {
            binding.does_section.spelling = "foreign_does".into();
        }
        C1ProducerCorruption::DoesSectionRange => corrupt_range(&mut binding.does_section.range),
        _ => {}
    }
    true
}

#[cfg(not(test))]
pub(crate) fn apply_c1_parser_producer_corruption(
    _: &mut crate::parser::CanonicalBackendSignatureBinding,
) -> bool {
    true
}

#[cfg(test)]
pub(crate) fn apply_c1_core_producer_corruption(
    view: &mut core_lower::CanonicalCheckedAddCoreView,
) {
    let Some(corruption) = current_c1_corruption() else {
        return;
    };
    let foreign = foreign_c1_candidate();
    let subcase = current_c1_corruption_subcase().unwrap_or(C1CorruptionSubcase::Primary);
    if subcase == C1CorruptionSubcase::OrderedMemberSwap {
        match corruption {
            C1ProducerCorruption::ReturnOperationIdentity => {
                std::mem::swap(
                    &mut view.return_operation_identity,
                    &mut view.add_node_identity,
                );
                return;
            }
            C1ProducerCorruption::LeftNodeIdentity => {
                std::mem::swap(&mut view.left_node_identity, &mut view.right_node_identity);
                return;
            }
            C1ProducerCorruption::OperationKind(0) => {
                view.operation_kinds.swap(0, 1);
                return;
            }
            C1ProducerCorruption::OperandValueIdentity(0) => {
                std::mem::swap(
                    &mut view.left_value_identity,
                    &mut view.right_value_identity,
                );
                return;
            }
            _ => {}
        }
    }
    match corruption {
        C1ProducerCorruption::StatementCount => {
            view.statement_count = view.statement_count.saturating_add(1);
        }
        C1ProducerCorruption::StatementNodeIdentity => {
            view.statement_node_identity = "foreign-statement-node".into();
        }
        C1ProducerCorruption::StatementKind => view.statement_kind = "foreign_statement",
        C1ProducerCorruption::BlockRelationship => {
            view.block_relationship = "foreign_relationship";
        }
        C1ProducerCorruption::BlockDepthBefore => {
            view.block_depth_before = view.block_depth_before.saturating_add(1);
        }
        C1ProducerCorruption::BlockDepthAfter => {
            view.block_depth_after = view.block_depth_after.saturating_add(1);
        }
        C1ProducerCorruption::AddNodeIdentity => {
            view.add_node_identity = "foreign-add".into();
        }
        C1ProducerCorruption::AddKind => view.add_kind = "foreign_add_kind",
        C1ProducerCorruption::AddOperator => view.add_operator = "foreign_add_operator",
        C1ProducerCorruption::AddCompletion => view.add_completion = "foreign_completion",
        C1ProducerCorruption::LeftNodeIdentity => {
            view.left_node_identity = "foreign-left".into();
        }
        C1ProducerCorruption::LeftKind => view.left_kind = "foreign_left_kind",
        C1ProducerCorruption::LeftCompletion => view.left_completion = "foreign_completion",
        C1ProducerCorruption::RightNodeIdentity => {
            view.right_node_identity = "foreign-right".into();
        }
        C1ProducerCorruption::RightKind => view.right_kind = "foreign_right_kind",
        C1ProducerCorruption::RightCompletion => view.right_completion = "foreign_completion",
        C1ProducerCorruption::OrderedChildRelationship => {
            view.ordered_child_relationship = "foreign-child-relationship".into();
        }
        C1ProducerCorruption::BlockIdentity => view.block_identity = "foreign-block".into(),
        C1ProducerCorruption::ReturnOperationIdentity => {
            view.return_operation_identity = "foreign-return".into();
        }
        C1ProducerCorruption::OperationKind(index) => {
            view.operation_kinds[index] = "foreign_operation";
        }
        C1ProducerCorruption::OperandValueIdentity(index) => {
            if index == 0 {
                view.left_value_identity = "foreign-left-value".into();
            } else {
                view.right_value_identity = "foreign-right-value".into();
            }
        }
        C1ProducerCorruption::ResultValueIdentity => {
            view.result_value_identity = "foreign-result-value".into();
        }
        C1ProducerCorruption::OverflowEdgeIdentity => {
            view.overflow_edge_identity = "foreign-overflow-edge".into();
        }
        C1ProducerCorruption::OverflowStatus => view.overflow_status = "foreign_overflow_v0",
        C1ProducerCorruption::CoherentOwnerSubstitution
            if subcase != C1CorruptionSubcase::PostProducerJoin =>
        {
            let foreign =
                foreign.expect("coherent owner substitution requires a real foreign task");
            view.source_revision = decode_hex(&foreign.source_revision).into();
            view.normalized_path = foreign.normalized_path;
            view.semantic_file_index = foreign.semantic_file_index;
            view.module_token_identity = foreign.module_token_identity;
            view.module_identity = foreign.module_identity;
            view.module_display_name = foreign.module_display_name;
            view.module_range = foreign.module_range;
            view.item_path = foreign.item_path;
            view.item_kind = foreign.item_kind;
            view.function_identity = foreign.function_identity;
            view.function_display_name = foreign.function_display_name;
            view.function_range = foreign.function_range;
            view.linkage_identity = foreign.linkage_identity;
        }
        C1ProducerCorruption::CoherentOperandSubstitution
            if subcase != C1CorruptionSubcase::PostProducerJoin =>
        {
            let foreign =
                foreign.expect("coherent operand substitution requires a real foreign task");
            view.left_node_identity = foreign.child_node_identities[0].clone();
            view.left_value_identity = foreign.operand_value_identities[0].clone();
            view.ordered_child_relationship = foreign.ordered_child_relationship;
        }
        _ => {}
    }
}

#[cfg(not(test))]
pub(crate) fn apply_c1_core_producer_corruption(_: &mut core_lower::CanonicalCheckedAddCoreView) {}

#[cfg(test)]
pub(crate) fn apply_c1_resolver_producer_corruption(
    view: &mut resolve::CanonicalBackendResolverView,
) {
    let Some(corruption) = current_c1_corruption() else {
        return;
    };
    match corruption {
        C1ProducerCorruption::ResolverFunctionIdentity => {
            view.function_definition_identity = "foreign-resolver-function".into();
        }
        C1ProducerCorruption::ResolverUseIdentity(index) => {
            if index == 0 {
                view.left_use_identity = "foreign-resolver-use".into();
            } else {
                view.right_use_identity = "foreign-resolver-use".into();
            }
        }
        C1ProducerCorruption::ResolverUseOrder => {
            std::mem::swap(&mut view.left_use_identity, &mut view.right_use_identity);
        }
        C1ProducerCorruption::ResolverDefinitionIdentity(index) => {
            if index == 0 {
                view.left_definition_identity = "foreign-resolver-definition".into();
            } else {
                view.right_definition_identity = "foreign-resolver-definition".into();
            }
        }
        C1ProducerCorruption::ResolverDefinitionOrder => {
            std::mem::swap(
                &mut view.left_definition_identity,
                &mut view.right_definition_identity,
            );
        }
        C1ProducerCorruption::ResolverDistinctBinding => {
            view.distinct_binding_status = "foreign_distinct_binding_v0";
        }
        C1ProducerCorruption::CoherentOperandSubstitution
            if current_c1_corruption_subcase() != Some(C1CorruptionSubcase::PostProducerJoin) =>
        {
            let foreign =
                foreign_c1_candidate().expect("coherent operand substitution needs foreign facts");
            view.left_use_identity = foreign.resolver_use_identities[0].clone();
            view.left_definition_identity = foreign.resolver_definition_identities[0].clone();
        }
        _ => {}
    }
}

#[cfg(not(test))]
pub(crate) fn apply_c1_resolver_producer_corruption(_: &mut resolve::CanonicalBackendResolverView) {
}

#[cfg(test)]
pub(crate) fn apply_c1_type_producer_corruption(
    view: &mut full_type_check::CanonicalBackendTypeView,
) {
    let Some(corruption) = current_c1_corruption() else {
        return;
    };
    match corruption {
        C1ProducerCorruption::TypeFunctionIdentity => {
            view.function_identity = "foreign-type-function".into();
        }
        C1ProducerCorruption::ParameterTypeIdentity(index) => {
            view.parameter_type_ids[index] = "hum.type.foreign.v0".into();
        }
        C1ProducerCorruption::OperandTypeIdentity(index) => {
            view.operand_type_ids[index] = "hum.type.foreign.v0".into();
        }
        C1ProducerCorruption::AddResultTypeIdentity => {
            view.add_result_type_id = "hum.type.foreign.v0".into();
        }
        C1ProducerCorruption::FunctionResultTypeIdentity => {
            view.function_result_type_id = "hum.type.foreign.v0".into();
        }
        C1ProducerCorruption::TypeAcceptedStatus => view.status = "foreign_type_pass_v0",
        _ => {}
    }
}

#[cfg(not(test))]
pub(crate) fn apply_c1_type_producer_corruption(_: &mut full_type_check::CanonicalBackendTypeView) {
}

#[cfg(test)]
pub(crate) fn apply_c1_effect_producer_corruption(
    view: &mut effect_check::CanonicalBackendEffectView,
) {
    match current_c1_corruption() {
        Some(C1ProducerCorruption::EffectFunctionIdentity) => {
            view.function_identity = "foreign-effect-function".into();
        }
        Some(C1ProducerCorruption::EffectIdentity) => {
            view.effect_identity = "hum.effect.foreign.v0".into();
        }
        Some(C1ProducerCorruption::AuthorityIdentity) => {
            view.authority_identity = "hum.authority.foreign.v0".into();
        }
        Some(C1ProducerCorruption::EffectAcceptedStatus) => view.status = "foreign_effect_pass_v0",
        _ => {}
    }
}

#[cfg(not(test))]
pub(crate) fn apply_c1_effect_producer_corruption(
    _: &mut effect_check::CanonicalBackendEffectView,
) {
}

#[cfg(test)]
pub(crate) fn apply_c1_ownership_producer_corruption(
    view: &mut ownership_check::CanonicalBackendOwnershipView,
) {
    match current_c1_corruption() {
        Some(C1ProducerCorruption::OwnershipFunctionIdentity) => {
            view.function_identity = "foreign-ownership-function".into();
        }
        Some(C1ProducerCorruption::OwnershipIdentity) => {
            view.ownership_identity = "hum.ownership.foreign.v0".into();
        }
        Some(C1ProducerCorruption::OwnershipAcceptedStatus) => {
            view.status = "foreign_ownership_pass_v0";
        }
        _ => {}
    }
}

#[cfg(not(test))]
pub(crate) fn apply_c1_ownership_producer_corruption(
    _: &mut ownership_check::CanonicalBackendOwnershipView,
) {
}

#[cfg(test)]
pub(crate) fn apply_c1_resource_producer_corruption(
    view: &mut resource_check::CanonicalBackendResourceView,
) {
    match current_c1_corruption() {
        Some(C1ProducerCorruption::ResourceFunctionIdentity) => {
            view.function_identity = "foreign-resource-function".into();
        }
        Some(C1ProducerCorruption::ResourceIdentity) => {
            view.resource_identity = "hum.resource.foreign.v0".into();
        }
        Some(C1ProducerCorruption::ResourceProducerStatus) => {
            view.status = "foreign_resource_pass_v0";
        }
        Some(C1ProducerCorruption::ResourceAllocationStatus) => {
            view.allocation_status = "foreign_allocation_pass_v0";
        }
        _ => {}
    }
}

#[cfg(not(test))]
pub(crate) fn apply_c1_resource_producer_corruption(
    _: &mut resource_check::CanonicalBackendResourceView,
) {
}

#[cfg(test)]
pub(crate) fn apply_c1_profile_producer_corruption(
    view: &mut profile_check::CanonicalBackendProfileView,
) {
    match current_c1_corruption() {
        Some(C1ProducerCorruption::ProfileFunctionIdentity) => {
            view.function_identity = "foreign-profile-function".into();
        }
        Some(C1ProducerCorruption::ProfileIdentity) => {
            view.profile_identity = "foreign-profile";
        }
        Some(C1ProducerCorruption::ProfileProducerStatus) => {
            view.status = "foreign_profile_pass_v0";
        }
        Some(C1ProducerCorruption::ProfilePolicyStatus) => {
            view.profile_status = "foreign_profile_policy_v0";
        }
        _ => {}
    }
}

#[cfg(not(test))]
pub(crate) fn apply_c1_profile_producer_corruption(
    _: &mut profile_check::CanonicalBackendProfileView,
) {
}

#[cfg(test)]
fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("hex is ASCII");
            u8::from_str_radix(text, 16).expect("source revision is hex")
        })
        .collect()
}

pub fn ir_readiness_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let blocked = report.blocked_count();
    let mut out = String::new();
    out.push_str(&format!("Hum IR readiness ({IR_READINESS_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!(
        "summary: files={} items={} tasks={} tests={} lowering_candidates={} ready_for_ir=0 blocked={} errors={} warnings={} body_grammar_candidates={} body_grammar_recognized_lines={} body_grammar_unsupported_lines={} resolver_status={} resolver_errors={} unresolved_references={} type_check_status={} type_errors={} unknown_type_references={} checked_returns={} rejected_returns={} unchecked_returns={}\n",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.candidates.len(),
        blocked,
        report.errors,
        report.warnings,
        report.body_grammar_candidates(),
        report.body_grammar_recognized_lines(),
        report.body_grammar_unsupported_lines(),
        report.resolve_summary.status,
        report.resolve_summary.resolver_errors,
        report.resolve_summary.unresolved_references,
        report.type_check_summary.status,
        report.type_check_summary.type_errors,
        report.type_check_summary.unknown_type_references,
        report.type_check_summary.checked_returns,
        report.type_check_summary.rejected_returns,
        report.type_check_summary.unchecked_returns
    ));
    out.push_str(&format!(
        "core_contract_schema: {}\n",
        core_contract::CORE_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "ir_contract_schema: {}\n",
        ir_contract::IR_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "resolver: schema={} status={} mode={} scopes={} definitions={} references={} resolved={} unresolved={} external={} duplicate_definitions={} mutable_place_errors={} resolver_errors={} resolver_warnings={}\n",
        report.resolve_summary.schema,
        report.resolve_summary.status,
        report.resolve_summary.mode,
        report.resolve_summary.scopes,
        report.resolve_summary.definitions,
        report.resolve_summary.references,
        report.resolve_summary.resolved_references,
        report.resolve_summary.unresolved_references,
        report.resolve_summary.external_references,
        report.resolve_summary.duplicate_definitions,
        report.resolve_summary.mutable_place_errors,
        report.resolve_summary.resolver_errors,
        report.resolve_summary.resolver_warnings
    ));
    out.push_str(&format!(
        "type_check: schema={} status={} mode={} checked_declarations={} rejected_declarations={} checked_returns={} rejected_returns={} unchecked_returns={} type_errors={} unknown_type_references={}\n",
        report.type_check_summary.schema,
        report.type_check_summary.status,
        report.type_check_summary.mode,
        report.type_check_summary.checked_declarations,
        report.type_check_summary.rejected_declarations,
        report.type_check_summary.checked_returns,
        report.type_check_summary.rejected_returns,
        report.type_check_summary.unchecked_returns,
        report.type_check_summary.type_errors,
        report.type_check_summary.unknown_type_references
    ));
    out.push_str(&format!(
        "core_preview: schema={} status={} core_candidates={} lowerable_preview_statements={} blocked_statements={} expression_previews={} expression_ast_nodes={} typed_expression_previews={}\n",
        report.core_preview_summary.schema,
        report.core_preview_summary.status,
        report.core_preview_summary.core_candidates,
        report.core_preview_summary.lowerable_preview_statements,
        report.core_preview_summary.blocked_statements,
        report.core_preview_summary.expression_previews,
        report.core_preview_summary.expression_ast_nodes,
        report.core_preview_summary.typed_expression_previews
    ));
    out.push_str(&format!(
        "core_lower: schema={} status={} core_items={} lowered_items={} blocked_items={} lowered_operations={} blocked_operations={} execution_ready={} ir_ready={}\n",
        report.core_lower_summary.schema,
        report.core_lower_summary.status,
        report.core_lower_summary.core_items,
        report.core_lower_summary.lowered_items,
        report.core_lower_summary.blocked_items,
        report.core_lower_summary.lowered_operations,
        report.core_lower_summary.blocked_operations,
        report.core_lower_summary.execution_ready,
        report.core_lower_summary.ir_ready
    ));
    out.push_str(&format!(
        "core_verify: schema={} status={} mode={} core_items={} verified_items={} lower_blocked_items={} operations={} verified_operations={} lower_blocked_operations={} checks={} failed_checks={} execution_ready={} ir_ready={}\n",
        report.core_verify_summary.schema,
        report.core_verify_summary.status,
        report.core_verify_summary.mode,
        report.core_verify_summary.core_items,
        report.core_verify_summary.verified_items,
        report.core_verify_summary.lower_blocked_items,
        report.core_verify_summary.operations,
        report.core_verify_summary.verified_operations,
        report.core_verify_summary.lower_blocked_operations,
        report.core_verify_summary.checks,
        report.core_verify_summary.failed_checks,
        report.core_verify_summary.execution_ready,
        report.core_verify_summary.ir_ready
    ));
    out.push_str(&format!(
        "full_type_check: schema={} status={} mode={} body_items={} statements={} checked_statements={} accepted_statements={} rejected_statements={} unchecked_statements={} unsupported_statements={} blocking_issues={} execution_ready={} ir_ready={}\n",
        report.full_type_check_summary.schema,
        report.full_type_check_summary.status,
        report.full_type_check_summary.mode,
        report.full_type_check_summary.body_items,
        report.full_type_check_summary.statements,
        report.full_type_check_summary.checked_statements,
        report.full_type_check_summary.accepted_statements,
        report.full_type_check_summary.rejected_statements,
        report.full_type_check_summary.unchecked_statements,
        report.full_type_check_summary.unsupported_statements,
        report.full_type_check_summary.blocking_issues,
        report.full_type_check_summary.execution_ready,
        report.full_type_check_summary.ir_ready
    ));
    out.push_str(&format!(
        "effect_check: schema={} status={} mode={} effect_items={} statements={} checked_statements={} accepted_statements={} rejected_statements={} unchecked_statements={} boundary_checks={} rejected_boundary_checks={} blocking_issues={} execution_ready={} ir_ready={}\n",
        report.effect_check_summary.schema,
        report.effect_check_summary.status,
        report.effect_check_summary.mode,
        report.effect_check_summary.effect_items,
        report.effect_check_summary.statements,
        report.effect_check_summary.checked_statements,
        report.effect_check_summary.accepted_statements,
        report.effect_check_summary.rejected_statements,
        report.effect_check_summary.unchecked_statements,
        report.effect_check_summary.boundary_checks,
        report.effect_check_summary.rejected_boundary_checks,
        report.effect_check_summary.blocking_issues,
        report.effect_check_summary.execution_ready,
        report.effect_check_summary.ir_ready
    ));
    out.push_str(&format!(
        "ownership_check: schema={} status={} mode={} ownership_items={} statements={} checked_statements={} accepted_statements={} rejected_statements={} unchecked_statements={} boundary_checks={} rejected_boundary_checks={} blocking_issues={} execution_ready={} ir_ready={}\n",
        report.ownership_check_summary.schema,
        report.ownership_check_summary.status,
        report.ownership_check_summary.mode,
        report.ownership_check_summary.ownership_items,
        report.ownership_check_summary.statements,
        report.ownership_check_summary.checked_statements,
        report.ownership_check_summary.accepted_statements,
        report.ownership_check_summary.rejected_statements,
        report.ownership_check_summary.unchecked_statements,
        report.ownership_check_summary.boundary_checks,
        report.ownership_check_summary.rejected_boundary_checks,
        report.ownership_check_summary.blocking_issues,
        report.ownership_check_summary.execution_ready,
        report.ownership_check_summary.ir_ready
    ));
    out.push_str(&format!(
        "resource_check: schema={} status={} mode={} resource_items={} resource_claims={} allocation_claims={} allocation_free_claims={} checks={} accepted_checks={} rejected_checks={} unchecked_checks={} blocking_issues={} proof_ready={} execution_ready={} ir_ready={}\n",
        report.resource_check_summary.schema,
        report.resource_check_summary.status,
        report.resource_check_summary.mode,
        report.resource_check_summary.resource_items,
        report.resource_check_summary.resource_claims,
        report.resource_check_summary.allocation_claims,
        report.resource_check_summary.allocation_free_claims,
        report.resource_check_summary.checks,
        report.resource_check_summary.accepted_checks,
        report.resource_check_summary.rejected_checks,
        report.resource_check_summary.unchecked_checks,
        report.resource_check_summary.blocking_issues,
        report.resource_check_summary.proof_ready,
        report.resource_check_summary.execution_ready,
        report.resource_check_summary.ir_ready
    ));
    out.push_str(&format!(
        "profile_check: schema={} status={} mode={} profile_items={} declared_profiles={} default_profiles={} known_profiles={} unknown_profiles={} strict_profiles={} checks={} accepted_checks={} rejected_checks={} unchecked_checks={} blocking_issues={} proof_ready={} execution_ready={} ir_ready={}\n",
        report.profile_check_summary.schema,
        report.profile_check_summary.status,
        report.profile_check_summary.mode,
        report.profile_check_summary.profile_items,
        report.profile_check_summary.declared_profiles,
        report.profile_check_summary.default_profiles,
        report.profile_check_summary.known_profiles,
        report.profile_check_summary.unknown_profiles,
        report.profile_check_summary.strict_profiles,
        report.profile_check_summary.checks,
        report.profile_check_summary.accepted_checks,
        report.profile_check_summary.rejected_checks,
        report.profile_check_summary.unchecked_checks,
        report.profile_check_summary.blocking_issues,
        report.profile_check_summary.proof_ready,
        report.profile_check_summary.execution_ready,
        report.profile_check_summary.ir_ready
    ));
    out.push_str("pass_status:\n");
    for pass in PASS_STATUSES {
        out.push_str(&format!(
            "  {} [{}]: {}\n",
            pass.name, pass.status, pass.source
        ));
    }

    if report.candidates.is_empty() {
        out.push_str("lowering_candidates: none\n");
        return out;
    }

    out.push_str("lowering_candidates:\n");
    for candidate in &report.candidates {
        out.push_str(&format!(
            "  {}:{}:{} [{}] {} `{}` -> {}\n",
            candidate.span.file,
            candidate.span.line,
            candidate.span.column,
            candidate.status,
            candidate.kind,
            candidate.name,
            candidate.target_layer
        ));
        out.push_str(&format!("    graph_node_id: {}\n", candidate.graph_node_id));
        out.push_str(&format!(
            "    facts_available: {}\n",
            candidate.facts_available.join(", ")
        ));
        out.push_str(&format!(
            "    missing_passes: {}\n",
            candidate.missing_passes.join(", ")
        ));
        out.push_str(&format!(
            "    blocking_reasons: {}\n",
            candidate.blocking_reasons.join(", ")
        ));
        if let Some(body_grammar) = &candidate.body_grammar {
            out.push_str(&format!(
                "    body_grammar: {} meaningful_lines={} recognized_lines={} unsupported_lines={}\n",
                body_grammar.status,
                body_grammar.meaningful_lines,
                body_grammar.recognized_lines,
                body_grammar.unsupported_lines
            ));
        }
        if let Some(closure) = &candidate.checked_add_producer_closure {
            out.push_str(&format!(
                "    checked_add_producer_closure: {} source_revision={} semantic_file_index={} module={} function={} linkage={} parameters=[{}, {}] parameter_types=[{}, {}] result_type={} block={} operations=[{}, {}] children=[{}, {}] resolver_bindings=[{}, {}] type={} effect={} authority={} ownership={} resource={} profile={} overflow={} missing=[{}, {}]\n",
                closure.status,
                closure.source_revision,
                closure.semantic_file_index,
                closure.module_identity,
                closure.function_identity,
                closure.linkage_identity,
                closure.parameter_identities[0],
                closure.parameter_identities[1],
                closure.parameter_type_identities[0],
                closure.parameter_type_identities[1],
                closure.function_result_type_identity,
                closure.block_identity,
                closure.operation_identities[0],
                closure.operation_identities[1],
                closure.child_node_identities[0],
                closure.child_node_identities[1],
                closure.resolver_definition_identities[0],
                closure.resolver_definition_identities[1],
                closure.add_result_type_identity,
                closure.effect_identity,
                closure.authority_identity,
                closure.ownership_identity,
                closure.resource_identity,
                closure.profile_identity,
                closure.overflow_edge_identity,
                closure.missing_artifact,
                closure.missing_verifier,
            ));
            out.push_str(&format!(
                "    checked_add_signature: path={} item_path={:?} item_kind={} module_token={} module_range={}:{}:{}+{} function_range={}:{}:{}+{} ordinals={:?} parameter_ranges={:?} parameter_type_tokens={:?} parameter_type_ranges={:?} permissions={:?} result_token={} result_range={}:{}:{}+{} result_explicit={} does_slot={} does_token={} does_range={}:{}:{}+{}\n",
                closure.normalized_path,
                closure.item_path,
                closure.item_kind,
                closure.module_token_identity,
                closure.module_range.start.file,
                closure.module_range.start.line,
                closure.module_range.start.column,
                closure.module_range.byte_len,
                closure.function_range.start.file,
                closure.function_range.start.line,
                closure.function_range.start.column,
                closure.function_range.byte_len,
                closure.parameter_ordinals,
                closure.parameter_ranges,
                closure.parameter_type_token_identities,
                closure.parameter_type_ranges,
                closure.parameter_permissions,
                closure.result_type_token_identity,
                closure.result_type_range.start.file,
                closure.result_type_range.start.line,
                closure.result_type_range.start.column,
                closure.result_type_range.byte_len,
                closure.result_type_explicit,
                closure.does_section_slot,
                closure.does_section_identity,
                closure.does_section_range.start.file,
                closure.does_section_range.start.line,
                closure.does_section_range.start.column,
                closure.does_section_range.byte_len,
            ));
            out.push_str(&format!(
                "    checked_add_body: statements={} statement={} kind={} relationship={} depths={}/{} add={}/{}/{}/{} children={:?} kinds={:?} completions={:?} ordered={} values={:?}/{} overflow={}/{}\n",
                closure.statement_count,
                closure.statement_node_identity,
                closure.statement_kind,
                closure.block_relationship,
                closure.block_depth_before,
                closure.block_depth_after,
                closure.operation_identities[1],
                closure.add_kind,
                closure.add_operator,
                closure.add_completion,
                closure.child_node_identities,
                closure.child_kinds,
                closure.child_completions,
                closure.ordered_child_relationship,
                closure.operand_value_identities,
                closure.result_value_identity,
                closure.overflow_edge_identity,
                closure.overflow_status,
            ));
            out.push_str(&format!(
                "    checked_add_join: resolver_uses={:?} resolver_targets={:?} use_order={} target_order={} distinct={} parameter_types={:?} operand_types={:?} passes=[{}] unsupported=[{}]\n",
                closure.resolver_use_identities,
                closure.resolver_definition_identities,
                closure.resolver_use_order_identity,
                closure.resolver_definition_order_identity,
                closure.resolver_distinct_binding_status,
                closure.parameter_type_identities,
                closure.operand_type_identities,
                closure.accepted_passes.join(", "),
                closure.unsupported_facts.join(", "),
            ));
        } else if let Some(blocker) = candidate.checked_add_producer_blocker {
            out.push_str(&format!(
                "    checked_add_producer_closure: unsupported blocker={blocker}\n"
            ));
        }
    }

    out
}

pub fn ir_readiness_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", IR_READINESS_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
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
        "ir_contract_schema",
        ir_contract::IR_CONTRACT_SCHEMA,
        true,
    );
    push_resolver_summary(&mut out, &report.resolve_summary, 2, true);
    push_type_check_summary(&mut out, &report.type_check_summary, 2, true);
    push_core_preview_summary(&mut out, &report.core_preview_summary, 2, true);
    push_core_lower_summary(&mut out, &report.core_lower_summary, 2, true);
    push_core_verify_summary(&mut out, &report.core_verify_summary, 2, true);
    push_full_type_check_summary(&mut out, &report.full_type_check_summary, 2, true);
    push_effect_check_summary(&mut out, &report.effect_check_summary, 2, true);
    push_ownership_check_summary(&mut out, &report.ownership_check_summary, 2, true);
    push_resource_check_summary(&mut out, &report.resource_check_summary, 2, true);
    push_profile_check_summary(&mut out, &report.profile_check_summary, 2, true);
    push_summary(&mut out, &report, 2, true);
    push_pass_status(&mut out, 2, true);
    push_candidates(&mut out, &report.candidates, 2, true);
    push_string_array(
        &mut out,
        2,
        "non_goals_v0",
        &[
            "no IR emission",
            "no executable semantics",
            "no backend lowering",
            "no optimizer claim",
            "no proof of type or memory safety",
        ],
        false,
    );
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> IrReadinessReport {
    let resolve_summary = resolve::resolve_readiness_summary(program, diagnostics);
    let type_check_summary = type_check::type_check_summary(program, diagnostics);
    let core_preview_summary = core_preview::core_preview_readiness_summary(program, diagnostics);
    let core_lower_summary = core_lower::core_lower_readiness_summary(program, diagnostics);
    let core_verify_summary = core_verify::core_verify_readiness_summary(program, diagnostics);
    let full_type_check_summary = full_type_check::full_type_check_summary(program, diagnostics);
    let effect_check_summary = effect_check::effect_check_summary(program, diagnostics);
    let ownership_check_summary = ownership_check::ownership_check_summary(program, diagnostics);
    let resource_check_summary = resource_check::resource_check_summary(program, diagnostics);
    let profile_check_summary = profile_check::profile_check_summary(program, diagnostics);
    validate_diagnostic_transport_for_ir(program, diagnostics);
    let checked_returns = type_check::checked_return_summaries(program, diagnostics);
    let context = CandidateContext {
        program,
        diagnostics,
        resolve_summary: &resolve_summary,
        type_check_summary: &type_check_summary,
        core_preview_summary: &core_preview_summary,
        core_lower_summary: &core_lower_summary,
        core_verify_summary: &core_verify_summary,
        full_type_check_summary: &full_type_check_summary,
        effect_check_summary: &effect_check_summary,
        ownership_check_summary: &ownership_check_summary,
        resource_check_summary: &resource_check_summary,
        profile_check_summary: &profile_check_summary,
        checked_returns: &checked_returns,
    };
    let mut candidates = Vec::new();
    for file in &program.files {
        collect_candidates_from_items(&file.items, &context, &mut candidates);
    }

    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len().saturating_sub(errors);
    let tasks = candidates
        .iter()
        .filter(|candidate| candidate.kind == "task")
        .count();
    let tests = candidates
        .iter()
        .filter(|candidate| candidate.kind == "test")
        .count();

    IrReadinessReport {
        files: program.files.len(),
        items: candidates.len(),
        tasks,
        tests,
        errors,
        warnings,
        resolve_summary,
        type_check_summary,
        core_preview_summary,
        core_lower_summary,
        core_verify_summary,
        full_type_check_summary,
        effect_check_summary,
        ownership_check_summary,
        resource_check_summary,
        profile_check_summary,
        candidates,
    }
}

fn collect_candidates_from_items(
    items: &[Item],
    context: &CandidateContext<'_>,
    candidates: &mut Vec<LoweringCandidate>,
) {
    for item in items {
        candidates.push(lowering_candidate(item, context));
        if let Item::App(app) = item {
            collect_candidates_from_items(&app.items, context, candidates);
        }
    }
}

fn lowering_candidate(item: &Item, context: &CandidateContext<'_>) -> LoweringCandidate {
    let graph_node_id = node_id::span(
        "item",
        item.span(),
        &format!("{} {}", item.kind(), item.name()),
    );
    let has_errors = context
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);
    let has_resolver_errors = context.resolve_summary.resolver_errors > 0;
    let has_type_errors = context.type_check_summary.type_errors > 0;
    let has_core_verify_errors = context.core_verify_summary.failed_checks > 0;
    let has_full_type_check_errors = context.full_type_check_summary.blocking_issues > 0;
    let has_effect_check_errors = matches!(
        context.effect_check_summary.status,
        "effect_errors_v0" | "blocked_by_unchecked_effects_v0"
    );
    let has_ownership_check_errors = matches!(
        context.ownership_check_summary.status,
        "ownership_errors_v0" | "blocked_by_unchecked_ownership_facts_v0"
    );
    let has_resource_check_errors = matches!(
        context.resource_check_summary.status,
        "resource_errors_v0" | "blocked_by_unchecked_resource_facts_v0"
    );
    let has_profile_check_errors = matches!(
        context.profile_check_summary.status,
        "profile_errors_v0" | "blocked_by_unchecked_profile_policy_v0"
    );
    let blocking_reasons = blocking_reasons(CandidateBlockers {
        has_errors,
        has_resolver_errors,
        has_type_errors,
        has_core_verify_errors,
        has_full_type_check_errors,
        has_effect_check_errors,
        has_ownership_check_errors,
        has_resource_check_errors,
        has_profile_check_errors,
    });
    let section_names = item_sections(item)
        .iter()
        .map(|section| section.name.clone())
        .collect::<Vec<_>>();
    let body_grammar = body_grammar_for_item(context.program, item);
    let (checked_add_producer_closure, checked_add_producer_blocker) = match item {
        Item::Task(task) => match canonical_checked_add_producer_closure(
            context.program,
            context.diagnostics,
            item,
            task,
        ) {
            Ok(closure) => (Some(closure), None),
            Err(reason) => (None, Some(reason)),
        },
        Item::App(_) | Item::Type(_) | Item::Store(_) | Item::Test(_) => {
            (None, Some("canonical_backend_item_kind_unsupported_v0"))
        }
    };

    LoweringCandidate {
        id: readiness_id(item),
        kind: item.kind(),
        name: item.name().to_string(),
        graph_node_id,
        span: portable_span(item.span()),
        status: if has_errors {
            "blocked_by_source_errors"
        } else if has_resolver_errors {
            "blocked_by_resolver_errors"
        } else if has_type_errors {
            "blocked_by_type_errors"
        } else if has_core_verify_errors {
            "blocked_by_core_verify_errors"
        } else if has_full_type_check_errors {
            "blocked_by_full_type_check_errors"
        } else if has_effect_check_errors {
            "blocked_by_effect_check_errors"
        } else if has_ownership_check_errors {
            "blocked_by_ownership_check_errors"
        } else if has_resource_check_errors {
            "blocked_by_resource_check_errors"
        } else if has_profile_check_errors {
            "blocked_by_profile_check_errors"
        } else {
            "blocked_before_ir_verify"
        },
        current_layer: CURRENT_LAYER,
        target_layer: TARGET_LAYER,
        facts_available: facts_available(item, context),
        missing_passes: if has_full_type_check_errors {
            MISSING_IR_PASSES.to_vec()
        } else if has_effect_check_errors {
            MISSING_AFTER_FULL_TYPE_PASSES.to_vec()
        } else if has_ownership_check_errors {
            MISSING_AFTER_EFFECT_PASSES.to_vec()
        } else if has_resource_check_errors {
            MISSING_AFTER_OWNERSHIP_PASSES.to_vec()
        } else if has_profile_check_errors {
            MISSING_AFTER_RESOURCE_PASSES.to_vec()
        } else {
            MISSING_AFTER_PROFILE_PASSES.to_vec()
        },
        blocking_reasons,
        section_names,
        body_grammar,
        checked_add_producer_closure,
        checked_add_producer_blocker,
    }
}

fn facts_available(item: &Item, context: &CandidateContext<'_>) -> Vec<&'static str> {
    let mut facts = vec![
        "source_span",
        "semantic_graph_node_id",
        "item_kind",
        "item_name",
        "resolver_summary_v0",
        context.resolve_summary.status,
        "type_check_summary_v0",
        context.type_check_summary.status,
        "trivial_return_checks_v0",
        "core_preview_summary_v0",
        context.core_preview_summary.status,
        "core_lower_summary_v0",
        context.core_lower_summary.status,
        "core_verify_summary_v0",
        context.core_verify_summary.status,
        "full_type_check_summary_v0",
        context.full_type_check_summary.status,
        "effect_check_summary_v0",
        context.effect_check_summary.status,
        "ownership_check_summary_v0",
        context.ownership_check_summary.status,
        "resource_check_summary_v0",
        context.resource_check_summary.status,
        "profile_check_summary_v0",
        context.profile_check_summary.status,
    ];
    if context.core_lower_summary.core_items > 0 {
        facts.push("unverified_core_artifact_rows_v0");
    }
    if context.core_verify_summary.verified_operations > 0 {
        facts.push("verified_core_artifact_rows_v0");
    }
    if context.full_type_check_summary.accepted_statements > 0 {
        facts.push("recognized_body_type_facts_v0");
    }
    if context.effect_check_summary.accepted_statements > 0 {
        facts.push("recognized_effect_facts_v0");
    }
    if context.ownership_check_summary.accepted_statements > 0 {
        facts.push("recognized_ownership_facts_v0");
    }
    if context.resource_check_summary.accepted_checks > 0 {
        facts.push("recognized_resource_facts_v0");
    }
    if context.profile_check_summary.accepted_checks > 0 {
        facts.push("recognized_profile_policy_facts_v0");
    }
    if has_checked_return_type_slot(item, context.checked_returns) {
        facts.push("checked_return_expression_type_slots_v0");
    }

    let sections = item_sections(item);
    if !sections.is_empty() {
        facts.push("source_sections");
    }
    if sections.iter().any(has_meaningful_lines) {
        facts.push("section_line_spans");
    }

    match item {
        Item::App(app) => add_app_facts(app, &mut facts),
        Item::Type(type_def) => add_type_facts(type_def, &mut facts),
        Item::Store(store) => add_store_facts(store, &mut facts),
        Item::Task(task) => add_task_facts(context.program, item, task, &mut facts),
        Item::Test(test) => add_test_facts(context.program, item, test, &mut facts),
    }

    facts
}

fn has_checked_return_type_slot(
    item: &Item,
    checked_returns: &[type_check::CheckedReturnSummary],
) -> bool {
    let task = match item {
        Item::Task(task) => task,
        _ => return false,
    };
    checked_returns.iter().any(|checked_return| {
        checked_return.owner_kind == "task"
            && checked_return.owner_name == task.name
            && task_contains_span(task, &checked_return.source_span)
            && checked_return.actual_type.is_some()
            && matches!(
                checked_return.status,
                "accepted_return_expression_v0" | "rejected_return_type_mismatch_v0"
            )
    })
}

fn task_contains_span(task: &Task, span: &Span) -> bool {
    task.sections
        .iter()
        .flat_map(|section| section.lines.iter())
        .any(|line| line.span == *span)
}

fn add_app_facts(app: &App, facts: &mut Vec<&'static str>) {
    if !app.items.is_empty() {
        facts.push("nested_item_scope");
    }
}

fn add_type_facts(type_def: &TypeDef, facts: &mut Vec<&'static str>) {
    if !type_def.fields.is_empty() {
        facts.push("field_shapes");
    }
}

fn add_store_facts(store: &Store, facts: &mut Vec<&'static str>) {
    if !store.ty.trim().is_empty() {
        facts.push("store_type_annotation");
    }
}

fn add_task_facts(program: &Program, item: &Item, task: &Task, facts: &mut Vec<&'static str>) {
    if !task.params.is_empty() {
        facts.push("signature_params");
    }
    if task.result.is_some() {
        facts.push("signature_result");
    }
    add_section_family_facts(program, item, &task.sections, facts);
}

fn add_test_facts(program: &Program, item: &Item, test: &Test, facts: &mut Vec<&'static str>) {
    if !test.params.is_empty() {
        facts.push("signature_params");
    }
    if !test.modifiers.is_empty() {
        facts.push("test_modifiers");
    }
    if test.section("covers").is_some() {
        facts.push("test_coverage_hints");
    }
    add_section_family_facts(program, item, &test.sections, facts);
}

fn add_section_family_facts(
    program: &Program,
    item: &Item,
    sections: &[Section],
    facts: &mut Vec<&'static str>,
) {
    if has_any_section(sections, &["uses", "changes"]) {
        facts.push("effect_hints");
    }
    if has_any_section(
        sections,
        &[
            "needs",
            "ensures",
            "keeps",
            "protects",
            "trusts",
            "watch for",
        ],
    ) {
        facts.push("contract_hints");
    }
    if has_any_section(
        sections,
        &["cost", "allocates", "avoids", "tradeoffs", "optimizes"],
    ) {
        facts.push("resource_hints");
    }
    if has_any_section(
        sections,
        &["profile", "profiles", "runtime profile", "runtime profiles"],
    ) {
        facts.push("profile_hints");
    }
    if has_any_section(sections, &["does"]) {
        facts.push("body_text_captured");
    }
    if sections
        .iter()
        .find(|section| section.name == "does")
        .map(|section| {
            core_body::analyze_does_section(
                program
                    .canonical_core_expectation(item, section)
                    .expect("live readiness item must have parser authority"),
            )
        })
        .is_some_and(|report| report.recognized_lines > 0)
    {
        facts.push("body_grammar_partial_v0");
    }
}

fn body_grammar_for_item(program: &Program, item: &Item) -> Option<BodyGrammarReport> {
    item_sections(item)
        .iter()
        .find(|section| section.name == "does")
        .map(|section| {
            core_body::analyze_does_section(
                program
                    .canonical_core_expectation(item, section)
                    .expect("live readiness body must have parser authority"),
            )
        })
}

fn has_any_section(sections: &[Section], names: &[&str]) -> bool {
    sections
        .iter()
        .any(|section| names.contains(&section.name.as_str()) && has_meaningful_lines(section))
}

fn has_meaningful_lines(section: &Section) -> bool {
    section
        .lines
        .iter()
        .any(|line| is_meaningful_line_text(&line.text))
}

fn item_sections(item: &Item) -> &[Section] {
    match item {
        Item::App(item) => &item.sections,
        Item::Type(item) => &item.sections,
        Item::Store(item) => &item.sections,
        Item::Task(item) => &item.sections,
        Item::Test(item) => &item.sections,
    }
}

fn blocking_reasons(blockers: CandidateBlockers) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if blockers.has_errors {
        reasons.push("source_diagnostics_include_errors");
    }
    if blockers.has_resolver_errors {
        reasons.push("checked_resolver_errors");
    }
    if blockers.has_type_errors {
        reasons.push("type_check_errors");
    }
    if blockers.has_core_verify_errors {
        reasons.push("core_verify_errors");
    }
    if blockers.has_full_type_check_errors {
        reasons.push("full_type_check_errors");
    }
    if blockers.has_effect_check_errors {
        reasons.push("effect_check_errors");
    }
    if blockers.has_ownership_check_errors {
        reasons.push("ownership_check_errors");
    }
    if blockers.has_resource_check_errors {
        reasons.push("resource_check_errors");
    }
    if blockers.has_profile_check_errors {
        reasons.push("profile_check_errors");
    }
    reasons.push("ir_verify_not_implemented");
    reasons
}

impl IrReadinessReport {
    fn blocked_count(&self) -> usize {
        self.candidates
            .iter()
            .filter(|candidate| candidate.status.starts_with("blocked"))
            .count()
    }

    fn body_grammar_candidates(&self) -> usize {
        self.candidates
            .iter()
            .filter(|candidate| candidate.body_grammar.is_some())
            .count()
    }

    fn body_grammar_recognized_lines(&self) -> usize {
        self.candidates
            .iter()
            .filter_map(|candidate| candidate.body_grammar.as_ref())
            .map(|report| report.recognized_lines)
            .sum()
    }

    fn body_grammar_unsupported_lines(&self) -> usize {
        self.candidates
            .iter()
            .filter_map(|candidate| candidate.body_grammar.as_ref())
            .map(|report| report.unsupported_lines)
            .sum()
    }
}

fn readiness_id(item: &Item) -> String {
    prefixed_id(
        "hum_ir_ready",
        &format!(
            "{}_{}_{}_{}",
            item.kind(),
            item.name(),
            item.span().line,
            item.span().column
        ),
    )
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

fn portable_span(span: &Span) -> Span {
    Span {
        file: span.file.replace('\\', "/"),
        line: span.line,
        column: span.column,
    }
}

fn push_resolver_summary(
    out: &mut String,
    summary: &resolve::ResolveReadinessSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"resolver\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(out, indent + 2, "files", summary.files, true);
    push_usize_field(out, indent + 2, "items", summary.items, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        summary.source_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "source_warnings",
        summary.source_warnings,
        true,
    );
    push_usize_field(out, indent + 2, "scopes", summary.scopes, true);
    push_usize_field(out, indent + 2, "definitions", summary.definitions, true);
    push_usize_field(out, indent + 2, "references", summary.references, true);
    push_usize_field(
        out,
        indent + 2,
        "resolved_references",
        summary.resolved_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unresolved_references",
        summary.unresolved_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "external_references",
        summary.external_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "duplicate_definitions",
        summary.duplicate_definitions,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "mutable_place_errors",
        summary.mutable_place_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        summary.resolver_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_warnings",
        summary.resolver_warnings,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_type_check_summary(
    out: &mut String,
    summary: &type_check::TypeCheckSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"type_check\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        summary.source_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "source_warnings",
        summary.source_warnings,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        summary.resolver_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_declarations",
        summary.checked_declarations,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_declarations",
        summary.accepted_declarations,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_declarations",
        summary.rejected_declarations,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_type_references",
        summary.checked_type_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unknown_type_references",
        summary.unknown_type_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_returns",
        summary.checked_returns,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_returns",
        summary.accepted_returns,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_returns",
        summary.rejected_returns,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_returns",
        summary.unchecked_returns,
        true,
    );
    push_usize_field(out, indent + 2, "type_errors", summary.type_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "type_warnings",
        summary.type_warnings,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_core_preview_summary(
    out: &mut String,
    summary: &core_preview::CorePreviewReadinessSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"core_preview\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_usize_field(out, indent + 2, "files", summary.files, true);
    push_usize_field(out, indent + 2, "items", summary.items, true);
    push_usize_field(out, indent + 2, "tasks", summary.tasks, true);
    push_usize_field(out, indent + 2, "tests", summary.tests, true);
    push_usize_field(
        out,
        indent + 2,
        "core_candidates",
        summary.core_candidates,
        true,
    );
    push_usize_field(out, indent + 2, "errors", summary.errors, true);
    push_usize_field(out, indent + 2, "warnings", summary.warnings, true);
    push_usize_field(
        out,
        indent + 2,
        "lowerable_preview_statements",
        summary.lowerable_preview_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "contextual_preview_statements",
        summary.contextual_preview_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocked_statements",
        summary.blocked_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "expression_previews",
        summary.expression_previews,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "expression_ast_nodes",
        summary.expression_ast_nodes,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "typed_expression_previews",
        summary.typed_expression_previews,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_core_lower_summary(
    out: &mut String,
    summary: &core_lower::CoreLowerReadinessSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"core_lower\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_usize_field(out, indent + 2, "files", summary.files, true);
    push_usize_field(out, indent + 2, "items", summary.items, true);
    push_usize_field(out, indent + 2, "tasks", summary.tasks, true);
    push_usize_field(out, indent + 2, "tests", summary.tests, true);
    push_usize_field(out, indent + 2, "core_items", summary.core_items, true);
    push_usize_field(
        out,
        indent + 2,
        "lowered_items",
        summary.lowered_items,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocked_items",
        summary.blocked_items,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "lowered_operations",
        summary.lowered_operations,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocked_operations",
        summary.blocked_operations,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "execution_ready",
        summary.execution_ready,
        true,
    );
    push_usize_field(out, indent + 2, "ir_ready", summary.ir_ready, true);
    push_usize_field(out, indent + 2, "errors", summary.errors, true);
    push_usize_field(out, indent + 2, "warnings", summary.warnings, true);
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        summary.resolver_errors,
        true,
    );
    push_usize_field(out, indent + 2, "type_errors", summary.type_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "preview_blocked_statements",
        summary.preview_blocked_statements,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}
fn push_core_verify_summary(
    out: &mut String,
    summary: &core_verify::CoreVerifyReadinessSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"core_verify\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(out, indent + 2, "files", summary.files, true);
    push_usize_field(out, indent + 2, "items", summary.items, true);
    push_usize_field(out, indent + 2, "tasks", summary.tasks, true);
    push_usize_field(out, indent + 2, "tests", summary.tests, true);
    push_usize_field(out, indent + 2, "core_items", summary.core_items, true);
    push_usize_field(
        out,
        indent + 2,
        "verified_items",
        summary.verified_items,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "lower_blocked_items",
        summary.lower_blocked_items,
        true,
    );
    push_usize_field(out, indent + 2, "operations", summary.operations, true);
    push_usize_field(
        out,
        indent + 2,
        "verified_operations",
        summary.verified_operations,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "lower_blocked_operations",
        summary.lower_blocked_operations,
        true,
    );
    push_usize_field(out, indent + 2, "checks", summary.checks, true);
    push_usize_field(
        out,
        indent + 2,
        "passed_checks",
        summary.passed_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "failed_checks",
        summary.failed_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "execution_ready",
        summary.execution_ready,
        true,
    );
    push_usize_field(out, indent + 2, "ir_ready", summary.ir_ready, true);
    push_usize_field(out, indent + 2, "errors", summary.errors, true);
    push_usize_field(out, indent + 2, "warnings", summary.warnings, true);
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        summary.resolver_errors,
        true,
    );
    push_usize_field(out, indent + 2, "type_errors", summary.type_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "preview_blocked_statements",
        summary.preview_blocked_statements,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}
fn push_full_type_check_summary(
    out: &mut String,
    summary: &full_type_check::FullTypeCheckSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"full_type_check\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        summary.source_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        summary.resolver_errors,
        true,
    );
    push_usize_field(out, indent + 2, "type_errors", summary.type_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "core_verify_errors",
        summary.core_verify_errors,
        true,
    );
    push_usize_field(out, indent + 2, "items", summary.items, true);
    push_usize_field(out, indent + 2, "body_items", summary.body_items, true);
    push_usize_field(out, indent + 2, "statements", summary.statements, true);
    push_usize_field(
        out,
        indent + 2,
        "checked_statements",
        summary.checked_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_statements",
        summary.accepted_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_statements",
        summary.rejected_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_statements",
        summary.unchecked_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unsupported_statements",
        summary.unsupported_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocking_issues",
        summary.blocking_issues,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "execution_ready",
        summary.execution_ready,
        true,
    );
    push_usize_field(out, indent + 2, "ir_ready", summary.ir_ready, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}
fn push_summary(out: &mut String, report: &IrReadinessReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"lowering_candidates\": {}, \"ready_for_ir\": 0, \"blocked\": {}, \"errors\": {}, \"warnings\": {}, \"type_errors\": {}, \"unknown_type_references\": {}, \"checked_returns\": {}, \"rejected_returns\": {}, \"unchecked_returns\": {}, \"body_grammar_candidates\": {}, \"body_grammar_recognized_lines\": {}, \"body_grammar_unsupported_lines\": {}",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.candidates.len(),
        report.blocked_count(),
        report.errors,
        report.warnings,
        report.type_check_summary.type_errors,
        report.type_check_summary.unknown_type_references,
        report.type_check_summary.checked_returns,
        report.type_check_summary.rejected_returns,
        report.type_check_summary.unchecked_returns,
        report.body_grammar_candidates(),
        report.body_grammar_recognized_lines(),
        report.body_grammar_unsupported_lines()
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_effect_check_summary(
    out: &mut String,
    summary: &effect_check::EffectCheckSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"effect_check\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        summary.source_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        summary.resolver_errors,
        true,
    );
    push_usize_field(out, indent + 2, "type_errors", summary.type_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "core_verify_errors",
        summary.core_verify_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "full_type_check_errors",
        summary.full_type_check_errors,
        true,
    );
    push_usize_field(out, indent + 2, "items", summary.items, true);
    push_usize_field(out, indent + 2, "effect_items", summary.effect_items, true);
    push_usize_field(out, indent + 2, "statements", summary.statements, true);
    push_usize_field(
        out,
        indent + 2,
        "checked_statements",
        summary.checked_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_statements",
        summary.accepted_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_statements",
        summary.rejected_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_statements",
        summary.unchecked_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "boundary_checks",
        summary.boundary_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_boundary_checks",
        summary.rejected_boundary_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocking_issues",
        summary.blocking_issues,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "execution_ready",
        summary.execution_ready,
        true,
    );
    push_usize_field(out, indent + 2, "ir_ready", summary.ir_ready, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}
fn push_ownership_check_summary(
    out: &mut String,
    summary: &ownership_check::OwnershipCheckSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"ownership_check\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        summary.source_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        summary.resolver_errors,
        true,
    );
    push_usize_field(out, indent + 2, "type_errors", summary.type_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "core_verify_errors",
        summary.core_verify_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "full_type_check_errors",
        summary.full_type_check_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "effect_check_errors",
        summary.effect_check_errors,
        true,
    );
    push_usize_field(out, indent + 2, "items", summary.items, true);
    push_usize_field(
        out,
        indent + 2,
        "ownership_items",
        summary.ownership_items,
        true,
    );
    push_usize_field(out, indent + 2, "statements", summary.statements, true);
    push_usize_field(
        out,
        indent + 2,
        "checked_statements",
        summary.checked_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_statements",
        summary.accepted_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_statements",
        summary.rejected_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_statements",
        summary.unchecked_statements,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "boundary_checks",
        summary.boundary_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_boundary_checks",
        summary.rejected_boundary_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocking_issues",
        summary.blocking_issues,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "execution_ready",
        summary.execution_ready,
        true,
    );
    push_usize_field(out, indent + 2, "ir_ready", summary.ir_ready, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}
fn push_resource_check_summary(
    out: &mut String,
    summary: &resource_check::ResourceCheckSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"resource_check\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        summary.source_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "ownership_errors",
        summary.ownership_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resource_report_errors",
        summary.resource_report_errors,
        true,
    );
    push_usize_field(out, indent + 2, "tasks", summary.tasks, true);
    push_usize_field(
        out,
        indent + 2,
        "resource_items",
        summary.resource_items,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resource_claims",
        summary.resource_claims,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "allocation_claims",
        summary.allocation_claims,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "allocation_free_claims",
        summary.allocation_free_claims,
        true,
    );
    push_usize_field(out, indent + 2, "checks", summary.checks, true);
    push_usize_field(
        out,
        indent + 2,
        "accepted_checks",
        summary.accepted_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_checks",
        summary.rejected_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_checks",
        summary.unchecked_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocking_issues",
        summary.blocking_issues,
        true,
    );
    push_usize_field(out, indent + 2, "proof_ready", summary.proof_ready, true);
    push_usize_field(
        out,
        indent + 2,
        "execution_ready",
        summary.execution_ready,
        true,
    );
    push_usize_field(out, indent + 2, "ir_ready", summary.ir_ready, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_profile_check_summary(
    out: &mut String,
    summary: &profile_check::ProfileCheckSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"profile_check\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        summary.source_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resource_check_errors",
        summary.resource_check_errors,
        true,
    );
    push_usize_field(out, indent + 2, "tasks", summary.tasks, true);
    push_usize_field(
        out,
        indent + 2,
        "profile_items",
        summary.profile_items,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "declared_profiles",
        summary.declared_profiles,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "default_profiles",
        summary.default_profiles,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "known_profiles",
        summary.known_profiles,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unknown_profiles",
        summary.unknown_profiles,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "strict_profiles",
        summary.strict_profiles,
        true,
    );
    push_usize_field(out, indent + 2, "checks", summary.checks, true);
    push_usize_field(
        out,
        indent + 2,
        "accepted_checks",
        summary.accepted_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_checks",
        summary.rejected_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_checks",
        summary.unchecked_checks,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocking_issues",
        summary.blocking_issues,
        true,
    );
    push_usize_field(out, indent + 2, "proof_ready", summary.proof_ready, true);
    push_usize_field(
        out,
        indent + 2,
        "execution_ready",
        summary.execution_ready,
        true,
    );
    push_usize_field(out, indent + 2, "ir_ready", summary.ir_ready, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}
fn push_pass_status(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"pass_status\": [\n");
    for (index, pass) in PASS_STATUSES.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "name", pass.name, true);
        push_string_field(out, indent + 4, "status", pass.status, true);
        push_string_field(out, indent + 4, "source", pass.source, false);
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_candidates(out: &mut String, candidates: &[LoweringCandidate], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"lowering_candidates\": [\n");
    for (index, candidate) in candidates.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_candidate(out, candidate, indent + 2);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_candidate(out: &mut String, candidate: &LoweringCandidate, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &candidate.id, true);
    push_string_field(out, indent + 2, "kind", candidate.kind, true);
    push_string_field(out, indent + 2, "name", &candidate.name, true);
    push_string_field(
        out,
        indent + 2,
        "graph_node_id",
        &candidate.graph_node_id,
        true,
    );
    push_span_field(out, indent + 2, "source_span", &candidate.span, true);
    push_string_field(out, indent + 2, "status", candidate.status, true);
    push_string_field(
        out,
        indent + 2,
        "current_layer",
        candidate.current_layer,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "target_layer",
        candidate.target_layer,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "facts_available",
        &candidate.facts_available,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "missing_passes",
        &candidate.missing_passes,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "blocking_reasons",
        &candidate.blocking_reasons,
        true,
    );
    if let Some(body_grammar) = &candidate.body_grammar {
        push_body_grammar(out, indent + 2, body_grammar, true);
    }
    if let Some(closure) = &candidate.checked_add_producer_closure {
        push_checked_add_producer_closure(out, indent + 2, closure, true);
    } else {
        push_string_field(
            out,
            indent + 2,
            "checked_add_producer_blocker",
            candidate
                .checked_add_producer_blocker
                .unwrap_or("canonical_backend_not_applicable_v0"),
            true,
        );
    }
    push_owned_string_array(
        out,
        indent + 2,
        "source_sections",
        &candidate.section_names,
        false,
    );
    push_indent(out, indent);
    out.push('}');
}

fn push_checked_add_producer_closure(
    out: &mut String,
    indent: usize,
    closure: &CanonicalCheckedAddProducerClosure,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"checked_add_producer_closure\": {\n");
    push_string_field(out, indent + 2, "status", closure.status, true);
    push_string_field(
        out,
        indent + 2,
        "source_revision",
        &closure.source_revision,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "normalized_path",
        &closure.normalized_path,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "module_token_identity",
        &closure.module_token_identity,
        true,
    );
    push_parsed_range_field(out, indent + 2, "module_range", &closure.module_range, true);
    push_usize_array(out, indent + 2, "item_path", &closure.item_path, true);
    push_string_field(out, indent + 2, "item_kind", closure.item_kind, true);
    push_owned_string_array(
        out,
        indent + 2,
        "parameter_type_token_identities",
        &closure.parameter_type_token_identities,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "parameter_type_names",
        &closure.parameter_type_names,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "parameter_permissions",
        &closure.parameter_permissions,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "result_type_token_identity",
        &closure.result_type_token_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "result_type_name",
        &closure.result_type_name,
        true,
    );
    push_parsed_range_field(
        out,
        indent + 2,
        "result_type_range",
        &closure.result_type_range,
        true,
    );
    push_bool_field(
        out,
        indent + 2,
        "result_type_explicit",
        closure.result_type_explicit,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "semantic_file_index",
        closure.semantic_file_index,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "module_identity",
        &closure.module_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "module_display_name",
        &closure.module_display_name,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "function_identity",
        &closure.function_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "function_display_name",
        &closure.function_display_name,
        true,
    );
    push_parsed_range_field(
        out,
        indent + 2,
        "function_range",
        &closure.function_range,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "linkage_identity",
        &closure.linkage_identity,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "parameter_identities",
        &closure.parameter_identities,
        true,
    );
    push_usize_array(
        out,
        indent + 2,
        "parameter_ordinals",
        &closure.parameter_ordinals,
        true,
    );
    push_parsed_range_array(
        out,
        indent + 2,
        "parameter_ranges",
        &closure.parameter_ranges,
        true,
    );
    push_parsed_range_array(
        out,
        indent + 2,
        "parameter_type_ranges",
        &closure.parameter_type_ranges,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "operation_kinds",
        &closure.operation_kinds,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "parameter_names",
        &closure.parameter_names,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "does_section_slot",
        closure.does_section_slot,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "does_section_identity",
        &closure.does_section_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "does_section_name",
        &closure.does_section_name,
        true,
    );
    push_parsed_range_field(
        out,
        indent + 2,
        "does_section_range",
        &closure.does_section_range,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "statement_count",
        closure.statement_count,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "statement_node_identity",
        &closure.statement_node_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "statement_kind",
        closure.statement_kind,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "block_relationship",
        closure.block_relationship,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "block_depth_before",
        closure.block_depth_before,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "block_depth_after",
        closure.block_depth_after,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "block_identity",
        &closure.block_identity,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "operation_identities",
        &closure.operation_identities,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "child_node_identities",
        &closure.child_node_identities,
        true,
    );
    push_string_field(out, indent + 2, "add_kind", closure.add_kind, true);
    push_string_field(out, indent + 2, "add_operator", closure.add_operator, true);
    push_string_field(
        out,
        indent + 2,
        "add_completion",
        closure.add_completion,
        true,
    );
    push_string_array(out, indent + 2, "child_kinds", &closure.child_kinds, true);
    push_string_array(
        out,
        indent + 2,
        "child_completions",
        &closure.child_completions,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "ordered_child_relationship",
        &closure.ordered_child_relationship,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "operand_value_identities",
        &closure.operand_value_identities,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "result_value_identity",
        &closure.result_value_identity,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "resolver_use_identities",
        &closure.resolver_use_identities,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "resolver_definition_identities",
        &closure.resolver_definition_identities,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "resolver_use_order_identity",
        &closure.resolver_use_order_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "resolver_definition_order_identity",
        &closure.resolver_definition_order_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "resolver_distinct_binding_status",
        closure.resolver_distinct_binding_status,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "parameter_type_identities",
        &closure.parameter_type_identities,
        true,
    );
    push_owned_string_array(
        out,
        indent + 2,
        "operand_type_identities",
        &closure.operand_type_identities,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "add_result_type_identity",
        &closure.add_result_type_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "function_result_type_identity",
        &closure.function_result_type_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "effect_identity",
        &closure.effect_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "authority_identity",
        &closure.authority_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "ownership_identity",
        &closure.ownership_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "resource_identity",
        &closure.resource_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "profile_identity",
        &closure.profile_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "overflow_edge_identity",
        &closure.overflow_edge_identity,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "overflow_status",
        closure.overflow_status,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "accepted_passes",
        &closure.accepted_passes,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "unsupported_facts",
        &closure.unsupported_facts,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "missing_artifact",
        closure.missing_artifact,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "missing_verifier",
        closure.missing_verifier,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_body_grammar(out: &mut String, indent: usize, report: &BodyGrammarReport, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"body_grammar\": {\n");
    push_string_field(out, indent + 2, "status", report.status, true);
    push_string_field(
        out,
        indent + 2,
        "grammar_status",
        report.grammar_status,
        true,
    );
    push_usize_field(out, indent + 2, "total_lines", report.total_lines, true);
    push_usize_field(
        out,
        indent + 2,
        "meaningful_lines",
        report.meaningful_lines,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "recognized_lines",
        report.recognized_lines,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unsupported_lines",
        report.unsupported_lines,
        true,
    );
    push_body_statements(out, indent + 2, &report.statements, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_body_statements(
    out: &mut String,
    indent: usize,
    statements: &[BodyStatement],
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"statements\": [\n");
    for (index, statement) in statements.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_body_statement(out, indent + 2, statement);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_body_statement(out: &mut String, indent: usize, statement: &BodyStatement) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_span_field(out, indent + 2, "source_span", &statement.span, true);
    push_string_field(out, indent + 2, "text", &statement.text, true);
    push_string_field(out, indent + 2, "kind", statement.kind, true);
    push_string_field(out, indent + 2, "status", statement.status, true);
    push_optional_string_field(
        out,
        indent + 2,
        "expression_kind",
        statement.expression_kind,
        true,
    );
    push_optional_string_field(out, indent + 2, "reason", statement.reason, false);
    push_indent(out, indent);
    out.push('}');
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

fn push_parsed_range_field(
    out: &mut String,
    indent: usize,
    key: &str,
    range: &crate::ast::ParsedSourceRange,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": {\"file\": ");
    push_json_string(out, &range.start.file);
    out.push_str(&format!(
        ", \"line\": {}, \"column\": {}, \"byte_len\": {}}}",
        range.start.line, range.start.column, range.byte_len
    ));
    push_comma_newline(out, comma);
}

fn push_parsed_range_array(
    out: &mut String,
    indent: usize,
    key: &str,
    ranges: &[crate::ast::ParsedSourceRange],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    for (index, range) in ranges.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str("{\"file\": ");
        push_json_string(out, &range.start.file);
        out.push_str(&format!(
            ", \"line\": {}, \"column\": {}, \"byte_len\": {}}}",
            range.start.line, range.start.column, range.byte_len
        ));
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_usize_array(out: &mut String, indent: usize, key: &str, values: &[usize], comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&value.to_string());
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_bool_field(out: &mut String, indent: usize, key: &str, value: bool, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(if value { ": true" } else { ": false" });
    push_comma_newline(out, comma);
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

fn push_owned_string_array(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[String],
    comma: bool,
) {
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
    out.push_str(&format!(": {value}"));
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
    use crate::ast::{Item, Program};
    use crate::parser::{parse_source, parse_source_at_index};

    use super::{
        C1CorruptionSubcase, C1ProducerCorruption, ir_readiness_json, ir_readiness_text,
        with_c1_producer_corruption, with_c1_producer_corruption_subcase,
    };

    type ProjectionMutation = (&'static str, Box<dyn FnOnce(&mut Program)>);

    fn check_c1_issuance_source_layout(
        parser_source: &str,
        other_sources: &[(&str, &str)],
    ) -> Result<(), &'static str> {
        if parser_source.contains("pub(crate) fn from_selected")
            || parser_source.contains("pub fn from_selected")
        {
            return Err("canonical_backend_issuance_mint_exposed_v0");
        }
        if parser_source.contains("fn c1_decoy")
            || parser_source.contains("fn decoy_canonical_backend_signature")
        {
            return Err("canonical_backend_issuance_decoy_present_v0");
        }
        if other_sources.iter().any(|(_, source)| {
            source.contains("CanonicalBackendSignatureCapability::from_selected(")
                || source.contains(".select_and_issue_canonical_backend_signature(")
        }) {
            return Err("canonical_backend_issuance_alternate_file_v0");
        }
        let selector_start = parser_source
            .find("fn select_and_issue_canonical_backend_signature(")
            .ok_or("canonical_backend_issuance_selector_absent_v0")?;
        let selector_end = parser_source[selector_start..]
            .find("\n    fn parse_test_header(")
            .map(|offset| selector_start + offset)
            .ok_or("canonical_backend_issuance_selector_boundary_absent_v0")?;
        let constructor_expression = "CanonicalBackendSignatureCapability::from_selected(binding)";
        let constructor_positions = parser_source
            .match_indices(constructor_expression)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if constructor_positions
            .iter()
            .any(|position| !(selector_start..selector_end).contains(position))
        {
            return Err("canonical_backend_issuance_elsewhere_v0");
        }
        if constructor_positions.len() != 1 {
            return Err("canonical_backend_issuance_duplicate_v0");
        }
        let selector = &parser_source[selector_start..selector_end];
        if selector.contains(".or_else(")
            || selector.contains(".unwrap_or(")
            || selector.contains(".unwrap_or_else(")
            || selector.contains(".unwrap_or_default(")
        {
            return Err("canonical_backend_issuance_post_selection_fallback_v0");
        }
        let sections_at = parser_source
            .find("let sections =")
            .ok_or("canonical_backend_issuance_sections_absent_v0")?;
        let installation_at = parser_source
            .find("let signature = self.select_and_issue_canonical_backend_signature(")
            .ok_or("canonical_backend_issuance_installation_absent_v0")?;
        let task_constructor_at = parser_source
            .find("Item::Task(Task::parser_new(")
            .ok_or("canonical_backend_issuance_task_constructor_absent_v0")?;
        if installation_at < sections_at {
            return Err("canonical_backend_issuance_pre_selection_v0");
        }
        if installation_at > task_constructor_at {
            return Err("canonical_backend_issuance_post_constructor_v0");
        }
        if parser_source
            .matches(".select_and_issue_canonical_backend_signature(")
            .count()
            != 1
        {
            return Err("canonical_backend_issuance_elsewhere_v0");
        }
        let installation = &parser_source[installation_at..task_constructor_at + 600];
        if !installation.contains("signature,") {
            return Err("canonical_backend_issuance_foreign_installation_route_v0");
        }
        Ok(())
    }

    #[derive(Debug, Clone, Copy)]
    struct C1FactSpec {
        id: &'static str,
        owner: &'static str,
        corruption: C1ProducerCorruption,
        expected_blocker: &'static str,
        dormant_observations: usize,
    }

    macro_rules! c1_fact {
        ($id:literal, $owner:literal, $corruption:expr, $blocker:expr) => {
            C1FactSpec {
                id: $id,
                owner: $owner,
                corruption: $corruption,
                expected_blocker: $blocker,
                dormant_observations: 34,
            }
        };
    }

    const PARSER_BLOCKER: &str = "canonical_backend_signature_authority_mismatch_v0";
    const CORE_BLOCKER: &str = "canonical_backend_core_producer_corruption_v0";
    const CORE_JOIN_BLOCKER: &str = "canonical_backend_core_authority_join_mismatch_v0";
    const RESOLVER_BLOCKER: &str = "canonical_backend_resolver_producer_corruption_v0";
    const TYPE_BLOCKER: &str = "canonical_backend_type_producer_corruption_v0";
    const EFFECT_BLOCKER: &str = "canonical_backend_effect_producer_corruption_v0";
    const OWNERSHIP_BLOCKER: &str = "canonical_backend_ownership_producer_corruption_v0";
    const RESOURCE_BLOCKER: &str = "canonical_backend_resource_producer_corruption_v0";
    const PROFILE_BLOCKER: &str = "canonical_backend_profile_producer_corruption_v0";
    const JOIN_BLOCKER: &str = "canonical_backend_pass_or_blocker_join_mismatch_v0";
    const PRIMARY_SUBCASE: [C1CorruptionSubcase; 1] = [C1CorruptionSubcase::Primary];
    const RANGE_SUBCASES: [C1CorruptionSubcase; 4] = [
        C1CorruptionSubcase::RangeFile,
        C1CorruptionSubcase::RangeLine,
        C1CorruptionSubcase::RangeColumn,
        C1CorruptionSubcase::RangeByteLength,
    ];

    fn c1_fact_subcases(corruption: C1ProducerCorruption) -> &'static [C1CorruptionSubcase] {
        match corruption {
            C1ProducerCorruption::ModuleTokenRange
            | C1ProducerCorruption::FunctionTokenRange
            | C1ProducerCorruption::ParameterBinderRange(_)
            | C1ProducerCorruption::ParameterTypeRange(_)
            | C1ProducerCorruption::ResultTypeRange
            | C1ProducerCorruption::DoesSectionRange => &RANGE_SUBCASES,
            _ => &PRIMARY_SUBCASE,
        }
    }

    const C1_FACT_CATALOGUE: [C1FactSpec; 111] = [
        c1_fact!(
            "parser.selector",
            "parser",
            C1ProducerCorruption::SelectorDecision,
            "canonical_backend_signature_capability_absent_v0"
        ),
        c1_fact!(
            "parser.source_revision",
            "parser",
            C1ProducerCorruption::SourceRevision,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.normalized_path",
            "parser",
            C1ProducerCorruption::NormalizedPath,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.semantic_file_ordinal",
            "parser",
            C1ProducerCorruption::SemanticFileOrdinal,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.module.identity",
            "parser",
            C1ProducerCorruption::ModuleTokenIdentity,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.module.spelling",
            "parser",
            C1ProducerCorruption::ModuleTokenSpelling,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.module.range",
            "parser",
            C1ProducerCorruption::ModuleTokenRange,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.item.path",
            "parser",
            C1ProducerCorruption::ItemPath,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.item.kind",
            "parser",
            C1ProducerCorruption::ItemKind,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.function.identity",
            "parser",
            C1ProducerCorruption::FunctionTokenIdentity,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.function.spelling",
            "parser",
            C1ProducerCorruption::FunctionTokenSpelling,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.function.range",
            "parser",
            C1ProducerCorruption::FunctionTokenRange,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.linkage",
            "parser",
            C1ProducerCorruption::LinkageIdentity,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.0.ordinal",
            "parser",
            C1ProducerCorruption::ParameterOrdinal(0),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.0.binder.identity",
            "parser",
            C1ProducerCorruption::ParameterBinderIdentity(0),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.0.binder.spelling",
            "parser",
            C1ProducerCorruption::ParameterBinderSpelling(0),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.0.binder.range",
            "parser",
            C1ProducerCorruption::ParameterBinderRange(0),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.0.type.identity",
            "parser",
            C1ProducerCorruption::ParameterTypeTokenIdentity(0),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.0.type.spelling",
            "parser",
            C1ProducerCorruption::ParameterTypeSpelling(0),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.0.type.range",
            "parser",
            C1ProducerCorruption::ParameterTypeRange(0),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.0.permission",
            "parser",
            C1ProducerCorruption::ParameterPermission(0),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.1.ordinal",
            "parser",
            C1ProducerCorruption::ParameterOrdinal(1),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.1.binder.identity",
            "parser",
            C1ProducerCorruption::ParameterBinderIdentity(1),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.1.binder.spelling",
            "parser",
            C1ProducerCorruption::ParameterBinderSpelling(1),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.1.binder.range",
            "parser",
            C1ProducerCorruption::ParameterBinderRange(1),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.1.type.identity",
            "parser",
            C1ProducerCorruption::ParameterTypeTokenIdentity(1),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.1.type.spelling",
            "parser",
            C1ProducerCorruption::ParameterTypeSpelling(1),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.1.type.range",
            "parser",
            C1ProducerCorruption::ParameterTypeRange(1),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.parameter.1.permission",
            "parser",
            C1ProducerCorruption::ParameterPermission(1),
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.result.identity",
            "parser",
            C1ProducerCorruption::ResultTypeIdentity,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.result.spelling",
            "parser",
            C1ProducerCorruption::ResultTypeSpelling,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.result.range",
            "parser",
            C1ProducerCorruption::ResultTypeRange,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.result.explicit",
            "parser",
            C1ProducerCorruption::ResultTypeExplicitness,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.does.slot",
            "parser",
            C1ProducerCorruption::DoesSectionSlot,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.does.identity",
            "parser",
            C1ProducerCorruption::DoesSectionIdentity,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.does.spelling",
            "parser",
            C1ProducerCorruption::DoesSectionSpelling,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "parser.does.range",
            "parser",
            C1ProducerCorruption::DoesSectionRange,
            PARSER_BLOCKER
        ),
        c1_fact!(
            "core.statement.count",
            "core",
            C1ProducerCorruption::StatementCount,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.statement.node",
            "core",
            C1ProducerCorruption::StatementNodeIdentity,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.statement.kind",
            "core",
            C1ProducerCorruption::StatementKind,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.block.relationship",
            "core",
            C1ProducerCorruption::BlockRelationship,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.block.depth_before",
            "core",
            C1ProducerCorruption::BlockDepthBefore,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.block.depth_after",
            "core",
            C1ProducerCorruption::BlockDepthAfter,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.add.node",
            "core",
            C1ProducerCorruption::AddNodeIdentity,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.add.kind",
            "core",
            C1ProducerCorruption::AddKind,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.add.operator",
            "core",
            C1ProducerCorruption::AddOperator,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.add.completion",
            "core",
            C1ProducerCorruption::AddCompletion,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.left.node",
            "core",
            C1ProducerCorruption::LeftNodeIdentity,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.left.kind",
            "core",
            C1ProducerCorruption::LeftKind,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.left.completion",
            "core",
            C1ProducerCorruption::LeftCompletion,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.right.node",
            "core",
            C1ProducerCorruption::RightNodeIdentity,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.right.kind",
            "core",
            C1ProducerCorruption::RightKind,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.right.completion",
            "core",
            C1ProducerCorruption::RightCompletion,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.children.ordered",
            "core",
            C1ProducerCorruption::OrderedChildRelationship,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.block.identity",
            "core",
            C1ProducerCorruption::BlockIdentity,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.return.identity",
            "core",
            C1ProducerCorruption::ReturnOperationIdentity,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.operation.0.kind",
            "core",
            C1ProducerCorruption::OperationKind(0),
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.operation.1.kind",
            "core",
            C1ProducerCorruption::OperationKind(1),
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.value.0.identity",
            "core",
            C1ProducerCorruption::OperandValueIdentity(0),
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.value.1.identity",
            "core",
            C1ProducerCorruption::OperandValueIdentity(1),
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.value.result.identity",
            "core",
            C1ProducerCorruption::ResultValueIdentity,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.overflow.edge",
            "core",
            C1ProducerCorruption::OverflowEdgeIdentity,
            CORE_BLOCKER
        ),
        c1_fact!(
            "core.overflow.status",
            "core",
            C1ProducerCorruption::OverflowStatus,
            CORE_BLOCKER
        ),
        c1_fact!(
            "resolver.function",
            "resolver",
            C1ProducerCorruption::ResolverFunctionIdentity,
            RESOLVER_BLOCKER
        ),
        c1_fact!(
            "resolver.use.0",
            "resolver",
            C1ProducerCorruption::ResolverUseIdentity(0),
            RESOLVER_BLOCKER
        ),
        c1_fact!(
            "resolver.use.1",
            "resolver",
            C1ProducerCorruption::ResolverUseIdentity(1),
            RESOLVER_BLOCKER
        ),
        c1_fact!(
            "resolver.target.0",
            "resolver",
            C1ProducerCorruption::ResolverDefinitionIdentity(0),
            RESOLVER_BLOCKER
        ),
        c1_fact!(
            "resolver.target.1",
            "resolver",
            C1ProducerCorruption::ResolverDefinitionIdentity(1),
            RESOLVER_BLOCKER
        ),
        c1_fact!(
            "resolver.use_order",
            "resolver",
            C1ProducerCorruption::ResolverUseOrder,
            RESOLVER_BLOCKER
        ),
        c1_fact!(
            "resolver.definition_order",
            "resolver",
            C1ProducerCorruption::ResolverDefinitionOrder,
            RESOLVER_BLOCKER
        ),
        c1_fact!(
            "resolver.distinct",
            "resolver",
            C1ProducerCorruption::ResolverDistinctBinding,
            RESOLVER_BLOCKER
        ),
        c1_fact!(
            "type.function",
            "full_type",
            C1ProducerCorruption::TypeFunctionIdentity,
            TYPE_BLOCKER
        ),
        c1_fact!(
            "type.parameter.0",
            "full_type",
            C1ProducerCorruption::ParameterTypeIdentity(0),
            TYPE_BLOCKER
        ),
        c1_fact!(
            "type.parameter.1",
            "full_type",
            C1ProducerCorruption::ParameterTypeIdentity(1),
            TYPE_BLOCKER
        ),
        c1_fact!(
            "type.operand.0",
            "full_type",
            C1ProducerCorruption::OperandTypeIdentity(0),
            TYPE_BLOCKER
        ),
        c1_fact!(
            "type.operand.1",
            "full_type",
            C1ProducerCorruption::OperandTypeIdentity(1),
            TYPE_BLOCKER
        ),
        c1_fact!(
            "type.add_result",
            "full_type",
            C1ProducerCorruption::AddResultTypeIdentity,
            TYPE_BLOCKER
        ),
        c1_fact!(
            "type.function_result",
            "full_type",
            C1ProducerCorruption::FunctionResultTypeIdentity,
            TYPE_BLOCKER
        ),
        c1_fact!(
            "type.status",
            "full_type",
            C1ProducerCorruption::TypeAcceptedStatus,
            TYPE_BLOCKER
        ),
        c1_fact!(
            "effect.function",
            "effect",
            C1ProducerCorruption::EffectFunctionIdentity,
            EFFECT_BLOCKER
        ),
        c1_fact!(
            "effect.effect",
            "effect",
            C1ProducerCorruption::EffectIdentity,
            EFFECT_BLOCKER
        ),
        c1_fact!(
            "effect.authority",
            "effect",
            C1ProducerCorruption::AuthorityIdentity,
            EFFECT_BLOCKER
        ),
        c1_fact!(
            "effect.status",
            "effect",
            C1ProducerCorruption::EffectAcceptedStatus,
            EFFECT_BLOCKER
        ),
        c1_fact!(
            "ownership.function",
            "ownership",
            C1ProducerCorruption::OwnershipFunctionIdentity,
            OWNERSHIP_BLOCKER
        ),
        c1_fact!(
            "ownership.identity",
            "ownership",
            C1ProducerCorruption::OwnershipIdentity,
            OWNERSHIP_BLOCKER
        ),
        c1_fact!(
            "ownership.status",
            "ownership",
            C1ProducerCorruption::OwnershipAcceptedStatus,
            OWNERSHIP_BLOCKER
        ),
        c1_fact!(
            "resource.function",
            "resource",
            C1ProducerCorruption::ResourceFunctionIdentity,
            RESOURCE_BLOCKER
        ),
        c1_fact!(
            "resource.identity",
            "resource",
            C1ProducerCorruption::ResourceIdentity,
            RESOURCE_BLOCKER
        ),
        c1_fact!(
            "resource.allocation_status",
            "resource",
            C1ProducerCorruption::ResourceAllocationStatus,
            RESOURCE_BLOCKER
        ),
        c1_fact!(
            "resource.status",
            "resource",
            C1ProducerCorruption::ResourceProducerStatus,
            RESOURCE_BLOCKER
        ),
        c1_fact!(
            "profile.function",
            "profile",
            C1ProducerCorruption::ProfileFunctionIdentity,
            PROFILE_BLOCKER
        ),
        c1_fact!(
            "profile.identity",
            "profile",
            C1ProducerCorruption::ProfileIdentity,
            PROFILE_BLOCKER
        ),
        c1_fact!(
            "profile.policy_status",
            "profile",
            C1ProducerCorruption::ProfilePolicyStatus,
            PROFILE_BLOCKER
        ),
        c1_fact!(
            "profile.status",
            "profile",
            C1ProducerCorruption::ProfileProducerStatus,
            PROFILE_BLOCKER
        ),
        c1_fact!(
            "join.pass.0",
            "join",
            C1ProducerCorruption::AcceptedPass(0),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.1",
            "join",
            C1ProducerCorruption::AcceptedPass(1),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.2",
            "join",
            C1ProducerCorruption::AcceptedPass(2),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.3",
            "join",
            C1ProducerCorruption::AcceptedPass(3),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.4",
            "join",
            C1ProducerCorruption::AcceptedPass(4),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.5",
            "join",
            C1ProducerCorruption::AcceptedPass(5),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.6",
            "join",
            C1ProducerCorruption::AcceptedPass(6),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.7",
            "join",
            C1ProducerCorruption::AcceptedPass(7),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.8",
            "join",
            C1ProducerCorruption::AcceptedPass(8),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.9",
            "join",
            C1ProducerCorruption::AcceptedPass(9),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.10",
            "join",
            C1ProducerCorruption::AcceptedPass(10),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.11",
            "join",
            C1ProducerCorruption::AcceptedPass(11),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.pass.12",
            "join",
            C1ProducerCorruption::AcceptedPass(12),
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.unsupported",
            "join",
            C1ProducerCorruption::UnsupportedFact,
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.missing_artifact",
            "join",
            C1ProducerCorruption::MissingArtifact,
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.missing_verifier",
            "join",
            C1ProducerCorruption::MissingVerifier,
            JOIN_BLOCKER
        ),
        c1_fact!(
            "join.status",
            "join",
            C1ProducerCorruption::ClosureStatus,
            JOIN_BLOCKER
        ),
    ];

    fn candidate_for(
        program: &Program,
        diagnostics: &[crate::diagnostic::Diagnostic],
        file_index: usize,
        item_index: usize,
    ) -> super::CanonicalCheckedAddProducerCandidate {
        let item = &program.files[file_index].items[item_index];
        let Item::Task(task) = item else {
            panic!("C1 oracle requires a task");
        };
        let closure =
            super::canonical_checked_add_producer_closure(program, diagnostics, item, task)
                .expect("foreign same-shaped task must independently produce C1 facts");
        (*closure).clone()
    }

    fn minimal_program() -> Program {
        let parsed = parse_source(
            "examples/core/minimal_add.hum",
            include_str!("../examples/core/minimal_add.hum"),
        );
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        Program {
            files: vec![parsed.file],
        }
    }

    fn backend_boundary_result(program: &Program) -> Result<(), &'static str> {
        let task = program.files[0]
            .items
            .iter()
            .find_map(|item| match item {
                Item::Task(task) => Some(task),
                _ => None,
            })
            .ok_or("test_task_absent")?;
        program
            .canonical_backend_function_expectation(task)?
            .validate()
            .map(|_| ())
    }

    const C1_DORMANT_OBSERVATION_IDS: [&str; 34] = [
        "resolve.human",
        "resolve.json",
        "type_env.human",
        "type_env.json",
        "type_check.human",
        "type_check.json",
        "full_type.human",
        "full_type.json",
        "effect.human",
        "effect.json",
        "ownership.human",
        "ownership.json",
        "resource.human",
        "resource.json",
        "profile.human",
        "profile.json",
        "core_preview.human",
        "core_preview.json",
        "core_lower.human",
        "core_lower.json",
        "core_verify.human",
        "core_verify.json",
        "graph.program_json",
        "runtime.outcome",
        "runtime.diagnostics",
        "runtime.authority_events",
        "runtime.rendered_diagnostics",
        "source_diagnostics.values",
        "source_diagnostics.human",
        "source_diagnostics.json",
        "catalogue.human",
        "catalogue.json",
        "explain.human",
        "explain.json",
    ];

    fn check_c1_dormant_observation_ids(ids: &[&str]) -> Result<(), &'static str> {
        if ids.is_empty() {
            return Err("canonical_backend_dormant_observation_inventory_zero_v0");
        }
        if ids.iter().any(|id| id.starts_with("ir_readiness.")) {
            return Err("canonical_backend_dormant_observation_readiness_alias_v0");
        }
        if ids
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != ids.len()
        {
            return Err("canonical_backend_dormant_observation_duplicate_or_alias_v0");
        }
        if ids != C1_DORMANT_OBSERVATION_IDS {
            return Err("canonical_backend_dormant_observation_inventory_mismatch_v0");
        }
        Ok(())
    }

    fn validate_c1_dormant_observations(observations: &[(&'static str, String)]) {
        let ids = observations.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        assert_eq!(
            check_c1_dormant_observation_ids(&ids),
            Ok(()),
            "C1 dormant observation inventory was removed, duplicated, aliased, skipped, conditionally bypassed, renamed, or replaced"
        );
        for human_id in C1_DORMANT_OBSERVATION_IDS
            .iter()
            .filter(|id| id.ends_with(".human"))
        {
            let json_id = format!("{}.json", human_id.trim_end_matches(".human"));
            let human = observations
                .iter()
                .find(|(id, _)| id == human_id)
                .map(|(_, value)| value)
                .expect("independent dormant human column");
            let json = observations
                .iter()
                .find(|(id, _)| *id == json_id)
                .map(|(_, value)| value)
                .expect("independent dormant JSON column");
            assert_ne!(
                human, json,
                "dormant human and JSON columns must be independently observed: {human_id}"
            );
        }
    }

    fn c1_dormant_observations(
        program: &Program,
        diagnostics: &[crate::diagnostic::Diagnostic],
    ) -> Vec<(&'static str, String)> {
        let run =
            crate::run::run_program(program, Some("add"), &["1".to_string(), "2".to_string()]);
        let observations = vec![
            (
                "resolve.human",
                crate::resolve::resolve_text(program, diagnostics),
            ),
            (
                "resolve.json",
                crate::resolve::resolve_json(program, diagnostics),
            ),
            (
                "type_env.human",
                crate::type_env::type_env_text(program, diagnostics),
            ),
            (
                "type_env.json",
                crate::type_env::type_env_json(program, diagnostics),
            ),
            (
                "type_check.human",
                crate::type_check::type_check_text(program, diagnostics),
            ),
            (
                "type_check.json",
                crate::type_check::type_check_json(program, diagnostics),
            ),
            (
                "full_type.human",
                crate::full_type_check::full_type_check_text(program, diagnostics),
            ),
            (
                "full_type.json",
                crate::full_type_check::full_type_check_json(program, diagnostics),
            ),
            (
                "effect.human",
                crate::effect_check::effect_check_text(program, diagnostics),
            ),
            (
                "effect.json",
                crate::effect_check::effect_check_json(program, diagnostics),
            ),
            (
                "ownership.human",
                crate::ownership_check::ownership_check_text(program, diagnostics),
            ),
            (
                "ownership.json",
                crate::ownership_check::ownership_check_json(program, diagnostics),
            ),
            (
                "resource.human",
                crate::resource_check::resource_check_text(program, diagnostics),
            ),
            (
                "resource.json",
                crate::resource_check::resource_check_json(program, diagnostics),
            ),
            (
                "profile.human",
                crate::profile_check::profile_check_text(program, diagnostics),
            ),
            (
                "profile.json",
                crate::profile_check::profile_check_json(program, diagnostics),
            ),
            (
                "core_preview.human",
                crate::core_preview::core_preview_text(program, diagnostics),
            ),
            (
                "core_preview.json",
                crate::core_preview::core_preview_json(program, diagnostics),
            ),
            (
                "core_lower.human",
                crate::core_lower::core_lower_text(program, diagnostics),
            ),
            (
                "core_lower.json",
                crate::core_lower::core_lower_json(program, diagnostics),
            ),
            (
                "core_verify.human",
                crate::core_verify::core_verify_text(program, diagnostics),
            ),
            (
                "core_verify.json",
                crate::core_verify::core_verify_json(program, diagnostics),
            ),
            (
                "graph.program_json",
                crate::json::program_to_json(program, diagnostics),
            ),
            ("runtime.outcome", format!("{:?}", run.outcome)),
            ("runtime.diagnostics", format!("{:?}", run.diagnostics)),
            (
                "runtime.authority_events",
                format!("{:?}", run.authority_events),
            ),
            (
                "runtime.rendered_diagnostics",
                run.diagnostics
                    .iter()
                    .map(crate::diagnostic::Diagnostic::render)
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            ("source_diagnostics.values", format!("{diagnostics:?}")),
            (
                "source_diagnostics.human",
                diagnostics
                    .iter()
                    .map(crate::diagnostic::Diagnostic::render)
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            (
                "source_diagnostics.json",
                crate::diagnostics::check_json(program, diagnostics),
            ),
            ("catalogue.human", crate::diagnostics::diagnostics_text()),
            ("catalogue.json", crate::diagnostics::diagnostics_json()),
            (
                "explain.human",
                crate::explain::explain_text("H0001").expect("pinned diagnostic code"),
            ),
            (
                "explain.json",
                crate::explain::explain_json("H0001").expect("pinned diagnostic code"),
            ),
        ];
        validate_c1_dormant_observations(&observations);
        observations
    }

    #[test]
    fn ten_bc1_canonical_checked_add_producer_closure_is_complete_and_load_bearing() {
        let parsed = parse_source(
            "examples/core/minimal_add.hum",
            include_str!("../examples/core/minimal_add.hum"),
        );
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let diagnostics = parsed.diagnostics;
        let program = Program {
            files: vec![parsed.file],
        };
        let baseline_json = ir_readiness_json(&program, &diagnostics);
        let baseline_text = ir_readiness_text(&program, &diagnostics);
        let baseline_dormant = c1_dormant_observations(&program, &diagnostics);
        assert_eq!(ir_readiness_json(&program, &diagnostics), baseline_json);
        assert_eq!(ir_readiness_text(&program, &diagnostics), baseline_text);
        assert_eq!(
            c1_dormant_observations(&program, &diagnostics),
            baseline_dormant,
            "two clean dormant runs must be deterministic"
        );
        let foreign_source = r#"module examples.core.minimal_add

task prior(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing

  does:
    return a + b
}

task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing

  does:
    return a + b
}
"#;
        let foreign_parsed = parse_source_at_index("foreign/minimal_add.hum", foreign_source, 0);
        assert!(foreign_parsed.diagnostics.is_empty());
        let foreign_program = Program {
            files: vec![foreign_parsed.file],
        };
        let foreign = candidate_for(&foreign_program, &[], 0, 1);

        for expected in [
            "\"status\": \"canonical_checked_add_producer_closure_available_v0\"",
            "\"module_display_name\": \"examples.core.minimal_add\"",
            "\"function_display_name\": \"add\"",
            "\"parameter_names\": [\"a\", \"b\"]",
            "\"parameter_type_names\": [\"Int\", \"Int\"]",
            "\"parameter_permissions\": [\"borrow\", \"borrow\"]",
            "\"result_type_name\": \"Int\"",
            "\"operation_kinds\": [\"return\", \"checked_add\"]",
            "\"parameter_type_identities\": [\"hum.type.int.s64.v0\", \"hum.type.int.s64.v0\"]",
            "\"operand_type_identities\": [\"hum.type.int.s64.v0\", \"hum.type.int.s64.v0\"]",
            "\"add_result_type_identity\": \"hum.type.int.s64.v0\"",
            "\"function_result_type_identity\": \"hum.type.int.s64.v0\"",
            "\"effect_identity\": \"hum.effect.pure.v0\"",
            "\"authority_identity\": \"hum.authority.none.v0\"",
            "\"ownership_identity\": \"hum.ownership.accepted.no_transfer.v0\"",
            "\"profile_identity\": \"normal\"",
            "\"overflow_status\": \"checked_add_runtime_trap_exit_2_v0\"",
            "\"missing_artifact\": \"hum.backend_input.v0_absent_v0\"",
            "\"missing_verifier\": \"ir_verify_unimplemented_v0\"",
        ] {
            assert!(
                baseline_json.contains(expected),
                "missing independent oracle: {expected}"
            );
        }
        assert_eq!(C1_FACT_CATALOGUE.len(), 111);
        let ids = C1_FACT_CATALOGUE
            .iter()
            .map(|fact| fact.id)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), 111, "C1 fact IDs must be unique");
        let owners = C1_FACT_CATALOGUE.iter().fold(
            std::collections::BTreeMap::<&str, usize>::new(),
            |mut owners, fact| {
                *owners.entry(fact.owner).or_default() += 1;
                owners
            },
        );
        assert_eq!(
            owners,
            std::collections::BTreeMap::from([
                ("core", 26),
                ("effect", 4),
                ("full_type", 8),
                ("join", 17),
                ("ownership", 3),
                ("parser", 37),
                ("profile", 4),
                ("resolver", 8),
                ("resource", 4),
            ])
        );
        let dormant_ids = C1_DORMANT_OBSERVATION_IDS.to_vec();
        assert_eq!(check_c1_dormant_observation_ids(&dormant_ids), Ok(()));
        let mut removed = dormant_ids.clone();
        removed.pop();
        assert_eq!(
            check_c1_dormant_observation_ids(&removed),
            Err("canonical_backend_dormant_observation_inventory_mismatch_v0")
        );
        let mut duplicate = dormant_ids.clone();
        duplicate[1] = duplicate[0];
        assert_eq!(
            check_c1_dormant_observation_ids(&duplicate),
            Err("canonical_backend_dormant_observation_duplicate_or_alias_v0")
        );
        let mut renamed = dormant_ids.clone();
        renamed[0] = "resolve.renamed";
        assert_eq!(
            check_c1_dormant_observation_ids(&renamed),
            Err("canonical_backend_dormant_observation_inventory_mismatch_v0")
        );
        let mut readiness_replacement = dormant_ids.clone();
        readiness_replacement[0] = "ir_readiness.human";
        assert_eq!(
            check_c1_dormant_observation_ids(&readiness_replacement),
            Err("canonical_backend_dormant_observation_readiness_alias_v0")
        );
        assert_eq!(
            check_c1_dormant_observation_ids(&[]),
            Err("canonical_backend_dormant_observation_inventory_zero_v0")
        );
        for fact in C1_FACT_CATALOGUE {
            assert_eq!(fact.dormant_observations, 34);
            for &subcase in c1_fact_subcases(fact.corruption) {
                let (corrupted_text, corrupted_json, corrupted_dormant) =
                    with_c1_producer_corruption_subcase(
                        fact.corruption,
                        Some(foreign.clone()),
                        subcase,
                        || {
                            let parsed = parse_source(
                                "examples/core/minimal_add.hum",
                                include_str!("../examples/core/minimal_add.hum"),
                            );
                            let program = Program {
                                files: vec![parsed.file],
                            };
                            let first_text = ir_readiness_text(&program, &parsed.diagnostics);
                            let second_text = ir_readiness_text(&program, &parsed.diagnostics);
                            let first_json = ir_readiness_json(&program, &parsed.diagnostics);
                            let second_json = ir_readiness_json(&program, &parsed.diagnostics);
                            assert_eq!(
                                first_text, second_text,
                                "nondeterministic human row/subcase: {fact:?} {subcase:?}"
                            );
                            assert_eq!(
                                first_json, second_json,
                                "nondeterministic JSON row/subcase: {fact:?} {subcase:?}"
                            );
                            (
                                first_text,
                                first_json,
                                c1_dormant_observations(&program, &parsed.diagnostics),
                            )
                        },
                    );
                assert_ne!(
                    corrupted_text, baseline_text,
                    "inert human corruption: {fact:?} {subcase:?}"
                );
                assert_ne!(
                    corrupted_json, baseline_json,
                    "inert JSON corruption: {fact:?} {subcase:?}"
                );
                assert!(
                    !corrupted_json.contains(
                        "\"status\": \"canonical_checked_add_producer_closure_available_v0\""
                    ),
                    "corruption remained accepted: {fact:?} {subcase:?}"
                );
                assert!(
                    corrupted_json.contains(&format!(
                        "\"checked_add_producer_blocker\": \"{}\"",
                        fact.expected_blocker
                    )),
                    "JSON corruption did not fail closed with exact blocker: {fact:?} {subcase:?}\n{corrupted_json}"
                );
                assert!(
                    corrupted_text.contains(fact.expected_blocker),
                    "human corruption did not name exact blocker: {fact:?} {subcase:?}"
                );
                assert_eq!(
                    corrupted_dormant, baseline_dormant,
                    "C1 corruption escaped into a dormant surface: {fact:?} {subcase:?}"
                );
            }
        }

        for (corruption, blocker) in [
            (C1ProducerCorruption::ParameterOrdinal(0), PARSER_BLOCKER),
            (C1ProducerCorruption::ReturnOperationIdentity, CORE_BLOCKER),
            (C1ProducerCorruption::LeftNodeIdentity, CORE_BLOCKER),
            (C1ProducerCorruption::OperationKind(0), CORE_BLOCKER),
            (C1ProducerCorruption::OperandValueIdentity(0), CORE_BLOCKER),
        ] {
            let (json, dormant) = with_c1_producer_corruption_subcase(
                corruption,
                Some(foreign.clone()),
                C1CorruptionSubcase::OrderedMemberSwap,
                || {
                    let parsed = parse_source(
                        "examples/core/minimal_add.hum",
                        include_str!("../examples/core/minimal_add.hum"),
                    );
                    let program = Program {
                        files: vec![parsed.file],
                    };
                    (
                        ir_readiness_json(&program, &parsed.diagnostics),
                        c1_dormant_observations(&program, &parsed.diagnostics),
                    )
                },
            );
            assert!(
                json.contains(&format!("\"checked_add_producer_blocker\": \"{blocker}\"")),
                "ordered member-set-preserving swap did not reach its owning validator: {corruption:?}"
            );
            assert_eq!(dormant, baseline_dormant);
        }
        for pass_index in 0..13 {
            let (json, dormant) = with_c1_producer_corruption_subcase(
                C1ProducerCorruption::AcceptedPass(pass_index),
                Some(foreign.clone()),
                C1CorruptionSubcase::OrderedMemberSwap,
                || {
                    let parsed = parse_source(
                        "examples/core/minimal_add.hum",
                        include_str!("../examples/core/minimal_add.hum"),
                    );
                    let program = Program {
                        files: vec![parsed.file],
                    };
                    (
                        ir_readiness_json(&program, &parsed.diagnostics),
                        c1_dormant_observations(&program, &parsed.diagnostics),
                    )
                },
            );
            assert!(
                json.contains(&format!(
                    "\"checked_add_producer_blocker\": \"{JOIN_BLOCKER}\""
                )),
                "accepted-pass member-set-preserving swap did not fail at the join: {pass_index}"
            );
            assert_eq!(dormant, baseline_dormant);
        }

        let deterministic_corruption = || {
            with_c1_producer_corruption(
                C1ProducerCorruption::EffectIdentity,
                Some(foreign.clone()),
                || {
                    let parsed = parse_source(
                        "examples/core/minimal_add.hum",
                        include_str!("../examples/core/minimal_add.hum"),
                    );
                    let program = Program {
                        files: vec![parsed.file],
                    };
                    (
                        ir_readiness_text(&program, &parsed.diagnostics),
                        ir_readiness_json(&program, &parsed.diagnostics),
                        c1_dormant_observations(&program, &parsed.diagnostics),
                    )
                },
            )
        };
        assert_eq!(
            deterministic_corruption(),
            deterministic_corruption(),
            "two fresh corrupted runs must be byte-deterministic"
        );
    }

    #[test]
    fn ten_bc1_same_shaped_substitution_authority_and_dormancy_fail_closed() {
        let parsed = parse_source(
            "examples/core/minimal_add.hum",
            include_str!("../examples/core/minimal_add.hum"),
        );
        let diagnostics = parsed.diagnostics;
        let mut program = Program {
            files: vec![parsed.file],
        };
        let dormant_before = c1_dormant_observations(&program, &diagnostics);
        let foreign_source = r#"module examples.core.minimal_add

task prior(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing

  does:
    return a + b
}

task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing

  does:
    return a + b
}
"#;
        let foreign_parsed = parse_source_at_index("foreign/minimal_add.hum", foreign_source, 0);
        let foreign_program = Program {
            files: vec![foreign_parsed.file],
        };
        let (foreign_body, foreign_section_body) = match &foreign_program.files[0].items[1] {
            Item::Task(task) => (
                task.body_syntax.clone(),
                task.section("does")
                    .expect("foreign does")
                    .body_syntax
                    .clone(),
            ),
            _ => panic!("foreign C1 task"),
        };
        let foreign = candidate_for(&foreign_program, &[], 0, 1);
        for (corruption, local_blocker) in [
            (
                C1ProducerCorruption::CoherentOwnerSubstitution,
                CORE_BLOCKER,
            ),
            (
                C1ProducerCorruption::CoherentOperandSubstitution,
                CORE_BLOCKER,
            ),
        ] {
            for (subcase, exact_blocker) in [
                (C1CorruptionSubcase::Primary, local_blocker),
                (C1CorruptionSubcase::PostProducerJoin, CORE_JOIN_BLOCKER),
            ] {
                let (active_text, active_json, dormant_during_corruption) =
                    with_c1_producer_corruption_subcase(
                        corruption,
                        Some(foreign.clone()),
                        subcase,
                        || {
                            (
                                ir_readiness_text(&program, &diagnostics),
                                ir_readiness_json(&program, &diagnostics),
                                c1_dormant_observations(&program, &diagnostics),
                            )
                        },
                    );
                assert!(
                    active_text.contains(exact_blocker),
                    "coherent substitution missed exact human blocker: {corruption:?} {subcase:?}\n{active_text}"
                );
                assert!(
                    active_json.contains(&format!(
                        "\"checked_add_producer_blocker\": \"{exact_blocker}\""
                    )),
                    "coherent substitution missed exact JSON blocker: {corruption:?} {subcase:?}"
                );
                assert_eq!(dormant_during_corruption, dormant_before);
            }
        }

        let baseline_text = ir_readiness_text(&program, &diagnostics);
        let baseline_json = ir_readiness_json(&program, &diagnostics);
        let baseline_resolve_text = crate::resolve::resolve_text(&program, &diagnostics);
        let baseline_resolve_json = crate::resolve::resolve_json(&program, &diagnostics);
        {
            let Item::Task(task) = &mut program.files[0].items[0] else {
                panic!("independent oracle expects one task");
            };
            task.body_syntax = foreign_body.clone();
        }
        assert_eq!(ir_readiness_text(&program, &diagnostics), baseline_text);
        assert_eq!(ir_readiness_json(&program, &diagnostics), baseline_json);
        assert_eq!(
            crate::resolve::resolve_text(&program, &diagnostics),
            baseline_resolve_text
        );
        assert_eq!(
            crate::resolve::resolve_json(&program, &diagnostics),
            baseline_resolve_json
        );
        assert_eq!(
            c1_dormant_observations(&program, &diagnostics),
            dormant_before,
            "legacy Task.body_syntax clone became a C1 or dormant authority"
        );

        let mut live_section_only = minimal_program();
        let Item::Task(task) = &mut live_section_only.files[0].items[0] else {
            panic!("task")
        };
        task.sections
            .iter_mut()
            .find(|section| section.name == "does")
            .expect("does")
            .body_syntax = foreign_section_body.clone();
        assert_eq!(
            backend_boundary_result(&live_section_only),
            Err("canonical_core_section_projection_mismatch_v0"),
            "live Section projection must remain load-bearing"
        );

        let mut coherent_clone_and_section = minimal_program();
        let Item::Task(task) = &mut coherent_clone_and_section.files[0].items[0] else {
            panic!("task")
        };
        task.body_syntax = foreign_body.clone();
        task.sections
            .iter_mut()
            .find(|section| section.name == "does")
            .expect("does")
            .body_syntax = foreign_section_body;
        assert_eq!(
            backend_boundary_result(&coherent_clone_and_section),
            Err("canonical_core_section_projection_mismatch_v0"),
            "coherent legacy-clone/Section projection substitution must retain the original authority rejection"
        );

        let first = parse_source_at_index(
            "same/first.hum",
            include_str!("../examples/core/minimal_add.hum"),
            0,
        );
        let second = parse_source_at_index(
            "same/second.hum",
            include_str!("../examples/core/minimal_add.hum"),
            1,
        );
        assert!(first.diagnostics.is_empty() && second.diagnostics.is_empty());
        let same_shaped = Program {
            files: vec![first.file, second.file],
        };
        let same_shaped_json = ir_readiness_json(&same_shaped, &[]);
        assert_eq!(
            same_shaped_json
                .matches("\"status\": \"canonical_checked_add_producer_closure_available_v0\"")
                .count(),
            2
        );
        assert!(same_shaped_json.contains("\"semantic_file_index\": 0"));
        assert!(same_shaped_json.contains("\"semantic_file_index\": 1"));

        let Item::Task(task) = &mut program.files[0].items[0] else {
            panic!("independent oracle expects one task");
        };
        task.remove_canonical_backend_signature_for_test();
        let without_authority = ir_readiness_json(&program, &diagnostics);
        assert!(without_authority.contains(
            "\"checked_add_producer_blocker\": \"canonical_backend_signature_capability_absent_v0\""
        ));

        let mut projection_mutations: Vec<ProjectionMutation> = vec![
            (
                "module",
                Box::new(|program| program.files[0].module = Some("foreign.module".into())),
            ),
            (
                "function",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.name = "foreign_add".into();
                }),
            ),
            (
                "function_range",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.span.column += 1;
                }),
            ),
            (
                "parameter_order",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.params.swap(0, 1);
                }),
            ),
            (
                "parameter_binder",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.params[0].name = "foreign_a".into();
                }),
            ),
            (
                "parameter_range",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.params[0].span.column += 1;
                }),
            ),
            (
                "parameter_type",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.params[0].ty = "UInt".into();
                }),
            ),
            (
                "parameter_permission",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.params[0].permission = crate::ast::ParamPermission::Change;
                }),
            ),
            (
                "result",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.result = Some("UInt".into());
                }),
            ),
            (
                "result_range",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.result_syntax
                        .as_mut()
                        .expect("result syntax")
                        .span
                        .column += 1;
                }),
            ),
            (
                "does_relationship",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.sections
                        .iter_mut()
                        .find(|section| section.name == "does")
                        .expect("does")
                        .name = "foreign_does".into();
                }),
            ),
            (
                "does_range",
                Box::new(|program| {
                    let Item::Task(task) = &mut program.files[0].items[0] else {
                        panic!("task");
                    };
                    task.sections
                        .iter_mut()
                        .find(|section| section.name == "does")
                        .expect("does")
                        .span
                        .column += 1;
                }),
            ),
        ];
        for (label, mutate) in projection_mutations.drain(..) {
            let mut program = minimal_program();
            mutate(&mut program);
            let expected = match label {
                "does_relationship" => "canonical_backend_does_section_absent_v0",
                "does_range" => "canonical_core_section_projection_mismatch_v0",
                _ => "canonical_backend_signature_projection_mismatch_v0",
            };
            assert_eq!(
                backend_boundary_result(&program),
                Err(expected),
                "direct {label} projection mutation missed its exact authority boundary"
            );
        }

        let two_task_source = r#"module examples.core.minimal_add

task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a + b
}

task add_again(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a + b
}
"#;
        let parsed = parse_source("same/items.hum", two_task_source);
        assert!(parsed.diagnostics.is_empty());
        let mut moved = Program {
            files: vec![parsed.file],
        };
        moved.files[0].items.swap(0, 1);
        assert_eq!(
            backend_boundary_result(&moved),
            Err("canonical_core_item_witness_mismatch_v0"),
            "direct item-path projection mutation missed the owner witness"
        );

        let mut kind_changed = minimal_program();
        let original_task = match &kind_changed.files[0].items[0] {
            Item::Task(task) => task.clone(),
            _ => panic!("task"),
        };
        let test = parse_source(
            "same/test.hum",
            "module same.test\ntest add(a: Int, b: Int) {\n  does:\n    return a + b\n}\n",
        );
        kind_changed.files[0].items[0] = test.file.items[0].clone();
        assert_eq!(
            kind_changed
                .canonical_backend_function_expectation(&original_task)
                .map(|_| ()),
            Err("canonical_backend_live_task_reference_mismatch_v0"),
            "direct item-kind replacement missed the live-task boundary"
        );

        let mut target = minimal_program();
        let foreign_revision_source = format!(
            "{}\n# distinct source revision with equal signature spelling\n",
            include_str!("../examples/core/minimal_add.hum")
        );
        let foreign_revision = parse_source("foreign/revision.hum", &foreign_revision_source);
        let Item::Task(foreign_task) = &foreign_revision.file.items[0] else {
            panic!("foreign task");
        };
        let Item::Task(target_task) = &mut target.files[0].items[0] else {
            panic!("target task");
        };
        target_task.substitute_canonical_backend_signature_from_for_test(foreign_task);
        assert_eq!(
            backend_boundary_result(&target),
            Err("canonical_backend_signature_projection_mismatch_v0"),
            "same-shaped foreign revision capability missed signature validation"
        );

        let mut target = minimal_program();
        let foreign_ordinal = parse_source_at_index(
            "foreign/ordinal.hum",
            include_str!("../examples/core/minimal_add.hum"),
            1,
        );
        let Item::Task(foreign_task) = &foreign_ordinal.file.items[0] else {
            panic!("foreign task");
        };
        let Item::Task(target_task) = &mut target.files[0].items[0] else {
            panic!("target task");
        };
        target_task.substitute_canonical_backend_signature_from_for_test(foreign_task);
        assert_eq!(
            backend_boundary_result(&target),
            Err("canonical_backend_signature_projection_mismatch_v0"),
            "same-shaped foreign semantic ordinal capability missed signature validation"
        );

        let parsed = parse_source("same/items.hum", two_task_source);
        let mut target = minimal_program();
        let Item::Task(foreign_task) = &parsed.file.items[1] else {
            panic!("foreign path task");
        };
        let Item::Task(target_task) = &mut target.files[0].items[0] else {
            panic!("target task");
        };
        target_task.substitute_canonical_backend_signature_from_for_test(foreign_task);
        assert_eq!(
            backend_boundary_result(&target),
            Err("canonical_backend_signature_projection_mismatch_v0"),
            "same-shaped foreign item-path/equal-binder capability missed signature validation"
        );

        let foreign_exact = parse_source_at_index(
            "foreign/exact.hum",
            include_str!("../examples/core/minimal_add.hum"),
            0,
        );
        assert!(foreign_exact.diagnostics.is_empty());
        let Item::Task(foreign_task) = &foreign_exact.file.items[0] else {
            panic!("foreign exact task")
        };
        let foreign_does = foreign_task.section("does").expect("foreign exact does");

        let mut foreign_section_capability = minimal_program();
        let Item::Task(task) = &mut foreign_section_capability.files[0].items[0] else {
            panic!("task")
        };
        task.sections
            .iter_mut()
            .find(|section| section.name == "does")
            .expect("does")
            .corrupt_canonical_core_capability_from(foreign_does);
        assert_eq!(
            backend_boundary_result(&foreign_section_capability),
            Err("canonical_core_section_binding_mismatch_v0"),
            "foreign Section capability missed the retained Section binding"
        );

        let mut whole_section = minimal_program();
        let Item::Task(task) = &mut whole_section.files[0].items[0] else {
            panic!("task")
        };
        let does_slot = task
            .sections
            .iter()
            .position(|section| section.name == "does")
            .expect("does");
        task.sections[does_slot] = foreign_does.clone();
        assert_eq!(
            backend_boundary_result(&whole_section),
            Err("canonical_core_section_binding_mismatch_v0"),
            "whole foreign Section missed the retained Section binding"
        );

        let mut whole_item = minimal_program();
        whole_item.files[0].items[0] = foreign_exact.file.items[0].clone();
        assert_eq!(
            backend_boundary_result(&whole_item),
            Err("canonical_core_item_witness_mismatch_v0"),
            "whole foreign item missed the destination owner witness"
        );

        let foreign_ordinal = parse_source_at_index(
            "examples/core/minimal_add.hum",
            include_str!("../examples/core/minimal_add.hum"),
            1,
        );
        let wrong_ordinal_program = Program {
            files: vec![foreign_ordinal.file],
        };
        assert_eq!(
            backend_boundary_result(&wrong_ordinal_program),
            Err("canonical_core_file_witness_mismatch_v0"),
            "whole file at the wrong semantic ordinal missed the file witness"
        );

        let revision_source = format!(
            "{}\n# same public task projection, distinct source revision\n",
            include_str!("../examples/core/minimal_add.hum")
        );
        let foreign_revision =
            parse_source_at_index("examples/core/minimal_add.hum", &revision_source, 0);
        let mut revision_item = minimal_program();
        revision_item.files[0].items[0] = foreign_revision.file.items[0].clone();
        assert_eq!(
            backend_boundary_result(&revision_item),
            Err("canonical_core_item_witness_mismatch_v0"),
            "whole item from a foreign source revision missed the owner witness"
        );
    }

    #[test]
    fn ten_bc1_source_audit_pins_real_join_and_forbids_later_work() {
        let source = include_str!("ir_readiness.rs");
        let join = source
            .split("fn canonical_checked_add_producer_closure(")
            .nth(1)
            .and_then(|tail| {
                tail.split("fn validate_canonical_checked_add_producer_closure(")
                    .next()
            })
            .expect("C1 producer join must exist");
        for required in [
            "canonical_backend_function_expectation",
            "try_analyze_backend_function",
            "canonical_checked_add_core_view",
            "canonical_backend_resolver_view",
            "canonical_backend_checked_add_types",
            "canonical_backend_checked_add_effect",
            "canonical_backend_checked_add_ownership",
            "canonical_backend_checked_add_resource",
            "canonical_backend_checked_add_profile",
            "c1_readiness_producer_facts",
            "validate_canonical_checked_add_producer_closure",
        ] {
            assert!(
                join.contains(required),
                "missing real producer join: {required}"
            );
        }
        for forbidden in [
            "VerifiedBackendInput",
            "hum.backend_input.v0\"",
            "cranelift",
            "write_object",
            "emit_clif",
        ] {
            assert!(
                !join.contains(forbidden),
                "later work leaked into C1: {forbidden}"
            );
        }
        assert!(
            !join.contains("apply_c1_corruption_for_test")
                && !join.contains("&mut CanonicalCheckedAddProducerClosure"),
            "the final C1 output must never be the corruption boundary"
        );

        let parser_source = include_str!("parser.rs");
        let issuer = parser_source
            .split("fn select_and_issue_canonical_backend_signature(")
            .nth(1)
            .and_then(|tail| tail.split("fn parse_test_header(").next())
            .expect("parser-owned signature issuer");
        for required in [
            "header: ParserBackendTaskHeaderEvents",
            "module_token",
            "backend_section_headers",
            "CanonicalBackendSignatureCapability::from_selected(binding)",
            "statement.block_relationship",
            "CanonicalExpressionKind::Binary",
            "ParsedBinaryOperator::Add",
        ] {
            assert!(
                issuer.contains(required),
                "parser signature issuer lost independent event authority: {required}"
            );
        }
        let constructor = parser_source
            .split("fn from_selected(mut binding: CanonicalBackendSignatureBinding)")
            .nth(1)
            .and_then(|tail| tail.split("pub(crate) fn binding").next())
            .expect("sole parser-private C1 capability constructor");
        for required in [
            "let retained = Arc::new(binding.clone())",
            "apply_c1_parser_producer_corruption(&mut binding)",
            "projection: Arc::new(binding)",
        ] {
            assert!(
                constructor.contains(required),
                "parser constructor lost retained authority ordering: {required}"
            );
        }
        assert_eq!(
            parser_source
                .matches("CanonicalBackendSignatureCapability::from_selected(")
                .count(),
            1,
            "C1 capability must have exactly one production construction expression"
        );
        for forbidden in [
            "task: &Task",
            "params: &[Param]",
            "result_syntax: Option<&TypeSyntax>",
        ] {
            assert!(
                !issuer.contains(forbidden),
                "parser issuer accepted a finished projection: {forbidden}"
            );
        }
        let other_issuance_sources = [
            ("ast.rs", include_str!("ast.rs")),
            ("core_body.rs", include_str!("core_body.rs")),
            ("core_expr.rs", include_str!("core_expr.rs")),
            ("core_lower.rs", include_str!("core_lower.rs")),
            ("effect_check.rs", include_str!("effect_check.rs")),
            ("full_type_check.rs", include_str!("full_type_check.rs")),
            ("ownership_check.rs", include_str!("ownership_check.rs")),
            ("profile_check.rs", include_str!("profile_check.rs")),
            ("resolve.rs", include_str!("resolve.rs")),
            ("resource_check.rs", include_str!("resource_check.rs")),
        ];
        assert_eq!(
            check_c1_issuance_source_layout(parser_source, &other_issuance_sources),
            Ok(())
        );
        let mut mint = parser_source.replacen(
            "fn from_selected(mut binding: CanonicalBackendSignatureBinding)",
            "pub(crate) fn from_selected(mut binding: CanonicalBackendSignatureBinding)",
            1,
        );
        assert_eq!(
            check_c1_issuance_source_layout(&mint, &other_issuance_sources),
            Err("canonical_backend_issuance_mint_exposed_v0")
        );
        mint.clear();
        let pre_selection = parser_source.replacen(
            "let sections =",
            "let signature = self.select_and_issue_canonical_backend_signature(item_path, &[], backend_header);\n        let sections =",
            1,
        );
        assert_eq!(
            check_c1_issuance_source_layout(&pre_selection, &other_issuance_sources),
            Err("canonical_backend_issuance_pre_selection_v0")
        );
        let post_selection_fallback = parser_source.replacen(
            "CanonicalBackendSignatureCapability::from_selected(binding)",
            "CanonicalBackendSignatureCapability::from_selected(binding).or_else(c1_fallback)",
            1,
        );
        assert_eq!(
            check_c1_issuance_source_layout(&post_selection_fallback, &other_issuance_sources),
            Err("canonical_backend_issuance_post_selection_fallback_v0")
        );
        let duplicate = parser_source.replacen(
            "CanonicalBackendSignatureCapability::from_selected(binding)",
            "CanonicalBackendSignatureCapability::from_selected(binding);\n        CanonicalBackendSignatureCapability::from_selected(binding)",
            1,
        );
        assert_eq!(
            check_c1_issuance_source_layout(&duplicate, &other_issuance_sources),
            Err("canonical_backend_issuance_duplicate_v0")
        );
        let decoy = format!(
            "{parser_source}\nfn c1_decoy(binding: CanonicalBackendSignatureBinding) {{ let _ = CanonicalBackendSignatureCapability::from_selected(binding); }}\n"
        );
        assert_eq!(
            check_c1_issuance_source_layout(&decoy, &other_issuance_sources),
            Err("canonical_backend_issuance_decoy_present_v0")
        );
        let alternate_file = [(
            "alternate.rs",
            "fn alternate(binding: CanonicalBackendSignatureBinding) { let _ = CanonicalBackendSignatureCapability::from_selected(binding); }",
        )];
        assert_eq!(
            check_c1_issuance_source_layout(parser_source, &alternate_file),
            Err("canonical_backend_issuance_alternate_file_v0")
        );
        let elsewhere = format!(
            "{parser_source}\nfn elsewhere(binding: CanonicalBackendSignatureBinding) {{ let _ = CanonicalBackendSignatureCapability::from_selected(binding); }}\n"
        );
        assert_eq!(
            check_c1_issuance_source_layout(&elsewhere, &other_issuance_sources),
            Err("canonical_backend_issuance_elsewhere_v0")
        );
        let foreign_installation = parser_source.replacen(
            "\n                signature,\n",
            "\n                None,\n",
            1,
        );
        assert_eq!(
            check_c1_issuance_source_layout(&foreign_installation, &other_issuance_sources),
            Err("canonical_backend_issuance_foreign_installation_route_v0")
        );

        let resolver_source = include_str!("resolve.rs");
        assert!(
            !resolver_source.contains("task.body_syntax"),
            "C1 resolver production may not read the legacy Task body clone"
        );
        for required in [
            "resolve_canonical_body: true",
            "resolve_canonical_body: false",
            "section.body_syntax",
            "canonical_core_expectation(item, section)",
            "resolve_canonical_call_occurrences",
            "resolve_canonical_callable_references",
        ] {
            assert!(
                resolver_source.contains(required),
                "C1 resolver lost live Section authority: {required}"
            );
        }
        for forbidden in [
            "parse_task_body_syntax",
            "BodyStatement.text",
            "rendered expression",
            "reparse",
        ] {
            assert!(
                !resolver_source.contains(forbidden),
                "forbidden C1 resolver body authority appeared: {forbidden}"
            );
        }

        let core_source = include_str!("core_lower.rs");
        let block_identity = core_source
            .split("fn canonical_backend_block_identity(")
            .nth(1)
            .and_then(|tail| tail.split("const NON_GOALS").next())
            .expect("canonical block identity");
        for required in [
            "source_revision",
            "semantic_file_index",
            "item_path",
            "does_section_slot",
            "does_section.identity",
            "does_section.range",
        ] {
            assert!(
                block_identity.contains(required),
                "block identity lost retained owner fact: {required}"
            );
        }
        assert!(
            !block_identity.contains("canonical-block:{}"),
            "display-only does-section identity must remain forbidden"
        );

        let catalogue_source = source
            .split("const C1_FACT_CATALOGUE:")
            .nth(1)
            .and_then(|tail| tail.split("fn candidate_for(").next())
            .expect("independently authored C1 fact catalogue");
        assert!(
            !catalogue_source.contains("C1_DORMANT_OBSERVATION_IDS"),
            "the C1 fact catalogue may not derive dormant inventory shape"
        );
        let dormant_inventory_source = source
            .split("const C1_DORMANT_OBSERVATION_IDS:")
            .nth(1)
            .and_then(|tail| tail.split("fn check_c1_dormant_observation_ids(").next())
            .expect("independently authored dormant observation inventory");
        assert!(
            !dormant_inventory_source.contains("C1_FACT_CATALOGUE"),
            "the dormant inventory may not derive its shape from the fact catalogue"
        );
        let dormant_observer_source = source
            .split("fn c1_dormant_observations(")
            .nth(1)
            .and_then(|tail| {
                tail.split(
                    "fn ten_bc1_canonical_checked_add_producer_closure_is_complete_and_load_bearing(",
                )
                .next()
            })
            .expect("concrete dormant observer");
        assert!(
            dormant_observer_source.contains("let observations = vec![")
                && !dormant_observer_source.contains("C1_DORMANT_OBSERVATION_IDS.iter().map",),
            "dormant observations must be concrete independent columns, not generated aliases"
        );

        for (name, producer, corruption, validation) in [
            (
                "core",
                include_str!("core_lower.rs"),
                "apply_c1_core_producer_corruption",
                "validate_canonical_checked_add_core_view",
            ),
            (
                "resolver",
                include_str!("resolve.rs"),
                "apply_c1_resolver_producer_corruption",
                "canonical_backend_resolver_producer_corruption_v0",
            ),
            (
                "type",
                include_str!("full_type_check.rs"),
                "apply_c1_type_producer_corruption",
                "canonical_backend_type_producer_corruption_v0",
            ),
            (
                "effect",
                include_str!("effect_check.rs"),
                "apply_c1_effect_producer_corruption",
                "canonical_backend_effect_producer_corruption_v0",
            ),
            (
                "ownership",
                include_str!("ownership_check.rs"),
                "apply_c1_ownership_producer_corruption",
                "canonical_backend_ownership_producer_corruption_v0",
            ),
            (
                "resource",
                include_str!("resource_check.rs"),
                "apply_c1_resource_producer_corruption",
                "canonical_backend_resource_producer_corruption_v0",
            ),
            (
                "profile",
                include_str!("profile_check.rs"),
                "apply_c1_profile_producer_corruption",
                "canonical_backend_profile_producer_corruption_v0",
            ),
        ] {
            let corruption_at = producer.find(corruption).expect("producer corruption seam");
            let validation_at = producer
                .find(validation)
                .expect("producer-local validation");
            assert!(
                corruption_at < validation_at,
                "{name} corruption must precede producer-local validation"
            );
        }
    }

    #[test]
    fn ten_bc1_unsupported_and_malformed_shapes_remain_explicit_blockers() {
        let cases = [
            (
                "missing-module.hum",
                r#"task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a + b
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "zero-parameters.hum",
                r#"module controls.zero
task add() -> Int {
  cost:
    allocates: nothing
  does:
    return 1
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "one-parameter.hum",
                r#"module controls.one
task add(a: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a + a
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "three-parameters.hum",
                r#"module controls.three
task add(a: Int, b: Int, c: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a + b
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "no-does.hum",
                r#"module controls.no_does
task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
}
"#,
                "canonical_backend_does_section_absent_v0",
                false,
            ),
            (
                "two-statements.hum",
                r#"module controls.two_statements
task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a + b
    return a + b
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "subtract.hum",
                r#"module controls.subtract
task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a - b
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "literal.hum",
                r#"module controls.literal
task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return 1
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "reversed-binders.hum",
                r#"module controls.reversed
task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return b + a
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "repeated-first-binder.hum",
                r#"module controls.repeated
task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a + a
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "permission-wrapper.hum",
                r#"module controls.permission
task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return borrow a
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "nested-add.hum",
                r#"module controls.nested
task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a + (b + a)
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "malformed-expression.hum",
                r#"module controls.malformed
task add(a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a +
}
"#,
                "canonical_backend_signature_capability_absent_v0",
                false,
            ),
            (
                "test-item.hum",
                r#"module controls.test_item
test add(a: Int, b: Int) {
  does:
    return a + b
}
"#,
                "canonical_backend_item_kind_unsupported_v0",
                false,
            ),
            (
                "store-item.hum",
                r#"module controls.store_item
store add: Int {
}
"#,
                "canonical_backend_item_kind_unsupported_v0",
                false,
            ),
            (
                "app-item.hum",
                r#"module controls.app_item
app add {
}
"#,
                "canonical_backend_item_kind_unsupported_v0",
                false,
            ),
        ];
        assert_eq!(cases.len(), 16);
        for (path, source, exact_blocker, expects_source_error) in cases {
            let parsed = parse_source(path, source);
            assert_eq!(
                !parsed.diagnostics.is_empty(),
                expects_source_error,
                "unexpected source outcome for {path}: {:?}",
                parsed.diagnostics
            );
            let diagnostics = parsed.diagnostics;
            let program = Program {
                files: vec![parsed.file],
            };
            let json = ir_readiness_json(&program, &diagnostics);
            let text = ir_readiness_text(&program, &diagnostics);
            assert!(
                !json.contains(
                    "\"status\": \"canonical_checked_add_producer_closure_available_v0\""
                ),
                "unsupported or malformed case became accepted: {path}"
            );
            assert!(
                json.contains(&format!(
                    "\"checked_add_producer_blocker\": \"{exact_blocker}\""
                )),
                "unsupported or malformed case missed exact JSON blocker: {path}\n{json}"
            );
            assert!(
                text.contains(exact_blocker),
                "unsupported or malformed case missed exact human blocker: {path}"
            );
        }

        let downstream = [
            (
                "wrong-type.hum",
                r#"module controls.wrong_type
task add(a: UInt, b: UInt) -> UInt {
  cost:
    allocates: nothing
  does:
    return a + b
}
"#,
                "canonical_backend_full_type_exact_statement_not_accepted_v0",
            ),
            (
                "non-default-permission.hum",
                r#"module controls.permission
task add(change a: Int, b: Int) -> Int {
  cost:
    allocates: nothing
  does:
    return a + b
}
"#,
                "canonical_backend_ownership_not_accepted_v0",
            ),
            (
                "non-pure-effect.hum",
                r#"module controls.effect
task add(a: Int, b: Int) -> Int {
  changes:
    file state
  cost:
    allocates: nothing
  does:
    return a + b
}
"#,
                "canonical_backend_effect_not_accepted_v0",
            ),
            (
                "missing-allocation.hum",
                r#"module controls.missing_allocation
task add(a: Int, b: Int) -> Int {
  does:
    return a + b
}
"#,
                "canonical_backend_resource_item_blocked_v0",
            ),
            (
                "wrong-allocation.hum",
                r#"module controls.wrong_allocation
task add(a: Int, b: Int) -> Int {
  cost:
    allocates: one value
  does:
    return a + b
}
"#,
                "canonical_backend_resource_missing_allocation_free_claim_v0",
            ),
            (
                "non-normal-profile.hum",
                r#"module controls.profile
task add(a: Int, b: Int) -> Int {
  profiles:
    containerized_service
  cost:
    allocates: nothing
  does:
    return a + b
}
"#,
                "canonical_backend_profile_item_blocked_v0",
            ),
        ];
        assert_eq!(downstream.len(), 6);
        for (path, source, exact_blocker) in downstream {
            let parsed = parse_source(path, source);
            assert!(
                parsed.diagnostics.is_empty(),
                "{path}: {:?}",
                parsed.diagnostics
            );
            let diagnostics = parsed.diagnostics;
            let program = Program {
                files: vec![parsed.file],
            };
            let item = &program.files[0].items[0];
            let Item::Task(task) = item else {
                panic!("downstream control must be a task")
            };
            assert_eq!(
                program
                    .canonical_backend_function_expectation(task)
                    .and_then(|expectation| expectation.validate())
                    .map(|_| ()),
                Ok(()),
                "{path} must retain real parser/F4 authority before its owning producer"
            );
            assert_eq!(
                super::canonical_checked_add_producer_closure(&program, &diagnostics, item, task,)
                    .map(|_| ()),
                Err(exact_blocker),
                "{path} missed its exact owning downstream producer"
            );
            let json = ir_readiness_json(&program, &diagnostics);
            let text = ir_readiness_text(&program, &diagnostics);
            assert!(
                json.contains(&format!(
                    "\"checked_add_producer_blocker\": \"{exact_blocker}\""
                )),
                "{path} missed exact JSON downstream blocker"
            );
            assert!(
                text.contains(exact_blocker),
                "{path} missed exact human downstream blocker"
            );
        }
    }

    #[test]
    fn text_report_lists_lowering_candidates_without_emitting_ir() {
        let program = demo_program();
        let text = ir_readiness_text(&program, &[]);

        assert!(text.contains("Hum IR readiness (hum.ir_readiness.v0)"));
        assert!(text.contains("core_contract_schema: hum.core_contract.v0"));
        assert!(text.contains("resolver: schema=hum.resolve.v0 status=checked_resolver_v0"));
        assert!(text.contains("lowering_candidates=4 ready_for_ir=0 blocked=4"));
        assert!(text.contains("body_grammar_candidates=2"));
        assert!(
            text.contains(
                "type_check_status=declaration_annotations_and_trivial_returns_checked_v0"
            )
        );
        assert!(text.contains("type_errors=0 unknown_type_references=0"));
        assert!(text.contains("type_check: schema=hum.type_check.v0"));
        assert!(text.contains("core_preview: schema=hum.core_preview.v0"));
        assert!(text.contains("core_lower: schema=hum.core_lower.v0"));
        assert!(text.contains("core_verify: schema=hum.core_verify.v0"));
        assert!(text.contains("typed_expression_previews=1"));
        assert!(text.contains("pass_status:"));
        assert!(text.contains("body_grammar [partial_v0]"));
        assert!(text.contains("core_preview [preview_v0]"));
        assert!(text.contains("type_check [declaration_and_trivial_return_check_available]"));
        assert!(text.contains("core_lowering [unverified_core_artifact_v0]"));
        assert!(text.contains("core_verify [verified_non_executing_core_artifact_v0]"));
        assert!(text.contains("task `add_task`"));
        assert!(text.contains("missing_passes: full_type_check"));
        assert!(text.contains("effect_check"));
        assert!(text.contains("resource_check: schema=hum.resource_check.v0"));
    }

    #[test]
    fn json_report_lists_facts_and_blockers() {
        let program = demo_program();
        let json = ir_readiness_json(&program, &[]);

        assert!(json.contains("\"schema\": \"hum.ir_readiness.v0\""));
        assert!(json.contains("\"core_contract_schema\": \"hum.core_contract.v0\""));
        assert!(json.contains("\"ir_contract_schema\": \"hum.ir_contract.v0\""));
        assert!(json.contains("\"resolver\""));
        assert!(json.contains("\"schema\": \"hum.resolve.v0\""));
        assert!(json.contains("\"status\": \"checked_resolver_v0\""));
        assert!(json.contains("\"mode\": \"source_analysis_only_no_type_or_borrow_check\""));
        assert!(json.contains("\"resolver_errors\": 0"));
        assert!(json.contains("\"type_check\""));
        assert!(json.contains("\"schema\": \"hum.type_check.v0\""));
        assert!(json.contains("\"core_preview\""));
        assert!(json.contains("\"schema\": \"hum.core_preview.v0\""));
        assert!(json.contains("\"status\": \"preview_v0\""));
        assert!(json.contains("\"typed_expression_previews\": 1"));
        assert!(
            json.contains("\"status\": \"declaration_annotations_and_trivial_returns_checked_v0\"")
        );
        assert!(json.contains("\"type_errors\": 0"));
        assert!(json.contains("\"unknown_type_references\": 0"));
        assert!(json.contains("\"checked_resolver_v0\""));
        assert!(json.contains("\"type_check_summary_v0\""));
        assert!(json.contains("\"core_preview_summary_v0\""));
        assert!(json.contains("\"core_lower_summary_v0\""));
        assert!(json.contains("\"core_verify_summary_v0\""));
        assert!(json.contains("\"full_type_check_summary_v0\""));
        assert!(json.contains("\"effect_check_summary_v0\""));
        assert!(json.contains("\"resource_check_summary_v0\""));
        assert!(json.contains("\"profile_check_summary_v0\""));
        assert!(json.contains("\"schema\": \"hum.profile_check.v0\""));
        assert!(json.contains("\"schema\": \"hum.resource_check.v0\""));
        assert!(json.contains("\"recognized_core_resource_gate_available_v0\""));
        assert!(json.contains("\"unverified_core_artifact_rows_v0\""));
        assert!(json.contains("\"verified_core_artifact_rows_v0\""));
        assert!(json.contains("\"checked_return_expression_type_slots_v0\""));
        assert_eq!(
            json.matches("checked_return_expression_type_slots_v0")
                .count(),
            1
        );
        assert!(json.contains("\"declaration_annotations_and_trivial_returns_checked_v0\""));
        assert!(json.contains("\"ready_for_ir\": 0"));
        assert!(json.contains("\"body_grammar_candidates\": 2"));
        assert!(json.contains("\"body_grammar_unsupported_lines\": 1"));
        assert!(json.contains("\"status\": \"blocked_by_full_type_check_errors\""));
        assert!(!json.contains("\"allocation_resource_check_not_implemented\""));
        assert!(json.contains("\"recognized_core_profile_gate_available_v0\""));
        assert!(!json.contains("\"profile_check_not_implemented\""));
        assert!(json.contains("\"ir_verify_not_implemented\""));
        assert!(!json.contains("\"ownership_alias_check_not_implemented\""));
        assert!(json.contains("\"name\": \"body_grammar\""));
        assert!(json.contains("\"name\": \"core_preview\""));
        assert!(json.contains("\"core_lower\""));
        assert!(json.contains("\"schema\": \"hum.core_lower.v0\""));
        assert!(json.contains("\"core_verify\""));
        assert!(json.contains("\"schema\": \"hum.core_verify.v0\""));
        assert!(json.contains("\"mode\": \"non_executing_artifact_invariant_check_v0\""));
        assert!(json.contains("\"name\": \"core_lowering\""));
        assert!(json.contains("\"name\": \"core_verify\""));
        assert!(json.contains("\"status\": \"partial_v0\""));
        assert!(json.contains("\"body_grammar\""));
        assert!(json.contains("\"kind\": \"return\""));
        assert!(json.contains("\"reason\": \"surface_save_requires_store_lowering\""));
        assert!(json.contains("\"body_grammar_partial_v0\""));
        assert!(json.contains("\"name\": \"semantic_graph_build\""));
        assert!(json.contains("\"name\": \"type_check\""));
        assert!(json.contains("\"status\": \"declaration_and_trivial_return_check_available\""));
        assert!(json.contains("\"core_verify\""));
        assert!(json.contains("\"full_type_check\""));
        assert!(json.contains("\"effect_check\""));
        assert!(json.contains("\"schema\": \"hum.full_type_check.v0\""));
        assert!(json.contains("\"recognized_core_body_type_gate_available_v0\""));
        assert!(json.contains("\"recognized_core_effect_gate_available_v0\""));
        assert!(json.contains("\"full_type_check_errors\""));
        assert!(json.contains("\"status\": \"report_available_not_ir_pass\""));
        assert!(json.contains("\"effect_hints\""));
        assert!(json.contains("\"contract_hints\""));
        assert!(json.contains("\"body_text_captured\""));
        assert!(json.contains("\"no IR emission\""));
    }

    #[test]
    fn json_blocks_on_type_errors_before_lowering() {
        let source = r#"type Box {
  value: MissingType
}

task pass_box(item: Box) -> Box {
  does:
    return item
}
"#;
        let parsed = parse_source("bad_type.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = ir_readiness_json(&program, &[]);

        assert!(json.contains("\"type_check\""));
        assert!(json.contains("\"schema\": \"hum.type_check.v0\""));
        assert!(json.contains("\"core_preview\""));
        assert!(json.contains("\"schema\": \"hum.core_preview.v0\""));
        assert!(json.contains("\"status\": \"preview_v0\""));
        assert!(json.contains("\"typed_expression_previews\": 0"));
        assert!(json.contains("\"status\": \"type_errors_v0\""));
        assert!(json.contains("\"type_errors\": 1"));
        assert!(json.contains("\"unknown_type_references\": 1"));
        assert!(json.contains("\"status\": \"blocked_by_type_errors\""));
        assert!(json.contains("\"type_check_errors\""));
        assert!(json.contains("\"full_type_check_errors\""));
        assert!(json.contains("\"ready_for_ir\": 0"));
    }
    #[test]
    fn json_blocks_on_resolver_errors_before_lowering() {
        let source = r#"task bad_names() -> UInt {
  does:
    return missing
}
"#;
        let parsed = parse_source("bad.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = ir_readiness_json(&program, &[]);

        assert!(json.contains("\"status\": \"checked_resolver_with_errors_v0\""));
        assert!(json.contains("\"resolver_errors\": 1"));
        assert!(json.contains("\"status\": \"blocked_by_resolver_errors\""));
        assert!(json.contains("\"checked_resolver_errors\""));
        assert!(json.contains("\"ready_for_ir\": 0"));
    }

    fn demo_program() -> Program {
        let source = r#"type Task {
  title: Text
}

store tasks: list Task {
  why:
    remember tasks
}

task add_task(title: Text) -> Task {
  why:
    save a task

  changes:
    tasks

  ensures:
    task is visible

  does:
    let item = Task {
      title: title
    }
    save item in tasks
    return item
}

test add_task is visible {
  covers:
    add_task ensures task is visible

  does:
    expect task is visible
}
"#;
        let parsed = parse_source("demo.hum", source);
        Program {
            files: vec![parsed.file],
        }
    }
}
