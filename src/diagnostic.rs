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
    pub const APP_START_MISSING: Self = Self::new("H0610", "app start missing");
    pub const APP_START_EMPTY: Self = Self::new("H0611", "app start empty");
    pub const APP_START_DUPLICATE: Self = Self::new("H0612", "app start duplicated");
    pub const APP_START_INVALID_NAME: Self = Self::new("H0613", "invalid app start name");
    pub const APP_START_NOT_CHILD: Self = Self::new("H0614", "app start task is not a child");
    pub const MULTIPLE_EXECUTABLE_APPS: Self = Self::new("H0615", "multiple executable apps");
    pub const APP_START_INVALID_RESULT: Self = Self::new("H0616", "invalid app start result");
    pub const UNKNOWN_SOURCE_CAPABILITY: Self = Self::new("H0617", "unknown source capability");
    pub const MISSING_CALLER_CAPABILITY: Self =
        Self::new("H0618", "caller capability closure is incomplete");
    pub const APP_CAPABILITY_MISMATCH: Self =
        Self::new("H0619", "app capability maximum is incomplete");
    pub const ENTRY_CAPABILITY_BYPASS: Self =
        Self::new("H0620", "direct entry cannot carry external authority");
    pub const OUTPUT_CAPABILITY_UNDECLARED: Self =
        Self::new("H0621", "stdout operation lacks source authority");
    pub const INVALID_STDOUT_WRITE_CALL: Self = Self::new("H0622", "invalid stdout_write call");
    pub const RESERVED_BUILTIN_NAME: Self = Self::new("H0623", "reserved built-in name redeclared");
    pub const OUTPUT_RECURSION_UNSUPPORTED: Self =
        Self::new("H0624", "output-reachable recursion unsupported");
    pub const REPLAY_CAPABILITY_UNDECLARED: Self =
        Self::new("H0625", "replay operation lacks source authority");
    pub const INVALID_CLOCK_REPLAY_CALL: Self =
        Self::new("H0626", "invalid clock_replay_tick call");
    pub const RESERVED_REPLAY_BUILTIN_NAME: Self =
        Self::new("H0627", "reserved replay built-in name redeclared");
    pub const REPLAY_RECURSION_UNSUPPORTED: Self =
        Self::new("H0628", "replay-reachable recursion unsupported");
    pub const INVALID_PATH_BOUNDARY: Self = Self::new("H0629", "invalid opaque Path boundary");
    pub const PATH_SOURCE_CONSTRUCTION: Self = Self::new(
        "H0630",
        "opaque Path cannot be constructed or used in source",
    );
    pub const FILE_CAPABILITY_UNDECLARED: Self =
        Self::new("H0631", "file operation lacks source authority");
    pub const INVALID_FILE_READ_CALL: Self = Self::new("H0632", "invalid files_read_text call");
    pub const RESERVED_FILE_READ_BUILTIN_NAME: Self =
        Self::new("H0633", "reserved file-read built-in name redeclared");
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
    pub const STALE_FIELD_VIEW: Self = Self::new("H0807", "stale view");
    pub const WRITABLE_ALIAS_OVERLAP: Self = Self::new("H0808", "writable alias overlap");
    pub const UNSUPPORTED_WRITABLE_ALIAS: Self = Self::new("H0809", "unsupported writable alias");

    pub const FALLIBLE_CALL_REQUIRES_TRY: Self = Self::new("H0901", "fallible call requires try");
    pub const INCOMPATIBLE_FAILURE_PROPAGATION: Self =
        Self::new("H0902", "incompatible failure propagation");
    pub const FAILURE_WRAPPER_ROOT_MISMATCH: Self =
        Self::new("H0903", "failure wrapper root mismatch");
    pub const TRY_ON_INFALLIBLE_CALL: Self = Self::new("H0904", "try on infallible call");
    pub const DIRECT_FAILURE_ROOT_MISMATCH: Self =
        Self::new("H0905", "direct failure root mismatch");
    pub const UNSUPPORTED_TRY_EXPRESSION: Self = Self::new("H0906", "unsupported try expression");
    pub const MISSING_FAILURE_DECLARATION: Self = Self::new("H0907", "missing failure declaration");

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
