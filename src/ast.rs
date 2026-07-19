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
pub enum ParsedLexicalTokenKind {
    TextQuote,
    ParenthesisOpen,
    ParenthesisClose,
    ListOpen,
    ListClose,
    RecordOpen,
    RecordClose,
    Identifier,
    IntegerLiteral,
    ComparisonOperator,
    Comma,
    Dot,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedMalformedExpressionCause {
    UnterminatedTextLiteral,
    MissingDelimiter,
    MismatchedDelimiter,
    DelimiterDepthExceeded,
    MissingOperand,
    InvalidComparisonOperator,
    InvalidOperandStarter,
    MalformedFieldPlace,
    ListElementSeparator,
    ListTrailingComma,
    ListNonTextElement,
    IntegerLiteralOutOfRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedExpectedLexicalEvidence {
    Token(ParsedLexicalTokenKind),
    Operand,
    ComparisonOperator,
    Identifier,
    ListSeparatorOrClose,
    TextListElement,
    Int64Value,
    MaximumDelimiterDepth(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedActualLexicalEvidence {
    EndOfInput,
    Token {
        kind: ParsedLexicalTokenKind,
        range: ParsedSourceRange,
    },
    DelimiterDepth(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMalformedExpressionFact {
    pub cause: ParsedMalformedExpressionCause,
    pub offending: ParsedSourceRange,
    pub expected: ParsedExpectedLexicalEvidence,
    pub actual: ParsedActualLexicalEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedLexicalStatus {
    Complete,
    Malformed(ParsedMalformedExpressionFact),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedExpressionIntent {
    Return,
    Binding,
    SetValue,
    SaveValue,
    Condition,
    LoopCollection,
    LoopRangeStart,
    LoopRangeEnd,
    Failure,
    TestExpectation,
    NeedsPredicate,
    EnsuresPredicate,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedDelimiterKind {
    Parenthesis,
    List,
    Record,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDelimiterSyntax {
    pub kind: ParsedDelimiterKind,
    pub open: ParsedSourceRange,
    pub close: Option<ParsedSourceRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCallSyntaxFacts {
    pub open: ParsedSourceRange,
    pub close: Option<ParsedSourceRange>,
    pub separators: Vec<ParsedSourceRange>,
    pub trailing: Option<ParsedSourceRange>,
    pub gaps: Vec<ParsedSourceRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCanonicalNodeSyntax {
    pub node_id: ParserSyntaxNodeId,
    pub child_position: Vec<usize>,
    pub range: ParsedSourceRange,
    pub operator: Option<ParsedSourceRange>,
    pub delimiter: Option<ParsedDelimiterSyntax>,
    pub call: Option<ParsedCallSyntaxFacts>,
    pub delimiter_depth: usize,
    pub lexical_status: ParsedLexicalStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedTypedFailureWrapperKind {
    Try,
    Wrap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTypedFailureWrapperSyntax {
    pub kind: ParsedTypedFailureWrapperKind,
    pub try_keyword: ParsedSourceRange,
    pub failure_root: Option<ParsedIdentifier>,
    pub failure_variant: Option<ParsedIdentifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedExpressionOccurrenceFacts {
    pub root_node_id: ParserSyntaxNodeId,
    pub intent: ParsedExpressionIntent,
    pub intent_signal: Option<ParsedSourceRange>,
    pub maximum_delimiter_depth: usize,
    pub lexical_status: ParsedLexicalStatus,
    pub nodes: Vec<ParsedCanonicalNodeSyntax>,
    pub typed_failure_wrapper: Option<ParsedTypedFailureWrapperSyntax>,
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
        value: Box<CanonicalExpression>,
        failure_root: Option<ParsedIdentifier>,
        failure_variant: Option<ParsedIdentifier>,
    },
    Binary {
        operator: ParsedBinaryOperator,
        left: Box<CanonicalExpression>,
        right: Box<CanonicalExpression>,
    },
    Group(Box<CanonicalExpression>),
    Unsupported,
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
    pub syntax: ParsedStatementSyntaxFacts,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedLoopKind {
    While,
    ForEach,
    ForIndex,
    Unconditional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedLoopRelationshipKind {
    CollectionIn,
    RangeUntil,
    RangeThrough,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLoopRelationshipFacts {
    pub kind: ParsedLoopRelationshipKind,
    pub binding: ParsedIdentifier,
    pub introducer: ParsedSourceRange,
    pub bound: Option<ParsedSourceRange>,
    pub expression_nodes: Vec<ParserSyntaxNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedStatementSyntaxKind {
    Return,
    Binding { mutable: bool },
    Set,
    Save,
    Condition,
    Loop { kind: ParsedLoopKind },
    Failure,
    TestExpectation,
    BlockClose,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStatementSyntaxFacts {
    pub kind: ParsedStatementSyntaxKind,
    pub keyword: ParsedSourceRange,
    pub binding: Option<ParsedIdentifier>,
    pub target: Option<ParsedExpression>,
    pub destination: Option<ParsedIdentifier>,
    pub relationship_token: Option<ParsedSourceRange>,
    pub loop_relationship: Option<ParsedLoopRelationshipFacts>,
    pub expression_nodes: Vec<ParserSyntaxNodeId>,
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
    pub occurrence: ParsedExpressionOccurrenceFacts,
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
    pub expression_syntax: Vec<Option<ParsedExpression>>,
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
