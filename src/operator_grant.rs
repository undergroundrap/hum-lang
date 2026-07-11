use std::collections::BTreeSet;
use std::ffi::OsStr;

use crate::native_path::{ValidatedNativePath, strip_ascii_prefix, validate_native_path};

pub(crate) const STDOUT_WRITE: &str = "stdout.write";
pub(crate) const CLOCK_REPLAY: &str = "clock.replay";
pub(crate) const FILES_READ: &str = "files.read";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GrantDecision {
    Allowed,
    DeniedExplicit,
    DeniedDefault,
}

impl GrantDecision {
    pub(crate) fn effective(self) -> &'static str {
        match self {
            Self::Allowed => "allow",
            Self::DeniedExplicit | Self::DeniedDefault => "deny",
        }
    }

    pub(crate) fn reason(self) -> &'static str {
        match self {
            Self::Allowed => "exact_one_run_allow_v0",
            Self::DeniedExplicit => "exact_deny_overrides_allow_v0",
            Self::DeniedDefault => "default_empty_grant_set_v0",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct OperatorGrantPolicy {
    allows: BTreeSet<&'static str>,
    denies: BTreeSet<&'static str>,
    files_read: Option<ValidatedNativePath>,
}

impl OperatorGrantPolicy {
    pub(crate) fn allow(&mut self, text: &str) -> Result<(), String> {
        self.allow_os(OsStr::new(text))
    }

    pub(crate) fn deny(&mut self, text: &str) -> Result<(), String> {
        self.deny_os(OsStr::new(text))
    }

    pub(crate) fn allow_os(&mut self, text: &OsStr) -> Result<(), String> {
        if let Some(payload) = strip_ascii_prefix(text, "files.read=") {
            if payload.is_empty() {
                return Err(
                    "`hum run --allow files.read=<path>` requires one native path payload"
                        .to_string(),
                );
            }
            let validated = validate_native_path(&payload).map_err(|issue| {
                format!(
                    "`hum run --allow files.read=<path>` rejected because {}; reason={}; no host access was attempted",
                    issue.description(),
                    issue.reason()
                )
            })?;
            if let Some(existing) = &self.files_read
                && existing != &validated
            {
                return Err(
                    "`hum run` accepts at most one distinct native `files.read=<path>` grant; exact duplicates are idempotent"
                        .to_string(),
                );
            }
            self.files_read = Some(validated);
            self.allows.insert(FILES_READ);
            return Ok(());
        }
        self.allows
            .insert(parse_exact_capability(text, "--allow", false)?);
        Ok(())
    }

    pub(crate) fn deny_os(&mut self, text: &OsStr) -> Result<(), String> {
        self.denies
            .insert(parse_exact_capability(text, "--deny", true)?);
        Ok(())
    }

    pub(crate) fn stdout_write_decision(&self) -> GrantDecision {
        self.decision(STDOUT_WRITE)
    }

    pub(crate) fn clock_replay_decision(&self) -> GrantDecision {
        self.decision(CLOCK_REPLAY)
    }

    #[cfg(test)]
    pub(crate) fn files_read_decision(&self) -> GrantDecision {
        self.decision(FILES_READ)
    }

    #[cfg(test)]
    pub(crate) fn files_read_grant(&self) -> Option<&ValidatedNativePath> {
        self.files_read.as_ref()
    }

    fn decision(&self, capability: &'static str) -> GrantDecision {
        if self.denies.contains(capability) {
            GrantDecision::DeniedExplicit
        } else if self.allows.contains(capability) {
            GrantDecision::Allowed
        } else {
            GrantDecision::DeniedDefault
        }
    }

    pub(crate) fn allows_stdout_write(&self) -> bool {
        self.allows.contains(STDOUT_WRITE)
    }

    pub(crate) fn denies_stdout_write(&self) -> bool {
        self.denies.contains(STDOUT_WRITE)
    }

    pub(crate) fn allows_clock_replay(&self) -> bool {
        self.allows.contains(CLOCK_REPLAY)
    }

    pub(crate) fn denies_clock_replay(&self) -> bool {
        self.denies.contains(CLOCK_REPLAY)
    }
}

fn parse_exact_capability(
    text: &OsStr,
    flag: &str,
    files_read_deny_allowed: bool,
) -> Result<&'static str, String> {
    for capability in [STDOUT_WRITE, CLOCK_REPLAY, FILES_READ] {
        if text == OsStr::new(capability) && (capability != FILES_READ || files_read_deny_allowed) {
            return Ok(capability);
        }
        if strip_ascii_prefix(text, capability).is_some() {
            return Err(format!(
                "`hum run {flag}` carries a forbidden payload or incomplete grant; Session AB accepts exact `{STDOUT_WRITE}`, `{CLOCK_REPLAY}`, exact deny `{FILES_READ}`, or allow `{FILES_READ}=<native-path>` only"
            ));
        }
    }
    Err(format!(
        "`hum run {flag}` does not recognize this native grant"
    ))
}

#[cfg(test)]
mod tests {
    use super::{GrantDecision, OperatorGrantPolicy};

    #[test]
    fn exact_duplicates_are_idempotent_and_deny_wins() {
        let mut policy = OperatorGrantPolicy::default();
        assert_eq!(policy.stdout_write_decision(), GrantDecision::DeniedDefault);
        policy.allow("stdout.write").expect("allow");
        policy.allow("clock.replay").expect("clock allow");
        policy.allow("stdout.write").expect("duplicate allow");
        assert_eq!(policy.stdout_write_decision(), GrantDecision::Allowed);
        assert_eq!(policy.clock_replay_decision(), GrantDecision::Allowed);
        policy.deny("stdout.write").expect("deny");
        policy.deny("clock.replay").expect("clock deny");
        policy.deny("stdout.write").expect("duplicate deny");
        assert_eq!(
            policy.stdout_write_decision(),
            GrantDecision::DeniedExplicit
        );
        assert_eq!(
            policy.clock_replay_decision(),
            GrantDecision::DeniedExplicit
        );
    }

    #[test]
    fn unknown_and_payload_grants_fail_closed() {
        let mut policy = OperatorGrantPolicy::default();
        assert!(
            policy
                .allow("files.read")
                .unwrap_err()
                .contains("incomplete")
        );
        assert!(
            policy
                .allow("stdout.write=console")
                .unwrap_err()
                .contains("forbidden payload")
        );
        assert!(
            policy
                .allow("clock.replay=host")
                .unwrap_err()
                .contains("forbidden payload")
        );
        assert!(
            policy
                .deny("stdout.write:all")
                .unwrap_err()
                .contains("forbidden payload")
        );
    }

    #[cfg(windows)]
    #[test]
    fn exact_native_file_grant_is_single_idempotent_and_deny_wins() {
        use std::ffi::OsString;
        use std::os::windows::ffi::{OsStrExt, OsStringExt};

        let grant = |drive: char, suffix: &str| {
            OsString::from(format!("files.read={drive}:{}{}", char::from(92), suffix))
        };

        let mut policy = OperatorGrantPolicy::default();
        let first = grant('C', "hum-session-ab\\missing.bin");
        policy.allow_os(&first).expect("native grant");
        policy.allow_os(&first).expect("duplicate native grant");
        assert_eq!(policy.files_read_decision(), GrantDecision::Allowed);
        assert_eq!(
            policy.files_read_grant().expect("grant").locality(),
            "locality_unclassified"
        );
        assert!(
            policy
                .allow_os(&grant('D', "different\\missing.bin"))
                .unwrap_err()
                .contains("at most one distinct")
        );
        policy.deny("files.read").expect("deny");
        assert_eq!(policy.files_read_decision(), GrantDecision::DeniedExplicit);

        let mut native_units = "files.read=C:".encode_utf16().collect::<Vec<_>>();
        native_units.push(u16::from(b'\\'));
        native_units.extend("opaque".encode_utf16());
        native_units.push(u16::from(b'\\'));
        native_units.push(0xd800);
        let native_grant = OsString::from_wide(&native_units);
        let mut native_policy = OperatorGrantPolicy::default();
        native_policy
            .allow_os(&native_grant)
            .expect("non-String native grant");
        let path_offset = "files.read=".encode_utf16().count();
        assert_eq!(
            native_policy
                .files_read_grant()
                .expect("native grant")
                .as_os_str()
                .encode_wide()
                .collect::<Vec<_>>(),
            native_units[path_offset..]
        );
    }
}
