use crate::backend_contract;
use crate::capabilities;
use crate::core_contract;
use crate::core_preview;
use crate::diagnostic_catalog;
use crate::doctor;
use crate::ir_contract;
use crate::ir_readiness;
use crate::json;
use crate::lsp;
use crate::math_obligations;
use crate::resource_report;
use crate::runtime_profiles;
use crate::syntax;
use crate::target_facts;

pub const HUM_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const HUM_STATUS: &str = "pre-alpha";
pub const HUM_MILESTONE: &str = "0 semantic graph";

pub fn version_text() -> String {
    format!(
        "Hum {HUM_VERSION} {HUM_STATUS}\nmilestone: {HUM_MILESTONE}\ntarget: {}\nsemantic_graph_schema: {}\nsyntax_surface_schema: {}\ndiagnostic_explain_schema: {}\ndiagnostic_catalog_schema: {}\ncapabilities_schema: {}\ncore_contract_schema: {}\ncore_preview_schema: {}\nir_contract_schema: {}\nbackend_contract_schema: {}\nruntime_profiles_schema: {}\nruntime_profile_schema: {}\nlsp_capabilities_schema: {}\nmath_obligations_report_schema: {}\nmath_obligation_schema: {}\nresource_report_schema: {}\nir_readiness_schema: {}\ndoctor_schema: {}\ntarget_facts_schema: {}\ntarget_fact_record_schema: {}\n",
        target_name(),
        json::SEMANTIC_GRAPH_SCHEMA,
        syntax::SYNTAX_SCHEMA,
        diagnostic_catalog::DIAGNOSTIC_EXPLAIN_SCHEMA,
        diagnostic_catalog::DIAGNOSTIC_CATALOG_SCHEMA,
        capabilities::CAPABILITIES_SCHEMA,
        core_contract::CORE_CONTRACT_SCHEMA,
        core_preview::CORE_PREVIEW_SCHEMA,
        ir_contract::IR_CONTRACT_SCHEMA,
        backend_contract::BACKEND_CONTRACT_SCHEMA,
        runtime_profiles::RUNTIME_PROFILES_SCHEMA,
        runtime_profiles::RUNTIME_PROFILE_SCHEMA,
        lsp::LSP_CAPABILITIES_SCHEMA,
        math_obligations::MATH_OBLIGATIONS_REPORT_SCHEMA,
        math_obligations::MATH_OBLIGATION_SCHEMA,
        resource_report::RESOURCE_REPORT_SCHEMA,
        ir_readiness::IR_READINESS_SCHEMA,
        doctor::DOCTOR_SCHEMA,
        target_facts::TARGET_FACTS_SCHEMA,
        target_facts::TARGET_FACT_RECORD_SCHEMA
    )
}

pub fn version_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "target", &target_name(), true);
    push_indent(&mut out, 2);
    out.push_str("\"schemas\": {\n");
    push_string_field(
        &mut out,
        4,
        "semantic_graph",
        json::SEMANTIC_GRAPH_SCHEMA,
        true,
    );
    push_string_field(&mut out, 4, "syntax_surface", syntax::SYNTAX_SCHEMA, true);
    push_string_field(
        &mut out,
        4,
        "diagnostic_explain",
        diagnostic_catalog::DIAGNOSTIC_EXPLAIN_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "diagnostic_catalog",
        diagnostic_catalog::DIAGNOSTIC_CATALOG_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "capabilities",
        capabilities::CAPABILITIES_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "core_contract",
        core_contract::CORE_CONTRACT_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "core_preview",
        core_preview::CORE_PREVIEW_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "ir_contract",
        ir_contract::IR_CONTRACT_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "backend_contract",
        backend_contract::BACKEND_CONTRACT_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "runtime_profiles",
        runtime_profiles::RUNTIME_PROFILES_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "runtime_profile",
        runtime_profiles::RUNTIME_PROFILE_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "lsp_capabilities",
        lsp::LSP_CAPABILITIES_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "math_obligations_report",
        math_obligations::MATH_OBLIGATIONS_REPORT_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "math_obligation",
        math_obligations::MATH_OBLIGATION_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "resource_report",
        resource_report::RESOURCE_REPORT_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "ir_readiness",
        ir_readiness::IR_READINESS_SCHEMA,
        true,
    );
    push_string_field(&mut out, 4, "doctor", doctor::DOCTOR_SCHEMA, true);
    push_string_field(
        &mut out,
        4,
        "target_facts",
        target_facts::TARGET_FACTS_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        4,
        "target_fact_record",
        target_facts::TARGET_FACT_RECORD_SCHEMA,
        false,
    );
    push_indent(&mut out, 2);
    out.push_str("},\n");
    push_indent(&mut out, 2);
    out.push_str("\"build\": {\n");
    push_optional_string_field(
        &mut out,
        4,
        "commit",
        option_env!("HUM_BUILD_COMMIT"),
        false,
    );
    push_indent(&mut out, 2);
    out.push_str("}\n");
    out.push_str("}\n");
    out
}

fn target_name() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

fn push_string_field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_json_string(out, value);
    push_comma_newline(out, comma);
}

fn push_optional_string_field(
    out: &mut String,
    indent: usize,
    key: &str,
    value: Option<&str>,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    match value {
        Some(value) => push_json_string(out, value),
        None => out.push_str("null"),
    }
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
    use super::{HUM_VERSION, version_json, version_text};

    #[test]
    fn cargo_version_matches_repo_version_file() {
        assert_eq!(HUM_VERSION, include_str!("../VERSION").trim());
    }

    #[test]
    fn text_version_reports_hum_version_and_schemas() {
        let text = version_text();

        assert!(text.contains("Hum 0.0.1 pre-alpha"));
        assert!(text.contains("semantic_graph_schema: hum.semantic_graph.v0"));
        assert!(text.contains("syntax_surface_schema: hum.syntax_surface.v0"));
        assert!(text.contains("diagnostic_explain_schema: hum.diagnostic_explain.v0"));
        assert!(text.contains("diagnostic_catalog_schema: hum.diagnostic_catalog.v0"));
        assert!(text.contains("capabilities_schema: hum.capabilities.v0"));
        assert!(text.contains("core_contract_schema: hum.core_contract.v0"));
        assert!(text.contains("core_preview_schema: hum.core_preview.v0"));
        assert!(text.contains("ir_contract_schema: hum.ir_contract.v0"));
        assert!(text.contains("backend_contract_schema: hum.backend_contract.v0"));
        assert!(text.contains("runtime_profiles_schema: hum.runtime_profiles.v0"));
        assert!(text.contains("runtime_profile_schema: hum.runtime_profile.v0"));
        assert!(text.contains("lsp_capabilities_schema: hum.lsp_capabilities.v0"));
        assert!(text.contains("math_obligations_report_schema: hum.math_obligations.v0"));
        assert!(text.contains("math_obligation_schema: hum.math_obligation.v0"));
        assert!(text.contains("resource_report_schema: hum.resource_report.v0"));
        assert!(text.contains("ir_readiness_schema: hum.ir_readiness.v0"));
        assert!(text.contains("doctor_schema: hum.doctor.v0"));
        assert!(text.contains("target_facts_schema: hum.target_facts.v0"));
        assert!(text.contains("target_fact_record_schema: hum.target_fact_record.v0"));
    }

    #[test]
    fn json_version_reports_machine_readable_identity() {
        let json = version_json();

        assert!(json.contains("\"tool\": \"hum\""));
        assert!(json.contains("\"version\": \"0.0.1\""));
        assert!(json.contains("\"semantic_graph\": \"hum.semantic_graph.v0\""));
        assert!(json.contains("\"syntax_surface\": \"hum.syntax_surface.v0\""));
        assert!(json.contains("\"diagnostic_explain\": \"hum.diagnostic_explain.v0\""));
        assert!(json.contains("\"diagnostic_catalog\": \"hum.diagnostic_catalog.v0\""));
        assert!(json.contains("\"capabilities\": \"hum.capabilities.v0\""));
        assert!(json.contains("\"core_contract\": \"hum.core_contract.v0\""));
        assert!(json.contains("\"core_preview\": \"hum.core_preview.v0\""));
        assert!(json.contains("\"ir_contract\": \"hum.ir_contract.v0\""));
        assert!(json.contains("\"backend_contract\": \"hum.backend_contract.v0\""));
        assert!(json.contains("\"runtime_profiles\": \"hum.runtime_profiles.v0\""));
        assert!(json.contains("\"runtime_profile\": \"hum.runtime_profile.v0\""));
        assert!(json.contains("\"lsp_capabilities\": \"hum.lsp_capabilities.v0\""));
        assert!(json.contains("\"math_obligations_report\": \"hum.math_obligations.v0\""));
        assert!(json.contains("\"math_obligation\": \"hum.math_obligation.v0\""));
        assert!(json.contains("\"resource_report\": \"hum.resource_report.v0\""));
        assert!(json.contains("\"ir_readiness\": \"hum.ir_readiness.v0\""));
        assert!(json.contains("\"doctor\": \"hum.doctor.v0\""));
        assert!(json.contains("\"target_facts\": \"hum.target_facts.v0\""));
        assert!(json.contains("\"target_fact_record\": \"hum.target_fact_record.v0\""));
    }
}
