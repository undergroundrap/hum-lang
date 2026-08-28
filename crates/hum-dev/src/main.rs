mod cleanup;
mod command;
mod commit_message;
mod identity;
mod summary;

use std::{
    ffi::OsStr,
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
    ProcessCommand::new(executable)
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .args(["-NoLogo", "-NoProfile", "-File", invocation.script])
        .args(invocation.arguments)
        .output()
        .map_err(|error| format!("legacy adapter launch failed: {error}"))
}

#[rustfmt::skip]
fn run_legacy(profile: EvidenceProfile) -> Result<i32, String> {
    let invocation = legacy_invocation(profile);
    let output = launch_legacy(OsStr::new(invocation.executable), &invocation)?;
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
        Command::Evidence(profile) => run_legacy(profile),
        Command::EvidenceSummarize => {
            let bytes = summary::summarize_without_authenticated_records()?;
            io::stdout()
                .write_all(&bytes)
                .map_err(|error| error.to_string())?;
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
        Command::WorkOrderStatusFacts => {
            Err("workorder status-facts is reserved for WO25 Unit B".into())
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
        let mappings = [(EvidenceProfile::Focused, "tools/check_all.ps1", &["-EvidenceTier", "Fast"][..]), (EvidenceProfile::Full, "tools/check_all.ps1", &["-EvidenceTier", "Fast"][..]), (EvidenceProfile::Exhaustive, "tools/check_all.ps1", &["-EvidenceTier", "Exhaustive"][..]), (EvidenceProfile::Status, "tools/check_workorder_status_boundary.ps1", &[][..])];
        for (profile, script, arguments) in mappings { assert_eq!(legacy_invocation(profile), LegacyInvocation { executable: "pwsh", script, arguments }); }
        let production = include_str!("main.rs").split_once("#[cfg(test)]").unwrap().0;
        assert!(!production.contains(concat!("HUM_DEV_LEGACY_", "EQUIVALENCE_PROBE")));
        assert!(!production.contains("var_os("));
        let probe = concat!("HUM_DEV_LEGACY_", "EQUIVALENCE_PROBE");
        if std::env::var_os(probe).is_some() { for (profile, script, arguments) in mappings { assert_eq!(legacy_invocation(profile), LegacyInvocation { executable: "pwsh", script, arguments }); } return; }
        let poisoned = Command::new(std::env::current_exe().unwrap()).args(["--exact", "cli::legacy_equivalence_preserves_exit_stages_and_stream_hashes", "--nocapture"]).env(probe, "1").output().unwrap();
        assert!(poisoned.status.success(), "{}", String::from_utf8_lossy(&poisoned.stderr));
        let executable = pwsh(OsStr::new("pwsh"));
        let injected = LegacyInvocation { executable: "pwsh", script: "tools/check_alpha_claims.ps1", arguments: &[] };
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
}
