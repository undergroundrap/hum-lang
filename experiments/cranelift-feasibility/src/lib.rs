use cranelift_codegen::ir::{AbiParam, InstBuilder, MemFlagsData, UserFuncName, types};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, default_libcall_names};
use cranelift_object::{ObjectBuilder, ObjectModule};
use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

pub const GENERATED_SYMBOL: &str = "hum_spike_checked_add";

#[derive(Debug, Clone)]
pub struct RepoPaths {
    pub root: PathBuf,
    pub hum: PathBuf,
    pub minimal_add: PathBuf,
    pub unsupported_subtract: PathBuf,
    pub native_driver: PathBuf,
    pub proof_dir: PathBuf,
}

impl RepoPaths {
    pub fn discover() -> Result<Self, String> {
        let experiment = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = experiment
            .parent()
            .and_then(Path::parent)
            .ok_or("experiment must remain two levels below the repository root")?
            .to_path_buf();
        let hum_name = if cfg!(windows) { "hum.exe" } else { "hum" };
        let hum = std::env::var_os("HUM_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("target").join("debug").join(hum_name));
        Ok(Self {
            minimal_add: root.join("examples/core/minimal_add.hum"),
            unsupported_subtract: experiment.join("cases/unsupported_subtract.hum"),
            native_driver: experiment.join("support/native_driver.rs"),
            proof_dir: experiment.join("target/proof"),
            root,
            hum,
        })
    }
}

#[derive(Debug)]
pub struct CommandEvidence {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
    pub elapsed: Duration,
}

impl CommandEvidence {
    fn from_output(output: Output, elapsed: Duration) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            status: output.status.code().unwrap_or(-1),
            elapsed,
        }
    }
}

#[derive(Debug)]
pub struct HumEvidence {
    pub check: CommandEvidence,
    pub core_lower: Value,
    pub core_verify: Value,
    pub resolve: Value,
    pub full_type_check: Value,
    pub ir_readiness: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringBlocker {
    pub code: &'static str,
    pub missing_facts: Vec<&'static str>,
    pub current_ir_ready: u64,
    pub current_type_status: String,
    pub missing_passes: Vec<String>,
}

#[derive(Debug)]
pub struct JitEvidence {
    pub clif: String,
    pub cases: Vec<AddCase>,
    pub elapsed: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddCase {
    pub left: i64,
    pub right: i64,
    pub status: i32,
    pub value: Option<i64>,
}

#[derive(Debug)]
pub struct ObjectEvidence {
    pub object_path: PathBuf,
    pub clif_path: PathBuf,
    pub object_bytes: usize,
    pub clif: String,
    pub elapsed: Duration,
}

#[derive(Debug)]
pub struct NativeEvidence {
    pub executable_path: PathBuf,
    pub compile: CommandEvidence,
    pub ordinary: Vec<(i64, i64, CommandEvidence)>,
    pub overflow: CommandEvidence,
}

pub fn collect_hum_evidence(paths: &RepoPaths, source: &Path) -> Result<HumEvidence, String> {
    let (check, core_lower) = collect_checked_core(paths, source)?;
    Ok(HumEvidence {
        check,
        core_lower,
        core_verify: run_hum_json(paths, "core-verify", source)?,
        resolve: run_hum_json(paths, "resolve", source)?,
        full_type_check: run_hum_json(paths, "full-type-check", source)?,
        ir_readiness: run_hum_json(paths, "ir-readiness", source)?,
    })
}

pub fn collect_checked_core(
    paths: &RepoPaths,
    source: &Path,
) -> Result<(CommandEvidence, Value), String> {
    require_file(&paths.hum, "Hum executable")?;
    require_file(source, "Hum source")?;
    let check = run_hum(paths, [OsStr::new("check"), source.as_os_str()])?;
    if check.status != 0 {
        return Err(format!(
            "Hum check rejected {}: {}",
            source.display(),
            check.stderr
        ));
    }
    Ok((check, run_hum_json(paths, "core-lower", source)?))
}

pub fn assess_hum_add(evidence: &HumEvidence) -> Result<(), LoweringBlocker> {
    let expression = &evidence.core_lower["core_items"][0]["operations"][0]["expression"];
    let ir_ready = evidence.core_lower["summary"]["ir_ready"]
        .as_u64()
        .unwrap_or(u64::MAX);
    let type_status = expression["type_status"]
        .as_str()
        .unwrap_or("missing")
        .to_string();
    let missing_passes = evidence.ir_readiness["lowering_candidates"][0]["missing_passes"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    Err(LoweringBlocker {
        code: "production_artifact_not_backend_lowerable_v0",
        missing_facts: vec![
            "ordered expression-child identities",
            "operand-to-resolver-definition bindings",
            "backend-consumable checked expression type",
            "implemented IR verification pass",
            "IR-ready production artifact",
        ],
        current_ir_ready: ir_ready,
        current_type_status: type_status,
        missing_passes,
    })
}

pub fn operator_from_core(evidence: &HumEvidence) -> Option<&str> {
    operator_from_core_report(&evidence.core_lower)
}

pub fn operator_from_core_report(report: &Value) -> Option<&str> {
    report["core_items"][0]["operations"][0]["expression"]["operator"].as_str()
}

pub fn require_capability_probe_operator(report: &Value, expected: &str) -> Result<(), String> {
    match operator_from_core_report(report) {
        Some(operator) if operator == expected => Ok(()),
        Some(operator) => Err(format!(
            "unsupported Core operator `{operator}`; no interpreter fallback is permitted"
        )),
        None => {
            Err("Core artifact has no expression operator; no fallback is permitted".to_string())
        }
    }
}

pub fn run_hum_add(paths: &RepoPaths, left: i64, right: i64) -> Result<CommandEvidence, String> {
    let left = left.to_string();
    let right = right.to_string();
    run_hum(
        paths,
        [
            OsStr::new("run"),
            paths.minimal_add.as_os_str(),
            OsStr::new("--entry"),
            OsStr::new("add"),
            OsStr::new("--args"),
            OsStr::new(&left),
            OsStr::new(&right),
        ],
    )
}

pub fn build_jit_checked_add(cases: &[(i64, i64)]) -> Result<JitEvidence, String> {
    let started = Instant::now();
    let mut flag_builder = settings::builder();
    flag_builder
        .set("opt_level", "speed")
        .map_err(|error| error.to_string())?;
    let isa = cranelift_native::builder()
        .map_err(|error| error.to_string())?
        .finish(settings::Flags::new(flag_builder))
        .map_err(|error| error.to_string())?;
    let builder = JITBuilder::with_isa(isa, default_libcall_names());
    let mut module = JITModule::new(builder);
    let (function_id, clif) = define_checked_add(&mut module)?;
    module
        .finalize_definitions()
        .map_err(|error| error.to_string())?;
    let function = module.get_finalized_function(function_id);
    let mut results = Vec::with_capacity(cases.len());
    for &(left, right) in cases {
        let mut value = 0_i64;
        // SAFETY: Cranelift emitted this pointer from the exact C-compatible
        // signature `(i64, i64, *mut i64) -> i32`; the module remains alive.
        let callable: unsafe extern "C" fn(i64, i64, *mut i64) -> i32 =
            unsafe { std::mem::transmute(function) };
        // SAFETY: `value` is a valid writable i64 for the duration of the call.
        let status = unsafe { callable(left, right, &mut value) };
        results.push(AddCase {
            left,
            right,
            status,
            value: (status == 0).then_some(value),
        });
    }
    // SAFETY: all derived function pointers are out of use at this point.
    unsafe { module.free_memory() };
    Ok(JitEvidence {
        clif,
        cases: results,
        elapsed: started.elapsed(),
    })
}

pub fn emit_checked_add_object(paths: &RepoPaths) -> Result<ObjectEvidence, String> {
    fs::create_dir_all(&paths.proof_dir).map_err(|error| error.to_string())?;
    let started = Instant::now();
    let mut flag_builder = settings::builder();
    flag_builder
        .set("opt_level", "speed")
        .map_err(|error| error.to_string())?;
    let isa = cranelift_native::builder()
        .map_err(|error| error.to_string())?
        .finish(settings::Flags::new(flag_builder))
        .map_err(|error| error.to_string())?;
    let object_builder =
        ObjectBuilder::new(isa, "hum_cranelift_feasibility", default_libcall_names())
            .map_err(|error| error.to_string())?;
    let mut module = ObjectModule::new(object_builder);
    let (_, clif) = define_checked_add(&mut module)?;
    let bytes = module.finish().emit().map_err(|error| error.to_string())?;
    let object_path = paths.proof_dir.join(if cfg!(windows) {
        "hum_spike_checked_add.obj"
    } else {
        "hum_spike_checked_add.o"
    });
    let clif_path = paths.proof_dir.join("hum_spike_checked_add.clif");
    fs::write(&object_path, &bytes).map_err(|error| error.to_string())?;
    fs::write(&clif_path, &clif).map_err(|error| error.to_string())?;
    Ok(ObjectEvidence {
        object_path,
        clif_path,
        object_bytes: bytes.len(),
        clif,
        elapsed: started.elapsed(),
    })
}

pub fn link_and_run_native(
    paths: &RepoPaths,
    object: &ObjectEvidence,
    cases: &[(i64, i64)],
) -> Result<NativeEvidence, String> {
    require_file(&paths.native_driver, "native driver source")?;
    let executable_path = paths.proof_dir.join(if cfg!(windows) {
        "hum_spike_native.exe"
    } else {
        "hum_spike_native"
    });
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let started = Instant::now();
    let output = Command::new(&rustc)
        .current_dir(&paths.root)
        .arg(&paths.native_driver)
        .arg("--edition=2024")
        .arg("-C")
        .arg("opt-level=2")
        .arg("-C")
        .arg(format!("link-arg={}", object.object_path.display()))
        .arg("-o")
        .arg(&executable_path)
        .output()
        .map_err(|error| format!("failed to invoke rustc: {error}"))?;
    let compile = CommandEvidence::from_output(output, started.elapsed());
    if compile.status != 0 {
        return Err(format!("native link failed: {}", compile.stderr));
    }

    let mut ordinary = Vec::new();
    for &(left, right) in cases {
        let result = run_command(
            Command::new(&executable_path)
                .arg(left.to_string())
                .arg(right.to_string()),
        )?;
        ordinary.push((left, right, result));
    }
    let overflow = run_command(
        Command::new(&executable_path)
            .arg(i64::MAX.to_string())
            .arg("1"),
    )?;
    Ok(NativeEvidence {
        executable_path,
        compile,
        ordinary,
        overflow,
    })
}

fn define_checked_add<M: Module>(
    module: &mut M,
) -> Result<(cranelift_module::FuncId, String), String> {
    let pointer_type = module.target_config().pointer_type();
    let mut signature = module.make_signature();
    signature.params.push(AbiParam::new(types::I64));
    signature.params.push(AbiParam::new(types::I64));
    signature.params.push(AbiParam::new(pointer_type));
    signature.returns.push(AbiParam::new(types::I32));
    let function_id = module
        .declare_function(GENERATED_SYMBOL, Linkage::Export, &signature)
        .map_err(|error| error.to_string())?;

    let mut context = module.make_context();
    context.func.signature = signature;
    context.func.name = UserFuncName::user(0, function_id.as_u32());
    let mut builder_context = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
        let entry = builder.create_block();
        let success = builder.create_block();
        let overflow = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);
        let params = builder.block_params(entry).to_vec();
        let (sum, did_overflow) = builder.ins().sadd_overflow(params[0], params[1]);
        builder
            .ins()
            .brif(did_overflow, overflow, &[], success, &[]);

        builder.switch_to_block(success);
        builder.seal_block(success);
        builder
            .ins()
            .store(MemFlagsData::trusted(), sum, params[2], 0);
        let ok = builder.ins().iconst(types::I32, 0);
        builder.ins().return_(&[ok]);

        builder.switch_to_block(overflow);
        builder.seal_block(overflow);
        let failed = builder.ins().iconst(types::I32, 1);
        builder.ins().return_(&[failed]);
        builder.finalize();
    }
    let clif = context.func.display().to_string();
    module
        .define_function(function_id, &mut context)
        .map_err(|error| error.to_string())?;
    module.clear_context(&mut context);
    Ok((function_id, clif))
}

fn run_hum_json(paths: &RepoPaths, command: &str, source: &Path) -> Result<Value, String> {
    let evidence = run_hum(
        paths,
        [
            OsStr::new(command),
            OsStr::new("--format=json"),
            source.as_os_str(),
        ],
    )?;
    if evidence.status != 0 {
        return Err(format!("Hum {command} failed: {}", evidence.stderr));
    }
    serde_json::from_str(&evidence.stdout)
        .map_err(|error| format!("Hum {command} returned invalid JSON: {error}"))
}

fn run_hum<I, S>(paths: &RepoPaths, args: I) -> Result<CommandEvidence, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(&paths.hum);
    command.current_dir(&paths.root).args(args);
    run_command(&mut command)
}

fn run_command(command: &mut Command) -> Result<CommandEvidence, String> {
    let started = Instant::now();
    let output = command
        .output()
        .map_err(|error| format!("failed to run {command:?}: {error}"))?;
    Ok(CommandEvidence::from_output(output, started.elapsed()))
}

fn require_file(path: &Path, label: &str) -> Result<(), String> {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("{label} does not exist at {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORDINARY_CASES: &[(i64, i64)] = &[(2, 3), (-7, 11), (0, 0), (1_000_000, 24)];

    #[test]
    fn real_hum_artifact_is_checked_but_honestly_blocked_from_backend_lowering() {
        let paths = RepoPaths::discover().expect("paths");
        let evidence = collect_hum_evidence(&paths, &paths.minimal_add).expect("Hum evidence");
        assert_eq!(evidence.core_lower["schema"], "hum.core_lower.v0");
        assert_eq!(evidence.core_verify["schema"], "hum.core_verify.v0");
        assert_eq!(evidence.resolve["schema"], "hum.resolve.v0");
        assert_eq!(evidence.full_type_check["schema"], "hum.full_type_check.v0");
        assert_eq!(evidence.ir_readiness["schema"], "hum.ir_readiness.v0");
        assert_eq!(operator_from_core(&evidence), Some("add"));
        let blocker = assess_hum_add(&evidence).expect_err("artifact must fail closed");
        assert_eq!(blocker.current_ir_ready, 0);
        assert_eq!(blocker.current_type_status, "not_type_checked_v0");
        assert!(
            blocker
                .missing_passes
                .iter()
                .any(|pass| pass == "ir_verify")
        );
        assert_eq!(blocker.missing_facts.len(), 5);
    }

    #[test]
    fn real_unsupported_core_operator_is_rejected_without_fallback() {
        let paths = RepoPaths::discover().expect("paths");
        let (_, core) = collect_checked_core(&paths, &paths.unsupported_subtract)
            .expect("checked Core evidence");
        assert_eq!(operator_from_core_report(&core), Some("sub"));
        let error =
            require_capability_probe_operator(&core, "add").expect_err("subtract must be rejected");
        assert!(error.contains("unsupported Core operator `sub`"));
        assert!(error.contains("no interpreter fallback"));
    }

    #[test]
    fn cranelift_jit_has_exact_checked_add_abi() {
        let evidence = build_jit_checked_add(
            &ORDINARY_CASES
                .iter()
                .copied()
                .chain([(i64::MAX, 1), (i64::MIN, -1)])
                .collect::<Vec<_>>(),
        )
        .expect("JIT evidence");
        for case in evidence.cases {
            match case.left.checked_add(case.right) {
                Some(expected) => {
                    assert_eq!(case.status, 0);
                    assert_eq!(case.value, Some(expected));
                }
                None => {
                    assert_eq!(case.status, 1);
                    assert_eq!(case.value, None);
                }
            }
        }
        assert!(evidence.clif.contains("sadd_overflow"));
        assert!(evidence.clif.contains("store"));
    }

    #[test]
    fn native_object_links_and_matches_real_interpreter_on_ordinary_inputs() {
        if !cfg!(windows) {
            return;
        }
        let paths = RepoPaths::discover().expect("paths");
        let object = emit_checked_add_object(&paths).expect("object");
        let native = link_and_run_native(&paths, &object, ORDINARY_CASES).expect("native proof");
        for (left, right, generated) in native.ordinary {
            let interpreted = run_hum_add(&paths, left, right).expect("Hum interpreter");
            assert_eq!(generated.status, 0);
            assert_eq!(interpreted.status, 0);
            assert_eq!(generated.stdout, interpreted.stdout);
        }
        let interpreted_overflow = run_hum_add(&paths, i64::MAX, 1).expect("Hum overflow");
        assert_eq!(native.overflow.status, 2);
        assert_eq!(interpreted_overflow.status, 2);
        assert_eq!(native.overflow.stderr, interpreted_overflow.stderr);
    }
}
