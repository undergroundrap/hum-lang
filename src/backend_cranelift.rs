//! Verified-only Cranelift backend boundary for the canonical minimal-add probe.

use crate::ir_verify::{
    VerifiedBackendInput, VerifiedConstantTextBackendInput, VerifiedIntegerSignBackendInput,
};
use crate::sha256;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{AbiParam, InstBuilder, MemFlagsData, SourceLoc, UserFuncName, types};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::verifier::verify_function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, default_libcall_names};
use std::fmt::Write as _;

pub const BACKEND_PROBE_SCHEMA: &str = "hum.backend_probe.v0";
const CRANELIFT_VERSION: &str = "0.133.1";
const CAPABILITY_ORIGIN: &str = "verified_backend_input_callback_v0";
const ARTIFACT_ID: &str = "sha256:a37707c23cc20a1720e45de901624e3101183a77ec1b5eb4ed55095b5097b82f";
const SOURCE_REVISION: &str =
    "sha256:aeae6ae9de975eee9873c3d9ece891e66bd7d6881b5035c24b1a11f3902a52b6";

const ORDINARY_PROBES: [(i64, i64); 4] = [(2, 3), (-7, 11), (0, 0), (1_000_000, 24)];
const OVERFLOW_PROBES: [(i64, i64); 2] = [(i64::MAX, 1), (i64::MIN, -1)];
const OVERFLOW_SENTINEL: i64 = 0x5a5a_5a5a_5a5a_5a5a;

fn cranelift_codegen_version_is_compatible(version: &str) -> bool {
    if version == CRANELIFT_VERSION {
        return true;
    }
    let Some(revision) = version
        .strip_prefix(CRANELIFT_VERSION)
        .and_then(|suffix| suffix.strip_prefix('-'))
    else {
        return false;
    };
    revision.len() == 9 && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn normalized_pinned_versions(versions: [&str; 5]) -> [&str; 5] {
    [
        if cranelift_codegen_version_is_compatible(versions[0]) {
            CRANELIFT_VERSION
        } else {
            versions[0]
        },
        versions[1],
        versions[2],
        versions[3],
        versions[4],
    ]
}

const ROW_IDS: [&str; 15] = [
    "B01", "B02", "B03", "B04", "B05", "B06", "B07", "B08", "B09", "B10", "B11", "B12", "B13",
    "B14", "B15",
];
const ROW_CLASSES: [&str; 15] = [
    "verified_capability_admission_unavailable",
    "unsupported_cranelift_api",
    "function_declaration_unsupported",
    "abi_construction_failed",
    "checked_add_selection_failed",
    "overflow_control_flow_failed",
    "source_location_mapping_failed",
    "unsupported_or_unavailable_target",
    "cranelift_verification_failed",
    "jit_declaration_failed",
    "jit_definition_failed",
    "jit_finalization_failed",
    "ordinary_execution_mismatch",
    "overflow_execution_mismatch",
    "incomplete_backend_evidence",
];
const ROW_PROPERTIES: [&str; 15] = [
    "verified capability admission",
    "pinned Cranelift API",
    "internal function declaration plan",
    "internal ABI and ordered block parameters",
    "fact-derived checked-add selection",
    "checked overflow control flow",
    "authenticated operation source location",
    "required native target ISA",
    "Cranelift function verification",
    "JIT function declaration",
    "JIT function definition",
    "JIT finalization and owned code pointer",
    "ordinary finalized-code execution",
    "overflow finalized-code execution",
    "complete ordered backend evidence",
];
const ROW_OWNERS: [&str; 15] = [
    "adapter admission",
    "backend compatibility gate",
    "module declaration plan",
    "signature/block builder",
    "instruction selector",
    "CFG/status builder",
    "source-map builder",
    "ISA builder",
    "Cranelift verifier",
    "JIT module declaration",
    "JIT module definition",
    "JIT module finalization",
    "ordinary probe runner",
    "overflow probe runner",
    "report/readiness gate",
];

#[allow(unexpected_cfgs, dead_code)]
mod verified_only_compile_proof {
    #[cfg(hum_compile_fail_backend_adapter_raw_inputs)]
    fn raw_inputs_must_not_compile() {
        let _ = super::probe(&Vec::<u8>::new());
        let _ = super::probe(&crate::ast::Program::default());
        let _ = super::probe(&(2_i64, 3_i64));
    }

    #[cfg(hum_compile_fail_backend_fault_seam_in_production)]
    fn production_fault_seam_must_not_compile() {
        let _ = super::BackendProbeFault::RejectVerifiedAdmission;
    }

    #[cfg(hum_compile_fail_verified_constant_text_backend_input_construction)]
    fn verified_constant_text_backend_input_construction_must_not_compile() {
        let _ = crate::ir_verify::VerifiedConstantTextBackendInput {
            projection: None.unwrap(),
            _artifact: std::marker::PhantomData,
        };
    }

    #[cfg(hum_compile_fail_verified_constant_text_backend_input_lifetime)]
    fn verified_constant_text_backend_input_lifetime_must_not_escape(
        program: &crate::ast::Program,
        diagnostics: &[crate::diagnostic::Diagnostic],
        layout: &crate::app_entry::CanonicalNativeLayout<'_>,
        authority: &crate::type_check::CanonicalConstantTextTypeAuthority,
        artifact: &[u8],
    ) -> crate::ir_verify::VerifiedConstantTextBackendInput<'static> {
        crate::ir_verify::with_verified_constant_text_backend_input(
            program,
            diagnostics,
            layout,
            authority,
            artifact,
            |capability| capability,
        )
        .unwrap()
    }

    #[cfg(hum_compile_fail_verified_backend_input_cross_substitution)]
    fn verified_backend_inputs_must_not_substitute(
        integer: &crate::ir_verify::VerifiedIntegerSignBackendInput<'_>,
        text: &crate::ir_verify::VerifiedConstantTextBackendInput<'_>,
    ) {
        let _ = super::execute_constant_text(integer);
        let _ = super::execute_integer_sign(text, 0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Decision {
    Go,
    NoGo,
}

impl Decision {
    fn as_str(self) -> &'static str {
        match self {
            Self::Go => "GO",
            Self::NoGo => "NO_GO",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BackendRow {
    id: &'static str,
    decision: Decision,
    class: String,
    owner: &'static str,
    property: &'static str,
    observed: String,
    required: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProbeResult {
    left: i64,
    right: i64,
    status: i32,
    result: Option<i64>,
    result_slot: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackendProbeReport {
    decision: Decision,
    ir_ready: usize,
    backend_ready: usize,
    target_triple: String,
    artifact_id: String,
    source_revision: String,
    rows: Vec<BackendRow>,
    clif_sha256: Option<String>,
    clif_instruction: Option<&'static str>,
    source_location: Option<u32>,
    declared: bool,
    defined: bool,
    finalized: bool,
    probes: Vec<ProbeResult>,
    consumption: BackendConsumption,
}

impl BackendProbeReport {
    pub(crate) fn go(&self) -> bool {
        self.decision == Decision::Go && self.backend_ready == 1
    }

    pub(crate) fn structurally_valid(&self) -> bool {
        if self.ir_ready != 1
            || self.artifact_id != ARTIFACT_ID
            || self.source_revision != SOURCE_REVISION
            || self.rows.len() != 15
            || !self.rows.iter().enumerate().all(|(index, row)| {
                row.id == ROW_IDS[index]
                    && row.owner == ROW_OWNERS[index]
                    && row.property == ROW_PROPERTIES[index]
                    && row.required == required_value(index)
            })
        {
            return false;
        }
        match self.primary_no_go_index() {
            None => {
                self.go()
                    && self.target_triple == required_target_triple()
                    && self.rows.iter().all(|row| {
                        row.decision == Decision::Go
                            && row.class == "GO"
                            && row.observed == "authenticated"
                    })
                    && self.clif_sha256.as_deref().is_some_and(is_sha256_id)
                    && self.clif_instruction == Some("sadd_overflow")
                    && self
                        .source_location
                        .is_some_and(|location| !SourceLoc::new(location).is_default())
                    && self.declared
                    && self.defined
                    && self.finalized
                    && probe_results_are_exact(&self.probes)
            }
            Some(primary) => {
                self.decision == Decision::NoGo
                    && self.backend_ready == 0
                    && self.rows.iter().enumerate().all(|(index, row)| {
                        if index < primary {
                            row.decision == Decision::Go
                                && row.class == "GO"
                                && row.observed == "authenticated"
                        } else if index == primary {
                            row.decision == Decision::NoGo
                                && row.class == ROW_CLASSES[index]
                                && !row.observed.is_empty()
                        } else {
                            row.decision == Decision::NoGo
                                && row.class == format!("blocked_by_{}", ROW_IDS[primary])
                                && row.observed == "not executed"
                        }
                    })
            }
        }
    }

    fn primary_no_go_index(&self) -> Option<usize> {
        self.rows
            .iter()
            .position(|row| row.decision == Decision::NoGo && !row.class.starts_with("blocked_by_"))
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendProbeFault {
    RejectVerifiedAdmission,
    RejectPinnedCraneliftApi,
    RejectFunctionDeclaration,
    RejectAbiConstruction,
    RejectCheckedAddSelection,
    RejectOverflowControlFlow,
    RejectSourceLocation,
    RejectTargetIsa,
    RejectClifVerification,
    RejectModuleDeclaration,
    RejectFunctionDefinition,
    RejectFinalization,
    CorruptOrdinaryExecution,
    CorruptOverflowExecution,
    DropEvidenceRow,
}

#[cfg(test)]
macro_rules! define_backend_probe_fault_mapping {
    ($($variant:ident => $ordinal:literal),+ $(,)?) => {
        const FAULTS: [BackendProbeFault; 15] = [$(BackendProbeFault::$variant),+];

        fn fault_ordinal(fault: BackendProbeFault) -> usize {
            match fault {
                $(BackendProbeFault::$variant => $ordinal),+
            }
        }
    };
}

#[cfg(test)]
define_backend_probe_fault_mapping! {
    RejectVerifiedAdmission => 0,
    RejectPinnedCraneliftApi => 1,
    RejectFunctionDeclaration => 2,
    RejectAbiConstruction => 3,
    RejectCheckedAddSelection => 4,
    RejectOverflowControlFlow => 5,
    RejectSourceLocation => 6,
    RejectTargetIsa => 7,
    RejectClifVerification => 8,
    RejectModuleDeclaration => 9,
    RejectFunctionDefinition => 10,
    RejectFinalization => 11,
    CorruptOrdinaryExecution => 12,
    CorruptOverflowExecution => 13,
    DropEvidenceRow => 14,
}

#[cfg(test)]
fn fault_at(fault: Option<BackendProbeFault>, row: usize) -> bool {
    fault.is_some_and(|fault| fault_ordinal(fault) == row)
}

#[cfg(not(test))]
fn fault_at(_: Option<()>, _: usize) -> bool {
    false
}

#[cfg(test)]
type BackendConsumption = (bool, bool, [usize; 3]);

#[cfg(not(test))]
type BackendConsumption = ();

fn verified_operator<'a>(
    _evidence: &mut ProbeEvidence,
    input: &'a VerifiedBackendInput<'_>,
) -> &'a str {
    #[cfg(test)]
    {
        _evidence.consumption.2[0] += 1;
    }
    input.overflow_edge().1
}

struct CompiledProbe {
    _module: JITModule,
    code: *const u8,
    clif: String,
    clif_sha256: String,
    source_location: u32,
}

struct ProbeEvidence {
    target_triple: String,
    compiled: Option<CompiledProbe>,
    probes: Vec<ProbeResult>,
    primary_failure: Option<(usize, String)>,
    declared: bool,
    defined: bool,
    finalized: bool,
    consumption: BackendConsumption,
}

pub(crate) fn probe(input: &VerifiedBackendInput<'_>) -> BackendProbeReport {
    probe_with_fault(input, None)
}

#[cfg(test)]
fn probe_for_test(
    input: &VerifiedBackendInput<'_>,
    fault: BackendProbeFault,
) -> BackendProbeReport {
    probe_with_fault(input, Some(fault))
}

#[cfg(test)]
type Fault = BackendProbeFault;
#[cfg(not(test))]
type Fault = ();

fn probe_with_fault(input: &VerifiedBackendInput<'_>, fault: Option<Fault>) -> BackendProbeReport {
    let mut evidence = ProbeEvidence {
        target_triple: host_target_label(),
        compiled: None,
        probes: Vec::new(),
        primary_failure: None,
        declared: false,
        defined: false,
        finalized: false,
        consumption: Default::default(),
    };

    macro_rules! stop {
        ($row:expr, $observed:expr) => {{
            evidence.primary_failure = Some(($row, $observed.to_string()));
            return finish_report(input, evidence);
        }};
    }

    if fault_at(fault, 0) || input.schema() != crate::backend_input::BACKEND_INPUT_SCHEMA
        || input.artifact_id() != ARTIFACT_ID
        || input.compiler_version() != env!("CARGO_PKG_VERSION")
        || input.semantic_contract() != "hum.canonical_minimal_add_backend_facts.v0"
        || input.target_context() != "target_independent_checked_i64_v0"
        || input.source_revision() != SOURCE_REVISION
        || input.source_path() != crate::backend_input::SOURCE_PATH
        || input.profile() != "normal"
        || !input.required_passes().eq("parse,semantic_graph_build,resolve,body_grammar,core_preview,core_lowering,core_verify,type_check,full_type_check,effect_check,ownership_alias_check,allocation_resource_check,contract_evidence_linking_checked_empty_for_exact_item,profile_check".split(','))
    {
        stop!(0, "verified capability getters were incomplete");
    }

    let observed_versions = [
        cranelift_codegen::VERSION,
        cranelift_frontend::VERSION,
        cranelift_jit::VERSION,
        cranelift_module::VERSION,
        cranelift_native::VERSION,
    ];
    let pinned_versions = normalized_pinned_versions(observed_versions);
    if fault_at(fault, 1) || pinned_versions != [CRANELIFT_VERSION; 5] {
        stop!(1, format!("versions={observed_versions:?}"));
    }

    let (linkage_kind, linkage_symbol) = input.linkage();
    if fault_at(fault, 2)
        || input.function_id() != "function:0"
        || linkage_kind != "internal"
        || linkage_symbol != "hum_fn_0"
    {
        stop!(
            2,
            format!(
                "function_id={};linkage={linkage_kind};symbol={linkage_symbol}",
                input.function_id()
            )
        );
    }

    let parameter_ids = input.parameter_value_ids();
    let definition_ids = input.parameter_definition_ids();
    let parameter_types = input.parameter_types();
    let (_, result_type) = input.result();
    if fault_at(fault, 3)
        || parameter_ids[0] == parameter_ids[1]
        || definition_ids[0] == definition_ids[1]
        || parameter_types != ["type:int64", "type:int64"]
        || result_type != "type:int64"
    {
        stop!(
            3,
            "verified ABI facts were not two ordered distinct i64 inputs and i64 result"
        );
    }

    let overflow_operation = verified_operator(&mut evidence, input);
    if fault_at(fault, 4) || overflow_operation != "checked_add" || input.operation_id().is_empty()
    {
        stop!(4, format!("operation={overflow_operation}"));
    }

    let (overflow_type, _, overflow_behavior) = input.overflow_edge();
    if fault_at(fault, 5)
        || overflow_type != "signed_64"
        || overflow_behavior != "runtime_trap_on_overflow"
    {
        stop!(
            5,
            format!("edge={overflow_type}/{overflow_operation}/{overflow_behavior}")
        );
    }

    let (line, column) = input.operation_span();
    let source_location = source_location_bits(line, column);
    if fault_at(fault, 6) || source_location.is_none() {
        stop!(6, format!("line={line};column={column}"));
    }
    let source_location = source_location.expect("source location checked above");

    if fault_at(fault, 7)
        || !target_is_required(
            std::env::consts::ARCH,
            std::env::consts::OS,
            host_target_environment(),
        )
    {
        stop!(7, format!("target={}", host_target_label()));
    }
    let mut flag_builder = settings::builder();
    if flag_builder.set("use_colocated_libcalls", "false").is_err()
        || flag_builder.set("is_pic", "false").is_err()
    {
        stop!(7, "required JIT flags unavailable");
    }
    let isa_builder = match cranelift_native::builder() {
        Ok(builder) => builder,
        Err(error) => stop!(7, format!("native ISA builder unavailable: {error}")),
    };
    let isa = match isa_builder.finish(settings::Flags::new(flag_builder)) {
        Ok(isa) => isa,
        Err(error) => stop!(7, format!("native ISA initialization failed: {error}")),
    };
    evidence.target_triple = isa.triple().to_string();
    if !matches!(
        evidence.target_triple.as_str(),
        "x86_64-pc-windows-msvc" | "x86_64-unknown-linux-gnu"
    ) {
        stop!(7, format!("target={}", evidence.target_triple));
    }
    let mut module = JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()));

    let mut signature = module.make_signature();
    signature.params.push(AbiParam::new(types::I64));
    signature.params.push(AbiParam::new(types::I64));
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature.returns.push(AbiParam::new(types::I32));
    let mut context = module.make_context();
    context.func.signature = signature.clone();
    context.func.name = UserFuncName::user(0, 0);
    let mut function_context = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        let success = builder.create_block();
        let overflow = builder.create_block();
        builder.switch_to_block(entry);
        builder.append_block_params_for_function_params(entry);
        let params = builder.block_params(entry);
        let (left, right, result_slot) = (params[0], params[1], params[2]);
        #[cfg(test)]
        {
            evidence.consumption.1 =
                (left, right, result_slot) == (params[0], params[1], params[2]);
        }
        builder.set_srcloc(SourceLoc::new(source_location));
        let (sum, overflowed) = builder.ins().sadd_overflow(left, right);
        builder.ins().brif(overflowed, overflow, &[], success, &[]);
        builder.switch_to_block(success);
        builder
            .ins()
            .store(MemFlagsData::new(), sum, result_slot, 0);
        let ok = builder.ins().iconst(types::I32, 0);
        builder.ins().return_(&[ok]);
        builder.switch_to_block(overflow);
        let failed = builder.ins().iconst(types::I32, 1);
        builder.ins().return_(&[failed]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    let clif = context.func.display().to_string();
    if fault_at(fault, 4) || clif.matches("sadd_overflow").count() != 1 {
        stop!(4, "emitted CLIF did not contain exactly one sadd_overflow");
    }
    if fault_at(fault, 5) || clif.matches("brif").count() != 1 || clif.matches("store").count() != 1
    {
        stop!(
            5,
            "emitted CLIF did not contain exact branch/store control flow"
        );
    }
    if fault_at(fault, 6)
        || !clif.contains(&SourceLoc::new(source_location).to_string())
        || SourceLoc::new(source_location).is_default()
    {
        stop!(
            6,
            "emitted operation lacked the exact non-default source location"
        );
    }

    #[cfg(test)]
    if fault_at(fault, 8) {
        context.func.signature.returns[0] = AbiParam::new(types::I64);
    }
    let verification = verify_function(&context.func, module.isa());
    if verification.is_err() {
        stop!(8, format!("{:?}", verification.err()));
    }

    let declaration_symbol = linkage_symbol;
    #[cfg(test)]
    {
        evidence.consumption.0 = declaration_symbol == linkage_symbol;
    }
    let declaration = module.declare_function(declaration_symbol, Linkage::Local, &signature);
    if fault_at(fault, 9) || declaration.is_err() {
        stop!(9, format!("{:?}", declaration.err()));
    }
    let function_id = declaration.expect("declaration checked above");
    evidence.declared = true;
    context.func.name = UserFuncName::user(0, function_id.as_u32());

    let definition = module.define_function(function_id, &mut context);
    if fault_at(fault, 10) || definition.is_err() {
        stop!(10, format!("{:?}", definition.err()));
    }
    evidence.defined = true;

    let finalization = module.finalize_definitions();
    if fault_at(fault, 11) || finalization.is_err() {
        stop!(11, format!("{:?}", finalization.err()));
    }
    let code = module.get_finalized_function(function_id);
    if code.is_null() {
        stop!(11, "finalized code pointer was null");
    }
    evidence.finalized = true;
    let clif_sha256 = format!(
        "sha256:{}",
        sha256::lowercase_hex(&sha256::digest(clif.as_bytes()).expect("small CLIF is hashable"))
    );
    evidence.compiled = Some(CompiledProbe {
        _module: module,
        code,
        clif,
        clif_sha256,
        source_location,
    });

    for (ordinal, (left, right)) in ORDINARY_PROBES.into_iter().enumerate() {
        let mut result_slot = 0_i64;
        let code = evidence
            .compiled
            .as_ref()
            .expect("compiled probe retained")
            .code;
        let status = invoke_finalized_uniform(
            code,
            left,
            right,
            &mut result_slot,
            &mut evidence.consumption,
            1,
        );
        let oracle = left.checked_add(right);
        let result = (status == 0).then_some(result_slot);
        evidence.probes.push(ProbeResult {
            left,
            right,
            status,
            result,
            result_slot,
        });
        if fault_at(fault, 12) || status != 0 || result != oracle {
            stop!(
                12,
                format!(
                    "ordinary_probe={ordinal};status={status};result={result:?};oracle={oracle:?}"
                )
            );
        }
    }

    for (ordinal, (left, right)) in OVERFLOW_PROBES.into_iter().enumerate() {
        let mut result_slot = OVERFLOW_SENTINEL;
        let code = evidence
            .compiled
            .as_ref()
            .expect("compiled probe retained")
            .code;
        let status = invoke_finalized_uniform(
            code,
            left,
            right,
            &mut result_slot,
            &mut evidence.consumption,
            2,
        );
        let result = (status == 0).then_some(result_slot);
        evidence.probes.push(ProbeResult {
            left,
            right,
            status,
            result,
            result_slot,
        });
        if fault_at(fault, 13)
            || status != 1
            || result.is_some()
            || result_slot != OVERFLOW_SENTINEL
            || left.checked_add(right).is_some()
        {
            stop!(
                13,
                format!("overflow_probe={ordinal};status={status};result={result:?}")
            );
        }
    }

    if fault_at(fault, 14) || !backend_evidence_is_complete(&evidence) {
        stop!(
            14,
            "required backend evidence was incomplete or inconsistent"
        );
    }
    finish_report(input, evidence)
}

fn backend_evidence_is_complete(evidence: &ProbeEvidence) -> bool {
    let Some(compiled) = evidence.compiled.as_ref() else {
        return false;
    };
    evidence.declared
        && evidence.defined
        && evidence.finalized
        && !compiled.code.is_null()
        && compiled.clif.matches("sadd_overflow").count() == 1
        && compiled.clif.matches("brif").count() == 1
        && compiled.clif.matches("store").count() == 1
        && !SourceLoc::new(compiled.source_location).is_default()
        && probe_results_are_exact(&evidence.probes)
}

#[allow(unsafe_code)]
fn invoke_finalized_uniform(
    code: *const u8,
    left: i64,
    right: i64,
    result_slot: &mut i64,
    _consumption: &mut BackendConsumption,
    _probe_class: usize,
) -> i32 {
    // SAFETY: `code` is non-null finalized JIT memory owned by the retained
    // `JITModule`; its definition has the exact C ABI below, and `result_slot`
    // is non-null, aligned, initialized, and live for the complete invocation.
    let status = unsafe {
        let function =
            std::mem::transmute::<*const u8, unsafe extern "C" fn(i64, i64, *mut i64) -> i32>(code);
        function(left, right, std::ptr::from_mut(result_slot))
    };
    #[cfg(test)]
    {
        _consumption.2[_probe_class] += 1;
    }
    status
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeIntegerSignExecution {
    #[cfg(test)]
    pub(crate) value: i64,
    pub(crate) tag: i64,
    pub(crate) literal: String,
    pub(crate) target_triple: String,
    pub(crate) clif_sha256: String,
    #[cfg(test)]
    pub(crate) clif: String,
    pub(crate) ir_ready: usize,
    pub(crate) backend_ready: usize,
}

pub(crate) fn execute_integer_sign(
    input: &VerifiedIntegerSignBackendInput<'_>,
    value: i64,
) -> Result<NativeIntegerSignExecution, String> {
    if input.schema() != crate::backend_input::INTEGER_SIGN_BACKEND_INPUT_SCHEMA
        || input.compiler_version() != env!("CARGO_PKG_VERSION")
        || input.target_context() != crate::backend_input::TARGET_CONTEXT
        || input.artifact_id().is_empty()
        || input.required_passes() != crate::backend_input::REQUIRED_PASSES
    {
        return Err("verified integer-sign capability admission failed".to_string());
    }
    let (source_revision, source_path, module_name, app_name, entry_name) = input.source_identity();
    if !source_revision.starts_with("sha256:")
        || !source_path.starts_with("programs/")
        || module_name != format!("programs.{app_name}")
        || entry_name != "run_tool"
    {
        return Err("verified integer-sign source identity failed".to_string());
    }
    let branches = input.branches();
    if branches[0].predicate != "signed_less_than_zero"
        || branches[0].tag != 0
        || branches[1].predicate != "equal_to_zero"
        || branches[1].tag != 1
        || branches[2].predicate != "fallthrough"
        || branches[2].tag != 2
    {
        return Err("verified integer-sign control-flow facts failed".to_string());
    }
    if !target_is_required(
        std::env::consts::ARCH,
        std::env::consts::OS,
        host_target_environment(),
    ) {
        return Err(format!(
            "unsupported native target `{}`",
            host_target_label()
        ));
    }
    let observed_versions = [
        cranelift_codegen::VERSION,
        cranelift_frontend::VERSION,
        cranelift_jit::VERSION,
        cranelift_module::VERSION,
        cranelift_native::VERSION,
    ];
    if normalized_pinned_versions(observed_versions) != [CRANELIFT_VERSION; 5] {
        return Err("pinned Cranelift API unavailable".to_string());
    }
    let mut flag_builder = settings::builder();
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|error| format!("required JIT flag unavailable: {error}"))?;
    flag_builder
        .set("is_pic", "false")
        .map_err(|error| format!("required JIT flag unavailable: {error}"))?;
    let isa = cranelift_native::builder()
        .map_err(|error| format!("native ISA builder unavailable: {error}"))?
        .finish(settings::Flags::new(flag_builder))
        .map_err(|error| format!("native ISA initialization failed: {error}"))?;
    let target_triple = isa.triple().to_string();
    if !matches!(
        target_triple.as_str(),
        "x86_64-pc-windows-msvc" | "x86_64-unknown-linux-gnu"
    ) {
        return Err(format!("unsupported native target `{target_triple}`"));
    }
    let mut module = JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()));
    let mut signature = module.make_signature();
    signature.params.push(AbiParam::new(types::I64));
    signature.params.push(AbiParam::new(types::I64));
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature.returns.push(AbiParam::new(types::I32));
    let mut context = module.make_context();
    context.func.signature = signature.clone();
    context.func.name = UserFuncName::user(0, 1);
    let mut function_context = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        let negative = builder.create_block();
        let zero_test = builder.create_block();
        let zero = builder.create_block();
        let positive = builder.create_block();
        let emit = builder.create_block();
        builder.append_block_param(emit, types::I64);
        builder.switch_to_block(entry);
        builder.append_block_params_for_function_params(entry);
        let params = builder.block_params(entry);
        let (argument, result_slot) = (params[0], params[2]);
        let first_location = source_location_bits(
            branches[0].predicate_span.line,
            branches[0].predicate_span.column,
        )
        .ok_or_else(|| "invalid negative predicate source location".to_string())?;
        builder.set_srcloc(SourceLoc::new(first_location));
        let is_negative = builder.ins().icmp_imm(IntCC::SignedLessThan, argument, 0);
        builder
            .ins()
            .brif(is_negative, negative, &[], zero_test, &[]);
        builder.switch_to_block(negative);
        let negative_tag = builder.ins().iconst(types::I64, branches[0].tag);
        builder.ins().jump(emit, &[negative_tag.into()]);
        builder.switch_to_block(zero_test);
        let second_location = source_location_bits(
            branches[1].predicate_span.line,
            branches[1].predicate_span.column,
        )
        .ok_or_else(|| "invalid zero predicate source location".to_string())?;
        builder.set_srcloc(SourceLoc::new(second_location));
        let is_zero = builder.ins().icmp_imm(IntCC::Equal, argument, 0);
        builder.ins().brif(is_zero, zero, &[], positive, &[]);
        builder.switch_to_block(zero);
        let zero_tag = builder.ins().iconst(types::I64, branches[1].tag);
        builder.ins().jump(emit, &[zero_tag.into()]);
        builder.switch_to_block(positive);
        let positive_tag = builder.ins().iconst(types::I64, branches[2].tag);
        builder.ins().jump(emit, &[positive_tag.into()]);
        builder.switch_to_block(emit);
        let selected = builder.block_params(emit)[0];
        builder
            .ins()
            .store(MemFlagsData::new(), selected, result_slot, 0);
        let ok = builder.ins().iconst(types::I32, 0);
        builder.ins().return_(&[ok]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    let clif = context.func.display().to_string();
    if clif.matches("brif").count() != 2
        || clif.matches("store").count() != 1
        || !clif.contains("icmp slt")
        || !clif.contains("icmp.i64 eq")
    {
        return Err("integer-sign CLIF shape verification failed".to_string());
    }
    verify_function(&context.func, module.isa())
        .map_err(|error| format!("Cranelift verification failed: {error}"))?;
    let function_id = module
        .declare_function("hum_integer_sign_0", Linkage::Local, &signature)
        .map_err(|error| format!("JIT declaration failed: {error}"))?;
    context.func.name = UserFuncName::user(0, function_id.as_u32());
    module
        .define_function(function_id, &mut context)
        .map_err(|error| format!("JIT definition failed: {error}"))?;
    module
        .finalize_definitions()
        .map_err(|error| format!("JIT finalization failed: {error}"))?;
    let code = module.get_finalized_function(function_id);
    if code.is_null() {
        return Err("finalized integer-sign code pointer was null".to_string());
    }
    let sentinel = 0x5a5a_5a5a_5a5a_5a5a_i64;
    let mut result_slot = sentinel;
    let status = invoke_finalized_uniform(
        code,
        value,
        0,
        &mut result_slot,
        &mut BackendConsumption::default(),
        0,
    );
    if status != 0 || result_slot == sentinel {
        return Err(format!(
            "integer-sign native invocation failed: status={status};result_slot={result_slot}"
        ));
    }
    let branch = branches
        .iter()
        .find(|branch| branch.tag == result_slot)
        .ok_or_else(|| format!("integer-sign native tag invalid: {result_slot}"))?;
    let clif_sha256 = format!(
        "sha256:{}",
        sha256::lowercase_hex(
            &sha256::digest(clif.as_bytes()).ok_or_else(|| "CLIF digest failed".to_string())?
        )
    );
    Ok(NativeIntegerSignExecution {
        #[cfg(test)]
        value,
        tag: result_slot,
        literal: branch.literal.clone(),
        target_triple,
        clif_sha256,
        #[cfg(test)]
        clif,
        ir_ready: 1,
        backend_ready: 1,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeConstantTextExecution {
    pub(crate) tag: i64,
    pub(crate) literal: String,
    pub(crate) target_triple: String,
    pub(crate) clif_sha256: String,
    #[cfg(test)]
    pub(crate) clif: String,
    pub(crate) invocation_count: usize,
    pub(crate) result_store_count: usize,
    pub(crate) ir_ready: usize,
    pub(crate) backend_ready: usize,
}

pub(crate) fn execute_constant_text(
    input: &VerifiedConstantTextBackendInput<'_>,
) -> Result<NativeConstantTextExecution, String> {
    if input.schema() != crate::backend_input::CONSTANT_TEXT_BACKEND_INPUT_SCHEMA
        || input.compiler_version() != env!("CARGO_PKG_VERSION")
        || input.target_context() != crate::backend_input::TARGET_CONTEXT
        || input.artifact_id().is_empty()
        || input.profile_id() != "normal"
        || input.required_passes() != crate::backend_input::REQUIRED_PASSES
    {
        return Err("verified constant-Text capability admission failed".to_string());
    }
    let (source_revision, source_path, module_name, app_name, entry_name) = input.source_identity();
    if !source_revision.starts_with("sha256:")
        || source_revision.len() != "sha256:".len() + 64
        || !source_path.starts_with("programs/")
        || module_name != format!("programs.{app_name}")
        || entry_name.is_empty()
    {
        return Err("verified constant-Text source identity failed".to_string());
    }
    let operation = input.operation();
    if operation.tag != 0 || operation.literal.is_empty() {
        return Err("verified constant-Text operation facts failed".to_string());
    }
    if !target_is_required(
        std::env::consts::ARCH,
        std::env::consts::OS,
        host_target_environment(),
    ) {
        return Err(format!(
            "unsupported native target `{}`",
            host_target_label()
        ));
    }
    let observed_versions = [
        cranelift_codegen::VERSION,
        cranelift_frontend::VERSION,
        cranelift_jit::VERSION,
        cranelift_module::VERSION,
        cranelift_native::VERSION,
    ];
    if normalized_pinned_versions(observed_versions) != [CRANELIFT_VERSION; 5] {
        return Err("pinned Cranelift API unavailable".to_string());
    }
    let mut flag_builder = settings::builder();
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|error| format!("required JIT flag unavailable: {error}"))?;
    flag_builder
        .set("is_pic", "false")
        .map_err(|error| format!("required JIT flag unavailable: {error}"))?;
    let isa = cranelift_native::builder()
        .map_err(|error| format!("native ISA builder unavailable: {error}"))?
        .finish(settings::Flags::new(flag_builder))
        .map_err(|error| format!("native ISA initialization failed: {error}"))?;
    let target_triple = isa.triple().to_string();
    if !matches!(
        target_triple.as_str(),
        "x86_64-pc-windows-msvc" | "x86_64-unknown-linux-gnu"
    ) {
        return Err(format!("unsupported native target `{target_triple}`"));
    }
    let mut module = JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()));
    let mut signature = module.make_signature();
    signature.params.push(AbiParam::new(types::I64));
    signature.params.push(AbiParam::new(types::I64));
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature.returns.push(AbiParam::new(types::I32));
    let mut context = module.make_context();
    context.func.signature = signature.clone();
    context.func.name = UserFuncName::user(0, 2);
    let mut function_context = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        builder.switch_to_block(entry);
        builder.append_block_params_for_function_params(entry);
        let result_slot = builder.block_params(entry)[2];
        let location =
            source_location_bits(operation.literal_span.line, operation.literal_span.column)
                .ok_or_else(|| "invalid constant-Text source location".to_string())?;
        builder.set_srcloc(SourceLoc::new(location));
        let tag = builder.ins().iconst(types::I64, operation.tag);
        builder
            .ins()
            .store(MemFlagsData::new(), tag, result_slot, 0);
        let ok = builder.ins().iconst(types::I32, 0);
        builder.ins().return_(&[ok]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    let clif = context.func.display().to_string();
    if clif.matches("store").count() != 1
        || clif.contains(&operation.literal)
        || clif.matches("iconst.i64 0").count() != 1
    {
        return Err("constant-Text CLIF shape verification failed".to_string());
    }
    verify_function(&context.func, module.isa())
        .map_err(|error| format!("Cranelift verification failed: {error}"))?;
    let function_id = module
        .declare_function("hum_constant_text_0", Linkage::Local, &signature)
        .map_err(|error| format!("JIT declaration failed: {error}"))?;
    context.func.name = UserFuncName::user(0, function_id.as_u32());
    module
        .define_function(function_id, &mut context)
        .map_err(|error| format!("JIT definition failed: {error}"))?;
    module
        .finalize_definitions()
        .map_err(|error| format!("JIT finalization failed: {error}"))?;
    let code = module.get_finalized_function(function_id);
    if code.is_null() {
        return Err("finalized constant-Text code pointer was null".to_string());
    }
    let sentinel = 0x5a5a_5a5a_5a5a_5a5a_i64;
    let mut result_slot = sentinel;
    let status = invoke_finalized_uniform(
        code,
        0,
        0,
        &mut result_slot,
        &mut BackendConsumption::default(),
        0,
    );
    if status != 0 || result_slot == sentinel || result_slot != operation.tag {
        return Err(format!(
            "constant-Text native invocation failed: status={status};result_slot={result_slot}"
        ));
    }
    let clif_sha256 = format!(
        "sha256:{}",
        sha256::lowercase_hex(
            &sha256::digest(clif.as_bytes()).ok_or_else(|| "CLIF digest failed".to_string())?
        )
    );
    Ok(NativeConstantTextExecution {
        tag: result_slot,
        literal: operation.literal.clone(),
        target_triple,
        clif_sha256,
        #[cfg(test)]
        clif,
        invocation_count: 1,
        result_store_count: 1,
        ir_ready: 1,
        backend_ready: 1,
    })
}

fn finish_report(input: &VerifiedBackendInput<'_>, evidence: ProbeEvidence) -> BackendProbeReport {
    let primary = evidence.primary_failure.as_ref().map(|(index, _)| *index);
    let rows = ROW_IDS
        .iter()
        .enumerate()
        .map(|(index, id)| {
            let (decision, class, observed) = match primary {
                None => (Decision::Go, "GO".to_string(), "authenticated".to_string()),
                Some(primary) if index < primary => {
                    (Decision::Go, "GO".to_string(), "authenticated".to_string())
                }
                Some(primary) if index == primary => (
                    Decision::NoGo,
                    ROW_CLASSES[index].to_string(),
                    evidence
                        .primary_failure
                        .as_ref()
                        .map(|(_, observed)| observed.clone())
                        .unwrap_or_default(),
                ),
                Some(primary) => (
                    Decision::NoGo,
                    format!("blocked_by_{}", ROW_IDS[primary]),
                    "not executed".to_string(),
                ),
            };
            BackendRow {
                id,
                decision,
                class,
                owner: ROW_OWNERS[index],
                property: ROW_PROPERTIES[index],
                observed,
                required: required_value(index).to_string(),
            }
        })
        .collect::<Vec<_>>();
    let decision = if primary.is_none() {
        Decision::Go
    } else {
        Decision::NoGo
    };
    let compiled = evidence.compiled.as_ref();
    let report = BackendProbeReport {
        decision,
        ir_ready: 1,
        backend_ready: usize::from(decision == Decision::Go),
        target_triple: evidence.target_triple,
        artifact_id: input.artifact_id().to_string(),
        source_revision: input.source_revision().to_string(),
        rows,
        clif_sha256: compiled.map(|compiled| compiled.clif_sha256.clone()),
        clif_instruction: compiled.and_then(|compiled| {
            (compiled.clif.matches("sadd_overflow").count() == 1).then_some("sadd_overflow")
        }),
        source_location: compiled.map(|compiled| compiled.source_location),
        declared: evidence.declared,
        defined: evidence.defined,
        finalized: evidence.finalized,
        probes: evidence.probes,
        consumption: evidence.consumption,
    };
    debug_assert!(report.structurally_valid());
    report
}

fn required_value(index: usize) -> &'static str {
    [
        "one live callback-scoped VerifiedBackendInput",
        "five production crates at exact 0.133.1",
        "function:0/internal/hum_fn_0",
        "(i64,i64,*mut i64)->i32 with ordered inputs",
        "one fact-derived sadd_overflow",
        "one brif, success store/status 0, overflow status 1",
        "one non-default SourceLoc from the verified operation span",
        "x86_64 Windows-MSVC or x86_64 Linux-GNU native ISA",
        "Cranelift verifier success",
        "one Local JIT declaration with exact signature",
        "one definition from the verified CLIF body",
        "finalized non-null module-owned code pointer",
        "four exact status-0 checked-add results",
        "two exact status-1 overflow results without values",
        "fifteen ordered GO rows and internally consistent evidence",
    ][index]
}

fn source_location_bits(line: usize, column: usize) -> Option<u32> {
    if line == 0 || column == 0 || line > u16::MAX as usize || column > u16::MAX as usize {
        return None;
    }
    Some(((line as u32) << 16) | column as u32)
}

fn target_is_required(architecture: &str, operating_system: &str, environment: &str) -> bool {
    architecture == "x86_64"
        && matches!(
            (operating_system, environment),
            ("windows", "msvc") | ("linux", "gnu")
        )
}

fn required_target_triple() -> &'static str {
    if cfg!(all(
        target_arch = "x86_64",
        target_os = "windows",
        target_env = "msvc"
    )) {
        "x86_64-pc-windows-msvc"
    } else if cfg!(all(
        target_arch = "x86_64",
        target_os = "linux",
        target_env = "gnu"
    )) {
        "x86_64-unknown-linux-gnu"
    } else {
        "unsupported"
    }
}

fn is_sha256_id(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn probe_results_are_exact(probes: &[ProbeResult]) -> bool {
    probes.len() == ORDINARY_PROBES.len() + OVERFLOW_PROBES.len()
        && probes[..ORDINARY_PROBES.len()]
            .iter()
            .zip(ORDINARY_PROBES)
            .all(|(probe, inputs)| {
                (probe.left, probe.right) == inputs
                    && probe.status == 0
                    && probe.result == probe.left.checked_add(probe.right)
                    && probe.result_slot == probe.result.expect("ordinary result checked above")
            })
        && probes[ORDINARY_PROBES.len()..]
            .iter()
            .zip(OVERFLOW_PROBES)
            .all(|(probe, inputs)| {
                (probe.left, probe.right) == inputs
                    && probe.status == 1
                    && probe.result.is_none()
                    && probe.result_slot == OVERFLOW_SENTINEL
                    && probe.left.checked_add(probe.right).is_none()
            })
}

fn host_target_environment() -> &'static str {
    if cfg!(target_env = "msvc") {
        "msvc"
    } else if cfg!(target_env = "gnu") {
        "gnu"
    } else {
        "other"
    }
}

fn host_target_label() -> String {
    format!(
        "{}-{}-{}",
        std::env::consts::ARCH,
        std::env::consts::OS,
        host_target_environment()
    )
}

pub(crate) fn backend_probe_text(report: &BackendProbeReport) -> String {
    let mut out = format!(
        "schema={BACKEND_PROBE_SCHEMA}\ndecision={}\nir_ready={}\nbackend_ready={}\ntarget_triple={}\ncranelift_version={CRANELIFT_VERSION}\nartifact_id={}\nsource_revision={}\nverified_capability_origin={CAPABILITY_ORIGIN}\nclif_sha256={}\nclif_instruction={}\nsource_location={}\ncompile_disposition=declared:{};defined:{};finalized:{}\n",
        report.decision.as_str(),
        report.ir_ready,
        report.backend_ready,
        report.target_triple,
        report.artifact_id,
        report.source_revision,
        report.clif_sha256.as_deref().unwrap_or(""),
        report.clif_instruction.unwrap_or(""),
        report
            .source_location
            .map_or(String::new(), |value| value.to_string()),
        report.declared,
        report.defined,
        report.finalized
    );
    for row in &report.rows {
        let _ = writeln!(
            out,
            "row={} decision={} class={} owner={} property={} observed={} required={}",
            row.id,
            row.decision.as_str(),
            row.class,
            row.owner,
            row.property,
            row.observed,
            row.required
        );
    }
    for probe in &report.probes {
        let result = probe
            .result
            .map_or("null".to_string(), |value| value.to_string());
        let _ = writeln!(
            out,
            "probe left={} right={} status={} result={result}",
            probe.left, probe.right, probe.status
        );
    }
    out
}

pub(crate) fn backend_probe_json(report: &BackendProbeReport) -> String {
    let mut out = format!(
        "{{\n  \"schema\": {},\n  \"decision\": {},\n  \"ir_ready\": {},\n  \"backend_ready\": {},\n  \"target_triple\": {},\n  \"cranelift_version\": {},\n  \"artifact_id\": {},\n  \"source_revision\": {},\n  \"verified_capability_origin\": {},\n  \"clif_sha256\": {},\n  \"clif_instruction\": {},\n  \"source_location\": {},\n  \"compile\": {{\"declared\": {}, \"defined\": {}, \"finalized\": {}}},\n  \"rows\": [\n",
        quote(BACKEND_PROBE_SCHEMA),
        quote(report.decision.as_str()),
        report.ir_ready,
        report.backend_ready,
        quote(&report.target_triple),
        quote(CRANELIFT_VERSION),
        quote(&report.artifact_id),
        quote(&report.source_revision),
        quote(CAPABILITY_ORIGIN),
        report
            .clif_sha256
            .as_deref()
            .map_or("null".to_string(), quote),
        report.clif_instruction.map_or("null".to_string(), quote),
        report
            .source_location
            .map_or("null".to_string(), |value| value.to_string()),
        report.declared,
        report.defined,
        report.finalized
    );
    for (index, row) in report.rows.iter().enumerate() {
        let comma = if index + 1 == report.rows.len() {
            ""
        } else {
            ","
        };
        let _ = writeln!(
            out,
            "    {{\"id\": {}, \"decision\": {}, \"class\": {}, \"owner\": {}, \"property\": {}, \"observed\": {}, \"required\": {}}}{comma}",
            quote(row.id),
            quote(row.decision.as_str()),
            quote(&row.class),
            quote(row.owner),
            quote(row.property),
            quote(&row.observed),
            quote(&row.required)
        );
    }
    out.push_str("  ],\n  \"probes\": [\n");
    for (index, probe) in report.probes.iter().enumerate() {
        let comma = if index + 1 == report.probes.len() {
            ""
        } else {
            ","
        };
        let result = probe
            .result
            .map_or("null".to_string(), |value| value.to_string());
        let _ = writeln!(
            out,
            "    {{\"left\": {}, \"right\": {}, \"status\": {}, \"result\": {result}}}{comma}",
            probe.left, probe.right, probe.status
        );
    }
    out.push_str("  ]\n}\n");
    out
}

fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(out, "\\u{:04x}", character as u32);
            }
            character => out.push(character),
        }
    }
    out.push('\"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Program;

    fn integer_sign_execution(source: &str, value: i64) -> NativeIntegerSignExecution {
        let parsed = crate::parser::parse_source("programs/integer_sign.hum", source);
        let checked = crate::check::check_parse_output(&parsed);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        assert!(
            checked
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.severity != crate::diagnostic::Severity::Error)
        );
        let program = Program {
            files: vec![parsed.file],
        };
        let entry = crate::app_entry::analyze(&program)
            .entry
            .expect("canonical app entry");
        let layout = crate::app_entry::analyze_canonical_native_layout(
            &program,
            "programs/integer_sign.hum",
            Some(&entry),
        )
        .layout
        .expect("canonical native layout");
        let displacement = Program {
            files: vec![crate::parser::parse_source("empty.hum", "").file],
        };
        let _ = crate::typed_failure::analyze_program(&displacement);
        let artifact = crate::backend_input::canonical_integer_sign_artifact(
            &program,
            &checked.diagnostics,
            &layout,
        )
        .expect("canonical integer-sign artifact");
        let mut calls = 0;
        let execution = crate::ir_verify::with_verified_integer_sign_backend_input(
            &program,
            &checked.diagnostics,
            &layout,
            artifact.bytes(),
            |verified| {
                calls += 1;
                execute_integer_sign(&verified, value)
            },
        )
        .expect("verified integer-sign authority")
        .expect("integer-sign native execution");
        assert_eq!(calls, 1);
        execution
    }

    fn constant_text_execution(source: &str) -> NativeConstantTextExecution {
        let parsed = crate::parser::parse_source("programs/hello_world.hum", source);
        let checked = crate::check::check_parse_output(&parsed);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        let program = Program {
            files: vec![parsed.file],
        };
        let entry = crate::app_entry::analyze(&program).entry.expect("entry");
        let layout = crate::app_entry::analyze_canonical_native_layout(
            &program,
            "programs/hello_world.hum",
            Some(&entry),
        )
        .layout
        .expect("layout");
        let authority = crate::type_check::canonical_constant_text_type_authority(
            &program,
            &layout,
            &checked.diagnostics,
        )
        .expect("constant Text authority");
        let artifact = crate::backend_input::canonical_constant_text_artifact(
            &program,
            &checked.diagnostics,
            &layout,
            &authority,
        )
        .expect("artifact");
        crate::ir_verify::with_verified_constant_text_backend_input(
            &program,
            &checked.diagnostics,
            &layout,
            &authority,
            artifact.bytes(),
            |verified| execute_constant_text(&verified),
        )
        .expect("verified capability")
        .expect("native execution")
    }

    fn valid_probe(fault: Option<BackendProbeFault>) -> BackendProbeReport {
        let source = include_str!("../examples/core/minimal_add.hum");
        let parsed = crate::parser::parse_source(crate::backend_input::SOURCE_PATH, source);
        let checked = crate::check::check_parse_output(&parsed);
        assert!(parsed.diagnostics.is_empty() && checked.diagnostics.is_empty());
        let program = Program {
            files: vec![parsed.file],
        };
        let artifact = crate::backend_input::canonical_minimal_add_artifact(&program, &[])
            .expect("valid fixture must issue canonical backend input");
        let mut callback_count = 0;
        let (verification, observed) = crate::ir_verify::with_verified_backend_input(
            &program,
            &[],
            artifact.bytes(),
            |verified| {
                callback_count += 1;
                fault.map_or_else(
                    || probe(&verified),
                    |fault| probe_for_test(&verified, fault),
                )
            },
        );
        assert!(verification.accepted());
        assert_eq!(callback_count, 1);
        observed.expect("valid callback must return a probe observation")
    }

    #[test]
    fn verified_minimal_add_emits_checked_cranelift_ir() {
        let report = valid_probe(None);
        assert!(report.go() && report.structurally_valid());
        assert_eq!(report.clif_instruction, Some("sadd_overflow"));
    }

    #[test]
    fn integer_sign_lowering_is_source_driven_and_load_bearing() {
        let source = include_str!("../programs/integer_sign.hum");
        assert!(target_is_required("x86_64", "windows", "msvc"));
        assert!(target_is_required("x86_64", "linux", "gnu"));
        assert!(!target_is_required("aarch64", "macos", "other"));
        for (value, expected_tag, expected_literal) in [
            (-7, 0, "negative"),
            (-1, 0, "negative"),
            (0, 1, "zero"),
            (1, 2, "positive"),
            (9, 2, "positive"),
        ] {
            let execution = integer_sign_execution(source, value);
            assert_eq!(execution.value, value);
            assert_eq!(execution.literal, expected_literal);
            assert_eq!(execution.tag, expected_tag);
            assert_eq!((execution.ir_ready, execution.backend_ready), (1, 1));
            assert!(matches!(
                execution.target_triple.as_str(),
                "x86_64-pc-windows-msvc" | "x86_64-unknown-linux-gnu"
            ));
            assert_eq!(execution.clif.matches("brif").count(), 2);
            assert_eq!(execution.clif.matches("store").count(), 1);
            assert_eq!(execution.clif.matches("icmp slt").count(), 1);
            assert_eq!(execution.clif.matches("icmp.i64 eq").count(), 1);
            assert!(!execution.clif.contains(expected_literal));
        }

        let changed = source.replacen("\"negative\"", "\"below\"", 1);
        assert_ne!(changed.as_bytes(), source.as_bytes());
        let execution = integer_sign_execution(&changed, -7);
        assert_eq!((execution.tag, execution.literal.as_str()), (0, "below"));
        assert_eq!((execution.ir_ready, execution.backend_ready), (1, 1));

        for (from, to) in [
            ("value < 0", "value < 1"),
            ("value == 0", "value == 1"),
            (
                "return written\n      }\n      let written",
                "return written\n      }\n      if value > 0 {\n      let written",
            ),
        ] {
            let mutated = source.replacen(from, to, 1);
            assert_ne!(mutated.as_bytes(), source.as_bytes());
            let result = std::panic::catch_unwind(|| integer_sign_execution(&mutated, -7));
            assert!(
                result.is_err(),
                "unsupported semantic mutation escaped admission"
            );
        }

        let restored = integer_sign_execution(source, -7);
        assert_eq!((restored.tag, restored.literal.as_str()), (0, "negative"));
    }

    #[test]
    fn hello_world_lowering_is_source_driven_and_load_bearing() {
        let unsupported_backend_ready = usize::from(target_is_required("x86_64", "macos", "other"));
        assert_eq!(
            unsupported_backend_ready, 0,
            "unsupported target earned forbidden backend-ready evidence"
        );
        let source = include_str!("../programs/hello_world.hum");
        let execution = constant_text_execution(source);
        assert_eq!(execution.literal, "Hello, world!");
        assert_eq!(execution.tag, 0);
        assert_eq!(
            execution.invocation_count, 1,
            "finalized constant-Text invocation count was not exactly one"
        );
        assert_eq!(execution.result_store_count, 1);
        assert_eq!((execution.ir_ready, execution.backend_ready), (1, 1));
        assert_eq!(execution.clif.matches("store").count(), 1);
        assert!(!execution.clif.contains("Hello, world!"));
        assert!(matches!(
            execution.target_triple.as_str(),
            "x86_64-pc-windows-msvc" | "x86_64-unknown-linux-gnu"
        ));

        let changed = source.replace("Hello, world!", "Source facts win");
        let changed_execution = constant_text_execution(&changed);
        assert_eq!(
            changed_execution.literal, "Source facts win",
            "Rust literal stopped source-literal artifact/output parity"
        );
        assert_eq!(changed_execution.tag, 0);
        assert!(!execution.clif_sha256.is_empty());
    }

    #[test]
    fn minimal_add_jit_probe_matrix_is_exact() {
        let report = valid_probe(None);
        let expected = [
            (2, 3, 0, Some(5)),
            (-7, 11, 0, Some(4)),
            (0, 0, 0, Some(0)),
            (1_000_000, 24, 0, Some(1_000_024)),
            (i64::MAX, 1, 1, None),
            (i64::MIN, -1, 1, None),
        ];
        assert_eq!(report.probes.len(), expected.len());
        for (probe, expected) in report.probes.iter().zip(expected) {
            assert_eq!(
                (probe.left, probe.right, probe.status, probe.result),
                expected
            );
            assert_eq!(probe.result, probe.left.checked_add(probe.right));
            assert_eq!(probe.result_slot, probe.result.unwrap_or(OVERFLOW_SENTINEL));
        }
    }

    #[test]
    fn backend_go_no_go_rows_are_complete_and_load_bearing() {
        assert_eq!(FAULTS.len(), 15);
        let report = valid_probe(None);
        assert!(report.go());
        let row_ids = report.rows.iter().map(|row| row.id).collect::<Vec<_>>();
        assert_eq!(row_ids, ROW_IDS);
        let consumption = &report.consumption;
        assert!(consumption.0, "B03 linkage consumption");
        assert!(consumption.1, "B04 parameter-order consumption");
        assert!(consumption.2[0] == 1, "B05 operator-getter consumption");
        assert!(consumption.2[1] == 4, "B13 finalized ordinary invocation");
        assert!(consumption.2[2] == 2, "B14 finalized overflow invocation");
        for (ordinal, fault) in FAULTS.into_iter().enumerate() {
            assert_eq!(fault_ordinal(fault), ordinal);
            let report = valid_probe(Some(fault));
            assert!(!report.go() && report.structurally_valid() && report.backend_ready == 0);
            assert_eq!(report.rows[ordinal].class, ROW_CLASSES[ordinal]);
            assert!(ordinal == 0 || report.rows[ordinal].class != ROW_CLASSES[ordinal - 1]);
            let next = ordinal + 1;
            assert!(next == ROW_CLASSES.len() || report.rows[ordinal].class != ROW_CLASSES[next]);
            assert!(ordinal != 1 || report.probes.is_empty());
        }
    }

    #[test]
    fn unsupported_targets_are_explicit_no_go() {
        assert!(cranelift_codegen_version_is_compatible("0.133.1"));
        assert!(cranelift_codegen_version_is_compatible("0.133.1-012345678"));
        for version in [
            "0.133.2",
            "0.133.1-",
            "0.133.1-01234567",
            "0.133.1-0123456789",
            "0.133.1-01234567g",
            "0.133.1-012345678-extra",
        ] {
            assert!(!cranelift_codegen_version_is_compatible(version));
        }
        let mut versions = [CRANELIFT_VERSION; 5];
        versions[0] = "0.133.1-012345678";
        assert_eq!(normalized_pinned_versions(versions), [CRANELIFT_VERSION; 5]);
        for index in 1..5 {
            versions = [CRANELIFT_VERSION; 5];
            versions[index] = "0.133.1-012345678";
            assert_ne!(normalized_pinned_versions(versions), [CRANELIFT_VERSION; 5]);
        }
        assert!(target_is_required("x86_64", "windows", "msvc"));
        assert!(target_is_required("x86_64", "linux", "gnu"));
        for target in [
            ("x86_64", "windows", "gnu"),
            ("x86_64", "linux", "musl"),
            ("aarch64", "linux", "gnu"),
            ("x86_64", "macos", "other"),
            ("wasm32", "unknown", "other"),
            ("s390x", "linux", "gnu"),
        ] {
            assert!(!target_is_required(target.0, target.1, target.2));
        }
        let report = valid_probe(Some(BackendProbeFault::RejectTargetIsa));
        assert_eq!(report.rows[7].class, "unsupported_or_unavailable_target");
        assert_eq!(report.backend_ready, 0);
        assert!(report.probes.is_empty());
    }
}
