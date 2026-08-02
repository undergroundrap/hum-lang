use crate::diagnostic::Span;
use std::fmt;
use std::sync::Arc;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub(crate) payload: Vec<CanonicalPayloadEvent>,
    pub(crate) completion: CanonicalCompletionEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalExpressionKind {
    Unit,
    Identifier(String),
    Field {
        base: Box<CanonicalExpression>,
        field: String,
    },
    ElementPlace {
        base: Box<CanonicalExpression>,
        index: u64,
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
        failure_root: Option<String>,
        failure_variant: Option<String>,
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
pub(crate) enum CanonicalCommonNodeKind {
    Unit,
    Identifier,
    Field,
    ElementPlace,
    UIntLiteral,
    IntLiteral,
    BoolLiteral,
    TextLiteral,
    ListLiteral,
    RecordLiteral,
    Call,
    Permission,
    Try,
    Binary,
    Group,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalCommonChildRole {
    FieldBase,
    ElementBase,
    ListElement,
    RecordFieldValue,
    CallCallee,
    CallArgument,
    PermissionValue,
    TryValue,
    BinaryLeft,
    BinaryRight,
    GroupValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalCommonLexicalStatus {
    Complete,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CanonicalLexicalTokenKind {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CanonicalMalformedCause {
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
pub(crate) enum CanonicalExpectedLexicalEvidence {
    Token(CanonicalLexicalTokenKind),
    Operand,
    ComparisonOperator,
    Identifier,
    ListSeparatorOrClose,
    TextListElement,
    Int64Value,
    MaximumDelimiterDepth(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CanonicalActualLexicalEvidence {
    EndOfInput,
    Token {
        kind: CanonicalLexicalTokenKind,
        range: ParsedSourceRange,
        spelling: String,
    },
    DelimiterDepth(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalMalformedEvent {
    pub(crate) cause: CanonicalMalformedCause,
    pub(crate) producing_event: ParsedSourceRange,
    pub(crate) offending: ParsedSourceRange,
    pub(crate) consumed: ParsedSourceRange,
    pub(crate) expected: CanonicalExpectedLexicalEvidence,
    pub(crate) actual: CanonicalActualLexicalEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CanonicalCompletionEvent {
    Complete,
    Unsupported(Box<CanonicalMalformedEvent>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalReductionChildEvent {
    pub(crate) role: CanonicalCommonChildRole,
    pub(crate) ordinal: usize,
    pub(crate) event: Box<CanonicalReductionEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalReductionEvent {
    pub(crate) range: ParsedSourceRange,
    pub(crate) kind: CanonicalCommonNodeKind,
    pub(crate) children: Vec<CanonicalReductionChildEvent>,
    pub(crate) delimiter_depth_before: usize,
    pub(crate) delimiter_depth_after: usize,
    pub(crate) lexical_status: CanonicalCommonLexicalStatus,
    pub(crate) payload: Vec<CanonicalPayloadEvent>,
    pub(crate) completion: CanonicalCompletionEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalLexicalTokenEvent {
    pub(crate) range: ParsedSourceRange,
    pub(crate) spelling: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CanonicalPayloadField {
    UnitPosition,
    IdentifierToken,
    IdentifierValue,
    UIntDigitsToken,
    UIntValue,
    IntSignToken,
    IntDigitsToken,
    IntValue,
    IntSignedLiteral,
    BoolToken,
    BoolValue,
    TextOpenQuote,
    TextCloseQuote,
    TextRawContent,
    TextEscapeEvents,
    TextDecodedValue,
    TextTerminated,
    FieldBaseEdge,
    FieldDotToken,
    FieldNameToken,
    FieldValue,
    ElementBaseEdge,
    ElementOpenBracket,
    ElementCloseBracket,
    ElementIndexToken,
    ElementIndexValue,
    ElementPlaceRole,
    DelimiterPair,
    DelimiterNestingParent,
    DelimiterSemanticGaps,
    DelimiterSeparators,
    AggregateEmpty,
    AggregateTrailing,
    GroupValueEdge,
    ListElementEdges,
    RecordNameToken,
    RecordFieldTokens,
    RecordColonTokens,
    RecordValueEdges,
    CallCalleeEdge,
    CallArgumentEdges,
    CallAdjacency,
    CallCloseState,
    CallTrailingState,
    BinaryOperator,
    BinaryOperatorTokens,
    BinaryOperatorRange,
    BinaryPrecedence,
    BinaryAssociativity,
    BinaryLeftBoundary,
    BinaryRightBoundary,
    BinaryReductionOrder,
    BinaryChildRoles,
    PermissionKeyword,
    PermissionDiscriminant,
    PermissionGap,
    PermissionValueEdge,
    TryKeyword,
    TryValueEdge,
    TryWrapperRelation,
    TryFailureRootToken,
    TryDotToken,
    TryFailureVariantToken,
    TryWrapperKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalDelimiterKind {
    Parenthesis,
    List,
    Record,
    Element,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalAssociativity {
    Left,
    #[cfg(test)]
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalTryWrapperKind {
    Propagate,
    Wrap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CanonicalPayloadEventValue {
    Position(Span),
    Token(ParsedSourceRange, String),
    Tokens(Vec<(ParsedSourceRange, String)>),
    Range(ParsedSourceRange),
    Ranges(Vec<ParsedSourceRange>),
    Text(String),
    UInt(u64),
    Int(i64),
    Bool(bool),
    Usize(usize),
    Bools(Vec<bool>),
    Parent,
    ChildOrdinal(usize),
    ChildOrdinals(Vec<usize>),
    DelimiterPair {
        kind: CanonicalDelimiterKind,
        open: ParsedSourceRange,
        close: ParsedSourceRange,
    },
    Operator(ParsedBinaryOperator),
    Permission(ParamPermission),
    Associativity(CanonicalAssociativity),
    WrapperKind(CanonicalTryWrapperKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalPayloadEvent {
    pub(crate) field: CanonicalPayloadField,
    pub(crate) value: CanonicalPayloadEventValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalExpressionRoleEvent {
    ReturnValue,
    BindingValue,
    SetValue,
    SavedValue,
    Condition,
    LoopCollection,
    LoopRangeStart,
    LoopRangeEnd,
    FailureValue,
    TestExpectation,
    NeedsPredicate,
    EnsuresPredicate,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalExpressionIntentEvent {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalOccurrenceAssignmentEvent {
    pub(crate) expression_node_id: ParserSyntaxNodeId,
    pub(crate) role: CanonicalExpressionRoleEvent,
    pub(crate) intent: CanonicalExpressionIntentEvent,
    pub(crate) predicate_recognized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedBlockRelationship {
    None,
    Opens,
    Closes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CanonicalStatementKindEvent {
    NeedsPredicate,
    EnsuresPredicate,
    Return,
    ImmutableBinding,
    MutableBinding,
    Set,
    Save,
    Fail,
    Expect,
    FreeExpression,
    If,
    While,
    ForEach,
    ForIndexUntil,
    ForIndexThrough,
    UnconditionalLoop,
    BlockClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CanonicalStatementEventField {
    Kind,
    Section,
    Line,
    Statement,
    Keyword,
    PhraseTokens,
    Binder,
    BinderRelationship,
    TypeBoundary,
    AssignmentToken,
    RelationshipToken,
    TargetRoot,
    ValueRoot,
    DestinationToken,
    StartRoot,
    EndRoot,
    OrderedRoots,
    BlockOwner,
    BlockDepthBefore,
    BlockDepthAfter,
    BlockRelationship,
    BlockOpenToken,
    BlockCloseToken,
    ExpressionAbsent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CanonicalStatementEventValue {
    Kind(CanonicalStatementKindEvent),
    Text(String),
    Range(ParsedSourceRange),
    Token {
        slot: usize,
        range: ParsedSourceRange,
        spelling: String,
    },
    Tokens(Vec<(usize, ParsedSourceRange, String)>),
    TokenReference(usize),
    Root {
        ordinal: usize,
        node: ParserSyntaxNodeId,
    },
    Roots(Vec<(usize, ParserSyntaxNodeId)>),
    Usize(usize),
    Bool(bool),
    BlockRelationship(ParsedBlockRelationship),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalStatementEventFact {
    pub(crate) field: CanonicalStatementEventField,
    pub(crate) value: CanonicalStatementEventValue,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Program {
    pub files: Vec<SourceFile>,
}

#[derive(Clone)]
pub struct SourceFile {
    pub path: String,
    pub module: Option<String>,
    pub items: Vec<Item>,
    canonical_core_file_witness: Option<CanonicalCoreFileWitness>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    App(App),
    Type(TypeDef),
    Store(Store),
    Task(Task),
    Test(Test),
}

#[derive(Clone)]
pub struct App {
    pub name: String,
    pub sections: Vec<Section>,
    pub items: Vec<Item>,
    pub span: Span,
    canonical_core_owner_witness: Option<CanonicalCoreOwnerWitness>,
}

#[derive(Clone)]
pub struct TypeDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub sections: Vec<Section>,
    pub span: Span,
    canonical_core_owner_witness: Option<CanonicalCoreOwnerWitness>,
}

#[derive(Clone)]
pub struct Store {
    pub name: String,
    pub ty: String,
    pub sections: Vec<Section>,
    pub span: Span,
    canonical_core_owner_witness: Option<CanonicalCoreOwnerWitness>,
}

#[derive(Clone)]
pub struct Task {
    pub name: String,
    pub params: Vec<Param>,
    pub result: Option<String>,
    pub result_syntax: Option<TypeSyntax>,
    pub sections: Vec<Section>,
    pub effect_syntax: Vec<ParsedEffectDeclaration>,
    pub body_syntax: Vec<ParsedBodyStatement>,
    pub span: Span,
    canonical_core_owner_witness: Option<CanonicalCoreOwnerWitness>,
}

#[derive(Clone)]
pub struct Test {
    pub name: String,
    pub params: Vec<Param>,
    pub modifiers: Vec<String>,
    pub sections: Vec<Section>,
    pub span: Span,
    canonical_core_owner_witness: Option<CanonicalCoreOwnerWitness>,
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
    pub span: Span,
    pub source_node_id: ParserSyntaxNodeId,
    pub block_relationship: ParsedBlockRelationship,
    pub block_depth_before: usize,
    pub block_depth_after: usize,
    pub core_kind: &'static str,
    pub core_status: &'static str,
    pub core_expression_kind: Option<&'static str>,
    pub core_reason: Option<&'static str>,
    pub(crate) canonical_extra_occurrences: Vec<ParsedExpression>,
    pub(crate) canonical_assignments: Vec<CanonicalOccurrenceAssignmentEvent>,
    pub(crate) canonical_statement_projection: Vec<CanonicalStatementEventFact>,
    pub(crate) canonical_statement_authority: Vec<CanonicalStatementEventFact>,
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
    pub(crate) canonical_event: CanonicalReductionEvent,
    pub(crate) canonical_tokens: Vec<CanonicalLexicalTokenEvent>,
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

#[derive(Clone)]
pub struct Section {
    pub name: String,
    pub lines: Vec<SectionLine>,
    pub body_syntax: Vec<Option<ParsedBodyStatement>>,
    pub span: Span,
    canonical_core_seal_capability: Option<CanonicalCoreSealCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionLine {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalCoreFileBinding {
    pub(crate) source_revision: Arc<[u8]>,
    pub(crate) semantic_file_index: usize,
    pub(crate) normalized_path: Arc<str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalTaskSignatureSegmentKind {
    Token,
    HorizontalSpace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalTaskSignatureSegment {
    pub(crate) kind: CanonicalTaskSignatureSegmentKind,
    pub(crate) spelling: Arc<str>,
    pub(crate) range: ParsedSourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalTaskSignatureSlice {
    pub(crate) spelling: Arc<str>,
    pub(crate) range: ParsedSourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalTaskSignatureParameterFacts {
    pub(crate) permission: Option<CanonicalTaskSignatureSlice>,
    pub(crate) name: CanonicalTaskSignatureSlice,
    pub(crate) colon: CanonicalTaskSignatureSlice,
    pub(crate) type_syntax: CanonicalTaskSignatureSlice,
    pub(crate) comma: Option<CanonicalTaskSignatureSlice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalTaskSignatureSyntaxFacts {
    pub(crate) task_keyword: CanonicalTaskSignatureSlice,
    pub(crate) task_name: CanonicalTaskSignatureSlice,
    pub(crate) open_params: Option<CanonicalTaskSignatureSlice>,
    pub(crate) params: Vec<CanonicalTaskSignatureParameterFacts>,
    pub(crate) close_params: Option<CanonicalTaskSignatureSlice>,
    pub(crate) result_arrow: Option<CanonicalTaskSignatureSlice>,
    pub(crate) result_type: Option<CanonicalTaskSignatureSlice>,
}

pub(crate) struct CanonicalTaskSignatureParserFacts {
    pub(crate) file: CanonicalCoreFileBinding,
    pub(crate) item_path: Arc<[usize]>,
    pub(crate) raw_header: Arc<str>,
    pub(crate) header_range: ParsedSourceRange,
    pub(crate) segments: Vec<CanonicalTaskSignatureSegment>,
    pub(crate) syntax: Option<CanonicalTaskSignatureSyntaxFacts>,
    pub(crate) task_name: String,
    pub(crate) task_span: Span,
    pub(crate) params: Vec<Param>,
    pub(crate) result: Option<String>,
    pub(crate) result_syntax: Option<TypeSyntax>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CanonicalTaskSignatureSnapshot {
    file: CanonicalCoreFileBinding,
    item_path: Arc<[usize]>,
    raw_header: Arc<str>,
    header_range: ParsedSourceRange,
    segments: Arc<[CanonicalTaskSignatureSegment]>,
    syntax: Option<CanonicalTaskSignatureSyntaxFacts>,
    task_name: Arc<str>,
    task_span: Span,
    params: Arc<[Param]>,
    result: Option<Arc<str>>,
    result_syntax: Option<TypeSyntax>,
}

impl fmt::Debug for CanonicalTaskSignatureSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<private parser task-signature authority>")
    }
}

fn canonical_source_revision_slice<'a>(
    source_revision: &'a [u8],
    range: &ParsedSourceRange,
) -> Result<&'a [u8], &'static str> {
    let target_line = range
        .start
        .line
        .checked_sub(1)
        .ok_or("canonical_task_signature_source_line_underflow_v0")?;
    let target_column = range
        .start
        .column
        .checked_sub(1)
        .ok_or("canonical_task_signature_source_column_underflow_v0")?;
    let source = std::str::from_utf8(source_revision)
        .map_err(|_| "canonical_task_signature_source_utf8_invalid_v0")?;
    let line = source
        .split('\n')
        .nth(target_line)
        .ok_or("canonical_task_signature_source_line_absent_v0")?;
    let line = line.strip_suffix('\r').unwrap_or(line);
    let char_count = line.chars().count();
    if target_column > char_count {
        return Err("canonical_task_signature_source_column_outside_line_v0");
    }
    let byte_column = if target_column == char_count {
        line.len()
    } else {
        line.char_indices()
            .nth(target_column)
            .map(|(offset, _)| offset)
            .ok_or("canonical_task_signature_source_column_absent_v0")?
    };
    let byte_end = byte_column
        .checked_add(range.byte_len)
        .ok_or("canonical_task_signature_source_range_overflow_v0")?;
    line.as_bytes()
        .get(byte_column..byte_end)
        .ok_or("canonical_task_signature_source_range_outside_line_v0")
}

impl CanonicalTaskSignatureSnapshot {
    pub(crate) fn from_parser_facts(
        _issuance: &crate::parser::CanonicalCoreParserIssuance,
        facts: CanonicalTaskSignatureParserFacts,
    ) -> Self {
        Self {
            file: facts.file,
            item_path: facts.item_path,
            raw_header: facts.raw_header,
            header_range: facts.header_range,
            segments: facts.segments.into(),
            syntax: facts.syntax,
            task_name: facts.task_name.into(),
            task_span: facts.task_span,
            params: facts.params.into(),
            result: facts.result.map(Into::into),
            result_syntax: facts.result_syntax,
        }
    }

    fn validate_retained_facts(
        &self,
        file: &CanonicalCoreFileBinding,
        item_path: &[usize],
    ) -> Result<(), &'static str> {
        if &self.file != file || self.item_path.as_ref() != item_path {
            return Err("canonical_task_signature_owner_binding_mismatch_v0");
        }
        if self.raw_header.is_empty() || !self.raw_header.is_ascii() {
            return Err("canonical_task_signature_header_bytes_invalid_v0");
        }
        if self.header_range.start.file != file.normalized_path.as_ref()
            && self.header_range.start.file.replace('\\', "/") != file.normalized_path.as_ref()
        {
            return Err("canonical_task_signature_header_file_mismatch_v0");
        }
        if self.header_range.byte_len != self.raw_header.len() {
            return Err("canonical_task_signature_header_length_mismatch_v0");
        }
        if canonical_source_revision_slice(file.source_revision.as_ref(), &self.header_range)?
            != self.raw_header.as_bytes()
        {
            return Err("canonical_task_signature_header_source_mismatch_v0");
        }
        let header_end = self
            .header_range
            .start
            .column
            .checked_add(self.header_range.byte_len)
            .ok_or("canonical_task_signature_header_range_overflow_v0")?;
        let mut rebuilt = String::with_capacity(self.raw_header.len());
        let mut expected_column = self.header_range.start.column;
        for segment in self.segments.iter() {
            if segment.range.start.file != self.header_range.start.file
                || segment.range.start.line != self.header_range.start.line
                || segment.range.start.column != expected_column
                || segment.range.byte_len != segment.spelling.len()
                || !segment.spelling.is_ascii()
            {
                return Err("canonical_task_signature_segment_range_mismatch_v0");
            }
            if canonical_source_revision_slice(file.source_revision.as_ref(), &segment.range)?
                != segment.spelling.as_bytes()
            {
                return Err("canonical_task_signature_segment_source_mismatch_v0");
            }
            match segment.kind {
                CanonicalTaskSignatureSegmentKind::Token => {
                    if segment
                        .spelling
                        .bytes()
                        .any(|byte| matches!(byte, b' ' | b'\t'))
                    {
                        return Err("canonical_task_signature_token_contains_gap_v0");
                    }
                }
                CanonicalTaskSignatureSegmentKind::HorizontalSpace => {
                    if segment.spelling.is_empty()
                        || segment
                            .spelling
                            .bytes()
                            .any(|byte| !matches!(byte, b' ' | b'\t'))
                    {
                        return Err("canonical_task_signature_gap_invalid_v0");
                    }
                }
            }
            expected_column = expected_column
                .checked_add(segment.range.byte_len)
                .ok_or("canonical_task_signature_segment_end_overflow_v0")?;
            if expected_column > header_end {
                return Err("canonical_task_signature_segment_outside_header_v0");
            }
            rebuilt.push_str(&segment.spelling);
        }
        if expected_column != header_end || rebuilt.as_bytes() != self.raw_header.as_bytes() {
            return Err("canonical_task_signature_segment_inventory_mismatch_v0");
        }

        let syntax = self
            .syntax
            .as_ref()
            .ok_or("canonical_task_signature_syntax_unavailable_v0")?;
        if syntax.task_keyword.spelling.as_ref() != "task"
            || syntax.task_name.spelling.as_ref() != self.task_name.as_ref()
            || syntax.params.len() != self.params.len()
            || syntax.open_params.is_some() != syntax.close_params.is_some()
            || (!self.params.is_empty() && syntax.open_params.is_none())
            || syntax.result_arrow.is_some() != self.result.is_some()
            || syntax.result_type.is_some() != self.result.is_some()
            || self.result_syntax.is_some() != self.result.is_some()
        {
            return Err("canonical_task_signature_syntax_projection_mismatch_v0");
        }
        if syntax
            .open_params
            .as_ref()
            .is_some_and(|value| value.spelling.as_ref() != "(")
            || syntax
                .close_params
                .as_ref()
                .is_some_and(|value| value.spelling.as_ref() != ")")
        {
            return Err("canonical_task_signature_delimiter_projection_mismatch_v0");
        }
        if self.task_span != self.header_range.start {
            return Err("canonical_task_signature_task_span_mismatch_v0");
        }

        let mut slices = Vec::new();
        slices.push(&syntax.task_keyword);
        slices.push(&syntax.task_name);
        if let Some(open) = &syntax.open_params {
            slices.push(open);
        }
        for (ordinal, (retained, projected)) in
            syntax.params.iter().zip(self.params.iter()).enumerate()
        {
            if retained.permission.is_some() != projected.permission_explicit
                || retained.name.spelling.as_ref() != projected.name
                || retained.colon.spelling.as_ref() != ":"
                || retained.type_syntax.spelling.as_ref() != projected.ty
                || retained.comma.is_some() != (ordinal + 1 < self.params.len())
            {
                return Err("canonical_task_signature_parameter_projection_mismatch_v0");
            }
            let first = retained.permission.as_ref().unwrap_or(&retained.name);
            if projected.span != first.range.start
                || projected.type_syntax.span != retained.type_syntax.range.start
            {
                return Err("canonical_task_signature_parameter_range_mismatch_v0");
            }
            let (_, colon_end) = self.slice_offsets(&retained.colon)?;
            let (type_start, _) = self.slice_offsets(&retained.type_syntax)?;
            if projected.type_hws_valid != (type_start > colon_end) {
                return Err("canonical_task_signature_type_gap_mismatch_v0");
            }
            if ordinal > 0 {
                let previous_comma = syntax.params[ordinal - 1]
                    .comma
                    .as_ref()
                    .ok_or("canonical_task_signature_separator_missing_v0")?;
                let (_, comma_end) = self.slice_offsets(previous_comma)?;
                let (parameter_start, _) = self.slice_offsets(first)?;
                if projected.separator_hws_valid != (parameter_start > comma_end) {
                    return Err("canonical_task_signature_separator_gap_mismatch_v0");
                }
            } else if !projected.separator_hws_valid {
                return Err("canonical_task_signature_first_separator_mismatch_v0");
            }
            if let Some(permission) = &retained.permission {
                if permission.spelling.as_ref() != projected.permission.as_str() {
                    return Err("canonical_task_signature_permission_projection_mismatch_v0");
                }
                slices.push(permission);
            }
            slices.push(&retained.name);
            slices.push(&retained.colon);
            slices.push(&retained.type_syntax);
            if let Some(comma) = &retained.comma {
                if comma.spelling.as_ref() != "," {
                    return Err("canonical_task_signature_comma_projection_mismatch_v0");
                }
                slices.push(comma);
            }
        }
        if let Some(close) = &syntax.close_params {
            slices.push(close);
        }
        if let Some(arrow) = &syntax.result_arrow {
            if arrow.spelling.as_ref() != "->" {
                return Err("canonical_task_signature_arrow_projection_mismatch_v0");
            }
            slices.push(arrow);
        }
        if let Some(result_type) = &syntax.result_type {
            if self.result.as_deref() != Some(result_type.spelling.as_ref()) {
                return Err("canonical_task_signature_result_projection_mismatch_v0");
            }
            if self.result_syntax.as_ref().map(|syntax| &syntax.span)
                != Some(&result_type.range.start)
            {
                return Err("canonical_task_signature_result_range_mismatch_v0");
            }
            slices.push(result_type);
        }

        let mut prior_end = 0usize;
        for (ordinal, slice) in slices.iter().enumerate() {
            let (start, end) = self.slice_offsets(slice)?;
            if (ordinal == 0 && start != 0) || start < prior_end {
                return Err("canonical_task_signature_slice_order_mismatch_v0");
            }
            if self.raw_header.get(start..end) != Some(slice.spelling.as_ref()) {
                return Err("canonical_task_signature_slice_spelling_mismatch_v0");
            }
            if canonical_source_revision_slice(file.source_revision.as_ref(), &slice.range)?
                != slice.spelling.as_bytes()
            {
                return Err("canonical_task_signature_slice_source_mismatch_v0");
            }
            prior_end = end;
        }
        if prior_end != self.raw_header.len() {
            return Err("canonical_task_signature_trailing_token_mismatch_v0");
        }

        for segment in self
            .segments
            .iter()
            .filter(|segment| segment.kind == CanonicalTaskSignatureSegmentKind::Token)
        {
            let segment_start = segment
                .range
                .start
                .column
                .checked_sub(self.header_range.start.column)
                .ok_or("canonical_task_signature_segment_start_underflow_v0")?;
            let segment_end = segment_start
                .checked_add(segment.range.byte_len)
                .ok_or("canonical_task_signature_segment_end_overflow_v0")?;
            let covered = slices.iter().any(|slice| {
                self.slice_offsets(slice)
                    .is_ok_and(|(start, end)| start <= segment_start && segment_end <= end)
            });
            if !covered {
                return Err("canonical_task_signature_extra_token_v0");
            }
        }
        Ok(())
    }

    fn slice_offsets(
        &self,
        slice: &CanonicalTaskSignatureSlice,
    ) -> Result<(usize, usize), &'static str> {
        if slice.range.start.file != self.header_range.start.file
            || slice.range.start.line != self.header_range.start.line
            || slice.range.byte_len != slice.spelling.len()
            || !slice.spelling.is_ascii()
        {
            return Err("canonical_task_signature_slice_range_mismatch_v0");
        }
        let start = slice
            .range
            .start
            .column
            .checked_sub(self.header_range.start.column)
            .ok_or("canonical_task_signature_slice_start_underflow_v0")?;
        let end = start
            .checked_add(slice.range.byte_len)
            .ok_or("canonical_task_signature_slice_end_overflow_v0")?;
        if end > self.header_range.byte_len {
            return Err("canonical_task_signature_slice_outside_header_v0");
        }
        Ok((start, end))
    }

    fn matches_live_task(&self, task: &Task) -> Result<(), &'static str> {
        if self.task_name.as_ref() != task.name
            || self.task_span != task.span
            || self.params.as_ref() != task.params.as_slice()
            || self.result.as_deref() != task.result.as_deref()
            || self.result_syntax != task.result_syntax
        {
            return Err("canonical_task_signature_live_projection_mismatch_v0");
        }
        Ok(())
    }

    fn matches_lowered_candidate(
        &self,
        kind: &str,
        name: &str,
        span: &Span,
        params: &[Param],
        result: Option<&str>,
    ) -> bool {
        kind == "task"
            && self
                .validate_retained_facts(&self.file, &self.item_path)
                .is_ok()
            && self.task_name.as_ref() == name
            && self.task_span.file.replace('\\', "/") == span.file.replace('\\', "/")
            && self.task_span.line == span.line
            && self.task_span.column == span.column
            && self.params.as_ref() == params
            && self.result.as_deref() == result
    }
}

pub(crate) struct AuthenticatedCanonicalTaskSignature {
    snapshot: CanonicalTaskSignatureSnapshot,
}

impl AuthenticatedCanonicalTaskSignature {
    pub(crate) fn matches_lowered_candidate(
        &self,
        kind: &str,
        name: &str,
        span: &Span,
        params: &[Param],
        result: Option<&str>,
    ) -> bool {
        self.snapshot
            .matches_lowered_candidate(kind, name, span, params, result)
    }
}

pub(crate) struct CanonicalTaskSignatureRejection(&'static str);

impl CanonicalTaskSignatureRejection {
    fn new(reason: &'static str) -> Self {
        Self(reason)
    }

    pub(crate) fn reason(&self) -> &'static str {
        self.0
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum CanonicalTaskSignatureCorruption {
    Missing,
    ResultRangeRelocated,
    SameSpelledBodyRelocation,
    CoherentRangeRelocation,
    CoherentTaskNameSubstitution,
    CoherentParameterNameSubstitution,
    CoherentParameterTypeSubstitution,
    CoherentResultSubstitution,
    ParameterOmission,
    ParameterDuplication,
    ParameterReorder,
    PermissionSubstitution,
    NameSubstitution,
    TypeSubstitution,
    ResultOrderSubstitution,
    ForeignTask,
    ForeignRevision,
    OverlappingRanges,
    DuplicatedRange,
    AbsentRange,
    ExtraRange,
    ImpossibleEnd,
    Overflow,
    Underflow,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CanonicalCoreOwnerBinding {
    pub(crate) file: CanonicalCoreFileBinding,
    pub(crate) item_path: Arc<[usize]>,
    pub(crate) item_kind: &'static str,
    pub(crate) section_slots: Arc<[Arc<str>]>,
    task_signature: Option<CanonicalTaskSignatureSnapshot>,
}

impl fmt::Debug for CanonicalCoreOwnerBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalCoreOwnerBinding")
            .field("file", &self.file)
            .field("item_path", &self.item_path)
            .field("item_kind", &self.item_kind)
            .field("section_slots", &self.section_slots)
            .field(
                "task_signature",
                &self.task_signature.as_ref().map(|_| "<private>"),
            )
            .finish()
    }
}

impl CanonicalCoreOwnerBinding {
    pub(crate) fn from_parser_parts(
        _issuance: &crate::parser::CanonicalCoreParserIssuance,
        file: CanonicalCoreFileBinding,
        item_path: Arc<[usize]>,
        item_kind: &'static str,
        section_slots: Arc<[Arc<str>]>,
        task_signature: Option<CanonicalTaskSignatureSnapshot>,
    ) -> Self {
        Self {
            file,
            item_path,
            item_kind,
            section_slots,
            task_signature,
        }
    }

    #[cfg(test)]
    pub(crate) fn corrupt_task_signature_for_test(
        &self,
        corruption: CanonicalTaskSignatureCorruption,
    ) -> Self {
        let mut corrupted = self.clone();
        if matches!(corruption, CanonicalTaskSignatureCorruption::Missing) {
            corrupted.task_signature = None;
            return corrupted;
        }
        let snapshot = corrupted
            .task_signature
            .as_mut()
            .expect("task-signature corruption requires a task snapshot");
        match corruption {
            CanonicalTaskSignatureCorruption::Missing => unreachable!(),
            CanonicalTaskSignatureCorruption::ResultRangeRelocated => {
                let result = snapshot
                    .syntax
                    .as_mut()
                    .and_then(|syntax| syntax.result_type.as_mut())
                    .expect("result range");
                result.range.start.line = result.range.start.line.saturating_add(3);
            }
            CanonicalTaskSignatureCorruption::SameSpelledBodyRelocation => {
                let result_spelling = snapshot
                    .syntax
                    .as_ref()
                    .and_then(|syntax| syntax.result_type.as_ref())
                    .expect("result range")
                    .spelling
                    .clone();
                let relocated = matching_body_text_span(snapshot, &result_spelling);
                snapshot
                    .syntax
                    .as_mut()
                    .and_then(|syntax| syntax.result_type.as_mut())
                    .expect("result range")
                    .range
                    .start = relocated;
            }
            CanonicalTaskSignatureCorruption::CoherentRangeRelocation => {
                shift_task_signature_snapshot(snapshot, 0, 1);
            }
            CanonicalTaskSignatureCorruption::CoherentTaskNameSubstitution => {
                let range = snapshot
                    .syntax
                    .as_ref()
                    .expect("task syntax")
                    .task_name
                    .range
                    .clone();
                coherently_replace_task_signature_slice(snapshot, &range, "foreign_auth");
                snapshot
                    .syntax
                    .as_mut()
                    .expect("task syntax")
                    .task_name
                    .spelling = Arc::from("foreign_auth");
                snapshot.task_name = Arc::from("foreign_auth");
            }
            CanonicalTaskSignatureCorruption::CoherentParameterNameSubstitution => {
                let range = snapshot.syntax.as_ref().expect("task syntax").params[0]
                    .name
                    .range
                    .clone();
                coherently_replace_task_signature_slice(snapshot, &range, "west");
                snapshot.syntax.as_mut().expect("task syntax").params[0]
                    .name
                    .spelling = Arc::from("west");
                let params = Arc::make_mut(&mut snapshot.params);
                params[0].name = "west".to_string();
            }
            CanonicalTaskSignatureCorruption::CoherentParameterTypeSubstitution => {
                let range = snapshot.syntax.as_ref().expect("task syntax").params[0]
                    .type_syntax
                    .range
                    .clone();
                coherently_replace_task_signature_slice(snapshot, &range, "Txt");
                snapshot.syntax.as_mut().expect("task syntax").params[0]
                    .type_syntax
                    .spelling = Arc::from("Txt");
                let params = Arc::make_mut(&mut snapshot.params);
                params[0].ty = "Txt".to_string();
                let TypeSyntaxKind::Named { name } = &mut params[0].type_syntax.kind else {
                    panic!("coherent parameter-type probe requires a named type")
                };
                *name = "Txt".to_string();
            }
            CanonicalTaskSignatureCorruption::CoherentResultSubstitution => {
                let range = snapshot
                    .syntax
                    .as_ref()
                    .and_then(|syntax| syntax.result_type.as_ref())
                    .expect("result syntax")
                    .range
                    .clone();
                coherently_replace_task_signature_slice(snapshot, &range, "Txt");
                snapshot
                    .syntax
                    .as_mut()
                    .and_then(|syntax| syntax.result_type.as_mut())
                    .expect("result syntax")
                    .spelling = Arc::from("Txt");
                snapshot.result = Some(Arc::from("Txt"));
                let TypeSyntaxKind::Named { name } = &mut snapshot
                    .result_syntax
                    .as_mut()
                    .expect("result type syntax")
                    .kind
                else {
                    panic!("coherent result probe requires a named type")
                };
                *name = "Txt".to_string();
            }
            CanonicalTaskSignatureCorruption::ParameterOmission => {
                let mut params = snapshot.params.to_vec();
                params.pop();
                snapshot.params = params.into();
                snapshot.syntax.as_mut().expect("task syntax").params.pop();
            }
            CanonicalTaskSignatureCorruption::ParameterDuplication => {
                let mut params = snapshot.params.to_vec();
                params.push(params[0].clone());
                snapshot.params = params.into();
                let syntax = snapshot.syntax.as_mut().expect("task syntax");
                syntax.params.push(syntax.params[0].clone());
            }
            CanonicalTaskSignatureCorruption::ParameterReorder => {
                let mut params = snapshot.params.to_vec();
                params.swap(0, 1);
                snapshot.params = params.into();
                snapshot
                    .syntax
                    .as_mut()
                    .expect("task syntax")
                    .params
                    .swap(0, 1);
            }
            CanonicalTaskSignatureCorruption::PermissionSubstitution => {
                let mut params = snapshot.params.to_vec();
                params[0].permission = if params[0].permission == ParamPermission::Borrow {
                    ParamPermission::Change
                } else {
                    ParamPermission::Borrow
                };
                snapshot.params = params.into();
            }
            CanonicalTaskSignatureCorruption::NameSubstitution => {
                let mut params = snapshot.params.to_vec();
                params[0].name.push_str("_foreign");
                snapshot.params = params.into();
            }
            CanonicalTaskSignatureCorruption::TypeSubstitution => {
                let mut params = snapshot.params.to_vec();
                params[0].ty = "ForeignType".to_string();
                snapshot.params = params.into();
            }
            CanonicalTaskSignatureCorruption::ResultOrderSubstitution => {
                let syntax = snapshot.syntax.as_mut().expect("task syntax");
                std::mem::swap(&mut syntax.result_arrow, &mut syntax.result_type);
            }
            CanonicalTaskSignatureCorruption::ForeignTask => {
                snapshot.task_name = Arc::from("foreign_task");
            }
            CanonicalTaskSignatureCorruption::ForeignRevision => {
                snapshot.file.source_revision = Arc::from(&b"foreign-revision"[..]);
            }
            CanonicalTaskSignatureCorruption::OverlappingRanges => {
                let segments = Arc::make_mut(&mut snapshot.segments);
                segments[1].range.start.column = segments[0].range.start.column;
            }
            CanonicalTaskSignatureCorruption::DuplicatedRange => {
                let segments = Arc::make_mut(&mut snapshot.segments);
                segments[1].range = segments[0].range.clone();
            }
            CanonicalTaskSignatureCorruption::AbsentRange => {
                let mut segments = snapshot.segments.to_vec();
                segments.remove(1);
                snapshot.segments = segments.into();
            }
            CanonicalTaskSignatureCorruption::ExtraRange => {
                let mut segments = snapshot.segments.to_vec();
                segments.push(segments[0].clone());
                snapshot.segments = segments.into();
            }
            CanonicalTaskSignatureCorruption::ImpossibleEnd => {
                let segments = Arc::make_mut(&mut snapshot.segments);
                segments[0].range.byte_len = segments[0]
                    .range
                    .byte_len
                    .saturating_add(snapshot.raw_header.len());
            }
            CanonicalTaskSignatureCorruption::Overflow => {
                snapshot.header_range.start.column = usize::MAX;
            }
            CanonicalTaskSignatureCorruption::Underflow => {
                snapshot.header_range.start.column = snapshot
                    .segments
                    .first()
                    .expect("signature segment")
                    .range
                    .start
                    .column
                    .saturating_add(1);
            }
        }
        corrupted
    }
}

#[cfg(test)]
fn matching_body_text_span(snapshot: &CanonicalTaskSignatureSnapshot, spelling: &str) -> Span {
    let source = std::str::from_utf8(snapshot.file.source_revision.as_ref())
        .expect("test source revision is UTF-8");
    source
        .split('\n')
        .enumerate()
        .skip(snapshot.header_range.start.line)
        .find_map(|(line_index, line)| {
            let byte_column = line.find(spelling)?;
            Some(Span::new(
                snapshot.header_range.start.file.clone(),
                line_index
                    .checked_add(1)
                    .expect("test line does not overflow"),
                line[..byte_column]
                    .chars()
                    .count()
                    .checked_add(1)
                    .expect("test column does not overflow"),
            ))
        })
        .expect("test source contains same-spelled body text")
}

#[cfg(test)]
fn coherently_replace_task_signature_slice(
    snapshot: &mut CanonicalTaskSignatureSnapshot,
    range: &ParsedSourceRange,
    replacement: &str,
) {
    let start = range
        .start
        .column
        .checked_sub(snapshot.header_range.start.column)
        .expect("coherent substitution range starts inside the header");
    let end = start
        .checked_add(range.byte_len)
        .expect("coherent substitution range does not overflow");
    assert_eq!(range.byte_len, replacement.len());
    let mut raw_header = snapshot.raw_header.to_string();
    raw_header.replace_range(start..end, replacement);
    snapshot.raw_header = Arc::from(raw_header);

    let segment = Arc::make_mut(&mut snapshot.segments)
        .iter_mut()
        .find(|segment| segment.range == *range)
        .expect("coherent substitution targets one retained token segment");
    segment.spelling = Arc::from(replacement);
}

#[cfg(test)]
fn shift_task_signature_snapshot(
    snapshot: &mut CanonicalTaskSignatureSnapshot,
    line_delta: usize,
    column_delta: usize,
) {
    fn shift_range(range: &mut ParsedSourceRange, line_delta: usize, column_delta: usize) {
        range.start.line = range.start.line.saturating_add(line_delta);
        range.start.column = range.start.column.saturating_add(column_delta);
    }
    shift_range(&mut snapshot.header_range, line_delta, column_delta);
    for segment in Arc::make_mut(&mut snapshot.segments) {
        shift_range(&mut segment.range, line_delta, column_delta);
    }
    let Some(syntax) = snapshot.syntax.as_mut() else {
        return;
    };
    for slice in [&mut syntax.task_keyword, &mut syntax.task_name] {
        shift_range(&mut slice.range, line_delta, column_delta);
    }
    if let Some(open) = &mut syntax.open_params {
        shift_range(&mut open.range, line_delta, column_delta);
    }
    if let Some(close) = &mut syntax.close_params {
        shift_range(&mut close.range, line_delta, column_delta);
    }
    for parameter in &mut syntax.params {
        if let Some(permission) = &mut parameter.permission {
            shift_range(&mut permission.range, line_delta, column_delta);
        }
        shift_range(&mut parameter.name.range, line_delta, column_delta);
        shift_range(&mut parameter.colon.range, line_delta, column_delta);
        shift_range(&mut parameter.type_syntax.range, line_delta, column_delta);
        if let Some(comma) = &mut parameter.comma {
            shift_range(&mut comma.range, line_delta, column_delta);
        }
    }
    if let Some(arrow) = &mut syntax.result_arrow {
        shift_range(&mut arrow.range, line_delta, column_delta);
    }
    if let Some(result_type) = &mut syntax.result_type {
        shift_range(&mut result_type.range, line_delta, column_delta);
    }
}

pub(crate) trait CanonicalCoreFileVerifier: Send + Sync {
    fn binding(&self) -> &CanonicalCoreFileBinding;
}

pub(crate) trait CanonicalCoreOwnerVerifier: Send + Sync {
    fn binding(&self) -> &CanonicalCoreOwnerBinding;

    #[cfg(test)]
    fn corrupt_task_signature_for_test(
        &self,
        corruption: CanonicalTaskSignatureCorruption,
    ) -> Arc<dyn CanonicalCoreOwnerVerifier>;
}

pub(crate) trait CanonicalCoreSectionVerifier: Send + Sync {
    fn validate(
        &self,
        file: &CanonicalCoreFileBinding,
        owner: &CanonicalCoreOwnerBinding,
        section_slot: usize,
        section: &Section,
    ) -> Result<(), &'static str>;

    #[cfg(test)]
    fn corrupt_retained_authority_for_test(
        &self,
        domain: CanonicalCoreRetainedAuthorityDomain,
    ) -> Arc<dyn CanonicalCoreSectionVerifier>;
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum CanonicalCoreRetainedAuthorityDomain {
    SourceOwner,
    Occurrence,
    Statement,
}

pub(crate) trait CanonicalCoreParseContextVerifier: Send + Sync {
    fn binding(&self) -> &CanonicalCoreFileBinding;
}

#[derive(Clone)]
pub(crate) struct CanonicalCoreFileWitness(Arc<dyn CanonicalCoreFileVerifier>);

#[derive(Clone)]
pub(crate) struct CanonicalCoreOwnerWitness(Arc<dyn CanonicalCoreOwnerVerifier>);

#[derive(Clone)]
pub(crate) struct CanonicalCoreSealCapability(Arc<dyn CanonicalCoreSectionVerifier>);

#[derive(Clone)]
pub(crate) struct CanonicalCoreParseContext(Arc<dyn CanonicalCoreParseContextVerifier>);

impl CanonicalCoreFileWitness {
    pub(crate) fn parser_issue(
        _issuance: &crate::parser::CanonicalCoreParserIssuance,
        verifier: Arc<dyn CanonicalCoreFileVerifier>,
    ) -> Self {
        Self(verifier)
    }

    pub(crate) fn binding(&self) -> &CanonicalCoreFileBinding {
        self.0.binding()
    }
}

impl CanonicalCoreOwnerWitness {
    pub(crate) fn parser_issue(
        _issuance: &crate::parser::CanonicalCoreParserIssuance,
        verifier: Arc<dyn CanonicalCoreOwnerVerifier>,
    ) -> Self {
        Self(verifier)
    }

    fn binding(&self) -> &CanonicalCoreOwnerBinding {
        self.0.binding()
    }
}

impl CanonicalCoreSealCapability {
    pub(crate) fn parser_issue(
        _issuance: &crate::parser::CanonicalCoreParserIssuance,
        verifier: Arc<dyn CanonicalCoreSectionVerifier>,
    ) -> Self {
        Self(verifier)
    }
}

impl CanonicalCoreParseContext {
    pub(crate) fn parser_issue(
        _issuance: &crate::parser::CanonicalCoreParserIssuance,
        verifier: Arc<dyn CanonicalCoreParseContextVerifier>,
    ) -> Self {
        Self(verifier)
    }

    pub(crate) fn binding(&self) -> &CanonicalCoreFileBinding {
        self.0.binding()
    }
}

#[allow(unexpected_cfgs)]
mod canonical_core_owner_foreign_issue_compile_proof {
    #[cfg(hum_compile_fail_canonical_core_owner_foreign_issue)]
    fn canonical_core_owner_foreign_issue_must_not_compile() {
        let _ = crate::parser::CanonicalCoreParserIssuance::new(); // canonical_core_owner_foreign_issue_must_not_compile
    }
}

impl fmt::Debug for CanonicalCoreFileWitness {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<private parser authority>")
    }
}

impl fmt::Debug for CanonicalCoreOwnerWitness {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<private parser authority>")
    }
}

impl fmt::Debug for CanonicalCoreSealCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<private parser authority>")
    }
}

impl fmt::Debug for CanonicalCoreParseContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<private parser context>")
    }
}

#[derive(Clone, Copy)]
enum CanonicalCoreContainerRef<'a> {
    Parse {
        file: &'a SourceFile,
        context: &'a CanonicalCoreParseContext,
    },
    Program(&'a Program),
}

pub(crate) struct CanonicalCoreSectionExpectation<'a> {
    container: CanonicalCoreContainerRef<'a>,
    file: &'a SourceFile,
    item: &'a Item,
    section: &'a Section,
    file_ordinal: usize,
    item_path: Vec<usize>,
    section_slot: usize,
}

pub(crate) struct ValidatedCoreSection<'a> {
    section: &'a Section,
}

impl<'a> ValidatedCoreSection<'a> {
    pub(crate) fn section(&self) -> &'a Section {
        self.section
    }
}

impl SourceFile {
    pub(crate) fn parser_new(
        path: String,
        module: Option<String>,
        items: Vec<Item>,
        witness: CanonicalCoreFileWitness,
    ) -> Self {
        Self {
            path,
            module,
            items,
            canonical_core_file_witness: Some(witness),
        }
    }

    pub(crate) fn empty_non_authoritative(
        path: String,
        module: Option<String>,
        items: Vec<Item>,
    ) -> Self {
        debug_assert!(items.is_empty());
        Self {
            path,
            module,
            items,
            canonical_core_file_witness: None,
        }
    }

    fn canonical_core_file_witness(&self) -> Result<&CanonicalCoreFileWitness, &'static str> {
        self.canonical_core_file_witness
            .as_ref()
            .ok_or("canonical_core_file_witness_absent_v0")
    }

    #[cfg(test)]
    pub(crate) fn corrupt_canonical_core_file_witness_from(&mut self, foreign: &SourceFile) {
        self.canonical_core_file_witness = foreign.canonical_core_file_witness.clone();
    }

    #[cfg(test)]
    pub(crate) fn remove_canonical_core_file_witness(&mut self) {
        self.canonical_core_file_witness = None;
    }
}

macro_rules! impl_item_authority_constructor {
    ($name:ident { $($field:ident : $type:ty),* $(,)? }) => {
        impl $name {
            #[allow(clippy::too_many_arguments)]
            pub(crate) fn parser_new(
                $($field: $type,)*
                canonical_core_owner_witness: CanonicalCoreOwnerWitness,
            ) -> Self {
                Self { $($field,)* canonical_core_owner_witness: Some(canonical_core_owner_witness) }
            }
        }
    };
}

impl_item_authority_constructor!(App {
    name: String,
    sections: Vec<Section>,
    items: Vec<Item>,
    span: Span,
});
impl_item_authority_constructor!(TypeDef {
    name: String,
    fields: Vec<Field>,
    sections: Vec<Section>,
    span: Span,
});
impl_item_authority_constructor!(Store {
    name: String,
    ty: String,
    sections: Vec<Section>,
    span: Span,
});
impl_item_authority_constructor!(Task {
    name: String,
    params: Vec<Param>,
    result: Option<String>,
    result_syntax: Option<TypeSyntax>,
    sections: Vec<Section>,
    effect_syntax: Vec<ParsedEffectDeclaration>,
    body_syntax: Vec<ParsedBodyStatement>,
    span: Span,
});
impl_item_authority_constructor!(Test {
    name: String,
    params: Vec<Param>,
    modifiers: Vec<String>,
    sections: Vec<Section>,
    span: Span,
});

impl Section {
    pub(crate) fn parser_new(
        name: String,
        lines: Vec<SectionLine>,
        body_syntax: Vec<Option<ParsedBodyStatement>>,
        span: Span,
        capability: CanonicalCoreSealCapability,
    ) -> Self {
        Self {
            name,
            lines,
            body_syntax,
            span,
            canonical_core_seal_capability: Some(capability),
        }
    }

    fn canonical_core_seal_capability(&self) -> Result<&CanonicalCoreSealCapability, &'static str> {
        self.canonical_core_seal_capability
            .as_ref()
            .ok_or("canonical_core_section_capability_absent_v0")
    }

    #[cfg(test)]
    pub(crate) fn corrupt_canonical_core_capability_from(&mut self, foreign: &Section) {
        self.canonical_core_seal_capability = foreign.canonical_core_seal_capability.clone();
    }

    #[cfg(test)]
    pub(crate) fn corrupt_canonical_core_retained_authority(
        &mut self,
        domain: CanonicalCoreRetainedAuthorityDomain,
    ) {
        let capability = self
            .canonical_core_seal_capability
            .as_ref()
            .expect("parser-produced Section capability");
        self.canonical_core_seal_capability = Some(CanonicalCoreSealCapability(
            capability.0.corrupt_retained_authority_for_test(domain),
        ));
    }

    #[cfg(test)]
    pub(crate) fn remove_canonical_core_capability(&mut self) {
        self.canonical_core_seal_capability = None;
    }
}

impl Item {
    fn canonical_core_owner_witness(&self) -> Result<&CanonicalCoreOwnerWitness, &'static str> {
        let witness = match self {
            Item::App(item) => &item.canonical_core_owner_witness,
            Item::Type(item) => &item.canonical_core_owner_witness,
            Item::Store(item) => &item.canonical_core_owner_witness,
            Item::Task(item) => &item.canonical_core_owner_witness,
            Item::Test(item) => &item.canonical_core_owner_witness,
        };
        witness
            .as_ref()
            .ok_or("canonical_core_item_witness_absent_v0")
    }

    fn sections(&self) -> &[Section] {
        match self {
            Item::App(item) => &item.sections,
            Item::Type(item) => &item.sections,
            Item::Store(item) => &item.sections,
            Item::Task(item) => &item.sections,
            Item::Test(item) => &item.sections,
        }
    }

    fn nested_items(&self) -> &[Item] {
        match self {
            Item::App(item) => &item.items,
            _ => &[],
        }
    }

    #[cfg(test)]
    pub(crate) fn corrupt_canonical_core_owner_witness_from(&mut self, foreign: &Item) {
        let foreign = foreign
            .canonical_core_owner_witness()
            .expect("foreign parser item witness")
            .clone();
        match self {
            Item::App(item) => item.canonical_core_owner_witness = Some(foreign),
            Item::Type(item) => item.canonical_core_owner_witness = Some(foreign),
            Item::Store(item) => item.canonical_core_owner_witness = Some(foreign),
            Item::Task(item) => item.canonical_core_owner_witness = Some(foreign),
            Item::Test(item) => item.canonical_core_owner_witness = Some(foreign),
        }
    }

    #[cfg(test)]
    pub(crate) fn remove_canonical_core_owner_witness(&mut self) {
        match self {
            Item::App(item) => item.canonical_core_owner_witness = None,
            Item::Type(item) => item.canonical_core_owner_witness = None,
            Item::Store(item) => item.canonical_core_owner_witness = None,
            Item::Task(item) => item.canonical_core_owner_witness = None,
            Item::Test(item) => item.canonical_core_owner_witness = None,
        }
    }

    #[cfg(test)]
    pub(crate) fn corrupt_canonical_task_signature(
        &mut self,
        corruption: CanonicalTaskSignatureCorruption,
    ) {
        let witness = self
            .canonical_core_owner_witness()
            .expect("parser-produced item witness");
        let corrupted =
            CanonicalCoreOwnerWitness(witness.0.corrupt_task_signature_for_test(corruption));
        match self {
            Item::App(item) => item.canonical_core_owner_witness = Some(corrupted),
            Item::Type(item) => item.canonical_core_owner_witness = Some(corrupted),
            Item::Store(item) => item.canonical_core_owner_witness = Some(corrupted),
            Item::Task(item) => item.canonical_core_owner_witness = Some(corrupted),
            Item::Test(item) => item.canonical_core_owner_witness = Some(corrupted),
        }
    }
}

impl Program {
    pub(crate) fn canonical_core_expectation<'a>(
        &'a self,
        item: &'a Item,
        section: &'a Section,
    ) -> Result<CanonicalCoreSectionExpectation<'a>, &'static str> {
        for (file_ordinal, file) in self.files.iter().enumerate() {
            if let Some((item_path, section_slot)) = locate_item_section(&file.items, item, section)
            {
                return CanonicalCoreSectionExpectation::new(
                    CanonicalCoreContainerRef::Program(self),
                    file,
                    item,
                    section,
                    file_ordinal,
                    item_path,
                    section_slot,
                );
            }
        }
        Err("canonical_core_live_program_reference_mismatch_v0")
    }

    pub(crate) fn canonical_core_expectation_for_task<'a>(
        &'a self,
        task: &'a Task,
        section: &'a Section,
    ) -> Result<CanonicalCoreSectionExpectation<'a>, &'static str> {
        for file in &self.files {
            if let Some(item) = find_task_item(&file.items, task) {
                return self.canonical_core_expectation(item, section);
            }
        }
        Err("canonical_core_live_task_reference_mismatch_v0")
    }

    pub(crate) fn authenticate_canonical_task_signature(
        &self,
        task: &Task,
    ) -> Result<AuthenticatedCanonicalTaskSignature, CanonicalTaskSignatureRejection> {
        for (file_ordinal, file) in self.files.iter().enumerate() {
            let mut item_path = Vec::new();
            if let Some(item) = find_task_item_with_path(&file.items, task, &mut item_path) {
                let file_witness = file
                    .canonical_core_file_witness()
                    .map_err(CanonicalTaskSignatureRejection::new)?;
                let file_binding = file_witness.binding();
                if file_binding.semantic_file_index != file_ordinal
                    || file_binding.normalized_path.as_ref() != file.path.replace('\\', "/")
                {
                    return Err(CanonicalTaskSignatureRejection::new(
                        "canonical_core_file_witness_mismatch_v0",
                    ));
                }
                let owner = item
                    .canonical_core_owner_witness()
                    .map_err(CanonicalTaskSignatureRejection::new)?
                    .binding();
                if &owner.file != file_binding
                    || owner.item_path.as_ref() != item_path.as_slice()
                    || owner.item_kind != "task"
                    || owner.section_slots.len() != task.sections.len()
                    || owner
                        .section_slots
                        .iter()
                        .zip(&task.sections)
                        .any(|(expected, actual)| expected.as_ref() != actual.name)
                {
                    return Err(CanonicalTaskSignatureRejection::new(
                        "canonical_core_item_witness_mismatch_v0",
                    ));
                }
                let snapshot = owner.task_signature.as_ref().ok_or_else(|| {
                    CanonicalTaskSignatureRejection::new(
                        "canonical_task_signature_snapshot_absent_v0",
                    )
                })?;
                snapshot
                    .validate_retained_facts(file_binding, &item_path)
                    .map_err(CanonicalTaskSignatureRejection::new)?;
                snapshot
                    .matches_live_task(task)
                    .map_err(CanonicalTaskSignatureRejection::new)?;
                return Ok(AuthenticatedCanonicalTaskSignature {
                    snapshot: snapshot.clone(),
                });
            }
        }
        Err(CanonicalTaskSignatureRejection::new(
            "canonical_core_live_task_reference_mismatch_v0",
        ))
    }
}

pub(crate) fn canonical_core_parse_expectation<'a>(
    file: &'a SourceFile,
    context: &'a CanonicalCoreParseContext,
    item: &'a Item,
    section: &'a Section,
) -> Result<CanonicalCoreSectionExpectation<'a>, &'static str> {
    let (item_path, section_slot) = locate_item_section(&file.items, item, section)
        .ok_or("canonical_core_live_parse_reference_mismatch_v0")?;
    CanonicalCoreSectionExpectation::new(
        CanonicalCoreContainerRef::Parse { file, context },
        file,
        item,
        section,
        context.binding().semantic_file_index,
        item_path,
        section_slot,
    )
}

fn locate_item_section(
    items: &[Item],
    target_item: &Item,
    target_section: &Section,
) -> Option<(Vec<usize>, usize)> {
    fn walk(
        items: &[Item],
        target_item: &Item,
        target_section: &Section,
        prefix: &mut Vec<usize>,
    ) -> Option<(Vec<usize>, usize)> {
        for (ordinal, item) in items.iter().enumerate() {
            prefix.push(ordinal);
            if std::ptr::eq(item, target_item) {
                let section_slot = item
                    .sections()
                    .iter()
                    .position(|candidate| std::ptr::eq(candidate, target_section))?;
                return Some((prefix.clone(), section_slot));
            }
            if let Some(found) = walk(item.nested_items(), target_item, target_section, prefix) {
                return Some(found);
            }
            prefix.pop();
        }
        None
    }

    walk(items, target_item, target_section, &mut Vec::new())
}

fn find_task_item<'a>(items: &'a [Item], target: &Task) -> Option<&'a Item> {
    for item in items {
        if matches!(item, Item::Task(task) if std::ptr::eq(task, target)) {
            return Some(item);
        }
        if let Some(found) = find_task_item(item.nested_items(), target) {
            return Some(found);
        }
    }
    None
}

fn find_task_item_with_path<'a>(
    items: &'a [Item],
    target: &Task,
    path: &mut Vec<usize>,
) -> Option<&'a Item> {
    for (ordinal, item) in items.iter().enumerate() {
        path.push(ordinal);
        if matches!(item, Item::Task(task) if std::ptr::eq(task, target)) {
            return Some(item);
        }
        if let Some(found) = find_task_item_with_path(item.nested_items(), target, path) {
            return Some(found);
        }
        path.pop();
    }
    None
}

impl<'a> CanonicalCoreSectionExpectation<'a> {
    fn new(
        container: CanonicalCoreContainerRef<'a>,
        file: &'a SourceFile,
        item: &'a Item,
        section: &'a Section,
        file_ordinal: usize,
        item_path: Vec<usize>,
        section_slot: usize,
    ) -> Result<Self, &'static str> {
        let expectation = Self {
            container,
            file,
            item,
            section,
            file_ordinal,
            item_path,
            section_slot,
        };
        expectation.recheck_live_traversal()?;
        Ok(expectation)
    }

    fn recheck_live_traversal(&self) -> Result<(), &'static str> {
        match self.container {
            CanonicalCoreContainerRef::Parse { file, .. } => {
                if !std::ptr::eq(file, self.file) {
                    return Err("canonical_core_parse_container_substitution_v0");
                }
                let (path, slot) = locate_item_section(&file.items, self.item, self.section)
                    .ok_or("canonical_core_live_parse_reference_mismatch_v0")?;
                if path != self.item_path || slot != self.section_slot {
                    return Err("canonical_core_live_parse_traversal_changed_v0");
                }
            }
            CanonicalCoreContainerRef::Program(program) => {
                let file = program
                    .files
                    .get(self.file_ordinal)
                    .ok_or("canonical_core_program_file_ordinal_missing_v0")?;
                if !std::ptr::eq(file, self.file) {
                    return Err("canonical_core_program_file_substitution_v0");
                }
                let (path, slot) = locate_item_section(&file.items, self.item, self.section)
                    .ok_or("canonical_core_live_program_reference_mismatch_v0")?;
                if path != self.item_path || slot != self.section_slot {
                    return Err("canonical_core_live_program_traversal_changed_v0");
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate(self) -> Result<ValidatedCoreSection<'a>, &'static str> {
        self.recheck_live_traversal()?;
        let file_witness = self.file.canonical_core_file_witness()?;
        let file_binding = file_witness.binding();
        if let CanonicalCoreContainerRef::Parse { context, .. } = self.container
            && context.binding() != file_binding
        {
            return Err("canonical_core_parse_context_mismatch_v0");
        }
        if file_binding.semantic_file_index != self.file_ordinal
            || file_binding.normalized_path.as_ref() != self.file.path.replace('\\', "/")
        {
            return Err("canonical_core_file_witness_mismatch_v0");
        }
        let owner = self.item.canonical_core_owner_witness()?.binding();
        if &owner.file != file_binding
            || owner.item_path.as_ref() != self.item_path.as_slice()
            || owner.item_kind != self.item.kind()
            || owner.section_slots.len() != self.item.sections().len()
            || owner
                .section_slots
                .iter()
                .zip(self.item.sections())
                .any(|(expected, actual)| expected.as_ref() != actual.name)
        {
            return Err("canonical_core_item_witness_mismatch_v0");
        }
        self.section.canonical_core_seal_capability()?.0.validate(
            file_binding,
            owner,
            self.section_slot,
            self.section,
        )?;
        Ok(ValidatedCoreSection {
            section: self.section,
        })
    }
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceFile")
            .field("path", &self.path)
            .field("module", &self.module)
            .field("items", &self.items)
            .finish()
    }
}

impl PartialEq for SourceFile {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.module == other.module && self.items == other.items
    }
}

impl Eq for SourceFile {}

macro_rules! impl_public_item_debug_eq {
    ($name:ident, [$($field:ident),+ $(,)?]) => {
        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut debug = formatter.debug_struct(stringify!($name));
                $(debug.field(stringify!($field), &self.$field);)+
                debug.finish()
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                true $(&& self.$field == other.$field)+
            }
        }

        impl Eq for $name {}
    };
}

impl_public_item_debug_eq!(App, [name, sections, items, span]);
impl_public_item_debug_eq!(TypeDef, [name, fields, sections, span]);
impl_public_item_debug_eq!(Store, [name, ty, sections, span]);
impl_public_item_debug_eq!(
    Task,
    [
        name,
        params,
        result,
        result_syntax,
        sections,
        effect_syntax,
        body_syntax,
        span,
    ]
);
impl_public_item_debug_eq!(Test, [name, params, modifiers, sections, span]);
impl_public_item_debug_eq!(Section, [name, lines, body_syntax, span]);

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
