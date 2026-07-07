use crate::core_contract;
use crate::version;

pub const IR_CONTRACT_SCHEMA: &str = "hum.ir_contract.v0";

struct IrLayer {
    id: &'static str,
    stage: u8,
    status: &'static str,
    role: &'static str,
}

const SEMANTIC_OWNER: &str = "hum_ir";
const BACKEND_CONTRACT_SCHEMA: &str = "hum.backend_contract.v0";

const IR_LAYERS: &[IrLayer] = &[
    IrLayer {
        id: "surface_hum",
        stage: 1,
        status: "current",
        role: "human-readable source captured by the parser and AST",
    },
    IrLayer {
        id: "semantic_graph",
        stage: 2,
        status: "current",
        role: "source facts for tools, agents, diagnostics, and evidence links",
    },
    IrLayer {
        id: "core_hum",
        stage: 3,
        status: "design",
        role: "small executable core for typed values, places, effects, and failure",
    },
    IrLayer {
        id: "hum_ir",
        stage: 4,
        status: "planned",
        role: "compiler-owned semantic IR consumed by interpreters and backend adapters",
    },
    IrLayer {
        id: "backend_adapter_input",
        stage: 5,
        status: "planned",
        role: "verified Hum IR plus explicit unsupported or weakened facts",
    },
];

const REQUIRED_CARRIED_FACTS: &[&str] = &[
    "source_spans",
    "semantic_graph_node_ids",
    "module_file_and_item_identity",
    "task_test_type_and_store_identity",
    "typed_values_and_places",
    "mutation_and_effect_facts",
    "typed_failure_edges",
    "contract_preconditions_postconditions_and_invariants",
    "evidence_and_math_obligation_links",
    "ownership_and_aliasing_assumptions",
    "allocation_and_resource_facts",
    "profile_guards",
    "unsafe_and_foreign_boundaries",
    "debug_and_profiling_provenance",
    "unsupported_or_weakened_facts",
];

const REQUIRED_PASSES: &[&str] = &[
    "parse",
    "semantic_graph_build",
    "core_lowering",
    "type_check",
    "effect_check",
    "ownership_alias_check",
    "allocation_resource_check",
    "contract_evidence_linking",
    "profile_check",
    "ir_verify",
];

const NODE_FAMILIES_V0: &[&str] = &[
    "module",
    "app",
    "type_def",
    "store",
    "task",
    "test",
    "block",
    "statement",
    "expression",
    "call",
    "effect",
    "failure",
    "contract",
    "resource",
    "profile_guard",
    "provenance",
];

const RULES: &[&str] = &[
    "Hum IR is the semantic owner named by the backend contract.",
    "Surface Hum must lower through Core Hum before backend IR.",
    "Every Hum IR artifact must be printable, JSON-serializable, and verifier-checked.",
    "Every backend adapter consumes verified Hum IR, not raw surface syntax.",
    "Every unsupported or weakened fact must be explicit in the IR or adapter report.",
    "Optimizations may change shape only when carried facts remain valid or are reported as changed.",
];

const NON_GOALS_V0: &[&str] = &[
    "no IR emission for source files",
    "no executable semantics",
    "no type checker implementation",
    "no optimizer implementation",
    "no backend lowering",
    "no generated artifact",
];

pub fn ir_contract_text() -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Hum IR contract ({IR_CONTRACT_SCHEMA})\ntool: hum {} {}\nmilestone: {}\nsemantic_owner: {SEMANTIC_OWNER}\ncore_contract_schema: {}\nbackend_contract_schema: {BACKEND_CONTRACT_SCHEMA}\n",
        version::HUM_VERSION,
        version::HUM_STATUS,
        version::HUM_MILESTONE,
        core_contract::CORE_CONTRACT_SCHEMA
    ));
    out.push_str("ir_layers:\n");
    for layer in IR_LAYERS {
        out.push_str(&format!(
            "  {}. {} [{}]: {}\n",
            layer.stage, layer.id, layer.status, layer.role
        ));
    }
    out.push_str("required_carried_facts:\n");
    for fact in REQUIRED_CARRIED_FACTS {
        out.push_str(&format!("  {fact}\n"));
    }
    out.push_str("required_passes:\n");
    for pass in REQUIRED_PASSES {
        out.push_str(&format!("  {pass}\n"));
    }
    out.push_str("node_families_v0:\n");
    for family in NODE_FAMILIES_V0 {
        out.push_str(&format!("  {family}\n"));
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

pub fn ir_contract_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", IR_CONTRACT_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "semantic_owner", SEMANTIC_OWNER, true);
    push_string_field(
        &mut out,
        2,
        "core_contract_schema",
        core_contract::CORE_CONTRACT_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "backend_contract_schema",
        BACKEND_CONTRACT_SCHEMA,
        true,
    );
    push_ir_layers(&mut out, 2, true);
    push_string_array(
        &mut out,
        2,
        "required_carried_facts",
        REQUIRED_CARRIED_FACTS,
        true,
    );
    push_string_array(&mut out, 2, "required_passes", REQUIRED_PASSES, true);
    push_string_array(&mut out, 2, "node_families_v0", NODE_FAMILIES_V0, true);
    push_string_array(&mut out, 2, "rules", RULES, true);
    push_string_array(&mut out, 2, "non_goals_v0", NON_GOALS_V0, false);
    out.push_str("}\n");
    out
}

fn push_ir_layers(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"ir_layers\": [\n");
    for (index, layer) in IR_LAYERS.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "id", layer.id, true);
        push_u8_field(out, indent + 4, "stage", layer.stage, true);
        push_string_field(out, indent + 4, "status", layer.status, true);
        push_string_field(out, indent + 4, "role", layer.role, false);
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

fn push_u8_field(out: &mut String, indent: usize, key: &str, value: u8, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(&format!(": {value}"));
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
    use super::{ir_contract_json, ir_contract_text};

    #[test]
    fn text_ir_contract_lists_semantic_layers() {
        let text = ir_contract_text();

        assert!(text.contains("Hum IR contract (hum.ir_contract.v0)"));
        assert!(text.contains("semantic_owner: hum_ir"));
        assert!(text.contains("core_contract_schema: hum.core_contract.v0"));
        assert!(text.contains("1. surface_hum [current]"));
        assert!(text.contains("4. hum_ir [planned]"));
        assert!(text.contains("required_carried_facts"));
    }

    #[test]
    fn json_ir_contract_is_machine_readable() {
        let json = ir_contract_json();

        assert!(json.contains("\"schema\": \"hum.ir_contract.v0\""));
        assert!(json.contains("\"semantic_owner\": \"hum_ir\""));
        assert!(json.contains("\"core_contract_schema\": \"hum.core_contract.v0\""));
        assert!(json.contains("\"backend_contract_schema\": \"hum.backend_contract.v0\""));
        assert!(json.contains("\"id\": \"core_hum\""));
        assert!(json.contains("\"id\": \"hum_ir\""));
        assert!(json.contains("\"typed_failure_edges\""));
        assert!(json.contains("\"ir_verify\""));
        assert!(json.contains("\"no IR emission for source files\""));
    }
}
