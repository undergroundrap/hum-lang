use cranelift_codegen::settings::{self, Configurable};
use serde_json::{Value, json};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

pub const STOP_CODE: &str = "verified_backend_input_artifact_absent_v0";
pub const REQUIRED_ARTIFACT_SCHEMA: &str = "hum.backend_input.v0";
pub const REQUIRED_VERIFIER_SCHEMA: &str = "hum.ir_verify.v0";

#[derive(Debug, Clone)]
pub struct RepoPaths {
    pub root: PathBuf,
    pub hum: PathBuf,
    pub source: PathBuf,
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
            source: root.join("examples/core/minimal_add.hum"),
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
    pub effect_check: Value,
    pub ownership_check: Value,
    pub resource_check: Value,
    pub profile_check: Value,
    pub ir_readiness: Value,
    pub stage_milliseconds: Vec<(&'static str, u128)>,
    pub stage_statuses: Vec<(&'static str, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringStop {
    pub code: &'static str,
    pub requirement: &'static str,
    pub owner: &'static str,
    pub required_shape: &'static str,
    pub required_transport: &'static str,
    pub observed: String,
}

#[derive(Debug)]
pub struct AttemptReport {
    pub source: PathBuf,
    pub cranelift_codegen_version: &'static str,
    pub target_triple: String,
    pub hum_check_status: i32,
    pub stop: LoweringStop,
    pub findings: Value,
    pub stage_milliseconds: Vec<(&'static str, u128)>,
    pub stage_statuses: Vec<(&'static str, i32)>,
    pub total_elapsed: Duration,
}

impl AttemptReport {
    pub fn to_json(&self) -> Value {
        json!({
            "experiment": "hum.cranelift_lowering_contract.v0",
            "source": self.source.to_string_lossy(),
            "cranelift": {
                "codegen_version": self.cranelift_codegen_version,
                "target_triple": self.target_triple,
                "initialization": "go"
            },
            "hum_check_exit": self.hum_check_status,
            "lowering_attempt": "no_go",
            "clif_instructions_emitted": 0,
            "stop": {
                "code": self.stop.code,
                "requirement": self.stop.requirement,
                "owner": self.stop.owner,
                "required_shape": self.stop.required_shape,
                "required_transport": self.stop.required_transport,
                "observed": self.stop.observed
            },
            "findings": self.findings,
            "stage_milliseconds": self
                .stage_milliseconds
                .iter()
                .map(|(stage, elapsed)| json!({"stage": stage, "elapsed_ms": elapsed}))
                .collect::<Vec<_>>(),
            "stage_exit_statuses": self
                .stage_statuses
                .iter()
                .map(|(stage, status)| json!({"stage": stage, "exit": status}))
                .collect::<Vec<_>>(),
            "total_elapsed_ms": self.total_elapsed.as_millis()
        })
    }
}

pub fn attempt_real_lowering(paths: &RepoPaths) -> Result<AttemptReport, String> {
    require_file(&paths.hum, "Hum executable")?;
    require_file(&paths.source, "Hum source")?;
    let total = Instant::now();

    let mut flag_builder = settings::builder();
    flag_builder
        .set("opt_level", "speed")
        .map_err(|error| format!("Cranelift settings failed: {error}"))?;
    let isa = cranelift_native::builder()
        .map_err(|error| format!("Cranelift native target is unavailable: {error}"))?
        .finish(settings::Flags::new(flag_builder))
        .map_err(|error| format!("Cranelift ISA construction failed: {error}"))?;

    let evidence = collect_hum_evidence(paths)?;
    if evidence.check.status != 0 {
        return Err(format!(
            "real Hum source failed `hum check`: {}",
            evidence.check.stderr
        ));
    }

    let stop = require_verified_backend_input(&evidence)
        .expect_err("current production outputs must fail closed before Cranelift IR emission");
    let findings = findings_from(&evidence);

    Ok(AttemptReport {
        source: paths.source.clone(),
        cranelift_codegen_version: cranelift_codegen::VERSION,
        target_triple: isa.triple().to_string(),
        hum_check_status: evidence.check.status,
        stop,
        findings,
        stage_milliseconds: evidence.stage_milliseconds,
        stage_statuses: evidence.stage_statuses,
        total_elapsed: total.elapsed(),
    })
}

pub fn collect_hum_evidence(paths: &RepoPaths) -> Result<HumEvidence, String> {
    let check = run_hum(
        paths,
        [
            OsStr::new("check"),
            OsStr::new("--format=json"),
            paths.source.as_os_str(),
        ],
    )?;
    let mut timings = vec![("check", check.elapsed.as_millis())];
    let mut statuses = vec![("check", check.status)];
    let (core_lower, elapsed, status) = run_hum_json(paths, "core-lower")?;
    timings.push(("core-lower", elapsed.as_millis()));
    statuses.push(("core-lower", status));
    let (core_verify, elapsed, status) = run_hum_json(paths, "core-verify")?;
    timings.push(("core-verify", elapsed.as_millis()));
    statuses.push(("core-verify", status));
    let (resolve, elapsed, status) = run_hum_json(paths, "resolve")?;
    timings.push(("resolve", elapsed.as_millis()));
    statuses.push(("resolve", status));
    let (full_type_check, elapsed, status) = run_hum_json(paths, "full-type-check")?;
    timings.push(("full-type-check", elapsed.as_millis()));
    statuses.push(("full-type-check", status));
    let (effect_check, elapsed, status) = run_hum_json(paths, "effect-check")?;
    timings.push(("effect-check", elapsed.as_millis()));
    statuses.push(("effect-check", status));
    let (ownership_check, elapsed, status) = run_hum_json(paths, "ownership-check")?;
    timings.push(("ownership-check", elapsed.as_millis()));
    statuses.push(("ownership-check", status));
    let (resource_check, elapsed, status) = run_hum_json(paths, "resource-check")?;
    timings.push(("resource-check", elapsed.as_millis()));
    statuses.push(("resource-check", status));
    let (profile_check, elapsed, status) = run_hum_json(paths, "profile-check")?;
    timings.push(("profile-check", elapsed.as_millis()));
    statuses.push(("profile-check", status));
    let (ir_readiness, elapsed, status) = run_hum_json(paths, "ir-readiness")?;
    timings.push(("ir-readiness", elapsed.as_millis()));
    statuses.push(("ir-readiness", status));

    Ok(HumEvidence {
        check,
        core_lower,
        core_verify,
        resolve,
        full_type_check,
        effect_check,
        ownership_check,
        resource_check,
        profile_check,
        ir_readiness,
        stage_milliseconds: timings,
        stage_statuses: statuses,
    })
}

fn require_verified_backend_input(
    evidence: &HumEvidence,
) -> Result<VerifiedBackendInput<'_>, LoweringStop> {
    let core_schema = evidence.core_lower["schema"]
        .as_str()
        .unwrap_or("missing_schema");
    let core_status = evidence.core_lower["lowering_status"]
        .as_str()
        .unwrap_or("missing_status");
    let verification = evidence.core_verify["verification_status"]
        .as_str()
        .unwrap_or("missing_status");
    let ir_ready = evidence.core_lower["summary"]["ir_ready"]
        .as_u64()
        .unwrap_or_default();
    let missing_passes =
        string_array(&evidence.ir_readiness["lowering_candidates"][0]["missing_passes"]).join(",");
    Err(LoweringStop {
        code: STOP_CODE,
        requirement: "one verifier-bound backend input",
        owner: "Hum IR emission plus ir_verify",
        required_shape: "a hum.backend_input.v0 artifact accepted by hum.ir_verify.v0 and exposed only as an opaque VerifiedBackendInput capability",
        required_transport: "the adapter receives the exact verified in-process capability; serialized JSON is observational and must be re-verified in the consuming process",
        observed: format!(
            "schema={core_schema}; lowering_status={core_status}; core_verification={verification}; ir_ready={ir_ready}; missing_passes={missing_passes}"
        ),
    })
}

#[derive(Debug)]
struct VerifiedBackendInput<'a> {
    #[allow(dead_code)]
    artifact: &'a [u8],
}

fn findings_from(evidence: &HumEvidence) -> Value {
    let expression = &evidence.core_lower["core_items"][0]["operations"][0]["expression"];
    let references = evidence.resolve["references"]
        .as_array()
        .map(Vec::len)
        .unwrap_or_default();
    let resolved_references = evidence.resolve["summary"]["resolved_references"]
        .as_u64()
        .unwrap_or_default();
    let full_type_statement = &evidence.full_type_check["typed_items"][0]["statements"][0];
    let missing_passes =
        string_array(&evidence.ir_readiness["lowering_candidates"][0]["missing_passes"]);
    let blocking_reasons =
        string_array(&evidence.ir_readiness["lowering_candidates"][0]["blocking_reasons"]);

    json!({
        "expression_structure": {
            "status": "boundary_no_go",
            "core_kind": expression["kind"],
            "core_operator": expression["operator"],
            "core_node_count": expression["node_count"],
            "ordered_child_ids_present": expression.get("ordered_child_ids").is_some(),
            "nodes_present": expression.get("nodes").is_some(),
            "text_present": expression.get("text").is_some(),
            "reason": "the serialized Core expression is a preview and exposes no canonical node table or ordered child identities"
        },
        "resolver_bindings": {
            "status": "boundary_no_go",
            "references": references,
            "resolved_references": resolved_references,
            "public_reference_has_definition_id": evidence.resolve["references"][0]
                .get("resolved_definition_id")
                .is_some(),
            "public_reference_has_canonical_node_id": evidence.resolve["references"][0]
                .get("canonical_node_id")
                .is_some(),
            "core_operand_binding_table_present": evidence.core_lower
                .get("operand_definition_bindings")
                .is_some(),
            "reason": "resolution succeeds, but no verifier-bound artifact connects each Core operand node to its resolver definition"
        },
        "checked_types": {
            "status": "boundary_no_go",
            "core_type_status": expression["type_status"],
            "core_type_text": expression["type_text"],
            "statement_status": full_type_statement["status"],
            "statement_actual_type": full_type_statement["actual_type"],
            "statement_type_source": full_type_statement["type_source"],
            "reason": "the accepted statement type is reported separately and is not attached to the canonical Core expression node"
        },
        "effects_and_authority": {
            "status": "boundary_no_go",
            "effect_status": evidence.effect_check["status"],
            "effect_blockers": evidence.effect_check["summary"]["blocking_issues"],
            "ownership_status": evidence.ownership_check["status"],
            "ownership_blockers": evidence.ownership_check["summary"]["blocking_issues"],
            "resource_status": evidence.resource_check["status"],
            "resource_blockers": evidence.resource_check["summary"]["blocking_issues"],
            "profile_status": evidence.profile_check["status"],
            "reason": "accepted effect and ownership reports are not bound to Core nodes, while the real program remains blocked by resource policy"
        },
        "verification": {
            "status": "no_go",
            "core_verify_status": evidence.core_verify["verification_status"],
            "core_verify_ir_ready": evidence.core_verify["summary"]["ir_ready"],
            "ir_verify_present": !missing_passes.iter().any(|pass| pass == "ir_verify"),
            "missing_passes": missing_passes,
            "blocking_reasons": blocking_reasons,
            "reason": "core-verify emits a separate report; it does not return a non-forgeable capability bound to backend input bytes, and ir_verify is not implemented"
        },
        "artifact": {
            "status": "no_go",
            "required_schema": REQUIRED_ARTIFACT_SCHEMA,
            "required_verifier_schema": REQUIRED_VERIFIER_SCHEMA,
            "ir_ready": evidence.ir_readiness["summary"]["ready_for_ir"],
            "reason": "no Hum IR or backend-input artifact exists"
        }
    })
}

fn string_array(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn run_hum_json(
    paths: &RepoPaths,
    command: &'static str,
) -> Result<(Value, Duration, i32), String> {
    let evidence = run_hum(
        paths,
        [
            OsStr::new(command),
            OsStr::new("--format=json"),
            paths.source.as_os_str(),
        ],
    )?;
    let value = serde_json::from_str(&evidence.stdout).map_err(|error| {
        format!(
            "Hum {command} returned invalid JSON at exit {}: {error}; stderr={}",
            evidence.status, evidence.stderr
        )
    })?;
    Ok((value, evidence.elapsed, evidence.status))
}

fn run_hum<I, S>(paths: &RepoPaths, args: I) -> Result<CommandEvidence, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let started = Instant::now();
    let output = Command::new(&paths.hum)
        .current_dir(&paths.root)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run {}: {error}", paths.hum.display()))?;
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

    #[test]
    fn real_program_stops_before_cranelift_ir_without_verified_artifact() {
        let paths = RepoPaths::discover().expect("paths");
        let report = attempt_real_lowering(&paths).expect("bounded NO-GO is a valid result");
        assert_eq!(report.hum_check_status, 0);
        assert_eq!(report.stop.code, STOP_CODE);
        assert_eq!(report.to_json()["clif_instructions_emitted"], 0);
        assert_eq!(
            report.to_json()["findings"]["artifact"]["required_schema"],
            REQUIRED_ARTIFACT_SCHEMA
        );
    }

    #[test]
    fn forged_verified_flag_on_cli_projection_is_not_authority() {
        let paths = RepoPaths::discover().expect("paths");
        let mut evidence = collect_hum_evidence(&paths).expect("real Hum evidence");
        evidence.core_lower["verified"] = Value::Bool(true);
        evidence.core_lower["verification_status"] = Value::String("verified".to_string());
        let stop = require_verified_backend_input(&evidence)
            .expect_err("a caller-supplied JSON flag must not mint authority");
        assert_eq!(stop.code, STOP_CODE);
        assert!(stop.observed.contains("unverified_core_artifact_v0"));
    }

    #[test]
    fn probe_reports_every_backend_contract_domain() {
        let paths = RepoPaths::discover().expect("paths");
        let report = attempt_real_lowering(&paths).expect("bounded NO-GO");
        let findings = &report.to_json()["findings"];
        for key in [
            "expression_structure",
            "resolver_bindings",
            "checked_types",
            "effects_and_authority",
            "verification",
            "artifact",
        ] {
            assert!(findings.get(key).is_some(), "missing contract domain {key}");
        }
    }
}
