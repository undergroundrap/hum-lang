use std::path::PathBuf;

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
    EvidenceSummarize(PathBuf),
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
            [group, profile, flag, output] if group == "evidence" && profile == "summarize" && flag == "--output" => Ok(Self::EvidenceSummarize(output.into())),
            [group, profile] if group == "evidence" => Ok(Self::Evidence(EvidenceProfile::parse(profile)?)),
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
            _ => Err("usage: hum-dev evidence <focused|status|full|exhaustive> | evidence summarize --output PATH | commit-message check <--subject TEXT|--file PATH> | candidate identity [--repository PATH] | cleanup verify | workorder status-facts --input PATH --base-sha256 HASH --status-body-file PATH --gate-body-file PATH --output PATH".into()),
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
        EvidenceProfile::Focused | EvidenceProfile::Full => LegacyInvocation { executable: "pwsh", script: "tools/check_all.ps1", arguments: &["-EvidenceTier", "Fast"] },
        EvidenceProfile::Exhaustive => LegacyInvocation { executable: "pwsh", script: "tools/check_all.ps1", arguments: &["-EvidenceTier", "Exhaustive"] },
        EvidenceProfile::Status => LegacyInvocation { executable: "pwsh", script: "tools/check_workorder_status_boundary.ps1", arguments: &[] },
    }
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
            assert_eq!(
                Command::parse(["evidence".into(), name.into()]),
                Ok(Command::Evidence(expected))
            );
        }
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
                &["-EvidenceTier", "Fast"][..],
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
