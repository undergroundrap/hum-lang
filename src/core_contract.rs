use crate::core_body;
use crate::ir_contract;
use crate::json;
use crate::version;

pub const CORE_CONTRACT_SCHEMA: &str = "hum.core_contract.v0";

struct CoreCatalog {
    name: &'static str,
    status: &'static str,
    role: &'static str,
    items: &'static [&'static str],
}

struct ContractLowering {
    section: &'static str,
    lowers_to: &'static str,
    blame: &'static str,
}

struct AcceptanceGate {
    id: &'static str,
    status: &'static str,
    requirement: &'static str,
}

const CORE_CATALOGS: &[CoreCatalog] = &[
    CoreCatalog {
        name: "values",
        status: "design",
        role: "runtime values the first executable core must model",
        items: &[
            "unit",
            "bool",
            "int",
            "uint",
            "text",
            "bytes",
            "record_value",
            "variant_value",
            "maybe_value",
            "result_value",
        ],
    },
    CoreCatalog {
        name: "types",
        status: "design",
        role: "type forms the first checker must understand before Hum IR",
        items: &[
            "Unit",
            "Bool",
            "Int",
            "UInt",
            "Text",
            "Bytes",
            "record",
            "variant",
            "maybe T",
            "Result<T, E>",
        ],
    },
    CoreCatalog {
        name: "places",
        status: "design",
        role: "readable or writable locations with explicit mutation permission",
        items: &[
            "local",
            "mutable_local",
            "record_field",
            "checked_index",
            "store_entry_deferred",
        ],
    },
    CoreCatalog {
        name: "expressions",
        status: "design",
        role: "side-effect-controlled expressions that can be typed and lowered",
        items: &[
            "literal",
            "name",
            "field_read",
            "checked_index_read",
            "record_construction",
            "variant_construction",
            "task_call",
            "unary_operation",
            "binary_operation",
            "if_expression",
            "match_expression",
            "try_expression",
        ],
    },
    CoreCatalog {
        name: "statements",
        status: "design",
        role: "boring executable steps inside a checked task body",
        items: &[
            "let_binding",
            "mutable_binding",
            "set_place",
            "expression_statement",
            "if_statement",
            "match_statement",
            "while_loop",
            "loop",
            "for_each",
            "for_index",
            "return",
            "fail",
            "break",
            "continue",
        ],
    },
    CoreCatalog {
        name: "effects",
        status: "design",
        role: "closed starter effect set visible to profiles and diagnostics",
        items: &[
            "read", "change", "allocate", "free", "time", "random", "file", "network", "block",
            "spawn", "unsafe", "foreign", "panic",
        ],
    },
];

const CONTRACT_LOWERING: &[ContractLowering] = &[
    ContractLowering {
        section: "needs",
        lowers_to: "precondition_obligation_and_debug_entry_check",
        blame: "caller_or_context",
    },
    ContractLowering {
        section: "ensures",
        lowers_to: "postcondition_obligation_and_debug_exit_check",
        blame: "callee",
    },
    ContractLowering {
        section: "keeps",
        lowers_to: "invariant_obligation",
        blame: "loop_or_state_owner",
    },
    ContractLowering {
        section: "uses",
        lowers_to: "read_and_effect_permission",
        blame: "task_interface",
    },
    ContractLowering {
        section: "changes",
        lowers_to: "mutation_permission",
        blame: "task_interface",
    },
    ContractLowering {
        section: "fails when",
        lowers_to: "allowed_typed_failure_variant",
        blame: "callee_and_caller_must_handle",
    },
    ContractLowering {
        section: "cost",
        lowers_to: "resource_obligation_or_deferred_claim",
        blame: "implementation_and_benchmark_evidence",
    },
    ContractLowering {
        section: "allocates",
        lowers_to: "allocation_resource_obligation",
        blame: "implementation_and_profile",
    },
    ContractLowering {
        section: "protects",
        lowers_to: "security_or_safety_obligation",
        blame: "boundary_owner",
    },
    ContractLowering {
        section: "trusts",
        lowers_to: "explicit_unchecked_assumption",
        blame: "reviewer_and_integration_boundary",
    },
];

const ACCEPTANCE_GATES: &[AcceptanceGate] = &[
    AcceptanceGate {
        id: "parse",
        status: "current",
        requirement: "source item and section structure is captured",
    },
    AcceptanceGate {
        id: "semantic_graph_build",
        status: "current",
        requirement: "source spans, node ids, sections, symbols, and obligations are emitted",
    },
    AcceptanceGate {
        id: "body_grammar",
        status: core_body::CORE_BODY_GRAMMAR_STATUS,
        requirement: "does blocks expose first recognized statement and expression candidates",
    },
    AcceptanceGate {
        id: "core_lowering",
        status: "planned",
        requirement: "surface bodies lower to only Core Hum families listed here",
    },
    AcceptanceGate {
        id: "type_check",
        status: "planned",
        requirement: "values, places, calls, and failure paths have explicit types",
    },
    AcceptanceGate {
        id: "effect_check",
        status: "planned",
        requirement: "inferred effects fit declared uses, changes, fails, allocates, and profiles",
    },
    AcceptanceGate {
        id: "profile_check",
        status: "planned",
        requirement: "profiles can reject forbidden core operations before backend work",
    },
    AcceptanceGate {
        id: "core_interpreter",
        status: "planned",
        requirement: "a tiny checked subset runs before native backend claims",
    },
    AcceptanceGate {
        id: "core_verify",
        status: "planned",
        requirement: "Core Hum artifacts are printable, JSON-serializable, and verifier-checked",
    },
];

const RULES: &[&str] = &[
    "Surface Hum is stable only after it has a Core Hum meaning.",
    "Core Hum is smaller than Surface Hum and easier to test.",
    "Every core operation has explicit inputs, outputs, effects, failure behavior, and source provenance.",
    "Mutation requires a declared mutable place.",
    "Failure is a typed value path, not hidden control flow.",
    "Profiles restrict core operations, not prose.",
    "Agents may produce source, but they do not define core meaning.",
    "Hum IR consumes checked Core Hum, not raw surface body text.",
];

const NON_GOALS_V0: &[&str] = &[
    "no source-to-core lowering",
    "no executable semantics",
    "no type checker implementation",
    "no effect checker implementation",
    "no interpreter implementation",
    "no optimizer implementation",
    "no backend IR",
    "no generated artifact",
    "no safety proof",
    "no solver or network dependency",
];

pub fn core_contract_text() -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Core Hum contract ({CORE_CONTRACT_SCHEMA})\ntool: hum {} {}\nmilestone: {}\nlowers_from_schema: {}\nlowers_to_schema: {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS,
        version::HUM_MILESTONE,
        json::SEMANTIC_GRAPH_SCHEMA,
        ir_contract::IR_CONTRACT_SCHEMA
    ));
    out.push_str("core_catalogs:\n");
    for catalog in CORE_CATALOGS {
        out.push_str(&format!(
            "  {} [{}]: {}\n",
            catalog.name, catalog.status, catalog.role
        ));
        out.push_str(&format!("    items: {}\n", catalog.items.join(", ")));
    }
    out.push_str("contract_lowering:\n");
    for lowering in CONTRACT_LOWERING {
        out.push_str(&format!(
            "  {} -> {} [{}]\n",
            lowering.section, lowering.lowers_to, lowering.blame
        ));
    }
    out.push_str("acceptance_gates:\n");
    for gate in ACCEPTANCE_GATES {
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

pub fn core_contract_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", CORE_CONTRACT_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(
        &mut out,
        2,
        "lowers_from_schema",
        json::SEMANTIC_GRAPH_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "lowers_to_schema",
        ir_contract::IR_CONTRACT_SCHEMA,
        true,
    );
    push_core_catalogs(&mut out, 2, true);
    push_contract_lowering(&mut out, 2, true);
    push_acceptance_gates(&mut out, 2, true);
    push_string_array(&mut out, 2, "rules", RULES, true);
    push_string_array(&mut out, 2, "non_goals_v0", NON_GOALS_V0, false);
    out.push_str("}\n");
    out
}

fn push_core_catalogs(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"core_catalogs\": [\n");
    for (index, catalog) in CORE_CATALOGS.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "name", catalog.name, true);
        push_string_field(out, indent + 4, "status", catalog.status, true);
        push_string_field(out, indent + 4, "role", catalog.role, true);
        push_string_array(out, indent + 4, "items", catalog.items, false);
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_contract_lowering(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"contract_lowering\": [\n");
    for (index, lowering) in CONTRACT_LOWERING.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "section", lowering.section, true);
        push_string_field(out, indent + 4, "lowers_to", lowering.lowers_to, true);
        push_string_field(out, indent + 4, "blame", lowering.blame, false);
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_acceptance_gates(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"acceptance_gates\": [\n");
    for (index, gate) in ACCEPTANCE_GATES.iter().enumerate() {
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
    use super::{core_contract_json, core_contract_text};

    #[test]
    fn text_core_contract_lists_boring_core_surface() {
        let text = core_contract_text();

        assert!(text.contains("Core Hum contract (hum.core_contract.v0)"));
        assert!(text.contains("lowers_from_schema: hum.semantic_graph.v0"));
        assert!(text.contains("lowers_to_schema: hum.ir_contract.v0"));
        assert!(text.contains("statements [design]"));
        assert!(text.contains("set_place"));
        assert!(text.contains("body_grammar [partial_v0]"));
        assert!(text.contains("core_lowering [planned]"));
        assert!(text.contains("no interpreter implementation"));
    }

    #[test]
    fn json_core_contract_is_machine_readable() {
        let json = core_contract_json();

        assert!(json.contains("\"schema\": \"hum.core_contract.v0\""));
        assert!(json.contains("\"lowers_from_schema\": \"hum.semantic_graph.v0\""));
        assert!(json.contains("\"lowers_to_schema\": \"hum.ir_contract.v0\""));
        assert!(json.contains("\"name\": \"statements\""));
        assert!(json.contains("\"set_place\""));
        assert!(json.contains("\"section\": \"needs\""));
        assert!(json.contains("\"id\": \"body_grammar\""));
        assert!(json.contains("\"status\": \"partial_v0\""));
        assert!(json.contains("\"id\": \"core_lowering\""));
        assert!(json.contains("\"no executable semantics\""));
    }
}
