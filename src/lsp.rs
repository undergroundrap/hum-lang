use crate::capabilities;
use crate::diagnostics;
use crate::json;
use crate::syntax;

pub const LSP_CAPABILITIES_SCHEMA: &str = "hum.lsp_capabilities.v0";

struct LspCapability {
    method: &'static str,
    status: &'static str,
    source: &'static str,
    schema: &'static str,
    note: &'static str,
}

const LSP_CAPABILITIES: &[LspCapability] = &[
    LspCapability {
        method: "textDocument/publishDiagnostics",
        status: "adapter-ready",
        source: "hum check --format json",
        schema: diagnostics::CHECK_DIAGNOSTICS_SCHEMA,
        note: "Adapters can publish diagnostics from source-backed spans.",
    },
    LspCapability {
        method: "textDocument/documentSymbol",
        status: "adapter-ready",
        source: "hum graph files[].symbols",
        schema: json::SEMANTIC_GRAPH_SCHEMA,
        note: "Current symbols cover apps, tasks, tests, types, stores, and type fields.",
    },
    LspCapability {
        method: "textDocument/foldingRange",
        status: "adapter-ready",
        source: "hum graph files[].folding_ranges",
        schema: json::SEMANTIC_GRAPH_SCHEMA,
        note: "Current folding ranges cover intent sections.",
    },
    LspCapability {
        method: "textDocument/hover",
        status: "adapter-ready",
        source: "hum syntax section_headers.section_catalog",
        schema: syntax::SYNTAX_SCHEMA,
        note: "Section hovers are adapter-ready; declared-name hovers remain planned.",
    },
    LspCapability {
        method: "textDocument/semanticTokens/full",
        status: "planned",
        source: "hum syntax semantic_tokens",
        schema: syntax::SYNTAX_SCHEMA,
        note: "The legend is current; per-file token ranges wait for precise source ranges.",
    },
    LspCapability {
        method: "textDocument/formatting",
        status: "planned",
        source: "humfmt",
        schema: "none",
        note: "Formatting waits for lossless trivia and comment preservation.",
    },
    LspCapability {
        method: "workspace/symbol",
        status: "deferred",
        source: "future Nectar workspace graph",
        schema: "none",
        note: "Workspace symbols wait for package and module semantics.",
    },
];

pub fn lsp_capabilities_text() -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Hum LSP capabilities ({LSP_CAPABILITIES_SCHEMA})\nserver_command: hum lsp\nserver_status: planned\njson_rpc_server: false\nsource_capabilities_schema: {}\n",
        capabilities::CAPABILITIES_SCHEMA
    ));
    out.push_str("capabilities:\n");
    for capability in LSP_CAPABILITIES {
        out.push_str(&format!(
            "  {} [{}]: {}\n",
            capability.method, capability.status, capability.source
        ));
    }
    out
}

pub fn lsp_capabilities_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", LSP_CAPABILITIES_SCHEMA, true);
    push_string_field(&mut out, 2, "server_command", "hum lsp", true);
    push_string_field(&mut out, 2, "server_status", "planned", true);
    push_bool_field(&mut out, 2, "json_rpc_server", false, true);
    push_string_field(
        &mut out,
        2,
        "source_capabilities_schema",
        capabilities::CAPABILITIES_SCHEMA,
        true,
    );
    push_lsp_capabilities(&mut out, 2, false);
    out.push_str("}\n");
    out
}

fn push_lsp_capabilities(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"capabilities\": [\n");
    for (index, capability) in LSP_CAPABILITIES.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "method", capability.method, true);
        push_string_field(out, indent + 4, "status", capability.status, true);
        push_string_field(out, indent + 4, "source", capability.source, true);
        push_string_field(out, indent + 4, "schema", capability.schema, true);
        push_string_field(out, indent + 4, "note", capability.note, false);
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

fn push_bool_field(out: &mut String, indent: usize, key: &str, value: bool, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    out.push_str(if value { "true" } else { "false" });
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
    use super::{lsp_capabilities_json, lsp_capabilities_text};

    #[test]
    fn text_lsp_capabilities_report_preview_status() {
        let text = lsp_capabilities_text();

        assert!(text.contains("Hum LSP capabilities (hum.lsp_capabilities.v0)"));
        assert!(text.contains("json_rpc_server: false"));
        assert!(text.contains("textDocument/documentSymbol"));
    }

    #[test]
    fn json_lsp_capabilities_report_adapter_ready_methods() {
        let json = lsp_capabilities_json();

        assert!(json.contains("\"schema\": \"hum.lsp_capabilities.v0\""));
        assert!(json.contains("\"json_rpc_server\": false"));
        assert!(json.contains("\"method\": \"textDocument/publishDiagnostics\""));
        assert!(json.contains("\"method\": \"textDocument/documentSymbol\""));
        assert!(json.contains("\"source_capabilities_schema\": \"hum.capabilities.v0\""));
    }
}
