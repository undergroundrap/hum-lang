use crate::diagnostic::{DiagnosticCode, Severity};

pub const DIAGNOSTIC_EXPLAIN_SCHEMA: &str = "hum.diagnostic_explain.v0";
pub const DIAGNOSTIC_CATALOG_SCHEMA: &str = "hum.diagnostic_catalog.v0";
#[cfg(test)]
const UNALLOCATED_PROFILE_DIAGNOSTIC: &str = "<unallocated-profile-diagnostic>";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticCodeKey(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticFamilyKey(&'static str);

impl DiagnosticFamilyKey {
    pub const SOURCE_SHAPE: Self = Self("source_shape");
    pub const INTENT_SHAPE: Self = Self("intent_shape");
    pub const DECLARED_STATE_EFFECTS: Self = Self("declared_state_effects");
    pub const COST_CONTRACTS: Self = Self("cost_contracts");
    pub const SECURITY_TRUST: Self = Self("security_trust");
    pub const TEST_EVIDENCE: Self = Self("test_evidence");
    pub const FRONT_END_SEMANTICS: Self = Self("front_end_semantics");
    pub const EXECUTABLE_CONTRACTS: Self = Self("executable_contracts");
    pub const OWNERSHIP_BORROWING: Self = Self("ownership_borrowing");
    pub const NOMINAL_TYPED_FAILURE: Self = Self("nominal_typed_failure");
    pub const UNSAFE_FFI_PROVENANCE: Self = Self("unsafe_ffi_provenance");
    pub const RUNTIME_PROFILE_POLICY: Self = Self("runtime_profile_policy");
    pub const TARGET_BACKEND_METADATA: Self = Self("target_backend_metadata");
    pub const CONCURRENCY_MEMORY_ORDERING: Self = Self("concurrency_memory_ordering");
    pub const CALLABLE_EFFECT_ROWS: Self = Self("callable_effect_rows");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationStatus {
    Active,
    Reserved,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticFamilySpec {
    pub start: u16,
    pub end: u16,
    pub key: DiagnosticFamilyKey,
    pub semantic_owner: &'static str,
    pub status: AllocationStatus,
    pub doctrine: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticCodeAllocation {
    pub key: DiagnosticCodeKey,
    pub public_ordinal: u16,
    pub spelling: &'static str,
    pub title: &'static str,
    pub family: DiagnosticFamilyKey,
    pub semantic_owner: &'static str,
    pub owning_stage: &'static str,
    pub status: AllocationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DiagnosticCauseKey(u16);

impl DiagnosticCauseKey {
    pub(crate) const fn ordinal(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiagnosticCauseSpec {
    pub(crate) key: DiagnosticCauseKey,
    pub(crate) reason: &'static str,
    pub(crate) code: DiagnosticCode,
    pub(crate) semantic_owner: &'static str,
    pub(crate) owning_stage: &'static str,
    pub(crate) origin_kind: &'static str,
    pub(crate) route_kind: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiagnosticPrecedenceSpec {
    pub(crate) dominant_causes: &'static [DiagnosticCauseKey],
    pub(crate) suppressed_causes: &'static [DiagnosticCauseKey],
    pub(crate) relationship: &'static str,
    pub(crate) applying_owner: &'static str,
}

macro_rules! diagnostic_causes {
    ($(($index:expr, $reason:literal, $code:ident, $owner:literal, $stage:literal, $origin:literal, $route:literal)),+ $(,)?) => {
        pub(crate) const DIAGNOSTIC_CAUSES: &[DiagnosticCauseSpec] = &[
            $(DiagnosticCauseSpec {
                key: DiagnosticCauseKey($index),
                reason: $reason,
                code: DiagnosticCode::$code,
                semantic_owner: $owner,
                owning_stage: $stage,
                origin_kind: $origin,
                route_kind: $route,
            }),+
        ];
    };
}

diagnostic_causes!(
    (
        0,
        "fallible_call_requires_try_v0",
        FALLIBLE_CALL_REQUIRES_TRY,
        "typed_failure_analysis",
        "full_type_check",
        "typed_failure_statement",
        "typed_failure_relationship"
    ),
    (
        1,
        "unwrapped_failure_roots_must_match_v0",
        INCOMPATIBLE_FAILURE_PROPAGATION,
        "typed_failure_analysis",
        "full_type_check",
        "typed_failure_statement",
        "typed_failure_relationship"
    ),
    (
        2,
        "failure_wrapper_root_must_match_caller_v0",
        FAILURE_WRAPPER_ROOT_MISMATCH,
        "typed_failure_analysis",
        "full_type_check",
        "typed_failure_statement",
        "typed_failure_relationship"
    ),
    (
        3,
        "try_requires_fallible_callee_v0",
        TRY_ON_INFALLIBLE_CALL,
        "typed_failure_analysis",
        "full_type_check",
        "typed_failure_statement",
        "typed_failure_relationship"
    ),
    (
        4,
        "direct_failure_root_must_match_caller_v0",
        DIRECT_FAILURE_ROOT_MISMATCH,
        "typed_failure_analysis",
        "full_type_check",
        "typed_failure_statement",
        "typed_failure_relationship"
    ),
    (
        5,
        "try_requires_unannotated_let_binding_v0",
        UNSUPPORTED_TRY_EXPRESSION,
        "typed_failure_analysis",
        "full_type_check",
        "typed_failure_statement",
        "typed_failure_relationship"
    ),
    (
        6,
        "unsupported_try_expression_shape_v0",
        UNSUPPORTED_TRY_EXPRESSION,
        "typed_failure_analysis",
        "full_type_check",
        "typed_failure_statement",
        "typed_failure_relationship"
    ),
    (
        7,
        "try_callee_not_known_v0",
        UNSUPPORTED_TRY_EXPRESSION,
        "typed_failure_analysis",
        "full_type_check",
        "typed_failure_statement",
        "typed_failure_relationship"
    ),
    (
        8,
        "typed_failure_requires_fails_when_v0",
        MISSING_FAILURE_DECLARATION,
        "typed_failure_analysis",
        "effect_check",
        "typed_failure_statement",
        "typed_failure_relationship"
    ),
    (
        9,
        "multiple_direct_callable_applications_unsupported_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        10,
        "receiver_parameter_shape_outside_al_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        11,
        "permission_bearing_callable_parameter_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        12,
        "callable_parameter_horizontal_whitespace_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        13,
        "callable_type_requires_task_open_paren_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        14,
        "callable_type_missing_close_paren_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        15,
        "callable_type_requires_space_before_arrow_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        16,
        "callable_type_missing_arrow_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        17,
        "callable_type_requires_result_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        18,
        "unsupported_callable_expression_shape_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        19,
        "callable_type_shape_outside_al_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        20,
        "callable_storage_or_return_unsupported_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        21,
        "required_exactly_one_callable_application_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        22,
        "multiple_callable_applications_unsupported_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        23,
        "indirect_application_shape_outside_al_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        24,
        "indirect_argument_outside_al_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        25,
        "recursive_callable_relationship_unsupported_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        26,
        "cross_file_callable_value_unsupported_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        27,
        "receiver_call_shape_outside_al_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        28,
        "direct_application_horizontal_whitespace_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        29,
        "task_value_shape_outside_al_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        30,
        "task_value_resolved_to_non_task_v0",
        INVALID_CALLABLE_FORM,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        31,
        "receiver_ordinary_signature_mismatch_v0",
        CALLABLE_SIGNATURE_MISMATCH,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        32,
        "indirect_argument_count_mismatch_v0",
        CALLABLE_SIGNATURE_MISMATCH,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        33,
        "callable_failure_root_mismatch_v0",
        CALLABLE_SIGNATURE_MISMATCH,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        34,
        "callable_input_type_mismatch_v0",
        CALLABLE_SIGNATURE_MISMATCH,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        35,
        "callable_result_type_mismatch_v0",
        CALLABLE_SIGNATURE_MISMATCH,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        36,
        "callable_latent_row_outside_bounded_am_slice_v0",
        CALLABLE_SIGNATURE_MISMATCH,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        37,
        "receiver_argument_type_mismatch_v0",
        CALLABLE_SIGNATURE_MISMATCH,
        "callable_analysis",
        "shared_preflight",
        "callable_relationship",
        "callable_definition_application_route"
    ),
    (
        38,
        "callable_precedence_item_header_missing_open_brace_v0",
        ITEM_HEADER_MISSING_OPEN_BRACE,
        "parser",
        "parser",
        "prior_diagnostic_node",
        "callable_precedence_relationship"
    ),
    (
        39,
        "callable_precedence_unexpected_signature_text_v0",
        UNEXPECTED_SIGNATURE_TEXT,
        "parser",
        "parser",
        "prior_diagnostic_node",
        "callable_precedence_relationship"
    ),
    (
        40,
        "callable_precedence_missing_close_paren_v0",
        CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN,
        "parser",
        "parser",
        "prior_diagnostic_node",
        "callable_precedence_relationship"
    ),
    (
        41,
        "callable_precedence_parameter_missing_type_v0",
        PARAMETER_MISSING_TYPE,
        "parser",
        "parser",
        "prior_diagnostic_node",
        "callable_precedence_relationship"
    ),
    (
        42,
        "callable_precedence_invalid_identifier_v0",
        INVALID_IDENTIFIER,
        "parser",
        "parser",
        "prior_diagnostic_node",
        "callable_precedence_relationship"
    ),
    (
        43,
        "callable_precedence_duplicate_name_v0",
        DUPLICATE_NAME_IN_SCOPE,
        "resolver",
        "resolver",
        "prior_diagnostic_node",
        "callable_precedence_relationship"
    ),
    (
        44,
        "callable_precedence_unknown_type_relationship_v0",
        UNKNOWN_TYPE_NAME,
        "type_check",
        "type_check",
        "prior_diagnostic_node",
        "callable_precedence_relationship"
    ),
    (
        45,
        "unknown_type_outside_callable_relationship_v0",
        UNKNOWN_TYPE_NAME,
        "type_check",
        "type_check",
        "prior_diagnostic_node",
        "unrelated_diagnostic_relationship"
    ),
    (
        46,
        "callable_precedence_opaque_path_relationship_v0",
        PATH_SOURCE_CONSTRUCTION,
        "path_boundary",
        "path_boundary",
        "prior_diagnostic_node",
        "callable_precedence_relationship"
    ),
    (
        47,
        "opaque_path_outside_callable_relationship_v0",
        PATH_SOURCE_CONSTRUCTION,
        "path_boundary",
        "path_boundary",
        "prior_diagnostic_node",
        "unrelated_diagnostic_relationship"
    )
);

pub(crate) fn diagnostic_cause(
    code: DiagnosticCode,
    reason: &str,
) -> Option<&'static DiagnosticCauseSpec> {
    DIAGNOSTIC_CAUSES
        .iter()
        .find(|cause| cause.code == code && cause.reason == reason)
}

const H090_CAUSES: &[DiagnosticCauseKey] = &[
    DiagnosticCauseKey(0),
    DiagnosticCauseKey(1),
    DiagnosticCauseKey(2),
    DiagnosticCauseKey(3),
    DiagnosticCauseKey(4),
    DiagnosticCauseKey(5),
    DiagnosticCauseKey(6),
    DiagnosticCauseKey(7),
    DiagnosticCauseKey(8),
];
const H1401_CAUSES: &[DiagnosticCauseKey] = &[
    DiagnosticCauseKey(9),
    DiagnosticCauseKey(10),
    DiagnosticCauseKey(11),
    DiagnosticCauseKey(12),
    DiagnosticCauseKey(13),
    DiagnosticCauseKey(14),
    DiagnosticCauseKey(15),
    DiagnosticCauseKey(16),
    DiagnosticCauseKey(17),
    DiagnosticCauseKey(18),
    DiagnosticCauseKey(19),
    DiagnosticCauseKey(20),
    DiagnosticCauseKey(21),
    DiagnosticCauseKey(22),
    DiagnosticCauseKey(23),
    DiagnosticCauseKey(24),
    DiagnosticCauseKey(25),
    DiagnosticCauseKey(26),
    DiagnosticCauseKey(27),
    DiagnosticCauseKey(28),
    DiagnosticCauseKey(29),
    DiagnosticCauseKey(30),
];
const H1402_CAUSES: &[DiagnosticCauseKey] = &[
    DiagnosticCauseKey(31),
    DiagnosticCauseKey(32),
    DiagnosticCauseKey(33),
    DiagnosticCauseKey(34),
    DiagnosticCauseKey(35),
    DiagnosticCauseKey(36),
    DiagnosticCauseKey(37),
];
const SOURCE_H1401_PRECEDENCE_CAUSES: &[DiagnosticCauseKey] = &[
    DiagnosticCauseKey(38),
    DiagnosticCauseKey(39),
    DiagnosticCauseKey(40),
    DiagnosticCauseKey(41),
    DiagnosticCauseKey(42),
    DiagnosticCauseKey(43),
    DiagnosticCauseKey(46),
];
const UNKNOWN_TYPE_H1402_PRECEDENCE_CAUSES: &[DiagnosticCauseKey] = &[DiagnosticCauseKey(44)];

pub(crate) const DIAGNOSTIC_PRECEDENCE: &[DiagnosticPrecedenceSpec] = &[
    DiagnosticPrecedenceSpec {
        dominant_causes: H090_CAUSES,
        suppressed_causes: &[
            DiagnosticCauseKey(9),
            DiagnosticCauseKey(10),
            DiagnosticCauseKey(11),
            DiagnosticCauseKey(12),
            DiagnosticCauseKey(13),
            DiagnosticCauseKey(14),
            DiagnosticCauseKey(15),
            DiagnosticCauseKey(16),
            DiagnosticCauseKey(17),
            DiagnosticCauseKey(18),
            DiagnosticCauseKey(19),
            DiagnosticCauseKey(20),
            DiagnosticCauseKey(21),
            DiagnosticCauseKey(22),
            DiagnosticCauseKey(23),
            DiagnosticCauseKey(24),
            DiagnosticCauseKey(25),
            DiagnosticCauseKey(26),
            DiagnosticCauseKey(27),
            DiagnosticCauseKey(28),
            DiagnosticCauseKey(29),
            DiagnosticCauseKey(30),
            DiagnosticCauseKey(31),
            DiagnosticCauseKey(32),
            DiagnosticCauseKey(33),
            DiagnosticCauseKey(34),
            DiagnosticCauseKey(35),
            DiagnosticCauseKey(36),
            DiagnosticCauseKey(37),
        ],
        relationship: "shared_callable_relationship_site_v0",
        applying_owner: "callable_analysis",
    },
    DiagnosticPrecedenceSpec {
        dominant_causes: SOURCE_H1401_PRECEDENCE_CAUSES,
        suppressed_causes: H1401_CAUSES,
        relationship: "shared_callable_relationship_site_v0",
        applying_owner: "callable_analysis",
    },
    DiagnosticPrecedenceSpec {
        dominant_causes: UNKNOWN_TYPE_H1402_PRECEDENCE_CAUSES,
        suppressed_causes: H1402_CAUSES,
        relationship: "shared_callable_relationship_site_v0",
        applying_owner: "callable_analysis",
    },
];

pub(crate) fn precedence_spec(
    dominant_cause: DiagnosticCauseKey,
    suppressed_cause: DiagnosticCauseKey,
) -> Option<DiagnosticPrecedenceSpec> {
    DIAGNOSTIC_PRECEDENCE
        .iter()
        .find(|rule| {
            rule.dominant_causes.contains(&dominant_cause)
                && rule.suppressed_causes.contains(&suppressed_cause)
        })
        .copied()
}

const fn historical_public_ordinal(key: DiagnosticCodeKey) -> u16 {
    match key.0 {
        0..=63 => key.0,
        64..=84 => key.0 + 2,
        85 => 64,
        86 => 65,
        _ => u16::MAX,
    }
}

pub const DIAGNOSTIC_FAMILIES: &[DiagnosticFamilySpec] = &[
    DiagnosticFamilySpec {
        start: 0,
        end: 99,
        key: DiagnosticFamilyKey::SOURCE_SHAPE,
        semantic_owner: "source_shape",
        status: AllocationStatus::Active,
        doctrine: &["decisions/0001", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 100,
        end: 199,
        key: DiagnosticFamilyKey::INTENT_SHAPE,
        semantic_owner: "intent_shape",
        status: AllocationStatus::Active,
        doctrine: &["decisions/0009", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 200,
        end: 299,
        key: DiagnosticFamilyKey::DECLARED_STATE_EFFECTS,
        semantic_owner: "declared_state_effects",
        status: AllocationStatus::Active,
        doctrine: &["decisions/0010", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 300,
        end: 399,
        key: DiagnosticFamilyKey::COST_CONTRACTS,
        semantic_owner: "cost_contracts",
        status: AllocationStatus::Active,
        doctrine: &["decisions/0006", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 400,
        end: 499,
        key: DiagnosticFamilyKey::SECURITY_TRUST,
        semantic_owner: "security_trust",
        status: AllocationStatus::Active,
        doctrine: &["docs/SECURITY_MODEL.md", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 500,
        end: 599,
        key: DiagnosticFamilyKey::TEST_EVIDENCE,
        semantic_owner: "test_evidence",
        status: AllocationStatus::Active,
        doctrine: &["decisions/0004", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 600,
        end: 699,
        key: DiagnosticFamilyKey::FRONT_END_SEMANTICS,
        semantic_owner: "front_end_semantics",
        status: AllocationStatus::Active,
        doctrine: &["decisions/0011", "decisions/0017", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 700,
        end: 799,
        key: DiagnosticFamilyKey::EXECUTABLE_CONTRACTS,
        semantic_owner: "executable_contracts",
        status: AllocationStatus::Active,
        doctrine: &["decisions/0015", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 800,
        end: 899,
        key: DiagnosticFamilyKey::OWNERSHIP_BORROWING,
        semantic_owner: "ownership_borrowing",
        status: AllocationStatus::Active,
        doctrine: &["decisions/0014", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 900,
        end: 999,
        key: DiagnosticFamilyKey::NOMINAL_TYPED_FAILURE,
        semantic_owner: "nominal_typed_failure",
        status: AllocationStatus::Active,
        doctrine: &["decisions/0016", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 1000,
        end: 1099,
        key: DiagnosticFamilyKey::UNSAFE_FFI_PROVENANCE,
        semantic_owner: "unsafe_ffi_provenance",
        status: AllocationStatus::Reserved,
        doctrine: &["docs/UNSAFE_POLICY.md", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 1100,
        end: 1199,
        key: DiagnosticFamilyKey::RUNTIME_PROFILE_POLICY,
        semantic_owner: "runtime_profile_policy",
        status: AllocationStatus::Reserved,
        doctrine: &["docs/RUNTIME_PROFILES.md", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 1200,
        end: 1299,
        key: DiagnosticFamilyKey::TARGET_BACKEND_METADATA,
        semantic_owner: "target_backend_metadata",
        status: AllocationStatus::Active,
        doctrine: &["docs/PORTABILITY_BOUNDARY_MODEL.md", "WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 1300,
        end: 1399,
        key: DiagnosticFamilyKey::CONCURRENCY_MEMORY_ORDERING,
        semantic_owner: "concurrency_memory_ordering",
        status: AllocationStatus::Reserved,
        doctrine: &["WORKORDER.md"],
    },
    DiagnosticFamilySpec {
        start: 1400,
        end: 1499,
        key: DiagnosticFamilyKey::CALLABLE_EFFECT_ROWS,
        semantic_owner: "callable_effect_rows",
        status: AllocationStatus::Active,
        doctrine: &["decisions/0018", "WORKORDER.md"],
    },
];

macro_rules! diagnostic_code_allocations {
    ($(($index:expr, $name:ident, $spelling:literal, $title:literal, $family:ident, $owner:literal, $stage:literal)),+ $(,)?) => {
        impl DiagnosticCodeKey {
            $(pub const $name: Self = Self($index);)+
        }

        impl DiagnosticCode {
            $(pub const $name: Self = Self::from_key(DiagnosticCodeKey::$name);)+
        }

        pub const DIAGNOSTIC_CODE_ALLOCATIONS: &[DiagnosticCodeAllocation] = &[
            $(DiagnosticCodeAllocation {
                key: DiagnosticCodeKey::$name,
                public_ordinal: historical_public_ordinal(DiagnosticCodeKey::$name),
                spelling: $spelling,
                title: $title,
                family: DiagnosticFamilyKey::$family,
                semantic_owner: $owner,
                owning_stage: $stage,
                status: AllocationStatus::Active,
            }),+
        ];
    };
}

#[rustfmt::skip]
diagnostic_code_allocations!(
    (
        0,
        UNEXPECTED_TOP_LEVEL_LINE,
        "H0001",
        "unexpected top-level line",
        SOURCE_SHAPE,
        "source_shape",
        "parser"
    ),
    (
        1,
        NESTED_ITEM_EXTENDS_PAST_BLOCK,
        "H0002",
        "nested item extends past containing block",
        SOURCE_SHAPE,
        "source_shape",
        "parser"
    ),
    (
        2,
        ITEM_HEADER_MISSING_OPEN_BRACE,
        "H0003",
        "item header missing opening brace",
        SOURCE_SHAPE,
        "source_shape",
        "parser"
    ),
    (
        3,
        ITEM_BLOCK_MISSING_CLOSE_BRACE,
        "H0004",
        "item block missing closing brace",
        SOURCE_SHAPE,
        "source_shape",
        "parser"
    ),
    (
        4,
        UNKNOWN_ITEM_KIND,
        "H0005",
        "unknown item kind",
        SOURCE_SHAPE,
        "source_shape",
        "parser"
    ),
    (
        5,
        UNEXPECTED_SIGNATURE_TEXT,
        "H0006",
        "unexpected callable signature text",
        SOURCE_SHAPE,
        "source_shape",
        "parser"
    ),
    (
        6,
        CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN,
        "H0007",
        "callable signature missing close parenthesis",
        SOURCE_SHAPE,
        "source_shape",
        "parser"
    ),
    (
        7,
        PARAMETER_MISSING_TYPE,
        "H0008",
        "parameter missing type",
        SOURCE_SHAPE,
        "source_shape",
        "parser"
    ),
    (
        8,
        INVALID_IDENTIFIER,
        "H0009",
        "invalid identifier",
        SOURCE_SHAPE,
        "source_shape",
        "parser"
    ),
    (
        9,
        APP_MISSING_WHY,
        "H0101",
        "app missing why section",
        INTENT_SHAPE,
        "intent_shape",
        "check"
    ),
    (
        10,
        TYPE_MISSING_SHAPE,
        "H0102",
        "type missing shape",
        INTENT_SHAPE,
        "intent_shape",
        "check"
    ),
    (
        11,
        STORE_MISSING_TYPE,
        "H0103",
        "store missing type",
        INTENT_SHAPE,
        "intent_shape",
        "check"
    ),
    (
        12,
        STORE_MISSING_PURPOSE,
        "H0104",
        "store missing purpose",
        INTENT_SHAPE,
        "intent_shape",
        "check"
    ),
    (
        13,
        MISSING_REQUIRED_SECTION,
        "H0105",
        "item missing required section",
        INTENT_SHAPE,
        "intent_shape",
        "check"
    ),
    (
        14,
        DUPLICATE_SECTION,
        "H0106",
        "duplicate section",
        INTENT_SHAPE,
        "intent_shape",
        "check"
    ),
    (
        15,
        TASK_MISSING_NEEDS,
        "H0107",
        "task missing needs section",
        INTENT_SHAPE,
        "intent_shape",
        "check"
    ),
    (
        16,
        SECTION_OUT_OF_ORDER,
        "H0108",
        "section out of order",
        INTENT_SHAPE,
        "intent_shape",
        "check"
    ),
    (
        17,
        TASK_MISSING_ENSURES,
        "H0109",
        "task return missing ensures section",
        INTENT_SHAPE,
        "intent_shape",
        "check"
    ),
    (
        18,
        HOLLOW_CONTRACT_LINE,
        "H0110",
        "hollow contract line",
        INTENT_SHAPE,
        "intent_shape",
        "check"
    ),
    (
        19,
        UNDECLARED_SAVE_TARGET,
        "H0201",
        "save target not declared in changes",
        DECLARED_STATE_EFFECTS,
        "declared_state_effects",
        "check"
    ),
    (
        20,
        UNDECLARED_SET_TARGET,
        "H0202",
        "set target not declared mutable",
        DECLARED_STATE_EFFECTS,
        "declared_state_effects",
        "check"
    ),
    (
        21,
        TASK_MISSING_COST,
        "H0301",
        "task missing cost section",
        COST_CONTRACTS,
        "cost_contracts",
        "check"
    ),
    (
        22,
        COST_MISSING_CHECK,
        "H0302",
        "cost missing check level",
        COST_CONTRACTS,
        "cost_contracts",
        "check"
    ),
    (
        23,
        COMPILE_COST_MISSING_TIME,
        "H0303",
        "compile cost missing time claim",
        COST_CONTRACTS,
        "cost_contracts",
        "check"
    ),
    (
        24,
        CONSTANT_COST_HAS_FOR_EACH,
        "H0304",
        "constant cost claim has iteration",
        COST_CONTRACTS,
        "cost_contracts",
        "check"
    ),
    (
        25,
        COMPILE_COST_UNBOUNDED_WHILE,
        "H0305",
        "compile cost has unbounded-looking while",
        COST_CONTRACTS,
        "cost_contracts",
        "check"
    ),
    (
        26,
        SECURITY_MISSING_PROTECTS,
        "H0401",
        "security-sensitive task missing protects",
        SECURITY_TRUST,
        "security_trust",
        "check"
    ),
    (
        27,
        TRUSTS_MISSING_PROTECTS,
        "H0402",
        "trust boundary missing protects",
        SECURITY_TRUST,
        "security_trust",
        "check"
    ),
    (
        28,
        TEST_MISSING_COVERS,
        "H0501",
        "test missing covers section",
        TEST_EVIDENCE,
        "test_evidence",
        "check"
    ),
    (
        29,
        REGRESSION_MISSING_NOTE,
        "H0502",
        "regression test missing regression note",
        TEST_EVIDENCE,
        "test_evidence",
        "check"
    ),
    (
        30,
        UNRESOLVED_NAME,
        "H0601",
        "unresolved name",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "resolve"
    ),
    (
        31,
        DUPLICATE_NAME_IN_SCOPE,
        "H0602",
        "duplicate name in scope",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "resolve"
    ),
    (
        32,
        SET_TARGET_IMMUTABLE,
        "H0603",
        "set target is immutable",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "resolve"
    ),
    (
        33,
        READ_BEFORE_DECLARE,
        "H0604",
        "read before declaration",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "resolve"
    ),
    (
        34,
        UNKNOWN_TYPE_NAME,
        "H0605",
        "unknown type name",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "type_check"
    ),
    (
        35,
        RETURN_TYPE_MISMATCH,
        "H0606",
        "return type mismatch",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "type_check"
    ),
    (
        36,
        APP_START_MISSING,
        "H0610",
        "app start missing",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "app_entry"
    ),
    (
        37,
        APP_START_EMPTY,
        "H0611",
        "app start empty",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "app_entry"
    ),
    (
        38,
        APP_START_DUPLICATE,
        "H0612",
        "app start duplicated",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "app_entry"
    ),
    (
        39,
        APP_START_INVALID_NAME,
        "H0613",
        "invalid app start name",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "app_entry"
    ),
    (
        40,
        APP_START_NOT_CHILD,
        "H0614",
        "app start task is not a child",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "app_entry"
    ),
    (
        41,
        MULTIPLE_EXECUTABLE_APPS,
        "H0615",
        "multiple executable apps",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "app_entry"
    ),
    (
        42,
        APP_START_INVALID_RESULT,
        "H0616",
        "invalid app start result",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "app_entry"
    ),
    (
        43,
        UNKNOWN_SOURCE_CAPABILITY,
        "H0617",
        "unknown source capability",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        44,
        MISSING_CALLER_CAPABILITY,
        "H0618",
        "caller capability closure is incomplete",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        45,
        APP_CAPABILITY_MISMATCH,
        "H0619",
        "app capability maximum is incomplete",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        46,
        ENTRY_CAPABILITY_BYPASS,
        "H0620",
        "direct entry cannot carry external authority",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        47,
        OUTPUT_CAPABILITY_UNDECLARED,
        "H0621",
        "stdout operation lacks source authority",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        48,
        INVALID_STDOUT_WRITE_CALL,
        "H0622",
        "invalid stdout_write call",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        49,
        RESERVED_BUILTIN_NAME,
        "H0623",
        "reserved built-in name redeclared",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        50,
        OUTPUT_RECURSION_UNSUPPORTED,
        "H0624",
        "output-reachable recursion unsupported",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        51,
        REPLAY_CAPABILITY_UNDECLARED,
        "H0625",
        "replay operation lacks source authority",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        52,
        INVALID_CLOCK_REPLAY_CALL,
        "H0626",
        "invalid clock_replay_tick call",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        53,
        RESERVED_REPLAY_BUILTIN_NAME,
        "H0627",
        "reserved replay built-in name redeclared",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        54,
        REPLAY_RECURSION_UNSUPPORTED,
        "H0628",
        "replay-reachable recursion unsupported",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        55,
        INVALID_PATH_BOUNDARY,
        "H0629",
        "invalid opaque Path boundary",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "path_boundary"
    ),
    (
        56,
        PATH_SOURCE_CONSTRUCTION,
        "H0630",
        "opaque Path cannot be constructed or used in source",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "path_boundary"
    ),
    (
        57,
        FILE_CAPABILITY_UNDECLARED,
        "H0631",
        "file operation lacks source authority",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "capability_root"
    ),
    (
        58,
        INVALID_FILE_READ_CALL,
        "H0632",
        "invalid files_read_text call",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "file_read"
    ),
    (
        59,
        RESERVED_FILE_READ_BUILTIN_NAME,
        "H0633",
        "reserved file-read built-in name redeclared",
        FRONT_END_SEMANTICS,
        "front_end_semantics",
        "file_read"
    ),
    (
        60,
        UNCHECKED_PROSE_CONTRACT,
        "H0701",
        "unchecked prose contract",
        EXECUTABLE_CONTRACTS,
        "executable_contracts",
        "predicate"
    ),
    (
        61,
        NEEDS_CONTRACT_VIOLATION,
        "H0702",
        "needs contract violation",
        EXECUTABLE_CONTRACTS,
        "executable_contracts",
        "run"
    ),
    (
        62,
        ENSURES_CONTRACT_VIOLATION,
        "H0703",
        "ensures contract violation",
        EXECUTABLE_CONTRACTS,
        "executable_contracts",
        "run"
    ),
    (
        63,
        INVALID_EXECUTABLE_PREDICATE,
        "H0704",
        "invalid executable predicate",
        EXECUTABLE_CONTRACTS,
        "executable_contracts",
        "predicate"
    ),
    (
        64,
        USE_AFTER_MOVE,
        "H0801",
        "use after move",
        OWNERSHIP_BORROWING,
        "ownership_borrowing",
        "ownership_check"
    ),
    (
        65,
        BORROW_PARAMETER_MUTATION,
        "H0802",
        "borrowed parameter written",
        OWNERSHIP_BORROWING,
        "ownership_borrowing",
        "ownership_check"
    ),
    (
        66,
        LINEAR_RESOURCE_NOT_CONSUMED,
        "H0803",
        "linear resource not consumed",
        OWNERSHIP_BORROWING,
        "ownership_borrowing",
        "ownership_check"
    ),
    (
        67,
        LINEAR_RESOURCE_CONSUMED_TWICE,
        "H0804",
        "linear resource consumed twice",
        OWNERSHIP_BORROWING,
        "ownership_borrowing",
        "ownership_check"
    ),
    (
        68,
        RETURN_DEPENDENCY_NOT_PARAMETER,
        "H0805",
        "return dependency is not a parameter",
        OWNERSHIP_BORROWING,
        "ownership_borrowing",
        "ownership_check"
    ),
    (
        69,
        ITERATION_MUTATION_CONFLICT,
        "H0806",
        "iteration mutation conflict",
        OWNERSHIP_BORROWING,
        "ownership_borrowing",
        "ownership_check"
    ),
    (
        70,
        STALE_FIELD_VIEW,
        "H0807",
        "stale view",
        OWNERSHIP_BORROWING,
        "ownership_borrowing",
        "ownership_check"
    ),
    (
        71,
        WRITABLE_ALIAS_OVERLAP,
        "H0808",
        "writable alias overlap",
        OWNERSHIP_BORROWING,
        "ownership_borrowing",
        "ownership_check"
    ),
    (
        72,
        UNSUPPORTED_WRITABLE_ALIAS,
        "H0809",
        "unsupported writable alias",
        OWNERSHIP_BORROWING,
        "ownership_borrowing",
        "ownership_check"
    ),
    (
        73,
        FALLIBLE_CALL_REQUIRES_TRY,
        "H0901",
        "fallible call requires try",
        NOMINAL_TYPED_FAILURE,
        "nominal_typed_failure",
        "full_type_check"
    ),
    (
        74,
        INCOMPATIBLE_FAILURE_PROPAGATION,
        "H0902",
        "incompatible failure propagation",
        NOMINAL_TYPED_FAILURE,
        "nominal_typed_failure",
        "full_type_check"
    ),
    (
        75,
        FAILURE_WRAPPER_ROOT_MISMATCH,
        "H0903",
        "failure wrapper root mismatch",
        NOMINAL_TYPED_FAILURE,
        "nominal_typed_failure",
        "full_type_check"
    ),
    (
        76,
        TRY_ON_INFALLIBLE_CALL,
        "H0904",
        "try on infallible call",
        NOMINAL_TYPED_FAILURE,
        "nominal_typed_failure",
        "full_type_check"
    ),
    (
        77,
        DIRECT_FAILURE_ROOT_MISMATCH,
        "H0905",
        "direct failure root mismatch",
        NOMINAL_TYPED_FAILURE,
        "nominal_typed_failure",
        "full_type_check"
    ),
    (
        78,
        UNSUPPORTED_TRY_EXPRESSION,
        "H0906",
        "unsupported try expression",
        NOMINAL_TYPED_FAILURE,
        "nominal_typed_failure",
        "full_type_check"
    ),
    (
        79,
        MISSING_FAILURE_DECLARATION,
        "H0907",
        "missing failure declaration",
        NOMINAL_TYPED_FAILURE,
        "nominal_typed_failure",
        "effect_check"
    ),
    (
        80,
        UNKNOWN_TARGET_FACT_RECORD,
        "H1201",
        "unknown target fact record",
        TARGET_BACKEND_METADATA,
        "target_backend_metadata",
        "check"
    ),
    (
        81,
        UNKNOWN_CAPABILITY_FAMILY,
        "H1202",
        "unknown capability family",
        TARGET_BACKEND_METADATA,
        "target_backend_metadata",
        "check"
    ),
    (
        82,
        UNSUPPORTED_TARGET_DECLARATION,
        "H1203",
        "unsupported target declaration",
        TARGET_BACKEND_METADATA,
        "target_backend_metadata",
        "check"
    ),
    (
        83,
        REQUIRED_CAPABILITY_UNAVAILABLE,
        "H1204",
        "required capability unavailable on target",
        TARGET_BACKEND_METADATA,
        "target_backend_metadata",
        "check"
    ),
    (
        84,
        CONFLICTING_TARGET_CAPABILITY,
        "H1205",
        "conflicting target capability declaration",
        TARGET_BACKEND_METADATA,
        "target_backend_metadata",
        "check"
    ),
    (
        85,
        INVALID_CALLABLE_FORM,
        "H1401",
        "invalid or unsupported callable form",
        CALLABLE_EFFECT_ROWS,
        "callable_effect_rows",
        "callable"
    ),
    (
        86,
        CALLABLE_SIGNATURE_MISMATCH,
        "H1402",
        "callable signature mismatch",
        CALLABLE_EFFECT_ROWS,
        "callable_effect_rows",
        "callable"
    ),
);

pub fn allocation(key: DiagnosticCodeKey) -> &'static DiagnosticCodeAllocation {
    DIAGNOSTIC_CODE_ALLOCATIONS
        .iter()
        .find(|allocation| allocation.key == key)
        .expect("diagnostic code key must be registered")
}

#[derive(Debug, Clone, Copy)]
pub struct DiagnosticInfo {
    pub code: DiagnosticCode,
    pub default_severity: Severity,
    pub explanation: &'static str,
    pub repair: &'static str,
}

pub const DIAGNOSTICS: &[DiagnosticInfo] = &[
    DiagnosticInfo {
        code: DiagnosticCode::UNEXPECTED_TOP_LEVEL_LINE,
        default_severity: Severity::Warning,
        explanation: "A top-level line does not start a recognized Hum item. Milestone 0 only recognizes module declarations and app, type, store, task, or test items.",
        repair: "Move the line into a recognized item body, turn it into a section line, or remove it until the surface grammar supports it.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::NESTED_ITEM_EXTENDS_PAST_BLOCK,
        default_severity: Severity::Error,
        explanation: "A nested item crosses the closing brace of the app or item that contains it, so the parser cannot trust the block structure.",
        repair: "Check the surrounding braces and keep the nested item fully inside its parent block.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::ITEM_HEADER_MISSING_OPEN_BRACE,
        default_severity: Severity::Error,
        explanation: "An item header is missing the opening brace that starts its body.",
        repair: "Add `{` at the end of the item header or rewrite the line as section text inside an existing item.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::ITEM_BLOCK_MISSING_CLOSE_BRACE,
        default_severity: Severity::Error,
        explanation: "An item body starts but does not close before the file ends or its parent block ends.",
        repair: "Add the missing `}` at the intended end of the item body.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNKNOWN_ITEM_KIND,
        default_severity: Severity::Warning,
        explanation: "The parser found an item-like header whose first word is not a current Hum item kind.",
        repair: "Use `app`, `type`, `store`, `task`, or `test`, or keep the idea in a checked section until the item kind is designed.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNEXPECTED_SIGNATURE_TEXT,
        default_severity: Severity::Warning,
        explanation: "A task or test signature has trailing text after the part Milestone 0 understands.",
        repair: "Move the extra text into a checked section or simplify the signature to the current parser contract.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN,
        default_severity: Severity::Error,
        explanation: "A task or test parameter list starts with `(` but does not close with `)`.",
        repair: "Close the parameter list or remove the opening parenthesis if the item takes no parameters.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::PARAMETER_MISSING_TYPE,
        default_severity: Severity::Error,
        explanation: "A parameter does not declare an explicit type, so later checks cannot know what the callable expects.",
        repair: "Write the parameter as `name: Type`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INVALID_IDENTIFIER,
        default_severity: Severity::Error,
        explanation: "A Hum identifier uses one deterministic token. Value names use snake_case and type names use PascalCase, so spaced names cannot be parsed as symbols.",
        repair: "Use a single token such as `remember_work_item`; put human phrasing in `why:` or another prose section.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_MISSING_WHY,
        default_severity: Severity::Warning,
        explanation: "An app lacks a `why:` section, leaving the application purpose invisible to readers and tools.",
        repair: "Add a `why:` section that states the app's purpose in plain language.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TYPE_MISSING_SHAPE,
        default_severity: Severity::Warning,
        explanation: "A type has no visible fields or shape facts, so it does not yet explain what data it represents.",
        repair: "Add fields or an intent section that describes the type's shape.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::STORE_MISSING_TYPE,
        default_severity: Severity::Warning,
        explanation: "A store does not declare the type of data it contains.",
        repair: "Write the store header with a type, such as `store sessions: list Session`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::STORE_MISSING_PURPOSE,
        default_severity: Severity::Warning,
        explanation: "A store lacks visible purpose, which makes persistence and shared state harder to review.",
        repair: "Add `why:` or another accepted purpose section explaining what the store remembers and why.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::MISSING_REQUIRED_SECTION,
        default_severity: Severity::Error,
        explanation: "An item is missing a section Milestone 0 needs, such as `does:`, or nontrivial behavior is missing visible purpose in `why:`.",
        repair: "Add `does:` when the body is missing. Add `why:` when effects, failures, or body size make purpose non-obvious.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::DUPLICATE_SECTION,
        default_severity: Severity::Warning,
        explanation: "The same section appears more than once in one item, which can split important intent across multiple places.",
        repair: "Merge the repeated sections into one section in canonical order.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TASK_MISSING_NEEDS,
        default_severity: Severity::Warning,
        explanation: "A task has a risky boundary but no `needs:` section, so real caller responsibilities may be hidden.",
        repair: "Add `needs:` only when callers must satisfy a real precondition; avoid filler for pure local code.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::SECTION_OUT_OF_ORDER,
        default_severity: Severity::Warning,
        explanation: "A known section appears after a later section in Hum's canonical review order.",
        repair: "Move the section into the documented order so readers see purpose, capabilities, contracts, risks, cost, and implementation consistently.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TASK_MISSING_ENSURES,
        default_severity: Severity::Warning,
        explanation: "A returning task crosses a nontrivial boundary without an `ensures:` section, so its success promise may be hidden.",
        repair: "Add `ensures:` when the result promise is not obvious from a small pure body.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::HOLLOW_CONTRACT_LINE,
        default_severity: Severity::Warning,
        explanation: "A contract-like section contains a line that is too generic, tautological, or placeholder-shaped to reject a meaningful wrong implementation.",
        repair: "Replace it with a specific claim, edge case, boundary, allocation limit, or success condition that could catch a real mistake.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNDECLARED_SAVE_TARGET,
        default_severity: Severity::Error,
        explanation: "A `does:` body saves into a resource that is not listed under `changes:`, so mutation would be hidden from readers and tools.",
        repair: "Add the resource under `changes:` if the mutation is intended, or remove the save.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNDECLARED_SET_TARGET,
        default_severity: Severity::Error,
        explanation: "A `does:` body sets a name that is neither a local `change` binding nor a declared changed resource.",
        repair: "Declare the local name with `change`, list the resource under `changes:`, or avoid the mutation.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TASK_MISSING_COST,
        default_severity: Severity::Warning,
        explanation: "A task has loops, effects, or a larger body but no `cost:` section, so resource expectations are not visible for review.",
        repair: "Add `cost:` when resource behavior is worth reviewing; avoid filler for tiny pure tasks.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::COST_MISSING_CHECK,
        default_severity: Severity::Warning,
        explanation: "A `cost:` block lacks a `check:` level, so tools do not know whether the claim is informational or enforced.",
        repair: "Add `check: warn`, `check: compile`, or another accepted check level as the cost model matures.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::COMPILE_COST_MISSING_TIME,
        default_severity: Severity::Error,
        explanation: "A task asks for compile-time cost checking but does not state a `time:` claim.",
        repair: "Add a `time:` claim such as `time: O(1)` or lower the check level until the cost can be stated honestly.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::CONSTANT_COST_HAS_FOR_EACH,
        default_severity: Severity::Error,
        explanation: "A task claims constant time while visibly using `for each`, which usually depends on input size.",
        repair: "Change the time claim, remove the iteration, or record a narrower bound with a clear reason.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::COMPILE_COST_UNBOUNDED_WHILE,
        default_severity: Severity::Error,
        explanation: "A compile-checked cost claim contains a `while` loop without an obvious bound.",
        repair: "Use a visibly bounded loop, add a clearer bound, or lower the check level until the checker can prove the claim.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::SECURITY_MISSING_PROTECTS,
        default_severity: Severity::Warning,
        explanation: "A task touches security-sensitive names but does not state what it protects.",
        repair: "Add a `protects:` section that names the security property, or remove the sensitive dependency.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TRUSTS_MISSING_PROTECTS,
        default_severity: Severity::Warning,
        explanation: "A task declares a trust boundary without a matching safety or security promise.",
        repair: "Add `protects:` lines that say what must remain true across the trusted boundary.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TEST_MISSING_COVERS,
        default_severity: Severity::Warning,
        explanation: "A test does not say which promise it covers, so it cannot serve as first-class evidence.",
        repair: "Add a `covers:` section that points at the task obligation or promise the test checks.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::REGRESSION_MISSING_NOTE,
        default_severity: Severity::Warning,
        explanation: "A regression test does not record the bug shape it prevents from returning.",
        repair: "Add a `regression:` section describing the old failure mode.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNRESOLVED_NAME,
        default_severity: Severity::Error,
        explanation: "A checked resolver found a name that is not visible from the current lexical scope or declared dependency boundary.",
        repair: "Declare the name before use, add a matching item, or list the external dependency under `uses:` when it is intentional.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::DUPLICATE_NAME_IN_SCOPE,
        default_severity: Severity::Error,
        explanation: "Two definitions in one scope normalize to the same name, so references would be ambiguous.",
        repair: "Rename one binding or move it into a narrower block so the scope has one definition for the name.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::SET_TARGET_IMMUTABLE,
        default_severity: Severity::Error,
        explanation: "A `set` target resolves to a visible definition, but that definition is not a mutable place or declared change permission.",
        repair: "Declare the local with `change`, target a declared `changes:` permission, or keep the value immutable.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::READ_BEFORE_DECLARE,
        default_severity: Severity::Error,
        explanation: "A name is read before its later local declaration, which makes the data dependency order unclear.",
        repair: "Move the declaration above the read or pass the value in through an explicit parameter.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNKNOWN_TYPE_NAME,
        default_severity: Severity::Error,
        explanation: "A declaration annotation names a type that is neither declared in the source nor reserved by the current Hum type environment.",
        repair: "Declare the type, use a reserved type root, or wait for imports/packages before relying on external type names.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::RETURN_TYPE_MISMATCH,
        default_severity: Severity::Error,
        explanation: "A task return expression has a trivial source-visible type that does not match the task result type or Result/Option/Maybe success type.",
        repair: "Return the expected value type, change the task result annotation, or keep complex expressions unchecked until full expression typing exists.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_MISSING,
        default_severity: Severity::Error,
        explanation: "A top-level executable app has no `starts with:` declaration, so `hum run` has no structural task root.",
        repair: "Add exactly one `starts with:` section containing one bare name of a task directly nested in the app.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_EMPTY,
        default_severity: Severity::Error,
        explanation: "An executable app has a `starts with:` section but no meaningful start-task name.",
        repair: "Put exactly one bare snake_case direct-child task name under `starts with:`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_DUPLICATE,
        default_severity: Severity::Error,
        explanation: "An executable app has multiple `starts with:` sections or multiple meaningful start declarations, making its root ambiguous.",
        repair: "Keep one `starts with:` section with one meaningful bare task name.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_INVALID_NAME,
        default_severity: Severity::Error,
        explanation: "An app start declaration is not one bare snake_case value identifier.",
        repair: "Use a direct-child task name such as `run_tool`, without call syntax, paths, assignments, or state initialization.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_NOT_CHILD,
        default_severity: Severity::Error,
        explanation: "An app start name does not resolve to a task directly nested in that app. App mode never falls back to global task lookup.",
        repair: "Nest the named task directly in the app or name an existing direct child.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::MULTIPLE_EXECUTABLE_APPS,
        default_severity: Severity::Error,
        explanation: "Run input contains more than one top-level app, so there is no unique executable program root.",
        repair: "Run exactly one top-level app input, or use `--entry <task>` for a direct legacy task probe.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_INVALID_RESULT,
        default_severity: Severity::Error,
        explanation: "An app start task does not return `Unit` or `Result Unit, E`, so app completion would expose an unsupported program result.",
        repair: "Return `Unit` (including an omitted result) or `Result Unit, ErrorType` from the start task.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNKNOWN_SOURCE_CAPABILITY,
        default_severity: Severity::Error,
        explanation: "An executable app or one of its tasks uses a dotted external-capability spelling outside Session Y's exact `stdout.write`, `clock.replay`, and `files.read` vocabulary.",
        repair: "Use one exact pinned capability ID, or keep an ordinary non-capability dependency under `uses:` without an external capability-family spelling.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::MISSING_CALLER_CAPABILITY,
        default_severity: Severity::Error,
        explanation: "A caller reaches a callee capability closure that is not included in the caller's own exact source-authority budget.",
        repair: "Add the exact capability under the caller's `uses:` section or remove/restructure the call path.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_CAPABILITY_MISMATCH,
        default_severity: Severity::Error,
        explanation: "The app maximum source-authority declaration does not cover a pinned capability in the start task's closed direct-call route.",
        repair: "Add the exact capability under the app's `uses:` section or remove it from the reachable start-task closure; source declaration is not operator consent.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::ENTRY_CAPABILITY_BYPASS,
        default_severity: Severity::Error,
        explanation: "An explicit `--entry` task reaches pinned external source authority, but direct entry is a pure probe outside the structural app authority root.",
        repair: "Run the structural app without `--entry`, or select a task whose complete direct-call closure is pure.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::OUTPUT_CAPABILITY_UNDECLARED,
        default_severity: Severity::Error,
        explanation: "A `stdout_write` call is executable only when both its task and containing app declare exact `stdout.write` source authority.",
        repair: "Add exact `stdout.write` to the blamed task and app `uses:` sections, preserving caller closure, or remove the output call.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INVALID_STDOUT_WRITE_CALL,
        default_severity: Severity::Error,
        explanation: "The bounded output built-in has exactly one signature: `stdout_write(text: Text) -> Result Unit, OutputError`.",
        repair: "Pass exactly one checked `Text` argument and handle the typed `OutputError` with the authorized explicit propagation or wrapping form.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::RESERVED_BUILTIN_NAME,
        default_severity: Severity::Error,
        explanation: "A user task redeclares the exact `stdout_write` name reserved for Hum's bounded output built-in, which would split callable identity across stages.",
        repair: "Rename the user task and keep `stdout_write` reserved for `stdout_write(text: Text) -> Result Unit, OutputError`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::OUTPUT_RECURSION_UNSUPPORTED,
        default_severity: Severity::Error,
        explanation: "A recursive call cycle is reachable from the structural app start and can reach bounded output, but Session Z audit routes require finite exact call-occurrence identities.",
        repair: "Rewrite the output-bearing recursion as an explicit bounded loop or a non-recursive task chain so every output route has a finite auditable call path.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::REPLAY_CAPABILITY_UNDECLARED,
        default_severity: Severity::Error,
        explanation: "A `clock_replay_tick` call is executable only when both its task and containing app declare exact `clock.replay` source authority.",
        repair: "Add exact `clock.replay` to the blamed task and app `uses:` sections, preserving caller closure, or remove the replay call.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INVALID_CLOCK_REPLAY_CALL,
        default_severity: Severity::Error,
        explanation: "The runner replay built-in has exactly one signature: `clock_replay_tick() -> Result UInt, ReplayClockError`.",
        repair: "Call `clock_replay_tick()` with no arguments and handle the typed `ReplayClockError` with explicit propagation or wrapping.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::RESERVED_REPLAY_BUILTIN_NAME,
        default_severity: Severity::Error,
        explanation: "A user task redeclares the exact `clock_replay_tick` name reserved for Hum's runner-provided replay built-in, which would split callable identity across stages.",
        repair: "Rename the user task and keep `clock_replay_tick` reserved for `clock_replay_tick() -> Result UInt, ReplayClockError`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::REPLAY_RECURSION_UNSUPPORTED,
        default_severity: Severity::Error,
        explanation: "A recursive call cycle is reachable from the structural app start and can reach replay input, but Session AA audit routes require finite exact call-occurrence identities.",
        repair: "Rewrite the replay-bearing recursion as an explicit bounded loop or a non-recursive task chain so every replay route has a finite auditable call path.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INVALID_PATH_BOUNDARY,
        default_severity: Severity::Error,
        explanation: "Opaque Path identity appears outside the single runner-constructed structural app start parameter, or appears in a return or storage declaration.",
        repair: "Keep exactly one `Path` parameter only on the structural app start task; Path has no source construction, return, or storage surface in Session AB.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::PATH_SOURCE_CONSTRUCTION,
        default_severity: Severity::Error,
        explanation: "Hum source attempts to construct, pass, inspect, store, return, compare, or otherwise use the runner-owned opaque Path value outside the exact hardened file-read consumption.",
        repair: "Provide Path only through structural app entry and pass it only as the sole argument to `files_read_text`; keep every other Path operation absent.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::FILE_CAPABILITY_UNDECLARED,
        default_severity: Severity::Error,
        explanation: "A `files_read_text` call is executable only when its task, every caller, and the containing app declare exact `files.read` source authority.",
        repair: "Add exact `files.read` to the blamed task and app `uses:` sections, preserving caller closure, or remove the file call; source declaration does not grant operator consent.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INVALID_FILE_READ_CALL,
        default_severity: Severity::Error,
        explanation: "The hardened file-read built-in has exactly one signature: `files_read_text(path: Path) -> Result Text, FileReadError`.",
        repair: "Pass exactly the runner-owned opaque Path and handle `FileReadError` with explicit propagation or causal wrapping.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::RESERVED_FILE_READ_BUILTIN_NAME,
        default_severity: Severity::Error,
        explanation: "A user task redeclares the exact `files_read_text` name reserved for Hum's hardened exact-file reader, which would split callable identity across stages.",
        repair: "Rename the user task and keep `files_read_text` reserved for `files_read_text(path: Path) -> Result Text, FileReadError`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNCHECKED_PROSE_CONTRACT,
        default_severity: Severity::Warning,
        explanation: "`hum run` saw a meaningful `needs:` or `ensures:` line with no Predicate v2 intent signal, so it remains visible honest prose rather than executable contract text.",
        repair: "Use one typed Predicate v2 comparison such as `b != 0`, `result.title == old(item.title)`, `result == \"parse\"`, or `result == list_count(items, \"hum\")` when the contract is meant to execute now; keep prose when it is intentionally unchecked.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::NEEDS_CONTRACT_VIOLATION,
        default_severity: Severity::Error,
        explanation: "A runtime `needs:` predicate evaluated to false at task entry, so the caller or calling context broke the precondition.",
        repair: "Change the call arguments or add a caller-side guard before invoking the task.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::ENSURES_CONTRACT_VIOLATION,
        default_severity: Severity::Error,
        explanation: "A runtime `ensures:` predicate evaluated to false after a successful return, so the task implementation broke its own success promise.",
        repair: "Fix the task body or adjust the contract to match the intended result.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INVALID_EXECUTABLE_PREDICATE,
        default_severity: Severity::Error,
        explanation: "A contract line signals executable Predicate v2 intent but has malformed syntax, an ineligible or unresolved place, or an unsupported operand/operator type. It is rejected before evaluation rather than treated as prose.",
        repair: "Use one complete typed comparison over eligible task parameters or `result`, with exact Text/List Text equality or `list_count(List Text, Text)` where appropriate.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INVALID_CALLABLE_FORM,
        default_severity: Severity::Error,
        explanation: "A callable type, task value, or indirect call signals passed-callable intent but falls outside the one-parameter pure callable slice.",
        repair: "Use one same-file `task(UInt) -> UInt` value, pass it to one receiver, and invoke it exactly once as `transform(value)`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::CALLABLE_SIGNATURE_MISMATCH,
        default_severity: Severity::Error,
        explanation: "A passed task value does not match the receiver's exact callable input, result, or closed-empty latent-effect requirements.",
        repair: "Pass a same-file infallible `task(UInt) -> UInt` whose inferred latent effect row is closed and empty.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::USE_AFTER_MOVE,
        default_severity: Severity::Error,
        explanation: "A value was used after an earlier `consume` argument or return moved its ownership authority away from the current place.",
        repair: "Use the value before the move site named in the help text, or create and pass a fresh owned value.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::BORROW_PARAMETER_MUTATION,
        default_severity: Severity::Error,
        explanation: "A parameter with default `borrow` permission was targeted by `set` or used to acquire a writable field alias, which would hide mutation behind a read-only signature.",
        repair: "Mark the parameter `change`, copy into a `change` local before acquiring the alias or writing, or remove the mutation.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED,
        default_severity: Severity::Error,
        explanation: "A recognized linear resource reached a return, failure, or fallthrough path without exactly one visible `consume` action.",
        repair: "Commit, rollback, close, or transfer the resource exactly once on the path named in the help text.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE,
        default_severity: Severity::Error,
        explanation: "A recognized linear resource was consumed after an earlier commit, rollback, close, transfer, or consume already ended it on that path.",
        repair: "Remove the second consume, split the control flow, or create a fresh resource before consuming again.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER,
        default_severity: Severity::Error,
        explanation: "A returned-view `from` relationship names a source that is not a task parameter, or the returned expression does not visibly come from that source.",
        repair: "Name a parameter after `from` and return that bare parameter in the V0 subset; locals, internal references, and complex expressions remain explicit future repairs.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::ITERATION_MUTATION_CONFLICT,
        default_severity: Severity::Error,
        explanation: "A list was structurally mutated while a `for each` loop was actively iterating that same collection.",
        repair: "Collect mutations after the loop, or iterate over a separate snapshot/list before calling `list_append` on the original collection.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::STALE_FIELD_VIEW,
        default_severity: Severity::Error,
        explanation: "A local field or element view was used after the source it viewed was changed by a recognized invalidating operation.",
        repair: "Re-borrow the view after the write or append, or bind a value copy before the invalidating operation if stale observation was intended.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::WRITABLE_ALIAS_OVERLAP,
        default_severity: Severity::Error,
        explanation: "A direct field is accessed through another path while a writable alias to overlapping storage is still live.",
        repair: "Use a definitely distinct direct field, or move the access after the writable alias's last syntactic use.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS,
        default_severity: Severity::Error,
        explanation: "A writable alias uses a shape or lifetime outside Hum's narrow straight-line direct-field slice.",
        repair: "Keep the alias local and straight-line over one direct field; do not pass, return, store, nest, rebind, or use it across control flow.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY,
        default_severity: Severity::Error,
        explanation: "A known task with a nominal error result was called without explicit propagation or wrapping.",
        repair: "Write `try call()` for a matching caller error root, or `try call() or fail CallerError.context` to preserve and wrap the cause.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION,
        default_severity: Severity::Error,
        explanation: "An unwrapped `try` would propagate a callee error root different from the caller's declared error root.",
        repair: "Wrap the cause under the caller's root or make the nominal roots equal deliberately.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::FAILURE_WRAPPER_ROOT_MISMATCH,
        default_severity: Severity::Error,
        explanation: "A causal wrapper uses a nominal root different from the caller's declared error root.",
        repair: "Use a wrapper variant under the caller's declared error root.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TRY_ON_INFALLIBLE_CALL,
        default_severity: Severity::Error,
        explanation: "`try` was applied to a known task whose result has no typed error root.",
        repair: "Remove `try` and call the infallible task directly.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::DIRECT_FAILURE_ROOT_MISMATCH,
        default_severity: Severity::Error,
        explanation: "A direct `fail Root.variant` uses a root different from the enclosing task result.",
        repair: "Fail with a variant under the caller's declared error root or change the result deliberately.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNSUPPORTED_TRY_EXPRESSION,
        default_severity: Severity::Error,
        explanation: "A `try` expression is outside the exact direct named-call Session W slice.",
        repair: "Use `let value = try named_call(...)` with ordinary value arguments, optionally followed by `or fail CallerError.context`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::MISSING_FAILURE_DECLARATION,
        default_severity: Severity::Error,
        explanation: "A task can directly fail, propagate, or wrap a typed failure but has no meaningful `fails when:` declaration.",
        repair: "Add a concrete `fails when:` condition for the visible failure path.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNKNOWN_TARGET_FACT_RECORD,
        default_severity: Severity::Error,
        explanation: "A `targets:` section names a target fact record that Hum does not publish in `hum target-facts`.",
        repair: "Use one of the fixture record IDs from `hum target-facts --format json`, or add the record to the target facts catalog before depending on it.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNKNOWN_CAPABILITY_FAMILY,
        default_severity: Severity::Error,
        explanation: "A `targets:` section names a capability family that Hum does not publish in `hum target-facts`.",
        repair: "Use one of the capability families from `hum target-facts --format json`, or add the family to the target facts catalog first.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNSUPPORTED_TARGET_DECLARATION,
        default_severity: Severity::Error,
        explanation: "A meaningful `targets:` line does not use a current formal key, so Hum would otherwise ignore a portability promise.",
        repair: "Use `triple:`, `record:`, `target:`, `requires:`, or `denies:` for Milestone 0 target declarations.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::REQUIRED_CAPABILITY_UNAVAILABLE,
        default_severity: Severity::Error,
        explanation: "A `targets:` section requires a capability family that a declared target fact record marks absent or unavailable.",
        repair: "Choose a target that provides the capability, remove the requirement, or add an adapter/profile design before depending on it.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::CONFLICTING_TARGET_CAPABILITY,
        default_severity: Severity::Error,
        explanation: "A `targets:` section both requires and denies the same capability family, making the portability intent contradictory.",
        repair: "Remove one declaration, or split the target policy into separate tasks/profiles with different capability intent.",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RegistrySummary {
    active_codes: usize,
    retired_codes: usize,
    reserved_families: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistryValidationError {
    MalformedFamilyInterval,
    DuplicateFamilyKey,
    MalformedRegistryMetadata,
    FamilyIntervalOwnerConflict,
    OverlappingFamilyIntervals,
    MalformedCodeSpelling,
    DuplicateCodeSpelling,
    DuplicateCodeKey,
    NonDeterministicCodeOrder,
    InvalidPublicOrdinal,
    CodeOutsideDeclaredFamily,
    CodeOwnedByDifferentFamily,
    ReservedFamilyExactAllocation,
    ReservedExactCodeAllocation,
    MissingCatalogDetail,
    DuplicateCatalogDetail,
    UnknownCatalogDetail,
    CatalogProjectionMismatch,
    CatalogProjectionOrderMismatch,
    RetiredCodeReuseOrMutation,
    DuplicateCauseKey,
    DuplicateCauseReason,
    UnknownCauseCode,
    MalformedCauseMetadata,
    MissingCauseCoverage,
    MalformedPrecedenceRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RetiredCodeRecord {
    allocation: DiagnosticCodeAllocation,
    default_severity: Severity,
    explanation: &'static str,
    repair: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CatalogProjection<'a> {
    key: DiagnosticCodeKey,
    spelling: &'a str,
    title: &'a str,
    default_severity: Severity,
    explanation: &'a str,
    repair: &'a str,
}

const FROZEN_RETIRED_CODES: &[RetiredCodeRecord] = &[];

fn parse_code_spelling(spelling: &str) -> Result<u16, RegistryValidationError> {
    let bytes = spelling.as_bytes();
    if bytes.len() != 5 || bytes[0] != b'H' || !bytes[1..].iter().all(u8::is_ascii_digit) {
        return Err(RegistryValidationError::MalformedCodeSpelling);
    }
    spelling[1..]
        .parse::<u16>()
        .map_err(|_| RegistryValidationError::MalformedCodeSpelling)
}

fn validate_registry(
    families: &[DiagnosticFamilySpec],
    allocations: &[DiagnosticCodeAllocation],
    details: &[DiagnosticInfo],
    frozen_retired: &[RetiredCodeRecord],
) -> Result<RegistrySummary, RegistryValidationError> {
    for (index, family) in families.iter().enumerate() {
        if family.start > family.end || family.end > 9999 {
            return Err(RegistryValidationError::MalformedFamilyInterval);
        }
        for other in &families[..index] {
            if family.key == other.key {
                return Err(RegistryValidationError::DuplicateFamilyKey);
            }
            if family.start == other.start
                && family.end == other.end
                && family.semantic_owner != other.semantic_owner
            {
                return Err(RegistryValidationError::FamilyIntervalOwnerConflict);
            }
            if family.start <= other.end && other.start <= family.end {
                return Err(RegistryValidationError::OverlappingFamilyIntervals);
            }
        }
        if family.key.0.is_empty()
            || family.semantic_owner.is_empty()
            || family.key.0 != family.semantic_owner
            || family.doctrine.is_empty()
            || family.doctrine.iter().any(|reference| reference.is_empty())
        {
            return Err(RegistryValidationError::MalformedRegistryMetadata);
        }
    }

    let mut previous_number = None;
    for (index, allocation) in allocations.iter().enumerate() {
        if allocation.title.is_empty()
            || allocation.semantic_owner.is_empty()
            || allocation.owning_stage.is_empty()
        {
            return Err(RegistryValidationError::MalformedRegistryMetadata);
        }
        for other in &allocations[..index] {
            if allocation.spelling == other.spelling {
                return Err(RegistryValidationError::DuplicateCodeSpelling);
            }
            if allocation.key == other.key {
                return Err(RegistryValidationError::DuplicateCodeKey);
            }
            if allocation.public_ordinal == other.public_ordinal {
                return Err(RegistryValidationError::InvalidPublicOrdinal);
            }
        }
        if usize::from(allocation.public_ordinal) >= allocations.len() {
            return Err(RegistryValidationError::InvalidPublicOrdinal);
        }

        let number = parse_code_spelling(allocation.spelling)?;
        if let Some(previous) = previous_number
            && number <= previous
        {
            return Err(RegistryValidationError::NonDeterministicCodeOrder);
        }
        previous_number = Some(number);

        if allocation.status == AllocationStatus::Reserved {
            return Err(RegistryValidationError::ReservedExactCodeAllocation);
        }
        let declared_family = families
            .iter()
            .find(|family| family.key == allocation.family)
            .ok_or(RegistryValidationError::CodeOutsideDeclaredFamily)?;
        if number < declared_family.start || number > declared_family.end {
            return Err(RegistryValidationError::CodeOutsideDeclaredFamily);
        }
        let numeric_family = families
            .iter()
            .find(|family| number >= family.start && number <= family.end)
            .ok_or(RegistryValidationError::CodeOutsideDeclaredFamily)?;
        if numeric_family.key != allocation.family
            || numeric_family.semantic_owner != allocation.semantic_owner
        {
            return Err(RegistryValidationError::CodeOwnedByDifferentFamily);
        }
        if numeric_family.status == AllocationStatus::Reserved {
            return Err(RegistryValidationError::ReservedFamilyExactAllocation);
        }
    }

    for (index, detail) in details.iter().enumerate() {
        let key = detail.code.key();
        if details[..index].iter().any(|other| other.code.key() == key) {
            return Err(RegistryValidationError::DuplicateCatalogDetail);
        }
        if !allocations.iter().any(|allocation| allocation.key == key) {
            return Err(RegistryValidationError::UnknownCatalogDetail);
        }
    }
    if allocations.iter().any(|allocation| {
        !details
            .iter()
            .any(|detail| detail.code.key() == allocation.key)
    }) {
        return Err(RegistryValidationError::MissingCatalogDetail);
    }

    validate_retired_history(allocations, details, frozen_retired)?;

    Ok(RegistrySummary {
        active_codes: allocations
            .iter()
            .filter(|allocation| allocation.status == AllocationStatus::Active)
            .count(),
        retired_codes: allocations
            .iter()
            .filter(|allocation| allocation.status == AllocationStatus::Retired)
            .count(),
        reserved_families: families
            .iter()
            .filter(|family| family.status == AllocationStatus::Reserved)
            .count(),
    })
}

fn validate_retired_history(
    allocations: &[DiagnosticCodeAllocation],
    details: &[DiagnosticInfo],
    frozen_retired: &[RetiredCodeRecord],
) -> Result<(), RegistryValidationError> {
    for prior in frozen_retired {
        let Some(current) = allocations.iter().find(|allocation| {
            allocation.key == prior.allocation.key
                || allocation.spelling == prior.allocation.spelling
        }) else {
            return Err(RegistryValidationError::RetiredCodeReuseOrMutation);
        };
        let Some(detail) = details
            .iter()
            .find(|detail| detail.code.key() == current.key)
        else {
            return Err(RegistryValidationError::RetiredCodeReuseOrMutation);
        };
        if current != &prior.allocation
            || current.status != AllocationStatus::Retired
            || detail.default_severity != prior.default_severity
            || detail.explanation != prior.explanation
            || detail.repair != prior.repair
        {
            return Err(RegistryValidationError::RetiredCodeReuseOrMutation);
        }
    }
    Ok(())
}

fn catalog_projection(details: &[DiagnosticInfo]) -> Vec<CatalogProjection<'static>> {
    details
        .iter()
        .map(|detail| CatalogProjection {
            key: detail.code.key(),
            spelling: detail.code.as_str(),
            title: detail.code.title(),
            default_severity: detail.default_severity,
            explanation: detail.explanation,
            repair: detail.repair,
        })
        .collect()
}

fn validate_catalog_projection(
    allocations: &[DiagnosticCodeAllocation],
    details: &[DiagnosticInfo],
    projection: &[CatalogProjection<'_>],
) -> Result<(), RegistryValidationError> {
    if projection.len() != allocations.len() || projection.len() != details.len() {
        return Err(RegistryValidationError::CatalogProjectionMismatch);
    }
    for (index, projected) in projection.iter().enumerate() {
        if projection[..index]
            .iter()
            .any(|other| other.key == projected.key)
        {
            return Err(RegistryValidationError::CatalogProjectionMismatch);
        }
        let Some(allocation) = allocations
            .iter()
            .find(|allocation| allocation.key == projected.key)
        else {
            return Err(RegistryValidationError::CatalogProjectionMismatch);
        };
        let Some(detail) = details
            .iter()
            .find(|detail| detail.code.key() == projected.key)
        else {
            return Err(RegistryValidationError::CatalogProjectionMismatch);
        };
        if usize::from(allocation.public_ordinal) != index {
            return Err(RegistryValidationError::CatalogProjectionOrderMismatch);
        }
        if projected.spelling != allocation.spelling
            || projected.title != allocation.title
            || projected.default_severity != detail.default_severity
            || projected.explanation != detail.explanation
            || projected.repair != detail.repair
        {
            return Err(RegistryValidationError::CatalogProjectionMismatch);
        }
    }
    Ok(())
}

fn validate_static_registry() -> Result<RegistrySummary, RegistryValidationError> {
    let summary = validate_registry(
        DIAGNOSTIC_FAMILIES,
        DIAGNOSTIC_CODE_ALLOCATIONS,
        DIAGNOSTICS,
        FROZEN_RETIRED_CODES,
    )?;
    validate_catalog_projection(
        DIAGNOSTIC_CODE_ALLOCATIONS,
        DIAGNOSTICS,
        &catalog_projection(DIAGNOSTICS),
    )?;
    validate_causes(DIAGNOSTIC_CAUSES, DIAGNOSTIC_PRECEDENCE)?;
    Ok(summary)
}

fn validate_causes(
    causes: &[DiagnosticCauseSpec],
    precedence: &[DiagnosticPrecedenceSpec],
) -> Result<(), RegistryValidationError> {
    for (index, cause) in causes.iter().enumerate() {
        if causes[..index].iter().any(|other| other.key == cause.key) {
            return Err(RegistryValidationError::DuplicateCauseKey);
        }
        if causes[..index]
            .iter()
            .any(|other| other.code == cause.code && other.reason == cause.reason)
        {
            return Err(RegistryValidationError::DuplicateCauseReason);
        }
        if !DIAGNOSTIC_CODE_ALLOCATIONS
            .iter()
            .any(|allocation| allocation.key == cause.code.key())
        {
            return Err(RegistryValidationError::UnknownCauseCode);
        }
        if cause.reason.is_empty()
            || cause.semantic_owner.is_empty()
            || cause.owning_stage.is_empty()
            || cause.origin_kind.is_empty()
            || cause.route_kind.is_empty()
        {
            return Err(RegistryValidationError::MalformedCauseMetadata);
        }
    }
    for code in [
        DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY,
        DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION,
        DiagnosticCode::FAILURE_WRAPPER_ROOT_MISMATCH,
        DiagnosticCode::TRY_ON_INFALLIBLE_CALL,
        DiagnosticCode::DIRECT_FAILURE_ROOT_MISMATCH,
        DiagnosticCode::UNSUPPORTED_TRY_EXPRESSION,
        DiagnosticCode::MISSING_FAILURE_DECLARATION,
        DiagnosticCode::INVALID_CALLABLE_FORM,
        DiagnosticCode::CALLABLE_SIGNATURE_MISMATCH,
    ] {
        if !causes.iter().any(|cause| cause.code == code) {
            return Err(RegistryValidationError::MissingCauseCoverage);
        }
    }
    if causes != DIAGNOSTIC_CAUSES {
        return Err(RegistryValidationError::MalformedCauseMetadata);
    }
    for rule in precedence {
        if rule.relationship.is_empty()
            || rule.applying_owner.is_empty()
            || rule.dominant_causes.is_empty()
            || rule.suppressed_causes.is_empty()
            || rule
                .dominant_causes
                .iter()
                .chain(rule.suppressed_causes.iter())
                .any(|key| !causes.iter().any(|cause| cause.key == *key))
            || rule
                .dominant_causes
                .iter()
                .enumerate()
                .any(|(index, key)| rule.dominant_causes[..index].contains(key))
            || rule
                .suppressed_causes
                .iter()
                .enumerate()
                .any(|(index, key)| rule.suppressed_causes[..index].contains(key))
        {
            return Err(RegistryValidationError::MalformedPrecedenceRule);
        }
    }
    if precedence != DIAGNOSTIC_PRECEDENCE {
        return Err(RegistryValidationError::MalformedPrecedenceRule);
    }
    Ok(())
}

pub fn all() -> &'static [DiagnosticInfo] {
    validate_static_registry().expect("canonical diagnostic registry must be valid");
    DIAGNOSTICS
}

pub fn find(code: &str) -> Option<&'static DiagnosticInfo> {
    let code = code.trim();
    all()
        .iter()
        .find(|info| info.code.as_str().eq_ignore_ascii_case(code))
}

#[cfg(test)]
mod tests {
    use super::{
        AllocationStatus, CatalogProjection, DIAGNOSTIC_CAUSES, DIAGNOSTIC_CODE_ALLOCATIONS,
        DIAGNOSTIC_FAMILIES, DIAGNOSTIC_PRECEDENCE, DIAGNOSTICS, DiagnosticCodeAllocation,
        DiagnosticCodeKey, DiagnosticFamilyKey, DiagnosticInfo, RegistryValidationError,
        RetiredCodeRecord, UNALLOCATED_PROFILE_DIAGNOSTIC, all, catalog_projection, find,
        parse_code_spelling, validate_catalog_projection, validate_causes, validate_registry,
        validate_retired_history, validate_static_registry,
    };
    use crate::diagnostic::DiagnosticCode;

    #[test]
    fn catalog_contains_known_codes() {
        assert_eq!(
            find("H0201").map(|info| info.code),
            Some(DiagnosticCode::UNDECLARED_SAVE_TARGET)
        );
        assert_eq!(
            find("h0502").map(|info| info.code),
            Some(DiagnosticCode::REGRESSION_MISSING_NOTE)
        );
        assert_eq!(
            find("H0601").map(|info| info.code),
            Some(DiagnosticCode::UNRESOLVED_NAME)
        );
        assert_eq!(
            find("H0604").map(|info| info.code),
            Some(DiagnosticCode::READ_BEFORE_DECLARE)
        );
        assert_eq!(
            find("H0605").map(|info| info.code),
            Some(DiagnosticCode::UNKNOWN_TYPE_NAME)
        );
        assert_eq!(
            find("H0606").map(|info| info.code),
            Some(DiagnosticCode::RETURN_TYPE_MISMATCH)
        );
        assert_eq!(
            find("H0701").map(|info| info.code),
            Some(DiagnosticCode::UNCHECKED_PROSE_CONTRACT)
        );
        assert_eq!(
            find("H0703").map(|info| info.code),
            Some(DiagnosticCode::ENSURES_CONTRACT_VIOLATION)
        );
        assert_eq!(
            find("H0801").map(|info| info.code),
            Some(DiagnosticCode::USE_AFTER_MOVE)
        );
        assert_eq!(
            find("H0802").map(|info| info.code),
            Some(DiagnosticCode::BORROW_PARAMETER_MUTATION)
        );
        assert_eq!(
            find("H0803").map(|info| info.code),
            Some(DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED)
        );
        assert_eq!(
            find("H0804").map(|info| info.code),
            Some(DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE)
        );
        assert_eq!(
            find("H0805").map(|info| info.code),
            Some(DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER)
        );
        assert_eq!(
            find("H0807").map(|info| info.code),
            Some(DiagnosticCode::STALE_FIELD_VIEW)
        );
        assert_eq!(
            find("H0808").map(|info| info.code),
            Some(DiagnosticCode::WRITABLE_ALIAS_OVERLAP)
        );
        assert_eq!(
            find("H0809").map(|info| info.code),
            Some(DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS)
        );
        assert_eq!(
            find("H0907").map(|info| info.code),
            Some(DiagnosticCode::MISSING_FAILURE_DECLARATION)
        );
        assert_eq!(
            find("H1201").map(|info| info.code),
            Some(DiagnosticCode::UNKNOWN_TARGET_FACT_RECORD)
        );
        assert_eq!(
            find("H1204").map(|info| info.code),
            Some(DiagnosticCode::REQUIRED_CAPABILITY_UNAVAILABLE)
        );
        assert_eq!(
            find("H1205").map(|info| info.code),
            Some(DiagnosticCode::CONFLICTING_TARGET_CAPABILITY)
        );
        assert_eq!(
            find("H1401").map(|info| info.code),
            Some(DiagnosticCode::INVALID_CALLABLE_FORM)
        );
        assert_eq!(
            find("H1402").map(|info| info.code),
            Some(DiagnosticCode::CALLABLE_SIGNATURE_MISMATCH)
        );
        assert!(find("H9999").is_none());
    }

    #[test]
    fn catalog_has_unique_codes() {
        let mut codes = all()
            .iter()
            .map(|info| info.code.as_str())
            .collect::<Vec<_>>();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), all().len());
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum DocumentKind {
        HumanCatalog,
        DiagnosticsSchema,
        EffectReport,
        Security,
        Unsafe,
        RuntimeProfiles,
        LanguageSubset,
        Portability,
    }

    #[derive(Clone, Copy)]
    struct CheckedDocument<'a> {
        kind: DocumentKind,
        path: &'static str,
        text: &'a str,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DocumentValidationError {
        MissingAuthorityLink,
        UnknownExactCode,
        WildcardCodeEscape,
        ContradictoryExactCode,
        ContradictoryFamilyOwner,
        MissingActiveCode,
        MissingFamily,
        InvalidPlaceholder,
    }

    fn checked_documents() -> [CheckedDocument<'static>; 8] {
        [
            CheckedDocument {
                kind: DocumentKind::HumanCatalog,
                path: "docs/DIAGNOSTICS.md",
                text: include_str!("../docs/DIAGNOSTICS.md"),
            },
            CheckedDocument {
                kind: DocumentKind::DiagnosticsSchema,
                path: "docs/DIAGNOSTICS_SCHEMA_0_1.md",
                text: include_str!("../docs/DIAGNOSTICS_SCHEMA_0_1.md"),
            },
            CheckedDocument {
                kind: DocumentKind::EffectReport,
                path: "docs/EFFECT_REPORT_SCHEMA_0_1.md",
                text: include_str!("../docs/EFFECT_REPORT_SCHEMA_0_1.md"),
            },
            CheckedDocument {
                kind: DocumentKind::Security,
                path: "docs/SECURITY_MODEL.md",
                text: include_str!("../docs/SECURITY_MODEL.md"),
            },
            CheckedDocument {
                kind: DocumentKind::Unsafe,
                path: "docs/UNSAFE_POLICY.md",
                text: include_str!("../docs/UNSAFE_POLICY.md"),
            },
            CheckedDocument {
                kind: DocumentKind::RuntimeProfiles,
                path: "docs/RUNTIME_PROFILES.md",
                text: include_str!("../docs/RUNTIME_PROFILES.md"),
            },
            CheckedDocument {
                kind: DocumentKind::LanguageSubset,
                path: "docs/LANGUAGE_SUBSET_0_1.md",
                text: include_str!("../docs/LANGUAGE_SUBSET_0_1.md"),
            },
            CheckedDocument {
                kind: DocumentKind::Portability,
                path: "docs/PORTABILITY_BOUNDARY_MODEL.md",
                text: include_str!("../docs/PORTABILITY_BOUNDARY_MODEL.md"),
            },
        ]
    }

    fn status_text(status: AllocationStatus) -> &'static str {
        match status {
            AllocationStatus::Active => "active",
            AllocationStatus::Reserved => "reserved",
            AllocationStatus::Retired => "retired",
        }
    }

    fn strip_ticks(value: &str) -> &str {
        value.trim().trim_matches('`')
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct ExactCodeToken {
        number: u16,
        start: usize,
    }

    fn is_code_shape_lead(byte: u8) -> bool {
        byte.is_ascii_digit() || matches!(byte, b'x' | b'X' | b'?' | b'*' | b'_' | b'#')
    }

    fn is_code_shape_continuation(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || matches!(byte, b'?' | b'*' | b'_' | b'#')
    }

    fn exact_code_tokens(text: &str) -> Result<Vec<ExactCodeToken>, DocumentValidationError> {
        let bytes = text.as_bytes();
        let mut codes = Vec::new();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] != b'H'
                || (index > 0
                    && (bytes[index - 1].is_ascii_alphanumeric() || bytes[index - 1] == b'_'))
                || index + 1 == bytes.len()
                || !is_code_shape_lead(bytes[index + 1])
            {
                index += 1;
                continue;
            }

            let has_exact_digits = index + 5 <= bytes.len()
                && bytes[index + 1..index + 5].iter().all(u8::is_ascii_digit);
            if !has_exact_digits {
                return Err(DocumentValidationError::WildcardCodeEscape);
            }

            let exact_end = index + 5;
            let is_range = exact_end + 6 <= bytes.len()
                && bytes[exact_end] == b'-'
                && bytes[exact_end + 1] == b'H'
                && bytes[exact_end + 2..exact_end + 6]
                    .iter()
                    .all(u8::is_ascii_digit)
                && (exact_end + 6 == bytes.len()
                    || !is_code_shape_continuation(bytes[exact_end + 6]));
            if is_range {
                index = exact_end + 6;
                continue;
            }
            if exact_end < bytes.len() && is_code_shape_continuation(bytes[exact_end]) {
                return Err(DocumentValidationError::WildcardCodeEscape);
            }

            let spelling = &text[index..exact_end];
            let number = parse_code_spelling(spelling)
                .map_err(|_| DocumentValidationError::UnknownExactCode)?;
            if !DIAGNOSTIC_CODE_ALLOCATIONS
                .iter()
                .any(|allocation| allocation.spelling == spelling)
            {
                return Err(DocumentValidationError::UnknownExactCode);
            }
            codes.push(ExactCodeToken {
                number,
                start: index,
            });
            index = exact_end;
        }
        Ok(codes)
    }

    fn validate_schema_exact_code_claim(
        text: &str,
        token: ExactCodeToken,
    ) -> Result<(), DocumentValidationError> {
        let spelling = &text[token.start..token.start + 5];
        let allocation = DIAGNOSTIC_CODE_ALLOCATIONS
            .iter()
            .find(|allocation| allocation.spelling == spelling)
            .ok_or(DocumentValidationError::UnknownExactCode)?;
        let detail = DIAGNOSTICS
            .iter()
            .find(|detail| detail.code.key() == allocation.key)
            .ok_or(DocumentValidationError::MissingActiveCode)?;
        let object_start = text[..token.start]
            .rfind('{')
            .ok_or(DocumentValidationError::ContradictoryExactCode)?;
        let object_end = token.start
            + text[token.start..]
                .find('}')
                .ok_or(DocumentValidationError::ContradictoryExactCode)?;
        let object = &text[object_start..=object_end];
        let expected_code = format!("\"code\": \"{}\"", allocation.spelling);
        let expected_title = format!("\"title\": \"{}\"", allocation.title);
        let expected_severity = format!("\"severity\": \"{}\"", detail.default_severity.as_str());
        if !object.contains(&expected_code)
            || !object.contains(&expected_title)
            || !object.contains(&expected_severity)
        {
            return Err(DocumentValidationError::ContradictoryExactCode);
        }
        Ok(())
    }

    fn validate_document_tokens(
        kind: DocumentKind,
        text: &str,
    ) -> Result<(), DocumentValidationError> {
        let lines = text.lines().collect::<Vec<_>>();
        for line in &lines {
            let bytes = line.as_bytes();
            for index in 0..bytes.len().saturating_sub(10) {
                let range = &bytes[index..index + 11];
                if range[0] != b'H'
                    || range[5] != b'-'
                    || range[6] != b'H'
                    || !range[1..5].iter().all(u8::is_ascii_digit)
                    || !range[7..11].iter().all(u8::is_ascii_digit)
                {
                    continue;
                }
                let start = std::str::from_utf8(&range[1..5])
                    .expect("ASCII family start")
                    .parse::<u16>()
                    .expect("numeric family start");
                let end = std::str::from_utf8(&range[7..11])
                    .expect("ASCII family end")
                    .parse::<u16>()
                    .expect("numeric family end");
                let Some(family) = DIAGNOSTIC_FAMILIES
                    .iter()
                    .find(|family| family.start == start && family.end == end)
                else {
                    return Err(DocumentValidationError::ContradictoryFamilyOwner);
                };
                if !line.contains(family.key.0) {
                    return Err(DocumentValidationError::ContradictoryFamilyOwner);
                }
            }
        }

        let codes = exact_code_tokens(text)?;
        if kind == DocumentKind::DiagnosticsSchema {
            for code in codes {
                validate_schema_exact_code_claim(text, code)?;
            }
        } else if kind != DocumentKind::HumanCatalog && !codes.is_empty() {
            return Err(DocumentValidationError::ContradictoryExactCode);
        }

        let placeholder_count = text.matches(UNALLOCATED_PROFILE_DIAGNOSTIC).count();
        let other_placeholder = text
            .replace(UNALLOCATED_PROFILE_DIAGNOSTIC, "")
            .contains("<unallocated-");
        if other_placeholder
            || (kind == DocumentKind::RuntimeProfiles && placeholder_count != 1)
            || (kind != DocumentKind::RuntimeProfiles && placeholder_count != 0)
        {
            return Err(DocumentValidationError::InvalidPlaceholder);
        }
        Ok(())
    }

    fn validate_human_projection(text: &str) -> Result<(), DocumentValidationError> {
        let mut code_rows = Vec::new();
        for line in text
            .lines()
            .filter(|line| line.trim_start().starts_with('|'))
        {
            let cells = line.split('|').map(str::trim).collect::<Vec<_>>();
            if cells.len() < 5 {
                continue;
            }
            let code = strip_ticks(cells[1]);
            if code.len() == 5
                && code.starts_with('H')
                && code[1..].bytes().all(|byte| byte.is_ascii_digit())
            {
                let Some(allocation) = DIAGNOSTIC_CODE_ALLOCATIONS
                    .iter()
                    .find(|allocation| allocation.spelling == code)
                else {
                    return Err(DocumentValidationError::UnknownExactCode);
                };
                let Some(detail) = DIAGNOSTICS
                    .iter()
                    .find(|detail| detail.code.key() == allocation.key)
                else {
                    return Err(DocumentValidationError::MissingActiveCode);
                };
                if cells[2] != detail.default_severity.as_str() || cells[3] != allocation.title {
                    return Err(DocumentValidationError::ContradictoryExactCode);
                }
                if code_rows.contains(&allocation.key) {
                    return Err(DocumentValidationError::ContradictoryExactCode);
                }
                code_rows.push(allocation.key);
            }
        }
        if DIAGNOSTIC_CODE_ALLOCATIONS
            .iter()
            .filter(|allocation| allocation.status == AllocationStatus::Active)
            .any(|allocation| !code_rows.contains(&allocation.key))
        {
            return Err(DocumentValidationError::MissingActiveCode);
        }

        for family in DIAGNOSTIC_FAMILIES {
            let range = format!("H{:04}-H{:04}", family.start, family.end);
            let matching = text.lines().find(|line| {
                let cells = line.split('|').map(str::trim).collect::<Vec<_>>();
                cells.len() >= 5 && strip_ticks(cells[1]) == range
            });
            let Some(line) = matching else {
                return Err(DocumentValidationError::MissingFamily);
            };
            let cells = line.split('|').map(str::trim).collect::<Vec<_>>();
            if cells[2] != status_text(family.status) || strip_ticks(cells[3]) != family.key.0 {
                return Err(DocumentValidationError::ContradictoryFamilyOwner);
            }
        }
        Ok(())
    }

    fn validate_checked_documents(
        documents: &[CheckedDocument<'_>],
    ) -> Result<(), DocumentValidationError> {
        for document in documents {
            if !document.text.contains("../src/diagnostic_catalog.rs")
                || (document.kind != DocumentKind::HumanCatalog
                    && !document.text.contains("DIAGNOSTICS.md"))
                || (document.kind == DocumentKind::HumanCatalog
                    && !document.text.contains("human projection"))
            {
                return Err(DocumentValidationError::MissingAuthorityLink);
            }
            validate_document_tokens(document.kind, document.text)?;
            if document.kind == DocumentKind::HumanCatalog {
                validate_human_projection(document.text)?;
            }
            assert!(document.path.starts_with("docs/"));
        }
        Ok(())
    }

    fn one_detail(code: DiagnosticCode) -> DiagnosticInfo {
        DiagnosticInfo {
            code,
            default_severity: crate::diagnostic::Severity::Error,
            explanation: "frozen explanation",
            repair: "frozen repair",
        }
    }

    #[test]
    fn canonical_registry_and_checked_projections_are_valid() {
        let summary = validate_static_registry().expect("canonical registry");
        assert_eq!(summary.active_codes, 87);
        assert_eq!(summary.retired_codes, 0);
        assert_eq!(summary.reserved_families, 3);
        assert_eq!(validate_static_registry(), Ok(summary));
        validate_checked_documents(&checked_documents()).expect("checked documents");
        assert_eq!(DIAGNOSTIC_CAUSES.len(), 48);
        assert_eq!(DIAGNOSTIC_PRECEDENCE.len(), 3);
        for dominant in super::H090_CAUSES {
            for suppressed in super::H1401_CAUSES.iter().chain(super::H1402_CAUSES.iter()) {
                assert!(super::precedence_spec(*dominant, *suppressed).is_some());
            }
        }
        assert!(
            super::precedence_spec(super::DiagnosticCauseKey(44), super::DiagnosticCauseKey(31))
                .is_some()
        );
        assert!(
            super::precedence_spec(super::DiagnosticCauseKey(45), super::DiagnosticCauseKey(31))
                .is_none()
        );
    }

    #[test]
    fn cause_registry_rejects_every_identity_and_owner_mutation() {
        assert_eq!(
            validate_causes(DIAGNOSTIC_CAUSES, DIAGNOSTIC_PRECEDENCE),
            Ok(())
        );

        let mut duplicate_key = DIAGNOSTIC_CAUSES.to_vec();
        duplicate_key[1].key = duplicate_key[0].key;
        assert_eq!(
            validate_causes(&duplicate_key, DIAGNOSTIC_PRECEDENCE),
            Err(RegistryValidationError::DuplicateCauseKey)
        );

        let mut duplicate_reason = DIAGNOSTIC_CAUSES.to_vec();
        duplicate_reason[1].code = duplicate_reason[0].code;
        duplicate_reason[1].reason = duplicate_reason[0].reason;
        assert_eq!(
            validate_causes(&duplicate_reason, DIAGNOSTIC_PRECEDENCE),
            Err(RegistryValidationError::DuplicateCauseReason)
        );

        for field in 0..4 {
            let mut malformed = DIAGNOSTIC_CAUSES.to_vec();
            match field {
                0 => malformed[0].semantic_owner = "",
                1 => malformed[0].owning_stage = "",
                2 => malformed[0].origin_kind = "",
                _ => malformed[0].route_kind = "",
            }
            assert_eq!(
                validate_causes(&malformed, DIAGNOSTIC_PRECEDENCE),
                Err(RegistryValidationError::MalformedCauseMetadata),
                "field {field}"
            );
        }

        for field in 0..2 {
            let mut malformed = DIAGNOSTIC_CAUSES.to_vec();
            if field == 0 {
                malformed[0].key = super::DiagnosticCauseKey(999);
            } else {
                malformed[0].reason = "replacement_reason_v0";
            }
            assert_eq!(
                validate_causes(&malformed, DIAGNOSTIC_PRECEDENCE),
                Err(RegistryValidationError::MalformedCauseMetadata),
                "identity field {field}"
            );
        }

        let without_h0907 = DIAGNOSTIC_CAUSES
            .iter()
            .copied()
            .filter(|cause| cause.code != DiagnosticCode::MISSING_FAILURE_DECLARATION)
            .collect::<Vec<_>>();
        assert_eq!(
            validate_causes(&without_h0907, DIAGNOSTIC_PRECEDENCE),
            Err(RegistryValidationError::MissingCauseCoverage)
        );

        let mut bad_precedence = DIAGNOSTIC_PRECEDENCE.to_vec();
        bad_precedence[0].relationship = "";
        assert_eq!(
            validate_causes(DIAGNOSTIC_CAUSES, &bad_precedence),
            Err(RegistryValidationError::MalformedPrecedenceRule)
        );
        for field in 0..4 {
            let mut malformed = DIAGNOSTIC_PRECEDENCE.to_vec();
            match field {
                0 => malformed[0].dominant_causes = &[super::DiagnosticCauseKey(999)],
                1 => malformed[0].suppressed_causes = &[super::DiagnosticCauseKey(999)],
                2 => malformed[0].relationship = "replacement_relationship_v0",
                _ => malformed[0].applying_owner = "replacement_owner",
            }
            assert_eq!(
                validate_causes(DIAGNOSTIC_CAUSES, &malformed),
                Err(RegistryValidationError::MalformedPrecedenceRule),
                "precedence field {field}"
            );
        }
    }

    #[test]
    fn family_interval_failures_are_independent() {
        let mut overlap = DIAGNOSTIC_FAMILIES.to_vec();
        overlap[1].start = overlap[0].end;
        assert_eq!(
            validate_registry(&overlap, DIAGNOSTIC_CODE_ALLOCATIONS, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::OverlappingFamilyIntervals)
        );

        let mut conflicting_owner = DIAGNOSTIC_FAMILIES.to_vec();
        conflicting_owner[1].start = conflicting_owner[0].start;
        conflicting_owner[1].end = conflicting_owner[0].end;
        conflicting_owner[1].semantic_owner = "different_owner";
        assert_eq!(
            validate_registry(
                &conflicting_owner,
                DIAGNOSTIC_CODE_ALLOCATIONS,
                DIAGNOSTICS,
                &[]
            ),
            Err(RegistryValidationError::FamilyIntervalOwnerConflict)
        );

        let mut duplicate_key = DIAGNOSTIC_FAMILIES.to_vec();
        duplicate_key[1].key = duplicate_key[0].key;
        assert_eq!(
            validate_registry(
                &duplicate_key,
                DIAGNOSTIC_CODE_ALLOCATIONS,
                DIAGNOSTICS,
                &[]
            ),
            Err(RegistryValidationError::DuplicateFamilyKey)
        );

        let mut missing_doctrine = DIAGNOSTIC_FAMILIES.to_vec();
        missing_doctrine[0].doctrine = &[];
        assert_eq!(
            validate_registry(
                &missing_doctrine,
                DIAGNOSTIC_CODE_ALLOCATIONS,
                DIAGNOSTICS,
                &[]
            ),
            Err(RegistryValidationError::MalformedRegistryMetadata)
        );
    }

    #[test]
    fn exact_code_identity_and_ownership_failures_are_independent() {
        let mut duplicate_spelling = DIAGNOSTIC_CODE_ALLOCATIONS.to_vec();
        duplicate_spelling[1].spelling = duplicate_spelling[0].spelling;
        assert_eq!(
            validate_registry(DIAGNOSTIC_FAMILIES, &duplicate_spelling, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::DuplicateCodeSpelling)
        );

        let mut duplicate_key = DIAGNOSTIC_CODE_ALLOCATIONS.to_vec();
        duplicate_key[1].key = duplicate_key[0].key;
        assert_eq!(
            validate_registry(DIAGNOSTIC_FAMILIES, &duplicate_key, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::DuplicateCodeKey)
        );

        let mut outside = DIAGNOSTIC_CODE_ALLOCATIONS.to_vec();
        outside[0].family = DiagnosticFamilyKey::INTENT_SHAPE;
        outside[0].semantic_owner = "intent_shape";
        assert_eq!(
            validate_registry(DIAGNOSTIC_FAMILIES, &outside, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::CodeOutsideDeclaredFamily)
        );

        let mut wrong_owner = DIAGNOSTIC_CODE_ALLOCATIONS.to_vec();
        wrong_owner[0].semantic_owner = "different_owner";
        assert_eq!(
            validate_registry(DIAGNOSTIC_FAMILIES, &wrong_owner, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::CodeOwnedByDifferentFamily)
        );

        wrong_owner[0].status = AllocationStatus::Retired;
        assert_eq!(
            validate_registry(DIAGNOSTIC_FAMILIES, &wrong_owner, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::CodeOwnedByDifferentFamily)
        );

        let mut reordered = DIAGNOSTIC_CODE_ALLOCATIONS.to_vec();
        reordered.swap(0, 1);
        assert_eq!(
            validate_registry(DIAGNOSTIC_FAMILIES, &reordered, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::NonDeterministicCodeOrder)
        );

        let mut duplicate_public_ordinal = DIAGNOSTIC_CODE_ALLOCATIONS.to_vec();
        duplicate_public_ordinal[1].public_ordinal = duplicate_public_ordinal[0].public_ordinal;
        assert_eq!(
            validate_registry(
                DIAGNOSTIC_FAMILIES,
                &duplicate_public_ordinal,
                DIAGNOSTICS,
                &[]
            ),
            Err(RegistryValidationError::InvalidPublicOrdinal)
        );

        let mut missing_public_ordinal = DIAGNOSTIC_CODE_ALLOCATIONS.to_vec();
        missing_public_ordinal[0].public_ordinal = u16::MAX;
        assert_eq!(
            validate_registry(
                DIAGNOSTIC_FAMILIES,
                &missing_public_ordinal,
                DIAGNOSTICS,
                &[]
            ),
            Err(RegistryValidationError::InvalidPublicOrdinal)
        );

        let mut missing_stage = DIAGNOSTIC_CODE_ALLOCATIONS.to_vec();
        missing_stage[0].owning_stage = "";
        assert_eq!(
            validate_registry(DIAGNOSTIC_FAMILIES, &missing_stage, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::MalformedRegistryMetadata)
        );
    }

    #[test]
    fn reserved_and_malformed_allocations_fail_closed() {
        let reserved_key = DiagnosticCodeKey(500);
        let reserved = DiagnosticCodeAllocation {
            key: reserved_key,
            public_ordinal: 0,
            spelling: "H1001",
            title: "must remain unallocated",
            family: DiagnosticFamilyKey::UNSAFE_FFI_PROVENANCE,
            semantic_owner: "unsafe_ffi_provenance",
            owning_stage: "none",
            status: AllocationStatus::Active,
        };
        let reserved_detail = one_detail(DiagnosticCode::from_key(reserved_key));
        assert_eq!(
            validate_registry(
                &[DIAGNOSTIC_FAMILIES[10]],
                &[reserved],
                &[reserved_detail],
                &[]
            ),
            Err(RegistryValidationError::ReservedFamilyExactAllocation)
        );

        let mut exact_reserved = DIAGNOSTIC_CODE_ALLOCATIONS.to_vec();
        exact_reserved[0].status = AllocationStatus::Reserved;
        assert_eq!(
            validate_registry(DIAGNOSTIC_FAMILIES, &exact_reserved, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::ReservedExactCodeAllocation)
        );

        let mut malformed = DIAGNOSTIC_CODE_ALLOCATIONS.to_vec();
        malformed[0].spelling = "H001";
        assert_eq!(
            validate_registry(DIAGNOSTIC_FAMILIES, &malformed, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::MalformedCodeSpelling)
        );

        let mut inverted = DIAGNOSTIC_FAMILIES.to_vec();
        inverted[0].start = 100;
        inverted[0].end = 99;
        assert_eq!(
            validate_registry(&inverted, DIAGNOSTIC_CODE_ALLOCATIONS, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::MalformedFamilyInterval)
        );

        let mut beyond = DIAGNOSTIC_FAMILIES.to_vec();
        beyond[14].end = 10_000;
        assert_eq!(
            validate_registry(&beyond, DIAGNOSTIC_CODE_ALLOCATIONS, DIAGNOSTICS, &[]),
            Err(RegistryValidationError::MalformedFamilyInterval)
        );
    }

    #[test]
    fn retired_allocations_are_append_only_and_semantically_frozen() {
        let mut allocation = DIAGNOSTIC_CODE_ALLOCATIONS[0];
        allocation.status = AllocationStatus::Retired;
        let detail = DIAGNOSTICS[0];
        let prior = RetiredCodeRecord {
            allocation,
            default_severity: detail.default_severity,
            explanation: detail.explanation,
            repair: detail.repair,
        };
        assert_eq!(
            validate_retired_history(&[allocation], &[detail], &[prior]),
            Ok(())
        );

        let mut reused = allocation;
        reused.status = AllocationStatus::Active;
        assert_eq!(
            validate_retired_history(&[reused], &[detail], &[prior]),
            Err(RegistryValidationError::RetiredCodeReuseOrMutation)
        );

        let mut mutated = allocation;
        mutated.title = "mutated retired meaning";
        assert_eq!(
            validate_retired_history(&[mutated], &[detail], &[prior]),
            Err(RegistryValidationError::RetiredCodeReuseOrMutation)
        );
    }

    #[test]
    fn public_catalog_projection_rejects_every_semantic_field_mismatch() {
        let baseline = catalog_projection(DIAGNOSTICS);
        assert_eq!(
            validate_catalog_projection(DIAGNOSTIC_CODE_ALLOCATIONS, DIAGNOSTICS, &baseline),
            Ok(())
        );

        for field in 0..6 {
            let mut mutation = baseline.clone();
            mutation[0] = match field {
                0 => CatalogProjection {
                    key: baseline[1].key,
                    ..mutation[0]
                },
                1 => CatalogProjection {
                    spelling: "H9999",
                    ..mutation[0]
                },
                2 => CatalogProjection {
                    title: "wrong title",
                    ..mutation[0]
                },
                3 => CatalogProjection {
                    default_severity: crate::diagnostic::Severity::Error,
                    ..mutation[0]
                },
                4 => CatalogProjection {
                    explanation: "wrong explanation",
                    ..mutation[0]
                },
                _ => CatalogProjection {
                    repair: "wrong repair",
                    ..mutation[0]
                },
            };
            if field == 3 && baseline[0].default_severity == crate::diagnostic::Severity::Error {
                mutation[0].default_severity = crate::diagnostic::Severity::Warning;
            }
            assert_eq!(
                validate_catalog_projection(DIAGNOSTIC_CODE_ALLOCATIONS, DIAGNOSTICS, &mutation),
                Err(if field == 0 {
                    RegistryValidationError::CatalogProjectionOrderMismatch
                } else {
                    RegistryValidationError::CatalogProjectionMismatch
                }),
                "field {field}"
            );
        }

        let mut swapped = baseline.clone();
        swapped.swap(0, 1);
        assert_eq!(
            validate_catalog_projection(DIAGNOSTIC_CODE_ALLOCATIONS, DIAGNOSTICS, &swapped),
            Err(RegistryValidationError::CatalogProjectionOrderMismatch)
        );

        let missing = &baseline[1..];
        assert_eq!(
            validate_catalog_projection(DIAGNOSTIC_CODE_ALLOCATIONS, DIAGNOSTICS, missing),
            Err(RegistryValidationError::CatalogProjectionMismatch)
        );

        let mut duplicate = baseline.clone();
        duplicate[1] = duplicate[0];
        assert_eq!(
            validate_catalog_projection(DIAGNOSTIC_CODE_ALLOCATIONS, DIAGNOSTICS, &duplicate),
            Err(RegistryValidationError::CatalogProjectionMismatch)
        );

        let mut extra = baseline.clone();
        extra.push(baseline[0]);
        assert_eq!(
            validate_catalog_projection(DIAGNOSTIC_CODE_ALLOCATIONS, DIAGNOSTICS, &extra),
            Err(RegistryValidationError::CatalogProjectionMismatch)
        );
    }

    #[test]
    fn catalog_detail_coverage_rejects_missing_duplicate_and_unknown_rows() {
        let missing = &DIAGNOSTICS[1..];
        assert_eq!(
            validate_registry(
                DIAGNOSTIC_FAMILIES,
                DIAGNOSTIC_CODE_ALLOCATIONS,
                missing,
                &[]
            ),
            Err(RegistryValidationError::MissingCatalogDetail)
        );

        let mut duplicate = DIAGNOSTICS.to_vec();
        duplicate.push(DIAGNOSTICS[0]);
        assert_eq!(
            validate_registry(
                DIAGNOSTIC_FAMILIES,
                DIAGNOSTIC_CODE_ALLOCATIONS,
                &duplicate,
                &[]
            ),
            Err(RegistryValidationError::DuplicateCatalogDetail)
        );

        let unknown = one_detail(DiagnosticCode::from_key(DiagnosticCodeKey(999)));
        let mut with_unknown = DIAGNOSTICS.to_vec();
        with_unknown.push(unknown);
        assert_eq!(
            validate_registry(
                DIAGNOSTIC_FAMILIES,
                DIAGNOSTIC_CODE_ALLOCATIONS,
                &with_unknown,
                &[]
            ),
            Err(RegistryValidationError::UnknownCatalogDetail)
        );
    }

    #[test]
    fn checked_documents_reject_unknown_contradictory_and_missing_projections() {
        assert_eq!(
            validate_document_tokens(DocumentKind::Security, "future H9999 allocation"),
            Err(DocumentValidationError::UnknownExactCode)
        );
        for shape in ["H12xx", "H10??", "H100*", "H10_1", "H1x01", "H1001suffix"] {
            assert_eq!(
                validate_document_tokens(DocumentKind::Portability, shape),
                Err(DocumentValidationError::WildcardCodeEscape),
                "shape {shape}"
            );
        }
        assert_eq!(
            validate_document_tokens(
                DocumentKind::Security,
                "`H1000-H1099` is owned by `wrong_owner`"
            ),
            Err(DocumentValidationError::ContradictoryFamilyOwner)
        );

        let human = include_str!("../docs/DIAGNOSTICS.md");
        let wrong_title = human.replacen(
            "| `H0001` | warning | unexpected top-level line |",
            "| `H0001` | warning | wrong title |",
            1,
        );
        assert_eq!(
            validate_human_projection(&wrong_title),
            Err(DocumentValidationError::ContradictoryExactCode)
        );

        let wrong_family = human.replacen(
            "| `H0000-H0099` | active | `source_shape` |",
            "| `H0000-H0099` | active | `wrong_owner` |",
            1,
        );
        assert_eq!(
            validate_human_projection(&wrong_family),
            Err(DocumentValidationError::ContradictoryFamilyOwner)
        );

        let missing_code = human
            .lines()
            .filter(|line| !line.starts_with("| `H1402` |"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            validate_human_projection(&missing_code),
            Err(DocumentValidationError::MissingActiveCode)
        );

        let missing_family = human
            .lines()
            .filter(|line| !line.starts_with("| `H1400-H1499` |"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            validate_human_projection(&missing_family),
            Err(DocumentValidationError::MissingFamily)
        );

        for document in checked_documents()
            .into_iter()
            .filter(|document| document.kind != DocumentKind::HumanCatalog)
        {
            let substituted = format!(
                "{}\n\nerror[H0201]: runtime profile denied this operation\n",
                document.text
            );
            assert_eq!(
                validate_document_tokens(document.kind, &substituted),
                Err(DocumentValidationError::ContradictoryExactCode),
                "known-code semantic substitution in {}",
                document.path
            );
        }
    }
}
