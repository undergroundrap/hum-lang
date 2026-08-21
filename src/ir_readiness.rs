use crate::ast::{
    App, CanonicalExpressionKind, Item, ParsedBinaryOperator, ParsedBodyStatement,
    ParsedBodyStatementKind, Program, Section, Store, Task, Test, TypeDef,
};
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
use crate::ir_verify;
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
    ir_ready: usize,
    ready_for_ir: usize,
    backend_ready: usize,
    backend_blocking_reasons: Vec<&'static str>,
    section_names: Vec<String>,
    body_grammar: Option<BodyGrammarReport>,
}

#[allow(unexpected_cfgs)]
mod removed_raw_backend_authority_surface {
    #[cfg(hum_compile_fail_canonical_minimal_add_backend_facts_escape)]
    pub(crate) use crate::backend_input::CanonicalMinimalAddBackendFactsAccess;

    #[allow(dead_code)]
    #[cfg(hum_compile_fail_canonical_minimal_add_backend_facts_escape)]
    pub(crate) fn with_canonical_minimal_add_backend_facts<R>(
        program: &super::Program,
        diagnostics: &[super::Diagnostic],
        item: &super::Item,
        statement: &super::ParsedBodyStatement,
        consume: impl for<'facts> FnOnce(CanonicalMinimalAddBackendFactsAccess<'facts>) -> R,
    ) -> Option<R> {
        crate::backend_input::with_canonical_minimal_add_backend_facts(
            program,
            diagnostics,
            item,
            statement,
            consume,
        )
    }
}

#[allow(unexpected_cfgs)]
mod canonical_minimal_add_backend_facts_escape_compile_proof {
    #[cfg(hum_compile_fail_canonical_minimal_add_backend_facts_escape)]
    mod enabled {
        use super::super::removed_raw_backend_authority_surface::{
            CanonicalMinimalAddBackendFactsAccess, with_canonical_minimal_add_backend_facts,
        };
        use crate::{
            ast::{Item, ParsedBodyStatement, Program},
            diagnostic::Diagnostic,
        };

        type StaticAccess = CanonicalMinimalAddBackendFactsAccess<'static>;

        fn backend_facts_return_escape_must_not_compile_value(
            access: StaticAccess,
        ) -> StaticAccess {
            access
        }

        fn backend_facts_return_escape_must_not_compile(
            program: &Program,
            diagnostics: &[Diagnostic],
            item: &Item,
            statement: &ParsedBodyStatement,
        ) -> StaticAccess {
            with_canonical_minimal_add_backend_facts(
                program,
                diagnostics,
                item,
                statement,
                backend_facts_return_escape_must_not_compile_value,
            )
            .unwrap()
        }

        fn backend_facts_static_escape_must_not_compile(
            program: &Program,
            diagnostics: &[Diagnostic],
            item: &Item,
            statement: &ParsedBodyStatement,
        ) {
            let mut backend_facts_static_escape_must_not_compile: Option<StaticAccess> = None;
            let _ = with_canonical_minimal_add_backend_facts(
                program,
                diagnostics,
                item,
                statement,
                |access| backend_facts_static_escape_must_not_compile = Some(access),
            );
        }

        fn backend_facts_collection_escape_must_not_compile(
            program: &Program,
            diagnostics: &[Diagnostic],
            item: &Item,
            statement: &ParsedBodyStatement,
        ) {
            let mut backend_facts_collection_escape_must_not_compile: Vec<StaticAccess> = vec![];
            let _ = with_canonical_minimal_add_backend_facts(
                program,
                diagnostics,
                item,
                statement,
                |access| backend_facts_collection_escape_must_not_compile.push(access),
            );
        }

        fn backend_facts_foreign_construction_must_not_compile(access: StaticAccess) {
            let backend_facts_foreign_construction_must_not_compile =
                CanonicalMinimalAddBackendFactsAccess {
                    facts: access.facts,
                };
            let _ = backend_facts_foreign_construction_must_not_compile;
        }
    }
}

#[allow(unexpected_cfgs)]
mod verified_backend_input_authority_compile_proof {
    #[cfg(hum_compile_fail_verified_backend_input_authority)]
    mod enabled {
        use crate::{
            ast::{Item, ParsedBodyStatement, Program},
            backend_input::CanonicalBackendInputArtifact,
            diagnostic::Diagnostic,
            ir_verify::{IrVerifyReport, LiveIdentityRequest, VerifiedBackendInput},
        };
        use std::marker::PhantomData;

        fn consume_verified_backend_input(_: VerifiedBackendInput<'_>) {}

        fn verified_backend_input_private_construction_must_not_compile() {
            let _ = VerifiedBackendInput {
                projection: None.unwrap(),
                _artifact: PhantomData,
            };
        }

        fn verified_backend_input_private_field_access_must_not_compile(
            value: VerifiedBackendInput<'_>,
        ) {
            let _ = value.projection;
        }

        fn verified_backend_input_fabricated_conversion_must_not_compile(bytes: &[u8]) {
            let _: VerifiedBackendInput<'_> = bytes.into();
        }

        fn raw_artifact_substitution_must_not_compile(value: CanonicalBackendInputArtifact) {
            consume_verified_backend_input(value);
        }

        fn raw_report_substitution_must_not_compile(value: IrVerifyReport) {
            consume_verified_backend_input(value);
        }

        fn raw_fixture_substitution_must_not_compile(value: &[u8]) {
            consume_verified_backend_input(value);
        }

        fn verified_backend_input_lifetime_escape_must_not_compile(
            program: &Program,
            diagnostics: &[Diagnostic],
            artifact: &[u8],
        ) -> VerifiedBackendInput<'static> {
            let mut escaped = None;
            let _ = crate::ir_verify::with_verified_backend_input(
                program,
                diagnostics,
                artifact,
                |value| escaped = Some(value),
            );
            escaped.unwrap()
        }

        fn sibling_raw_facts_type_must_not_compile(
            _: crate::backend_input::CanonicalMinimalAddBackendFacts<'static, 'static>,
        ) {
        }

        fn sibling_live_request_construction_must_not_compile() {
            let _ = LiveIdentityRequest {
                expected: None.unwrap(),
                observed: None,
                program_identity: 0,
            };
        }

        fn sibling_raw_authority_call_without_request_must_not_compile(
            program: &Program,
            diagnostics: &[Diagnostic],
            _item: &Item,
            _statement: &ParsedBodyStatement,
        ) {
            let _ = crate::backend_input::bind_canonical_minimal_add_live_identity(
                &mut (),
                program,
                diagnostics,
            );
        }
    }
}

#[allow(unexpected_cfgs)]
mod verified_backend_input_construction_compile_proof {
    #[cfg(hum_compile_fail_verified_backend_input_construction)]
    mod enabled {
        use crate::ir_verify::{LiveIdentityRequest, VerifiedBackendInput};
        use std::marker::PhantomData;

        fn private_construction_must_not_compile() {
            let _ = VerifiedBackendInput {
                projection: None.unwrap(),
                _artifact: PhantomData,
            };
        }

        fn private_live_request_must_not_compile() {
            let _ = LiveIdentityRequest {
                expected: None.unwrap(),
                observed: None,
                program_identity: 0,
            };
        }
    }
}

#[allow(unexpected_cfgs)]
mod verified_minimal_add_wrapper_construction_compile_proof {
    #[cfg(any(
        hum_compile_fail_canonical_minimal_add_backend_facts_escape,
        hum_compile_fail_verified_minimal_add_full_type_construction,
        hum_compile_fail_verified_minimal_add_effect_construction,
        hum_compile_fail_verified_minimal_add_ownership_construction,
        hum_compile_fail_verified_minimal_add_resource_construction,
        hum_compile_fail_verified_minimal_add_profile_construction
    ))]
    mod enabled {
        #[cfg(any(
            hum_compile_fail_canonical_minimal_add_backend_facts_escape,
            hum_compile_fail_verified_minimal_add_full_type_construction
        ))]
        fn verified_minimal_add_full_type_sibling_construction_must_not_compile(
            verified_type: crate::core_verify::VerifiedCanonicalMinimalAddTypeResult<'_>,
        ) {
            let verified_minimal_add_full_type_sibling_construction_must_not_compile =
                crate::full_type_check::VerifiedMinimalAddFullType(verified_type);
            let _ = verified_minimal_add_full_type_sibling_construction_must_not_compile;
        }

        #[cfg(any(
            hum_compile_fail_canonical_minimal_add_backend_facts_escape,
            hum_compile_fail_verified_minimal_add_effect_construction
        ))]
        fn verified_minimal_add_effect_sibling_construction_must_not_compile(
            full_type: crate::full_type_check::VerifiedMinimalAddFullType<'_>,
        ) {
            let verified_minimal_add_effect_sibling_construction_must_not_compile =
                crate::effect_check::VerifiedMinimalAddEffect(full_type);
            let _ = verified_minimal_add_effect_sibling_construction_must_not_compile;
        }

        #[cfg(any(
            hum_compile_fail_canonical_minimal_add_backend_facts_escape,
            hum_compile_fail_verified_minimal_add_ownership_construction
        ))]
        fn verified_minimal_add_ownership_sibling_construction_must_not_compile(
            effect: crate::effect_check::VerifiedMinimalAddEffect<'_>,
        ) {
            let verified_minimal_add_ownership_sibling_construction_must_not_compile =
                crate::ownership_check::VerifiedMinimalAddOwnership(effect);
            let _ = verified_minimal_add_ownership_sibling_construction_must_not_compile;
        }

        #[cfg(any(
            hum_compile_fail_canonical_minimal_add_backend_facts_escape,
            hum_compile_fail_verified_minimal_add_resource_construction
        ))]
        fn verified_minimal_add_resource_sibling_construction_must_not_compile(
            ownership: crate::ownership_check::VerifiedMinimalAddOwnership<'_>,
        ) {
            let verified_minimal_add_resource_sibling_construction_must_not_compile =
                crate::resource_check::VerifiedMinimalAddResource(todo!());
            let _ = ownership;
            let _ = verified_minimal_add_resource_sibling_construction_must_not_compile;
        }

        #[cfg(any(
            hum_compile_fail_canonical_minimal_add_backend_facts_escape,
            hum_compile_fail_verified_minimal_add_profile_construction
        ))]
        fn verified_minimal_add_profile_sibling_construction_must_not_compile(
            resource: crate::resource_check::VerifiedMinimalAddResource<'_>,
        ) {
            let verified_minimal_add_profile_sibling_construction_must_not_compile =
                crate::profile_check::VerifiedMinimalAddProfile(todo!());
            let _ = resource;
            let _ = verified_minimal_add_profile_sibling_construction_must_not_compile;
        }
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
        status: "implemented_canonical_minimal_add_backend_input_v0",
        source: ir_verify::IR_VERIFY_SCHEMA,
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
const MISSING_AFTER_PROFILE_PASSES: &[&str] = &["ir_verify"];

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
        "summary: files={} items={} tasks={} tests={} lowering_candidates={} ir_ready={} ready_for_ir={} backend_ready=0 blocked={} errors={} warnings={} body_grammar_candidates={} body_grammar_recognized_lines={} body_grammar_unsupported_lines={} resolver_status={} resolver_errors={} unresolved_references={} type_check_status={} type_errors={} unknown_type_references={} checked_returns={} rejected_returns={} unchecked_returns={}\n",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.candidates.len(),
        report.ready_count(),
        report.ready_count(),
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
        out.push_str(&format!("    ir_ready: {}\n", candidate.ir_ready));
        out.push_str(&format!("    ready_for_ir: {}\n", candidate.ready_for_ir));
        out.push_str(&format!("    backend_ready: {}\n", candidate.backend_ready));
        out.push_str(&format!(
            "    backend_blocking_reasons: {}\n",
            candidate.backend_blocking_reasons.join(", ")
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
    let diagnostic_transport = profile_check::diagnostic_transport(program, diagnostics)
        .expect("profile check must supply IR and graph occurrence projections");
    let diagnostic_occurrences = diagnostic_transport.authoritative();
    profile_check::validate_prior_blocker_projection(program, diagnostics)
        .expect("static prior-blocker projection must preserve exact occurrence identity");
    diagnostic_transport
        .ir_projection()
        .validate_against("ir_readiness", diagnostic_occurrences)
        .expect("IR readiness must consume the separately carried profile projection");
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
    let blockers = CandidateBlockers {
        has_errors,
        has_resolver_errors,
        has_type_errors,
        has_core_verify_errors,
        has_full_type_check_errors,
        has_effect_check_errors,
        has_ownership_check_errors,
        has_resource_check_errors,
        has_profile_check_errors,
    };
    let section_names = item_sections(item)
        .iter()
        .map(|section| section.name.clone())
        .collect::<Vec<_>>();
    let body_grammar = body_grammar_for_item(context.program, item);
    let backend_facts = candidate_backend_facts(item, context);
    let backend_verified = backend_facts;
    let blocking_reasons = blocking_reasons(blockers, backend_verified);

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
        } else if backend_verified {
            "ready_for_ir_with_verified_backend_input_v0"
        } else {
            "blocked_before_ir_verify"
        },
        current_layer: CURRENT_LAYER,
        target_layer: TARGET_LAYER,
        facts_available: facts_available(item, context, backend_facts),
        missing_passes: if backend_verified {
            Vec::new()
        } else if has_full_type_check_errors {
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
        ir_ready: usize::from(backend_verified),
        ready_for_ir: usize::from(backend_verified),
        backend_ready: 0,
        backend_blocking_reasons: backend_verified
            .then_some(vec!["backend_adapter_not_implemented"])
            .unwrap_or_default(),
        section_names,
        body_grammar,
    }
}

fn candidate_backend_facts(item: &Item, context: &CandidateContext<'_>) -> bool {
    let Item::Task(task) = item else {
        return false;
    };
    let mut statements = task.body_syntax.iter().filter(|statement| {
        matches!(
            &statement.kind,
            ParsedBodyStatementKind::Return(expression)
                if matches!(
                    expression.canonical.kind,
                    CanonicalExpressionKind::Binary {
                        operator: ParsedBinaryOperator::Add,
                        ..
                    }
                )
        )
    });
    let Some(statement): Option<&ParsedBodyStatement> = statements.next() else {
        return false;
    };
    let _ = statement;
    if statements.next().is_some() {
        return false;
    }
    crate::backend_input::canonical_minimal_add_artifact(context.program, context.diagnostics)
        .is_some_and(|artifact| {
            let (report, observed) = ir_verify::with_verified_backend_input(
                context.program,
                context.diagnostics,
                artifact.bytes(),
                |verified| {
                    let parameter_ids = verified.parameter_value_ids();
                    let definition_ids = verified.parameter_definition_ids();
                    let parameter_types = verified.parameter_types();
                    let parameter_spans = verified.parameter_spans();
                    let (linkage_kind, linkage_symbol) = verified.linkage();
                    let (result_id, result_type) = verified.result();
                    let (overflow_type, overflow_operation, overflow_behavior) =
                        verified.overflow_edge();
                    verified.schema() == crate::backend_input::BACKEND_INPUT_SCHEMA
                        && verified.artifact_id() == artifact.artifact_id()
                        && verified.compiler_version() == crate::version::HUM_VERSION
                        && verified.semantic_contract() == crate::backend_input::SEMANTIC_CONTRACT
                        && verified.target_context() == crate::backend_input::TARGET_CONTEXT
                        && verified.source_revision()
                            == crate::backend_input::SOURCE_REVISION_SHA256
                        && verified.source_path() == crate::backend_input::SOURCE_PATH
                        && verified.function_id() == "function:0"
                        && linkage_kind == "internal"
                        && !linkage_symbol.is_empty()
                        && parameter_ids[0] != parameter_ids[1]
                        && definition_ids[0] != definition_ids[1]
                        && parameter_types == ["type:int64", "type:int64"]
                        && parameter_spans
                            .iter()
                            .all(|(line, column)| *line > 0 && *column > 0)
                        && !verified.operation_id().is_empty()
                        && !result_id.is_empty()
                        && result_type == "type:int64"
                        && verified.operation_span().0 > 0
                        && overflow_type == "signed_64"
                        && overflow_operation == "checked_add"
                        && overflow_behavior == "runtime_trap_on_overflow"
                        && verified.profile() == "normal"
                        && verified.required_passes().count() == 14
                },
            );
            report.accepted() && observed == Some(true)
        })
}

fn facts_available(
    item: &Item,
    context: &CandidateContext<'_>,
    backend_facts: bool,
) -> Vec<&'static str> {
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

    if backend_facts {
        facts.extend([
            "canonical_minimal_add_backend_facts_v0",
            "source_and_operation_identity_bound_v0",
            "ordered_resolver_bindings_bound_v0",
            "verified_checked_type_bound_v0",
            "effect_checked_empty_v0",
            "ownership_checked_empty_v0",
            "resource_checked_empty_v0",
            "normal_profile_checked_v0",
            "checked_i64_overflow_trap_bound_v0",
            "canonical_backend_input_bytes_v0",
            "sha256_payload_identity_verified_v0",
            "ir_verify_passed_v0",
            "verified_backend_input_capability_lent_v0",
        ]);
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

fn blocking_reasons(blockers: CandidateBlockers, backend_verified: bool) -> Vec<&'static str> {
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
    if !backend_verified {
        reasons.push("canonical_backend_input_not_verified_v0");
    }
    reasons
}

impl IrReadinessReport {
    fn ready_count(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.ir_ready)
            .sum()
    }
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
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"lowering_candidates\": {}, \"ir_ready\": {}, \"ready_for_ir\": {}, \"backend_ready\": 0, \"blocked\": {}, \"errors\": {}, \"warnings\": {}, \"type_errors\": {}, \"unknown_type_references\": {}, \"checked_returns\": {}, \"rejected_returns\": {}, \"unchecked_returns\": {}, \"body_grammar_candidates\": {}, \"body_grammar_recognized_lines\": {}, \"body_grammar_unsupported_lines\": {}",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.candidates.len(),
        report.ready_count(),
        report.ready_count(),
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
    push_usize_field(out, indent + 2, "ir_ready", candidate.ir_ready, true);
    push_usize_field(
        out,
        indent + 2,
        "ready_for_ir",
        candidate.ready_for_ir,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "backend_ready",
        candidate.backend_ready,
        true,
    );
    push_string_array(
        out,
        indent + 2,
        "backend_blocking_reasons",
        &candidate.backend_blocking_reasons,
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
    use crate::ast::Program;
    use crate::parser::parse_source;

    use super::{ir_readiness_json, ir_readiness_text};

    #[test]
    fn text_report_lists_lowering_candidates_without_emitting_ir() {
        let program = demo_program();
        let text = ir_readiness_text(&program, &[]);

        assert!(text.contains("Hum IR readiness (hum.ir_readiness.v0)"));
        assert!(text.contains("core_contract_schema: hum.core_contract.v0"));
        assert!(text.contains("resolver: schema=hum.resolve.v0 status=checked_resolver_v0"));
        assert!(
            text.contains(
                "lowering_candidates=4 ir_ready=0 ready_for_ir=0 backend_ready=0 blocked=4"
            )
        );
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
        assert!(json.contains("\"implemented_canonical_minimal_add_backend_input_v0\""));
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

    #[test]
    fn canonical_minimal_add_is_ir_ready_only_after_live_verification() {
        type PublicReports = [(&'static str, String); 10];

        fn subject(source: &str) -> (Program, Vec<crate::diagnostic::Diagnostic>) {
            let parsed = parse_source("examples/core/minimal_add.hum", source);
            (
                Program {
                    files: vec![parsed.file],
                },
                parsed.diagnostics,
            )
        }

        fn access(
            program: &Program,
            diagnostics: &[crate::diagnostic::Diagnostic],
        ) -> Option<Vec<String>> {
            let item = &program.files[0].items[0];
            let crate::ast::Item::Task(task) = item else {
                return None;
            };
            let _ = &task.body_syntax[0];
            let artifact =
                crate::backend_input::canonical_minimal_add_artifact(program, diagnostics)?;
            let text = std::str::from_utf8(artifact.bytes()).ok()?;
            let checked_empty = [
                ("effects", "\"effects\":[]"),
                ("ownership_transfers", "\"ownership_transfers\":[]"),
                ("allocations", "\"allocations\":[]"),
                ("contract_predicates", "\"contract_predicates\":[]"),
                ("evidence_obligations", "\"evidence_obligations\":[]"),
                ("unsupported_or_weakened", "\"unsupported\":[]"),
                ("external_authority", "\"external_authority\":[]"),
            ];
            let (report, snapshot) = crate::ir_verify::with_verified_backend_input(
                program,
                diagnostics,
                artifact.bytes(),
                |verified| {
                    let mut snapshot = verified
                        .required_passes()
                        .map(str::to_string)
                        .collect::<Vec<_>>();
                    snapshot.extend(checked_empty.map(|(name, needle)| {
                        assert!(text.contains(needle), "missing checked-empty {name}");
                        format!("checked_empty:{name}")
                    }));
                    snapshot.push(verified.overflow_edge().1.to_string());
                    snapshot
                },
            );
            report.accepted().then_some(snapshot).flatten()
        }

        fn public_reports(
            program: &Program,
            diagnostics: &[crate::diagnostic::Diagnostic],
        ) -> PublicReports {
            [
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
            ]
        }

        fn assert_public_chain_unblocked(
            program: &Program,
            diagnostics: &[crate::diagnostic::Diagnostic],
        ) {
            assert_eq!(
                crate::full_type_check::full_type_check_summary(program, diagnostics)
                    .blocking_issues,
                0
            );
            assert_eq!(
                crate::effect_check::effect_check_summary(program, diagnostics).blocking_issues,
                0
            );
            assert_eq!(
                crate::ownership_check::ownership_check_summary(program, diagnostics)
                    .blocking_issues,
                0
            );
            assert_eq!(
                crate::resource_check::resource_check_summary(program, diagnostics).blocking_issues,
                0
            );
            assert_eq!(
                crate::profile_check::profile_check_summary(program, diagnostics).blocking_issues,
                0
            );
        }

        fn foreign_final_profile_lineage(
            honest_program: &Program,
            honest_diagnostics: &[crate::diagnostic::Diagnostic],
            foreign_program: &Program,
            foreign_diagnostics: &[crate::diagnostic::Diagnostic],
        ) -> (Option<Vec<String>>, usize) {
            let honest_item = &honest_program.files[0].items[0];
            let crate::ast::Item::Task(honest_task) = honest_item else {
                panic!("honest subject must be a task")
            };
            let foreign_item = &foreign_program.files[0].items[0];
            let crate::ast::Item::Task(foreign_task) = foreign_item else {
                panic!("foreign subject must be a task")
            };
            crate::profile_check::with_profile_for_ir_readiness(
                honest_program,
                honest_diagnostics,
                |honest_access| {
                    let honest_profile = honest_access
                        .canonical_minimal_add_for(honest_item, &honest_task.body_syntax[0])
                        .expect("honest actual profile authority");
                    crate::profile_check::with_profile_for_ir_readiness(
                        foreign_program,
                        foreign_diagnostics,
                        |foreign_access| {
                            let foreign_profile = foreign_access
                                .canonical_minimal_add_for(
                                    foreign_item,
                                    &foreign_task.body_syntax[0],
                                )
                                .expect("foreign actual profile authority");
                            assert_ne!(
                                std::ptr::from_ref(honest_program).addr(),
                                std::ptr::from_ref(foreign_program).addr()
                            );
                            crate::backend_input::issue_with_final_profile_lineage_for_test(
                                honest_program,
                                honest_item,
                                &honest_task.body_syntax[0],
                                honest_profile,
                                &foreign_profile,
                            )
                        },
                    )
                },
            )
        }

        let source = include_str!("../examples/core/minimal_add.hum");
        let (program, diagnostics) = subject(source);
        let snapshot = access(&program, &diagnostics).expect("complete backend facts");
        assert_eq!(
            &snapshot[..14],
            [
                "parse",
                "semantic_graph_build",
                "resolve",
                "body_grammar",
                "core_preview",
                "core_lowering",
                "core_verify",
                "type_check",
                "full_type_check",
                "effect_check",
                "ownership_alias_check",
                "allocation_resource_check",
                "contract_evidence_linking_checked_empty_for_exact_item",
                "profile_check",
            ]
        );
        assert_eq!(
            &snapshot[14..21],
            [
                "checked_empty:effects",
                "checked_empty:ownership_transfers",
                "checked_empty:allocations",
                "checked_empty:contract_predicates",
                "checked_empty:evidence_obligations",
                "checked_empty:unsupported_or_weakened",
                "checked_empty:external_authority",
            ]
        );
        assert_eq!(snapshot.last().map(String::as_str), Some("checked_add"));

        let (foreign_program, foreign_diagnostics) = subject(source);
        assert_public_chain_unblocked(&program, &diagnostics);
        assert_public_chain_unblocked(&foreign_program, &foreign_diagnostics);
        assert_eq!(
            public_reports(&program, &diagnostics),
            public_reports(&foreign_program, &foreign_diagnostics),
            "independent Programs must expose byte-identical public reports"
        );
        assert!(access(&program, &diagnostics).is_some());
        assert!(access(&foreign_program, &foreign_diagnostics).is_some());
        let (foreign_access, comparisons) = foreign_final_profile_lineage(
            &program,
            &diagnostics,
            &foreign_program,
            &foreign_diagnostics,
        );
        assert_eq!(comparisons, 1, "the final comparison must run exactly once");
        assert!(
            foreign_access.is_none(),
            "foreign actual profile lineage must not receive final facts access"
        );
        assert!(access(&program, &diagnostics).is_some(), "honest issuance");

        let text = ir_readiness_text(&program, &diagnostics);
        let json = ir_readiness_json(&program, &diagnostics);
        assert_eq!(text, ir_readiness_text(&program, &diagnostics));
        assert_eq!(json, ir_readiness_json(&program, &diagnostics));
        assert!(text.contains("[ready_for_ir_with_verified_backend_input_v0]"));
        assert!(text.contains("missing_passes: \n"));
        assert!(text.contains("blocking_reasons: \n"));
        assert!(json.contains("\"ir_ready\": 1"));
        assert!(json.contains("\"ready_for_ir\": 1"));
        assert!(json.contains("\"backend_ready\": 0"));
        assert!(json.contains("\"missing_passes\": []"));
        let expected_fact_suffix = [
            "canonical_minimal_add_backend_facts_v0",
            "source_and_operation_identity_bound_v0",
            "ordered_resolver_bindings_bound_v0",
            "verified_checked_type_bound_v0",
            "effect_checked_empty_v0",
            "ownership_checked_empty_v0",
            "resource_checked_empty_v0",
            "normal_profile_checked_v0",
            "checked_i64_overflow_trap_bound_v0",
            "canonical_backend_input_bytes_v0",
            "sha256_payload_identity_verified_v0",
            "ir_verify_passed_v0",
            "verified_backend_input_capability_lent_v0",
        ];
        let expected_human_suffix = expected_fact_suffix.join(", ");
        let human_facts_line = text
            .lines()
            .find(|line| line.contains(expected_fact_suffix[0]))
            .expect("human readiness facts for the canonical minimal add");
        let human_suffix_start = human_facts_line
            .find(expected_fact_suffix[0])
            .expect("human canonical backend-facts suffix");
        assert_eq!(
            &human_facts_line[human_suffix_start..],
            expected_human_suffix,
            "human readiness facts must preserve the exact backend-facts suffix",
        );

        let expected_json_suffix = format!(
            "{}],",
            expected_fact_suffix
                .iter()
                .map(|fact| format!("\"{fact}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let json_facts_line = json
            .lines()
            .find(|line| line.contains(expected_fact_suffix[0]))
            .expect("JSON readiness facts for the canonical minimal add");
        let json_suffix_start = json_facts_line
            .find(&format!("\"{}\"", expected_fact_suffix[0]))
            .expect("JSON canonical backend-facts suffix");
        assert_eq!(
            &json_facts_line[json_suffix_start..],
            expected_json_suffix,
            "JSON readiness facts must preserve the exact backend-facts suffix",
        );
        let json_blocking_reasons = json
            .lines()
            .find(|line| line.contains("\"blocking_reasons\":"))
            .expect("JSON readiness blocker for the canonical minimal add");
        assert_eq!(json_blocking_reasons.trim(), "\"blocking_reasons\": [],",);

        let access_missing = || access(&program, &diagnostics).is_none();
        for corruption in [
            "missing_core_preview",
            "blocked_core_preview",
            "duplicate_core_preview",
            "foreign_core_preview",
            "reordered_core_preview",
        ] {
            crate::core_verify::set_backend_pass_corruption_for_test(corruption);
            assert!(access_missing(), "{corruption}");
        }
        let standard = &[
            "missing",
            "rejected",
            "unchecked",
            "foreign",
            "fabricated",
            "global",
        ];
        for (stage, corruptions) in [
            ("full_type_check", &standard[..]),
            (
                "effect_check",
                &[
                    "missing",
                    "target",
                    "declaration",
                    "rejected",
                    "unchecked",
                    "foreign",
                    "global",
                ][..],
            ),
            (
                "ownership_check",
                &[
                    "missing", "move", "borrow", "alias", "transfer", "rejected", "foreign",
                    "global",
                ][..],
            ),
            ("resource_check", &standard[..]),
            (
                "profile_check",
                &[
                    "missing",
                    "unknown",
                    "strict",
                    "rejected",
                    "fabricated",
                    "foreign",
                    "global",
                ][..],
            ),
        ] {
            for corruption in corruptions {
                crate::type_check::set_wo19_stage_corruption(stage, corruption);
                assert!(access_missing(), "{stage}:{corruption}");
            }
        }
        for pass in 0..14 {
            for corruption in [
                "pass_missing",
                "pass_failed",
                "pass_skipped",
                "pass_zero",
                "pass_duplicate",
                "pass_foreign",
                "pass_reordered",
            ] {
                crate::backend_input::set_corruption_for_test(corruption, pass);
                assert!(access_missing(), "{corruption}:{pass}");
            }
        }
        for state in 0..7 {
            for corruption in [
                "empty_missing",
                "empty_not_checked",
                "empty_unsupported",
                "empty_substituted",
                "empty_corrupted",
                "empty_duplicate",
                "empty_reordered",
            ] {
                crate::backend_input::set_corruption_for_test(corruption, state);
                assert!(access_missing(), "{corruption}:{state}");
            }
        }
        for corruption in [
            "edge_omitted",
            "edge_duplicate",
            "edge_reordered",
            "edge_wrong_type",
            "edge_wraparound",
            "edge_wrong_width",
            "edge_foreign_trap",
        ] {
            crate::backend_input::set_corruption_for_test(corruption, 0);
            assert!(access_missing(), "{corruption}");
        }
        for corrupted in [
            source.replace("  allocates:\n    nothing\n\n", ""),
            source.replace(
                "  allocates:\n    nothing",
                "  allocates:\n    nothing\n    nothing",
            ),
            source.replace("return a + b", "return b + a"),
            source.replace("return a + b", "return a + a"),
            source.replace("  does:", "  ensures:\n    result is positive\n\n  does:"),
            source.replace("  does:", "  protects:\n    add is correct\n\n  does:"),
        ] {
            let (corrupted_program, corrupted_diagnostics) = subject(&corrupted);
            assert!(
                access(&corrupted_program, &corrupted_diagnostics).is_none(),
                "unexpected authority for:\n{corrupted}"
            );
        }
        crate::backend_input::set_artifact_target_context_corruption_for_test(
            "foreign_target_context_v0",
        );
        let rejected = ir_readiness_json(&program, &diagnostics);
        assert!(rejected.contains("\"ir_ready\": 0"));
        assert!(rejected.contains("\"ready_for_ir\": 0"));
        assert!(rejected.contains("\"backend_ready\": 0"));
        assert!(rejected.contains("\"status\": \"blocked_before_ir_verify\""));
        assert!(rejected.contains("\"canonical_backend_input_not_verified_v0\""));
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
