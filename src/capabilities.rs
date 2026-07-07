use crate::backend_contract;
use crate::core_contract;
use crate::core_preview;
use crate::diagnostic_catalog;
use crate::diagnostics;
use crate::doctor;
use crate::evidence;
use crate::ir_contract;
use crate::ir_readiness;
use crate::json;
use crate::lsp;
use crate::math_obligations;
use crate::resource_report;
use crate::syntax;
use crate::target_facts;
use crate::version;

pub const CAPABILITIES_SCHEMA: &str = "hum.capabilities.v0";

struct CommandCapability {
    name: &'static str,
    command: &'static str,
    schema: &'static str,
    status: &'static str,
    purpose: &'static str,
}

struct EditorCapability {
    name: &'static str,
    status: &'static str,
    source: &'static str,
    schema: &'static str,
}

const COMMANDS: &[CommandCapability] = &[
    CommandCapability {
        name: "check_json",
        command: "hum check --format json <file-or-dir>...",
        schema: diagnostics::CHECK_DIAGNOSTICS_SCHEMA,
        status: "adapter-ready",
        purpose: "source-backed diagnostics for editors, CI, and agents",
    },
    CommandCapability {
        name: "graph",
        command: "hum graph <file-or-dir>...",
        schema: json::SEMANTIC_GRAPH_SCHEMA,
        status: "adapter-ready",
        purpose: "semantic graph facts for tools and agents",
    },
    CommandCapability {
        name: "evidence_json",
        command: "hum evidence --format json <file-or-dir>...",
        schema: evidence::EVIDENCE_REPORT_SCHEMA,
        status: "adapter-ready",
        purpose: "security and trust evidence obligation report for humans, agents, and CI",
    },
    CommandCapability {
        name: "math_obligations_json",
        command: "hum math-obligations --format json <file-or-dir>...",
        schema: math_obligations::MATH_OBLIGATIONS_REPORT_SCHEMA,
        status: "adapter-ready",
        purpose: "external-validator math obligation candidates without running solvers",
    },
    CommandCapability {
        name: "resource_report_json",
        command: "hum resource-report --format json <file-or-dir>...",
        schema: resource_report::RESOURCE_REPORT_SCHEMA,
        status: "adapter-ready",
        purpose: "source-declared resource, layout, and optimization claim inventory",
    },
    CommandCapability {
        name: "core_preview_json",
        command: "hum core-preview --format json <file-or-dir>...",
        schema: core_preview::CORE_PREVIEW_SCHEMA,
        status: "adapter-ready",
        purpose: "Core Hum preview candidates and blockers without execution",
    },
    CommandCapability {
        name: "ir_readiness_json",
        command: "hum ir-readiness --format json <file-or-dir>...",
        schema: ir_readiness::IR_READINESS_SCHEMA,
        status: "adapter-ready",
        purpose: "source readiness report for future Core Hum and Hum IR lowering",
    },
    CommandCapability {
        name: "syntax",
        command: "hum syntax",
        schema: syntax::SYNTAX_SCHEMA,
        status: "current",
        purpose: "editor-neutral syntax metadata, section hovers, and semantic-token legend",
    },
    CommandCapability {
        name: "syntax_textmate",
        command: "hum syntax --format textmate",
        schema: syntax::SYNTAX_SCHEMA,
        status: "current",
        purpose: "generated TextMate grammar for baseline highlighting",
    },
    CommandCapability {
        name: "explain_json",
        command: "hum explain <H####> --format json",
        schema: diagnostic_catalog::DIAGNOSTIC_EXPLAIN_SCHEMA,
        status: "adapter-ready",
        purpose: "offline explanation for one stable diagnostic code",
    },
    CommandCapability {
        name: "diagnostics_json",
        command: "hum diagnostics --format json",
        schema: diagnostic_catalog::DIAGNOSTIC_CATALOG_SCHEMA,
        status: "adapter-ready",
        purpose: "offline catalog of stable diagnostic codes",
    },
    CommandCapability {
        name: "capabilities_json",
        command: "hum capabilities --format json",
        schema: CAPABILITIES_SCHEMA,
        status: "current",
        purpose: "toolchain surface discovery for adapters and agents",
    },
    CommandCapability {
        name: "core_contract_json",
        command: "hum core-contract --format json",
        schema: core_contract::CORE_CONTRACT_SCHEMA,
        status: "current",
        purpose: "Core Hum executable subset and surface-to-core acceptance contract",
    },
    CommandCapability {
        name: "ir_contract_json",
        command: "hum ir-contract --format json",
        schema: ir_contract::IR_CONTRACT_SCHEMA,
        status: "current",
        purpose: "Hum IR ownership, carried facts, required passes, and preservation contract",
    },
    CommandCapability {
        name: "backend_contract_json",
        command: "hum backend-contract --format json",
        schema: backend_contract::BACKEND_CONTRACT_SCHEMA,
        status: "current",
        purpose: "swappable backend ladder and adapter preservation contract",
    },
    CommandCapability {
        name: "lsp_capabilities_json",
        command: "hum lsp --capabilities --format json",
        schema: lsp::LSP_CAPABILITIES_SCHEMA,
        status: "current",
        purpose: "LSP adapter capability preview without starting server mode",
    },
    CommandCapability {
        name: "doctor_json",
        command: "hum doctor --format json",
        schema: doctor::DOCTOR_SCHEMA,
        status: "current",
        purpose: "portable repo setup and guardrail health report",
    },
    CommandCapability {
        name: "target_facts_json",
        command: "hum target-facts --format json",
        schema: target_facts::TARGET_FACTS_SCHEMA,
        status: "current",
        purpose: "target-fact field catalog and portability fixture records",
    },
];

const EDITOR_CAPABILITIES: &[EditorCapability] = &[
    EditorCapability {
        name: "diagnostics",
        status: "adapter-ready",
        source: "hum check --format json",
        schema: diagnostics::CHECK_DIAGNOSTICS_SCHEMA,
    },
    EditorCapability {
        name: "diagnostic_explanations",
        status: "adapter-ready",
        source: "hum explain --format json; hum diagnostics --format json",
        schema: diagnostic_catalog::DIAGNOSTIC_CATALOG_SCHEMA,
    },
    EditorCapability {
        name: "document_symbols",
        status: "adapter-ready",
        source: "hum graph files[].symbols",
        schema: json::SEMANTIC_GRAPH_SCHEMA,
    },
    EditorCapability {
        name: "folding_ranges",
        status: "adapter-ready",
        source: "hum graph files[].folding_ranges",
        schema: json::SEMANTIC_GRAPH_SCHEMA,
    },
    EditorCapability {
        name: "section_hover",
        status: "adapter-ready",
        source: "hum syntax section_headers.section_catalog",
        schema: syntax::SYNTAX_SCHEMA,
    },
    EditorCapability {
        name: "semantic_token_legend",
        status: "current",
        source: "hum syntax semantic_tokens",
        schema: syntax::SYNTAX_SCHEMA,
    },
    EditorCapability {
        name: "textmate_highlighting",
        status: "current",
        source: "hum syntax --format textmate",
        schema: syntax::SYNTAX_SCHEMA,
    },
    EditorCapability {
        name: "editor_recovery_fixtures",
        status: "current",
        source: "fixtures/editor; tools/check_editor_fixtures.ps1",
        schema: json::SEMANTIC_GRAPH_SCHEMA,
    },
    EditorCapability {
        name: "formatting",
        status: "planned",
        source: "humfmt",
        schema: "none",
    },
    EditorCapability {
        name: "code_actions",
        status: "planned",
        source: "hum.check.v0 plus diagnostic catalog",
        schema: diagnostics::CHECK_DIAGNOSTICS_SCHEMA,
    },
    EditorCapability {
        name: "go_to_definition",
        status: "deferred",
        source: "future symbol table and reference graph",
        schema: "none",
    },
    EditorCapability {
        name: "jupyter_kernel",
        status: "deferred",
        source: "future safe execution model",
        schema: "none",
    },
];

pub fn capabilities_text() -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Hum capabilities ({CAPABILITIES_SCHEMA})\ntool: hum {} {}\nmilestone: {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS,
        version::HUM_MILESTONE
    ));
    out.push_str("schemas:\n");
    out.push_str(&format!(
        "  semantic_graph: {}\n",
        json::SEMANTIC_GRAPH_SCHEMA
    ));
    out.push_str(&format!("  syntax_surface: {}\n", syntax::SYNTAX_SCHEMA));
    out.push_str(&format!(
        "  check_diagnostics: {}\n",
        diagnostics::CHECK_DIAGNOSTICS_SCHEMA
    ));
    out.push_str(&format!(
        "  evidence_report: {}\n",
        evidence::EVIDENCE_REPORT_SCHEMA
    ));
    out.push_str(&format!(
        "  math_obligations_report: {}\n",
        math_obligations::MATH_OBLIGATIONS_REPORT_SCHEMA
    ));
    out.push_str(&format!(
        "  math_obligation: {}\n",
        math_obligations::MATH_OBLIGATION_SCHEMA
    ));
    out.push_str(&format!(
        "  resource_report: {}\n",
        resource_report::RESOURCE_REPORT_SCHEMA
    ));
    out.push_str(&format!(
        "  core_preview: {}\n  ir_readiness: {}\n",
        core_preview::CORE_PREVIEW_SCHEMA,
        ir_readiness::IR_READINESS_SCHEMA
    ));
    out.push_str(&format!(
        "  diagnostic_explain: {}\n",
        diagnostic_catalog::DIAGNOSTIC_EXPLAIN_SCHEMA
    ));
    out.push_str(&format!(
        "  diagnostic_catalog: {}\n",
        diagnostic_catalog::DIAGNOSTIC_CATALOG_SCHEMA
    ));
    out.push_str(&format!("  capabilities: {}\n", CAPABILITIES_SCHEMA));
    out.push_str(&format!(
        "  core_contract: {}\n",
        core_contract::CORE_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "  ir_contract: {}\n",
        ir_contract::IR_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "  backend_contract: {}\n",
        backend_contract::BACKEND_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "  lsp_capabilities: {}\n",
        lsp::LSP_CAPABILITIES_SCHEMA
    ));
    out.push_str(&format!("  doctor: {}\n", doctor::DOCTOR_SCHEMA));
    out.push_str(&format!(
        "  target_facts: {}\n  target_fact_record: {}\n",
        target_facts::TARGET_FACTS_SCHEMA,
        target_facts::TARGET_FACT_RECORD_SCHEMA
    ));
    out.push_str("commands:\n");
    for command in COMMANDS {
        out.push_str(&format!(
            "  {} [{}]: {}\n",
            command.name, command.status, command.command
        ));
    }
    out.push_str("editor_capabilities:\n");
    for capability in EDITOR_CAPABILITIES {
        out.push_str(&format!(
            "  {} [{}]: {}\n",
            capability.name, capability.status, capability.source
        ));
    }
    out
}

pub fn capabilities_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", CAPABILITIES_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_schemas(&mut out, 2, true);
    push_commands(&mut out, 2, true);
    push_editor_capabilities(&mut out, 2, false);
    out.push_str("}\n");
    out
}

fn push_schemas(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"schemas\": {\n");
    push_string_field(
        out,
        indent + 2,
        "semantic_graph",
        json::SEMANTIC_GRAPH_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "syntax_surface",
        syntax::SYNTAX_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "check_diagnostics",
        diagnostics::CHECK_DIAGNOSTICS_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "evidence_report",
        evidence::EVIDENCE_REPORT_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "math_obligations_report",
        math_obligations::MATH_OBLIGATIONS_REPORT_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "math_obligation",
        math_obligations::MATH_OBLIGATION_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "resource_report",
        resource_report::RESOURCE_REPORT_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "ir_readiness",
        ir_readiness::IR_READINESS_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "core_preview",
        core_preview::CORE_PREVIEW_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "diagnostic_explain",
        diagnostic_catalog::DIAGNOSTIC_EXPLAIN_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "diagnostic_catalog",
        diagnostic_catalog::DIAGNOSTIC_CATALOG_SCHEMA,
        true,
    );
    push_string_field(out, indent + 2, "capabilities", CAPABILITIES_SCHEMA, true);
    push_string_field(
        out,
        indent + 2,
        "core_contract",
        core_contract::CORE_CONTRACT_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "ir_contract",
        ir_contract::IR_CONTRACT_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "backend_contract",
        backend_contract::BACKEND_CONTRACT_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "lsp_capabilities",
        lsp::LSP_CAPABILITIES_SCHEMA,
        true,
    );
    push_string_field(out, indent + 2, "doctor", doctor::DOCTOR_SCHEMA, true);
    push_string_field(
        out,
        indent + 2,
        "target_facts",
        target_facts::TARGET_FACTS_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "target_fact_record",
        target_facts::TARGET_FACT_RECORD_SCHEMA,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_commands(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"commands\": [\n");
    for (index, command) in COMMANDS.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "name", command.name, true);
        push_string_field(out, indent + 4, "command", command.command, true);
        push_string_field(out, indent + 4, "schema", command.schema, true);
        push_string_field(out, indent + 4, "status", command.status, true);
        push_string_field(out, indent + 4, "purpose", command.purpose, false);
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_editor_capabilities(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"editor_capabilities\": [\n");
    for (index, capability) in EDITOR_CAPABILITIES.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "name", capability.name, true);
        push_string_field(out, indent + 4, "status", capability.status, true);
        push_string_field(out, indent + 4, "source", capability.source, true);
        push_string_field(out, indent + 4, "schema", capability.schema, false);
        push_indent(out, indent + 2);
        out.push('}');
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
    use super::{capabilities_json, capabilities_text};

    #[test]
    fn text_capabilities_report_tool_surfaces() {
        let text = capabilities_text();

        assert!(text.contains("Hum capabilities (hum.capabilities.v0)"));
        assert!(text.contains("hum check --format json"));
        assert!(text.contains("hum evidence --format json"));
        assert!(text.contains("document_symbols"));
        assert!(text.contains("semantic_token_legend"));
    }

    #[test]
    fn json_capabilities_report_machine_readable_surfaces() {
        let json = capabilities_json();

        assert!(json.contains("\"schema\": \"hum.capabilities.v0\""));
        assert!(json.contains("\"check_diagnostics\": \"hum.check.v0\""));
        assert!(json.contains("\"evidence_report\": \"hum.evidence.v0\""));
        assert!(json.contains("\"math_obligations_report\": \"hum.math_obligations.v0\""));
        assert!(json.contains("\"math_obligation\": \"hum.math_obligation.v0\""));
        assert!(json.contains("\"resource_report\": \"hum.resource_report.v0\""));
        assert!(json.contains("\"ir_readiness\": \"hum.ir_readiness.v0\""));
        assert!(json.contains("\"core_preview\": \"hum.core_preview.v0\""));
        assert!(json.contains("\"semantic_graph\": \"hum.semantic_graph.v0\""));
        assert!(json.contains("\"syntax_surface\": \"hum.syntax_surface.v0\""));
        assert!(json.contains("\"capabilities\": \"hum.capabilities.v0\""));
        assert!(json.contains("\"core_contract\": \"hum.core_contract.v0\""));
        assert!(json.contains("\"ir_contract\": \"hum.ir_contract.v0\""));
        assert!(json.contains("\"backend_contract\": \"hum.backend_contract.v0\""));
        assert!(json.contains("\"lsp_capabilities\": \"hum.lsp_capabilities.v0\""));
        assert!(json.contains("\"doctor\": \"hum.doctor.v0\""));
        assert!(json.contains("\"target_facts\": \"hum.target_facts.v0\""));
        assert!(json.contains("\"target_fact_record\": \"hum.target_fact_record.v0\""));
        assert!(json.contains("\"name\": \"doctor_json\""));
        assert!(json.contains("\"name\": \"target_facts_json\""));
        assert!(json.contains("\"name\": \"core_contract_json\""));
        assert!(json.contains("\"name\": \"ir_contract_json\""));
        assert!(json.contains("\"name\": \"backend_contract_json\""));
        assert!(json.contains("\"name\": \"evidence_json\""));
        assert!(json.contains("\"name\": \"math_obligations_json\""));
        assert!(json.contains("\"name\": \"resource_report_json\""));
        assert!(json.contains("\"name\": \"ir_readiness_json\""));
        assert!(json.contains("\"name\": \"core_preview_json\""));
        assert!(json.contains("\"name\": \"folding_ranges\""));
        assert!(json.contains("\"status\": \"adapter-ready\""));
    }
}
