use crate::version;

pub const RUNTIME_PROFILES_SCHEMA: &str = "hum.runtime_profiles.v0";
pub const RUNTIME_PROFILE_SCHEMA: &str = "hum.runtime_profile.v0";

struct RuntimeProfile {
    id: &'static str,
    source_spelling: &'static str,
    status: &'static str,
    purpose: &'static str,
    forbids_by_default: &'static [&'static str],
    requires_evidence: &'static [&'static str],
    allowed_capability_families: &'static [&'static str],
    denied_capability_families: &'static [&'static str],
}

const RUNTIME_PROFILES: &[RuntimeProfile] = &[
    RuntimeProfile {
        id: "normal",
        source_spelling: "normal",
        status: "reserved_v0",
        purpose: "ordinary checked Hum programs with safe defaults and full diagnostics",
        forbids_by_default: &[
            "undeclared unsafe",
            "hidden foreign calls",
            "hidden effects",
        ],
        requires_evidence: &["diagnostics", "semantic graph", "declared effects"],
        allowed_capability_families: &[
            "os.filesystem",
            "os.clock",
            "os.random",
            "os.process",
            "os.network",
        ],
        denied_capability_families: &[],
    },
    RuntimeProfile {
        id: "containerized_service",
        source_spelling: "containerized service",
        status: "reserved_v0",
        purpose: "services intended for Docker, OCI runtimes, Kubernetes, or similar schedulers",
        forbids_by_default: &[
            "privileged containers",
            "host filesystem mounts",
            "host network",
            "hidden listening ports",
            "undeclared environment secret reads",
        ],
        requires_evidence: &[
            "declared ports and protocols",
            "health and readiness behavior",
            "cpu and memory budgets",
            "filesystem mount policy",
            "sbom and provenance evidence",
        ],
        allowed_capability_families: &[
            "os.filesystem",
            "os.clock",
            "os.random",
            "os.process",
            "os.network",
            "artifact.release",
        ],
        denied_capability_families: &["sandbox.host"],
    },
    RuntimeProfile {
        id: "agent_tool_sandbox",
        source_spelling: "agent tool sandbox",
        status: "reserved_v0",
        purpose: "MCP servers, agent-callable CLIs, IDE tools, CI repair tools, and codegen helpers",
        forbids_by_default: &[
            "token passthrough",
            "raw shell command strings",
            "hidden network access",
            "hidden repo mutation",
            "trusting tool descriptions as proof",
        ],
        requires_evidence: &[
            "exact input schema",
            "exact output schema",
            "declared read and write capabilities",
            "dry-run behavior for mutation",
            "secret redaction policy",
        ],
        allowed_capability_families: &["os.filesystem", "os.clock", "os.random", "sandbox.host"],
        denied_capability_families: &["os.network", "os.process"],
    },
    RuntimeProfile {
        id: "footprint_constrained",
        source_spelling: "footprint constrained",
        status: "reserved_v0",
        purpose: "small tools, embedded runtimes, bootstraps, demos, rescue tools, and plugins",
        forbids_by_default: &[
            "hidden runtime services",
            "hidden dynamic code loading",
            "hidden background threads",
            "hidden network or cloud dependency",
            "unbudgeted binary-size growth",
        ],
        requires_evidence: &[
            "binary-size budget",
            "startup-time budget",
            "memory floor",
            "dependency count",
            "deterministic artifact policy",
        ],
        allowed_capability_families: &["target.layout", "target.memory", "os.clock"],
        denied_capability_families: &["os.network", "os.process"],
    },
    RuntimeProfile {
        id: "embedded_no_heap",
        source_spelling: "embedded no heap",
        status: "reserved_v0",
        purpose: "microcontrollers, firmware, drivers, and constrained devices without heap allocation",
        forbids_by_default: &[
            "hidden heap allocation",
            "runtime allocation",
            "unbounded recursion",
            "implicit large stack objects",
        ],
        requires_evidence: &[
            "stack estimate",
            "static memory map",
            "target description",
            "no-heap stdlib subset",
        ],
        allowed_capability_families: &["target.layout", "target.cpu", "target.memory"],
        denied_capability_families: &["os.filesystem", "os.process", "os.network", "sandbox.host"],
    },
    RuntimeProfile {
        id: "hard_realtime",
        source_spelling: "hard realtime",
        status: "reserved_v0",
        purpose: "control loops, audio engines, robotics, medical control, and strict-deadline jobs",
        forbids_by_default: &[
            "unbounded allocation",
            "blocking io",
            "unbounded locks",
            "hidden background reclamation",
            "runtime code generation",
        ],
        requires_evidence: &[
            "deadline behavior",
            "wcet estimate or measured bound",
            "stack bound",
            "scheduling policy",
            "watchdog or fail-safe behavior",
        ],
        allowed_capability_families: &["target.layout", "target.cpu", "target.memory", "os.clock"],
        denied_capability_families: &["os.filesystem", "os.process", "os.network"],
    },
    RuntimeProfile {
        id: "safety_critical",
        source_spelling: "safety critical",
        status: "reserved_v0",
        purpose: "high-assurance systems where failure policy, traceability, and evidence dominate",
        forbids_by_default: &[
            "panic unwind across critical boundaries",
            "ignored Result",
            "unsafe without review packet",
            "foreign without abi contract",
            "implicit numeric narrowing",
        ],
        requires_evidence: &[
            "traceability graph",
            "deterministic build manifest",
            "dependency evidence packet",
            "risk-control links",
            "test or proof evidence",
        ],
        allowed_capability_families: &[
            "target.layout",
            "target.cpu",
            "target.memory",
            "artifact.release",
        ],
        denied_capability_families: &["os.network", "os.process"],
    },
    RuntimeProfile {
        id: "certified_toolchain",
        source_spelling: "certified toolchain",
        status: "reserved_v0",
        purpose: "builds where the compiler, stdlib subset, and tools are part of audited evidence",
        forbids_by_default: &[
            "floating compiler version",
            "floating stdlib subset",
            "unrecorded tool upgrade",
            "unsigned release artifact",
        ],
        requires_evidence: &[
            "compiler version lock",
            "stdlib subset lock",
            "target lock",
            "artifact hashes",
            "tool qualification packet",
        ],
        allowed_capability_families: &[
            "artifact.release",
            "target.layout",
            "target.cpu",
            "target.memory",
        ],
        denied_capability_families: &["os.network"],
    },
];

const RULES: &[&str] = &[
    "Profiles are policy contracts, not build modes, until enforcement exists.",
    "Strict profiles must be smaller and more boring than normal Hum.",
    "Profile capability denial overrides target availability.",
    "Unknown or conflicting profile rules fail closed.",
    "No profile can certify safety without source facts, target facts, toolchain identity, and evidence.",
];

const NON_GOALS_V0: &[&str] = &[
    "no profile syntax enforcement",
    "no stdlib narrowing",
    "no executable runtime behavior",
    "no certification claim",
    "no target selection",
    "no host probing",
    "no performance or footprint measurement",
];

pub fn runtime_profiles_text() -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Hum runtime profiles ({RUNTIME_PROFILES_SCHEMA})\ntool: hum {} {}\nmilestone: {}\nprofile_schema: {RUNTIME_PROFILE_SCHEMA}\nmode: contract_only_no_profile_enforcement\n",
        version::HUM_VERSION,
        version::HUM_STATUS,
        version::HUM_MILESTONE
    ));
    out.push_str("profiles:\n");
    for profile in RUNTIME_PROFILES {
        out.push_str(&format!(
            "  {} [{}]: {} ({})\n",
            profile.id, profile.status, profile.source_spelling, profile.purpose
        ));
    }
    out.push_str("rules:\n");
    for rule in RULES {
        out.push_str(&format!("  {rule}\n"));
    }
    out.push_str("non_goals_v0:\n");
    for non_goal in NON_GOALS_V0 {
        out.push_str(&format!("  {non_goal}\n"));
    }
    out
}

pub fn runtime_profiles_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", RUNTIME_PROFILES_SCHEMA, true);
    push_string_field(&mut out, 2, "profile_schema", RUNTIME_PROFILE_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(
        &mut out,
        2,
        "mode",
        "contract_only_no_profile_enforcement",
        true,
    );
    push_profiles(&mut out, 2, true);
    push_string_array(&mut out, 2, "rules", RULES, true);
    push_string_array(&mut out, 2, "non_goals_v0", NON_GOALS_V0, false);
    out.push_str("}\n");
    out
}

fn push_profiles(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"profiles\": [\n");
    for (index, profile) in RUNTIME_PROFILES.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "schema", RUNTIME_PROFILE_SCHEMA, true);
        push_string_field(out, indent + 4, "id", profile.id, true);
        push_string_field(
            out,
            indent + 4,
            "source_spelling",
            profile.source_spelling,
            true,
        );
        push_string_field(out, indent + 4, "status", profile.status, true);
        push_string_field(out, indent + 4, "purpose", profile.purpose, true);
        push_string_array(
            out,
            indent + 4,
            "forbids_by_default",
            profile.forbids_by_default,
            true,
        );
        push_string_array(
            out,
            indent + 4,
            "requires_evidence",
            profile.requires_evidence,
            true,
        );
        push_string_array(
            out,
            indent + 4,
            "allowed_capability_families",
            profile.allowed_capability_families,
            true,
        );
        push_string_array(
            out,
            indent + 4,
            "denied_capability_families",
            profile.denied_capability_families,
            false,
        );
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_string_array(out: &mut String, indent: usize, key: &str, values: &[&str], comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [\n");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        push_json_string(out, value);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_string_field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_json_string(out, value);
    push_comma_newline(out, comma);
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn push_comma_newline(out: &mut String, comma: bool) {
    if comma {
        out.push(',');
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::{runtime_profiles_json, runtime_profiles_text};

    #[test]
    fn text_runtime_profiles_lists_strict_profiles_and_non_goals() {
        let text = runtime_profiles_text();

        assert!(text.contains("Hum runtime profiles (hum.runtime_profiles.v0)"));
        assert!(text.contains("profile_schema: hum.runtime_profile.v0"));
        assert!(text.contains("mode: contract_only_no_profile_enforcement"));
        assert!(text.contains("agent_tool_sandbox [reserved_v0]"));
        assert!(text.contains("footprint_constrained [reserved_v0]"));
        assert!(text.contains("no profile syntax enforcement"));
    }

    #[test]
    fn json_runtime_profiles_is_machine_readable() {
        let json = runtime_profiles_json();

        assert!(json.contains("\"schema\": \"hum.runtime_profiles.v0\""));
        assert!(json.contains("\"profile_schema\": \"hum.runtime_profile.v0\""));
        assert!(json.contains("\"mode\": \"contract_only_no_profile_enforcement\""));
        assert!(json.contains("\"id\": \"agent_tool_sandbox\""));
        assert!(json.contains("\"id\": \"hard_realtime\""));
        assert!(json.contains("\"denied_capability_families\""));
        assert!(json.contains("\"os.network\""));
        assert!(json.contains("\"no certification claim\""));
    }
}
