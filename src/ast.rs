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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalCoreOwnerBinding {
    pub(crate) file: CanonicalCoreFileBinding,
    pub(crate) item_path: Arc<[usize]>,
    pub(crate) item_kind: &'static str,
    pub(crate) section_slots: Arc<[Arc<str>]>,
}

pub(crate) trait CanonicalCoreFileVerifier: Send + Sync {
    fn binding(&self) -> &CanonicalCoreFileBinding;
}

pub(crate) trait CanonicalCoreOwnerVerifier: Send + Sync {
    fn binding(&self) -> &CanonicalCoreOwnerBinding;
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
    pub(crate) fn parser_issue(verifier: Arc<dyn CanonicalCoreFileVerifier>) -> Self {
        Self(verifier)
    }

    pub(crate) fn binding(&self) -> &CanonicalCoreFileBinding {
        self.0.binding()
    }
}

impl CanonicalCoreOwnerWitness {
    pub(crate) fn parser_issue(verifier: Arc<dyn CanonicalCoreOwnerVerifier>) -> Self {
        Self(verifier)
    }

    fn binding(&self) -> &CanonicalCoreOwnerBinding {
        self.0.binding()
    }
}

impl CanonicalCoreSealCapability {
    pub(crate) fn parser_issue(verifier: Arc<dyn CanonicalCoreSectionVerifier>) -> Self {
        Self(verifier)
    }
}

impl CanonicalCoreParseContext {
    pub(crate) fn parser_issue(verifier: Arc<dyn CanonicalCoreParseContextVerifier>) -> Self {
        Self(verifier)
    }

    pub(crate) fn binding(&self) -> &CanonicalCoreFileBinding {
        self.0.binding()
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
