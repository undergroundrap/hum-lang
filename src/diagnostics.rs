use crate::ast::Program;
use crate::diagnostic::{Diagnostic, Severity};
use crate::diagnostic_catalog::{self, DIAGNOSTIC_CATALOG_SCHEMA, DiagnosticInfo};

pub const CHECK_DIAGNOSTICS_SCHEMA: &str = "hum.check.v0";

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

pub fn check_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len().saturating_sub(errors);

    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", CHECK_DIAGNOSTICS_SCHEMA, true);
    push_indent(&mut out, 2);
    out.push_str(&format!(
        "\"summary\": {{\"files\": {}, \"errors\": {}, \"warnings\": {}}},\n",
        program.files.len(),
        errors,
        warnings
    ));
    push_indent(&mut out, 2);
    if diagnostics.is_empty() {
        out.push_str("\"diagnostics\": []\n");
    } else {
        out.push_str("\"diagnostics\": [\n");
        for (index, diagnostic) in diagnostics.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_source_diagnostic(&mut out, diagnostic, 4);
        }
        out.push_str("\n  ]\n");
    }
    out.push_str("}\n");
    out
}

pub(crate) fn inject_json(mut report: String, diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return report;
    }
    let mut field = String::from("  \"pipeline_diagnostics\": [\n");
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            field.push_str(",\n");
        }
        push_source_diagnostic(&mut field, diagnostic, 4);
    }
    field.push_str("\n  ]");
    if let Some(close) = report.rfind("\n}") {
        let comma = if report[..close].trim_end().ends_with(',') {
            ""
        } else {
            ","
        };
        report.insert_str(close, &format!("{comma}\n{field}"));
    }
    report
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

fn push_source_diagnostic(out: &mut String, diagnostic: &Diagnostic, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "code", diagnostic.code.as_str(), true);
    push_string_field(out, indent + 2, "title", diagnostic.code.title(), true);
    push_string_field(
        out,
        indent + 2,
        "severity",
        diagnostic.severity.as_str(),
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "message",
        &diagnostic.message,
        diagnostic.span.is_some() || diagnostic.help.is_some(),
    );
    if let Some(span) = &diagnostic.span {
        push_indent(out, indent + 2);
        out.push_str(&format!(
            "\"span\": {{\"file\": {}, \"line\": {}, \"column\": {}}}",
            json_string(&span.file),
            span.line,
            span.column
        ));
        if !diagnostic.related_spans.is_empty() || diagnostic.help.is_some() {
            out.push(',');
        }
        out.push('\n');
    }
    if !diagnostic.related_spans.is_empty() {
        push_indent(out, indent + 2);
        out.push_str("\"related_spans\": [\n");
        for (index, related) in diagnostic.related_spans.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 4);
            out.push('{');
            out.push_str(&format!(
                "\"label\": {}, \"span\": {{\"file\": {}, \"line\": {}, \"column\": {}}}",
                json_string(&related.label),
                json_string(&related.span.file),
                related.span.line,
                related.span.column
            ));
            out.push('}');
        }
        out.push('\n');
        push_indent(out, indent + 2);
        out.push(']');
        if diagnostic.help.is_some() {
            out.push(',');
        }
        out.push('\n');
    }
    if let Some(help) = &diagnostic.help {
        push_string_field(out, indent + 2, "help", help, false);
    }
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

fn json_string(value: &str) -> String {
    let mut out = String::new();
    push_json_string(&mut out, value);
    out
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
    use super::{check_json, diagnostics_json, diagnostics_text};
    use crate::ast::Program;
    use crate::diagnostic::{Diagnostic, DiagnosticCode, Span};

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

    #[test]
    fn registry_catalog_and_check_projections_are_semantically_equivalent() {
        let catalog = crate::diagnostic_catalog::all();
        assert_eq!(catalog.len(), 87);

        let text = diagnostics_text();
        assert!(text.starts_with("Hum diagnostics (87 codes)\n"));
        let json = diagnostics_json();
        assert!(json.contains("\"count\": 87"));

        for info in catalog {
            let text_row = format!(
                "{} {} {}\n",
                info.code.as_str(),
                info.default_severity.as_str(),
                info.code.title()
            );
            assert_eq!(text.matches(&text_row).count(), 1, "{}", info.code.as_str());
            for expected in [
                format!("\"code\": {}", super::json_string(info.code.as_str())),
                format!("\"title\": {}", super::json_string(info.code.title())),
                format!(
                    "\"default_severity\": {}",
                    super::json_string(info.default_severity.as_str())
                ),
                format!("\"explanation\": {}", super::json_string(info.explanation)),
                format!("\"repair\": {}", super::json_string(info.repair)),
            ] {
                assert!(
                    json.contains(&expected),
                    "{} missing {expected}",
                    info.code.as_str()
                );
            }

            let source = Diagnostic {
                code: info.code,
                severity: info.default_severity,
                message: "projection probe".to_owned(),
                span: None,
                related_spans: Vec::new(),
                help: None,
            };
            let check = check_json(&Program::default(), &[source]);
            assert!(check.contains(&format!(
                "\"code\": {}",
                super::json_string(info.code.as_str())
            )));
            assert!(check.contains(&format!(
                "\"title\": {}",
                super::json_string(info.code.title())
            )));
        }
    }

    #[test]
    fn check_json_lists_source_diagnostics() {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UNDECLARED_SAVE_TARGET,
            "save target is not declared",
            Some(Span::new("bad.hum", 7, 5)),
        )
        .with_help("Add the target under `changes:`.");
        let program = Program::default();
        let json = check_json(&program, &[diagnostic]);

        assert!(json.contains("\"schema\": \"hum.check.v0\""));
        assert!(json.contains("\"summary\": {\"files\": 0, \"errors\": 1, \"warnings\": 0}"));
        assert!(json.contains("\"code\": \"H0201\""));
        assert!(json.contains("\"span\": {\"file\": \"bad.hum\", \"line\": 7, \"column\": 5}"));
        assert!(json.contains("\"help\": \"Add the target under `changes:`.\""));
    }
}
