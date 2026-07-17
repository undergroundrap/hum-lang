use crate::diagnostic::Span;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParserSyntaxNodeId(String);

impl ParserSyntaxNodeId {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub(crate) fn child(&self, role: &str) -> Self {
        Self(format!("{}:{role}", self.0))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSourceRange {
    pub start: Span,
    pub byte_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedBinaryOperator {
    Multiply,
    Divide,
    Add,
    Subtract,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Is,
    Does,
    Returns,
    FailsWith,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalExpression {
    pub node_id: ParserSyntaxNodeId,
    pub range: ParsedSourceRange,
    pub kind: CanonicalExpressionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalExpressionKind {
    Unit,
    Identifier(String),
    Field {
        base: Box<CanonicalExpression>,
        field: String,
    },
    Element {
        base: Box<CanonicalExpression>,
        index: usize,
    },
    UIntLiteral(u64),
    IntLiteral(i64),
    BoolLiteral(bool),
    TextLiteral(String),
    ListLiteral(Vec<CanonicalExpression>),
    RecordLiteral {
        name: String,
        fields: Vec<(String, CanonicalExpression)>,
    },
    Call {
        callee: Box<CanonicalExpression>,
        arguments: Vec<CanonicalExpression>,
    },
    Permission {
        permission: ParamPermission,
        value: Box<CanonicalExpression>,
    },
    Try {
        call: Box<CanonicalExpression>,
        wrapper: Option<String>,
    },
    Binary {
        operator: ParsedBinaryOperator,
        operator_range: ParsedSourceRange,
        left: Box<CanonicalExpression>,
        right: Box<CanonicalExpression>,
    },
    Group(Box<CanonicalExpression>),
    Unsupported,
}

impl CanonicalExpression {
    pub(crate) fn direct_identifier(&self) -> Option<&str> {
        match &self.kind {
            CanonicalExpressionKind::Identifier(name) => Some(name),
            _ => None,
        }
    }

    pub(crate) fn direct_place(&self) -> Option<(&str, Option<&str>, Option<usize>)> {
        match &self.kind {
            CanonicalExpressionKind::Identifier(root) => Some((root, None, None)),
            CanonicalExpressionKind::Field { base, field } => {
                Some((base.direct_identifier()?, Some(field), None))
            }
            CanonicalExpressionKind::Element { base, index } => {
                Some((base.direct_identifier()?, None, Some(*index)))
            }
            _ => None,
        }
    }

    pub(crate) fn operators_in_source_order(&self) -> Vec<ParsedBinaryOperator> {
        fn collect(expression: &CanonicalExpression, out: &mut Vec<ParsedBinaryOperator>) {
            match &expression.kind {
                CanonicalExpressionKind::Binary {
                    operator,
                    left,
                    right,
                    ..
                } => {
                    collect(left, out);
                    out.push(*operator);
                    collect(right, out);
                }
                CanonicalExpressionKind::Field { base, .. }
                | CanonicalExpressionKind::Element { base, .. }
                | CanonicalExpressionKind::Group(base)
                | CanonicalExpressionKind::Permission { value: base, .. } => collect(base, out),
                CanonicalExpressionKind::Try { call, .. } => collect(call, out),
                CanonicalExpressionKind::ListLiteral(values) => {
                    for value in values {
                        collect(value, out);
                    }
                }
                CanonicalExpressionKind::RecordLiteral { fields, .. } => {
                    for (_, value) in fields {
                        collect(value, out);
                    }
                }
                CanonicalExpressionKind::Call { callee, arguments } => {
                    collect(callee, out);
                    for argument in arguments {
                        collect(argument, out);
                    }
                }
                CanonicalExpressionKind::Unit
                | CanonicalExpressionKind::Identifier(_)
                | CanonicalExpressionKind::UIntLiteral(_)
                | CanonicalExpressionKind::IntLiteral(_)
                | CanonicalExpressionKind::BoolLiteral(_)
                | CanonicalExpressionKind::TextLiteral(_)
                | CanonicalExpressionKind::Unsupported => {}
            }
        }
        let mut operators = Vec::new();
        collect(self, &mut operators);
        operators
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionLexicalCause {
    ArbitraryHelperCall,
    ChainedComparison,
    DelimiterDepthExceeded,
    IntegerLiteralOutOfRange,
    InvalidComparisonOperator,
    InvalidOperandStarter,
    KnownCallRequiresNoGap,
    ListCountWrongArity,
    ListTextLiteralRequiresTextElements,
    ListTextLiteralSeparator,
    ListTextLiteralTrailingComma,
    MalformedSyntacticPlace,
    MismatchedOrMissingDelimiter,
    MissingOperand,
    TrailingTokens,
    UnterminatedTextLiteral,
}

impl ExpressionLexicalCause {
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::ArbitraryHelperCall => "arbitrary_helper_call_not_allowed_v2",
            Self::ChainedComparison => "chained_comparison_not_supported_v0",
            Self::DelimiterDepthExceeded => "delimiter_depth_exceeded_v2",
            Self::IntegerLiteralOutOfRange => "integer_literal_out_of_range_v2",
            Self::InvalidComparisonOperator => "invalid_comparison_operator_v2",
            Self::InvalidOperandStarter => "invalid_operand_starter_v2",
            Self::KnownCallRequiresNoGap => "known_call_requires_no_gap_v2",
            Self::ListCountWrongArity => "list_count_wrong_arity_v2",
            Self::ListTextLiteralRequiresTextElements => {
                "list_text_literal_requires_text_elements_v2"
            }
            Self::ListTextLiteralSeparator => "list_text_literal_separator_v2",
            Self::ListTextLiteralTrailingComma => "list_text_literal_trailing_comma_v2",
            Self::MalformedSyntacticPlace => "malformed_syntactic_place_v2",
            Self::MismatchedOrMissingDelimiter => "mismatched_or_missing_delimiter_v2",
            Self::MissingOperand => "missing_operand_v2",
            Self::TrailingTokens => "trailing_tokens_after_predicate_v2",
            Self::UnterminatedTextLiteral => "unterminated_text_literal_v2",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedExpressionLexicalIssue {
    pub cause: ExpressionLexicalCause,
    pub range: ParsedSourceRange,
    pub expected: &'static str,
    pub actual: String,
    pub related_range: Option<ParsedSourceRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedOperatorOccurrence {
    pub node_id: ParserSyntaxNodeId,
    pub operator: ParsedBinaryOperator,
    pub range: ParsedSourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCallSyntax {
    pub node_id: ParserSyntaxNodeId,
    pub open_range: ParsedSourceRange,
    pub close_range: Option<ParsedSourceRange>,
    pub separator_ranges: Vec<ParsedSourceRange>,
    pub trailing_range: Option<ParsedSourceRange>,
    pub close_status: ParsedCallCloseStatus,
    pub trailing_status: ParsedCallTrailingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedBlockRelationship {
    None,
    Opens,
    Closes,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Program {
    pub files: Vec<SourceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub module: Option<String>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    App(App),
    Type(TypeDef),
    Store(Store),
    Task(Task),
    Test(Test),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub name: String,
    pub sections: Vec<Section>,
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub sections: Vec<Section>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    pub name: String,
    pub ty: String,
    pub sections: Vec<Section>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub name: String,
    pub params: Vec<Param>,
    pub result: Option<String>,
    pub result_syntax: Option<TypeSyntax>,
    pub sections: Vec<Section>,
    pub effect_syntax: Vec<ParsedEffectDeclaration>,
    pub body_syntax: Vec<ParsedBodyStatement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Test {
    pub name: String,
    pub params: Vec<Param>,
    pub modifiers: Vec<String>,
    pub sections: Vec<Section>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamPermission {
    Borrow,
    Change,
    Consume,
}

impl ParamPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            ParamPermission::Borrow => "borrow",
            ParamPermission::Change => "change",
            ParamPermission::Consume => "consume",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: String,
    pub type_syntax: TypeSyntax,
    pub permission: ParamPermission,
    pub permission_explicit: bool,
    pub type_hws_valid: bool,
    pub separator_hws_valid: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSyntax {
    pub kind: TypeSyntaxKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSyntaxKind {
    Named {
        name: String,
    },
    Result {
        value: Box<TypeSyntax>,
        failure_root: String,
    },
    Callable(CallableTypeSyntax),
    CallableCandidate {
        reason: &'static str,
    },
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableTypeSyntax {
    pub inputs: Vec<TypeSyntax>,
    pub result: Box<TypeSyntax>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBodyStatement {
    pub kind: ParsedBodyStatementKind,
    pub facts: ParsedStatementFacts,
    pub span: Span,
    pub source_node_id: ParserSyntaxNodeId,
    pub block_relationship: ParsedBlockRelationship,
    pub block_depth_before: usize,
    pub block_depth_after: usize,
    pub core_kind: &'static str,
    pub core_status: &'static str,
    pub core_expression_kind: Option<&'static str>,
    pub core_reason: Option<&'static str>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedStatementFacts {
    pub binding_annotation: Option<String>,
    pub set_target: Option<ParsedExpression>,
    pub save_target: Option<ParsedIdentifier>,
    pub loop_binding: Option<ParsedIdentifier>,
    pub loop_collection: Option<ParsedExpression>,
    pub condition: Option<ParsedExpression>,
    pub record_field: Option<ParsedIdentifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedBodyStatementKind {
    Return(ParsedExpression),
    Binding {
        mutable: bool,
        name: Option<ParsedIdentifier>,
        value: Option<ParsedExpression>,
    },
    Other {
        expressions: Vec<ParsedExpression>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEffectDeclaration {
    pub kind: ParsedEffectDeclarationKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedEffectDeclarationKind {
    Use,
    Change,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedExpression {
    pub kind: ParsedExpressionKind,
    pub span: Span,
    pub canonical: CanonicalExpression,
    pub intent_range: Option<ParsedSourceRange>,
    pub lexical_issue: Option<ParsedExpressionLexicalIssue>,
    pub max_delimiter_depth: usize,
    pub operator_occurrences: Vec<ParsedOperatorOccurrence>,
    pub call_syntax: Vec<ParsedCallSyntax>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedExpressionKind {
    Identifier(ParsedIdentifier),
    UIntLiteral(u64),
    Call(ParsedCall),
    Permission {
        permission: ParamPermission,
        value: Box<ParsedExpression>,
    },
    Compound {
        operands: Vec<ParsedExpression>,
    },
    Unsupported {
        reason: &'static str,
    },
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCall {
    pub callee: Box<ParsedExpression>,
    pub arguments: Vec<ParsedExpression>,
    pub argument_separators_hws_valid: bool,
    pub close_status: ParsedCallCloseStatus,
    pub trailing_status: ParsedCallTrailingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedCallCloseStatus {
    Closed,
    Missing,
    Mismatched,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedCallTrailingStatus {
    Complete,
    ExtraClose,
    Chained,
    Prose,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIdentifier {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    pub lines: Vec<SectionLine>,
    pub body_syntax: Vec<Option<ParsedBodyStatement>>,
    pub predicate_syntax: Vec<Option<ParsedExpression>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionLine {
    pub text: String,
    pub span: Span,
}

impl Item {
    pub fn kind(&self) -> &'static str {
        match self {
            Item::App(_) => "app",
            Item::Type(_) => "type",
            Item::Store(_) => "store",
            Item::Task(_) => "task",
            Item::Test(_) => "test",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Item::App(item) => &item.name,
            Item::Type(item) => &item.name,
            Item::Store(item) => &item.name,
            Item::Task(item) => &item.name,
            Item::Test(item) => &item.name,
        }
    }

    pub fn span(&self) -> &Span {
        match self {
            Item::App(item) => &item.span,
            Item::Type(item) => &item.span,
            Item::Store(item) => &item.span,
            Item::Task(item) => &item.span,
            Item::Test(item) => &item.span,
        }
    }
}

impl Task {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }
}

impl Test {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }
}

impl App {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }
}

impl TypeDef {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }
}

impl Store {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }
}

pub fn find_section<'a>(sections: &'a [Section], name: &str) -> Option<&'a Section> {
    sections.iter().find(|section| section.name == name)
}
