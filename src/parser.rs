use crate::ast::{
    App, CallableTypeSyntax, CanonicalCommonChildRole, CanonicalCommonLexicalStatus,
    CanonicalCommonNodeKind, CanonicalExpression, CanonicalExpressionIntentEvent,
    CanonicalExpressionKind, CanonicalExpressionRoleEvent, CanonicalLexicalTokenEvent,
    CanonicalOccurrenceAssignmentEvent, CanonicalReductionChildEvent, CanonicalReductionEvent,
    Field, Item, Param, ParamPermission, ParsedBinaryOperator, ParsedBlockRelationship,
    ParsedBodyStatement, ParsedBodyStatementKind, ParsedCall, ParsedCallCloseStatus,
    ParsedCallTrailingStatus, ParsedEffectDeclaration, ParsedEffectDeclarationKind,
    ParsedExpression, ParsedExpressionKind, ParsedIdentifier, ParsedSourceRange,
    ParserSyntaxNodeId, Section, SectionLine, SourceFile, Store, Task, Test, TypeDef, TypeSyntax,
    TypeSyntaxKind,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOccurrence, Span};
use crate::syntax;
use crate::typed_failure;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalOwnerIdentity {
    domain: u8,
    revision: Arc<[u8]>,
    traversal: Arc<[usize]>,
}

macro_rules! opaque_source_owner_id {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq)]
            struct $name(CanonicalOwnerIdentity);
        )+
    };
}

opaque_source_owner_id!(
    CanonicalSourceBlob,
    CanonicalSemanticFile,
    CanonicalItemOwner,
    CanonicalSectionOwner,
    CanonicalStatementOwner,
    CanonicalAuthorityHandle,
);

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalSourceRevision(Arc<[u8]>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum CanonicalSourceOwnerFact {
    SourceBlob(CanonicalSourceBlob),
    SemanticFile(CanonicalSemanticFile),
    SourceRevision(CanonicalSourceRevision),
    Item(CanonicalItemOwner),
    Section(CanonicalSectionOwner),
    Statement(CanonicalStatementOwner),
    AuthorityHandle(CanonicalAuthorityHandle),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalSourceOwnerAuthority {
    source_blob: CanonicalSourceBlob,
    semantic_file: CanonicalSemanticFile,
    source_revision: CanonicalSourceRevision,
    item: CanonicalItemOwner,
    section: CanonicalSectionOwner,
    statement: CanonicalStatementOwner,
    handle: CanonicalAuthorityHandle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalSourceOwnerSeal {
    projection: Vec<CanonicalSourceOwnerFact>,
    authority: CanonicalSourceOwnerAuthority,
}

fn source_owner_identity(
    domain: u8,
    revision: &CanonicalSourceRevision,
    traversal: &[usize],
) -> CanonicalOwnerIdentity {
    CanonicalOwnerIdentity {
        domain,
        revision: revision.0.clone(),
        traversal: traversal.into(),
    }
}

fn source_owner_child_identity(
    domain: u8,
    parent: &CanonicalOwnerIdentity,
    ordinal: usize,
) -> CanonicalOwnerIdentity {
    let mut traversal = parent.traversal.to_vec();
    traversal.push(ordinal);
    CanonicalOwnerIdentity {
        domain,
        revision: parent.revision.clone(),
        traversal: traversal.into(),
    }
}

fn source_owner_fact_matches(
    authority: &CanonicalSourceOwnerAuthority,
    index: usize,
    fact: &CanonicalSourceOwnerFact,
) -> bool {
    match fact {
        CanonicalSourceOwnerFact::SourceBlob(value) => {
            index == 0 && value == &authority.source_blob
        }
        CanonicalSourceOwnerFact::SemanticFile(value) => {
            index == 1 && value == &authority.semantic_file
        }
        CanonicalSourceOwnerFact::SourceRevision(value) => {
            index == 2 && value == &authority.source_revision
        }
        CanonicalSourceOwnerFact::Item(value) => index == 3 && value == &authority.item,
        CanonicalSourceOwnerFact::Section(value) => index == 4 && value == &authority.section,
        CanonicalSourceOwnerFact::Statement(value) => index == 5 && value == &authority.statement,
        CanonicalSourceOwnerFact::AuthorityHandle(value) => {
            index == 6 && value == &authority.handle
        }
    }
}

fn source_owner_authority_is_coherent(authority: &CanonicalSourceOwnerAuthority) -> bool {
    authority.source_blob.0.domain == 1
        && authority.source_blob.0.revision.as_ref() == authority.source_revision.0.as_ref()
}

fn validate_source_owner_seal(seal: &CanonicalSourceOwnerSeal) -> Result<(), &'static str> {
    if seal.projection.len() != 7 {
        return Err("canonical_source_owner_field_count_corrupt_v0");
    }
    if !source_owner_authority_is_coherent(&seal.authority)
        || seal
            .projection
            .iter()
            .enumerate()
            .any(|(index, fact)| !source_owner_fact_matches(&seal.authority, index, fact))
    {
        return Err("canonical_source_owner_authority_mismatch_v0");
    }
    Ok(())
}

macro_rules! opaque_occurrence_id {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq)]
            struct $name(CanonicalOwnerIdentity);
        )+
    };
}

opaque_occurrence_id!(
    CanonicalOccurrenceIdentity,
    CanonicalNodeIdentity,
    CanonicalTokenIdentity,
    CanonicalReductionIdentity,
    CanonicalAssigningEvent,
    CanonicalPredicateEvent,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CanonicalExpressionRole {
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
enum CanonicalExpressionIntent {
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
enum CanonicalPredicateRecognition {
    Present(CanonicalPredicateEvent),
    Absent(CanonicalPredicateEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CanonicalSealFact {
    SourceBlob(CanonicalSourceBlob),
    SemanticFile(CanonicalSemanticFile),
    SourceRevision(CanonicalSourceRevision),
    Item(CanonicalItemOwner),
    Section(CanonicalSectionOwner),
    Statement(CanonicalStatementOwner),
    AuthorityHandle(CanonicalAuthorityHandle),
    Occurrence(CanonicalOccurrenceIdentity),
    ExpressionRole(CanonicalExpressionRole),
    Root(CanonicalNodeIdentity),
    RootRange(ParsedSourceRange),
    RootReduction(CanonicalReductionIdentity),
    Intent(CanonicalExpressionIntent),
    AssigningEvent(CanonicalAssigningEvent),
    AssignmentSyntaxNode(CanonicalAssigningEvent, ParserSyntaxNodeId),
    PredicateRecognition(CanonicalPredicateRecognition),
    PreorderCount(usize),
    MaximumDelimiterDepth(usize),
    NodeIdentity(CanonicalNodeIdentity),
    NodeOccurrence(CanonicalNodeIdentity, CanonicalOccurrenceIdentity),
    TokenIdentity(CanonicalTokenIdentity),
    ReductionIdentity(CanonicalNodeIdentity, CanonicalReductionIdentity),
    ParentIdentity(CanonicalNodeIdentity, Option<CanonicalNodeIdentity>),
    ChildRole(CanonicalNodeIdentity, Option<CanonicalCommonChildRole>),
    ChildOrdinal(CanonicalNodeIdentity, Option<usize>),
    PreorderOrdinal(CanonicalNodeIdentity, usize),
    NodeSource(CanonicalNodeIdentity, CanonicalSemanticFile),
    NodeRange(CanonicalNodeIdentity, ParsedSourceRange),
    TokenInterval(
        CanonicalNodeIdentity,
        Option<(CanonicalTokenIdentity, CanonicalTokenIdentity)>,
    ),
    Kind(CanonicalNodeIdentity, CanonicalCommonNodeKind),
    OrderedChildren(CanonicalNodeIdentity, Vec<CanonicalNodeIdentity>),
    ChildCardinality(CanonicalNodeIdentity, usize),
    DelimiterDepthBefore(CanonicalNodeIdentity, usize),
    DelimiterDepthAfter(CanonicalNodeIdentity, usize),
    LexicalStatus(CanonicalNodeIdentity, CanonicalCommonLexicalStatus),
    IllegalFieldsAbsent(CanonicalNodeIdentity),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalOccurrenceSeal {
    projection: Vec<CanonicalSealFact>,
    authority: Vec<CanonicalSealFact>,
}

fn canonical_fact_matches(left: &CanonicalSealFact, right: &CanonicalSealFact) -> bool {
    match (left, right) {
        (CanonicalSealFact::SourceBlob(left), CanonicalSealFact::SourceBlob(right)) => {
            left == right
        }
        (CanonicalSealFact::SemanticFile(left), CanonicalSealFact::SemanticFile(right)) => {
            left == right
        }
        (CanonicalSealFact::SourceRevision(left), CanonicalSealFact::SourceRevision(right)) => {
            left == right
        }
        (CanonicalSealFact::Item(left), CanonicalSealFact::Item(right)) => left == right,
        (CanonicalSealFact::Section(left), CanonicalSealFact::Section(right)) => left == right,
        (CanonicalSealFact::Statement(left), CanonicalSealFact::Statement(right)) => left == right,
        (CanonicalSealFact::AuthorityHandle(left), CanonicalSealFact::AuthorityHandle(right)) => {
            left == right
        }
        (CanonicalSealFact::Occurrence(left), CanonicalSealFact::Occurrence(right)) => {
            left == right
        }
        (CanonicalSealFact::ExpressionRole(left), CanonicalSealFact::ExpressionRole(right)) => {
            left == right
        }
        (CanonicalSealFact::Root(left), CanonicalSealFact::Root(right)) => left == right,
        (CanonicalSealFact::RootRange(left), CanonicalSealFact::RootRange(right)) => left == right,
        (CanonicalSealFact::RootReduction(left), CanonicalSealFact::RootReduction(right)) => {
            left == right
        }
        (CanonicalSealFact::Intent(left), CanonicalSealFact::Intent(right)) => left == right,
        (CanonicalSealFact::AssigningEvent(left), CanonicalSealFact::AssigningEvent(right)) => {
            left == right
        }
        (
            CanonicalSealFact::AssignmentSyntaxNode(left_event, left_node),
            CanonicalSealFact::AssignmentSyntaxNode(right_event, right_node),
        ) => left_event == right_event && left_node == right_node,
        (
            CanonicalSealFact::PredicateRecognition(left),
            CanonicalSealFact::PredicateRecognition(right),
        ) => left == right,
        (CanonicalSealFact::PreorderCount(left), CanonicalSealFact::PreorderCount(right)) => {
            left == right
        }
        (
            CanonicalSealFact::MaximumDelimiterDepth(left),
            CanonicalSealFact::MaximumDelimiterDepth(right),
        ) => left == right,
        (CanonicalSealFact::NodeIdentity(left), CanonicalSealFact::NodeIdentity(right)) => {
            left == right
        }
        (
            CanonicalSealFact::NodeOccurrence(left_node, left),
            CanonicalSealFact::NodeOccurrence(right_node, right),
        ) => left_node == right_node && left == right,
        (CanonicalSealFact::TokenIdentity(left), CanonicalSealFact::TokenIdentity(right)) => {
            left == right
        }
        (
            CanonicalSealFact::ReductionIdentity(left_node, left),
            CanonicalSealFact::ReductionIdentity(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::ParentIdentity(left_node, left),
            CanonicalSealFact::ParentIdentity(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::ChildRole(left_node, left),
            CanonicalSealFact::ChildRole(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::ChildOrdinal(left_node, left),
            CanonicalSealFact::ChildOrdinal(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::PreorderOrdinal(left_node, left),
            CanonicalSealFact::PreorderOrdinal(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::NodeSource(left_node, left),
            CanonicalSealFact::NodeSource(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::NodeRange(left_node, left),
            CanonicalSealFact::NodeRange(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::TokenInterval(left_node, left),
            CanonicalSealFact::TokenInterval(right_node, right),
        ) => left_node == right_node && left == right,
        (CanonicalSealFact::Kind(left_node, left), CanonicalSealFact::Kind(right_node, right)) => {
            left_node == right_node && left == right
        }
        (
            CanonicalSealFact::OrderedChildren(left_node, left),
            CanonicalSealFact::OrderedChildren(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::ChildCardinality(left_node, left),
            CanonicalSealFact::ChildCardinality(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::DelimiterDepthBefore(left_node, left),
            CanonicalSealFact::DelimiterDepthBefore(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::DelimiterDepthAfter(left_node, left),
            CanonicalSealFact::DelimiterDepthAfter(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::LexicalStatus(left_node, left),
            CanonicalSealFact::LexicalStatus(right_node, right),
        ) => left_node == right_node && left == right,
        (
            CanonicalSealFact::IllegalFieldsAbsent(left),
            CanonicalSealFact::IllegalFieldsAbsent(right),
        ) => left == right,
        _ => false,
    }
}

fn validate_occurrence_seal(seal: &CanonicalOccurrenceSeal) -> Result<(), &'static str> {
    validate_occurrence_seal_inner(seal, None)
}

fn validate_occurrence_seal_inner(
    seal: &CanonicalOccurrenceSeal,
    ignored_index: Option<usize>,
) -> Result<(), &'static str> {
    if seal.projection.len() != seal.authority.len() {
        return Err("canonical_occurrence_field_count_corrupt_v0");
    }
    if seal
        .projection
        .iter()
        .zip(&seal.authority)
        .enumerate()
        .any(|(index, (left, right))| {
            Some(index) != ignored_index && !canonical_fact_matches(left, right)
        })
    {
        return Err("canonical_occurrence_authority_mismatch_v0");
    }
    Ok(())
}

#[cfg(test)]
fn validate_occurrence_seal_ignoring_one_fact(
    seal: &CanonicalOccurrenceSeal,
    ignored_index: usize,
) -> Result<(), &'static str> {
    validate_occurrence_seal_inner(seal, Some(ignored_index))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ParsedCallPosition {
    statement_index: usize,
    path: Vec<usize>,
}

impl ParsedCallPosition {
    pub(crate) fn statement_index(&self) -> usize {
        self.statement_index
    }

    pub(crate) fn stable_component(&self) -> String {
        format!(
            "statement-{}:path-{}",
            self.statement_index,
            self.path
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedCallIdentifierUse {
    pub(crate) name: String,
    pub(crate) ordinal: usize,
    pub(crate) consumed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedExecutableCallNode {
    pub(crate) position: ParsedCallPosition,
    pub(crate) callee: String,
    pub(crate) source: String,
    pub(crate) span: Span,
    pub(crate) identifier_uses: Vec<ParsedCallIdentifierUse>,
}

#[derive(Debug, Clone)]
pub struct ParseOutput {
    pub file: SourceFile,
    pub diagnostics: Vec<Diagnostic>,
    pub(crate) diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet,
    #[cfg_attr(not(test), allow(dead_code))]
    source_owner_seals: Vec<CanonicalSourceOwnerSeal>,
    #[cfg_attr(not(test), allow(dead_code))]
    occurrence_seals: Vec<CanonicalOccurrenceSeal>,
}

#[derive(Debug, Clone)]
struct SourceLine {
    number: usize,
    text: String,
}

#[derive(Debug, Clone, Copy)]
enum IdentifierKind {
    Value,
    Type,
}

struct Parser {
    path: String,
    semantic_file_index: usize,
    source_revision: CanonicalSourceRevision,
    source_blob: CanonicalSourceBlob,
    semantic_file: CanonicalSemanticFile,
    current_item_owner: Option<CanonicalItemOwner>,
    current_semantic_node: Option<String>,
    lines: Vec<SourceLine>,
    diagnostics: Vec<Diagnostic>,
    diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet,
    source_owner_seals: Vec<CanonicalSourceOwnerSeal>,
    occurrence_seals: Vec<CanonicalOccurrenceSeal>,
}

#[cfg(test)]
pub fn parse_source(path: impl Into<String>, source: &str) -> ParseOutput {
    parse_source_at_index(path, source, 0)
}

pub(crate) fn parse_source_at_index(
    path: impl Into<String>,
    source: &str,
    semantic_file_index: usize,
) -> ParseOutput {
    let path = path.into();
    let source_revision = CanonicalSourceRevision(source.as_bytes().into());
    let file_traversal = [semantic_file_index];
    let source_blob =
        CanonicalSourceBlob(source_owner_identity(1, &source_revision, &file_traversal));
    let semantic_file =
        CanonicalSemanticFile(source_owner_identity(2, &source_revision, &file_traversal));
    let lines = source
        .lines()
        .enumerate()
        .map(|(index, text)| SourceLine {
            number: index + 1,
            text: text.to_string(),
        })
        .collect();

    let mut parser = Parser {
        path: path.clone(),
        semantic_file_index,
        source_revision,
        source_blob,
        semantic_file,
        current_item_owner: None,
        current_semantic_node: None,
        lines,
        diagnostics: Vec::new(),
        diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet::default(),
        source_owner_seals: Vec::new(),
        occurrence_seals: Vec::new(),
    };
    let (module, items) = parser.parse_file_items();
    parser
        .diagnostic_occurrences
        .validate()
        .expect("parser diagnostics must use registered parser causes");

    ParseOutput {
        file: SourceFile {
            path,
            module,
            items,
        },
        diagnostics: parser.diagnostics,
        diagnostic_occurrences: parser.diagnostic_occurrences,
        source_owner_seals: parser.source_owner_seals,
        occurrence_seals: parser.occurrence_seals,
    }
}

impl Parser {
    fn parse_file_items(&mut self) -> (Option<String>, Vec<Item>) {
        let mut module = None;
        let mut items = Vec::new();
        let mut index = 0;

        while index < self.lines.len() {
            let line = self.lines[index].clone();
            let trimmed = line.text.trim();

            if is_ignorable(trimmed) {
                index += 1;
                continue;
            }

            if count_indent(&line.text) == 0 {
                if let Some(rest) = trimmed.strip_prefix("module ") {
                    let module_name = rest.trim().to_string();
                    self.validate_module_path(&module_name, line.number);
                    module = Some(module_name);
                    index += 1;
                    continue;
                }

                if syntax::is_item_start(trimmed) {
                    let item_path = vec![items.len()];
                    match self.parse_item_at_semantic_node(index, &item_path) {
                        Some((item, next_index)) => {
                            items.push(item);
                            index = next_index;
                        }
                        None => index += 1,
                    }
                    continue;
                }
            }

            self.emit(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(48),
                "top_level_line",
                Diagnostic::warning(
                    DiagnosticCode::UNEXPECTED_TOP_LEVEL_LINE,
                    "unexpected top-level line",
                    Some(self.span(line.number)),
                )
                .with_help(
                    "Hum milestone 0 accepts module, app, type, store, task, and test at the top level.",
                ),
            );
            index += 1;
        }

        (module, items)
    }

    fn parse_items_in_range(
        &mut self,
        start: usize,
        end: usize,
        item_indent: usize,
        parent_path: &[usize],
    ) -> Vec<Item> {
        let mut items = Vec::new();
        let mut index = start;
        while index < end {
            let line_number = self.lines[index].number;
            let line_text = self.lines[index].text.clone();
            let trimmed = line_text.trim();
            if is_ignorable(trimmed) {
                index += 1;
                continue;
            }

            if count_indent(&line_text) == item_indent && syntax::is_item_start(trimmed) {
                let mut item_path = parent_path.to_vec();
                item_path.push(items.len());
                match self.parse_item_at_semantic_node(index, &item_path) {
                    Some((item, next_index)) if next_index <= end + 1 => {
                        items.push(item);
                        index = next_index;
                    }
                    Some((_item, next_index)) => {
                        self.emit(
                            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(49),
                            "nested_item",
                            Diagnostic::error(
                                DiagnosticCode::NESTED_ITEM_EXTENDS_PAST_BLOCK,
                                "nested item extends past containing block",
                                Some(self.span(line_number)),
                            ),
                        );
                        index = next_index;
                    }
                    None => index += 1,
                }
            } else {
                index += 1;
            }
        }
        items
    }

    fn parse_item_at_semantic_node(
        &mut self,
        index: usize,
        item_path: &[usize],
    ) -> Option<(Item, usize)> {
        let semantic_node = format!(
            "resolver-item:file-{}:path-{}",
            self.semantic_file_index,
            item_path
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(".")
        );
        let prior = self.current_semantic_node.replace(semantic_node);
        let item_traversal = std::iter::once(self.semantic_file_index)
            .chain(item_path.iter().copied())
            .collect::<Vec<_>>();
        let prior_item =
            self.current_item_owner
                .replace(CanonicalItemOwner(source_owner_identity(
                    3,
                    &self.source_revision,
                    &item_traversal,
                )));
        let parsed = self.parse_item_at(index, item_path);
        self.current_item_owner = prior_item;
        self.current_semantic_node = prior;
        parsed
    }

    fn parse_item_at(&mut self, index: usize, item_path: &[usize]) -> Option<(Item, usize)> {
        let line = self.lines.get(index)?.clone();
        let trimmed = line.text.trim();
        let header = match trimmed.strip_suffix('{') {
            Some(header) => header.trim(),
            None => {
                self.emit(
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(50),
                    "item_header",
                    Diagnostic::error(
                        DiagnosticCode::ITEM_HEADER_MISSING_OPEN_BRACE,
                        "item header must end with `{`",
                        Some(self.span(line.number)),
                    )
                    .with_help(
                        "Write items as `task name(...) { ... }`, `type Name { ... }`, and so on.",
                    ),
                );
                return None;
            }
        };

        let close_index = match self.find_matching_close(index) {
            Some(close_index) => close_index,
            None => {
                self.emit(
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(51),
                    "item_block",
                    Diagnostic::error(
                        DiagnosticCode::ITEM_BLOCK_MISSING_CLOSE_BRACE,
                        "item block is missing a closing `}`",
                        Some(self.span(line.number)),
                    ),
                );
                return None;
            }
        };

        let item_indent = count_indent(&line.text);
        let body_start = index + 1;
        let body_end = close_index;
        let sections = self.parse_sections(body_start, body_end, item_indent + 2);
        let span = self.span(line.number);

        let item = if header.starts_with("app ") {
            let name = header.trim_start_matches("app ").trim().to_string();
            self.validate_identifier("app name", &name, IdentifierKind::Value, line.number);
            let nested_items =
                self.parse_items_in_range(body_start, body_end, item_indent + 2, item_path);
            Item::App(App {
                name,
                sections,
                items: nested_items,
                span,
            })
        } else if header.starts_with("type ") {
            let name = header.trim_start_matches("type ").trim().to_string();
            self.validate_identifier("type name", &name, IdentifierKind::Type, line.number);
            let fields = self.parse_fields(body_start, body_end, item_indent + 2);
            Item::Type(TypeDef {
                name,
                fields,
                sections,
                span,
            })
        } else if header.starts_with("store ") {
            let (name, ty) = parse_store_header(header);
            self.validate_identifier("store name", &name, IdentifierKind::Value, line.number);
            Item::Store(Store {
                name,
                ty,
                sections,
                span,
            })
        } else if header.starts_with("task ") {
            let (name, params, result, result_syntax) = self.parse_task_header(header, line.number);
            let effect_syntax = parse_task_effect_syntax(&sections);
            let body_syntax = parse_task_body_syntax(&sections);
            Item::Task(Task {
                name,
                params,
                result,
                result_syntax,
                sections,
                effect_syntax,
                body_syntax,
                span,
            })
        } else if header.starts_with("test ") {
            let (name, params, modifiers) = self.parse_test_header(header, line.number);
            Item::Test(Test {
                name,
                params,
                modifiers,
                sections,
                span,
            })
        } else {
            self.emit(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(52),
                "item_kind",
                Diagnostic::warning(
                    DiagnosticCode::UNKNOWN_ITEM_KIND,
                    "unknown item kind",
                    Some(self.span(line.number)),
                ),
            );
            return None;
        };

        Some((item, close_index + 1))
    }

    fn find_matching_close(&self, start: usize) -> Option<usize> {
        let mut depth = 0usize;
        let mut quoted = false;
        let mut escaped = false;
        for index in start..self.lines.len() {
            let text = &self.lines[index].text;
            for ch in text.chars() {
                if quoted {
                    if escaped {
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        quoted = false;
                    }
                    continue;
                }
                if ch == '"' {
                    quoted = true;
                    continue;
                }
                match ch {
                    '{' => depth = depth.saturating_add(1),
                    '}' => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            return Some(index);
                        }
                    }
                    _ => {}
                }
            }
            // Text literals are line-scoped in the current surface. A malformed
            // quote remains a malformed line; it cannot consume later item braces.
            quoted = false;
            escaped = false;
        }
        None
    }

    fn parse_sections(&mut self, start: usize, end: usize, section_indent: usize) -> Vec<Section> {
        let mut sections = Vec::new();
        let mut index = start;

        while index < end {
            let line = self.lines[index].clone();
            let trimmed = line.text.trim();
            if count_indent(&line.text) == section_indent && is_section_header(trimmed) {
                let name = trimmed.trim_end_matches(':').trim().to_string();
                let mut lines = Vec::new();
                let mut cursor = index + 1;

                while cursor < end {
                    let candidate = &self.lines[cursor];
                    let candidate_trimmed = candidate.text.trim();
                    let candidate_indent = count_indent(&candidate.text);
                    if candidate_indent == section_indent
                        && (is_section_header(candidate_trimmed)
                            || syntax::is_item_start(candidate_trimmed))
                    {
                        break;
                    }
                    lines.push(SectionLine {
                        text: candidate.text.trim().to_string(),
                        span: self.span(candidate.number),
                    });
                    cursor += 1;
                }

                let semantic_node = self
                    .current_semantic_node
                    .clone()
                    .unwrap_or_else(|| "resolver-item:unknown".to_string());
                let body_syntax = if name == "does" {
                    let retained = retain_body_syntax(&lines, &semantic_node);
                    for line in &lines {
                        if let Some(identifier) =
                            invalid_non_ascii_return_identifier(line.text.trim())
                        {
                            self.invalid_identifier(
                                "value",
                                identifier,
                                IdentifierKind::Value,
                                line.span.line,
                            );
                        }
                    }
                    retained
                } else {
                    vec![None; lines.len()]
                };
                let owners = self.retain_source_owner_records(sections.len(), &lines);
                self.retain_occurrence_records(
                    &name,
                    &lines,
                    &body_syntax,
                    &owners,
                    &semantic_node,
                );
                sections.push(Section {
                    name,
                    lines,
                    body_syntax,
                    span: self.span(line.number),
                });
                index = cursor;
            } else {
                index += 1;
            }
        }

        sections
    }

    fn retain_source_owner_records(
        &mut self,
        section_index: usize,
        lines: &[SectionLine],
    ) -> Vec<Option<CanonicalSourceOwnerSeal>> {
        let item = self
            .current_item_owner
            .clone()
            .expect("section parsing requires an owning item");
        let section = CanonicalSectionOwner(source_owner_child_identity(4, &item.0, section_index));
        let mut owners = Vec::with_capacity(lines.len());
        let mut statement_index = 0usize;
        for line in lines {
            if is_ignorable(line.text.trim()) {
                owners.push(None);
                continue;
            }
            let statement = CanonicalStatementOwner(source_owner_child_identity(
                5,
                &section.0,
                statement_index,
            ));
            let handle = CanonicalAuthorityHandle(source_owner_child_identity(6, &statement.0, 0));
            let projection = vec![
                CanonicalSourceOwnerFact::SourceBlob(self.source_blob.clone()),
                CanonicalSourceOwnerFact::SemanticFile(self.semantic_file.clone()),
                CanonicalSourceOwnerFact::SourceRevision(self.source_revision.clone()),
                CanonicalSourceOwnerFact::Item(item.clone()),
                CanonicalSourceOwnerFact::Section(section.clone()),
                CanonicalSourceOwnerFact::Statement(statement.clone()),
                CanonicalSourceOwnerFact::AuthorityHandle(handle.clone()),
            ];
            let authority = CanonicalSourceOwnerAuthority {
                source_blob: self.source_blob.clone(),
                semantic_file: self.semantic_file.clone(),
                source_revision: self.source_revision.clone(),
                item: item.clone(),
                section: section.clone(),
                statement: statement.clone(),
                handle,
            };
            let seal = CanonicalSourceOwnerSeal {
                projection,
                authority,
            };
            validate_source_owner_seal(&seal)
                .expect("parser source/owner projection must match retained authority");
            self.source_owner_seals.push(seal.clone());
            owners.push(Some(seal));
            statement_index += 1;
        }
        owners
    }

    fn retain_occurrence_records(
        &mut self,
        section_name: &str,
        lines: &[SectionLine],
        body_syntax: &[Option<ParsedBodyStatement>],
        owners: &[Option<CanonicalSourceOwnerSeal>],
        semantic_node: &str,
    ) {
        for (line_index, line) in lines.iter().enumerate() {
            let Some(owner) = owners.get(line_index).and_then(Option::as_ref) else {
                continue;
            };
            if section_name == "does" {
                let Some(statement) = body_syntax.get(line_index).and_then(Option::as_ref) else {
                    continue;
                };
                let roots = statement_expression_roots(statement, line.text.trim());
                assert_eq!(
                    roots.len(),
                    statement.canonical_assignments.len(),
                    "parser statement roots and retained assignments must stay aligned"
                );
                for (ordinal, ((expression, role, intent), assignment)) in roots
                    .into_iter()
                    .zip(&statement.canonical_assignments)
                    .enumerate()
                {
                    let seal =
                        build_occurrence_seal(expression, owner, role, intent, assignment, ordinal);
                    validate_occurrence_seal(&seal)
                        .expect("parser occurrence projection must match retained authority");
                    self.occurrence_seals.push(seal);
                }
            } else if matches!(section_name, "needs" | "ensures") {
                let role = if section_name == "needs" {
                    CanonicalExpressionRole::NeedsPredicate
                } else {
                    CanonicalExpressionRole::EnsuresPredicate
                };
                let intent = if section_name == "needs" {
                    CanonicalExpressionIntent::NeedsPredicate
                } else {
                    CanonicalExpressionIntent::EnsuresPredicate
                };
                let expression = parse_expression_syntax(
                    line.text.trim(),
                    line.span.clone(),
                    ParserSyntaxNodeId::new(format!(
                        "parser-contract:{semantic_node}:{section_name}:{line_index}"
                    )),
                );
                let assignment = CanonicalOccurrenceAssignmentEvent {
                    expression_node_id: expression.canonical.node_id.clone(),
                    role: if section_name == "needs" {
                        CanonicalExpressionRoleEvent::NeedsPredicate
                    } else {
                        CanonicalExpressionRoleEvent::EnsuresPredicate
                    },
                    intent: if section_name == "needs" {
                        CanonicalExpressionIntentEvent::NeedsPredicate
                    } else {
                        CanonicalExpressionIntentEvent::EnsuresPredicate
                    },
                    predicate_recognized: true,
                };
                let seal = build_occurrence_seal(&expression, owner, role, intent, &assignment, 0);
                validate_occurrence_seal(&seal)
                    .expect("parser predicate occurrence must match retained authority");
                self.occurrence_seals.push(seal);
            }
        }
    }

    fn parse_fields(&mut self, start: usize, end: usize, field_indent: usize) -> Vec<Field> {
        let mut fields = Vec::new();
        for index in start..end {
            let line = self.lines[index].clone();
            let trimmed = line.text.trim();
            if is_ignorable(trimmed) || count_indent(&line.text) != field_indent {
                continue;
            }
            if is_section_header(trimmed) || syntax::is_item_start(trimmed) {
                continue;
            }
            if let Some((name, ty)) = trimmed.split_once(':') {
                let name = name.trim().to_string();
                self.validate_identifier("field name", &name, IdentifierKind::Value, line.number);
                fields.push(Field {
                    name,
                    ty: ty.trim().to_string(),
                    span: self.span(line.number),
                });
            }
        }
        fields
    }

    fn parse_task_header(
        &mut self,
        header: &str,
        line_number: usize,
    ) -> (String, Vec<Param>, Option<String>, Option<TypeSyntax>) {
        let rest = header.trim_start_matches("task ").trim();
        let (signature, result, result_offset) = match find_top_level_arrow(rest) {
            Some(index) => (
                rest[..index].trim(),
                Some(rest[index + 2..].trim().to_string()),
                Some(index + 2 + rest[index + 2..].len() - rest[index + 2..].trim_start().len()),
            ),
            None => (rest, None, None),
        };

        let signature_column = self.span(line_number).column + "task ".len();
        let (name, params, trailing) =
            self.parse_callable_signature(signature, line_number, signature_column);
        self.validate_identifier("task name", &name, IdentifierKind::Value, line_number);
        if !trailing.trim().is_empty() {
            self.emit(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(53),
                "task_signature",
                Diagnostic::warning(
                    DiagnosticCode::UNEXPECTED_SIGNATURE_TEXT,
                    "unexpected text after task parameter list",
                    Some(self.span(line_number)),
                ),
            );
        }
        let result_syntax = result.as_ref().map(|result| {
            let column =
                self.span(line_number).column + "task ".len() + result_offset.unwrap_or_default();
            parse_type_syntax(result, Span::new(self.path.clone(), line_number, column))
        });
        (name, params, result, result_syntax)
    }

    fn parse_test_header(
        &mut self,
        header: &str,
        line_number: usize,
    ) -> (String, Vec<Param>, Vec<String>) {
        let rest = header.trim_start_matches("test ").trim();
        if rest.contains('(') {
            let signature_column = self.span(line_number).column + "test ".len();
            let (name, params, trailing) =
                self.parse_callable_signature(rest, line_number, signature_column);
            let modifiers = trailing
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>();
            (name, params, modifiers)
        } else {
            let mut words = rest.split_whitespace().collect::<Vec<_>>();
            let mut modifiers = Vec::new();
            while let Some(last) = words.last().copied() {
                if syntax::is_test_modifier(last) {
                    modifiers.insert(0, last.to_string());
                    words.pop();
                } else {
                    break;
                }
            }
            (words.join(" "), Vec::new(), modifiers)
        }
    }

    fn parse_callable_signature(
        &mut self,
        signature: &str,
        line_number: usize,
        signature_column: usize,
    ) -> (String, Vec<Param>, String) {
        let Some(open) = signature.find('(') else {
            return (signature.trim().to_string(), Vec::new(), String::new());
        };
        let Some(close) = matching_delimiter(signature, open, '(', ')') else {
            self.emit(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(54),
                "callable_signature",
                Diagnostic::error(
                    DiagnosticCode::CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN,
                    "callable signature is missing `)`",
                    Some(self.span(line_number)),
                ),
            );
            return (
                signature[..open].trim().to_string(),
                Vec::new(),
                String::new(),
            );
        };

        let name = signature[..open].trim().to_string();
        let params_text = &signature[open + 1..close];
        let trailing = signature[close + 1..].trim().to_string();
        let params = self.parse_params(
            params_text,
            line_number,
            signature_column + signature[..open + 1].chars().count(),
        );
        (name, params, trailing)
    }

    fn parse_params(
        &mut self,
        params_text: &str,
        line_number: usize,
        params_column: usize,
    ) -> Vec<Param> {
        let mut params = Vec::new();
        if params_text.trim().is_empty() {
            return params;
        }

        let mut byte_offset = 0;
        for (param_index, raw_param) in split_top_level_ranges(params_text, ',')
            .into_iter()
            .enumerate()
        {
            let raw_param = &params_text[raw_param.clone()];
            let param = raw_param.trim();
            let leading_bytes = raw_param.len() - raw_param.trim_start().len();
            let column = params_column
                + params_text[..byte_offset].chars().count()
                + raw_param[..leading_bytes].chars().count();
            let param_span = Span::new(self.path.clone(), line_number, column);
            if let Some((name, ty)) = param.split_once(':') {
                let (permission, permission_explicit, name) = parse_param_permission(name.trim());
                let name = name.to_string();
                let colon = param.find(':').unwrap_or_default();
                let raw_type = &param[colon + 1..];
                let type_hws_valid = raw_type
                    .as_bytes()
                    .first()
                    .is_some_and(|byte| matches!(byte, b' ' | b'\t'));
                let type_leading = raw_type.len() - raw_type.trim_start().len();
                let type_column = column + param[..colon + 1].chars().count() + type_leading;
                let ty = ty.trim().to_string();
                self.validate_identifier(
                    "parameter name",
                    &name,
                    IdentifierKind::Value,
                    line_number,
                );
                params.push(Param {
                    name,
                    type_syntax: parse_type_syntax(
                        &ty,
                        Span::new(self.path.clone(), line_number, type_column),
                    ),
                    ty,
                    permission,
                    permission_explicit,
                    type_hws_valid,
                    separator_hws_valid: param_index == 0 || leading_bytes > 0,
                    span: param_span,
                });
            } else {
                self.emit(
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(55),
                    "parameter",
                    Diagnostic::error(
                        DiagnosticCode::PARAMETER_MISSING_TYPE,
                        format!("parameter `{param}` is missing a type"),
                        Some(param_span),
                    ),
                );
            }
            byte_offset += raw_param.len() + 1;
        }
        params
    }

    fn validate_module_path(&mut self, module_name: &str, line_number: usize) {
        if module_name.is_empty() {
            self.invalid_identifier(
                "module path",
                module_name,
                IdentifierKind::Value,
                line_number,
            );
            return;
        }

        for segment in module_name.split('.') {
            if !is_valid_identifier(segment, IdentifierKind::Value) {
                self.invalid_identifier(
                    "module segment",
                    segment,
                    IdentifierKind::Value,
                    line_number,
                );
            }
        }
    }

    fn validate_identifier(
        &mut self,
        label: &str,
        name: &str,
        kind: IdentifierKind,
        line_number: usize,
    ) {
        if !is_valid_identifier(name, kind) {
            self.invalid_identifier(label, name, kind, line_number);
        }
    }

    fn invalid_identifier(
        &mut self,
        label: &str,
        name: &str,
        kind: IdentifierKind,
        line_number: usize,
    ) {
        let suggestion = identifier_suggestion(name, kind);
        self.emit(
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(56),
            "identifier",
            Diagnostic::error(
                DiagnosticCode::INVALID_IDENTIFIER,
                format!("{label} `{name}` is not a valid Hum identifier"),
                Some(self.span(line_number)),
            )
            .with_help(format!(
                "Use `{suggestion}`. Value names use snake_case, type names use PascalCase, and sentences belong in `why:`."
            )),
        );
    }

    fn emit(
        &mut self,
        cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
        node_role: &'static str,
        diagnostic: Diagnostic,
    ) {
        let event = self.diagnostics.len();
        let semantic_node = self.current_semantic_node.clone().unwrap_or_else(|| {
            format!(
                "parser-source:file-{}:event-{event}",
                self.semantic_file_index
            )
        });
        let semantic_origin =
            format!("parser-node:{semantic_node}:event-{event}:role-{node_role}",);
        let route = vec![
            format!("parser_file_index={}", self.semantic_file_index),
            format!("parser_event={event}"),
            format!("parser_node_role={node_role}"),
            format!("parser_semantic_node={semantic_node}"),
        ];
        let (diagnostic, occurrence) = DiagnosticOccurrence::producer_diagnostic(
            cause_key,
            diagnostic,
            semantic_origin,
            route,
        )
        .expect("parser diagnostic cause and producer identity must be registered");
        self.diagnostic_occurrences
            .insert_owned(occurrence)
            .expect("parser diagnostic occurrences must be unique");
        self.diagnostics.push(diagnostic);
    }

    fn span(&self, line_number: usize) -> Span {
        let column = self
            .lines
            .iter()
            .find(|line| line.number == line_number)
            .map_or(1, |line| first_visible_column(&line.text));
        Span::new(self.path.clone(), line_number, column)
    }
}

fn statement_expression_roots<'a>(
    statement: &'a ParsedBodyStatement,
    text: &str,
) -> Vec<(
    &'a ParsedExpression,
    CanonicalExpressionRole,
    CanonicalExpressionIntent,
)> {
    if !statement.canonical_extra_occurrences.is_empty() {
        return statement
            .canonical_extra_occurrences
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                let (role, intent) = projected_other_assignment(text, index);
                (expression, role, intent)
            })
            .collect();
    }
    match &statement.kind {
        ParsedBodyStatementKind::Return(expression) => vec![(
            expression,
            CanonicalExpressionRole::ReturnValue,
            CanonicalExpressionIntent::Return,
        )],
        ParsedBodyStatementKind::Binding {
            value: Some(expression),
            ..
        } => vec![(
            expression,
            CanonicalExpressionRole::BindingValue,
            CanonicalExpressionIntent::Binding,
        )],
        ParsedBodyStatementKind::Binding { value: None, .. } => Vec::new(),
        ParsedBodyStatementKind::Other { expressions } => expressions
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                let (role, intent) = projected_other_assignment(text, index);
                (expression, role, intent)
            })
            .collect(),
    }
}

fn projected_other_assignment(
    text: &str,
    index: usize,
) -> (CanonicalExpressionRole, CanonicalExpressionIntent) {
    if projected_keyword(text, "set") {
        (
            CanonicalExpressionRole::SetValue,
            CanonicalExpressionIntent::SetValue,
        )
    } else if projected_keyword(text, "save") {
        (
            CanonicalExpressionRole::SavedValue,
            CanonicalExpressionIntent::SaveValue,
        )
    } else if projected_keyword(text, "if") || projected_keyword(text, "while") {
        (
            CanonicalExpressionRole::Condition,
            CanonicalExpressionIntent::Condition,
        )
    } else if projected_keyword(text, "for each") {
        (
            CanonicalExpressionRole::LoopCollection,
            CanonicalExpressionIntent::LoopCollection,
        )
    } else if projected_keyword(text, "for index") && index == 0 {
        (
            CanonicalExpressionRole::LoopRangeStart,
            CanonicalExpressionIntent::LoopRangeStart,
        )
    } else if projected_keyword(text, "for index") {
        (
            CanonicalExpressionRole::LoopRangeEnd,
            CanonicalExpressionIntent::LoopRangeEnd,
        )
    } else if projected_keyword(text, "fail") {
        (
            CanonicalExpressionRole::FailureValue,
            CanonicalExpressionIntent::Failure,
        )
    } else if projected_keyword(text, "expect") {
        (
            CanonicalExpressionRole::TestExpectation,
            CanonicalExpressionIntent::TestExpectation,
        )
    } else {
        (
            CanonicalExpressionRole::Other,
            CanonicalExpressionIntent::Other,
        )
    }
}

fn projected_keyword(text: &str, keyword: &str) -> bool {
    text.strip_prefix(keyword)
        .is_some_and(|rest| rest.starts_with([' ', '\t']))
}

fn occurrence_identity(
    handle: &CanonicalAuthorityHandle,
    ordinal: usize,
) -> CanonicalOccurrenceIdentity {
    CanonicalOccurrenceIdentity(source_owner_child_identity(7, &handle.0, ordinal))
}

fn occurrence_node_identity(
    occurrence: &CanonicalOccurrenceIdentity,
    preorder: usize,
) -> CanonicalNodeIdentity {
    CanonicalNodeIdentity(source_owner_child_identity(8, &occurrence.0, preorder))
}

fn occurrence_token_identity(
    occurrence: &CanonicalOccurrenceIdentity,
    ordinal: usize,
) -> CanonicalTokenIdentity {
    CanonicalTokenIdentity(source_owner_child_identity(9, &occurrence.0, ordinal))
}

fn occurrence_reduction_identity(
    occurrence: &CanonicalOccurrenceIdentity,
    preorder: usize,
) -> CanonicalReductionIdentity {
    CanonicalReductionIdentity(source_owner_child_identity(10, &occurrence.0, preorder))
}

fn assigning_event_identity(
    handle: &CanonicalAuthorityHandle,
    ordinal: usize,
) -> CanonicalAssigningEvent {
    CanonicalAssigningEvent(source_owner_child_identity(11, &handle.0, ordinal))
}

fn predicate_event_identity(
    handle: &CanonicalAuthorityHandle,
    ordinal: usize,
) -> CanonicalPredicateEvent {
    CanonicalPredicateEvent(source_owner_child_identity(12, &handle.0, ordinal))
}

fn authority_role(role: CanonicalExpressionRoleEvent) -> CanonicalExpressionRole {
    match role {
        CanonicalExpressionRoleEvent::ReturnValue => CanonicalExpressionRole::ReturnValue,
        CanonicalExpressionRoleEvent::BindingValue => CanonicalExpressionRole::BindingValue,
        CanonicalExpressionRoleEvent::SetValue => CanonicalExpressionRole::SetValue,
        CanonicalExpressionRoleEvent::SavedValue => CanonicalExpressionRole::SavedValue,
        CanonicalExpressionRoleEvent::Condition => CanonicalExpressionRole::Condition,
        CanonicalExpressionRoleEvent::LoopCollection => CanonicalExpressionRole::LoopCollection,
        CanonicalExpressionRoleEvent::LoopRangeStart => CanonicalExpressionRole::LoopRangeStart,
        CanonicalExpressionRoleEvent::LoopRangeEnd => CanonicalExpressionRole::LoopRangeEnd,
        CanonicalExpressionRoleEvent::FailureValue => CanonicalExpressionRole::FailureValue,
        CanonicalExpressionRoleEvent::TestExpectation => CanonicalExpressionRole::TestExpectation,
        CanonicalExpressionRoleEvent::NeedsPredicate => CanonicalExpressionRole::NeedsPredicate,
        CanonicalExpressionRoleEvent::EnsuresPredicate => CanonicalExpressionRole::EnsuresPredicate,
        CanonicalExpressionRoleEvent::Other => CanonicalExpressionRole::Other,
    }
}

fn authority_intent(intent: CanonicalExpressionIntentEvent) -> CanonicalExpressionIntent {
    match intent {
        CanonicalExpressionIntentEvent::Return => CanonicalExpressionIntent::Return,
        CanonicalExpressionIntentEvent::Binding => CanonicalExpressionIntent::Binding,
        CanonicalExpressionIntentEvent::SetValue => CanonicalExpressionIntent::SetValue,
        CanonicalExpressionIntentEvent::SaveValue => CanonicalExpressionIntent::SaveValue,
        CanonicalExpressionIntentEvent::Condition => CanonicalExpressionIntent::Condition,
        CanonicalExpressionIntentEvent::LoopCollection => CanonicalExpressionIntent::LoopCollection,
        CanonicalExpressionIntentEvent::LoopRangeStart => CanonicalExpressionIntent::LoopRangeStart,
        CanonicalExpressionIntentEvent::LoopRangeEnd => CanonicalExpressionIntent::LoopRangeEnd,
        CanonicalExpressionIntentEvent::Failure => CanonicalExpressionIntent::Failure,
        CanonicalExpressionIntentEvent::TestExpectation => {
            CanonicalExpressionIntent::TestExpectation
        }
        CanonicalExpressionIntentEvent::NeedsPredicate => CanonicalExpressionIntent::NeedsPredicate,
        CanonicalExpressionIntentEvent::EnsuresPredicate => {
            CanonicalExpressionIntent::EnsuresPredicate
        }
        CanonicalExpressionIntentEvent::Other => CanonicalExpressionIntent::Other,
    }
}

fn role_is_predicate(role: CanonicalExpressionRole) -> bool {
    matches!(
        role,
        CanonicalExpressionRole::Condition
            | CanonicalExpressionRole::NeedsPredicate
            | CanonicalExpressionRole::EnsuresPredicate
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalNodeEvidence {
    range: ParsedSourceRange,
    kind: CanonicalCommonNodeKind,
    parent: Option<usize>,
    child_role: Option<CanonicalCommonChildRole>,
    child_ordinal: Option<usize>,
    children: Vec<usize>,
    delimiter_depth_before: usize,
    delimiter_depth_after: usize,
    lexical_status: CanonicalCommonLexicalStatus,
}

fn common_node_kind(expression: &CanonicalExpression) -> CanonicalCommonNodeKind {
    match expression.kind {
        CanonicalExpressionKind::Unit => CanonicalCommonNodeKind::Unit,
        CanonicalExpressionKind::Identifier(_) => CanonicalCommonNodeKind::Identifier,
        CanonicalExpressionKind::Field { .. } => CanonicalCommonNodeKind::Field,
        CanonicalExpressionKind::UIntLiteral(_) => CanonicalCommonNodeKind::UIntLiteral,
        CanonicalExpressionKind::IntLiteral(_) => CanonicalCommonNodeKind::IntLiteral,
        CanonicalExpressionKind::BoolLiteral(_) => CanonicalCommonNodeKind::BoolLiteral,
        CanonicalExpressionKind::TextLiteral(_) => CanonicalCommonNodeKind::TextLiteral,
        CanonicalExpressionKind::ListLiteral(_) => CanonicalCommonNodeKind::ListLiteral,
        CanonicalExpressionKind::RecordLiteral { .. } => CanonicalCommonNodeKind::RecordLiteral,
        CanonicalExpressionKind::Call { .. } => CanonicalCommonNodeKind::Call,
        CanonicalExpressionKind::Permission { .. } => CanonicalCommonNodeKind::Permission,
        CanonicalExpressionKind::Binary { .. } => CanonicalCommonNodeKind::Binary,
        CanonicalExpressionKind::Group(_) => CanonicalCommonNodeKind::Group,
        CanonicalExpressionKind::Unsupported => CanonicalCommonNodeKind::Unsupported,
    }
}

fn canonical_expression_children(
    expression: &CanonicalExpression,
) -> Vec<(CanonicalCommonChildRole, usize, &CanonicalExpression)> {
    match &expression.kind {
        CanonicalExpressionKind::Field { base, .. } => {
            vec![(CanonicalCommonChildRole::FieldBase, 0, base)]
        }
        CanonicalExpressionKind::ListLiteral(values) => values
            .iter()
            .enumerate()
            .map(|(index, value)| (CanonicalCommonChildRole::ListElement, index, value))
            .collect(),
        CanonicalExpressionKind::RecordLiteral { fields, .. } => fields
            .iter()
            .enumerate()
            .map(|(index, (_, value))| (CanonicalCommonChildRole::RecordFieldValue, index, value))
            .collect(),
        CanonicalExpressionKind::Call { callee, arguments } => {
            let mut children = vec![(CanonicalCommonChildRole::CallCallee, 0, callee.as_ref())];
            children.extend(arguments.iter().enumerate().map(|(index, argument)| {
                (CanonicalCommonChildRole::CallArgument, index, argument)
            }));
            children
        }
        CanonicalExpressionKind::Permission { value, .. } => {
            vec![(CanonicalCommonChildRole::PermissionValue, 0, value)]
        }
        CanonicalExpressionKind::Binary { left, right, .. } => vec![
            (CanonicalCommonChildRole::BinaryLeft, 0, left),
            (CanonicalCommonChildRole::BinaryRight, 1, right),
        ],
        CanonicalExpressionKind::Group(value) => {
            vec![(CanonicalCommonChildRole::GroupValue, 0, value)]
        }
        CanonicalExpressionKind::Unit
        | CanonicalExpressionKind::Identifier(_)
        | CanonicalExpressionKind::UIntLiteral(_)
        | CanonicalExpressionKind::IntLiteral(_)
        | CanonicalExpressionKind::BoolLiteral(_)
        | CanonicalExpressionKind::TextLiteral(_)
        | CanonicalExpressionKind::Unsupported => Vec::new(),
    }
}

fn nested_delimiter_child(role: CanonicalCommonChildRole) -> bool {
    matches!(
        role,
        CanonicalCommonChildRole::ListElement
            | CanonicalCommonChildRole::RecordFieldValue
            | CanonicalCommonChildRole::CallArgument
            | CanonicalCommonChildRole::GroupValue
    )
}

fn projected_node_evidence(expression: &CanonicalExpression) -> Vec<CanonicalNodeEvidence> {
    let mut evidence = Vec::new();
    append_projected_node_evidence(expression, None, None, None, 0, &mut evidence);
    evidence
}

fn append_projected_node_evidence(
    expression: &CanonicalExpression,
    parent: Option<usize>,
    child_role: Option<CanonicalCommonChildRole>,
    child_ordinal: Option<usize>,
    delimiter_depth: usize,
    evidence: &mut Vec<CanonicalNodeEvidence>,
) -> usize {
    let index = evidence.len();
    evidence.push(CanonicalNodeEvidence {
        range: expression.range.clone(),
        kind: common_node_kind(expression),
        parent,
        child_role,
        child_ordinal,
        children: Vec::new(),
        delimiter_depth_before: delimiter_depth,
        delimiter_depth_after: delimiter_depth,
        lexical_status: if matches!(expression.kind, CanonicalExpressionKind::Unsupported) {
            CanonicalCommonLexicalStatus::Unsupported
        } else {
            CanonicalCommonLexicalStatus::Complete
        },
    });
    let children = canonical_expression_children(expression)
        .into_iter()
        .map(|(role, ordinal, child)| {
            append_projected_node_evidence(
                child,
                Some(index),
                Some(role),
                Some(ordinal),
                delimiter_depth + usize::from(nested_delimiter_child(role)),
                evidence,
            )
        })
        .collect();
    evidence[index].children = children;
    index
}

fn retained_node_evidence(event: &CanonicalReductionEvent) -> Vec<CanonicalNodeEvidence> {
    let mut evidence = Vec::new();
    append_retained_node_evidence(event, None, None, None, &mut evidence);
    evidence
}

fn append_retained_node_evidence(
    event: &CanonicalReductionEvent,
    parent: Option<usize>,
    child_role: Option<CanonicalCommonChildRole>,
    child_ordinal: Option<usize>,
    evidence: &mut Vec<CanonicalNodeEvidence>,
) -> usize {
    let index = evidence.len();
    evidence.push(CanonicalNodeEvidence {
        range: event.range.clone(),
        kind: event.kind,
        parent,
        child_role,
        child_ordinal,
        children: Vec::new(),
        delimiter_depth_before: event.delimiter_depth_before,
        delimiter_depth_after: event.delimiter_depth_after,
        lexical_status: event.lexical_status,
    });
    let children = event
        .children
        .iter()
        .map(|child| {
            append_retained_node_evidence(
                &child.event,
                Some(index),
                Some(child.role),
                Some(child.ordinal),
                evidence,
            )
        })
        .collect();
    evidence[index].children = children;
    index
}

fn translated_owner_projection(owner: &CanonicalSourceOwnerSeal) -> Vec<CanonicalSealFact> {
    owner
        .projection
        .iter()
        .map(|fact| match fact {
            CanonicalSourceOwnerFact::SourceBlob(value) => {
                CanonicalSealFact::SourceBlob(value.clone())
            }
            CanonicalSourceOwnerFact::SemanticFile(value) => {
                CanonicalSealFact::SemanticFile(value.clone())
            }
            CanonicalSourceOwnerFact::SourceRevision(value) => {
                CanonicalSealFact::SourceRevision(value.clone())
            }
            CanonicalSourceOwnerFact::Item(value) => CanonicalSealFact::Item(value.clone()),
            CanonicalSourceOwnerFact::Section(value) => CanonicalSealFact::Section(value.clone()),
            CanonicalSourceOwnerFact::Statement(value) => {
                CanonicalSealFact::Statement(value.clone())
            }
            CanonicalSourceOwnerFact::AuthorityHandle(value) => {
                CanonicalSealFact::AuthorityHandle(value.clone())
            }
        })
        .collect()
}

fn retained_owner_authority(owner: &CanonicalSourceOwnerSeal) -> Vec<CanonicalSealFact> {
    vec![
        CanonicalSealFact::SourceBlob(owner.authority.source_blob.clone()),
        CanonicalSealFact::SemanticFile(owner.authority.semantic_file.clone()),
        CanonicalSealFact::SourceRevision(owner.authority.source_revision.clone()),
        CanonicalSealFact::Item(owner.authority.item.clone()),
        CanonicalSealFact::Section(owner.authority.section.clone()),
        CanonicalSealFact::Statement(owner.authority.statement.clone()),
        CanonicalSealFact::AuthorityHandle(owner.authority.handle.clone()),
    ]
}

struct OccurrenceFactInput<'a> {
    owner_facts: Vec<CanonicalSealFact>,
    handle: &'a CanonicalAuthorityHandle,
    semantic_file: &'a CanonicalSemanticFile,
    occurrence: CanonicalOccurrenceIdentity,
    role: CanonicalExpressionRole,
    intent: CanonicalExpressionIntent,
    assignment_node: &'a ParserSyntaxNodeId,
    predicate_recognized: bool,
    ordinal: usize,
    nodes: &'a [CanonicalNodeEvidence],
    tokens: &'a [CanonicalLexicalTokenEvent],
}

fn occurrence_facts(input: OccurrenceFactInput<'_>) -> Vec<CanonicalSealFact> {
    let mut facts = input.owner_facts;
    let root = occurrence_node_identity(&input.occurrence, 0);
    let root_reduction = occurrence_reduction_identity(&input.occurrence, 0);
    let assigning_event = assigning_event_identity(input.handle, input.ordinal);
    let predicate_event = predicate_event_identity(input.handle, input.ordinal);
    facts.extend([
        CanonicalSealFact::Occurrence(input.occurrence.clone()),
        CanonicalSealFact::ExpressionRole(input.role),
        CanonicalSealFact::Root(root),
        CanonicalSealFact::RootRange(input.nodes[0].range.clone()),
        CanonicalSealFact::RootReduction(root_reduction),
        CanonicalSealFact::Intent(input.intent),
        CanonicalSealFact::AssigningEvent(assigning_event.clone()),
        CanonicalSealFact::AssignmentSyntaxNode(assigning_event, input.assignment_node.clone()),
        CanonicalSealFact::PredicateRecognition(if input.predicate_recognized {
            CanonicalPredicateRecognition::Present(predicate_event)
        } else {
            CanonicalPredicateRecognition::Absent(predicate_event)
        }),
        CanonicalSealFact::PreorderCount(input.nodes.len()),
        CanonicalSealFact::MaximumDelimiterDepth(
            input
                .nodes
                .iter()
                .map(|node| node.delimiter_depth_before.max(node.delimiter_depth_after))
                .max()
                .unwrap_or_default(),
        ),
    ]);
    let token_ids = (0..input.tokens.len())
        .map(|index| occurrence_token_identity(&input.occurrence, index))
        .collect::<Vec<_>>();
    facts.extend(
        token_ids
            .iter()
            .cloned()
            .map(CanonicalSealFact::TokenIdentity),
    );
    for (preorder, node) in input.nodes.iter().enumerate() {
        let node_id = occurrence_node_identity(&input.occurrence, preorder);
        let parent = node
            .parent
            .map(|index| occurrence_node_identity(&input.occurrence, index));
        let children = node
            .children
            .iter()
            .map(|index| occurrence_node_identity(&input.occurrence, *index))
            .collect::<Vec<_>>();
        let interval = node_token_interval(node, input.tokens, &token_ids);
        facts.extend([
            CanonicalSealFact::NodeIdentity(node_id.clone()),
            CanonicalSealFact::NodeOccurrence(node_id.clone(), input.occurrence.clone()),
            CanonicalSealFact::ReductionIdentity(
                node_id.clone(),
                occurrence_reduction_identity(&input.occurrence, preorder),
            ),
            CanonicalSealFact::ParentIdentity(node_id.clone(), parent),
            CanonicalSealFact::ChildRole(node_id.clone(), node.child_role),
            CanonicalSealFact::ChildOrdinal(node_id.clone(), node.child_ordinal),
            CanonicalSealFact::PreorderOrdinal(node_id.clone(), preorder),
            CanonicalSealFact::NodeSource(node_id.clone(), input.semantic_file.clone()),
            CanonicalSealFact::NodeRange(node_id.clone(), node.range.clone()),
            CanonicalSealFact::TokenInterval(node_id.clone(), interval),
            CanonicalSealFact::Kind(node_id.clone(), node.kind),
            CanonicalSealFact::OrderedChildren(node_id.clone(), children),
            CanonicalSealFact::ChildCardinality(node_id.clone(), node.children.len()),
            CanonicalSealFact::DelimiterDepthBefore(node_id.clone(), node.delimiter_depth_before),
            CanonicalSealFact::DelimiterDepthAfter(node_id.clone(), node.delimiter_depth_after),
            CanonicalSealFact::LexicalStatus(node_id.clone(), node.lexical_status),
            CanonicalSealFact::IllegalFieldsAbsent(node_id),
        ]);
    }
    facts
}

fn node_token_interval(
    node: &CanonicalNodeEvidence,
    tokens: &[CanonicalLexicalTokenEvent],
    token_ids: &[CanonicalTokenIdentity],
) -> Option<(CanonicalTokenIdentity, CanonicalTokenIdentity)> {
    let indexes = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| range_contains(&node.range, &token.range).then_some(index))
        .collect::<Vec<_>>();
    indexes
        .first()
        .zip(indexes.last())
        .map(|(first, last)| (token_ids[*first].clone(), token_ids[*last].clone()))
}

fn range_contains(parent: &ParsedSourceRange, child: &ParsedSourceRange) -> bool {
    parent.start.file == child.start.file
        && parent.start.line == child.start.line
        && parent.start.column <= child.start.column
        && child.start.column + child.byte_len <= parent.start.column + parent.byte_len
}

fn build_occurrence_seal(
    expression: &ParsedExpression,
    owner: &CanonicalSourceOwnerSeal,
    projected_role: CanonicalExpressionRole,
    projected_intent: CanonicalExpressionIntent,
    assignment: &CanonicalOccurrenceAssignmentEvent,
    ordinal: usize,
) -> CanonicalOccurrenceSeal {
    let projected_occurrence = occurrence_identity(
        match owner.projection.last() {
            Some(CanonicalSourceOwnerFact::AuthorityHandle(handle)) => handle,
            _ => panic!("source/owner projection must end with its authority handle"),
        },
        ordinal,
    );
    let authority_occurrence = occurrence_identity(&owner.authority.handle, ordinal);
    let projected_nodes = projected_node_evidence(&expression.canonical);
    let retained_nodes = retained_node_evidence(&expression.canonical_event);
    let projected_handle = match owner.projection.last() {
        Some(CanonicalSourceOwnerFact::AuthorityHandle(handle)) => handle,
        _ => unreachable!("validated source/owner projection has a handle"),
    };
    let projection = occurrence_facts(OccurrenceFactInput {
        owner_facts: translated_owner_projection(owner),
        handle: projected_handle,
        semantic_file: match owner.projection.get(1) {
            Some(CanonicalSourceOwnerFact::SemanticFile(file)) => file,
            _ => panic!("source/owner projection must contain semantic file identity"),
        },
        occurrence: projected_occurrence,
        role: projected_role,
        intent: projected_intent,
        assignment_node: &expression.canonical.node_id,
        predicate_recognized: role_is_predicate(projected_role),
        ordinal,
        nodes: &projected_nodes,
        tokens: &expression.canonical_tokens,
    });
    let authority = occurrence_facts(OccurrenceFactInput {
        owner_facts: retained_owner_authority(owner),
        handle: &owner.authority.handle,
        semantic_file: &owner.authority.semantic_file,
        occurrence: authority_occurrence,
        role: authority_role(assignment.role),
        intent: authority_intent(assignment.intent),
        assignment_node: &assignment.expression_node_id,
        predicate_recognized: assignment.predicate_recognized,
        ordinal,
        nodes: &retained_nodes,
        tokens: &expression.canonical_tokens,
    });
    CanonicalOccurrenceSeal {
        projection,
        authority,
    }
}

fn parse_task_body_syntax(sections: &[Section]) -> Vec<ParsedBodyStatement> {
    let Some(section) = sections.iter().find(|section| section.name == "does") else {
        return Vec::new();
    };
    let statements = section
        .body_syntax
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    if validate_retained_body_syntax(&statements).is_ok() {
        statements
    } else {
        Vec::new()
    }
}

fn validate_retained_body_syntax(statements: &[ParsedBodyStatement]) -> Result<(), &'static str> {
    let mut depth = 0usize;
    let mut identities = std::collections::BTreeSet::new();
    for (index, statement) in statements.iter().enumerate() {
        if !identities.insert(statement.source_node_id.as_str())
            || !statement
                .source_node_id
                .as_str()
                .ends_with(&format!("statement-{index}"))
            || statement.block_depth_before != depth
        {
            return Err("parser_body_identity_or_depth_corrupt_v0");
        }
        depth = match statement.block_relationship {
            ParsedBlockRelationship::Opens => depth.saturating_add(1),
            ParsedBlockRelationship::Closes => depth.saturating_sub(1),
            ParsedBlockRelationship::None => depth,
        };
        if statement.block_depth_after != depth {
            return Err("parser_body_relationship_corrupt_v0");
        }
        match &statement.kind {
            ParsedBodyStatementKind::Return(expression) => {
                validate_canonical_expression(&expression.canonical)?;
            }
            ParsedBodyStatementKind::Binding { value, .. } => {
                if let Some(expression) = value {
                    validate_canonical_expression(&expression.canonical)?;
                }
            }
            ParsedBodyStatementKind::Other { expressions } => {
                for expression in expressions {
                    validate_canonical_expression(&expression.canonical)?;
                }
            }
        }
    }
    Ok(())
}

fn retain_body_syntax(
    lines: &[SectionLine],
    semantic_node: &str,
) -> Vec<Option<ParsedBodyStatement>> {
    let mut depth = 0usize;
    let mut statement_index = 0usize;
    let mut retained = Vec::with_capacity(lines.len());
    for line in lines {
        let text = line.text.trim();
        if text.is_empty() || text.starts_with('#') || text.starts_with("//") {
            retained.push(None);
            continue;
        }
        let node_id = ParserSyntaxNodeId::new(format!(
            "parser-body:{semantic_node}:statement-{statement_index}"
        ));
        let relationship = parser_block_relationship(text);
        let before = depth;
        depth = match relationship {
            ParsedBlockRelationship::Opens => depth.saturating_add(1),
            ParsedBlockRelationship::Closes => depth.saturating_sub(1),
            ParsedBlockRelationship::None => depth,
        };
        retained.push(parse_body_statement_syntax(
            line,
            node_id,
            relationship,
            before,
            depth,
        ));
        statement_index += 1;
    }
    retained
}

fn parser_block_relationship(text: &str) -> ParsedBlockRelationship {
    if text == "}" {
        ParsedBlockRelationship::Closes
    } else if unquoted_last_non_whitespace(text) == Some('{') {
        ParsedBlockRelationship::Opens
    } else {
        ParsedBlockRelationship::None
    }
}

fn invalid_non_ascii_return_identifier(text: &str) -> Option<&str> {
    let candidate = keyword_rest(text, "return")?.trim();
    (!candidate.is_empty()
        && !candidate.is_ascii()
        && candidate
            .chars()
            .all(|ch| ch == '_' || ch.is_alphanumeric())
        && !is_valid_identifier(candidate, IdentifierKind::Value))
    .then_some(candidate)
}

pub(crate) fn executable_call_nodes(
    statements: &[ParsedBodyStatement],
) -> Vec<ParsedExecutableCallNode> {
    let mut calls = Vec::new();
    for (statement_index, statement) in statements.iter().enumerate() {
        match &statement.kind {
            ParsedBodyStatementKind::Return(expression) => {
                collect_executable_calls(expression, statement_index, vec![0], &mut calls)
            }
            ParsedBodyStatementKind::Binding { value, .. } => {
                if let Some(expression) = value {
                    collect_executable_calls(expression, statement_index, vec![0], &mut calls);
                }
            }
            ParsedBodyStatementKind::Other { expressions } => {
                for (expression_index, expression) in expressions.iter().enumerate() {
                    collect_executable_calls(
                        expression,
                        statement_index,
                        vec![expression_index],
                        &mut calls,
                    );
                }
            }
        }
    }
    calls
}

fn collect_executable_calls(
    expression: &ParsedExpression,
    statement_index: usize,
    path: Vec<usize>,
    calls: &mut Vec<ParsedExecutableCallNode>,
) {
    match &expression.kind {
        ParsedExpressionKind::Call(call) => {
            if let ParsedExpressionKind::Identifier(identifier) = &call.callee.kind
                && is_executable_callee(&identifier.name)
            {
                let mut identifier_uses = Vec::new();
                for argument in &call.arguments {
                    collect_call_identifier_uses(argument, false, &mut identifier_uses);
                }
                for (ordinal, identifier_use) in identifier_uses.iter_mut().enumerate() {
                    identifier_use.ordinal = ordinal;
                }
                calls.push(ParsedExecutableCallNode {
                    position: ParsedCallPosition {
                        statement_index,
                        path: path.clone(),
                    },
                    callee: identifier.name.clone(),
                    source: render_parsed_expression(expression),
                    span: expression.span.clone(),
                    identifier_uses,
                });
            }
            for (argument_index, argument) in call.arguments.iter().enumerate() {
                let mut argument_path = path.clone();
                argument_path.push(argument_index);
                collect_executable_calls(argument, statement_index, argument_path, calls);
            }
        }
        ParsedExpressionKind::Permission { value, .. } => {
            collect_executable_calls(value, statement_index, path, calls);
        }
        ParsedExpressionKind::Compound { operands } => {
            for (operand_index, operand) in operands.iter().enumerate() {
                let mut operand_path = path.clone();
                operand_path.push(operand_index);
                collect_executable_calls(operand, statement_index, operand_path, calls);
            }
        }
        ParsedExpressionKind::Identifier(_)
        | ParsedExpressionKind::UIntLiteral(_)
        | ParsedExpressionKind::Unsupported { .. }
        | ParsedExpressionKind::Other => {}
    }
}

fn collect_call_identifier_uses(
    expression: &ParsedExpression,
    consumed: bool,
    identifier_uses: &mut Vec<ParsedCallIdentifierUse>,
) {
    match &expression.kind {
        ParsedExpressionKind::Identifier(identifier) => {
            identifier_uses.push(ParsedCallIdentifierUse {
                name: identifier.name.clone(),
                ordinal: 0,
                consumed,
            });
        }
        ParsedExpressionKind::Permission { permission, value } => collect_call_identifier_uses(
            value,
            consumed || *permission == ParamPermission::Consume,
            identifier_uses,
        ),
        ParsedExpressionKind::Compound { operands } => {
            for operand in operands {
                collect_call_identifier_uses(operand, consumed, identifier_uses);
            }
        }
        ParsedExpressionKind::Call(_)
        | ParsedExpressionKind::UIntLiteral(_)
        | ParsedExpressionKind::Unsupported { .. }
        | ParsedExpressionKind::Other => {}
    }
}

fn is_executable_callee(name: &str) -> bool {
    is_value_identifier(name)
        && !matches!(
            name,
            "borrow" | "change" | "consume" | "expect" | "fail" | "if" | "return" | "try" | "while"
        )
}

fn render_parsed_expression(expression: &ParsedExpression) -> String {
    match &expression.kind {
        ParsedExpressionKind::Identifier(identifier) => identifier.name.clone(),
        ParsedExpressionKind::UIntLiteral(value) => value.to_string(),
        ParsedExpressionKind::Call(call) => {
            let mut rendered = render_parsed_expression(&call.callee);
            rendered.push('(');
            rendered.push_str(
                &call
                    .arguments
                    .iter()
                    .map(render_parsed_expression)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            if call.close_status == ParsedCallCloseStatus::Closed {
                rendered.push(')');
            }
            rendered
        }
        ParsedExpressionKind::Permission { permission, value } => format!(
            "{} {}",
            match permission {
                ParamPermission::Borrow => "borrow",
                ParamPermission::Change => "change",
                ParamPermission::Consume => "consume",
            },
            render_parsed_expression(value)
        ),
        ParsedExpressionKind::Compound { operands } => operands
            .iter()
            .map(render_parsed_expression)
            .collect::<Vec<_>>()
            .join(" "),
        ParsedExpressionKind::Unsupported { .. } | ParsedExpressionKind::Other => String::new(),
    }
}

fn parse_task_effect_syntax(sections: &[Section]) -> Vec<ParsedEffectDeclaration> {
    sections
        .iter()
        .filter_map(|section| {
            let kind = match section.name.as_str() {
                "uses" => ParsedEffectDeclarationKind::Use,
                "changes" => ParsedEffectDeclarationKind::Change,
                "fails when" => ParsedEffectDeclarationKind::Failure,
                _ => return None,
            };
            Some(section.lines.iter().filter_map(move |line| {
                let text = line.text.trim();
                (!text.is_empty() && !text.starts_with('#') && !text.starts_with("//")).then(|| {
                    ParsedEffectDeclaration {
                        kind,
                        span: line.span.clone(),
                    }
                })
            }))
        })
        .flatten()
        .collect()
}

fn parse_body_statement_syntax(
    line: &SectionLine,
    source_node_id: ParserSyntaxNodeId,
    block_relationship: ParsedBlockRelationship,
    block_depth_before: usize,
    block_depth_after: usize,
) -> Option<ParsedBodyStatement> {
    let text = line.text.trim();
    if text.is_empty() || text.starts_with('#') || text.starts_with("//") {
        return None;
    }
    if text == "return" {
        let expression = parse_expression_syntax(
            "",
            offset_span(&line.span, text.len()),
            source_node_id.child("expression-0"),
        );
        return Some(parsed_body_statement(
            ParsedBodyStatementKind::Return(expression),
            line,
            source_node_id,
            block_relationship,
            block_depth_before,
            block_depth_after,
        ));
    }
    if let Some(rest) = keyword_rest(text, "return") {
        let offset = text.len() - rest.len();
        let expression = parse_expression_syntax(
            rest,
            offset_span(&line.span, offset),
            source_node_id.child("expression-0"),
        );
        return Some(parsed_body_statement(
            ParsedBodyStatementKind::Return(expression),
            line,
            source_node_id,
            block_relationship,
            block_depth_before,
            block_depth_after,
        ));
    }
    for (keyword, mutable) in [("let", false), ("change", true)] {
        if let Some(rest) = keyword_rest(text, keyword) {
            let rest_offset = text.len() - rest.len();
            let (left, value) = find_top_level_char(rest, '=').map_or((rest, None), |index| {
                (&rest[..index], Some(&rest[index + 1..]))
            });
            let name_text = left
                .split_once(':')
                .map_or(left, |(name, _annotation)| name)
                .trim();
            let name_offset = rest.find(name_text).unwrap_or_default();
            let name = is_value_identifier(name_text).then(|| ParsedIdentifier {
                name: name_text.to_string(),
                span: offset_span(&line.span, rest_offset + name_offset),
            });
            let value = value.map(|value| {
                let leading = value.len() - value.trim_start().len();
                let value = value.trim();
                let equals = find_top_level_char(rest, '=').unwrap_or(rest.len());
                parse_expression_syntax(
                    value,
                    offset_span(&line.span, rest_offset + equals + 1 + leading),
                    source_node_id.child("expression-0"),
                )
            });
            return Some(parsed_body_statement(
                ParsedBodyStatementKind::Binding {
                    mutable,
                    name,
                    value,
                },
                line,
                source_node_id,
                block_relationship,
                block_depth_before,
                block_depth_after,
            ));
        }
    }
    Some(parsed_body_statement(
        ParsedBodyStatementKind::Other {
            expressions: parse_other_statement_expressions(text, &line.span, &source_node_id),
        },
        line,
        source_node_id,
        block_relationship,
        block_depth_before,
        block_depth_after,
    ))
}

fn parsed_body_statement(
    kind: ParsedBodyStatementKind,
    line: &SectionLine,
    source_node_id: ParserSyntaxNodeId,
    block_relationship: ParsedBlockRelationship,
    block_depth_before: usize,
    block_depth_after: usize,
) -> ParsedBodyStatement {
    let (core_kind, core_status, core_expression_kind, core_reason) =
        parser_core_shape(line.text.trim(), &kind);
    let canonical_extra_occurrences =
        parse_for_index_occurrence_expressions(line.text.trim(), &line.span, &source_node_id);
    let canonical_assignments =
        retained_statement_assignments(line.text.trim(), &kind, &canonical_extra_occurrences);
    ParsedBodyStatement {
        kind,
        span: line.span.clone(),
        source_node_id,
        block_relationship,
        block_depth_before,
        block_depth_after,
        core_kind,
        core_status,
        core_expression_kind,
        core_reason,
        canonical_extra_occurrences,
        canonical_assignments,
    }
}

fn retained_statement_assignments(
    text: &str,
    kind: &ParsedBodyStatementKind,
    canonical_extra_occurrences: &[ParsedExpression],
) -> Vec<CanonicalOccurrenceAssignmentEvent> {
    if !canonical_extra_occurrences.is_empty() {
        return canonical_extra_occurrences
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                let (role, intent, predicate_recognized) = retained_other_assignment(text, index);
                assignment_event(expression, role, intent, predicate_recognized)
            })
            .collect();
    }
    match kind {
        ParsedBodyStatementKind::Return(expression) => vec![assignment_event(
            expression,
            CanonicalExpressionRoleEvent::ReturnValue,
            CanonicalExpressionIntentEvent::Return,
            false,
        )],
        ParsedBodyStatementKind::Binding {
            value: Some(expression),
            ..
        } => vec![assignment_event(
            expression,
            CanonicalExpressionRoleEvent::BindingValue,
            CanonicalExpressionIntentEvent::Binding,
            false,
        )],
        ParsedBodyStatementKind::Binding { value: None, .. } => Vec::new(),
        ParsedBodyStatementKind::Other { expressions } => expressions
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                let (role, intent, predicate_recognized) = retained_other_assignment(text, index);
                assignment_event(expression, role, intent, predicate_recognized)
            })
            .collect(),
    }
}

fn assignment_event(
    expression: &ParsedExpression,
    role: CanonicalExpressionRoleEvent,
    intent: CanonicalExpressionIntentEvent,
    predicate_recognized: bool,
) -> CanonicalOccurrenceAssignmentEvent {
    CanonicalOccurrenceAssignmentEvent {
        expression_node_id: expression.canonical.node_id.clone(),
        role,
        intent,
        predicate_recognized,
    }
}

fn retained_other_assignment(
    text: &str,
    index: usize,
) -> (
    CanonicalExpressionRoleEvent,
    CanonicalExpressionIntentEvent,
    bool,
) {
    if keyword_rest(text, "set").is_some() {
        (
            CanonicalExpressionRoleEvent::SetValue,
            CanonicalExpressionIntentEvent::SetValue,
            false,
        )
    } else if keyword_rest(text, "save").is_some() {
        (
            CanonicalExpressionRoleEvent::SavedValue,
            CanonicalExpressionIntentEvent::SaveValue,
            false,
        )
    } else if keyword_rest(text, "if").is_some() || keyword_rest(text, "while").is_some() {
        (
            CanonicalExpressionRoleEvent::Condition,
            CanonicalExpressionIntentEvent::Condition,
            true,
        )
    } else if keyword_rest(text, "for each").is_some() {
        (
            CanonicalExpressionRoleEvent::LoopCollection,
            CanonicalExpressionIntentEvent::LoopCollection,
            false,
        )
    } else if keyword_rest(text, "for index").is_some() && index == 0 {
        (
            CanonicalExpressionRoleEvent::LoopRangeStart,
            CanonicalExpressionIntentEvent::LoopRangeStart,
            false,
        )
    } else if keyword_rest(text, "for index").is_some() {
        (
            CanonicalExpressionRoleEvent::LoopRangeEnd,
            CanonicalExpressionIntentEvent::LoopRangeEnd,
            false,
        )
    } else if keyword_rest(text, "fail").is_some() {
        (
            CanonicalExpressionRoleEvent::FailureValue,
            CanonicalExpressionIntentEvent::Failure,
            false,
        )
    } else if keyword_rest(text, "expect").is_some() {
        (
            CanonicalExpressionRoleEvent::TestExpectation,
            CanonicalExpressionIntentEvent::TestExpectation,
            false,
        )
    } else {
        (
            CanonicalExpressionRoleEvent::Other,
            CanonicalExpressionIntentEvent::Other,
            false,
        )
    }
}

fn parser_core_shape(
    text: &str,
    kind: &ParsedBodyStatementKind,
) -> (
    &'static str,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
) {
    if text == "}" {
        return ("block_close", "recognized_v0", None, None);
    }
    if matches!(
        text.strip_suffix(':').map(str::trim),
        Some("keeps" | "changes" | "needs" | "ensures" | "watch for" | "cost" | "does")
    ) {
        return (
            "nested_intent_header",
            "recognized_v0",
            None,
            Some("nested_intent_lowering_not_implemented"),
        );
    }
    if let Some(rest) = keyword_rest(text, "if")
        && unquoted_last_non_whitespace(rest) == Some('{')
    {
        return (
            "if_header",
            "recognized_v0",
            Some(parser_expression_kind_for_condition(
                rest.trim_end_matches('{').trim_end(),
            )),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "while")
        && unquoted_last_non_whitespace(rest) == Some('{')
    {
        return (
            "while_header",
            "recognized_v0",
            Some(parser_expression_kind_for_condition(
                rest.trim_end_matches('{').trim_end(),
            )),
            None,
        );
    }
    if text == "loop {" {
        return ("loop_header", "recognized_v0", None, None);
    }
    if let Some(rest) = keyword_rest(text, "for each")
        && unquoted_last_non_whitespace(rest) == Some('{')
    {
        return (
            "for_each_header",
            "recognized_v0",
            Some(parser_expression_kind(
                rest.trim_end_matches('{').trim_end(),
            )),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "for index")
        && unquoted_last_non_whitespace(rest) == Some('{')
    {
        return (
            "for_index_header",
            "recognized_v0",
            Some(parser_expression_kind(
                rest.trim_end_matches('{').trim_end(),
            )),
            None,
        );
    }
    if text == "return" {
        return (
            "return",
            "recognized_v0",
            Some(parser_expression_kind("")),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "return") {
        return (
            "return",
            "recognized_v0",
            Some(parser_expression_kind(rest)),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "fail") {
        return (
            "fail",
            "recognized_v0",
            Some(parser_expression_kind(rest)),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "change") {
        let accepted = matches!(
            kind,
            ParsedBodyStatementKind::Binding { value: Some(_), .. }
        );
        return (
            "mutable_binding",
            if accepted {
                "recognized_v0"
            } else {
                "unsupported_v0"
            },
            rest.split_once('=')
                .map(|(_, value)| parser_expression_kind(value.trim())),
            (!accepted).then_some("binding_missing_initializer"),
        );
    }
    if let Some(rest) = keyword_rest(text, "let") {
        let accepted = matches!(
            kind,
            ParsedBodyStatementKind::Binding { value: Some(_), .. }
        );
        return (
            "let_binding",
            if accepted {
                "recognized_v0"
            } else {
                "unsupported_v0"
            },
            rest.split_once('=')
                .map(|(_, value)| parser_expression_kind(value.trim())),
            (!accepted).then_some("binding_missing_initializer"),
        );
    }
    if let Some(rest) = keyword_rest(text, "set") {
        return (
            "set_place",
            "recognized_v0",
            rest.split_once('=')
                .map(|(_, value)| parser_expression_kind(value.trim())),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "expect") {
        return (
            "test_expectation",
            "recognized_v0",
            Some(parser_expression_kind(rest)),
            Some("test_body_not_core_runtime"),
        );
    }
    if text.starts_with("save ") && text.contains(" in ") {
        return (
            "save_in_store",
            "unsupported_v0",
            None,
            Some("surface_save_requires_store_lowering"),
        );
    }
    if is_record_field_initializer_text(text) {
        return (
            "record_field_initializer",
            "recognized_v0",
            text.split_once(':')
                .map(|(_, value)| parser_expression_kind(value.trim())),
            Some("record_literal_lowering_not_implemented"),
        );
    }
    (
        "unknown_body_line",
        "unsupported_v0",
        None,
        Some("not_in_core_body_grammar_v0"),
    )
}

fn is_record_field_initializer_text(text: &str) -> bool {
    let Some((field, value)) = text.split_once(':') else {
        return false;
    };
    !text.ends_with(':')
        && !field.trim().is_empty()
        && field
            .trim()
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == ' ')
        && !value.trim().is_empty()
}

fn parser_expression_kind_for_condition(text: &str) -> &'static str {
    if parser_has_binary_operator(text) || text.contains(" is ") || text.contains(" does ") {
        "condition_text"
    } else {
        parser_expression_kind(text)
    }
}

fn parser_expression_kind(text: &str) -> &'static str {
    let text = text.trim();
    if typed_failure::is_try_candidate(text) {
        "try_call_like"
    } else if text.is_empty() {
        "unit"
    } else if text == "true" || text == "false" {
        "bool_literal"
    } else if text.chars().all(|ch| ch.is_ascii_digit()) {
        "int_literal"
    } else if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        "text_literal"
    } else if text.ends_with('{') {
        "record_literal_start"
    } else if text.contains('(') && text.contains(')') {
        "call_like"
    } else if parser_has_binary_operator(text) {
        "binary_expression"
    } else if text.contains('.') {
        "path_or_name"
    } else {
        "name_or_text"
    }
}

fn parser_has_binary_operator(text: &str) -> bool {
    [
        " == ", " != ", " <= ", " >= ", " < ", " > ", " + ", " - ", " * ", " / ", " and ", " or ",
    ]
    .iter()
    .any(|operator| text.contains(operator))
}

fn validate_canonical_expression(expression: &CanonicalExpression) -> Result<(), &'static str> {
    let mut identities = std::collections::BTreeSet::new();
    validate_canonical_node(expression, None, &mut identities)
}

fn validate_canonical_node(
    expression: &CanonicalExpression,
    expected_id: Option<&ParserSyntaxNodeId>,
    identities: &mut std::collections::BTreeSet<String>,
) -> Result<(), &'static str> {
    if expected_id.is_some_and(|expected| expected != &expression.node_id)
        || expression.node_id.as_str().is_empty()
        || !identities.insert(expression.node_id.as_str().to_string())
        || (expression.range.byte_len == 0
            && !matches!(expression.kind, CanonicalExpressionKind::Unit))
    {
        return Err("canonical_expression_identity_or_range_corrupt_v0");
    }
    let validate_child =
        |child: &CanonicalExpression,
         role: &str,
         identities: &mut std::collections::BTreeSet<String>| {
            if child.range.start.file != expression.range.start.file
                || child.range.start.line != expression.range.start.line
                || child.range.start.column < expression.range.start.column
                || child.range.start.column + child.range.byte_len
                    > expression.range.start.column + expression.range.byte_len
            {
                return Err("canonical_expression_child_range_corrupt_v0");
            }
            validate_canonical_node(child, Some(&expression.node_id.child(role)), identities)
        };
    match &expression.kind {
        CanonicalExpressionKind::Field { base, .. } => {
            validate_child(base, "field-base", identities)?;
        }
        CanonicalExpressionKind::ListLiteral(values) => {
            for (index, value) in values.iter().enumerate() {
                validate_child(value, &format!("list-item-{index}"), identities)?;
            }
        }
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            for (index, (_, value)) in fields.iter().enumerate() {
                validate_child(value, &format!("record-field-{index}"), identities)?;
            }
        }
        CanonicalExpressionKind::Call { callee, arguments } => {
            validate_child(callee, "call-callee", identities)?;
            for (index, argument) in arguments.iter().enumerate() {
                validate_child(argument, &format!("call-argument-{index}"), identities)?;
            }
        }
        CanonicalExpressionKind::Permission { value, .. } => {
            validate_child(value, "permission-value", identities)?;
        }
        CanonicalExpressionKind::Binary { left, right, .. } => {
            validate_child(left, "binary-left", identities)?;
            validate_child(right, "binary-right", identities)?;
            if left.range.start.column + left.range.byte_len > right.range.start.column {
                return Err("canonical_expression_child_order_corrupt_v0");
            }
        }
        CanonicalExpressionKind::Group(value) => {
            validate_child(value, "group-value", identities)?;
        }
        CanonicalExpressionKind::Unit
        | CanonicalExpressionKind::Identifier(_)
        | CanonicalExpressionKind::UIntLiteral(_)
        | CanonicalExpressionKind::IntLiteral(_)
        | CanonicalExpressionKind::BoolLiteral(_)
        | CanonicalExpressionKind::TextLiteral(_)
        | CanonicalExpressionKind::Unsupported => {}
    }
    Ok(())
}

fn parse_other_statement_expressions(
    text: &str,
    span: &Span,
    source_node_id: &ParserSyntaxNodeId,
) -> Vec<ParsedExpression> {
    let candidate = if let Some(rest) = keyword_rest(text, "set") {
        find_top_level_char(rest, '=')
            .map(|index| (&rest[index + 1..], text.len() - rest.len() + index + 1))
    } else if let Some(rest) = keyword_rest(text, "save") {
        let value = rest.split_once(" in ").map_or(rest, |(value, _)| value);
        Some((value, text.len() - rest.len()))
    } else if let Some(rest) = keyword_rest(text, "expect") {
        Some((rest, text.len() - rest.len()))
    } else if let Some(rest) = keyword_rest(text, "fail") {
        Some((rest, text.len() - rest.len()))
    } else if let Some(rest) = keyword_rest(text, "if") {
        Some((
            rest.trim_end_matches('{').trim_end(),
            text.len() - rest.len(),
        ))
    } else if let Some(rest) = keyword_rest(text, "while") {
        Some((
            rest.trim_end_matches('{').trim_end(),
            text.len() - rest.len(),
        ))
    } else if let Some(rest) = keyword_rest(text, "for each") {
        rest.split_once(" in ").map(|(_, collection)| {
            (
                collection.trim_end_matches('{').trim_end(),
                text.len() - collection.len(),
            )
        })
    } else if text != "}" && !text.ends_with(':') {
        Some((text, 0))
    } else {
        None
    };
    candidate
        .filter(|(expression, _)| !expression.trim().is_empty())
        .map(|(expression, offset)| {
            vec![parse_expression_syntax(
                expression,
                offset_span(span, offset),
                source_node_id.child("expression-0"),
            )]
        })
        .unwrap_or_default()
}

fn parse_for_index_occurrence_expressions(
    text: &str,
    span: &Span,
    source_node_id: &ParserSyntaxNodeId,
) -> Vec<ParsedExpression> {
    if let Some(rest) = keyword_rest(text, "for index") {
        let rest_offset = text.len() - rest.len();
        if let Some(from) = rest.find(" from ") {
            let bounds_offset = rest_offset + from + " from ".len();
            let bounds = &rest[from + " from ".len()..];
            let separator = bounds
                .find(" until ")
                .map(|index| (index, " until ".len()))
                .or_else(|| {
                    bounds
                        .find(" through ")
                        .map(|index| (index, " through ".len()))
                });
            if let Some((separator, separator_len)) = separator {
                let start_raw = &bounds[..separator];
                let end_raw = bounds[separator + separator_len..]
                    .trim_end_matches('{')
                    .trim_end();
                let start = start_raw.trim();
                if !start.is_empty() && !end_raw.is_empty() {
                    let start_leading = start_raw.len() - start_raw.trim_start().len();
                    let end_leading = bounds[separator + separator_len..].len()
                        - bounds[separator + separator_len..].trim_start().len();
                    return vec![
                        parse_expression_syntax(
                            start,
                            offset_span(span, bounds_offset + start_leading),
                            source_node_id.child("canonical-occurrence-expression-0"),
                        ),
                        parse_expression_syntax(
                            end_raw,
                            offset_span(
                                span,
                                bounds_offset + separator + separator_len + end_leading,
                            ),
                            source_node_id.child("canonical-occurrence-expression-1"),
                        ),
                    ];
                }
            }
        }
    }
    Vec::new()
}

struct CanonicalExpressionBuild {
    expression: CanonicalExpression,
    event: CanonicalReductionEvent,
}

fn parsed_expression(
    kind: ParsedExpressionKind,
    text: &str,
    span: Span,
    source_node_id: ParserSyntaxNodeId,
) -> ParsedExpression {
    let canonical = parse_canonical_expression(text, &span, source_node_id, 0);
    let canonical_tokens = canonical_lexical_events(text, &span);
    ParsedExpression {
        kind,
        span,
        canonical: canonical.expression,
        canonical_event: canonical.event,
        canonical_tokens,
    }
}

fn canonical_lexical_events(text: &str, span: &Span) -> Vec<CanonicalLexicalTokenEvent> {
    let leading = text.len() - text.trim_start().len();
    let text = text.trim();
    let span = offset_span(span, leading);
    let mut events = Vec::new();
    let mut index = 0usize;
    while index < text.len() {
        let ch = text[index..]
            .chars()
            .next()
            .expect("token cursor must remain on a character boundary");
        if ch.is_whitespace() {
            index += ch.len_utf8();
            continue;
        }
        let start = index;
        if ch == '"' {
            index += ch.len_utf8();
            let mut escaped = false;
            while index < text.len() {
                let next = text[index..]
                    .chars()
                    .next()
                    .expect("text token cursor must remain on a character boundary");
                index += next.len_utf8();
                if escaped {
                    escaped = false;
                } else if next == '\\' {
                    escaped = true;
                } else if next == '"' {
                    break;
                }
            }
        } else if ch.is_ascii_alphanumeric() || ch == '_' {
            index += ch.len_utf8();
            while index < text.len() {
                let next = text[index..]
                    .chars()
                    .next()
                    .expect("word token cursor must remain on a character boundary");
                if next.is_ascii_alphanumeric() || next == '_' {
                    index += next.len_utf8();
                } else {
                    break;
                }
            }
        } else {
            index += ch.len_utf8();
            if index < text.len()
                && matches!(
                    (ch, text.as_bytes()[index]),
                    ('=', b'=') | ('!', b'=') | ('<', b'=') | ('>', b'=')
                )
            {
                index += 1;
            }
        }
        events.push(CanonicalLexicalTokenEvent {
            range: ParsedSourceRange {
                start: offset_span(&span, start),
                byte_len: index - start,
            },
        });
    }
    events
}

fn parse_expression_syntax(
    text: &str,
    span: Span,
    source_node_id: ParserSyntaxNodeId,
) -> ParsedExpression {
    let leading = text.len() - text.trim_start().len();
    let text = text.trim();
    let span = offset_span(&span, leading);
    for (keyword, permission) in [
        ("borrow", ParamPermission::Borrow),
        ("change", ParamPermission::Change),
        ("consume", ParamPermission::Consume),
    ] {
        if let Some(rest) = keyword_rest(text, keyword) {
            let offset = text.len() - rest.len();
            return parsed_expression(
                ParsedExpressionKind::Permission {
                    permission,
                    value: Box::new(parse_expression_syntax(
                        rest,
                        offset_span(&span, offset),
                        source_node_id.child("permission-value"),
                    )),
                },
                text,
                span,
                source_node_id,
            );
        }
    }
    if is_value_identifier(text) {
        return parsed_expression(
            ParsedExpressionKind::Identifier(ParsedIdentifier {
                name: text.to_string(),
                span: span.clone(),
            }),
            text,
            span,
            source_node_id,
        );
    }
    if !text.is_empty() && text.chars().all(|ch| ch.is_ascii_digit()) {
        return parsed_expression(
            text.parse::<u64>().map_or(
                ParsedExpressionKind::Unsupported {
                    reason: "uint_literal_out_of_range_v0",
                },
                ParsedExpressionKind::UIntLiteral,
            ),
            text,
            span,
            source_node_id,
        );
    }

    let parser_owned_calls = parser_owned_top_level_call_ranges(text);
    if parser_owned_calls.len() > 1
        || parser_owned_calls
            .first()
            .is_some_and(|range| range.start > 0)
    {
        let operands = parser_owned_calls
            .into_iter()
            .enumerate()
            .map(|(index, range)| {
                parse_expression_syntax(
                    &text[range.clone()],
                    offset_span(&span, range.start),
                    source_node_id.child(&format!("compound-{index}")),
                )
            })
            .collect();
        return parsed_expression(
            ParsedExpressionKind::Compound { operands },
            text,
            span,
            source_node_id,
        );
    }

    if let Some(open) = text.find('(') {
        let callee_text = text[..open].trim();
        let callee_offset = text[..open].find(callee_text).unwrap_or_default();
        let callee = parse_expression_syntax(
            callee_text,
            offset_span(&span, callee_offset),
            source_node_id.child("call-callee"),
        );
        let (inside, close, trailing) = match matching_delimiter(text, open, '(', ')') {
            Some(close) => (
                &text[open + 1..close],
                ParsedCallCloseStatus::Closed,
                &text[close + 1..],
            ),
            None => (&text[open + 1..], ParsedCallCloseStatus::Missing, ""),
        };
        let argument_ranges = split_top_level_ranges(inside, ',');
        let argument_separators_hws_valid = argument_ranges.iter().skip(1).all(|range| {
            inside[range.clone()]
                .as_bytes()
                .first()
                .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
        });
        let arguments = argument_ranges
            .into_iter()
            .enumerate()
            .filter_map(|(index, range)| {
                let raw = &inside[range.clone()];
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return None;
                }
                let leading = raw.len() - raw.trim_start().len();
                Some(parse_expression_syntax(
                    trimmed,
                    offset_span(&span, open + 1 + range.start + leading),
                    source_node_id.child(&format!("call-argument-{index}")),
                ))
            })
            .collect();
        let trailing = classify_call_trailing(trailing);
        return parsed_expression(
            ParsedExpressionKind::Call(ParsedCall {
                callee: Box::new(callee),
                arguments,
                argument_separators_hws_valid,
                close_status: close,
                trailing_status: trailing,
            }),
            text,
            span,
            source_node_id,
        );
    }

    if let Some(open) = text.find('[')
        && text[open + 1..].contains(')')
    {
        let callee_text = text[..open].trim();
        let callee = parse_expression_syntax(
            callee_text,
            span.clone(),
            source_node_id.child("call-callee"),
        );
        return parsed_expression(
            ParsedExpressionKind::Call(ParsedCall {
                callee: Box::new(callee),
                arguments: Vec::new(),
                argument_separators_hws_valid: true,
                close_status: ParsedCallCloseStatus::Mismatched,
                trailing_status: ParsedCallTrailingStatus::Complete,
            }),
            text,
            span,
            source_node_id,
        );
    }

    let operands = compound_identifier_operands(text, &span, &source_node_id);
    if !operands.is_empty() {
        return parsed_expression(
            ParsedExpressionKind::Compound { operands },
            text,
            span,
            source_node_id,
        );
    }

    parsed_expression(
        if text.contains("task") || text.contains(')') || text.contains('(') {
            ParsedExpressionKind::Unsupported {
                reason: "unsupported_callable_expression_shape_v0",
            }
        } else {
            ParsedExpressionKind::Other
        },
        text,
        span,
        source_node_id,
    )
}

fn canonical_expression_build(
    node_id: ParserSyntaxNodeId,
    range: ParsedSourceRange,
    kind: CanonicalExpressionKind,
    event_kind: CanonicalCommonNodeKind,
    children: Vec<CanonicalReductionChildEvent>,
    delimiter_depth: usize,
) -> CanonicalExpressionBuild {
    CanonicalExpressionBuild {
        expression: CanonicalExpression {
            node_id,
            range: range.clone(),
            kind,
        },
        event: CanonicalReductionEvent {
            range,
            kind: event_kind,
            children,
            delimiter_depth_before: delimiter_depth,
            delimiter_depth_after: delimiter_depth,
            lexical_status: if event_kind == CanonicalCommonNodeKind::Unsupported {
                CanonicalCommonLexicalStatus::Unsupported
            } else {
                CanonicalCommonLexicalStatus::Complete
            },
        },
    }
}

fn reduction_child(
    role: CanonicalCommonChildRole,
    ordinal: usize,
    build: &CanonicalExpressionBuild,
) -> CanonicalReductionChildEvent {
    CanonicalReductionChildEvent {
        role,
        ordinal,
        event: Box::new(build.event.clone()),
    }
}

fn parse_canonical_expression(
    text: &str,
    span: &Span,
    node_id: ParserSyntaxNodeId,
    delimiter_depth: usize,
) -> CanonicalExpressionBuild {
    let leading = text.len() - text.trim_start().len();
    let text = text.trim();
    let span = offset_span(span, leading);
    let range = ParsedSourceRange {
        start: span.clone(),
        byte_len: text.len(),
    };

    if text.is_empty() {
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::Unit,
            CanonicalCommonNodeKind::Unit,
            Vec::new(),
            delimiter_depth,
        );
    }

    for (keyword, permission) in [
        ("borrow", ParamPermission::Borrow),
        ("change", ParamPermission::Change),
        ("consume", ParamPermission::Consume),
    ] {
        if let Some(rest) = keyword_rest(text, keyword) {
            let offset = text.len() - rest.len();
            let value = parse_canonical_expression(
                rest,
                &offset_span(&span, offset),
                node_id.child("permission-value"),
                delimiter_depth,
            );
            let children = vec![reduction_child(
                CanonicalCommonChildRole::PermissionValue,
                0,
                &value,
            )];
            return canonical_expression_build(
                node_id,
                range,
                CanonicalExpressionKind::Permission {
                    permission,
                    value: Box::new(value.expression),
                },
                CanonicalCommonNodeKind::Permission,
                children,
                delimiter_depth,
            );
        }
    }

    if let Ok(value) = text.parse::<i64>()
        && value < 0
    {
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::IntLiteral(value),
            CanonicalCommonNodeKind::IntLiteral,
            Vec::new(),
            delimiter_depth,
        );
    }

    if let Some((operator, start, end)) = top_level_binary_operator(text) {
        let left = parse_canonical_expression(
            &text[..start],
            &span,
            node_id.child("binary-left"),
            delimiter_depth,
        );
        let right = parse_canonical_expression(
            &text[end..],
            &offset_span(&span, end),
            node_id.child("binary-right"),
            delimiter_depth,
        );
        let children = vec![
            reduction_child(CanonicalCommonChildRole::BinaryLeft, 0, &left),
            reduction_child(CanonicalCommonChildRole::BinaryRight, 1, &right),
        ];
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::Binary {
                operator,
                left: Box::new(left.expression),
                right: Box::new(right.expression),
            },
            CanonicalCommonNodeKind::Binary,
            children,
            delimiter_depth,
        );
    }

    if text.starts_with('(')
        && matching_delimiter_quoted(text, 0, '(', ')') == Some(text.len().saturating_sub(1))
    {
        let value = parse_canonical_expression(
            &text[1..text.len() - 1],
            &offset_span(&span, 1),
            node_id.child("group-value"),
            delimiter_depth + 1,
        );
        let children = vec![reduction_child(
            CanonicalCommonChildRole::GroupValue,
            0,
            &value,
        )];
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::Group(Box::new(value.expression)),
            CanonicalCommonNodeKind::Group,
            children,
            delimiter_depth,
        );
    }

    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::TextLiteral(text[1..text.len() - 1].to_string()),
            CanonicalCommonNodeKind::TextLiteral,
            Vec::new(),
            delimiter_depth,
        );
    }
    if let Ok(value) = text.parse::<u64>() {
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::UIntLiteral(value),
            CanonicalCommonNodeKind::UIntLiteral,
            Vec::new(),
            delimiter_depth,
        );
    }
    if matches!(text, "true" | "false") {
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::BoolLiteral(text == "true"),
            CanonicalCommonNodeKind::BoolLiteral,
            Vec::new(),
            delimiter_depth,
        );
    }

    if text.starts_with('[')
        && matching_delimiter_quoted(text, 0, '[', ']') == Some(text.len().saturating_sub(1))
    {
        let inside = &text[1..text.len() - 1];
        let builds = split_top_level_ranges_quoted(inside, ',')
            .into_iter()
            .enumerate()
            .filter_map(|(index, value_range)| {
                let raw = &inside[value_range.clone()];
                (!raw.trim().is_empty()).then(|| {
                    parse_canonical_expression(
                        raw,
                        &offset_span(&span, 1 + value_range.start),
                        node_id.child(&format!("list-item-{index}")),
                        delimiter_depth + 1,
                    )
                })
            })
            .collect::<Vec<_>>();
        let children = builds
            .iter()
            .enumerate()
            .map(|(index, build)| {
                reduction_child(CanonicalCommonChildRole::ListElement, index, build)
            })
            .collect();
        let values = builds.into_iter().map(|build| build.expression).collect();
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::ListLiteral(values),
            CanonicalCommonNodeKind::ListLiteral,
            children,
            delimiter_depth,
        );
    }

    if text.ends_with('{') && is_type_identifier(text[..text.len() - 1].trim()) {
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::RecordLiteral {
                name: text[..text.len() - 1].trim().to_string(),
                fields: Vec::new(),
            },
            CanonicalCommonNodeKind::RecordLiteral,
            Vec::new(),
            delimiter_depth,
        );
    }

    if let Some(open) = text.find('{')
        && matching_delimiter_quoted(text, open, '{', '}') == Some(text.len().saturating_sub(1))
        && (text[..open].trim().is_empty() || is_type_identifier(text[..open].trim()))
    {
        let name = text[..open].trim().to_string();
        let inside = &text[open + 1..text.len() - 1];
        let builds = split_top_level_ranges_quoted(inside, ',')
            .into_iter()
            .enumerate()
            .filter_map(|(index, field_range)| {
                let raw = &inside[field_range.clone()];
                let (field, value) = raw.split_once(':')?;
                let field = field.trim();
                if !is_value_identifier(field) || value.trim().is_empty() {
                    return None;
                }
                let value_offset = raw.find(value).unwrap_or_default();
                Some((
                    field.to_string(),
                    parse_canonical_expression(
                        value,
                        &offset_span(&span, open + 1 + field_range.start + value_offset),
                        node_id.child(&format!("record-field-{index}")),
                        delimiter_depth + 1,
                    ),
                ))
            })
            .collect::<Vec<_>>();
        let children = builds
            .iter()
            .enumerate()
            .map(|(index, (_, build))| {
                reduction_child(CanonicalCommonChildRole::RecordFieldValue, index, build)
            })
            .collect();
        let fields = builds
            .into_iter()
            .map(|(field, build)| (field, build.expression))
            .collect();
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::RecordLiteral { name, fields },
            CanonicalCommonNodeKind::RecordLiteral,
            children,
            delimiter_depth,
        );
    }

    if let Some(open) = find_top_level_open_paren(text)
        && matching_delimiter_quoted(text, open, '(', ')') == Some(text.len().saturating_sub(1))
    {
        let inside = &text[open + 1..text.len() - 1];
        let callee = parse_canonical_expression(
            &text[..open],
            &span,
            node_id.child("call-callee"),
            delimiter_depth,
        );
        let argument_builds = split_top_level_ranges_quoted(inside, ',')
            .into_iter()
            .enumerate()
            .filter_map(|(index, argument_range)| {
                let raw = &inside[argument_range.clone()];
                (!raw.trim().is_empty()).then(|| {
                    parse_canonical_expression(
                        raw,
                        &offset_span(&span, open + 1 + argument_range.start),
                        node_id.child(&format!("call-argument-{index}")),
                        delimiter_depth + 1,
                    )
                })
            })
            .collect::<Vec<_>>();
        let mut children = vec![reduction_child(
            CanonicalCommonChildRole::CallCallee,
            0,
            &callee,
        )];
        children.extend(argument_builds.iter().enumerate().map(|(index, build)| {
            reduction_child(CanonicalCommonChildRole::CallArgument, index, build)
        }));
        let arguments = argument_builds
            .into_iter()
            .map(|build| build.expression)
            .collect();
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::Call {
                callee: Box::new(callee.expression),
                arguments,
            },
            CanonicalCommonNodeKind::Call,
            children,
            delimiter_depth,
        );
    }

    if let Some(dot) = find_top_level_dot(text) {
        let field = text[dot + 1..].trim();
        if is_value_identifier(field) {
            let base = parse_canonical_expression(
                &text[..dot],
                &span,
                node_id.child("field-base"),
                delimiter_depth,
            );
            let children = vec![reduction_child(
                CanonicalCommonChildRole::FieldBase,
                0,
                &base,
            )];
            return canonical_expression_build(
                node_id,
                range,
                CanonicalExpressionKind::Field {
                    base: Box::new(base.expression),
                    field: field.to_string(),
                },
                CanonicalCommonNodeKind::Field,
                children,
                delimiter_depth,
            );
        }
    }
    let (kind, event_kind) = if is_value_identifier(text) {
        (
            CanonicalExpressionKind::Identifier(text.to_string()),
            CanonicalCommonNodeKind::Identifier,
        )
    } else {
        (
            CanonicalExpressionKind::Unsupported,
            CanonicalCommonNodeKind::Unsupported,
        )
    };
    canonical_expression_build(
        node_id,
        range,
        kind,
        event_kind,
        Vec::new(),
        delimiter_depth,
    )
}

fn top_level_binary_operator(text: &str) -> Option<(ParsedBinaryOperator, usize, usize)> {
    let operators: &[(&str, ParsedBinaryOperator, u8)] = &[
        ("or", ParsedBinaryOperator::Or, 1),
        ("and", ParsedBinaryOperator::And, 2),
        ("==", ParsedBinaryOperator::Equal, 3),
        ("!=", ParsedBinaryOperator::NotEqual, 3),
        ("<=", ParsedBinaryOperator::LessEqual, 3),
        (">=", ParsedBinaryOperator::GreaterEqual, 3),
        ("<", ParsedBinaryOperator::Less, 3),
        (">", ParsedBinaryOperator::Greater, 3),
        ("fails with", ParsedBinaryOperator::FailsWith, 3),
        ("returns", ParsedBinaryOperator::Returns, 3),
        ("does", ParsedBinaryOperator::Does, 3),
        ("is", ParsedBinaryOperator::Is, 3),
        ("+", ParsedBinaryOperator::Add, 4),
        ("-", ParsedBinaryOperator::Subtract, 4),
        ("*", ParsedBinaryOperator::Multiply, 5),
        ("/", ParsedBinaryOperator::Divide, 5),
    ];
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    let mut best: Option<(ParsedBinaryOperator, usize, usize, u8)> = None;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 => {
                for (spelling, operator, precedence) in operators {
                    if text[index..].starts_with(spelling)
                        && operator_boundary(text, index, spelling)
                        && !is_unary_sign(text, index, spelling)
                        && best.is_none_or(|(_, _, _, current)| *precedence <= current)
                    {
                        best = Some((*operator, index, index + spelling.len(), *precedence));
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    best.map(|(operator, start, end, _)| (operator, start, end))
}

fn is_unary_sign(text: &str, start: usize, spelling: &str) -> bool {
    if !matches!(spelling, "+" | "-") {
        return false;
    }
    let before = text[..start].chars().rev().find(|ch| !ch.is_whitespace());
    before.is_none_or(|ch| {
        matches!(
            ch,
            '(' | '[' | '{' | ',' | '=' | '!' | '<' | '>' | '+' | '-' | '*' | '/'
        )
    })
}

fn operator_boundary(text: &str, start: usize, spelling: &str) -> bool {
    if !matches!(
        spelling,
        "and" | "or" | "is" | "does" | "returns" | "fails with"
    ) {
        return true;
    }
    let before = text[..start].chars().next_back();
    let after = text[start + spelling.len()..].chars().next();
    before.is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'))
        && after.is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'))
}

fn find_top_level_open_paren(text: &str) -> Option<usize> {
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        if ch == '"' {
            quoted = true;
        } else if ch == '(' {
            return Some(index);
        }
    }
    None
}

fn find_top_level_dot(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    let mut found = None;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '.' if depth == 0 => found = Some(index),
            _ => {}
        }
    }
    found
}

fn parser_owned_top_level_call_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
    let bytes = text.as_bytes();
    let mut calls = Vec::new();
    let mut index = 0;
    let mut quoted = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            quoted = true;
            index += 1;
            continue;
        }
        if !(byte.is_ascii_lowercase() || byte == b'_')
            || index.checked_sub(1).is_some_and(|previous| {
                bytes[previous].is_ascii_alphanumeric() || matches!(bytes[previous], b'_' | b'.')
            })
        {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
        {
            index += 1;
        }
        let callee = &text[start..index];
        let mut open = index;
        while open < bytes.len() && bytes[open].is_ascii_whitespace() {
            open += 1;
        }
        if !is_executable_callee(callee) || open >= bytes.len() || bytes[open] != b'(' {
            continue;
        }
        let end = matching_delimiter(text, open, '(', ')')
            .map_or(text.len(), |close| close + ')'.len_utf8());
        calls.push(start..end);
    }

    calls
        .iter()
        .filter(|call| {
            !calls
                .iter()
                .any(|candidate| candidate.start < call.start && call.end <= candidate.end)
        })
        .cloned()
        .collect()
}

fn compound_identifier_operands(
    text: &str,
    span: &Span,
    source_node_id: &ParserSyntaxNodeId,
) -> Vec<ParsedExpression> {
    let bytes = text.as_bytes();
    let mut operands = Vec::new();
    let mut index = 0;
    let mut quoted = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            quoted = true;
            index += 1;
            continue;
        }
        if byte.is_ascii_lowercase() || byte == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_lowercase()
                    || bytes[index].is_ascii_digit()
                    || bytes[index] == b'_')
            {
                index += 1;
            }
            let name = &text[start..index];
            if is_value_identifier(name) {
                let identifier_span = offset_span(span, start);
                operands.push(parsed_expression(
                    ParsedExpressionKind::Identifier(ParsedIdentifier {
                        name: name.to_string(),
                        span: identifier_span.clone(),
                    }),
                    name,
                    identifier_span,
                    source_node_id.child(&format!("compound-{}", operands.len())),
                ));
            }
            continue;
        }
        index += 1;
    }
    operands
}

fn classify_call_trailing(trailing: &str) -> ParsedCallTrailingStatus {
    let trailing = trailing.trim();
    if trailing.is_empty() {
        ParsedCallTrailingStatus::Complete
    } else if trailing.chars().all(|ch| ch == ')') {
        ParsedCallTrailingStatus::ExtraClose
    } else if trailing.starts_with('(') {
        ParsedCallTrailingStatus::Chained
    } else {
        ParsedCallTrailingStatus::Prose
    }
}

fn parse_type_syntax(text: &str, span: Span) -> TypeSyntax {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix("Result ")
        && let Some(comma) = find_top_level_char(rest, ',')
    {
        let value_text = rest[..comma].trim();
        let root = rest[comma + 1..].trim();
        return TypeSyntax {
            kind: TypeSyntaxKind::Result {
                value: Box::new(parse_type_syntax(value_text, span.clone())),
                failure_root: root.to_string(),
            },
            span,
        };
    }
    if text.starts_with("task") {
        return TypeSyntax {
            kind: parse_callable_type_syntax(text, &span),
            span,
        };
    }
    TypeSyntax {
        kind: if is_type_identifier(text) {
            TypeSyntaxKind::Named {
                name: text.to_string(),
            }
        } else {
            TypeSyntaxKind::Other
        },
        span,
    }
}

fn parse_callable_type_syntax(text: &str, span: &Span) -> TypeSyntaxKind {
    let Some(rest) = text.strip_prefix("task(") else {
        return TypeSyntaxKind::CallableCandidate {
            reason: "callable_type_requires_task_open_paren_v0",
        };
    };
    let open = "task".len();
    let Some(close) = matching_delimiter(text, open, '(', ')') else {
        return TypeSyntaxKind::CallableCandidate {
            reason: "callable_type_missing_close_paren_v0",
        };
    };
    let after = &text[close + 1..];
    let leading = after.len() - after.trim_start_matches([' ', '\t']).len();
    if leading == 0 {
        return TypeSyntaxKind::CallableCandidate {
            reason: "callable_type_requires_space_before_arrow_v0",
        };
    }
    let after = &after[leading..];
    let Some(result_text) = after.strip_prefix("->") else {
        return TypeSyntaxKind::CallableCandidate {
            reason: "callable_type_missing_arrow_v0",
        };
    };
    let result_leading = result_text.len() - result_text.trim_start_matches([' ', '\t']).len();
    if result_leading == 0 || result_text[result_leading..].is_empty() {
        return TypeSyntaxKind::CallableCandidate {
            reason: "callable_type_requires_result_v0",
        };
    }
    let inside = &rest[..close - open - 1];
    let inputs = split_top_level_ranges(inside, ',')
        .into_iter()
        .filter_map(|range| {
            let raw = &inside[range.clone()];
            let value = raw.trim();
            (!value.is_empty())
                .then(|| parse_type_syntax(value, offset_span(span, open + 1 + range.start)))
        })
        .collect();
    let result = parse_type_syntax(
        result_text[result_leading..].trim(),
        offset_span(span, close + 1 + leading + 2 + result_leading),
    );
    TypeSyntaxKind::Callable(CallableTypeSyntax {
        inputs,
        result: Box::new(result),
    })
}

fn keyword_rest<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    text.strip_prefix(keyword)
        .and_then(|rest| rest.strip_prefix([' ', '\t']))
        .map(|rest| rest.trim_start_matches([' ', '\t']))
}

fn find_top_level_arrow(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index + 1 < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b'-' if bytes[index + 1] == b'>' && depth == 0 => return Some(index),
            _ => {}
        }
        index += 1;
    }
    None
}

fn matching_delimiter(text: &str, open: usize, open_ch: char, close_ch: char) -> Option<usize> {
    matching_delimiter_quoted(text, open, open_ch, close_ch)
}

fn matching_delimiter_quoted(
    text: &str,
    open: usize,
    open_ch: char,
    close_ch: char,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices().skip_while(|(index, _)| *index < open) {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
        } else if ch == '"' {
            quoted = true;
        } else if ch == open_ch {
            depth += 1;
        } else if ch == close_ch {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn split_top_level_ranges(text: &str, delimiter: char) -> Vec<std::ops::Range<usize>> {
    split_top_level_ranges_quoted(text, delimiter)
}

fn split_top_level_ranges_quoted(text: &str, delimiter: char) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if ch == delimiter && depth == 0 => {
                ranges.push(start..index);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    ranges.push(start..text.len());
    ranges
}

fn find_top_level_char(text: &str, needle: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if ch == needle && depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

fn unquoted_last_non_whitespace(text: &str) -> Option<char> {
    let mut quoted = false;
    let mut escaped = false;
    let mut last = None;
    for ch in text.chars() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        if ch == '"' {
            quoted = true;
        } else if !ch.is_whitespace() {
            last = Some(ch);
        }
    }
    last
}

fn offset_span(span: &Span, byte_offset: usize) -> Span {
    Span::new(span.file.clone(), span.line, span.column + byte_offset)
}

fn is_value_identifier(text: &str) -> bool {
    is_valid_identifier(text, IdentifierKind::Value)
}

fn is_type_identifier(text: &str) -> bool {
    is_valid_identifier(text, IdentifierKind::Type)
}

fn parse_param_permission(raw_name: &str) -> (ParamPermission, bool, &str) {
    let raw_name = raw_name.trim();
    let Some(first) = raw_name.split_whitespace().next() else {
        return (ParamPermission::Borrow, false, raw_name);
    };
    let permission = match first {
        "borrow" => ParamPermission::Borrow,
        "change" => ParamPermission::Change,
        "consume" => ParamPermission::Consume,
        _ => return (ParamPermission::Borrow, false, raw_name),
    };
    let name = raw_name[first.len()..].trim();
    (permission, true, name)
}

fn is_valid_identifier(name: &str, kind: IdentifierKind) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    match kind {
        IdentifierKind::Value => {
            (first.is_ascii_lowercase() || first == '_')
                && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        }
        IdentifierKind::Type => {
            first.is_ascii_uppercase() && chars.all(|ch| ch.is_ascii_alphanumeric())
        }
    }
}

fn identifier_suggestion(name: &str, kind: IdentifierKind) -> String {
    match kind {
        IdentifierKind::Value => snake_identifier(name),
        IdentifierKind::Type => pascal_identifier(name),
    }
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
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        "name".to_string()
    } else if out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        format!("_{out}")
    } else {
        out
    }
}

fn pascal_identifier(text: &str) -> String {
    let words = text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    if words.is_empty() {
        return "Name".to_string();
    }

    let mut out = String::new();
    for word in words {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            for ch in chars {
                out.push(ch);
            }
        }
    }
    out
}

fn first_visible_column(text: &str) -> usize {
    text.chars()
        .position(|ch| !ch.is_whitespace())
        .map_or(1, |index| index + 1)
}

fn parse_store_header(header: &str) -> (String, String) {
    let rest = header.trim_start_matches("store ").trim();
    match rest.split_once(':') {
        Some((name, ty)) => (name.trim().to_string(), ty.trim().to_string()),
        None => (rest.to_string(), String::new()),
    }
}

fn count_indent(text: &str) -> usize {
    text.chars().take_while(|ch| *ch == ' ').count()
}

fn is_ignorable(trimmed: &str) -> bool {
    trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//")
}

fn is_section_header(trimmed: &str) -> bool {
    if !trimmed.ends_with(':') || trimmed.len() <= 1 {
        return false;
    }
    let name = trimmed.trim_end_matches(':').trim();
    !name.is_empty() && name.chars().all(|ch| ch.is_ascii_alphabetic() || ch == ' ')
}

#[cfg(test)]
mod tests {
    use super::{
        CanonicalAssigningEvent, CanonicalAuthorityHandle, CanonicalExpressionIntent,
        CanonicalExpressionRole, CanonicalItemOwner, CanonicalNodeIdentity,
        CanonicalOccurrenceIdentity, CanonicalOccurrenceSeal, CanonicalPredicateRecognition,
        CanonicalReductionIdentity, CanonicalSealFact, CanonicalSectionOwner,
        CanonicalSemanticFile, CanonicalSourceBlob, CanonicalSourceOwnerFact,
        CanonicalSourceOwnerSeal, CanonicalSourceRevision, CanonicalStatementOwner,
        CanonicalTokenIdentity, build_occurrence_seal, executable_call_nodes, parse_source,
        parse_source_at_index, source_owner_fact_matches, validate_canonical_expression,
        validate_occurrence_seal, validate_occurrence_seal_ignoring_one_fact,
        validate_retained_body_syntax, validate_source_owner_seal,
    };
    use crate::ast::{
        CanonicalCommonChildRole, CanonicalCommonLexicalStatus, CanonicalCommonNodeKind,
        CanonicalExpressionKind, Item, ParsedBinaryOperator, ParsedBlockRelationship,
        ParsedBodyStatementKind, ParsedCallCloseStatus, ParsedExpressionKind, ParsedSourceRange,
        ParserSyntaxNodeId, TypeSyntaxKind,
    };
    use crate::diagnostic::{DiagnosticCode, Severity};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum CanonicalSealField {
        SourceBlob,
        SemanticFile,
        SourceRevision,
        Item,
        Section,
        Statement,
        AuthorityHandle,
    }

    const CANONICAL_SEAL_FIELDS: [CanonicalSealField; 7] = [
        CanonicalSealField::SourceBlob,
        CanonicalSealField::SemanticFile,
        CanonicalSealField::SourceRevision,
        CanonicalSealField::Item,
        CanonicalSealField::Section,
        CanonicalSealField::Statement,
        CanonicalSealField::AuthorityHandle,
    ];

    #[derive(Debug, Clone, Copy)]
    enum ProjectionMutation {
        Corrupt,
        Missing,
        Duplicate,
        Reordered,
        Extra,
        Substituted,
    }

    const PROJECTION_MUTATIONS: [ProjectionMutation; 6] = [
        ProjectionMutation::Corrupt,
        ProjectionMutation::Missing,
        ProjectionMutation::Duplicate,
        ProjectionMutation::Reordered,
        ProjectionMutation::Extra,
        ProjectionMutation::Substituted,
    ];

    #[derive(Debug, Clone, Copy)]
    enum MatrixSabotage {
        ProducerArm,
        ValidatorArm,
        CatalogueRow,
        MutationOperator,
        PairCase,
        EqualLengthEvidence,
        ForeignOwnerEvidence,
    }

    fn fact_field(fact: &CanonicalSourceOwnerFact) -> CanonicalSealField {
        match fact {
            CanonicalSourceOwnerFact::SourceBlob(_) => CanonicalSealField::SourceBlob,
            CanonicalSourceOwnerFact::SemanticFile(_) => CanonicalSealField::SemanticFile,
            CanonicalSourceOwnerFact::SourceRevision(_) => CanonicalSealField::SourceRevision,
            CanonicalSourceOwnerFact::Item(_) => CanonicalSealField::Item,
            CanonicalSourceOwnerFact::Section(_) => CanonicalSealField::Section,
            CanonicalSourceOwnerFact::Statement(_) => CanonicalSealField::Statement,
            CanonicalSourceOwnerFact::AuthorityHandle(_) => CanonicalSealField::AuthorityHandle,
        }
    }

    fn corrupt_bytes(bytes: &std::sync::Arc<[u8]>) -> std::sync::Arc<[u8]> {
        let mut corrupted = bytes.to_vec();
        corrupted.push(0xff);
        corrupted.into()
    }

    fn corrupted_fact(fact: &CanonicalSourceOwnerFact) -> CanonicalSourceOwnerFact {
        macro_rules! corrupt_id {
            ($variant:ident, $kind:ident, $value:ident) => {{
                let mut identity = $value.0.clone();
                identity.domain ^= 0x80;
                CanonicalSourceOwnerFact::$variant($kind(identity))
            }};
        }
        match fact {
            CanonicalSourceOwnerFact::SourceBlob(value) => {
                corrupt_id!(SourceBlob, CanonicalSourceBlob, value)
            }
            CanonicalSourceOwnerFact::SemanticFile(value) => {
                corrupt_id!(SemanticFile, CanonicalSemanticFile, value)
            }
            CanonicalSourceOwnerFact::SourceRevision(value) => {
                CanonicalSourceOwnerFact::SourceRevision(CanonicalSourceRevision(corrupt_bytes(
                    &value.0,
                )))
            }
            CanonicalSourceOwnerFact::Item(value) => corrupt_id!(Item, CanonicalItemOwner, value),
            CanonicalSourceOwnerFact::Section(value) => {
                corrupt_id!(Section, CanonicalSectionOwner, value)
            }
            CanonicalSourceOwnerFact::Statement(value) => {
                corrupt_id!(Statement, CanonicalStatementOwner, value)
            }
            CanonicalSourceOwnerFact::AuthorityHandle(value) => {
                corrupt_id!(AuthorityHandle, CanonicalAuthorityHandle, value)
            }
        }
    }

    fn mutate_projection(
        seal: &CanonicalSourceOwnerSeal,
        foreign: &CanonicalSourceOwnerSeal,
        index: usize,
        mutation: ProjectionMutation,
    ) -> CanonicalSourceOwnerSeal {
        let mut mutated = seal.clone();
        match mutation {
            ProjectionMutation::Corrupt => {
                mutated.projection[index] = corrupted_fact(&mutated.projection[index]);
            }
            ProjectionMutation::Missing => {
                mutated.projection.remove(index);
            }
            ProjectionMutation::Duplicate => {
                mutated
                    .projection
                    .insert(index, mutated.projection[index].clone());
            }
            ProjectionMutation::Reordered => {
                let other = if index == 6 { 5 } else { index + 1 };
                mutated.projection.swap(index, other);
            }
            ProjectionMutation::Extra => {
                mutated.projection.push(foreign.projection[index].clone());
            }
            ProjectionMutation::Substituted => {
                mutated.projection[index] = foreign.projection[index].clone();
            }
        }
        mutated
    }

    #[derive(Debug, PartialEq, Eq)]
    struct SourceOwnerEvidence {
        inventory: Vec<Vec<CanonicalSourceOwnerFact>>,
        projection_reconstruction_rejected: bool,
        single_rejections: usize,
        pair_rejections: usize,
        foreign_rejections: usize,
        equal_length_distinct: bool,
    }

    fn source_owner_evidence(sabotage: Option<MatrixSabotage>) -> SourceOwnerEvidence {
        const SOURCE_A: &str =
            "# a\ntask same() -> UInt {\n  does:\n    return 1\n    return 1\n}\n";
        const SOURCE_B: &str =
            "# b\ntask same() -> UInt {\n  does:\n    return 1\n    return 1\n}\n";
        let foreign_source = if matches!(sabotage, Some(MatrixSabotage::EqualLengthEvidence)) {
            SOURCE_A
        } else {
            SOURCE_B
        };
        let first = parse_source("same.hum", SOURCE_A);
        let foreign = parse_source("same.hum", foreign_source);
        let renamed = parse_source("renamed.hum", SOURCE_A);
        assert!(first.diagnostics.is_empty() && foreign.diagnostics.is_empty());
        assert_eq!(
            first.file, foreign.file,
            "public owner projections must stay compatible"
        );
        assert_eq!(
            first.source_owner_seals, renamed.source_owner_seals,
            "filename must not mint identity"
        );
        let base = &first.source_owner_seals[0];
        let sibling = &first.source_owner_seals[1];
        let other = &foreign.source_owner_seals[0];
        let catalogue = if matches!(sabotage, Some(MatrixSabotage::CatalogueRow)) {
            &CANONICAL_SEAL_FIELDS[..6]
        } else {
            &CANONICAL_SEAL_FIELDS[..]
        };
        let operators = if matches!(sabotage, Some(MatrixSabotage::MutationOperator)) {
            &PROJECTION_MUTATIONS[..5]
        } else {
            &PROJECTION_MUTATIONS[..]
        };
        let mut reconstructed = mutate_projection(base, other, 0, ProjectionMutation::Corrupt);
        if matches!(sabotage, Some(MatrixSabotage::ProducerArm)) {
            reconstructed = base.clone();
        }
        if let CanonicalSourceOwnerFact::SourceBlob(value) = &reconstructed.projection[0] {
            reconstructed.authority.source_blob = value.clone();
        }
        let projection_reconstruction_rejected =
            validate_source_owner_seal(&reconstructed).is_err();
        let mut single_rejections = 0;
        for (index, field) in catalogue.iter().copied().enumerate() {
            assert_eq!(field, fact_field(&base.projection[index]));
            single_rejections += operators
                .iter()
                .filter(|mutation| {
                    let mutated = mutate_projection(base, other, index, **mutation);
                    if matches!(sabotage, Some(MatrixSabotage::ValidatorArm))
                        && index == 6
                        && matches!(**mutation, ProjectionMutation::Corrupt)
                    {
                        mutated.projection[..6]
                            .iter()
                            .enumerate()
                            .any(|(index, fact)| {
                                !source_owner_fact_matches(&mutated.authority, index, fact)
                            })
                    } else {
                        validate_source_owner_seal(&mutated).is_err()
                    }
                })
                .count();
        }
        let pair_limit = if matches!(sabotage, Some(MatrixSabotage::PairCase)) {
            20
        } else {
            21
        };
        let mut pair_rejections = 0;
        for left in 0..catalogue.len() {
            for right in left + 1..catalogue.len() {
                if pair_rejections == pair_limit {
                    break;
                }
                let mut pair = base.clone();
                pair.projection[left] = other.projection[left].clone();
                pair.projection[right] = other.projection[right].clone();
                pair_rejections += usize::from(validate_source_owner_seal(&pair).is_err());
            }
        }
        let mut foreign_cases = (0..catalogue.len())
            .map(|index| mutate_projection(base, other, index, ProjectionMutation::Substituted))
            .collect::<Vec<_>>();
        for index in [5, 6] {
            let mut cross_owner = base.clone();
            cross_owner.projection[index] = sibling.projection[index].clone();
            foreign_cases.push(cross_owner);
        }
        if matches!(sabotage, Some(MatrixSabotage::ForeignOwnerEvidence)) {
            foreign_cases.pop();
        }
        SourceOwnerEvidence {
            inventory: first
                .source_owner_seals
                .iter()
                .chain(foreign.source_owner_seals.iter())
                .map(|seal| seal.projection.clone())
                .collect(),
            projection_reconstruction_rejected,
            single_rejections,
            pair_rejections,
            foreign_rejections: foreign_cases
                .iter()
                .filter(|seal| validate_source_owner_seal(seal).is_err())
                .count(),
            equal_length_distinct: SOURCE_A.len() == foreign_source.len()
                && SOURCE_A.as_bytes() != foreign_source.as_bytes()
                && base.projection != other.projection,
        }
    }

    fn complete_source_owner_evidence(evidence: &SourceOwnerEvidence) -> bool {
        evidence.projection_reconstruction_rejected
            && evidence.single_rejections == 42
            && evidence.pair_rejections == 21
            && evidence.foreign_rejections == 9
            && evidence.equal_length_distinct
    }

    #[test]
    fn source_owner_authority_kernel_is_complete_and_load_bearing() {
        let first = source_owner_evidence(None);
        let second = source_owner_evidence(None);
        assert_eq!(
            first, second,
            "fresh private inventories must be deterministic"
        );
        assert!(complete_source_owner_evidence(&first));
        for sabotage in [
            MatrixSabotage::ProducerArm,
            MatrixSabotage::ValidatorArm,
            MatrixSabotage::CatalogueRow,
            MatrixSabotage::MutationOperator,
            MatrixSabotage::PairCase,
            MatrixSabotage::EqualLengthEvidence,
            MatrixSabotage::ForeignOwnerEvidence,
        ] {
            assert!(
                !complete_source_owner_evidence(&source_owner_evidence(Some(sabotage))),
                "{sabotage:?} sabotage stayed green"
            );
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum F1SealField {
        SourceBlob,
        SemanticFile,
        SourceRevision,
        Item,
        Section,
        Statement,
        AuthorityHandle,
        Occurrence,
        ExpressionRole,
        Root,
        RootRange,
        RootReduction,
        Intent,
        AssigningEvent,
        AssignmentSyntaxNode,
        PredicateRecognition,
        PreorderCount,
        MaximumDelimiterDepth,
        NodeIdentity,
        NodeOccurrence,
        TokenIdentity,
        ReductionIdentity,
        ParentIdentity,
        ChildRole,
        ChildOrdinal,
        PreorderOrdinal,
        NodeSource,
        NodeRange,
        TokenInterval,
        Kind,
        OrderedChildren,
        ChildCardinality,
        DelimiterDepthBefore,
        DelimiterDepthAfter,
        LexicalStatus,
        IllegalFieldsAbsent,
    }

    const F1_SEAL_FIELDS: [F1SealField; 36] = [
        F1SealField::SourceBlob,
        F1SealField::SemanticFile,
        F1SealField::SourceRevision,
        F1SealField::Item,
        F1SealField::Section,
        F1SealField::Statement,
        F1SealField::AuthorityHandle,
        F1SealField::Occurrence,
        F1SealField::ExpressionRole,
        F1SealField::Root,
        F1SealField::RootRange,
        F1SealField::RootReduction,
        F1SealField::Intent,
        F1SealField::AssigningEvent,
        F1SealField::AssignmentSyntaxNode,
        F1SealField::PredicateRecognition,
        F1SealField::PreorderCount,
        F1SealField::MaximumDelimiterDepth,
        F1SealField::NodeIdentity,
        F1SealField::NodeOccurrence,
        F1SealField::TokenIdentity,
        F1SealField::ReductionIdentity,
        F1SealField::ParentIdentity,
        F1SealField::ChildRole,
        F1SealField::ChildOrdinal,
        F1SealField::PreorderOrdinal,
        F1SealField::NodeSource,
        F1SealField::NodeRange,
        F1SealField::TokenInterval,
        F1SealField::Kind,
        F1SealField::OrderedChildren,
        F1SealField::ChildCardinality,
        F1SealField::DelimiterDepthBefore,
        F1SealField::DelimiterDepthAfter,
        F1SealField::LexicalStatus,
        F1SealField::IllegalFieldsAbsent,
    ];

    const F1_COMMON_KINDS: [CanonicalCommonNodeKind; 14] = [
        CanonicalCommonNodeKind::Unit,
        CanonicalCommonNodeKind::Identifier,
        CanonicalCommonNodeKind::Field,
        CanonicalCommonNodeKind::UIntLiteral,
        CanonicalCommonNodeKind::IntLiteral,
        CanonicalCommonNodeKind::BoolLiteral,
        CanonicalCommonNodeKind::TextLiteral,
        CanonicalCommonNodeKind::ListLiteral,
        CanonicalCommonNodeKind::RecordLiteral,
        CanonicalCommonNodeKind::Call,
        CanonicalCommonNodeKind::Permission,
        CanonicalCommonNodeKind::Binary,
        CanonicalCommonNodeKind::Group,
        CanonicalCommonNodeKind::Unsupported,
    ];

    const F1_CHILD_ROLES: [CanonicalCommonChildRole; 9] = [
        CanonicalCommonChildRole::FieldBase,
        CanonicalCommonChildRole::ListElement,
        CanonicalCommonChildRole::RecordFieldValue,
        CanonicalCommonChildRole::CallCallee,
        CanonicalCommonChildRole::CallArgument,
        CanonicalCommonChildRole::PermissionValue,
        CanonicalCommonChildRole::BinaryLeft,
        CanonicalCommonChildRole::BinaryRight,
        CanonicalCommonChildRole::GroupValue,
    ];

    const F1_ROLES: [CanonicalExpressionRole; 13] = [
        CanonicalExpressionRole::ReturnValue,
        CanonicalExpressionRole::BindingValue,
        CanonicalExpressionRole::SetValue,
        CanonicalExpressionRole::SavedValue,
        CanonicalExpressionRole::Condition,
        CanonicalExpressionRole::LoopCollection,
        CanonicalExpressionRole::LoopRangeStart,
        CanonicalExpressionRole::LoopRangeEnd,
        CanonicalExpressionRole::FailureValue,
        CanonicalExpressionRole::TestExpectation,
        CanonicalExpressionRole::NeedsPredicate,
        CanonicalExpressionRole::EnsuresPredicate,
        CanonicalExpressionRole::Other,
    ];

    const F1_SOURCE_A: &str = r#"# alpha
task seal(value: UInt) -> UInt {
  needs:
    value > 0
  ensures:
    value > 0
  does:
    return
    return call((value + 1), [value, 2], Thing { field: value }, target.field, borrow value, -1, true, "x")
    return call(value)
    return call(value)
    let first = value
    change second = 1
    set target = value
    save value in store
    if value > 0 {
      }
    while value < 2 {
      }
    for each item in [value] {
      }
    for index index from 0 until 2 {
      }
    fail "bad"
    expect true
    @
}
"#;

    const F1_SOURCE_B: &str = r#"# bravo
task seal(value: UInt) -> UInt {
  needs:
    value > 0
  ensures:
    value > 0
  does:
    return
    return call((value + 1), [value, 2], Thing { field: value }, target.field, borrow value, -1, true, "x")
    return call(value)
    return call(value)
    let first = value
    change second = 1
    set target = value
    save value in store
    if value > 0 {
      }
    while value < 2 {
      }
    for each item in [value] {
      }
    for index index from 0 until 2 {
      }
    fail "bad"
    expect true
    @
}
"#;

    fn f1_fact_field(fact: &CanonicalSealFact) -> F1SealField {
        match fact {
            CanonicalSealFact::SourceBlob(_) => F1SealField::SourceBlob,
            CanonicalSealFact::SemanticFile(_) => F1SealField::SemanticFile,
            CanonicalSealFact::SourceRevision(_) => F1SealField::SourceRevision,
            CanonicalSealFact::Item(_) => F1SealField::Item,
            CanonicalSealFact::Section(_) => F1SealField::Section,
            CanonicalSealFact::Statement(_) => F1SealField::Statement,
            CanonicalSealFact::AuthorityHandle(_) => F1SealField::AuthorityHandle,
            CanonicalSealFact::Occurrence(_) => F1SealField::Occurrence,
            CanonicalSealFact::ExpressionRole(_) => F1SealField::ExpressionRole,
            CanonicalSealFact::Root(_) => F1SealField::Root,
            CanonicalSealFact::RootRange(_) => F1SealField::RootRange,
            CanonicalSealFact::RootReduction(_) => F1SealField::RootReduction,
            CanonicalSealFact::Intent(_) => F1SealField::Intent,
            CanonicalSealFact::AssigningEvent(_) => F1SealField::AssigningEvent,
            CanonicalSealFact::AssignmentSyntaxNode(_, _) => F1SealField::AssignmentSyntaxNode,
            CanonicalSealFact::PredicateRecognition(_) => F1SealField::PredicateRecognition,
            CanonicalSealFact::PreorderCount(_) => F1SealField::PreorderCount,
            CanonicalSealFact::MaximumDelimiterDepth(_) => F1SealField::MaximumDelimiterDepth,
            CanonicalSealFact::NodeIdentity(_) => F1SealField::NodeIdentity,
            CanonicalSealFact::NodeOccurrence(_, _) => F1SealField::NodeOccurrence,
            CanonicalSealFact::TokenIdentity(_) => F1SealField::TokenIdentity,
            CanonicalSealFact::ReductionIdentity(_, _) => F1SealField::ReductionIdentity,
            CanonicalSealFact::ParentIdentity(_, _) => F1SealField::ParentIdentity,
            CanonicalSealFact::ChildRole(_, _) => F1SealField::ChildRole,
            CanonicalSealFact::ChildOrdinal(_, _) => F1SealField::ChildOrdinal,
            CanonicalSealFact::PreorderOrdinal(_, _) => F1SealField::PreorderOrdinal,
            CanonicalSealFact::NodeSource(_, _) => F1SealField::NodeSource,
            CanonicalSealFact::NodeRange(_, _) => F1SealField::NodeRange,
            CanonicalSealFact::TokenInterval(_, _) => F1SealField::TokenInterval,
            CanonicalSealFact::Kind(_, _) => F1SealField::Kind,
            CanonicalSealFact::OrderedChildren(_, _) => F1SealField::OrderedChildren,
            CanonicalSealFact::ChildCardinality(_, _) => F1SealField::ChildCardinality,
            CanonicalSealFact::DelimiterDepthBefore(_, _) => F1SealField::DelimiterDepthBefore,
            CanonicalSealFact::DelimiterDepthAfter(_, _) => F1SealField::DelimiterDepthAfter,
            CanonicalSealFact::LexicalStatus(_, _) => F1SealField::LexicalStatus,
            CanonicalSealFact::IllegalFieldsAbsent(_) => F1SealField::IllegalFieldsAbsent,
        }
    }

    fn corrupt_owner_identity(
        identity: &super::CanonicalOwnerIdentity,
    ) -> super::CanonicalOwnerIdentity {
        let mut identity = identity.clone();
        identity.domain ^= 0x40;
        identity
    }

    fn corrupt_node(value: &CanonicalNodeIdentity) -> CanonicalNodeIdentity {
        CanonicalNodeIdentity(corrupt_owner_identity(&value.0))
    }

    fn corrupt_token(value: &CanonicalTokenIdentity) -> CanonicalTokenIdentity {
        CanonicalTokenIdentity(corrupt_owner_identity(&value.0))
    }

    fn corrupt_reduction(value: &CanonicalReductionIdentity) -> CanonicalReductionIdentity {
        CanonicalReductionIdentity(corrupt_owner_identity(&value.0))
    }

    fn corrupt_occurrence(value: &CanonicalOccurrenceIdentity) -> CanonicalOccurrenceIdentity {
        CanonicalOccurrenceIdentity(corrupt_owner_identity(&value.0))
    }

    fn corrupt_assigning(value: &CanonicalAssigningEvent) -> CanonicalAssigningEvent {
        CanonicalAssigningEvent(corrupt_owner_identity(&value.0))
    }

    fn corrupt_range(value: &ParsedSourceRange) -> ParsedSourceRange {
        let mut value = value.clone();
        value.byte_len = value.byte_len.saturating_add(1);
        value
    }

    fn corrupted_f1_fact(fact: &CanonicalSealFact) -> CanonicalSealFact {
        macro_rules! corrupt_owner_fact {
            ($variant:ident, $kind:ident, $value:ident) => {
                CanonicalSealFact::$variant($kind(corrupt_owner_identity(&$value.0)))
            };
        }
        match fact {
            CanonicalSealFact::SourceBlob(value) => {
                corrupt_owner_fact!(SourceBlob, CanonicalSourceBlob, value)
            }
            CanonicalSealFact::SemanticFile(value) => {
                corrupt_owner_fact!(SemanticFile, CanonicalSemanticFile, value)
            }
            CanonicalSealFact::SourceRevision(value) => {
                CanonicalSealFact::SourceRevision(CanonicalSourceRevision(corrupt_bytes(&value.0)))
            }
            CanonicalSealFact::Item(value) => corrupt_owner_fact!(Item, CanonicalItemOwner, value),
            CanonicalSealFact::Section(value) => {
                corrupt_owner_fact!(Section, CanonicalSectionOwner, value)
            }
            CanonicalSealFact::Statement(value) => {
                corrupt_owner_fact!(Statement, CanonicalStatementOwner, value)
            }
            CanonicalSealFact::AuthorityHandle(value) => {
                corrupt_owner_fact!(AuthorityHandle, CanonicalAuthorityHandle, value)
            }
            CanonicalSealFact::Occurrence(value) => {
                CanonicalSealFact::Occurrence(corrupt_occurrence(value))
            }
            CanonicalSealFact::ExpressionRole(value) => {
                CanonicalSealFact::ExpressionRole(if *value == CanonicalExpressionRole::Other {
                    CanonicalExpressionRole::ReturnValue
                } else {
                    CanonicalExpressionRole::Other
                })
            }
            CanonicalSealFact::Root(value) => CanonicalSealFact::Root(corrupt_node(value)),
            CanonicalSealFact::RootRange(value) => {
                CanonicalSealFact::RootRange(corrupt_range(value))
            }
            CanonicalSealFact::RootReduction(value) => {
                CanonicalSealFact::RootReduction(corrupt_reduction(value))
            }
            CanonicalSealFact::Intent(value) => {
                CanonicalSealFact::Intent(if *value == CanonicalExpressionIntent::Other {
                    CanonicalExpressionIntent::Return
                } else {
                    CanonicalExpressionIntent::Other
                })
            }
            CanonicalSealFact::AssigningEvent(value) => {
                CanonicalSealFact::AssigningEvent(corrupt_assigning(value))
            }
            CanonicalSealFact::AssignmentSyntaxNode(event, node) => {
                CanonicalSealFact::AssignmentSyntaxNode(event.clone(), node.child("corrupt"))
            }
            CanonicalSealFact::PredicateRecognition(value) => {
                CanonicalSealFact::PredicateRecognition(match value {
                    CanonicalPredicateRecognition::Present(event) => {
                        CanonicalPredicateRecognition::Absent(event.clone())
                    }
                    CanonicalPredicateRecognition::Absent(event) => {
                        CanonicalPredicateRecognition::Present(event.clone())
                    }
                })
            }
            CanonicalSealFact::PreorderCount(value) => CanonicalSealFact::PreorderCount(value + 1),
            CanonicalSealFact::MaximumDelimiterDepth(value) => {
                CanonicalSealFact::MaximumDelimiterDepth(value + 1)
            }
            CanonicalSealFact::NodeIdentity(value) => {
                CanonicalSealFact::NodeIdentity(corrupt_node(value))
            }
            CanonicalSealFact::NodeOccurrence(node, occurrence) => {
                CanonicalSealFact::NodeOccurrence(node.clone(), corrupt_occurrence(occurrence))
            }
            CanonicalSealFact::TokenIdentity(value) => {
                CanonicalSealFact::TokenIdentity(corrupt_token(value))
            }
            CanonicalSealFact::ReductionIdentity(node, reduction) => {
                CanonicalSealFact::ReductionIdentity(node.clone(), corrupt_reduction(reduction))
            }
            CanonicalSealFact::ParentIdentity(node, parent) => CanonicalSealFact::ParentIdentity(
                node.clone(),
                Some(
                    parent
                        .as_ref()
                        .map_or_else(|| corrupt_node(node), corrupt_node),
                ),
            ),
            CanonicalSealFact::ChildRole(node, role) => CanonicalSealFact::ChildRole(
                node.clone(),
                if role.is_some() {
                    None
                } else {
                    Some(CanonicalCommonChildRole::BinaryLeft)
                },
            ),
            CanonicalSealFact::ChildOrdinal(node, ordinal) => {
                CanonicalSealFact::ChildOrdinal(node.clone(), Some(ordinal.unwrap_or_default() + 1))
            }
            CanonicalSealFact::PreorderOrdinal(node, ordinal) => {
                CanonicalSealFact::PreorderOrdinal(node.clone(), ordinal + 1)
            }
            CanonicalSealFact::NodeSource(node, source) => CanonicalSealFact::NodeSource(
                node.clone(),
                CanonicalSemanticFile(corrupt_owner_identity(&source.0)),
            ),
            CanonicalSealFact::NodeRange(node, range) => {
                CanonicalSealFact::NodeRange(node.clone(), corrupt_range(range))
            }
            CanonicalSealFact::TokenInterval(node, interval) => CanonicalSealFact::TokenInterval(
                node.clone(),
                interval.as_ref().map_or_else(
                    || {
                        let token = CanonicalTokenIdentity(super::source_owner_identity(
                            9,
                            &CanonicalSourceRevision(vec![0xff].into()),
                            &[0],
                        ));
                        Some((token.clone(), token))
                    },
                    |(first, last)| Some((corrupt_token(first), corrupt_token(last))),
                ),
            ),
            CanonicalSealFact::Kind(node, kind) => CanonicalSealFact::Kind(
                node.clone(),
                if *kind == CanonicalCommonNodeKind::Unit {
                    CanonicalCommonNodeKind::Identifier
                } else {
                    CanonicalCommonNodeKind::Unit
                },
            ),
            CanonicalSealFact::OrderedChildren(node, children) => {
                let mut children = children.clone();
                if children.len() > 1 {
                    children.reverse();
                } else {
                    children.push(corrupt_node(node));
                }
                CanonicalSealFact::OrderedChildren(node.clone(), children)
            }
            CanonicalSealFact::ChildCardinality(node, count) => {
                CanonicalSealFact::ChildCardinality(node.clone(), count + 1)
            }
            CanonicalSealFact::DelimiterDepthBefore(node, depth) => {
                CanonicalSealFact::DelimiterDepthBefore(node.clone(), depth + 1)
            }
            CanonicalSealFact::DelimiterDepthAfter(node, depth) => {
                CanonicalSealFact::DelimiterDepthAfter(node.clone(), depth + 1)
            }
            CanonicalSealFact::LexicalStatus(node, status) => CanonicalSealFact::LexicalStatus(
                node.clone(),
                if *status == CanonicalCommonLexicalStatus::Complete {
                    CanonicalCommonLexicalStatus::Unsupported
                } else {
                    CanonicalCommonLexicalStatus::Complete
                },
            ),
            CanonicalSealFact::IllegalFieldsAbsent(node) => {
                CanonicalSealFact::IllegalFieldsAbsent(corrupt_node(node))
            }
        }
    }

    fn f1_representatives(
        seal: &CanonicalOccurrenceSeal,
    ) -> std::collections::BTreeMap<F1SealField, usize> {
        let mut representatives = std::collections::BTreeMap::new();
        for (index, fact) in seal.projection.iter().enumerate() {
            representatives.entry(f1_fact_field(fact)).or_insert(index);
        }
        representatives
    }

    fn foreign_fact(
        field: F1SealField,
        base: &CanonicalSealFact,
        foreign: &[CanonicalOccurrenceSeal],
    ) -> CanonicalSealFact {
        foreign
            .iter()
            .flat_map(|seal| &seal.projection)
            .find(|fact| f1_fact_field(fact) == field && *fact != base)
            .cloned()
            .unwrap_or_else(|| panic!("foreign corpus lacks distinct {field:?} evidence"))
    }

    fn mutate_f1_projection(
        seal: &CanonicalOccurrenceSeal,
        foreign: &[CanonicalOccurrenceSeal],
        index: usize,
        mutation: ProjectionMutation,
    ) -> CanonicalOccurrenceSeal {
        let mut mutated = seal.clone();
        let field = f1_fact_field(&mutated.projection[index]);
        let replacement = foreign_fact(field, &mutated.projection[index], foreign);
        match mutation {
            ProjectionMutation::Corrupt => {
                mutated.projection[index] = corrupted_f1_fact(&mutated.projection[index]);
            }
            ProjectionMutation::Missing => {
                mutated.projection.remove(index);
            }
            ProjectionMutation::Duplicate => {
                mutated
                    .projection
                    .insert(index, mutated.projection[index].clone());
            }
            ProjectionMutation::Reordered => {
                let other = if index + 1 == mutated.projection.len() {
                    index - 1
                } else {
                    index + 1
                };
                mutated.projection.swap(index, other);
            }
            ProjectionMutation::Extra => mutated.projection.push(replacement),
            ProjectionMutation::Substituted => mutated.projection[index] = replacement,
        }
        mutated
    }

    #[derive(Debug, Clone, Copy)]
    enum F1Sabotage {
        ProducerArm,
        ValidatorArm,
        CatalogueRow,
        MutationOperator,
        PairCase,
        EqualLengthEvidence,
        CrossOccurrence,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct F1Evidence {
        deterministic: bool,
        public_compatible: bool,
        equal_length_distinct: bool,
        field_count: usize,
        kind_count: usize,
        role_count: usize,
        child_role_count: usize,
        lexical_status_count: usize,
        node_count: usize,
        token_count: usize,
        single_rejections: usize,
        pair_rejections: usize,
        cross_occurrence_rejections: usize,
        producer_event_rejected: bool,
        same_line_distinct: bool,
        horizontal_whitespace_compatible: bool,
        unit_position_exact: bool,
        root_token_interval_complete: bool,
        preorder_order_exact: bool,
    }

    fn occurrence_fact(seal: &CanonicalOccurrenceSeal, field: F1SealField) -> &CanonicalSealFact {
        seal.projection
            .iter()
            .find(|fact| f1_fact_field(fact) == field)
            .unwrap_or_else(|| panic!("occurrence lacks {field:?}"))
    }

    fn f1_evidence(sabotage: Option<F1Sabotage>) -> F1Evidence {
        let identity_foreign_source = if matches!(sabotage, Some(F1Sabotage::EqualLengthEvidence)) {
            F1_SOURCE_A
        } else {
            F1_SOURCE_B
        };
        let parsed = parse_source("f1.hum", F1_SOURCE_A);
        let repeated = parse_source("f1.hum", F1_SOURCE_A);
        let foreign = parse_source("f1.hum", F1_SOURCE_B);
        let identity_foreign = parse_source("f1.hum", identity_foreign_source);
        let renamed = parse_source("renamed.hum", F1_SOURCE_A);
        assert_eq!(F1_SOURCE_A.len(), identity_foreign_source.len());
        assert_eq!(parsed.file, repeated.file);
        assert_eq!(parsed.diagnostics, repeated.diagnostics);
        assert_eq!(parsed.file, foreign.file);
        assert_eq!(
            parsed.occurrence_seals.len(),
            renamed.occurrence_seals.len()
        );
        for (left, right) in parsed
            .occurrence_seals
            .iter()
            .zip(&renamed.occurrence_seals)
        {
            for field in [
                F1SealField::Occurrence,
                F1SealField::Root,
                F1SealField::RootReduction,
                F1SealField::AssigningEvent,
                F1SealField::NodeIdentity,
                F1SealField::TokenIdentity,
                F1SealField::ReductionIdentity,
            ] {
                assert_eq!(
                    left.projection
                        .iter()
                        .filter(|fact| f1_fact_field(fact) == field)
                        .collect::<Vec<_>>(),
                    right
                        .projection
                        .iter()
                        .filter(|fact| f1_fact_field(fact) == field)
                        .collect::<Vec<_>>(),
                    "filename changed private {field:?} identity"
                );
            }
        }
        let base = parsed
            .occurrence_seals
            .iter()
            .max_by_key(
                |seal| match occurrence_fact(seal, F1SealField::PreorderCount) {
                    CanonicalSealFact::PreorderCount(count) => *count,
                    _ => unreachable!(),
                },
            )
            .expect("F1 corpus occurrence");
        let catalogue = if matches!(sabotage, Some(F1Sabotage::CatalogueRow)) {
            &F1_SEAL_FIELDS[..35]
        } else {
            &F1_SEAL_FIELDS[..]
        };
        let operators = if matches!(sabotage, Some(F1Sabotage::MutationOperator)) {
            &PROJECTION_MUTATIONS[..5]
        } else {
            &PROJECTION_MUTATIONS[..]
        };
        let representatives = f1_representatives(base);
        assert_eq!(representatives.len(), 36);
        let mut single_rejections = 0usize;
        for field in catalogue {
            let index = representatives[field];
            for mutation in operators {
                let mutated =
                    mutate_f1_projection(base, &foreign.occurrence_seals, index, *mutation);
                let rejected = if matches!(sabotage, Some(F1Sabotage::ValidatorArm))
                    && *field == F1SealField::IllegalFieldsAbsent
                    && matches!(mutation, ProjectionMutation::Corrupt)
                {
                    validate_occurrence_seal_ignoring_one_fact(&mutated, index).is_err()
                } else {
                    validate_occurrence_seal(&mutated).is_err()
                };
                single_rejections += usize::from(rejected);
            }
        }
        let mut pair_rejections = 0usize;
        let pair_limit = if matches!(sabotage, Some(F1Sabotage::PairCase)) {
            629
        } else {
            630
        };
        'pairs: for left in 0..catalogue.len() {
            for right in left + 1..catalogue.len() {
                if pair_rejections == pair_limit {
                    break 'pairs;
                }
                let mut pair = base.clone();
                for field in [catalogue[left], catalogue[right]] {
                    let index = representatives[&field];
                    pair.projection[index] =
                        foreign_fact(field, &pair.projection[index], &foreign.occurrence_seals);
                }
                pair_rejections += usize::from(validate_occurrence_seal(&pair).is_err());
            }
        }
        let same_shaped = parsed
            .occurrence_seals
            .iter()
            .filter(|seal| {
                matches!(
                    occurrence_fact(seal, F1SealField::ExpressionRole),
                    CanonicalSealFact::ExpressionRole(CanonicalExpressionRole::ReturnValue)
                ) && matches!(
                    occurrence_fact(seal, F1SealField::PreorderCount),
                    CanonicalSealFact::PreorderCount(3)
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(same_shaped.len(), 2);
        const CROSS_FIELDS: [F1SealField; 15] = [
            F1SealField::Statement,
            F1SealField::AuthorityHandle,
            F1SealField::Occurrence,
            F1SealField::Root,
            F1SealField::RootRange,
            F1SealField::RootReduction,
            F1SealField::AssigningEvent,
            F1SealField::AssignmentSyntaxNode,
            F1SealField::PredicateRecognition,
            F1SealField::NodeIdentity,
            F1SealField::NodeOccurrence,
            F1SealField::TokenIdentity,
            F1SealField::ReductionIdentity,
            F1SealField::TokenInterval,
            F1SealField::OrderedChildren,
        ];
        let base_sibling = same_shaped[0];
        let foreign_sibling = same_shaped[1];
        let sibling_representatives = f1_representatives(base_sibling);
        let mut cross_cases = CROSS_FIELDS
            .iter()
            .map(|field| {
                let index = sibling_representatives[field];
                let mut substituted = base_sibling.clone();
                substituted.projection[index] = foreign_sibling.projection[index].clone();
                substituted
            })
            .collect::<Vec<_>>();
        let mut complete_substitution = base_sibling.clone();
        complete_substitution.projection = foreign_sibling.projection.clone();
        cross_cases.push(complete_substitution);
        if matches!(sabotage, Some(F1Sabotage::CrossOccurrence)) {
            cross_cases.pop();
        }
        let cross_occurrence_rejections = cross_cases
            .iter()
            .filter(|seal| validate_occurrence_seal(seal).is_err())
            .count();
        let leaf_source = "task leaf(value: UInt) -> UInt {\n  does:\n    return value\n}\n";
        let leaf = parse_source("leaf.hum", leaf_source);
        let Item::Task(task) = &leaf.file.items[0] else {
            panic!("leaf task")
        };
        let statement = &task.body_syntax[0];
        let ParsedBodyStatementKind::Return(expression) = &statement.kind else {
            panic!("leaf return")
        };
        let mut corrupted_expression = expression.clone();
        corrupted_expression.canonical_event.kind = CanonicalCommonNodeKind::Unsupported;
        corrupted_expression.canonical_event.lexical_status =
            CanonicalCommonLexicalStatus::Unsupported;
        if matches!(sabotage, Some(F1Sabotage::ProducerArm)) {
            corrupted_expression.canonical.kind = CanonicalExpressionKind::Unsupported;
        }
        let corrupted_seal = build_occurrence_seal(
            &corrupted_expression,
            &leaf.source_owner_seals[0],
            CanonicalExpressionRole::ReturnValue,
            CanonicalExpressionIntent::Return,
            &statement.canonical_assignments[0],
            0,
        );
        let same_line = parsed
            .occurrence_seals
            .windows(2)
            .find(|pair| {
                matches!(
                    occurrence_fact(&pair[0], F1SealField::ExpressionRole),
                    CanonicalSealFact::ExpressionRole(CanonicalExpressionRole::LoopRangeStart)
                ) && matches!(
                    occurrence_fact(&pair[1], F1SealField::ExpressionRole),
                    CanonicalSealFact::ExpressionRole(CanonicalExpressionRole::LoopRangeEnd)
                )
            })
            .expect("same-line loop roots");
        let tabbed = parse_source(
            "tabbed.hum",
            "task tabbed(value: UInt) -> UInt {\n  does:\n    set\ttarget = value\n    return value\n}\n",
        );
        let unit = parsed
            .occurrence_seals
            .iter()
            .find(|seal| {
                matches!(
                    occurrence_fact(seal, F1SealField::Kind),
                    CanonicalSealFact::Kind(_, CanonicalCommonNodeKind::Unit)
                )
            })
            .expect("Unit occurrence");
        let token_ids = base
            .projection
            .iter()
            .filter_map(|fact| match fact {
                CanonicalSealFact::TokenIdentity(identity) => Some(identity),
                _ => None,
            })
            .collect::<Vec<_>>();
        F1Evidence {
            deterministic: parsed.occurrence_seals == repeated.occurrence_seals,
            public_compatible: parsed.file == repeated.file
                && parsed.diagnostics == repeated.diagnostics,
            equal_length_distinct: F1_SOURCE_A.as_bytes() != identity_foreign_source.as_bytes()
                && parsed.occurrence_seals != identity_foreign.occurrence_seals,
            field_count: catalogue.len(),
            kind_count: F1_COMMON_KINDS
                .iter()
                .filter(|kind| {
                    parsed.occurrence_seals.iter().any(|seal| {
                        seal.projection.iter().any(|fact| {
                            matches!(fact, CanonicalSealFact::Kind(_, actual) if actual == *kind)
                        })
                    })
                })
                .count(),
            role_count: F1_ROLES
                .iter()
                .filter(|role| {
                    parsed.occurrence_seals.iter().any(|seal| {
                        matches!(
                            occurrence_fact(seal, F1SealField::ExpressionRole),
                            CanonicalSealFact::ExpressionRole(actual) if actual == *role
                        )
                    })
                })
                .count(),
            child_role_count: F1_CHILD_ROLES
                .iter()
                .filter(|role| {
                    parsed.occurrence_seals.iter().any(|seal| {
                        seal.projection.iter().any(|fact| {
                            matches!(fact, CanonicalSealFact::ChildRole(_, Some(actual)) if actual == *role)
                        })
                    })
                })
                .count(),
            lexical_status_count: [
                CanonicalCommonLexicalStatus::Complete,
                CanonicalCommonLexicalStatus::Unsupported,
            ]
            .iter()
            .filter(|status| {
                parsed.occurrence_seals.iter().any(|seal| {
                    seal.projection.iter().any(|fact| {
                        matches!(fact, CanonicalSealFact::LexicalStatus(_, actual) if actual == *status)
                    })
                })
            })
            .count(),
            node_count: parsed
                .occurrence_seals
                .iter()
                .map(
                    |seal| match occurrence_fact(seal, F1SealField::PreorderCount) {
                        CanonicalSealFact::PreorderCount(count) => *count,
                        _ => unreachable!(),
                    },
                )
                .sum(),
            token_count: parsed
                .occurrence_seals
                .iter()
                .flat_map(|seal| &seal.projection)
                .filter(|fact| f1_fact_field(fact) == F1SealField::TokenIdentity)
                .count(),
            single_rejections,
            pair_rejections,
            cross_occurrence_rejections,
            producer_event_rejected: validate_occurrence_seal(&corrupted_seal).is_err(),
            same_line_distinct: occurrence_fact(&same_line[0], F1SealField::Occurrence)
                != occurrence_fact(&same_line[1], F1SealField::Occurrence),
            horizontal_whitespace_compatible: matches!(
                occurrence_fact(&tabbed.occurrence_seals[0], F1SealField::ExpressionRole),
                CanonicalSealFact::ExpressionRole(CanonicalExpressionRole::SetValue)
            ),
            unit_position_exact: matches!(
                occurrence_fact(unit, F1SealField::RootRange),
                CanonicalSealFact::RootRange(range) if range.byte_len == 0
            ) && matches!(
                occurrence_fact(unit, F1SealField::TokenInterval),
                CanonicalSealFact::TokenInterval(_, None)
            ) && matches!(
                occurrence_fact(unit, F1SealField::ChildCardinality),
                CanonicalSealFact::ChildCardinality(_, 0)
            ) && !unit
                .projection
                .iter()
                .any(|fact| matches!(fact, CanonicalSealFact::TokenIdentity(_))),
            root_token_interval_complete: matches!(
                occurrence_fact(base, F1SealField::TokenInterval),
                CanonicalSealFact::TokenInterval(_, Some((first, last)))
                    if Some(first) == token_ids.first().copied()
                        && Some(last) == token_ids.last().copied()
            ),
            preorder_order_exact: parsed.occurrence_seals.iter().all(|seal| {
                seal.projection
                    .iter()
                    .filter_map(|fact| match fact {
                        CanonicalSealFact::PreorderOrdinal(_, ordinal) => Some(*ordinal),
                        _ => None,
                    })
                    .eq(0..match occurrence_fact(seal, F1SealField::PreorderCount) {
                        CanonicalSealFact::PreorderCount(count) => *count,
                        _ => unreachable!(),
                    })
            }),
        }
    }

    fn complete_f1_evidence(evidence: &F1Evidence) -> bool {
        evidence.deterministic
            && evidence.public_compatible
            && evidence.equal_length_distinct
            && evidence.field_count == 36
            && evidence.kind_count == 14
            && evidence.role_count == 13
            && evidence.child_role_count == 9
            && evidence.lexical_status_count == 2
            && evidence.node_count == 48
            && evidence.token_count == 67
            && evidence.single_rejections == 216
            && evidence.pair_rejections == 630
            && evidence.cross_occurrence_rejections == 16
            && evidence.producer_event_rejected
            && evidence.same_line_distinct
            && evidence.horizontal_whitespace_compatible
            && evidence.unit_position_exact
            && evidence.root_token_interval_complete
            && evidence.preorder_order_exact
    }

    #[test]
    fn occurrence_authority_and_common_node_topology_are_complete_and_load_bearing() {
        let first = f1_evidence(None);
        let second = f1_evidence(None);
        assert_eq!(first, second, "fresh F1 inventories must be deterministic");
        assert!(complete_f1_evidence(&first), "{first:#?}");
        for sabotage in [
            F1Sabotage::ProducerArm,
            F1Sabotage::ValidatorArm,
            F1Sabotage::CatalogueRow,
            F1Sabotage::MutationOperator,
            F1Sabotage::PairCase,
            F1Sabotage::EqualLengthEvidence,
            F1Sabotage::CrossOccurrence,
        ] {
            assert!(
                !complete_f1_evidence(&f1_evidence(Some(sabotage))),
                "{sabotage:?} sabotage stayed green"
            );
        }
    }

    #[test]
    fn parser_body_syntax_owns_repeated_sibling_and_nested_calls() {
        let parsed = parse_source(
            "parser-owned-calls.hum",
            r#"task caller(value: UInt) -> UInt {
  does:
    return leaf(value) + leaf(value) + leaf(leaf(consume value))
}
"#,
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let calls = executable_call_nodes(&task.body_syntax);
        assert_eq!(calls.len(), 4);
        assert_eq!(
            calls
                .iter()
                .map(|call| call.position.stable_component())
                .collect::<Vec<_>>(),
            [
                "statement-0:path-0.0",
                "statement-0:path-0.1",
                "statement-0:path-0.2",
                "statement-0:path-0.2.0",
            ]
        );
        assert_eq!(
            calls
                .iter()
                .map(|call| call.source.as_str())
                .collect::<Vec<_>>(),
            [
                "leaf(value)",
                "leaf(value)",
                "leaf(leaf(consume value))",
                "leaf(consume value)",
            ]
        );
        assert!(
            calls[3]
                .identifier_uses
                .iter()
                .any(|identifier| identifier.name == "value" && identifier.consumed)
        );
    }

    #[test]
    fn parser_occurrence_identity_uses_only_producer_owned_file_and_event_facts() {
        let first = parse_source_at_index("display-one.hum", "unexpected prose\n", 3);
        let renamed = parse_source_at_index("renamed-display.hum", "unexpected prose\n", 3);
        let other_file = parse_source_at_index("display-one.hum", "unexpected prose\n", 4);
        let first = first
            .diagnostic_occurrences
            .occurrences()
            .next()
            .expect("parser occurrence");
        let renamed = renamed
            .diagnostic_occurrences
            .occurrences()
            .next()
            .expect("renamed parser occurrence");
        let other_file = other_file
            .diagnostic_occurrences
            .occurrences()
            .next()
            .expect("other-file parser occurrence");

        assert_eq!(first.semantic_origin(), renamed.semantic_origin());
        assert_eq!(first.relationship_route(), renamed.relationship_route());
        assert_ne!(first.semantic_origin(), other_file.semantic_origin());
        assert!(!first.semantic_origin().contains(".hum"));
        assert!(
            first
                .relationship_route()
                .iter()
                .all(|entry| !entry.contains(".hum"))
        );
    }

    #[test]
    fn parses_task_with_sections() {
        let source = r#"module demo

task add_task(title: Text) -> Result Task, TaskError {
  why:
    remember a task

  changes:
    tasks

  does:
    save item in tasks
}
"#;
        let parsed = parse_source("demo.hum", source);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(parsed.file.module.as_deref(), Some("demo"));
        match &parsed.file.items[0] {
            Item::Task(task) => {
                assert_eq!(task.name, "add_task");
                assert_eq!(task.sections.len(), 3);
            }
            other => panic!("expected task, got {other:?}"),
        }
    }

    #[test]
    fn peels_test_modifier_from_name() {
        let source = r#"test blank title regression {
  why:
    keep a bug fixed
  does:
    expect fixed
}
"#;
        let parsed = parse_source("demo.hum", source);
        match &parsed.file.items[0] {
            Item::Test(test) => {
                assert_eq!(test.name, "blank title");
                assert_eq!(test.modifiers, vec!["regression"]);
            }
            other => panic!("expected test, got {other:?}"),
        }
    }

    #[test]
    fn parses_parameter_permission_modes() {
        let source = r#"task permissions(item: WorkItem, borrow view: WorkItem, change draft: WorkItem, consume owned: WorkItem) {
  does:
    return owned
}
"#;
        let parsed = parse_source("permissions.hum", source);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        match &parsed.file.items[0] {
            Item::Task(task) => {
                assert_eq!(
                    task.params[0].permission,
                    crate::ast::ParamPermission::Borrow
                );
                assert_eq!(
                    task.params[1].permission,
                    crate::ast::ParamPermission::Borrow
                );
                assert_eq!(
                    task.params[2].permission,
                    crate::ast::ParamPermission::Change
                );
                assert_eq!(
                    task.params[3].permission,
                    crate::ast::ParamPermission::Consume
                );
                assert_eq!(task.params[3].name, "owned");
            }
            other => panic!("expected task, got {other:?}"),
        }
    }

    #[test]
    fn reports_stable_code_for_untyped_parameter() {
        let source = r#"task save_task(title) {
  why:
    save it
  does:
    return title
}
"#;
        let parsed = parse_source("bad.hum", source);
        assert!(
            parsed
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == DiagnosticCode::PARAMETER_MISSING_TYPE)
        );
    }

    #[test]
    fn rejects_spaced_task_name_with_snake_case_help() {
        let source = r#"task save task(title: Text) {
  does:
    return title
}
"#;
        let parsed = parse_source("spaced-name.hum", source);
        assert!(parsed.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::INVALID_IDENTIFIER
                && diagnostic.severity == Severity::Error
                && diagnostic
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains("save_task"))
        }));
    }

    #[test]
    fn reports_stable_codes_for_malformed_sources() {
        let cases = [
            (
                "missing-open-brace.hum",
                "task save_task()\n  why:\n    save it\n",
                DiagnosticCode::ITEM_HEADER_MISSING_OPEN_BRACE,
                Severity::Error,
            ),
            (
                "missing-close-brace.hum",
                "task save_task() {\n  why:\n    save it\n",
                DiagnosticCode::ITEM_BLOCK_MISSING_CLOSE_BRACE,
                Severity::Error,
            ),
            (
                "missing-close-paren.hum",
                "task save_task(title: Text {\n  why:\n    save it\n}\n",
                DiagnosticCode::CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN,
                Severity::Error,
            ),
            (
                "unexpected-top-level.hum",
                "does:\n  orphan body line\n",
                DiagnosticCode::UNEXPECTED_TOP_LEVEL_LINE,
                Severity::Warning,
            ),
        ];

        for (path, source, expected_code, expected_severity) in cases {
            let parsed = parse_source(path, source);
            assert!(
                parsed.diagnostics.iter().any(|diagnostic| {
                    diagnostic.code == expected_code
                        && diagnostic.severity == expected_severity
                        && diagnostic
                            .span
                            .as_ref()
                            .is_some_and(|span| span.file == path)
                }),
                "expected {expected_code:?} in {path}, got {:?}",
                parsed.diagnostics
            );
        }
    }

    #[test]
    fn reports_missing_brace_for_nested_item_headers() {
        let source = r#"app demo {
  why:
    show nested parsing

  task nested_task()
    why:
      missing brace
}
"#;
        let parsed = parse_source("nested-bad.hum", source);
        assert!(parsed.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::ITEM_HEADER_MISSING_OPEN_BRACE
                && diagnostic.span.as_ref().is_some_and(|span| span.line == 5)
        }));
    }

    #[test]
    fn parses_nested_app_items_from_contract() {
        let source = r#"app demo {
  why:
    group the demo

  task add_task(title: Text) {
    why:
      add the task

    does:
      return title
  }
}
"#;
        let parsed = parse_source("nested.hum", source);
        assert!(parsed.diagnostics.is_empty());
        match &parsed.file.items[0] {
            Item::App(app) => {
                assert_eq!(app.items.len(), 1);
                assert!(matches!(&app.items[0], Item::Task(task) if task.name == "add_task"));
            }
            other => panic!("expected app, got {other:?}"),
        }
    }

    #[test]
    fn preserves_comment_lines_inside_sections() {
        let source = r#"task explain() {
  why:
    # keep this visible to graph consumers
    explain the thing

  does:
    return
}
"#;
        let parsed = parse_source("comments.hum", source);
        let task = match &parsed.file.items[0] {
            Item::Task(task) => task,
            other => panic!("expected task, got {other:?}"),
        };
        let why = task.section("why").expect("why section");
        assert_eq!(why.lines[0].text, "# keep this visible to graph consumers");
        assert_eq!(why.lines[1].text, "explain the thing");
    }

    #[test]
    fn callable_type_and_indirect_call_are_parser_owned_nodes() {
        let parsed = parse_source(
            "callable_nodes.hum",
            "task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {\n  does:\n    return transform(value)\n}\n",
        );
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let TypeSyntaxKind::Callable(callable) = &task.params[0].type_syntax.kind else {
            panic!("callable type")
        };
        assert_eq!(callable.inputs.len(), 1);
        let ParsedBodyStatementKind::Return(expression) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        let ParsedExpressionKind::Call(call) = &expression.kind else {
            panic!("call")
        };
        assert_eq!(call.close_status, ParsedCallCloseStatus::Closed);
        assert_eq!(call.arguments.len(), 1);
    }

    #[test]
    fn missing_indirect_close_remains_a_structured_candidate() {
        let parsed = parse_source(
            "callable_missing_close.hum",
            "task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {\n  does:\n    return transform(value\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let ParsedBodyStatementKind::Return(expression) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        let ParsedExpressionKind::Call(call) = &expression.kind else {
            panic!("call candidate")
        };
        assert_eq!(call.close_status, ParsedCallCloseStatus::Missing);
    }

    #[test]
    fn string_braces_and_escaped_quotes_do_not_close_items() {
        let parsed = parse_source(
            "text-braces.hum",
            r#"task braces() -> Text {
  does:
    let first = "}{"
    return "escaped \" } { remains text"
}

task after() -> UInt {
  does:
    return 7
}
"#,
        );
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.file.items.len(), 2);
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        assert_eq!(task.body_syntax.len(), 2);
        assert!(task.body_syntax.iter().all(|statement| {
            statement.block_relationship == ParsedBlockRelationship::None
                && statement.block_depth_before == 0
                && statement.block_depth_after == 0
        }));
    }

    #[test]
    fn quote_escape_and_brace_direction_sabotage_changes_scope_facts() {
        let valid = parse_source(
            "valid-scope.hum",
            "task first() -> Text {\n  does:\n    return \"escaped \\\" } { text\"\n}\n\ntask second() -> UInt {\n  does:\n    return 2\n}\n",
        );
        assert!(valid.diagnostics.is_empty());
        assert_eq!(valid.file.items.len(), 2);

        let escape_removed = parse_source(
            "escape-removed.hum",
            "task first() -> Text {\n  does:\n    return \"escaped \" } { text\"\n}\n\ntask second() -> UInt {\n  does:\n    return 2\n}\n",
        );
        assert!(
            !escape_removed.diagnostics.is_empty() || escape_removed.file.items.len() != 2,
            "removing the escaped quote must not preserve the valid scope fact"
        );

        let quotes_removed = parse_source(
            "quotes-removed.hum",
            "task first() -> Text {\n  does:\n    return }{\n}\n\ntask second() -> UInt {\n  does:\n    return 2\n}\n",
        );
        assert!(
            !quotes_removed.diagnostics.is_empty() || quotes_removed.file.items.len() != 2,
            "reversing unquoted brace direction must not preserve the valid scope fact"
        );
    }

    #[test]
    fn retained_block_relationship_corruption_fails_closed() {
        let parsed = parse_source(
            "block-relationships.hum",
            "task block(value: UInt) -> UInt {\n  does:\n    if value > 0 {\n      return value\n    }\n    return 0\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        assert!(validate_retained_body_syntax(&task.body_syntax).is_ok());

        let mut wrong_direction = task.body_syntax.clone();
        wrong_direction[0].block_relationship = ParsedBlockRelationship::Closes;
        assert!(validate_retained_body_syntax(&wrong_direction).is_err());

        let mut wrong_depth = task.body_syntax.clone();
        wrong_depth[1].block_depth_before += 1;
        assert!(validate_retained_body_syntax(&wrong_depth).is_err());

        let mut wrong_id = task.body_syntax.clone();
        wrong_id[1].source_node_id = wrong_id[0].source_node_id.clone();
        assert!(validate_retained_body_syntax(&wrong_id).is_err());
    }

    #[test]
    fn genuine_unclosed_item_still_owns_h0004() {
        for literal in ["}{", "{{", "}}", "plain"] {
            let parsed = parse_source(
                "real-unclosed.hum",
                &format!("task unclosed() -> Text {{\n  does:\n    return \"{literal}\"\n"),
            );
            assert_eq!(
                parsed
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.code == DiagnosticCode::ITEM_BLOCK_MISSING_CLOSE_BRACE
                            && diagnostic.span.as_ref().is_some_and(|span| span.line == 1)
                    })
                    .count(),
                1,
                "literal {literal} cannot repair the real block"
            );
        }
    }

    #[test]
    fn canonical_expression_tree_is_left_associative_and_precedence_aware() {
        let parsed = parse_source(
            "expression-tree.hum",
            "task expressions() -> UInt {\n  does:\n    let a = 8 * 6 / 4\n    let b = 20 - 6 - 4\n    let c = 2 + 3 * 4\n    return (2 + 3) * 4\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let expressions = task
            .body_syntax
            .iter()
            .map(|statement| match &statement.kind {
                ParsedBodyStatementKind::Binding {
                    value: Some(value), ..
                }
                | ParsedBodyStatementKind::Return(value) => &value.canonical,
                other => panic!("expression statement: {other:?}"),
            })
            .collect::<Vec<_>>();
        assert!(matches!(
            &expressions[0].kind,
            CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Divide,
                left,
                ..
            } if matches!(left.kind, CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Multiply, ..
            })
        ));
        assert!(matches!(
            &expressions[1].kind,
            CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Subtract,
                left,
                ..
            } if matches!(left.kind, CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Subtract, ..
            })
        ));
        assert!(matches!(
            &expressions[2].kind,
            CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Add,
                right,
                ..
            } if matches!(right.kind, CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Multiply, ..
            })
        ));
        assert!(matches!(
            &expressions[3].kind,
            CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Multiply,
                left,
                ..
            } if matches!(left.kind, CanonicalExpressionKind::Group(_))
        ));
        assert!(
            expressions
                .iter()
                .all(|expression| validate_canonical_expression(expression).is_ok())
        );
    }

    #[test]
    fn canonical_expression_corruption_fails_closed() {
        let parsed = parse_source(
            "expression-corruption.hum",
            "task expression() -> UInt {\n  does:\n    return 2 + 3 * 4\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let ParsedBodyStatementKind::Return(value) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        let mut wrong_id = value.canonical.clone();
        wrong_id.node_id = ParserSyntaxNodeId::new("substituted-node".to_string());
        assert!(validate_canonical_expression(&wrong_id).is_err());

        let mut wrong_span = value.canonical.clone();
        if let CanonicalExpressionKind::Binary { right, .. } = &mut wrong_span.kind {
            right.range.start.column =
                wrong_span.range.start.column + wrong_span.range.byte_len + 1;
        }
        assert!(validate_canonical_expression(&wrong_span).is_err());

        let mut reordered = value.canonical.clone();
        if let CanonicalExpressionKind::Binary { left, right, .. } = &mut reordered.kind {
            std::mem::swap(left, right);
        }
        assert!(validate_canonical_expression(&reordered).is_err());
    }

    #[test]
    fn non_ascii_unsupported_expression_is_utf8_safe() {
        let parsed = parse_source(
            "non-ascii-expression.hum",
            "task non_ascii() -> Text {\n  does:\n    return café\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let ParsedBodyStatementKind::Return(value) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        assert!(matches!(
            value.canonical.kind,
            CanonicalExpressionKind::Unsupported
        ));
        assert!(validate_canonical_expression(&value.canonical).is_ok());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(
            parsed.diagnostics[0].code,
            DiagnosticCode::INVALID_IDENTIFIER
        );
        assert_eq!(
            task.body_syntax[0].core_expression_kind,
            Some("name_or_text")
        );
    }

    #[test]
    fn signed_int_is_one_structural_literal_node() {
        let parsed = parse_source(
            "signed-int-expression.hum",
            "task signed() -> Int {\n  does:\n    return -1\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let ParsedBodyStatementKind::Return(value) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        assert!(matches!(
            value.canonical.kind,
            CanonicalExpressionKind::IntLiteral(-1)
        ));
        assert!(validate_canonical_expression(&value.canonical).is_ok());
        assert_eq!(
            task.body_syntax[0].core_expression_kind,
            Some("name_or_text")
        );
    }
}
