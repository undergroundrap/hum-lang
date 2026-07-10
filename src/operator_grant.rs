use std::collections::BTreeSet;

pub(crate) const STDOUT_WRITE: &str = "stdout.write";
pub(crate) const CLOCK_REPLAY: &str = "clock.replay";

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
}

impl OperatorGrantPolicy {
    pub(crate) fn allow(&mut self, text: &str) -> Result<(), String> {
        self.allows.insert(parse_exact_grant(text, "--allow")?);
        Ok(())
    }

    pub(crate) fn deny(&mut self, text: &str) -> Result<(), String> {
        self.denies.insert(parse_exact_grant(text, "--deny")?);
        Ok(())
    }

    pub(crate) fn stdout_write_decision(&self) -> GrantDecision {
        self.decision(STDOUT_WRITE)
    }

    pub(crate) fn clock_replay_decision(&self) -> GrantDecision {
        self.decision(CLOCK_REPLAY)
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

fn parse_exact_grant(text: &str, flag: &str) -> Result<&'static str, String> {
    for capability in [STDOUT_WRITE, CLOCK_REPLAY] {
        if text == capability {
            return Ok(capability);
        }
        if text.starts_with(capability) {
            return Err(format!(
                "`hum run {flag}` grant `{text}` carries a forbidden payload; Sessions Z-AA accept exact `{STDOUT_WRITE}` or `{CLOCK_REPLAY}` only"
            ));
        }
    }
    Err(format!(
        "`hum run {flag}` does not recognize `{text}`; Sessions Z-AA accept exact `{STDOUT_WRITE}` or `{CLOCK_REPLAY}` only"
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
                .contains("does not recognize")
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
}
