use crate::version;

pub const STATE_MODEL_SCHEMA: &str = "hum.state_model.v0";
pub const STATE_PERMISSION_SCHEMA: &str = "hum.state_permission.v0";

struct StateKind {
    id: &'static str,
    status: &'static str,
    surface: &'static str,
    role: &'static str,
    default_permission: &'static str,
    profile_pressure: &'static str,
}

struct StatePermission {
    id: &'static str,
    status: &'static str,
    meaning: &'static str,
    surface: &'static str,
    rejects: &'static [&'static str],
}

struct StateGate {
    id: &'static str,
    status: &'static str,
    requirement: &'static str,
}

const STATE_KINDS: &[StateKind] = &[
    StateKind {
        id: "immutable_value",
        status: "reference",
        surface: "let",
        role: "ordinary data that can be read freely and never changed in place",
        default_permission: "read",
        profile_pressure: "all profiles prefer this unless ownership, layout, or allocation evidence justifies mutation",
    },
    StateKind {
        id: "mutable_local",
        status: "current_preview",
        surface: "change plus set",
        role: "task-local state with explicit mutation permission",
        default_permission: "exclusive_change",
        profile_pressure: "strict profiles require bounded lifetime, visible allocation, and no hidden sharing",
    },
    StateKind {
        id: "place",
        status: "reference",
        surface: "name, field, checked index, future store entry",
        role: "readable or writable location addressed by Core Hum",
        default_permission: "read unless declared mutable",
        profile_pressure: "backend lowering must preserve aliasing and source-span facts for places",
    },
    StateKind {
        id: "store",
        status: "current_parse",
        surface: "store",
        role: "named persistent or shared state with purpose, type intent, and policy sections",
        default_permission: "read_only_until_listed_under_changes",
        profile_pressure: "strict profiles require explicit ownership, authority, durability, and concurrency policy",
    },
    StateKind {
        id: "linear_resource",
        status: "planned",
        surface: "future owned resource type",
        role: "file handles, sockets, locks, transactions, buffers, capabilities, and foreign resources that must be consumed exactly once or closed",
        default_permission: "owned_consume",
        profile_pressure: "agent tools, safety-critical code, and unsafe wrappers need exactly-once evidence",
    },
    StateKind {
        id: "shared_state",
        status: "planned",
        surface: "future shared safe forms",
        role: "state reachable from more than one task, thread, actor, or callback",
        default_permission: "forbidden_until_checked",
        profile_pressure: "requires explicit synchronization, replay, ownership, and failure policy before stable use",
    },
    StateKind {
        id: "external_state",
        status: "reference",
        surface: "uses, changes, targets, profiles",
        role: "filesystem, process, network, clock, randomness, device, environment, and host authority",
        default_permission: "forbidden_until_declared",
        profile_pressure: "offline and deterministic profiles deny hidden external state even when target facts say available",
    },
];

const PERMISSIONS: &[StatePermission] = &[
    StatePermission {
        id: "read",
        status: "reference",
        meaning: "observe without mutation, ownership transfer, or hidden IO",
        surface: "uses or local immutable access",
        rejects: &["hidden mutation", "hidden allocation", "hidden authority"],
    },
    StatePermission {
        id: "own",
        status: "planned",
        meaning: "hold responsibility for a value or resource, including drop or transfer",
        surface: "future owned value facts",
        rejects: &[
            "use after move",
            "double close",
            "untracked destructor effects",
        ],
    },
    StatePermission {
        id: "borrow",
        status: "planned",
        meaning: "temporary shared read access without ownership transfer",
        surface: "future borrow facts",
        rejects: &["mutation through shared read", "escaping borrowed value"],
    },
    StatePermission {
        id: "change",
        status: "current_preview",
        meaning: "exclusive permission to mutate a local place or declared external place",
        surface: "change, set, changes",
        rejects: &[
            "undeclared set target",
            "two mutable aliases",
            "hidden store mutation",
        ],
    },
    StatePermission {
        id: "consume",
        status: "planned",
        meaning: "exactly-once use for linear resources and capabilities",
        surface: "future linear resource facts",
        rejects: &["resource leak", "double use", "forgotten rollback or close"],
    },
    StatePermission {
        id: "share",
        status: "deferred",
        meaning: "checked shared access through actor, lock, atomic, region, or runtime policy",
        surface: "future concurrency model",
        rejects: &["data race", "unbounded lock", "unexplained memory ordering"],
    },
];

const GATES: &[StateGate] = &[
    StateGate {
        id: "declared_local_mutation",
        status: "current",
        requirement: "set name = ... requires local change name: ... or a matching changes entry",
    },
    StateGate {
        id: "store_identity",
        status: "current",
        requirement: "store declarations preserve name, type text, sections, and graph identity",
    },
    StateGate {
        id: "effect_permission_check",
        status: "planned",
        requirement: "inferred reads, writes, allocation, time, randomness, IO, unsafe, and foreign calls fit declared sections",
    },
    StateGate {
        id: "ownership_check",
        status: "planned",
        requirement: "moves, drops, and ownership transfers are checked before Hum IR lowering",
    },
    StateGate {
        id: "borrow_check",
        status: "planned",
        requirement: "shared borrows and exclusive changing borrows cannot overlap unsafely",
    },
    StateGate {
        id: "linear_resource_check",
        status: "planned",
        requirement: "linear resources are consumed, closed, committed, rolled back, or transferred exactly once",
    },
    StateGate {
        id: "concurrency_state_check",
        status: "deferred",
        requirement: "threads, actors, locks, atomics, callbacks, and async tasks expose state ownership and memory-order facts",
    },
];

const RULES: &[&str] = &[
    "Immutable values are the default paved road.",
    "Mutation requires source-visible permission.",
    "A store is named state with policy, not a casual global variable.",
    "Shared mutable state is forbidden until a checked sharing form exists.",
    "Linear resources require exactly-once close, commit, rollback, consume, or transfer.",
    "Borrowing is permission to observe or change for a bounded region, not hidden copying.",
    "External state is an effect and a capability, not an implementation detail.",
    "Profiles may make the state model stricter, never looser without evidence.",
];

const NON_GOALS_V0: &[&str] = &[
    "no borrow checker implementation",
    "no lifetime inference",
    "no move checker implementation",
    "no linear type checker",
    "no concurrency or memory-order model",
    "no garbage collector promise",
    "no executable semantics",
    "no optimizer or allocation placement claim",
];

pub fn state_model_text() -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Hum state model ({STATE_MODEL_SCHEMA})\ntool: hum {} {}\nmilestone: {}\npermission_schema: {STATE_PERMISSION_SCHEMA}\nmode: contract_only_partial_declared_mutation_check\n",
        version::HUM_VERSION,
        version::HUM_STATUS,
        version::HUM_MILESTONE
    ));
    out.push_str("state_kinds:\n");
    for kind in STATE_KINDS {
        out.push_str(&format!(
            "  {} [{}]: {}; default_permission={}; surface={}\n",
            kind.id, kind.status, kind.role, kind.default_permission, kind.surface
        ));
    }
    out.push_str("permissions:\n");
    for permission in PERMISSIONS {
        out.push_str(&format!(
            "  {} [{}]: {}; surface={}\n",
            permission.id, permission.status, permission.meaning, permission.surface
        ));
    }
    out.push_str("gates:\n");
    for gate in GATES {
        out.push_str(&format!(
            "  {} [{}]: {}\n",
            gate.id, gate.status, gate.requirement
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

pub fn state_model_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", STATE_MODEL_SCHEMA, true);
    push_string_field(
        &mut out,
        2,
        "permission_schema",
        STATE_PERMISSION_SCHEMA,
        true,
    );
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(
        &mut out,
        2,
        "mode",
        "contract_only_partial_declared_mutation_check",
        true,
    );
    push_state_kinds(&mut out, 2, true);
    push_permissions(&mut out, 2, true);
    push_gates(&mut out, 2, true);
    push_string_array(&mut out, 2, "rules", RULES, true);
    push_string_array(&mut out, 2, "non_goals_v0", NON_GOALS_V0, false);
    out.push_str("}\n");
    out
}

fn push_state_kinds(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"state_kinds\": [\n");
    for (index, kind) in STATE_KINDS.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "id", kind.id, true);
        push_string_field(out, indent + 4, "status", kind.status, true);
        push_string_field(out, indent + 4, "surface", kind.surface, true);
        push_string_field(out, indent + 4, "role", kind.role, true);
        push_string_field(
            out,
            indent + 4,
            "default_permission",
            kind.default_permission,
            true,
        );
        push_string_field(
            out,
            indent + 4,
            "profile_pressure",
            kind.profile_pressure,
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

fn push_permissions(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"permissions\": [\n");
    for (index, permission) in PERMISSIONS.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "schema", STATE_PERMISSION_SCHEMA, true);
        push_string_field(out, indent + 4, "id", permission.id, true);
        push_string_field(out, indent + 4, "status", permission.status, true);
        push_string_field(out, indent + 4, "meaning", permission.meaning, true);
        push_string_field(out, indent + 4, "surface", permission.surface, true);
        push_string_array(out, indent + 4, "rejects", permission.rejects, false);
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_gates(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"gates\": [\n");
    for (index, gate) in GATES.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "id", gate.id, true);
        push_string_field(out, indent + 4, "status", gate.status, true);
        push_string_field(out, indent + 4, "requirement", gate.requirement, false);
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
    use super::{state_model_json, state_model_text};

    #[test]
    fn text_state_model_lists_state_rules_and_non_goals() {
        let text = state_model_text();

        assert!(text.contains("Hum state model (hum.state_model.v0)"));
        assert!(text.contains("permission_schema: hum.state_permission.v0"));
        assert!(text.contains("mode: contract_only_partial_declared_mutation_check"));
        assert!(text.contains("mutable_local [current_preview]"));
        assert!(text.contains("linear_resource [planned]"));
        assert!(text.contains("no borrow checker implementation"));
    }

    #[test]
    fn json_state_model_is_machine_readable() {
        let json = state_model_json();

        assert!(json.contains("\"schema\": \"hum.state_model.v0\""));
        assert!(json.contains("\"permission_schema\": \"hum.state_permission.v0\""));
        assert!(json.contains("\"mode\": \"contract_only_partial_declared_mutation_check\""));
        assert!(json.contains("\"id\": \"immutable_value\""));
        assert!(json.contains("\"id\": \"linear_resource\""));
        assert!(json.contains("\"id\": \"declared_local_mutation\""));
        assert!(json.contains("\"id\": \"borrow_check\""));
        assert!(json.contains("\"no concurrency or memory-order model\""));
    }
}
