use crate::diagnostic_catalog::{self, DIAGNOSTIC_EXPLAIN_SCHEMA, DiagnosticInfo};

pub fn explain_text(code: &str) -> Result<String, String> {
    let info = diagnostic_info(code)?;
    let mut out = String::new();
    out.push_str(&format!("{}: {}\n", info.code.as_str(), info.code.title()));
    out.push_str(&format!(
        "default severity: {}\n\n",
        info.default_severity.as_str()
    ));
    out.push_str(info.explanation);
    out.push_str("\n\nrepair:\n  ");
    out.push_str(info.repair);
    out.push('\n');
    Ok(out)
}

pub fn explain_json(code: &str) -> Result<String, String> {
    let info = diagnostic_info(code)?;
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", DIAGNOSTIC_EXPLAIN_SCHEMA, true);
    push_string_field(&mut out, 2, "code", info.code.as_str(), true);
    push_string_field(&mut out, 2, "title", info.code.title(), true);
    push_string_field(
        &mut out,
        2,
        "default_severity",
        info.default_severity.as_str(),
        true,
    );
    push_string_field(&mut out, 2, "explanation", info.explanation, true);
    push_string_field(&mut out, 2, "repair", info.repair, false);
    out.push_str("}\n");
    Ok(out)
}

fn diagnostic_info(code: &str) -> Result<&'static DiagnosticInfo, String> {
    diagnostic_catalog::find(code).ok_or_else(|| {
        format!(
            "unknown diagnostic code `{}`; run `hum explain H0201` for a known code",
            code.trim()
        )
    })
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
    use super::{explain_json, explain_text};

    #[test]
    fn explains_known_code_as_text() {
        let text = explain_text("H0201").expect("known code");

        assert!(text.contains("H0201: save target not declared in changes"));
        assert!(text.contains("default severity: error"));
        assert!(text.contains("repair:"));
    }

    #[test]
    fn explains_known_code_as_json() {
        let json = explain_json("h0501").expect("known code");

        assert!(json.contains("\"schema\": \"hum.diagnostic_explain.v0\""));
        assert!(json.contains("\"code\": \"H0501\""));
        assert!(json.contains("\"default_severity\": \"warning\""));
    }

    #[test]
    fn rejects_unknown_code() {
        let error = explain_text("H9999").expect_err("unknown code");

        assert!(error.contains("unknown diagnostic code `H9999`"));
    }
}
