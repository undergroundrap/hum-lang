use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[cfg(windows)]
use crate::shell::{ExecutableBinding, fixed_system_command, known_folder, same_ordinary_file};
use crate::shell::{ShellEnvironment, ordinary_file};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceProfile {
    Focused,
    Status,
    Full,
    Exhaustive,
}

impl EvidenceProfile {
    pub fn name(self) -> &'static str {
        match self {
            Self::Focused => "focused",
            Self::Status => "status",
            Self::Full => "full",
            Self::Exhaustive => "exhaustive",
        }
    }
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "focused" => Ok(Self::Focused),
            "status" => Ok(Self::Status),
            "full" => Ok(Self::Full),
            "exhaustive" => Ok(Self::Exhaustive),
            _ => Err(format!("unknown evidence profile `{value}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageInput {
    Subject(String),
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Evidence(EvidenceProfile),
    EvidenceBound(EvidenceProfile, PathBuf),
    EvidenceSummarizeBound(PathBuf, PathBuf),
    CommitMessage(MessageInput),
    CandidateIdentity(PathBuf),
    CleanupVerify,
    WorkOrderStatusFacts {
        input: PathBuf,
        base_sha256: String,
        status_body: PathBuf,
        gate_body: PathBuf,
        output: PathBuf,
    },
}

impl Command {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let values: Vec<String> = args.into_iter().collect();
        match values.as_slice() {
            [group, profile, output_flag, output, pwsh_flag, pwsh] if group == "evidence" && profile == "summarize" && output_flag == "--output" && pwsh_flag == "--pwsh" => Ok(Self::EvidenceSummarizeBound(output.into(), pwsh.into())),
            [group, profile] if group == "evidence" && profile == "status" => Ok(Self::Evidence(EvidenceProfile::Status)),
            [group, profile] if group == "evidence" => { EvidenceProfile::parse(profile)?; Err("explicit --pwsh binding is required".into()) },
            [group, profile, flag, pwsh] if group == "evidence" && flag == "--pwsh" => {
                let profile = EvidenceProfile::parse(profile)?;
                if profile == EvidenceProfile::Status { return Err("status evidence does not execute PowerShell".into()); }
                Ok(Self::EvidenceBound(profile, pwsh.into()))
            }
            [group, action, flag, subject] if group == "commit-message" && action == "check" && flag == "--subject" =>
                Ok(Self::CommitMessage(MessageInput::Subject(subject.clone()))),
            [group, action, flag, path] if group == "commit-message" && action == "check" && flag == "--file" =>
                Ok(Self::CommitMessage(MessageInput::File(path.into()))),
            [group, action] if group == "candidate" && action == "identity" =>
                Ok(Self::CandidateIdentity(PathBuf::from("."))),
            [group, action, flag, path] if group == "candidate" && action == "identity" && flag == "--repository" =>
                Ok(Self::CandidateIdentity(path.into())),
            [group, action] if group == "cleanup" && action == "verify" => Ok(Self::CleanupVerify),
            [group, action, input_flag, input, base_flag, base_sha256, status_flag, status_body, gate_flag, gate_body, output_flag, output] if group == "workorder" && action == "status-facts" && input_flag == "--input" && base_flag == "--base-sha256" && status_flag == "--status-body-file" && gate_flag == "--gate-body-file" && output_flag == "--output" => Ok(Self::WorkOrderStatusFacts { input: input.into(), base_sha256: base_sha256.clone(), status_body: status_body.into(), gate_body: gate_body.into(), output: output.into() }),
            _ => Err("usage: hum-dev evidence <focused|full|exhaustive> --pwsh ABSOLUTE_PATH | evidence status | evidence summarize --output PATH --pwsh ABSOLUTE_PATH | commit-message check <--subject TEXT|--file PATH> | candidate identity [--repository PATH] | cleanup verify | workorder status-facts --input PATH --base-sha256 HASH --status-body-file PATH --gate-body-file PATH --output PATH".into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyInvocation {
    pub executable: &'static str,
    pub script: &'static str,
    pub arguments: &'static [&'static str],
}

#[rustfmt::skip]
pub fn legacy_invocation(profile: EvidenceProfile) -> LegacyInvocation {
    match profile {
        EvidenceProfile::Focused => LegacyInvocation { executable: "pwsh", script: "tools/check_all.ps1", arguments: &["-EvidenceTier", "Wo25UnitC"] },
        EvidenceProfile::Full => LegacyInvocation { executable: "pwsh", script: "tools/check_all.ps1", arguments: &["-EvidenceTier", "Fast"] },
        EvidenceProfile::Exhaustive => LegacyInvocation { executable: "pwsh", script: "tools/check_all.ps1", arguments: &["-EvidenceTier", "Exhaustive"] },
        EvidenceProfile::Status => LegacyInvocation { executable: "pwsh", script: "tools/check_workorder_status_boundary.ps1", arguments: &[] },
    }
}

#[rustfmt::skip]
pub(crate) fn production_environment(repository: &Path) -> Result<ShellEnvironment, String> {
    let mut env = ShellEnvironment::from_process(repository)?;
    #[cfg(windows)]
    {
        const TOOLS: &str = "INCLUDE,LIB,LIBPATH,VCINSTALLDIR,VCToolsInstallDir,VSCMD_ARG_HOST_ARCH,VSCMD_ARG_TGT_ARCH,WindowsSdkDir,WindowsSDKVersion,PATH";
        const BASE: &str = "SystemRoot,WINDIR,ProgramFiles,ProgramFiles(x86),ProgramData,TEMP,TMP,PATHEXT,PSModulePath";
        for key in TOOLS.split(',') {
            env.0.remove(OsStr::new(key));
        }
        let program_files = known_folder(0x26, "program_files")?;
        env.0.insert("ProgramFiles".into(), program_files.clone().into_os_string());
        let program_files_x86 = known_folder(0x2a, "program_files_x86")?; let program_files_x86_identity = program_files_x86.canonicalize().map_err(|e| format!("program_files_x86: native known folder cannot be canonicalized: {e}"))?;
        if let Some(value) = env.0.get(OsStr::new("ProgramFiles(x86)")) { let ambient = Path::new(value); let metadata = std::fs::symlink_metadata(ambient).map_err(|_| "program_files_x86: ambient binding differs from native known folder".to_string())?; let canonical = ambient.canonicalize().map_err(|_| "program_files_x86: ambient binding differs from native known folder".to_string())?; if !ambient.is_absolute() || !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 || !canonical.as_os_str().eq_ignore_ascii_case(program_files_x86_identity.as_os_str()) { return Err("program_files_x86: ambient binding differs from native known folder".into()); } }
        env.0.insert("ProgramFiles(x86)".into(), program_files_x86.clone().into_os_string());
        let cargo_bootstrap = Path::new(env!("CARGO"));
        let bin = cargo_bootstrap.parent().filter(|p| p.file_name() == Some(OsStr::new("bin"))).ok_or("cargo_layout: executable is not in bin")?;
        let owner = bin.parent().ok_or("cargo_layout: bin has no owner")?;
        let direct = owner.parent().filter(|p| p.file_name() == Some(OsStr::new("toolchains")));
        let rustup_home = match direct { Some(toolchains) => toolchains.parent().map(Path::to_path_buf), None if owner.file_name() == Some(OsStr::new(".cargo")) => owner.parent().map(|p| p.join(".rustup")), None => None }.ok_or("cargo_layout: unknown or ambiguous executable layout")?.canonicalize().map_err(|e| format!("rustup_root: {e}"))?;
        let cargo_home = rustup_home.parent().ok_or("cargo_root: rustup root has no owner")?.join(".cargo").canonicalize().map_err(|e| format!("cargo_root: {e}"))?;
        use std::os::windows::fs::MetadataExt;
        for (label, root) in [("cargo_root", &cargo_home), ("rustup_root", &rustup_home)] {
            let metadata = std::fs::symlink_metadata(root).map_err(|e| format!("{label}: {e}"))?;
            if !root.is_absolute() || !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 {
                return Err(format!("{label}: not an ordinary absolute directory"));
            }
        }
        let settings = std::fs::read_to_string(rustup_home.join("settings.toml")).map_err(|e| format!("rust_toolchain: {e}"))?;
        let names = settings.lines().filter_map(|line| line.strip_prefix("default_toolchain = \"").and_then(|v| v.strip_suffix('"'))).collect::<Vec<_>>();
        if names.len() != 1 || names[0].bytes().any(|b| !b.is_ascii_alphanumeric() && !matches!(b, b'.' | b'_' | b'-')) { return Err("rust_toolchain: default identity is missing or malformed".into()); }
        let tool_bin = rustup_home.join("toolchains").join(names[0]).join("bin");
        let cargo = tool_bin.join("cargo.exe"); let rustc = tool_bin.join("rustc.exe");
        ordinary_file(&cargo, "cargo_executable")?; ordinary_file(&rustc, "rustc_executable")?;
        env.0.insert("CARGO_HOME".into(), cargo_home.into_os_string());
        env.0.insert("RUSTUP_HOME".into(), rustup_home.into_os_string());
        env.0.insert("RUSTUP_TOOLCHAIN".into(), names[0].into());
        let vswhere = program_files_x86.join("Microsoft Visual Studio/Installer/vswhere.exe");
        let vswhere = ExecutableBinding::authenticate(vswhere, "toolchain_discovery")?;
        let mut child = BASE.split(',').map(|key| Ok((key, env.get(key)?.to_owned()))).collect::<Result<Vec<_>, String>>()?;
        let path = std::env::join_paths([tool_bin.clone(), cargo.parent().unwrap().to_owned(), program_files.join("Git/cmd"), fixed_system_command()?.parent().unwrap().to_owned()]).map_err(|e| format!("toolchain_bootstrap: {e}"))?;
        child.push(("PATH", path));
        let args = "-latest|-products|*|-requires|Microsoft.VisualStudio.Component.VC.Tools.x86.x64|-property|installationPath".split('|').collect::<Vec<_>>();
        let text = crate::clean_stdout(&vswhere, &args, &child)?;
        let installs = text.lines().filter(|line| !line.is_empty()).collect::<Vec<_>>();
        if installs.len() != 1 || !Path::new(installs[0]).is_absolute() {
            return Err("toolchain_discovery: expected exactly one absolute installation".into());
        }
        let producer = Path::new(installs[0]).join("Common7/Tools/VsDevCmd.bat");
        let command = fixed_system_command()?;
        ordinary_file(&producer, "toolchain_producer")?;
        child.push(("HUM_TOOLCHAIN_PRODUCER", producer.as_os_str().to_owned()));
        let script = "call \"%HUM_TOOLCHAIN_PRODUCER%\" -arch=x64 -host_arch=x64 >nul && set";
        let text = crate::clean_cmd_stdout(&command, script, &child, "toolchain_producer")?;
        let mut rows = std::collections::BTreeMap::new();
        for (key, value) in text.lines().filter_map(|line| line.split_once('=')) {
            if rows.insert(key.to_ascii_lowercase(), (key, value)).is_some() {
                return Err(format!("toolchain_producer: duplicate {key}"));
            }
        }
        for key in TOOLS.split(',') {
            let (actual, value) = rows.get(&key.to_ascii_lowercase()).ok_or_else(|| format!("windows_toolchain: missing {key}"))?;
            if *actual != key || value.is_empty() {
                return Err(format!("windows_toolchain: malformed {key}"));
            }
            env.0.insert(key.into(), (*value).into());
        }
        let effective = std::env::split_paths(env.get("PATH")?).map(|p| p.join("cargo.exe")).find(|p| p.is_file()).ok_or("cargo_executable: unreachable")?;
        if !same_ordinary_file(&cargo, &effective)? { return Err("cargo_executable: substituted effective Cargo".into()); }
    }
    env.authenticate(repository)?;
    Ok(env)
}
#[cfg(test)]
mod tests {
    use super::{Command, EvidenceProfile, LegacyInvocation, legacy_invocation};

    #[test]
    fn evidence_profiles_are_typed_and_fail_closed() {
        for (name, expected) in [
            ("focused", EvidenceProfile::Focused),
            ("status", EvidenceProfile::Status),
            ("full", EvidenceProfile::Full),
            ("exhaustive", EvidenceProfile::Exhaustive),
        ] {
            let parsed = Command::parse(["evidence".into(), name.into()]);
            if expected == EvidenceProfile::Status {
                assert_eq!(parsed, Ok(Command::Evidence(expected)));
            } else {
                assert!(parsed.is_err());
            }
        }
        let summarize =
            ["evidence", "summarize", "--output", "o", "--pwsh", "p"].map(str::to_owned);
        let expected = Command::EvidenceSummarizeBound("o".into(), "p".into());
        assert_eq!(Command::parse(summarize), Ok(expected));
        for invalid in ["", "fast", "Focused", "unknown"] {
            assert!(
                Command::parse(["evidence".into(), invalid.into()])
                    .unwrap_err()
                    .contains("unknown evidence profile")
            );
        }
        for (profile, executable, script, arguments) in [
            (
                EvidenceProfile::Focused,
                "pwsh",
                "tools/check_all.ps1",
                &["-EvidenceTier", "Wo25UnitC"][..],
            ),
            (
                EvidenceProfile::Full,
                "pwsh",
                "tools/check_all.ps1",
                &["-EvidenceTier", "Fast"][..],
            ),
            (
                EvidenceProfile::Exhaustive,
                "pwsh",
                "tools/check_all.ps1",
                &["-EvidenceTier", "Exhaustive"][..],
            ),
            (
                EvidenceProfile::Status,
                "pwsh",
                "tools/check_workorder_status_boundary.ps1",
                &[][..],
            ),
        ] {
            assert_eq!(
                legacy_invocation(profile),
                LegacyInvocation {
                    executable,
                    script,
                    arguments
                }
            );
        }
        assert!(
            Command::parse([
                "commit-message".into(),
                "check".into(),
                "--unknown".into(),
                "x".into()
            ])
            .is_err()
        );
    }
}
