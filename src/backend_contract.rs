use crate::ir_contract;
use crate::version;

pub const BACKEND_CONTRACT_SCHEMA: &str = "hum.backend_contract.v0";

struct BackendStage {
    id: &'static str,
    stage: u8,
    status: &'static str,
    role: &'static str,
    decision: &'static str,
}

const DECISION_ID: &str = "0008-adopt-swappable-backend-ladder";
const SEMANTIC_OWNER: &str = "hum_ir";

const BACKEND_STAGES: &[BackendStage] = &[
    BackendStage {
        id: "interpreter",
        stage: 1,
        status: "planned",
        role: "first executable semantics and contract behavior",
        decision: "required before native backend work",
    },
    BackendStage {
        id: "cranelift",
        stage: 2,
        status: "planned",
        role: "first native proof and fast local feedback",
        decision: "candidate after interpreter proof",
    },
    BackendStage {
        id: "llvm",
        stage: 3,
        status: "planned",
        role: "mature optimized native AOT builds",
        decision: "production backend target after stable Hum IR",
    },
    BackendStage {
        id: "mlir",
        stage: 4,
        status: "deferred",
        role: "future multi-level lowering for vector, tensor, sparse, GPU, or accelerator work",
        decision: "only when Hum facts justify it",
    },
    BackendStage {
        id: "wasm_or_c",
        stage: 5,
        status: "deferred",
        role: "portable or inspectable escape hatch",
        decision: "use only behind the same adapter contract",
    },
    BackendStage {
        id: "custom_hum_backend",
        stage: 6,
        status: "deferred",
        role: "future Hum-specific backend or optimization stack",
        decision: "only after measured facts show existing backends cannot use Hum truth well",
    },
];

const REQUIRED_FACTS: &[&str] = &[
    "source_spans_and_semantic_graph_node_ids",
    "task_test_type_and_store_identity",
    "typed_failure_behavior",
    "effect_and_capability_facts",
    "ownership_and_aliasing_assumptions",
    "allocation_and_resource_facts",
    "profile_restrictions",
    "unsafe_and_foreign_boundaries",
    "debug_and_profiling_provenance",
    "unsupported_features_or_weakened_guarantees",
];

const RULES: &[&str] = &[
    "Hum IR owns semantics; backend IRs are adapters.",
    "Surface Hum must not lower directly to backend IR.",
    "Backend adapters must preserve required facts or report explicit loss.",
    "Cranelift is a first native proof candidate, not a newer LLVM.",
    "LLVM is a mature optimized AOT target, not Hum's semantic center.",
    "MLIR and custom backend work require evidence from real Hum facts.",
    "No safety-critical credibility comes from a backend choice alone.",
];

const NON_GOALS_V0: &[&str] = &[
    "no code execution",
    "no backend selection",
    "no optimizer promise",
    "no generated artifact",
    "no performance claim",
    "no solver or network dependency",
];

pub fn backend_contract_text() -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Hum backend contract ({BACKEND_CONTRACT_SCHEMA})\ntool: hum {} {}\nmilestone: {}\ndecision: {DECISION_ID}\nsemantic_owner: {SEMANTIC_OWNER}\nsemantic_owner_schema: {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS,
        version::HUM_MILESTONE,
        ir_contract::IR_CONTRACT_SCHEMA
    ));
    out.push_str("backend_order:\n");
    for stage in BACKEND_STAGES {
        out.push_str(&format!(
            "  {}. {} [{}]: {}; {}\n",
            stage.stage, stage.id, stage.status, stage.role, stage.decision
        ));
    }
    out.push_str("adapter_must_preserve_or_report_loss:\n");
    for fact in REQUIRED_FACTS {
        out.push_str(&format!("  {fact}\n"));
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

pub fn backend_contract_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", BACKEND_CONTRACT_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "decision", DECISION_ID, true);
    push_string_field(&mut out, 2, "semantic_owner", SEMANTIC_OWNER, true);
    push_string_field(
        &mut out,
        2,
        "semantic_owner_schema",
        ir_contract::IR_CONTRACT_SCHEMA,
        true,
    );
    push_backend_stages(&mut out, 2, true);
    push_string_array(
        &mut out,
        2,
        "adapter_must_preserve_or_report_loss",
        REQUIRED_FACTS,
        true,
    );
    push_string_array(&mut out, 2, "rules", RULES, true);
    push_string_array(&mut out, 2, "non_goals_v0", NON_GOALS_V0, false);
    out.push_str("}\n");
    out
}

fn push_backend_stages(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"backend_order\": [\n");
    for (index, stage) in BACKEND_STAGES.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "id", stage.id, true);
        push_u8_field(out, indent + 4, "stage", stage.stage, true);
        push_string_field(out, indent + 4, "status", stage.status, true);
        push_string_field(out, indent + 4, "role", stage.role, true);
        push_string_field(out, indent + 4, "decision", stage.decision, false);
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
    use super::{backend_contract_json, backend_contract_text};

    #[test]
    fn text_backend_contract_lists_swappable_backend_ladder() {
        let text = backend_contract_text();

        assert!(text.contains("Hum backend contract (hum.backend_contract.v0)"));
        assert!(text.contains("decision: 0008-adopt-swappable-backend-ladder"));
        assert!(text.contains("1. interpreter [planned]"));
        assert!(text.contains("2. cranelift [planned]"));
        assert!(text.contains("3. llvm [planned]"));
        assert!(text.contains("semantic_owner: hum_ir"));
        assert!(text.contains("semantic_owner_schema: hum.ir_contract.v0"));
    }

    #[test]
    fn json_backend_contract_is_machine_readable() {
        let json = backend_contract_json();

        assert!(json.contains("\"schema\": \"hum.backend_contract.v0\""));
        assert!(json.contains("\"decision\": \"0008-adopt-swappable-backend-ladder\""));
        assert!(json.contains("\"semantic_owner\": \"hum_ir\""));
        assert!(json.contains("\"semantic_owner_schema\": \"hum.ir_contract.v0\""));
        assert!(json.contains("\"id\": \"interpreter\""));
        assert!(json.contains("\"id\": \"cranelift\""));
        assert!(json.contains("\"id\": \"llvm\""));
        assert!(json.contains("\"source_spans_and_semantic_graph_node_ids\""));
        assert!(json.contains("\"no code execution\""));
    }
}
