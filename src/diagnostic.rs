#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticCode {
    key: crate::diagnostic_catalog::DiagnosticCodeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatedSpan {
    pub label: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
    pub related_spans: Vec<RelatedSpan>,
    pub help: Option<String>,
}

impl DiagnosticCode {
    pub(crate) const fn from_key(key: crate::diagnostic_catalog::DiagnosticCodeKey) -> Self {
        Self { key }
    }

    pub(crate) const fn key(self) -> crate::diagnostic_catalog::DiagnosticCodeKey {
        self.key
    }

    pub fn as_str(self) -> &'static str {
        crate::diagnostic_catalog::allocation(self.key).spelling
    }

    pub fn title(self) -> &'static str {
        crate::diagnostic_catalog::allocation(self.key).title
    }
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

impl Span {
    pub fn new(file: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column,
        }
    }
}

impl Diagnostic {
    pub fn error(code: DiagnosticCode, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            span,
            related_spans: Vec::new(),
            help: None,
        }
    }

    pub fn warning(code: DiagnosticCode, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            message: message.into(),
            span,
            related_spans: Vec::new(),
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_related_span(mut self, label: impl Into<String>, span: Span) -> Self {
        self.related_spans.push(RelatedSpan {
            label: label.into(),
            span,
        });
        self
    }

    pub fn render(&self) -> String {
        let mut rendered = String::new();
        match &self.span {
            Some(span) => {
                rendered.push_str(&format!(
                    "{}:{}:{}: {}[{}]: {}",
                    span.file,
                    span.line,
                    span.column,
                    self.severity.as_str(),
                    self.code.as_str(),
                    self.message
                ));
            }
            None => {
                rendered.push_str(&format!(
                    "{}[{}]: {}",
                    self.severity.as_str(),
                    self.code.as_str(),
                    self.message
                ));
            }
        }
        if let Some(help) = &self.help {
            rendered.push_str("\n  help: ");
            rendered.push_str(help);
        }
        for related in &self.related_spans {
            rendered.push_str(&format!(
                "\n  related: {} at {}:{}:{}",
                related.label, related.span.file, related.span.line, related.span.column
            ));
        }
        rendered
    }
}
#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticCode, Span};

    #[test]
    fn render_includes_stable_code() {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UNDECLARED_SAVE_TARGET,
            "declared mutation is missing",
            Some(Span::new("bad.hum", 7, 3)),
        )
        .with_help("Add the target under `changes:`.");

        let rendered = diagnostic.render();
        assert!(rendered.contains("bad.hum:7:3: error[H0201]"));
        assert!(rendered.contains("help: Add the target under `changes:`."));
    }
}
