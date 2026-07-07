use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity, Span};
use crate::type_env::{self, TypeDeclaration, TypeEnvReport};
use crate::version;

pub const TYPE_CHECK_SCHEMA: &str = "hum.type_check.v0";
pub const TYPE_CHECK_MODE: &str = "declaration_annotation_check_no_expression_inference";

const NON_CLAIMS: &[&str] = &[
    "no expression type inference",
    "no body statement type checking",
    "no generic arity validation",
    "no trait or interface checking",
    "no layout or ABI proof",
    "no borrow checking",
    "no effect checking",
    "no executable semantics",
];

#[derive(Debug, Clone)]
struct TypeCheckReport {
    type_env: TypeEnvReport,
    checked_declarations: Vec<CheckedDeclaration>,
    diagnostics: Vec<TypeCheckDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeCheckSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub source_errors: usize,
    pub source_warnings: usize,
    pub resolver_errors: usize,
    pub checked_declarations: usize,
    pub accepted_declarations: usize,
    pub rejected_declarations: usize,
    pub checked_type_references: usize,
    pub unknown_type_references: usize,
    pub type_errors: usize,
    pub type_warnings: usize,
}

#[derive(Debug, Clone)]
struct CheckedDeclaration {
    id: String,
    declaration_id: String,
    declaration_kind: &'static str,
    owner_kind: &'static str,
    owner_name: String,
    name: String,
    resolver_definition_id: Option<String>,
    source_span: Span,
    type_text: String,
    type_references: Vec<CheckedTypeReference>,
    status: &'static str,
}

#[derive(Debug, Clone)]
struct CheckedTypeReference {
    text: String,
    normalized_name: String,
    role: &'static str,
    type_env_status: &'static str,
    check_status: &'static str,
}

#[derive(Debug, Clone)]
struct TypeCheckDiagnostic {
    code: DiagnosticCode,
    severity: Severity,
    title: &'static str,
    message: String,
    source_span: Span,
    help: &'static str,
    declaration_id: String,
    type_name: String,
}

pub fn type_check_has_errors(program: &crate::ast::Program, diagnostics: &[Diagnostic]) -> bool {
    let summary = type_check_summary(program, diagnostics);
    summary.source_errors > 0 || summary.resolver_errors > 0 || summary.type_errors > 0
}

pub fn type_check_summary(
    program: &crate::ast::Program,
    diagnostics: &[Diagnostic],
) -> TypeCheckSummary {
    let report = build_report(program, diagnostics);
    TypeCheckSummary {
        schema: TYPE_CHECK_SCHEMA,
        status: report.status(),
        mode: TYPE_CHECK_MODE,
        source_errors: report.source_errors(),
        source_warnings: report.type_env.source_warnings,
        resolver_errors: report.resolver_errors(),
        checked_declarations: report.checked_declarations.len(),
        accepted_declarations: report.accepted_declarations(),
        rejected_declarations: report.rejected_declarations(),
        checked_type_references: report.checked_type_references(),
        unknown_type_references: report.type_env.unknown_type_references(),
        type_errors: report.type_error_count(),
        type_warnings: report.type_warning_count(),
    }
}

pub fn type_check_text(program: &crate::ast::Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum type check ({TYPE_CHECK_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {TYPE_CHECK_MODE}\n"));
    out.push_str(&format!("status: {}\n", report.status()));
    out.push_str(&format!(
        "type_environment: schema={} status={} mode={} type_names={} declarations={} unknown_type_references={} resolver_errors={}\n",
        type_env::TYPE_ENV_SCHEMA,
        report.type_env.status(),
        type_env::TYPE_ENV_MODE,
        report.type_env.type_names.len(),
        report.type_env.declarations.len(),
        report.type_env.unknown_type_references(),
        report.resolver_errors()
    ));
    out.push_str(&format!(
        "summary: files={} items={} checked_declarations={} accepted_declarations={} rejected_declarations={} checked_type_references={} type_errors={} source_errors={} resolver_errors={}\n",
        report.type_env.files,
        report.type_env.items,
        report.checked_declarations.len(),
        report.accepted_declarations(),
        report.rejected_declarations(),
        report.checked_type_references(),
        report.type_error_count(),
        report.source_errors(),
        report.resolver_errors()
    ));

    if report.diagnostics.is_empty() {
        out.push_str("diagnostics: none\n");
    } else {
        out.push_str("diagnostics:\n");
        for diagnostic in &report.diagnostics {
            out.push_str(&format!(
                "  {}:{}:{}: {}[{}]: {}\n",
                diagnostic.source_span.file,
                diagnostic.source_span.line,
                diagnostic.source_span.column,
                diagnostic.severity.as_str(),
                diagnostic.code.as_str(),
                diagnostic.message
            ));
            out.push_str(&format!("    help: {}\n", diagnostic.help));
        }
    }

    if report.checked_declarations.is_empty() {
        out.push_str("checked_declarations: none\n");
    } else {
        out.push_str("checked_declarations:\n");
        for declaration in &report.checked_declarations {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} {} `{}`: {}\n",
                declaration.source_span.file,
                declaration.source_span.line,
                declaration.source_span.column,
                declaration.status,
                declaration.owner_kind,
                declaration.declaration_kind,
                declaration.name,
                declaration.type_text
            ));
            if !declaration.type_references.is_empty() {
                out.push_str("    type_references:");
                for reference in &declaration.type_references {
                    out.push_str(&format!(" {}[{}]", reference.text, reference.check_status));
                }
                out.push('\n');
            }
        }
    }

    out.push_str("non_claims:\n");
    for non_claim in NON_CLAIMS {
        out.push_str(&format!("  - {non_claim}\n"));
    }

    out
}

pub fn type_check_json(program: &crate::ast::Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", TYPE_CHECK_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", TYPE_CHECK_MODE, true);
    push_type_environment(&mut out, &report, 2, true);
    push_summary(&mut out, &report, 2, true);
    push_checked_declarations(&mut out, &report.checked_declarations, 2, true);
    push_diagnostics(&mut out, &report.diagnostics, 2, true);
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &crate::ast::Program, diagnostics: &[Diagnostic]) -> TypeCheckReport {
    let type_env_report = type_env::type_env_report(program, diagnostics);
    let blocked =
        type_env_report.source_errors > 0 || type_env_report.resolver_summary.resolver_errors > 0;
    let checked_declarations = type_env_report
        .declarations
        .iter()
        .map(|declaration| checked_declaration(declaration, blocked))
        .collect::<Vec<_>>();
    let diagnostics = if blocked {
        Vec::new()
    } else {
        type_diagnostics(&type_env_report.declarations)
    };

    TypeCheckReport {
        type_env: type_env_report,
        checked_declarations,
        diagnostics,
    }
}

fn checked_declaration(declaration: &TypeDeclaration, blocked: bool) -> CheckedDeclaration {
    let type_references = declaration
        .type_references
        .iter()
        .map(|reference| CheckedTypeReference {
            text: reference.text.clone(),
            normalized_name: reference.normalized_name.clone(),
            role: reference.role,
            type_env_status: reference.status,
            check_status: reference_check_status(reference.status, blocked),
        })
        .collect::<Vec<_>>();
    let status = if blocked {
        "not_checked_blocked_by_prior_errors_v0"
    } else if declaration
        .type_references
        .iter()
        .any(|reference| reference.status == "unknown_type_name_v0")
    {
        "rejected_unknown_type_name_v0"
    } else if declaration.status == "missing_type_annotation_v0" {
        "skipped_missing_type_annotation_v0"
    } else {
        "accepted_declaration_annotation_v0"
    };

    CheckedDeclaration {
        id: prefixed_id("hum_type_check_decl", &declaration.id),
        declaration_id: declaration.id.clone(),
        declaration_kind: declaration.declaration_kind,
        owner_kind: declaration.owner_kind,
        owner_name: declaration.owner_name.clone(),
        name: declaration.name.clone(),
        resolver_definition_id: declaration.resolver_definition_id.clone(),
        source_span: declaration.source_span.clone(),
        type_text: declaration.type_text.clone(),
        type_references,
        status,
    }
}

fn reference_check_status(type_env_status: &'static str, blocked: bool) -> &'static str {
    if blocked {
        "not_checked_prior_errors_v0"
    } else if type_env_status == "unknown_type_name_v0" {
        "rejected_unknown_type_name_v0"
    } else {
        "accepted_type_reference_v0"
    }
}

fn type_diagnostics(declarations: &[TypeDeclaration]) -> Vec<TypeCheckDiagnostic> {
    let mut diagnostics = Vec::new();
    for declaration in declarations {
        for reference in &declaration.type_references {
            if reference.status != "unknown_type_name_v0" {
                continue;
            }
            diagnostics.push(TypeCheckDiagnostic {
                code: DiagnosticCode::UNKNOWN_TYPE_NAME,
                severity: Severity::Error,
                title: DiagnosticCode::UNKNOWN_TYPE_NAME.title(),
                message: format!(
                    "unknown type `{}` in {} {} `{}` annotation",
                    reference.text,
                    declaration.owner_kind,
                    declaration.declaration_kind,
                    declaration.name
                ),
                source_span: declaration.source_span.clone(),
                help: "Declare the type, use a reserved type root, or wait for imports/packages before relying on external type names.",
                declaration_id: declaration.id.clone(),
                type_name: reference.text.clone(),
            });
        }
    }
    diagnostics
}

impl TypeCheckReport {
    fn status(&self) -> &'static str {
        if self.source_errors() > 0 {
            "blocked_by_source_errors"
        } else if self.resolver_errors() > 0 {
            "blocked_by_resolver_errors"
        } else if self.type_error_count() > 0 {
            "type_errors_v0"
        } else {
            "declaration_annotations_checked_v0"
        }
    }

    fn source_errors(&self) -> usize {
        self.type_env.source_errors
    }

    fn resolver_errors(&self) -> usize {
        self.type_env.resolver_summary.resolver_errors
    }

    fn type_error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .count()
    }

    fn type_warning_count(&self) -> usize {
        self.diagnostics
            .len()
            .saturating_sub(self.type_error_count())
    }

    fn accepted_declarations(&self) -> usize {
        self.checked_declarations
            .iter()
            .filter(|declaration| declaration.status == "accepted_declaration_annotation_v0")
            .count()
    }

    fn rejected_declarations(&self) -> usize {
        self.checked_declarations
            .iter()
            .filter(|declaration| declaration.status == "rejected_unknown_type_name_v0")
            .count()
    }

    fn checked_type_references(&self) -> usize {
        self.checked_declarations
            .iter()
            .map(|declaration| declaration.type_references.len())
            .sum()
    }
}

fn prefixed_id(prefix: &str, text: &str) -> String {
    let mut body = snake_identifier(text);
    if body.len() < 4 {
        body.push_str("_item");
    }
    if body.len() > 96 {
        body.truncate(96);
        body = body.trim_matches('_').to_string();
    }
    format!("{prefix}_{body}")
}

fn snake_identifier(text: &str) -> String {
    let mut out = String::new();
    let mut previous_was_separator = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !out.is_empty() {
            out.push('_');
            previous_was_separator = true;
        }
    }
    out.trim_matches('_').to_string()
}

fn push_type_environment(out: &mut String, report: &TypeCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "type_environment");
    out.push_str(": {\n");
    push_string_field(out, indent + 2, "schema", type_env::TYPE_ENV_SCHEMA, true);
    push_string_field(out, indent + 2, "status", report.type_env.status(), true);
    push_string_field(out, indent + 2, "mode", type_env::TYPE_ENV_MODE, true);
    push_usize_field(
        out,
        indent + 2,
        "type_names",
        report.type_env.type_names.len(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "declarations",
        report.type_env.declarations.len(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_references",
        report.type_env.type_reference_count(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unknown_type_references",
        report.type_env.unknown_type_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        report.resolver_errors(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_summary(out: &mut String, report: &TypeCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "summary");
    out.push_str(": {\n");
    push_usize_field(out, indent + 2, "files", report.type_env.files, true);
    push_usize_field(out, indent + 2, "items", report.type_env.items, true);
    push_usize_field(
        out,
        indent + 2,
        "source_errors",
        report.source_errors(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "source_warnings",
        report.type_env.source_warnings,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        report.resolver_errors(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_declarations",
        report.checked_declarations.len(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_declarations",
        report.accepted_declarations(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_declarations",
        report.rejected_declarations(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_type_references",
        report.checked_type_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unknown_type_references",
        report.type_env.unknown_type_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_errors",
        report.type_error_count(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_warnings",
        report.type_warning_count(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_checked_declarations(
    out: &mut String,
    declarations: &[CheckedDeclaration],
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "checked_declarations");
    out.push_str(": [");
    if !declarations.is_empty() {
        out.push('\n');
        for (index, declaration) in declarations.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_checked_declaration(out, declaration, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_checked_declaration(out: &mut String, declaration: &CheckedDeclaration, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &declaration.id, true);
    push_string_field(
        out,
        indent + 2,
        "declaration_id",
        &declaration.declaration_id,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "declaration_kind",
        declaration.declaration_kind,
        true,
    );
    push_string_field(out, indent + 2, "owner_kind", declaration.owner_kind, true);
    push_string_field(out, indent + 2, "owner_name", &declaration.owner_name, true);
    push_string_field(out, indent + 2, "name", &declaration.name, true);
    push_optional_string_field(
        out,
        indent + 2,
        "resolver_definition_id",
        declaration.resolver_definition_id.as_deref(),
        true,
    );
    push_span_field(
        out,
        indent + 2,
        "source_span",
        &declaration.source_span,
        true,
    );
    push_string_field(out, indent + 2, "type_text", &declaration.type_text, true);
    push_checked_type_references(out, &declaration.type_references, indent + 2, true);
    push_string_field(out, indent + 2, "status", declaration.status, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_checked_type_references(
    out: &mut String,
    references: &[CheckedTypeReference],
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "type_references");
    out.push_str(": [");
    if !references.is_empty() {
        out.push('\n');
        for (index, reference) in references.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            out.push_str("{\n");
            push_string_field(out, indent + 4, "text", &reference.text, true);
            push_string_field(
                out,
                indent + 4,
                "normalized_name",
                &reference.normalized_name,
                true,
            );
            push_string_field(out, indent + 4, "role", reference.role, true);
            push_string_field(
                out,
                indent + 4,
                "type_env_status",
                reference.type_env_status,
                true,
            );
            push_string_field(
                out,
                indent + 4,
                "check_status",
                reference.check_status,
                false,
            );
            push_indent(out, indent + 2);
            out.push('}');
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_diagnostics(
    out: &mut String,
    diagnostics: &[TypeCheckDiagnostic],
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "diagnostics");
    out.push_str(": [");
    if !diagnostics.is_empty() {
        out.push('\n');
        for (index, diagnostic) in diagnostics.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_diagnostic(out, diagnostic, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_diagnostic(out: &mut String, diagnostic: &TypeCheckDiagnostic, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "code", diagnostic.code.as_str(), true);
    push_string_field(out, indent + 2, "title", diagnostic.title, true);
    push_string_field(
        out,
        indent + 2,
        "severity",
        diagnostic.severity.as_str(),
        true,
    );
    push_string_field(out, indent + 2, "message", &diagnostic.message, true);
    push_span_field(
        out,
        indent + 2,
        "source_span",
        &diagnostic.source_span,
        true,
    );
    push_string_field(out, indent + 2, "help", diagnostic.help, true);
    push_string_field(
        out,
        indent + 2,
        "declaration_id",
        &diagnostic.declaration_id,
        true,
    );
    push_string_field(out, indent + 2, "type_name", &diagnostic.type_name, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_string_array(out: &mut String, indent: usize, key: &str, values: &[&str], comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(out, value);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_span_field(out: &mut String, indent: usize, key: &str, span: &Span, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": {");
    out.push_str("\"file\": ");
    push_json_string(out, &span.file);
    out.push_str(&format!(
        ", \"line\": {}, \"column\": {}",
        span.line, span.column
    ));
    out.push('}');
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

fn push_string_field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_json_string(out, value);
    push_comma_newline(out, comma);
}

fn push_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    out.push_str(&value.to_string());
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
    use crate::ast::Program;
    use crate::parser::parse_source;

    use super::{type_check_has_errors, type_check_json, type_check_text};

    #[test]
    fn json_rejects_unknown_declared_annotation_names() {
        let program = demo_program_with_unknown();
        let json = type_check_json(&program, &[]);

        assert!(type_check_has_errors(&program, &[]));
        assert!(json.contains("\"schema\": \"hum.type_check.v0\""));
        assert!(
            json.contains("\"mode\": \"declaration_annotation_check_no_expression_inference\"")
        );
        assert!(json.contains("\"schema\": \"hum.type_env.v0\""));
        assert!(json.contains("\"status\": \"type_errors_v0\""));
        assert!(json.contains("\"code\": \"H0605\""));
        assert!(json.contains("\"type_name\": \"WorkError\""));
        assert!(json.contains("\"check_status\": \"rejected_unknown_type_name_v0\""));
        assert!(json.contains("\"no expression type inference\""));
    }

    #[test]
    fn json_accepts_declared_and_reserved_annotation_names() {
        let program = demo_program_without_unknown();
        let json = type_check_json(&program, &[]);

        assert!(!type_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"declaration_annotations_checked_v0\""));
        assert!(json.contains("\"type_errors\": 0"));
        assert!(json.contains("\"accepted_declaration_annotation_v0\""));
        assert!(json.contains("\"accepted_type_reference_v0\""));
    }

    #[test]
    fn summary_reports_type_check_gate_counts() {
        let program = demo_program_with_unknown();
        let summary = super::type_check_summary(&program, &[]);

        assert_eq!(summary.schema, "hum.type_check.v0");
        assert_eq!(summary.status, "type_errors_v0");
        assert_eq!(
            summary.mode,
            "declaration_annotation_check_no_expression_inference"
        );
        assert_eq!(summary.source_errors, 0);
        assert_eq!(summary.resolver_errors, 0);
        assert_eq!(summary.type_errors, 1);
        assert_eq!(summary.unknown_type_references, 1);
        assert_eq!(summary.rejected_declarations, 1);
    }

    #[test]
    fn resolver_errors_block_type_check_authority() {
        let source = r#"task bad names() -> UInt {
  does:
    return missing
}
"#;
        let parsed = parse_source("bad.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = type_check_json(&program, &[]);

        assert!(type_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"blocked_by_resolver_errors\""));
        assert!(json.contains("\"type_errors\": 0"));
        assert!(json.contains("\"diagnostics\": []"));
    }

    #[test]
    fn text_report_names_unknown_type_diagnostics() {
        let program = demo_program_with_unknown();
        let text = type_check_text(&program, &[]);

        assert!(text.contains("Hum type check (hum.type_check.v0)"));
        assert!(text.contains("status: type_errors_v0"));
        assert!(text.contains("error[H0605]"));
        assert!(text.contains("WorkError[rejected_unknown_type_name_v0]"));
    }

    fn demo_program_with_unknown() -> Program {
        parse_program(
            r#"type WorkItem {
  title: Text
  done: Bool
}

task remember work item(title: Text) -> Result WorkItem, WorkError {
  changes:
    work items

  does:
    return title
}
"#,
        )
    }

    fn demo_program_without_unknown() -> Program {
        parse_program(
            r#"type WorkItem {
  title: Text
  done: Bool
}

type WorkError {
  code: Text
}

task remember work item(title: Text) -> Result WorkItem, WorkError {
  changes:
    work items

  does:
    return title
}
"#,
        )
    }

    fn parse_program(source: &str) -> Program {
        let parsed = parse_source("types.hum", source);
        Program {
            files: vec![parsed.file],
        }
    }
}
