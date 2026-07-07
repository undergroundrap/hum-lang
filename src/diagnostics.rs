use crate::diagnostic_catalog::{self, DIAGNOSTIC_CATALOG_SCHEMA, DiagnosticInfo};

pub fn diagnostics_text() -> String {
    let diagnostics = diagnostic_catalog::all();
    let mut out = String::new();
    out.push_str(&format!("Hum diagnostics ({} codes)\n", diagnostics.len()));
    for info in diagnostics {
        out.push_str(&format!(
            "{} {} {}\n",
            info.code.as_str(),
            info.default_severity.as_str(),
            info.code.title()
        ));
    }
    out
}

pub fn diagnostics_json() -> String {
    let diagnostics = diagnostic_catalog::all();
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", DIAGNOSTIC_CATALOG_SCHEMA, true);
    push_indent(&mut out, 2);
    out.push_str(&format!("\"count\": {},\n", diagnostics.len()));
    push_indent(&mut out, 2);
    out.push_str("\"diagnostics\": [\n");
    for (index, info) in diagnostics.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_diagnostic_info(&mut out, info, 4);
    }
    out.push_str("\n  ]\n");
    out.push_str("}\n");
    out
}

fn push_diagnostic_info(out: &mut String, info: &DiagnosticInfo, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "code", info.code.as_str(), true);
    push_string_field(out, indent + 2, "title", info.code.title(), true);
    push_string_field(
        out,
        indent + 2,
        "default_severity",
        info.default_severity.as_str(),
        true,
    );
    push_string_field(out, indent + 2, "explanation", info.explanation, true);
    push_string_field(out, indent + 2, "repair", info.repair, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_string_field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_json_string(out, value);
    if comma {
        out.push(',');
    }
    out.push('\n');
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

#[cfg(test)]
mod tests {
    use super::{diagnostics_json, diagnostics_text};

    #[test]
    fn text_catalog_lists_known_codes() {
        let text = diagnostics_text();

        assert!(text.contains("Hum diagnostics"));
        assert!(text.contains("H0201 error save target not declared in changes"));
        assert!(text.contains("H0501 warning test missing covers section"));
    }

    #[test]
    fn json_catalog_lists_machine_readable_codes() {
        let json = diagnostics_json();

        assert!(json.contains("\"schema\": \"hum.diagnostic_catalog.v0\""));
        assert!(json.contains("\"code\": \"H0201\""));
        assert!(json.contains("\"default_severity\": \"error\""));
        assert!(json.contains("\"repair\": \"Add the resource under `changes:`"));
    }
}
