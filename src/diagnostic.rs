#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticCode {
    value: &'static str,
    title: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
    pub help: Option<String>,
}

impl DiagnosticCode {
    pub const UNEXPECTED_TOP_LEVEL_LINE: Self = Self::new("H0001", "unexpected top-level line");
    pub const NESTED_ITEM_EXTENDS_PAST_BLOCK: Self =
        Self::new("H0002", "nested item extends past containing block");
    pub const ITEM_HEADER_MISSING_OPEN_BRACE: Self =
        Self::new("H0003", "item header missing opening brace");
    pub const ITEM_BLOCK_MISSING_CLOSE_BRACE: Self =
        Self::new("H0004", "item block missing closing brace");
    pub const UNKNOWN_ITEM_KIND: Self = Self::new("H0005", "unknown item kind");
    pub const UNEXPECTED_SIGNATURE_TEXT: Self =
        Self::new("H0006", "unexpected callable signature text");
    pub const CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN: Self =
        Self::new("H0007", "callable signature missing close parenthesis");
    pub const PARAMETER_MISSING_TYPE: Self = Self::new("H0008", "parameter missing type");
    pub const INVALID_IDENTIFIER: Self = Self::new("H0009", "invalid identifier");

    pub const APP_MISSING_WHY: Self = Self::new("H0101", "app missing why section");
    pub const TYPE_MISSING_SHAPE: Self = Self::new("H0102", "type missing shape");
    pub const STORE_MISSING_TYPE: Self = Self::new("H0103", "store missing type");
    pub const STORE_MISSING_PURPOSE: Self = Self::new("H0104", "store missing purpose");
    pub const MISSING_REQUIRED_SECTION: Self = Self::new("H0105", "item missing required section");
    pub const DUPLICATE_SECTION: Self = Self::new("H0106", "duplicate section");
    pub const TASK_MISSING_NEEDS: Self = Self::new("H0107", "task missing needs section");
    pub const SECTION_OUT_OF_ORDER: Self = Self::new("H0108", "section out of order");
    pub const TASK_MISSING_ENSURES: Self =
        Self::new("H0109", "task return missing ensures section");
    pub const HOLLOW_CONTRACT_LINE: Self = Self::new("H0110", "hollow contract line");

    pub const UNDECLARED_SAVE_TARGET: Self =
        Self::new("H0201", "save target not declared in changes");
    pub const UNDECLARED_SET_TARGET: Self = Self::new("H0202", "set target not declared mutable");

    pub const TASK_MISSING_COST: Self = Self::new("H0301", "task missing cost section");
    pub const COST_MISSING_CHECK: Self = Self::new("H0302", "cost missing check level");
    pub const COMPILE_COST_MISSING_TIME: Self =
        Self::new("H0303", "compile cost missing time claim");
    pub const CONSTANT_COST_HAS_FOR_EACH: Self =
        Self::new("H0304", "constant cost claim has iteration");
    pub const COMPILE_COST_UNBOUNDED_WHILE: Self =
        Self::new("H0305", "compile cost has unbounded-looking while");

    pub const SECURITY_MISSING_PROTECTS: Self =
        Self::new("H0401", "security-sensitive task missing protects");
    pub const TRUSTS_MISSING_PROTECTS: Self = Self::new("H0402", "trust boundary missing protects");

    pub const TEST_MISSING_COVERS: Self = Self::new("H0501", "test missing covers section");
    pub const REGRESSION_MISSING_NOTE: Self =
        Self::new("H0502", "regression test missing regression note");

    pub const UNRESOLVED_NAME: Self = Self::new("H0601", "unresolved name");
    pub const DUPLICATE_NAME_IN_SCOPE: Self = Self::new("H0602", "duplicate name in scope");
    pub const SET_TARGET_IMMUTABLE: Self = Self::new("H0603", "set target is immutable");
    pub const READ_BEFORE_DECLARE: Self = Self::new("H0604", "read before declaration");
    pub const UNKNOWN_TYPE_NAME: Self = Self::new("H0605", "unknown type name");
    pub const RETURN_TYPE_MISMATCH: Self = Self::new("H0606", "return type mismatch");
    pub const UNCHECKED_PROSE_CONTRACT: Self = Self::new("H0701", "unchecked prose contract");
    pub const NEEDS_CONTRACT_VIOLATION: Self = Self::new("H0702", "needs contract violation");
    pub const ENSURES_CONTRACT_VIOLATION: Self = Self::new("H0703", "ensures contract violation");

    pub const USE_AFTER_MOVE: Self = Self::new("H0801", "use after move");
    pub const BORROW_PARAMETER_MUTATION: Self = Self::new("H0802", "borrowed parameter written");
    pub const LINEAR_RESOURCE_NOT_CONSUMED: Self =
        Self::new("H0803", "linear resource not consumed");
    pub const LINEAR_RESOURCE_CONSUMED_TWICE: Self =
        Self::new("H0804", "linear resource consumed twice");
    pub const RETURN_DEPENDENCY_NOT_PARAMETER: Self =
        Self::new("H0805", "return dependency is not a parameter");
    pub const ITERATION_MUTATION_CONFLICT: Self = Self::new("H0806", "iteration mutation conflict");
    pub const STALE_FIELD_VIEW: Self = Self::new("H0807", "stale field view");

    pub const UNKNOWN_TARGET_FACT_RECORD: Self = Self::new("H1201", "unknown target fact record");
    pub const UNKNOWN_CAPABILITY_FAMILY: Self = Self::new("H1202", "unknown capability family");
    pub const UNSUPPORTED_TARGET_DECLARATION: Self =
        Self::new("H1203", "unsupported target declaration");
    pub const REQUIRED_CAPABILITY_UNAVAILABLE: Self =
        Self::new("H1204", "required capability unavailable on target");
    pub const CONFLICTING_TARGET_CAPABILITY: Self =
        Self::new("H1205", "conflicting target capability declaration");

    pub const fn new(value: &'static str, title: &'static str) -> Self {
        Self { value, title }
    }

    pub fn as_str(self) -> &'static str {
        self.value
    }

    pub fn title(self) -> &'static str {
        self.title
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
            help: None,
        }
    }

    pub fn warning(code: DiagnosticCode, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            message: message.into(),
            span,
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
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
