mod cleanup;
mod command;
mod commit_message;
mod identity;
mod shell;
mod status;
mod summary;
mod workorder;

use std::ffi::OsString;
use std::{
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::Path,
    process::{Command as ProcessCommand, ExitCode, Output},
};

use cleanup::OwnedResource;
use command::{Command, EvidenceProfile, MessageInput, legacy_invocation};
use commit_message::{validate_message_file, validate_subject};
use identity::CandidateIdentity;
use summary::{quoted, string_array};

fn fail(message: impl std::fmt::Display) -> ExitCode {
    eprintln!("hum-dev: {message}");
    ExitCode::from(2)
}

#[rustfmt::skip]
fn clean_stdout(program: &shell::ExecutableBinding, args: &[&str], env: &[(&str, OsString)]) -> Result<String, String> {
    program.reauthenticate()?; let label = program.predicate();
    let output = ProcessCommand::new(program.path()).env_clear().envs(env.iter().cloned()).args(args).output().map_err(|error| format!("{label}: {error}"))?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!("{label}: process failed"));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("{label}: stdout is not UTF-8"))
}

#[cfg(windows)]#[rustfmt::skip]
fn clean_cmd_stdout(program: &Path, script: &str, env: &[(&str, OsString)], label: &str) -> Result<String, String> { use std::os::windows::process::CommandExt; let output = ProcessCommand::new(program).env_clear().envs(env.iter().cloned()).args(["/d", "/s", "/c"]).raw_arg(script).output().map_err(|error| format!("{label}: launch: {error}"))?; let stderr = String::from_utf8(output.stderr).map_err(|_| format!("{label}: stderr is not UTF-8"))?; if !output.status.success() || !stderr.is_empty() { let code = output.status.code().unwrap_or(130); return Err(format!("{label}: exit {code}; stderr={:?}", stderr)); } String::from_utf8(output.stdout).map_err(|_| format!("{label}: stdout is not UTF-8")) }

#[rustfmt::skip]
fn candidate_json(candidate: &CandidateIdentity) -> String {
    let binding = candidate.binding(); assert!(binding.matches(&candidate.binding()), "candidate binding drifted");
    let mut out = String::from("{\"schema\":\"hum.candidate_identity.v1\",\"commit\":"); quoted(&mut out, &candidate.commit); out.push_str(",\"parents\":"); string_array(&mut out, &candidate.parents); out.push_str(",\"tree\":"); quoted(&mut out, &candidate.tree); out.push_str(",\"head_state\":"); quoted(&mut out, if candidate.head_ref.is_some() { "symbolic" } else { "detached" }); out.push_str(",\"head_ref\":"); if let Some(value) = &candidate.head_ref { quoted(&mut out, value); } else { out.push_str("null"); }
    out.push_str(&format!(",\"index_clean\":{},\"worktree_clean\":{},\"untracked_clean\":{},\"state_sha256\":", candidate.index_clean, candidate.worktree_clean, candidate.untracked_clean)); quoted(&mut out, &binding.state_sha256);
    out.push_str(",\"refs\":["); for (index, item) in candidate.refs.iter().enumerate() { if index != 0 { out.push(','); } out.push_str("{\"name\":"); quoted(&mut out, &item.name); out.push_str(",\"oid\":"); quoted(&mut out, &item.oid); out.push('}'); }
    out.push_str("],\"index\":["); for (index, item) in candidate.index_entries.iter().enumerate() { if index != 0 { out.push(','); } out.push_str("{\"path\":"); quoted(&mut out, &item.path); out.push_str(",\"mode\":"); quoted(&mut out, &item.mode); out.push_str(",\"oid\":"); quoted(&mut out, &item.oid); out.push_str(&format!(",\"stage\":{},\"intent_to_add\":{}}}", item.stage, item.intent_to_add)); }
    out.push_str(&format!("],\"raw\":{{\"additions\":{},\"deletions\":{}}},\"whitespace\":{{\"additions\":{},\"deletions\":{}}},\"paths\":[", candidate.raw_additions, candidate.raw_deletions, candidate.whitespace_additions, candidate.whitespace_deletions));
    for (index, path) in candidate.paths.iter().enumerate() { if index != 0 { out.push(','); } out.push_str("{\"path\":"); quoted(&mut out, &path.path); out.push_str(",\"head_mode\":"); if let Some(value) = &path.head_mode { quoted(&mut out, value); } else { out.push_str("null"); } out.push_str(",\"head_oid\":"); if let Some(value) = &path.head_oid { quoted(&mut out, value); } else { out.push_str("null"); } out.push_str(",\"worktree_kind\":"); quoted(&mut out, &path.worktree_kind); out.push_str(",\"worktree_mode\":"); if let Some(value) = &path.worktree_mode { quoted(&mut out, value); } else { out.push_str("null"); } out.push_str(",\"worktree_sha256\":"); if let Some(value) = &path.worktree_sha256 { quoted(&mut out, value); } else { out.push_str("null"); } out.push_str(&format!(",\"bytes\":{},\"untracked\":{}}}", path.bytes, path.untracked)); }
    out.push_str("]}\n"); out
}

fn launch_legacy(
    executable: &OsStr,
    invocation: &command::LegacyInvocation,
) -> Result<Output, String> {
    if invocation.executable != "pwsh" {
        return Err("legacy executable selector must be exactly pwsh".into());
    }
    let resolved = Path::new(executable);
    if !resolved.is_absolute() {
        return Err("pwsh_executable: explicit absolute --pwsh path is required".into());
    }
    let executable =
        shell::ExecutableBinding::authenticate(resolved.to_owned(), "pwsh_executable")?;
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let environment = command::production_environment(&repository)?;
    shell::PwshRequest {
        executable,
        repository: repository.clone(),
        script: repository.join(invocation.script),
        arguments: invocation.arguments.iter().map(Into::into).collect(),
        environment,
    }
    .launch()
}

#[rustfmt::skip]
fn run_legacy(profile: EvidenceProfile) -> Result<i32, String> {
    let _ = profile;
    Err("explicit orchestration binding is missing".into())
}

#[rustfmt::skip]
fn run_legacy_bound(profile: EvidenceProfile, pwsh: &Path) -> Result<i32, String> {
    let invocation = legacy_invocation(profile);
    let output = launch_legacy(pwsh.as_os_str(), &invocation)?;
    io::stdout().write_all(&output.stdout).map_err(|error| error.to_string())?; io::stderr().write_all(&output.stderr).map_err(|error| error.to_string())?;
    Ok(output.status.code().unwrap_or(130))
}

fn execute(command: Command) -> Result<i32, String> {
    match command {
        Command::CommitMessage(MessageInput::Subject(subject)) => {
            validate_subject(&subject).map_err(|error| error.to_string())?;
            println!(
                "accepted|subject_sha256={}",
                hum_sha256::digest_hex(subject.as_bytes())
            );
            Ok(0)
        }
        Command::CommitMessage(MessageInput::File(path)) => {
            let subject = validate_message_file(&path).map_err(|error| error.to_string())?;
            println!(
                "accepted|subject_sha256={}",
                hum_sha256::digest_hex(subject.as_bytes())
            );
            Ok(0)
        }
        Command::CandidateIdentity(repository) => {
            print!("{}", candidate_json(&CandidateIdentity::read(&repository)?));
            Ok(0)
        }
        Command::Evidence(EvidenceProfile::Status) => {
            print!("{}", status::run(&status::request_from_environment()?)?);
            Ok(0)
        }
        Command::Evidence(profile) => run_legacy(profile),
        Command::EvidenceBound(profile, pwsh) => run_legacy_bound(profile, &pwsh),
        Command::EvidenceSummarizeBound(output, pwsh) => {
            let pwsh = shell::ExecutableBinding::authenticate(pwsh, "pwsh_executable")?;
            let _unit_a_opacity = summary::summarize_without_authenticated_records;
            let executable = fs::read(std::env::current_exe().map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
            let candidate = CandidateIdentity::read(Path::new("."))?;
            if !candidate.index_clean
                || !candidate.worktree_clean
                || !candidate.untracked_clean
                || candidate.parents.len() != 1
            {
                return Err("summary producer candidate is not one clean linear commit".into());
            }
            let cargo_lock = fs::read("Cargo.lock").map_err(|error| error.to_string())?;
            let repository = fs::canonicalize(".").map_err(|error| error.to_string())?;
            let environment = command::production_environment(&repository)?;
            let orchestration = summary::authenticate_pwsh7(&pwsh, &environment)?;
            let job = summary::JobSummary::from_environment(
                &executable,
                status::resolve_current_job_id()?,
                &candidate,
                hum_sha256::digest_hex(&cargo_lock),
                &orchestration,
            )?;
            let bytes = job.canonical_bytes()?;
            fs::write(output, bytes).map_err(|error| error.to_string())?;
            println!("summary_artifact={}", job.artifact_name());
            println!("executable_artifact={}", job.executable_artifact_name());
            Ok(0)
        }
        Command::CleanupVerify => {
            let mut owned = OwnedResource::create("verify").map_err(|error| error.to_string())?;
            owned
                .write("probe", b"owned\n")
                .map_err(|error| error.to_string())?;
            let path = owned.path().display().to_string();
            owned.close().map_err(|error| error.to_string())?;
            println!("cleanup_closed|{}", hum_sha256::digest_hex(path.as_bytes()));
            Ok(0)
        }
        Command::WorkOrderStatusFacts {
            input,
            base_sha256,
            status_body,
            gate_body,
            output,
        } => {
            let facts = workorder::project_status(
                &fs::read(input).map_err(|e| e.to_string())?,
                &base_sha256,
                &fs::read_to_string(status_body).map_err(|e| e.to_string())?,
                &fs::read_to_string(gate_body).map_err(|e| e.to_string())?,
            )?;
            fs::write(output, &facts.bytes).map_err(|e| e.to_string())?;
            println!(
                "status_facts|base_sha256={}|immutable_sha256={}|projected_sha256={}",
                facts.base_sha256, facts.immutable_sha256, facts.projected_sha256
            );
            Ok(0)
        }
    }
}

fn main() -> ExitCode {
    let command = match Command::parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => return fail(error),
    };
    match execute(command) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(code) => u8::try_from(code).map_or(ExitCode::FAILURE, ExitCode::from),
        Err(error) => fail(error),
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod cli {
    use super::{EvidenceProfile, launch_legacy, legacy_invocation};
    use crate::command::LegacyInvocation;
    use std::{ffi::OsStr, path::{Path, PathBuf}, process::{Command, Output}, time::{Duration, Instant}};

    #[derive(Clone)]
    struct Record { exit: Option<i32>, stages: Vec<Vec<u8>>, stdout: Vec<u8>, stderr: Vec<u8> }
    fn record(output: Output) -> Record { Record { exit: output.status.code(), stages: output.stdout.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()).map(<[u8]>::to_vec).collect(), stdout: output.stdout, stderr: output.stderr } }
    fn equivalent(old: &Record, new: &Record) -> Result<(), &'static str> { if old.exit != new.exit { Err("legacy_exit") } else if old.stages != new.stages { Err("legacy_stage_order") } else if old.stdout != new.stdout { Err("legacy_stdout") } else if old.stderr != new.stderr { Err("legacy_stderr") } else { Ok(()) } }
    fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap() }
    fn pwsh(requested: &OsStr) -> PathBuf { let output = Command::new(requested).args(["-NoLogo", "-NoProfile", "-Command", "[Environment]::ProcessPath"]).output().unwrap(); assert!(output.status.success()); PathBuf::from(String::from_utf8(output.stdout).unwrap().trim()).canonicalize().unwrap() }
    fn direct(executable: &Path, script: &str) -> Output { Command::new(executable).current_dir(root()).args(["-NoLogo", "-NoProfile", "-File", script]).output().unwrap() }

    #[test]
    fn legacy_equivalence_preserves_exit_stages_and_stream_hashes() {
        let mappings = [(EvidenceProfile::Focused, "tools/check_all.ps1", &["-EvidenceTier", "Wo25UnitC"][..]), (EvidenceProfile::Full, "tools/check_all.ps1", &["-EvidenceTier", "Fast"][..]), (EvidenceProfile::Exhaustive, "tools/check_all.ps1", &["-EvidenceTier", "Exhaustive"][..]), (EvidenceProfile::Status, "tools/check_workorder_status_boundary.ps1", &[][..])];
        for (profile, script, arguments) in mappings { assert_eq!(legacy_invocation(profile), LegacyInvocation { executable: "pwsh", script, arguments }); }
        let production = include_str!("main.rs").split_once("#[cfg(test)]").unwrap().0;
        assert!(!production.contains(concat!("HUM_DEV_LEGACY_", "EQUIVALENCE_PROBE")));
        assert!(!production.contains("var_os("));
        assert!(production.contains("production_environment"), "legacy environment must be production-owned");
        let probe = concat!("HUM_DEV_LEGACY_", "EQUIVALENCE_PROBE");
        if std::env::var_os(probe).is_some() { for (profile, script, arguments) in mappings { assert_eq!(legacy_invocation(profile), LegacyInvocation { executable: "pwsh", script, arguments }); } return; }
        let poisoned = Command::new(std::env::current_exe().unwrap()).args(["--exact", "cli::legacy_equivalence_preserves_exit_stages_and_stream_hashes", "--nocapture"]).env(probe, "1").output().unwrap();
        assert!(poisoned.status.success(), "{}", String::from_utf8_lossy(&poisoned.stderr));
        let executable = pwsh(OsStr::new("pwsh"));
        let injected = LegacyInvocation { executable: "pwsh", script: "tools/check_alpha_claims.ps1", arguments: &[] };
        assert!(launch_legacy(root().join("Cargo.toml").as_os_str(), &injected).unwrap_err().starts_with("pwsh_version"));
        let mut old_samples = Vec::new(); let mut new_samples = Vec::new();
        for _ in 0..4 { let start = Instant::now(); let old = record(direct(&executable, injected.script)); old_samples.push((old, start.elapsed())); let start = Instant::now(); let new = record(launch_legacy(executable.as_os_str(), &injected).unwrap()); new_samples.push((new, start.elapsed())); }
        for ((old, _), (new, _)) in old_samples.iter().zip(&new_samples) { assert_eq!(equivalent(old, new), Ok(())); }
        let mut old_warm = old_samples[1..].iter().map(|(_, elapsed)| *elapsed).collect::<Vec<_>>(); old_warm.sort_unstable(); let mut new_warm = new_samples[1..].iter().map(|(_, elapsed)| *elapsed).collect::<Vec<_>>(); new_warm.sort_unstable(); assert!(old_warm[1] < Duration::from_secs(60)); assert!(new_warm[1] < Duration::from_secs(60));
        let old = &old_samples[0].0; let new = &new_samples[0].0;
        assert_eq!(hum_sha256::digest_hex(&old.stdout), hum_sha256::digest_hex(&new.stdout)); assert_eq!(hum_sha256::digest_hex(&old.stderr), hum_sha256::digest_hex(&new.stderr));
        let mut changed = new.clone(); changed.exit = Some(97); assert_eq!(equivalent(old, &changed), Err("legacy_exit"));
        let mut changed = new.clone(); changed.stages.reverse(); if changed.stages.len() < 2 { changed.stages.push(b"foreign-stage".to_vec()); } assert_eq!(equivalent(old, &changed), Err("legacy_stage_order"));
        let mut changed = new.clone(); changed.stdout.push(b'x'); assert_eq!(equivalent(old, &changed), Err("legacy_stdout"));
        let mut changed = new.clone(); changed.stderr.push(b'x'); assert_eq!(equivalent(old, &changed), Err("legacy_stderr"));
    }
    #[cfg(windows)]#[test]fn toolchain_discovery_binding_reauthenticates_before_launch(){use crate::{cleanup::OwnedResource,shell::{ExecutableBinding,stable_file_identity}};if let Some(path)=std::env::var_os("HUM_TOOLCHAIN_DISCOVERY_SENTINEL"){std::fs::write(path,b"launched").unwrap();return}let owned=OwnedResource::create("vswhere-binding").unwrap();let bytes=std::fs::read(std::env::current_exe().unwrap()).unwrap();let candidate=owned.write("vswhere.exe",&bytes).unwrap();let other=owned.write("other.exe",&bytes).unwrap();let sentinel=owned.path().join("launched");let env=[("HUM_TOOLCHAIN_DISCOVERY_SENTINEL",sentinel.as_os_str().to_owned())];let args=["--exact","cli::toolchain_discovery_binding_reauthenticates_before_launch","--nocapture"];let honest=ExecutableBinding::authenticate(candidate.clone(),"toolchain_discovery").unwrap();super::clean_stdout(&honest,&args,&env).unwrap();assert!(sentinel.exists());std::fs::remove_file(&sentinel).unwrap();let original_identity=stable_file_identity(&candidate).unwrap();let replaced=ExecutableBinding::authenticate(candidate.clone(),"toolchain_discovery").unwrap();std::fs::remove_file(&candidate).unwrap();std::fs::copy(&other,&candidate).unwrap();assert_eq!(std::fs::read(&candidate).unwrap(),bytes);assert_ne!(stable_file_identity(&candidate).unwrap(),original_identity);assert_eq!(super::clean_stdout(&replaced,&args,&env).unwrap_err(),"toolchain_discovery: file identity changed before launch");assert!(!sentinel.exists());let digest=ExecutableBinding::authenticate(candidate.clone(),"toolchain_discovery").unwrap();let digest_identity=stable_file_identity(&candidate).unwrap();let mut changed=bytes.clone();changed[0]^=1;std::fs::write(&candidate,&changed).unwrap();assert_eq!(stable_file_identity(&candidate).unwrap(),digest_identity);assert_eq!(super::clean_stdout(&digest,&args,&env).unwrap_err(),"toolchain_discovery: digest changed before launch");assert!(!sentinel.exists());std::fs::write(&candidate,&bytes).unwrap();let size=ExecutableBinding::authenticate(candidate.clone(),"toolchain_discovery").unwrap();changed=bytes.clone();changed.push(0);std::fs::write(&candidate,&changed).unwrap();assert_eq!(super::clean_stdout(&size,&args,&env).unwrap_err(),"toolchain_discovery: size changed before launch");assert!(!sentinel.exists());std::fs::remove_file(&candidate).unwrap();std::fs::hard_link(&other,&candidate).unwrap();assert_eq!(super::clean_stdout(&size,&args,&env).unwrap_err(),"toolchain_discovery: path is not an ordinary file");assert!(!sentinel.exists());std::fs::remove_file(&candidate).unwrap();let deleted=ExecutableBinding::authenticate(std::fs::copy(&other,&candidate).map(|_|candidate.clone()).unwrap(),"toolchain_discovery").unwrap();std::fs::remove_file(&candidate).unwrap();assert!(super::clean_stdout(&deleted,&args,&env).unwrap_err().starts_with("toolchain_discovery:"));assert!(!sentinel.exists());let malformed=ExecutableBinding::authenticate(std::fs::copy(&other,&candidate).map(|_|candidate.clone()).unwrap(),"toolchain_discovery").unwrap();std::fs::remove_file(&candidate).unwrap();std::fs::create_dir(&candidate).unwrap();assert_eq!(super::clean_stdout(&malformed,&args,&env).unwrap_err(),"toolchain_discovery: path is not an ordinary file");assert!(!sentinel.exists());drop(owned);}
}
