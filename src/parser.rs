use crate::ast::{
    App, CallableTypeSyntax, CanonicalActualLexicalEvidence, CanonicalAssociativity,
    CanonicalCommonChildRole, CanonicalCommonLexicalStatus, CanonicalCommonNodeKind,
    CanonicalCompletionEvent, CanonicalDelimiterKind, CanonicalExpectedLexicalEvidence,
    CanonicalExpression, CanonicalExpressionIntentEvent, CanonicalExpressionKind,
    CanonicalExpressionRoleEvent, CanonicalLexicalTokenEvent, CanonicalLexicalTokenKind,
    CanonicalMalformedCause, CanonicalMalformedEvent, CanonicalOccurrenceAssignmentEvent,
    CanonicalPayloadEvent, CanonicalPayloadEventValue, CanonicalPayloadField,
    CanonicalReductionChildEvent, CanonicalReductionEvent, CanonicalStatementEventFact,
    CanonicalStatementEventField, CanonicalStatementEventValue, CanonicalStatementKindEvent,
    CanonicalTryWrapperKind, Field, Item, Param, ParamPermission, ParsedBinaryOperator,
    ParsedBlockRelationship, ParsedBodyStatement, ParsedBodyStatementKind, ParsedCall,
    ParsedCallCloseStatus, ParsedCallTrailingStatus, ParsedEffectDeclaration,
    ParsedEffectDeclarationKind, ParsedExpression, ParsedExpressionKind, ParsedIdentifier,
    ParsedSourceRange, ParserSyntaxNodeId, Section, SectionLine, SourceFile, Store, Task, Test,
    TypeDef, TypeSyntax, TypeSyntaxKind,
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
    CanonicalPayloadIdentity,
    CanonicalMalformedEventIdentity,
    CanonicalStatementTokenIdentity,
    CanonicalStatementBlockIdentity,
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
enum CanonicalPayloadValue {
    Position(Span),
    Token(CanonicalTokenIdentity, ParsedSourceRange, String),
    Tokens(Vec<(CanonicalTokenIdentity, ParsedSourceRange, String)>),
    Range(ParsedSourceRange),
    Ranges(Vec<ParsedSourceRange>),
    Text(String),
    UInt(u64),
    Int(i64),
    Bool(bool),
    Usize(usize),
    Bools(Vec<bool>),
    Node(CanonicalNodeIdentity),
    Nodes(Vec<CanonicalNodeIdentity>),
    OptionalNode(Option<CanonicalNodeIdentity>),
    DelimiterPair {
        kind: CanonicalDelimiterKind,
        open: CanonicalTokenIdentity,
        open_range: ParsedSourceRange,
        close: CanonicalTokenIdentity,
        close_range: ParsedSourceRange,
    },
    Operator(ParsedBinaryOperator),
    Permission(ParamPermission),
    Associativity(CanonicalAssociativity),
    WrapperKind(CanonicalTryWrapperKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalPayloadSealFact {
    identity: CanonicalPayloadIdentity,
    node: CanonicalNodeIdentity,
    field: CanonicalPayloadField,
    value: CanonicalPayloadValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CanonicalMalformedSealField {
    Status,
    Node,
    Cause,
    ProducingEvent,
    OffendingRange,
    ConsumedRange,
    ExpectedEvidence,
    ActualEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CanonicalMalformedSealValue {
    Unsupported,
    Node(CanonicalNodeIdentity),
    Cause(CanonicalMalformedCause),
    ProducingEvent(CanonicalMalformedEventIdentity, ParsedSourceRange),
    Range(ParsedSourceRange),
    Expected(CanonicalExpectedLexicalEvidence),
    Actual(CanonicalActualLexicalEvidence),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalMalformedSealFact {
    field: CanonicalMalformedSealField,
    value: CanonicalMalformedSealValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CanonicalStatementSealValue {
    Kind(CanonicalStatementKindEvent),
    Section(CanonicalSectionOwner),
    Statement(CanonicalStatementOwner),
    Range(ParsedSourceRange),
    Token(CanonicalStatementTokenIdentity, ParsedSourceRange, String),
    Tokens(Vec<(CanonicalStatementTokenIdentity, ParsedSourceRange, String)>),
    TokenReference(CanonicalStatementTokenIdentity),
    Root(CanonicalOccurrenceIdentity, usize, ParserSyntaxNodeId),
    Roots(Vec<(CanonicalOccurrenceIdentity, usize, ParserSyntaxNodeId)>),
    Block(CanonicalStatementBlockIdentity),
    Usize(usize),
    Bool(bool),
    BlockRelationship(ParsedBlockRelationship),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalStatementSealFact {
    field: CanonicalStatementEventField,
    value: CanonicalStatementSealValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalStatementSeal {
    projection: Vec<CanonicalStatementSealFact>,
    authority: Vec<CanonicalStatementSealFact>,
    schema_authority: CanonicalStatementSchemaAuthority,
    block_authority: CanonicalStatementBlockAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalStatementSchemaAuthority {
    kind: CanonicalStatementKindEvent,
    fields: Vec<CanonicalStatementEventField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalStatementBlockAuthority {
    relationship: ParsedBlockRelationship,
    depth_before: usize,
    depth_after: usize,
    owner: Option<CanonicalStatementBlockIdentity>,
    open_token: Option<(CanonicalStatementTokenIdentity, ParsedSourceRange, String)>,
    close_token: Option<(CanonicalStatementTokenIdentity, ParsedSourceRange, String)>,
}

struct StatementSealOwner<'a> {
    section: &'a CanonicalSectionOwner,
    statement: &'a CanonicalStatementOwner,
    handle: &'a CanonicalAuthorityHandle,
    block: Option<&'a CanonicalStatementBlockIdentity>,
}

fn translate_statement_events(
    events: &[CanonicalStatementEventFact],
    owner: StatementSealOwner<'_>,
) -> Vec<CanonicalStatementSealFact> {
    let token_identity = |slot| {
        CanonicalStatementTokenIdentity(source_owner_child_identity(14, &owner.statement.0, slot))
    };
    let mut facts = Vec::with_capacity(events.len() + 1);
    for event in events {
        let value = match &event.value {
            CanonicalStatementEventValue::Kind(value) => {
                let value = CanonicalStatementSealValue::Kind(*value);
                facts.push(CanonicalStatementSealFact {
                    field: CanonicalStatementEventField::Kind,
                    value,
                });
                facts.push(CanonicalStatementSealFact {
                    field: CanonicalStatementEventField::Section,
                    value: CanonicalStatementSealValue::Section(owner.section.clone()),
                });
                continue;
            }
            CanonicalStatementEventValue::Text(_) => {
                CanonicalStatementSealValue::Statement(owner.statement.clone())
            }
            CanonicalStatementEventValue::Range(value) => {
                CanonicalStatementSealValue::Range(value.clone())
            }
            CanonicalStatementEventValue::Token {
                slot,
                range,
                spelling,
            } => CanonicalStatementSealValue::Token(
                token_identity(*slot),
                range.clone(),
                spelling.clone(),
            ),
            CanonicalStatementEventValue::Tokens(values) => CanonicalStatementSealValue::Tokens(
                values
                    .iter()
                    .map(|(slot, range, spelling)| {
                        (token_identity(*slot), range.clone(), spelling.clone())
                    })
                    .collect(),
            ),
            CanonicalStatementEventValue::TokenReference(slot) => {
                CanonicalStatementSealValue::TokenReference(token_identity(*slot))
            }
            CanonicalStatementEventValue::Root { ordinal, node } => {
                CanonicalStatementSealValue::Root(
                    occurrence_identity(owner.handle, *ordinal),
                    *ordinal,
                    node.clone(),
                )
            }
            CanonicalStatementEventValue::Roots(values) => CanonicalStatementSealValue::Roots(
                values
                    .iter()
                    .map(|(ordinal, node)| {
                        (
                            occurrence_identity(owner.handle, *ordinal),
                            *ordinal,
                            node.clone(),
                        )
                    })
                    .collect(),
            ),
            CanonicalStatementEventValue::Usize(value) => {
                CanonicalStatementSealValue::Usize(*value)
            }
            CanonicalStatementEventValue::Bool(value)
                if event.field == CanonicalStatementEventField::BlockOwner && *value =>
            {
                CanonicalStatementSealValue::Block(
                    owner
                        .block
                        .expect("statement block relationship requires an owning block")
                        .clone(),
                )
            }
            CanonicalStatementEventValue::Bool(value) => CanonicalStatementSealValue::Bool(*value),
            CanonicalStatementEventValue::BlockRelationship(value) => {
                CanonicalStatementSealValue::BlockRelationship(*value)
            }
        };
        facts.push(CanonicalStatementSealFact {
            field: event.field,
            value,
        });
    }
    facts
}

fn build_statement_seal(
    projection: &[CanonicalStatementEventFact],
    authority: &[CanonicalStatementEventFact],
    owner: &CanonicalSourceOwnerSeal,
    projected_block: Option<&CanonicalStatementBlockIdentity>,
    authority_block: Option<&CanonicalStatementBlockIdentity>,
) -> CanonicalStatementSeal {
    let (projected_section, projected_statement, projected_handle) = match (
        owner.projection.get(4),
        owner.projection.get(5),
        owner.projection.get(6),
    ) {
        (
            Some(CanonicalSourceOwnerFact::Section(section)),
            Some(CanonicalSourceOwnerFact::Statement(statement)),
            Some(CanonicalSourceOwnerFact::AuthorityHandle(handle)),
        ) => (section, statement, handle),
        _ => panic!("statement projection requires exact source-owner identities"),
    };
    let projection = translate_statement_events(
        projection,
        StatementSealOwner {
            section: projected_section,
            statement: projected_statement,
            handle: projected_handle,
            block: projected_block,
        },
    );
    let authority = translate_statement_events(
        authority,
        StatementSealOwner {
            section: &owner.authority.section,
            statement: &owner.authority.statement,
            handle: &owner.authority.handle,
            block: authority_block,
        },
    );
    let authority_value = |field| {
        authority
            .iter()
            .find(|fact| fact.field == field)
            .map(|fact| &fact.value)
    };
    let kind = match authority_value(CanonicalStatementEventField::Kind) {
        Some(CanonicalStatementSealValue::Kind(kind)) => *kind,
        _ => panic!("retained statement authority requires one kind"),
    };
    let block_authority = CanonicalStatementBlockAuthority {
        relationship: match authority_value(CanonicalStatementEventField::BlockRelationship) {
            Some(CanonicalStatementSealValue::BlockRelationship(value)) => *value,
            _ => panic!("retained statement authority requires a block relationship"),
        },
        depth_before: match authority_value(CanonicalStatementEventField::BlockDepthBefore) {
            Some(CanonicalStatementSealValue::Usize(value)) => *value,
            _ => panic!("retained statement authority requires block depth before"),
        },
        depth_after: match authority_value(CanonicalStatementEventField::BlockDepthAfter) {
            Some(CanonicalStatementSealValue::Usize(value)) => *value,
            _ => panic!("retained statement authority requires block depth after"),
        },
        owner: match authority_value(CanonicalStatementEventField::BlockOwner) {
            Some(CanonicalStatementSealValue::Block(value)) => Some(value.clone()),
            None => None,
            _ => panic!("retained statement authority has malformed block owner"),
        },
        open_token: match authority_value(CanonicalStatementEventField::BlockOpenToken) {
            Some(CanonicalStatementSealValue::Token(identity, range, spelling)) => {
                Some((identity.clone(), range.clone(), spelling.clone()))
            }
            None => None,
            _ => panic!("retained statement authority has malformed block opener"),
        },
        close_token: match authority_value(CanonicalStatementEventField::BlockCloseToken) {
            Some(CanonicalStatementSealValue::Token(identity, range, spelling)) => {
                Some((identity.clone(), range.clone(), spelling.clone()))
            }
            None => None,
            _ => panic!("retained statement authority has malformed block closer"),
        },
    };
    CanonicalStatementSeal {
        projection,
        schema_authority: CanonicalStatementSchemaAuthority {
            kind,
            fields: authority.iter().map(|fact| fact.field).collect(),
        },
        block_authority,
        authority,
    }
}

fn validate_statement_seal(seal: &CanonicalStatementSeal) -> Result<(), &'static str> {
    if seal.projection.len() != seal.authority.len() {
        return Err("canonical_statement_field_count_corrupt_v0");
    }
    if seal
        .projection
        .iter()
        .zip(&seal.authority)
        .any(|(projection, authority)| projection != authority)
    {
        return Err("canonical_statement_authority_mismatch_v0");
    }
    let fields = seal
        .projection
        .iter()
        .map(|fact| fact.field)
        .collect::<Vec<_>>();
    let authority_fields = seal
        .authority
        .iter()
        .map(|fact| fact.field)
        .collect::<Vec<_>>();
    if fields != seal.schema_authority.fields || authority_fields != seal.schema_authority.fields {
        return Err("canonical_statement_closed_schema_corrupt_v0");
    }
    if !fields.starts_with(&[
        CanonicalStatementEventField::Kind,
        CanonicalStatementEventField::Section,
        CanonicalStatementEventField::Line,
        CanonicalStatementEventField::Statement,
    ]) || !fields.contains(&CanonicalStatementEventField::OrderedRoots)
        || !fields.contains(&CanonicalStatementEventField::BlockDepthBefore)
        || !fields.contains(&CanonicalStatementEventField::BlockDepthAfter)
    {
        return Err("canonical_statement_shape_corrupt_v0");
    }
    for fact in &seal.projection {
        match &fact.value {
            CanonicalStatementSealValue::Token(_, range, spelling) => {
                if range.byte_len != spelling.len()
                    || !source_revision_contains_spelling(seal, range, spelling)
                {
                    return Err("canonical_statement_token_length_corrupt_v0");
                }
            }
            CanonicalStatementSealValue::Tokens(values)
                if values.iter().any(|(_, range, spelling)| {
                    range.byte_len != spelling.len()
                        || !source_revision_contains_spelling(seal, range, spelling)
                }) =>
            {
                return Err("canonical_statement_token_length_corrupt_v0");
            }
            _ => {}
        }
    }
    let value = |field| {
        seal.projection
            .iter()
            .find(|fact| fact.field == field)
            .map(|fact| &fact.value)
    };
    let kind = match value(CanonicalStatementEventField::Kind) {
        Some(CanonicalStatementSealValue::Kind(kind)) => *kind,
        _ => return Err("canonical_statement_kind_corrupt_v0"),
    };
    if kind != seal.schema_authority.kind {
        return Err("canonical_statement_kind_authority_corrupt_v0");
    }
    let section = match value(CanonicalStatementEventField::Section) {
        Some(CanonicalStatementSealValue::Section(section)) => section,
        _ => return Err("canonical_statement_section_identity_corrupt_v0"),
    };
    let statement = match value(CanonicalStatementEventField::Statement) {
        Some(CanonicalStatementSealValue::Statement(statement)) => statement,
        _ => return Err("canonical_statement_owner_identity_corrupt_v0"),
    };
    if section.0.domain != 4
        || statement.0.domain != 5
        || section.0.revision != statement.0.revision
        || !statement
            .0
            .traversal
            .starts_with(section.0.traversal.as_ref())
        || statement.0.traversal.len() != section.0.traversal.len() + 1
    {
        return Err("canonical_statement_owner_identity_corrupt_v0");
    }
    let token_is_owned = |identity: &CanonicalStatementTokenIdentity| {
        identity.0.domain == 14
            && identity.0.revision == statement.0.revision
            && identity
                .0
                .traversal
                .starts_with(statement.0.traversal.as_ref())
            && identity.0.traversal.len() == statement.0.traversal.len() + 1
    };
    let root_is_owned = |identity: &CanonicalOccurrenceIdentity| {
        identity.0.domain == 7
            && identity.0.revision == statement.0.revision
            && identity
                .0
                .traversal
                .starts_with(statement.0.traversal.as_ref())
            && identity.0.traversal.len() == statement.0.traversal.len() + 2
    };
    for fact in &seal.projection {
        let owned = match &fact.value {
            CanonicalStatementSealValue::Token(identity, _, _)
            | CanonicalStatementSealValue::TokenReference(identity) => token_is_owned(identity),
            CanonicalStatementSealValue::Tokens(values) => values
                .iter()
                .all(|(identity, _, _)| token_is_owned(identity)),
            CanonicalStatementSealValue::Root(identity, _, _) => root_is_owned(identity),
            CanonicalStatementSealValue::Roots(values) => values
                .iter()
                .all(|(identity, _, _)| root_is_owned(identity)),
            CanonicalStatementSealValue::Block(identity) => {
                identity.0.domain == 16 && identity.0.revision == statement.0.revision
            }
            _ => true,
        };
        if !owned {
            return Err("canonical_statement_child_identity_corrupt_v0");
        }
    }
    let keyword = match value(CanonicalStatementEventField::Keyword) {
        Some(CanonicalStatementSealValue::Token(_, _, spelling)) => Some(spelling.as_str()),
        None => None,
        _ => return Err("canonical_statement_keyword_corrupt_v0"),
    };
    let expected_keyword = match kind {
        CanonicalStatementKindEvent::NeedsPredicate => Some("needs"),
        CanonicalStatementKindEvent::EnsuresPredicate => Some("ensures"),
        CanonicalStatementKindEvent::Return => Some("return"),
        CanonicalStatementKindEvent::ImmutableBinding => Some("let"),
        CanonicalStatementKindEvent::MutableBinding => Some("change"),
        CanonicalStatementKindEvent::Set => Some("set"),
        CanonicalStatementKindEvent::Save => Some("save"),
        CanonicalStatementKindEvent::Fail => Some("fail"),
        CanonicalStatementKindEvent::Expect => Some("expect"),
        CanonicalStatementKindEvent::If => Some("if"),
        CanonicalStatementKindEvent::While => Some("while"),
        CanonicalStatementKindEvent::ForEach
        | CanonicalStatementKindEvent::ForIndexUntil
        | CanonicalStatementKindEvent::ForIndexThrough => Some("for"),
        CanonicalStatementKindEvent::UnconditionalLoop => Some("loop"),
        CanonicalStatementKindEvent::BlockClose => Some("}"),
        CanonicalStatementKindEvent::FreeExpression => None,
    };
    if keyword != expected_keyword {
        return Err("canonical_statement_keyword_corrupt_v0");
    }
    if let Some(CanonicalStatementSealValue::Token(binder, _, _)) =
        value(CanonicalStatementEventField::Binder)
        && !matches!(
            value(CanonicalStatementEventField::BinderRelationship),
            Some(CanonicalStatementSealValue::TokenReference(reference)) if reference == binder
        )
    {
        return Err("canonical_statement_binder_identity_corrupt_v0");
    }
    let roots = match value(CanonicalStatementEventField::OrderedRoots) {
        Some(CanonicalStatementSealValue::Roots(roots)) => roots,
        _ => return Err("canonical_statement_roots_corrupt_v0"),
    };
    if roots
        .iter()
        .enumerate()
        .any(|(index, (_, ordinal, _))| index != *ordinal)
    {
        return Err("canonical_statement_root_order_corrupt_v0");
    }
    for field in [
        CanonicalStatementEventField::TargetRoot,
        CanonicalStatementEventField::ValueRoot,
        CanonicalStatementEventField::StartRoot,
        CanonicalStatementEventField::EndRoot,
    ] {
        let Some(CanonicalStatementSealValue::Root(identity, ordinal, node)) = value(field) else {
            continue;
        };
        if roots.get(*ordinal) != Some(&(identity.clone(), *ordinal, node.clone())) {
            return Err("canonical_statement_root_relationship_corrupt_v0");
        }
    }
    let relationship_ordinal = |field| match value(field) {
        Some(CanonicalStatementSealValue::Root(_, ordinal, _)) => Some(*ordinal),
        None => None,
        _ => None,
    };
    let roles_exact = match kind {
        CanonicalStatementKindEvent::Set => {
            relationship_ordinal(CanonicalStatementEventField::ValueRoot) == Some(0)
                && relationship_ordinal(CanonicalStatementEventField::TargetRoot) == Some(1)
        }
        CanonicalStatementKindEvent::ForIndexUntil
        | CanonicalStatementKindEvent::ForIndexThrough => {
            relationship_ordinal(CanonicalStatementEventField::StartRoot) == Some(0)
                && relationship_ordinal(CanonicalStatementEventField::EndRoot) == Some(1)
        }
        _ => true,
    };
    if !roles_exact {
        return Err("canonical_statement_root_role_corrupt_v0");
    }
    let expected_roots = match kind {
        CanonicalStatementKindEvent::ImmutableBinding
        | CanonicalStatementKindEvent::MutableBinding
        | CanonicalStatementKindEvent::FreeExpression => {
            usize::from(value(CanonicalStatementEventField::ValueRoot).is_some())
        }
        CanonicalStatementKindEvent::Set
        | CanonicalStatementKindEvent::ForIndexUntil
        | CanonicalStatementKindEvent::ForIndexThrough => 2,
        CanonicalStatementKindEvent::UnconditionalLoop
        | CanonicalStatementKindEvent::BlockClose => 0,
        _ => 1,
    };
    if roots.len() != expected_roots
        || (expected_roots == 0)
            != matches!(
                value(CanonicalStatementEventField::ExpressionAbsent),
                Some(CanonicalStatementSealValue::Bool(true))
            )
    {
        return Err("canonical_statement_root_cardinality_corrupt_v0");
    }
    validate_statement_kind_schema(seal, kind, &fields, roots.len())?;
    let relationship_spellings = seal
        .projection
        .iter()
        .filter_map(|fact| {
            (fact.field == CanonicalStatementEventField::RelationshipToken).then_some(&fact.value)
        })
        .map(|value| match value {
            CanonicalStatementSealValue::Token(_, _, spelling) => Ok(spelling.as_str()),
            _ => Err("canonical_statement_relationship_token_corrupt_v0"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expected_relationships: &[&str] = match kind {
        CanonicalStatementKindEvent::NeedsPredicate
        | CanonicalStatementKindEvent::EnsuresPredicate => &[":"],
        CanonicalStatementKindEvent::Save | CanonicalStatementKindEvent::ForEach => &["in"],
        CanonicalStatementKindEvent::ForIndexUntil => &["from", "until"],
        CanonicalStatementKindEvent::ForIndexThrough => &["from", "through"],
        _ => &[],
    };
    if relationship_spellings != expected_relationships {
        return Err("canonical_statement_relationship_corrupt_v0");
    }
    let before = match value(CanonicalStatementEventField::BlockDepthBefore) {
        Some(CanonicalStatementSealValue::Usize(value)) => *value,
        _ => return Err("canonical_statement_block_depth_corrupt_v0"),
    };
    let after = match value(CanonicalStatementEventField::BlockDepthAfter) {
        Some(CanonicalStatementSealValue::Usize(value)) => *value,
        _ => return Err("canonical_statement_block_depth_corrupt_v0"),
    };
    let relationship = match value(CanonicalStatementEventField::BlockRelationship) {
        Some(CanonicalStatementSealValue::BlockRelationship(value)) => *value,
        _ => return Err("canonical_statement_block_relationship_corrupt_v0"),
    };
    let owner_present = matches!(
        value(CanonicalStatementEventField::BlockOwner),
        Some(CanonicalStatementSealValue::Block(_))
    );
    let open_present = value(CanonicalStatementEventField::BlockOpenToken).is_some();
    let close_present = value(CanonicalStatementEventField::BlockCloseToken).is_some();
    let block_exact = match relationship {
        ParsedBlockRelationship::None => {
            before == after && owner_present == (before > 0) && !open_present && !close_present
        }
        ParsedBlockRelationship::Opens => {
            after == before + 1 && owner_present && open_present && !close_present
        }
        ParsedBlockRelationship::Closes => {
            before == after + 1 && owner_present && !open_present && close_present
        }
    };
    if !block_exact {
        return Err("canonical_statement_block_topology_corrupt_v0");
    }
    let candidate_owner = match value(CanonicalStatementEventField::BlockOwner) {
        Some(CanonicalStatementSealValue::Block(identity)) => Some(identity),
        None => None,
        _ => return Err("canonical_statement_block_owner_corrupt_v0"),
    };
    let candidate_token = |field| match value(field) {
        Some(CanonicalStatementSealValue::Token(identity, range, spelling)) => {
            Some((identity.clone(), range.clone(), spelling.clone()))
        }
        None => None,
        _ => None,
    };
    if relationship != seal.block_authority.relationship
        || before != seal.block_authority.depth_before
        || after != seal.block_authority.depth_after
        || candidate_owner != seal.block_authority.owner.as_ref()
        || candidate_token(CanonicalStatementEventField::BlockOpenToken).as_ref()
            != seal.block_authority.open_token.as_ref()
        || candidate_token(CanonicalStatementEventField::BlockCloseToken).as_ref()
            != seal.block_authority.close_token.as_ref()
    {
        return Err("canonical_statement_block_authority_corrupt_v0");
    }
    Ok(())
}

fn source_revision_contains_spelling(
    seal: &CanonicalStatementSeal,
    range: &ParsedSourceRange,
    spelling: &str,
) -> bool {
    let revision = seal.projection.iter().find_map(|fact| match &fact.value {
        CanonicalStatementSealValue::Statement(statement) => Some(&statement.0.revision),
        _ => None,
    });
    let Some(revision) = revision else {
        return false;
    };
    let Some(line) = revision
        .as_ref()
        .split(|byte| *byte == b'\n')
        .nth(range.start.line - 1)
    else {
        return false;
    };
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    let start = range.start.column.saturating_sub(1);
    line.get(start..start + range.byte_len) == Some(spelling.as_bytes())
}

fn validate_statement_kind_schema(
    seal: &CanonicalStatementSeal,
    kind: CanonicalStatementKindEvent,
    fields: &[CanonicalStatementEventField],
    root_count: usize,
) -> Result<(), &'static str> {
    use CanonicalStatementEventField as F;
    use CanonicalStatementKindEvent as K;
    let common = [
        F::Kind,
        F::Section,
        F::Line,
        F::Statement,
        F::OrderedRoots,
        F::BlockDepthBefore,
        F::BlockDepthAfter,
        F::BlockRelationship,
        F::BlockOwner,
        F::BlockOpenToken,
        F::BlockCloseToken,
        F::ExpressionAbsent,
    ];
    let extras: &[F] = match kind {
        K::NeedsPredicate | K::EnsuresPredicate => {
            &[F::Keyword, F::RelationshipToken, F::ValueRoot]
        }
        K::Return | K::Fail | K::Expect | K::If | K::While => &[F::Keyword, F::ValueRoot],
        K::ImmutableBinding | K::MutableBinding => &[
            F::Keyword,
            F::Binder,
            F::BinderRelationship,
            F::TypeBoundary,
            F::AssignmentToken,
            F::ValueRoot,
        ],
        K::Set => &[F::Keyword, F::AssignmentToken, F::TargetRoot, F::ValueRoot],
        K::Save => &[
            F::Keyword,
            F::RelationshipToken,
            F::ValueRoot,
            F::DestinationToken,
        ],
        K::FreeExpression => &[F::ValueRoot],
        K::ForEach => &[
            F::Keyword,
            F::PhraseTokens,
            F::Binder,
            F::BinderRelationship,
            F::RelationshipToken,
            F::ValueRoot,
        ],
        K::ForIndexUntil | K::ForIndexThrough => &[
            F::Keyword,
            F::PhraseTokens,
            F::Binder,
            F::BinderRelationship,
            F::RelationshipToken,
            F::StartRoot,
            F::EndRoot,
        ],
        K::UnconditionalLoop | K::BlockClose => &[F::Keyword],
    };
    if fields
        .iter()
        .any(|field| !common.contains(field) && !extras.contains(field))
    {
        return Err("canonical_statement_forbidden_field_corrupt_v0");
    }
    let count = |field| {
        fields
            .iter()
            .filter(|candidate| **candidate == field)
            .count()
    };
    let require_once = |field| (count(field) == 1).then_some(()).ok_or(());
    for field in [F::Kind, F::Section, F::Line, F::Statement, F::OrderedRoots] {
        require_once(field).map_err(|()| "canonical_statement_required_field_corrupt_v0")?;
    }
    let required: &[F] = match kind {
        K::NeedsPredicate | K::EnsuresPredicate => {
            &[F::Keyword, F::RelationshipToken, F::ValueRoot]
        }
        K::Return | K::Fail | K::Expect | K::If | K::While => &[F::Keyword, F::ValueRoot],
        K::ImmutableBinding | K::MutableBinding => &[F::Keyword, F::Binder, F::BinderRelationship],
        K::Set => &[F::Keyword, F::AssignmentToken, F::TargetRoot, F::ValueRoot],
        K::Save => &[
            F::Keyword,
            F::RelationshipToken,
            F::ValueRoot,
            F::DestinationToken,
        ],
        K::ForEach => &[
            F::Keyword,
            F::PhraseTokens,
            F::Binder,
            F::BinderRelationship,
            F::RelationshipToken,
            F::ValueRoot,
        ],
        K::ForIndexUntil | K::ForIndexThrough => &[
            F::Keyword,
            F::PhraseTokens,
            F::Binder,
            F::BinderRelationship,
            F::StartRoot,
            F::EndRoot,
        ],
        K::UnconditionalLoop | K::BlockClose => &[F::Keyword, F::ExpressionAbsent],
        K::FreeExpression => &[],
    };
    for field in required {
        require_once(*field).map_err(|()| "canonical_statement_required_field_corrupt_v0")?;
    }
    let expected_relationship_count = match kind {
        K::ForIndexUntil | K::ForIndexThrough => 2,
        K::NeedsPredicate | K::EnsuresPredicate | K::Save | K::ForEach => 1,
        _ => 0,
    };
    if count(F::RelationshipToken) != expected_relationship_count {
        return Err("canonical_statement_relationship_count_corrupt_v0");
    }
    if kind == K::FreeExpression {
        if root_count == 0 {
            let line =
                statement_source_line(seal).ok_or("canonical_statement_source_line_corrupt_v0")?;
            let label = line.strip_suffix(':').is_some_and(|label| {
                !label.is_empty() && label.chars().all(|ch| ch.is_ascii_lowercase() || ch == ' ')
            });
            if !label || count(F::ExpressionAbsent) != 1 || count(F::ValueRoot) != 0 {
                return Err("canonical_statement_rootless_label_corrupt_v0");
            }
        } else if root_count != 1 || count(F::ValueRoot) != 1 || count(F::ExpressionAbsent) != 0 {
            return Err("canonical_statement_free_expression_root_corrupt_v0");
        }
    }
    Ok(())
}

fn statement_source_line(seal: &CanonicalStatementSeal) -> Option<&str> {
    let statement = seal.projection.iter().find_map(|fact| match &fact.value {
        CanonicalStatementSealValue::Statement(statement) => Some(statement),
        _ => None,
    })?;
    let range = seal
        .projection
        .iter()
        .find_map(|fact| match (&fact.field, &fact.value) {
            (CanonicalStatementEventField::Line, CanonicalStatementSealValue::Range(range)) => {
                Some(range)
            }
            _ => None,
        })?;
    std::str::from_utf8(source_revision_slice(&statement.0.revision, range)?).ok()
}

fn source_revision_slice<'a>(revision: &'a [u8], range: &ParsedSourceRange) -> Option<&'a [u8]> {
    let line = revision
        .split(|byte| *byte == b'\n')
        .nth(range.start.line.checked_sub(1)?)?;
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    let start = range.start.column.checked_sub(1)?;
    line.get(start..start.checked_add(range.byte_len)?)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalOccurrenceSeal {
    projection: Vec<CanonicalSealFact>,
    authority: Vec<CanonicalSealFact>,
    payload_projection: Vec<CanonicalPayloadSealFact>,
    payload_authority: Vec<CanonicalPayloadSealFact>,
    malformed_projection: Vec<CanonicalMalformedSealFact>,
    malformed_authority: Vec<CanonicalMalformedSealFact>,
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
    validate_occurrence_seal_inner(seal, None, None)
}

fn validate_occurrence_seal_inner(
    seal: &CanonicalOccurrenceSeal,
    ignored_index: Option<usize>,
    ignored_payload_index: Option<usize>,
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
    if seal.payload_projection.len() != seal.payload_authority.len() {
        return Err("canonical_payload_field_count_corrupt_v0");
    }
    if seal
        .payload_projection
        .iter()
        .zip(&seal.payload_authority)
        .enumerate()
        .any(|(index, (left, right))| Some(index) != ignored_payload_index && left != right)
    {
        return Err("canonical_payload_authority_mismatch_v0");
    }
    if seal.malformed_projection.len() != seal.malformed_authority.len() {
        return Err("canonical_malformed_field_count_corrupt_v0");
    }
    if seal
        .malformed_projection
        .iter()
        .zip(&seal.malformed_authority)
        .any(|(left, right)| left != right)
    {
        return Err("canonical_malformed_authority_mismatch_v0");
    }
    if seal.malformed_projection.is_empty() {
        if seal.projection.iter().any(|fact| {
            matches!(
                fact,
                CanonicalSealFact::Kind(_, CanonicalCommonNodeKind::Unsupported)
                    | CanonicalSealFact::LexicalStatus(
                        _,
                        CanonicalCommonLexicalStatus::Unsupported
                    )
            )
        }) {
            return Err("canonical_complete_contains_unsupported_v0");
        }
    } else {
        if !seal.malformed_projection.len().is_multiple_of(8) {
            return Err("canonical_malformed_shape_corrupt_v0");
        }
        for facts in seal.malformed_projection.chunks_exact(8) {
            if !facts.iter().map(|fact| fact.field).eq([
                CanonicalMalformedSealField::Status,
                CanonicalMalformedSealField::Node,
                CanonicalMalformedSealField::Cause,
                CanonicalMalformedSealField::ProducingEvent,
                CanonicalMalformedSealField::OffendingRange,
                CanonicalMalformedSealField::ConsumedRange,
                CanonicalMalformedSealField::ExpectedEvidence,
                CanonicalMalformedSealField::ActualEvidence,
            ]) || !matches!(facts[0].value, CanonicalMalformedSealValue::Unsupported)
            {
                return Err("canonical_malformed_shape_corrupt_v0");
            }
            validate_malformed_semantics(seal, facts)?;
        }
        let malformed_nodes = seal
            .malformed_projection
            .chunks_exact(8)
            .filter_map(|facts| match &facts[1].value {
                CanonicalMalformedSealValue::Node(node) => Some(node),
                _ => None,
            })
            .collect::<Vec<_>>();
        let unsupported_nodes = seal
            .projection
            .iter()
            .filter_map(|fact| match fact {
                CanonicalSealFact::LexicalStatus(
                    node,
                    CanonicalCommonLexicalStatus::Unsupported,
                ) => Some(node),
                _ => None,
            })
            .collect::<Vec<_>>();
        if malformed_nodes != unsupported_nodes {
            return Err("canonical_unsupported_node_evidence_incomplete_v0");
        }
    }
    for fact in &seal.projection {
        let CanonicalSealFact::Kind(node, kind) = fact else {
            continue;
        };
        let lexically_unsupported = seal.projection.iter().any(|fact| {
            matches!(
                fact,
                CanonicalSealFact::LexicalStatus(
                    status_node,
                    CanonicalCommonLexicalStatus::Unsupported
                ) if status_node == node
            )
        });
        let actual = seal
            .payload_projection
            .iter()
            .filter(|payload| &payload.node == node)
            .map(|payload| payload.field)
            .collect::<Vec<_>>();
        if lexically_unsupported {
            if !actual.is_empty() {
                return Err("unsupported_canonical_payload_present_v0");
            }
            continue;
        }
        if actual != expected_payload_fields(*kind) {
            return Err("canonical_payload_field_shape_corrupt_v0");
        }
    }
    Ok(())
}

fn malformed_actual_range(actual: &CanonicalActualLexicalEvidence) -> Option<&ParsedSourceRange> {
    match actual {
        CanonicalActualLexicalEvidence::Token { range, .. } => Some(range),
        CanonicalActualLexicalEvidence::EndOfInput
        | CanonicalActualLexicalEvidence::DelimiterDepth(_) => None,
    }
}

fn retained_source_completion(
    seal: &CanonicalOccurrenceSeal,
    node: &CanonicalNodeIdentity,
) -> Option<CanonicalMalformedEvent> {
    let revision = seal.projection.iter().find_map(|fact| match fact {
        CanonicalSealFact::SourceRevision(revision) => Some(revision),
        _ => None,
    })?;
    let kind = seal.projection.iter().find_map(|fact| match fact {
        CanonicalSealFact::Kind(candidate, kind) if candidate == node => Some(*kind),
        _ => None,
    })?;
    let range = seal.projection.iter().find_map(|fact| match fact {
        CanonicalSealFact::NodeRange(candidate, range) if candidate == node => Some(range),
        _ => None,
    })?;
    let source = std::str::from_utf8(source_revision_slice(&revision.0, range)?).ok()?;
    let root = seal.projection.iter().any(|fact| {
        matches!(fact, CanonicalSealFact::PreorderOrdinal(candidate, 0) if candidate == node)
    });
    let completion = root
        .then(|| retained_predicate_list_completion(source, &range.start))
        .flatten()
        .unwrap_or_else(|| retained_completion_event(kind, source, &range.start));
    match completion {
        CanonicalCompletionEvent::Unsupported(event) => Some(*event),
        CanonicalCompletionEvent::Complete => seal
            .projection
            .iter()
            .find_map(|fact| match fact {
                CanonicalSealFact::OrderedChildren(candidate, children) if candidate == node => {
                    Some(children)
                }
                _ => None,
            })?
            .iter()
            .find_map(|child| retained_source_completion(seal, child)),
    }
}

fn validate_malformed_semantics(
    seal: &CanonicalOccurrenceSeal,
    facts: &[CanonicalMalformedSealFact],
) -> Result<(), &'static str> {
    let node = match &facts[1].value {
        CanonicalMalformedSealValue::Node(value) => value,
        _ => return Err("canonical_malformed_node_shape_corrupt_v0"),
    };
    let cause = match &facts[2].value {
        CanonicalMalformedSealValue::Cause(value) => *value,
        _ => return Err("canonical_malformed_cause_shape_corrupt_v0"),
    };
    let (producing_identity, producing) = match &facts[3].value {
        CanonicalMalformedSealValue::ProducingEvent(identity, range) => (identity, range),
        _ => return Err("canonical_malformed_producer_shape_corrupt_v0"),
    };
    let offending = match &facts[4].value {
        CanonicalMalformedSealValue::Range(value) => value,
        _ => return Err("canonical_malformed_offending_shape_corrupt_v0"),
    };
    let consumed = match &facts[5].value {
        CanonicalMalformedSealValue::Range(value) => value,
        _ => return Err("canonical_malformed_consumed_shape_corrupt_v0"),
    };
    let expected = match &facts[6].value {
        CanonicalMalformedSealValue::Expected(value) => value,
        _ => return Err("canonical_malformed_expected_shape_corrupt_v0"),
    };
    let actual = match &facts[7].value {
        CanonicalMalformedSealValue::Actual(value) => value,
        _ => return Err("canonical_malformed_actual_shape_corrupt_v0"),
    };
    if !range_contains(consumed, producing)
        || !range_contains(consumed, offending)
        || malformed_actual_range(actual).is_some_and(|range| !range_contains(consumed, range))
    {
        return Err("canonical_malformed_range_corrupt_v0");
    }
    let occurrence = seal.projection.iter().find_map(|fact| match fact {
        CanonicalSealFact::Occurrence(identity) => Some(identity),
        _ => None,
    });
    let Some(occurrence) = occurrence else {
        return Err("canonical_malformed_occurrence_missing_v0");
    };
    let preorder = seal.projection.iter().find_map(|fact| match fact {
        CanonicalSealFact::PreorderOrdinal(candidate, preorder) if candidate == node => {
            Some(*preorder)
        }
        _ => None,
    });
    let Some(preorder) = preorder else {
        return Err("canonical_malformed_node_missing_v0");
    };
    if producing_identity
        != &CanonicalMalformedEventIdentity(source_owner_child_identity(
            13,
            &occurrence.0,
            preorder,
        ))
    {
        return Err("canonical_malformed_producer_identity_corrupt_v0");
    }
    let source_event = retained_source_completion(seal, node)
        .ok_or("canonical_malformed_source_event_missing_v0")?;
    if source_event.cause != cause
        || source_event.producing_event != *producing
        || source_event.offending != *offending
        || source_event.consumed != *consumed
        || source_event.expected != *expected
        || source_event.actual != *actual
    {
        return Err("canonical_malformed_source_binding_corrupt_v0");
    }
    use CanonicalActualLexicalEvidence as A;
    use CanonicalExpectedLexicalEvidence as E;
    use CanonicalLexicalTokenKind as T;
    use CanonicalMalformedCause as C;
    let exact = match cause {
        C::UnterminatedTextLiteral => {
            matches!(expected, E::Token(T::TextQuote)) && matches!(actual, A::EndOfInput)
        }
        C::MissingDelimiter => {
            matches!(
                expected,
                E::Token(T::ParenthesisClose | T::ListClose | T::RecordClose)
            ) && matches!(actual, A::EndOfInput)
        }
        C::MismatchedDelimiter => matches!(
            (expected, actual),
            (
                E::Token(T::ParenthesisClose | T::ListClose | T::RecordClose) | E::Operand,
                A::Token {
                    kind: T::ParenthesisClose | T::ListClose | T::RecordClose,
                    ..
                }
            )
        ),
        C::DelimiterDepthExceeded => matches!(
            (expected, actual),
            (
                E::MaximumDelimiterDepth(CANONICAL_MAX_DELIMITER_DEPTH),
                A::DelimiterDepth(depth)
            ) if *depth == CANONICAL_MAX_DELIMITER_DEPTH + 1
        ),
        C::MissingOperand => matches!(expected, E::Operand) && matches!(actual, A::EndOfInput),
        C::InvalidComparisonOperator => matches!(
            (expected, actual),
            (
                E::ComparisonOperator,
                A::Token {
                    kind: T::ComparisonOperator,
                    spelling,
                    ..
                }
            ) if !matches!(spelling.as_str(), "<" | "<=" | ">" | ">=" | "==" | "!=")
        ),
        C::InvalidOperandStarter => matches!((expected, actual), (E::Operand, A::Token { .. })),
        C::MalformedFieldPlace => matches!(
            (expected, actual),
            (
                E::Identifier,
                A::Token {
                    kind: T::Dot | T::Other,
                    ..
                }
            )
        ),
        C::ListElementSeparator => matches!(
            (expected, actual),
            (E::ListSeparatorOrClose, A::Token { .. })
        ),
        C::ListTrailingComma => matches!(
            (expected, actual),
            (
                E::TextListElement,
                A::Token {
                    kind: T::ListClose,
                    ..
                }
            )
        ),
        C::ListNonTextElement => matches!(
            (expected, actual),
            (E::TextListElement, A::Token { kind, .. }) if *kind != T::TextQuote
        ),
        C::IntegerLiteralOutOfRange => matches!(
            (expected, actual),
            (
                E::Int64Value,
                A::Token {
                    kind: T::IntegerLiteral,
                    spelling,
                    ..
                }
            ) if spelling.parse::<i64>().is_err()
        ),
    };
    let same_range = |left: &ParsedSourceRange, right: &ParsedSourceRange| left == right;
    let range_shape = match cause {
        C::UnterminatedTextLiteral => producing.byte_len == 1 && offending.byte_len > 0,
        C::MissingDelimiter | C::MissingOperand => offending.byte_len == 0,
        C::MismatchedDelimiter
        | C::InvalidComparisonOperator
        | C::InvalidOperandStarter
        | C::MalformedFieldPlace
        | C::ListElementSeparator
        | C::ListNonTextElement
        | C::IntegerLiteralOutOfRange => malformed_actual_range(actual)
            .is_some_and(|actual_range| same_range(actual_range, offending)),
        C::DelimiterDepthExceeded => same_range(producing, offending),
        C::ListTrailingComma => malformed_actual_range(actual)
            .is_some_and(|actual_range| actual_range.start.column > offending.start.column),
    };
    if !exact || !range_shape {
        return Err("canonical_malformed_evidence_corrupt_v0");
    }
    Ok(())
}

fn expected_payload_fields(kind: CanonicalCommonNodeKind) -> &'static [CanonicalPayloadField] {
    use CanonicalPayloadField as F;
    match kind {
        CanonicalCommonNodeKind::Unit => &[F::UnitPosition],
        CanonicalCommonNodeKind::Identifier => &[F::IdentifierToken, F::IdentifierValue],
        CanonicalCommonNodeKind::UIntLiteral => &[F::UIntDigitsToken, F::UIntValue],
        CanonicalCommonNodeKind::IntLiteral => &[
            F::IntSignToken,
            F::IntDigitsToken,
            F::IntValue,
            F::IntSignedLiteral,
        ],
        CanonicalCommonNodeKind::BoolLiteral => &[F::BoolToken, F::BoolValue],
        CanonicalCommonNodeKind::TextLiteral => &[
            F::TextOpenQuote,
            F::TextCloseQuote,
            F::TextRawContent,
            F::TextEscapeEvents,
            F::TextDecodedValue,
            F::TextTerminated,
        ],
        CanonicalCommonNodeKind::Field => &[
            F::FieldBaseEdge,
            F::FieldDotToken,
            F::FieldNameToken,
            F::FieldValue,
        ],
        CanonicalCommonNodeKind::ElementPlace => &[
            F::DelimiterPair,
            F::DelimiterNestingParent,
            F::DelimiterSemanticGaps,
            F::ElementBaseEdge,
            F::ElementOpenBracket,
            F::ElementCloseBracket,
            F::ElementIndexToken,
            F::ElementIndexValue,
            F::ElementPlaceRole,
        ],
        CanonicalCommonNodeKind::Group => &[
            F::DelimiterPair,
            F::DelimiterNestingParent,
            F::DelimiterSemanticGaps,
            F::GroupValueEdge,
        ],
        CanonicalCommonNodeKind::ListLiteral => &[
            F::DelimiterPair,
            F::DelimiterNestingParent,
            F::DelimiterSemanticGaps,
            F::DelimiterSeparators,
            F::AggregateEmpty,
            F::AggregateTrailing,
            F::ListElementEdges,
        ],
        CanonicalCommonNodeKind::RecordLiteral => &[
            F::DelimiterPair,
            F::DelimiterNestingParent,
            F::DelimiterSemanticGaps,
            F::RecordNameToken,
            F::RecordFieldTokens,
            F::RecordColonTokens,
            F::DelimiterSeparators,
            F::RecordValueEdges,
            F::AggregateEmpty,
            F::AggregateTrailing,
        ],
        CanonicalCommonNodeKind::Call => &[
            F::DelimiterPair,
            F::DelimiterNestingParent,
            F::DelimiterSemanticGaps,
            F::DelimiterSeparators,
            F::AggregateEmpty,
            F::AggregateTrailing,
            F::CallCalleeEdge,
            F::CallArgumentEdges,
            F::CallAdjacency,
            F::CallCloseState,
            F::CallTrailingState,
        ],
        CanonicalCommonNodeKind::Binary => &[
            F::BinaryOperator,
            F::BinaryOperatorTokens,
            F::BinaryOperatorRange,
            F::BinaryPrecedence,
            F::BinaryAssociativity,
            F::BinaryLeftBoundary,
            F::BinaryRightBoundary,
            F::BinaryReductionOrder,
            F::BinaryChildRoles,
        ],
        CanonicalCommonNodeKind::Permission => &[
            F::PermissionKeyword,
            F::PermissionDiscriminant,
            F::PermissionGap,
            F::PermissionValueEdge,
        ],
        CanonicalCommonNodeKind::Try => &[
            F::TryKeyword,
            F::TryValueEdge,
            F::TryWrapperRelation,
            F::TryFailureRootToken,
            F::TryDotToken,
            F::TryFailureVariantToken,
            F::TryWrapperKind,
        ],
        CanonicalCommonNodeKind::Unsupported => &[],
    }
}

#[cfg(test)]
fn validate_occurrence_seal_ignoring_one_fact(
    seal: &CanonicalOccurrenceSeal,
    ignored_index: usize,
) -> Result<(), &'static str> {
    validate_occurrence_seal_inner(seal, Some(ignored_index), None)
}

#[cfg(test)]
fn validate_occurrence_seal_ignoring_one_payload_fact(
    seal: &CanonicalOccurrenceSeal,
    ignored_index: usize,
) -> Result<(), &'static str> {
    validate_occurrence_seal_inner(seal, None, Some(ignored_index))
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
    #[cfg_attr(not(test), allow(dead_code))]
    statement_seals: Vec<CanonicalStatementSeal>,
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
    statement_seals: Vec<CanonicalStatementSeal>,
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
        statement_seals: Vec::new(),
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
        statement_seals: parser.statement_seals,
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
                    &self.span(line.number),
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
        section_span: &Span,
        lines: &[SectionLine],
        body_syntax: &[Option<ParsedBodyStatement>],
        owners: &[Option<CanonicalSourceOwnerSeal>],
        semantic_node: &str,
    ) {
        let mut projected_blocks = Vec::<CanonicalStatementBlockIdentity>::new();
        let mut authority_blocks = Vec::<CanonicalStatementBlockIdentity>::new();
        let mut authority_depth = 0usize;
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
                let projected_statement = match owner.projection.get(5) {
                    Some(CanonicalSourceOwnerFact::Statement(statement)) => statement,
                    _ => panic!("statement projection requires statement owner"),
                };
                let projected_open = CanonicalStatementBlockIdentity(source_owner_child_identity(
                    16,
                    &projected_statement.0,
                    0,
                ));
                let authority_open = CanonicalStatementBlockIdentity(source_owner_child_identity(
                    16,
                    &owner.authority.statement.0,
                    0,
                ));
                let authority_relationship =
                    retained_parser_block_relationship(line.text.trim(), &line.span);
                let authority_before = authority_depth;
                authority_depth = match authority_relationship {
                    ParsedBlockRelationship::Opens => authority_depth.saturating_add(1),
                    ParsedBlockRelationship::Closes => authority_depth.saturating_sub(1),
                    ParsedBlockRelationship::None => authority_depth,
                };
                let authority_context = StatementBlockContext {
                    relationship: authority_relationship,
                    depth_before: authority_before,
                    depth_after: authority_depth,
                };
                let projected_block = match statement.block_relationship {
                    ParsedBlockRelationship::Opens => Some(projected_open.clone()),
                    ParsedBlockRelationship::Closes => projected_blocks
                        .pop()
                        .or_else(|| Some(projected_open.clone())),
                    ParsedBlockRelationship::None => projected_blocks.last().cloned(),
                };
                let authority_block = match authority_relationship {
                    ParsedBlockRelationship::Opens => Some(authority_open.clone()),
                    ParsedBlockRelationship::Closes => authority_blocks
                        .pop()
                        .or_else(|| Some(authority_open.clone())),
                    ParsedBlockRelationship::None => authority_blocks.last().cloned(),
                };
                let mut authority_events = statement.canonical_statement_authority.clone();
                let expression_absent = authority_events
                    .last()
                    .is_some_and(|fact| {
                        fact.field == CanonicalStatementEventField::ExpressionAbsent
                    })
                    .then(|| authority_events.pop().expect("checked statement fact"));
                append_retained_block_events(
                    &mut authority_events,
                    line.text.trim(),
                    &line.span,
                    authority_context,
                );
                authority_events.extend(expression_absent);
                let seal = build_statement_seal(
                    &statement.canonical_statement_projection,
                    &authority_events,
                    owner,
                    projected_block.as_ref(),
                    authority_block.as_ref(),
                );
                validate_statement_seal(&seal)
                    .expect("parser statement projection must match retained authority");
                self.statement_seals.push(seal);
                if statement.block_relationship == ParsedBlockRelationship::Opens {
                    projected_blocks.push(projected_open);
                }
                if authority_relationship == ParsedBlockRelationship::Opens {
                    authority_blocks.push(authority_open);
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
                let mut expression = parse_expression_syntax(
                    line.text.trim(),
                    line.span.clone(),
                    ParserSyntaxNodeId::new(format!(
                        "parser-contract:{semantic_node}:{section_name}:{line_index}"
                    )),
                );
                apply_predicate_completion(&mut expression, line.text.trim(), &line.span);
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
                let (projection, authority) = contract_statement_events(
                    section_name,
                    section_span,
                    line,
                    &expression,
                    &assignment,
                );
                let seal = build_statement_seal(&projection, &authority, owner, None, None);
                validate_statement_seal(&seal)
                    .expect("parser contract relationship must match retained authority");
                self.statement_seals.push(seal);
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
    if projected_keyword(text, "set") && index == 0 {
        (
            CanonicalExpressionRole::SetValue,
            CanonicalExpressionIntent::SetValue,
        )
    } else if projected_keyword(text, "set") {
        (
            CanonicalExpressionRole::Other,
            CanonicalExpressionIntent::Other,
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
    completion: CanonicalCompletionEvent,
    payload: Vec<CanonicalPayloadEvent>,
}

fn common_node_kind(expression: &CanonicalExpression) -> CanonicalCommonNodeKind {
    match expression.kind {
        CanonicalExpressionKind::Unit => CanonicalCommonNodeKind::Unit,
        CanonicalExpressionKind::Identifier(_) => CanonicalCommonNodeKind::Identifier,
        CanonicalExpressionKind::Field { .. } => CanonicalCommonNodeKind::Field,
        CanonicalExpressionKind::ElementPlace { .. } => CanonicalCommonNodeKind::ElementPlace,
        CanonicalExpressionKind::UIntLiteral(_) => CanonicalCommonNodeKind::UIntLiteral,
        CanonicalExpressionKind::IntLiteral(_) => CanonicalCommonNodeKind::IntLiteral,
        CanonicalExpressionKind::BoolLiteral(_) => CanonicalCommonNodeKind::BoolLiteral,
        CanonicalExpressionKind::TextLiteral(_) => CanonicalCommonNodeKind::TextLiteral,
        CanonicalExpressionKind::ListLiteral(_) => CanonicalCommonNodeKind::ListLiteral,
        CanonicalExpressionKind::RecordLiteral { .. } => CanonicalCommonNodeKind::RecordLiteral,
        CanonicalExpressionKind::Call { .. } => CanonicalCommonNodeKind::Call,
        CanonicalExpressionKind::Permission { .. } => CanonicalCommonNodeKind::Permission,
        CanonicalExpressionKind::Try { .. } => CanonicalCommonNodeKind::Try,
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
        CanonicalExpressionKind::ElementPlace { base, .. } => {
            vec![(CanonicalCommonChildRole::ElementBase, 0, base)]
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
        CanonicalExpressionKind::Try { value, .. } => {
            vec![(CanonicalCommonChildRole::TryValue, 0, value)]
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
        lexical_status: if matches!(
            expression.completion,
            CanonicalCompletionEvent::Unsupported(_)
        ) || (expression.payload.is_empty()
            && !expected_payload_fields(common_node_kind(expression)).is_empty())
        {
            CanonicalCommonLexicalStatus::Unsupported
        } else {
            CanonicalCommonLexicalStatus::Complete
        },
        completion: expression.completion.clone(),
        payload: expression.payload.clone(),
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
        lexical_status: if matches!(event.completion, CanonicalCompletionEvent::Unsupported(_))
            || (event.payload.is_empty() && !expected_payload_fields(event.kind).is_empty())
        {
            CanonicalCommonLexicalStatus::Unsupported
        } else {
            event.lexical_status
        },
        completion: event.completion.clone(),
        payload: event.payload.clone(),
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

fn payload_facts(
    occurrence: &CanonicalOccurrenceIdentity,
    nodes: &[CanonicalNodeEvidence],
    tokens: &[CanonicalLexicalTokenEvent],
) -> Vec<CanonicalPayloadSealFact> {
    let token_ids = (0..tokens.len())
        .map(|index| occurrence_token_identity(occurrence, index))
        .collect::<Vec<_>>();
    let token_value = |range: &ParsedSourceRange, spelling: &str| {
        tokens
            .iter()
            .enumerate()
            .find(|(_, token)| token.range == *range && token.spelling == spelling)
            .map(|(index, token)| {
                CanonicalPayloadValue::Token(
                    token_ids[index].clone(),
                    token.range.clone(),
                    token.spelling.clone(),
                )
            })
            .unwrap_or_else(|| {
                panic!(
                    "payload token {spelling:?} at {range:?} must be a parser lexical event; retained tokens: {tokens:?}"
                )
            })
    };
    let tokens_value = |values: &[(ParsedSourceRange, String)]| {
        CanonicalPayloadValue::Tokens(
            values
                .iter()
                .map(|(range, spelling)| match token_value(range, spelling) {
                    CanonicalPayloadValue::Token(identity, range, spelling) => {
                        (identity, range, spelling)
                    }
                    _ => unreachable!(),
                })
                .collect(),
        )
    };
    let mut facts = Vec::new();
    for (preorder, node) in nodes.iter().enumerate() {
        let node_id = occurrence_node_identity(occurrence, preorder);
        for (payload_ordinal, event) in node.payload.iter().enumerate() {
            let value = match &event.value {
                CanonicalPayloadEventValue::Position(value) => {
                    CanonicalPayloadValue::Position(value.clone())
                }
                CanonicalPayloadEventValue::Token(range, spelling) => token_value(range, spelling),
                CanonicalPayloadEventValue::Tokens(values) => tokens_value(values),
                CanonicalPayloadEventValue::Range(value) => {
                    CanonicalPayloadValue::Range(value.clone())
                }
                CanonicalPayloadEventValue::Ranges(values) => {
                    CanonicalPayloadValue::Ranges(values.clone())
                }
                CanonicalPayloadEventValue::Text(value) => {
                    CanonicalPayloadValue::Text(value.clone())
                }
                CanonicalPayloadEventValue::UInt(value) => CanonicalPayloadValue::UInt(*value),
                CanonicalPayloadEventValue::Int(value) => CanonicalPayloadValue::Int(*value),
                CanonicalPayloadEventValue::Bool(value) => CanonicalPayloadValue::Bool(*value),
                CanonicalPayloadEventValue::Usize(value) => CanonicalPayloadValue::Usize(*value),
                CanonicalPayloadEventValue::Bools(values) => {
                    CanonicalPayloadValue::Bools(values.clone())
                }
                CanonicalPayloadEventValue::Parent => {
                    let mut parent = node.parent;
                    while let Some(index) = parent {
                        if matches!(
                            nodes[index].kind,
                            CanonicalCommonNodeKind::ElementPlace
                                | CanonicalCommonNodeKind::Group
                                | CanonicalCommonNodeKind::ListLiteral
                                | CanonicalCommonNodeKind::RecordLiteral
                                | CanonicalCommonNodeKind::Call
                        ) {
                            break;
                        }
                        parent = nodes[index].parent;
                    }
                    CanonicalPayloadValue::OptionalNode(
                        parent.map(|parent| occurrence_node_identity(occurrence, parent)),
                    )
                }
                CanonicalPayloadEventValue::ChildOrdinal(ordinal) => CanonicalPayloadValue::Node(
                    occurrence_node_identity(occurrence, node.children[*ordinal]),
                ),
                CanonicalPayloadEventValue::ChildOrdinals(ordinals) => {
                    CanonicalPayloadValue::Nodes(
                        ordinals
                            .iter()
                            .map(|ordinal| {
                                occurrence_node_identity(occurrence, node.children[*ordinal])
                            })
                            .collect(),
                    )
                }
                CanonicalPayloadEventValue::DelimiterPair { kind, open, close } => {
                    let open_value = token_value(open, &delimiter_spelling(*kind, true));
                    let close_value = token_value(close, &delimiter_spelling(*kind, false));
                    let CanonicalPayloadValue::Token(open_id, open_range, _) = open_value else {
                        unreachable!()
                    };
                    let CanonicalPayloadValue::Token(close_id, close_range, _) = close_value else {
                        unreachable!()
                    };
                    CanonicalPayloadValue::DelimiterPair {
                        kind: *kind,
                        open: open_id,
                        open_range,
                        close: close_id,
                        close_range,
                    }
                }
                CanonicalPayloadEventValue::Operator(value) => {
                    CanonicalPayloadValue::Operator(*value)
                }
                CanonicalPayloadEventValue::Permission(value) => {
                    CanonicalPayloadValue::Permission(*value)
                }
                CanonicalPayloadEventValue::Associativity(value) => {
                    CanonicalPayloadValue::Associativity(*value)
                }
                CanonicalPayloadEventValue::WrapperKind(value) => {
                    CanonicalPayloadValue::WrapperKind(*value)
                }
            };
            facts.push(CanonicalPayloadSealFact {
                identity: CanonicalPayloadIdentity(source_owner_child_identity(
                    16,
                    &node_id.0,
                    payload_ordinal,
                )),
                node: node_id.clone(),
                field: event.field,
                value,
            });
        }
    }
    facts
}

fn delimiter_spelling(kind: CanonicalDelimiterKind, open: bool) -> String {
    match (kind, open) {
        (CanonicalDelimiterKind::Parenthesis, true) => "(",
        (CanonicalDelimiterKind::Parenthesis, false) => ")",
        (CanonicalDelimiterKind::List | CanonicalDelimiterKind::Element, true) => "[",
        (CanonicalDelimiterKind::List | CanonicalDelimiterKind::Element, false) => "]",
        (CanonicalDelimiterKind::Record, true) => "{",
        (CanonicalDelimiterKind::Record, false) => "}",
    }
    .to_string()
}

fn source_range_at(span: &Span, start: usize, byte_len: usize) -> ParsedSourceRange {
    ParsedSourceRange {
        start: offset_span(span, start),
        byte_len,
    }
}

fn payload_token(
    field: CanonicalPayloadField,
    span: &Span,
    start: usize,
    spelling: &str,
) -> CanonicalPayloadEvent {
    CanonicalPayloadEvent {
        field,
        value: CanonicalPayloadEventValue::Token(
            source_range_at(span, start, spelling.len()),
            spelling.to_string(),
        ),
    }
}

fn payload_value(
    field: CanonicalPayloadField,
    value: CanonicalPayloadEventValue,
) -> CanonicalPayloadEvent {
    CanonicalPayloadEvent { field, value }
}

fn retained_delimiter_payload(
    kind: CanonicalDelimiterKind,
    span: &Span,
    text: &str,
    open: usize,
    close: usize,
) -> Vec<CanonicalPayloadEvent> {
    let open_range = source_range_at(span, open, 1);
    let close_range = source_range_at(span, close, 1);
    let gaps = retained_semantic_gap_ranges(text, span, open, close);
    vec![
        payload_value(
            CanonicalPayloadField::DelimiterPair,
            CanonicalPayloadEventValue::DelimiterPair {
                kind,
                open: open_range,
                close: close_range,
            },
        ),
        payload_value(
            CanonicalPayloadField::DelimiterNestingParent,
            CanonicalPayloadEventValue::Parent,
        ),
        payload_value(
            CanonicalPayloadField::DelimiterSemanticGaps,
            CanonicalPayloadEventValue::Ranges(gaps),
        ),
    ]
}

fn projected_separator_offsets(text: &str, separator: char) -> Vec<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    let mut offsets = Vec::new();
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
            _ if depth == 0 && ch == separator => offsets.push(index),
            _ => {}
        }
    }
    offsets
}

fn retained_separator_offsets(text: &str, separator: char) -> Vec<usize> {
    let mut stack = Vec::new();
    let mut quote_open = false;
    let mut escape_pending = false;
    let mut offsets = Vec::new();
    for (index, ch) in text.char_indices() {
        if quote_open {
            if escape_pending {
                escape_pending = false;
            } else {
                match ch {
                    '\\' => escape_pending = true,
                    '"' => quote_open = false,
                    _ => {}
                }
            }
            continue;
        }
        match ch {
            '"' => quote_open = true,
            '(' | '[' | '{' => stack.push(ch),
            ')' | ']' | '}' => {
                stack.pop();
            }
            _ if stack.is_empty() && ch == separator => offsets.push(index),
            _ => {}
        }
    }
    offsets
}

fn projected_top_level_separator_tokens(
    text: &str,
    span: &Span,
    separator: char,
) -> Vec<(ParsedSourceRange, String)> {
    projected_separator_offsets(text, separator)
        .into_iter()
        .map(|index| {
            (
                source_range_at(span, index, separator.len_utf8()),
                separator.to_string(),
            )
        })
        .collect()
}

fn retained_top_level_separator_tokens(
    text: &str,
    span: &Span,
    separator: char,
) -> Vec<(ParsedSourceRange, String)> {
    retained_separator_offsets(text, separator)
        .into_iter()
        .map(|index| {
            (
                source_range_at(span, index, separator.len_utf8()),
                separator.to_string(),
            )
        })
        .collect()
}

fn projected_semantic_gap_ranges(
    text: &str,
    span: &Span,
    open: usize,
    close: usize,
) -> Vec<ParsedSourceRange> {
    let inside = &text[open + 1..close];
    let inside_span = offset_span(span, open + 1);
    let mut ranges = Vec::new();
    for segment in split_top_level_ranges_quoted(inside, ',') {
        let value = &inside[segment.clone()];
        let leading = value.len() - value.trim_start().len();
        let trailing_start = value.trim_end().len();
        for (start, len) in [
            (segment.start, leading),
            (segment.start + trailing_start, value.len() - trailing_start),
        ] {
            if len > 0 {
                let range = source_range_at(&inside_span, start, len);
                if !ranges.contains(&range) {
                    ranges.push(range);
                }
            }
        }
    }
    ranges
}

fn retained_semantic_gap_ranges(
    text: &str,
    span: &Span,
    open: usize,
    close: usize,
) -> Vec<ParsedSourceRange> {
    let inside = &text[open + 1..close];
    let inside_span = offset_span(span, open + 1);
    let mut boundaries = retained_separator_offsets(inside, ',');
    boundaries.push(inside.len());
    let mut start = 0usize;
    let mut ranges = Vec::new();
    for end in boundaries {
        let value = &inside[start..end];
        let leading = value.len() - value.trim_start().len();
        let trailing_start = value.trim_end().len();
        for (offset, len) in [
            (start, leading),
            (start + trailing_start, value.len() - trailing_start),
        ] {
            if len > 0 {
                let range = source_range_at(&inside_span, offset, len);
                if !ranges.contains(&range) {
                    ranges.push(range);
                }
            }
        }
        start = end.saturating_add(1);
    }
    ranges
}

fn projected_call_adjacency_facts(text: &str, open: usize) -> Vec<bool> {
    let close = text.len() - 1;
    let inside = &text[open + 1..close];
    let separators = projected_separator_offsets(inside, ',');
    let mut facts = vec![
        text[..open]
            .chars()
            .next_back()
            .is_some_and(|ch| !ch.is_whitespace()),
        inside.chars().next().is_none_or(|ch| !ch.is_whitespace()),
    ];
    for index in separators {
        facts.push(
            inside[index + 1..]
                .chars()
                .next()
                .is_some_and(char::is_whitespace),
        );
    }
    facts.push(
        inside
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_whitespace()),
    );
    facts
}

fn retained_call_adjacency_facts(text: &str, open: usize) -> Vec<bool> {
    let close = text.len() - 1;
    let inside = &text[open + 1..close];
    let separators = retained_separator_offsets(inside, ',');
    let mut facts = Vec::with_capacity(separators.len() + 3);
    facts.push(
        text.get(..open)
            .and_then(|prefix| prefix.chars().next_back())
            .is_some_and(|ch| !ch.is_whitespace()),
    );
    facts.push(inside.chars().next().is_none_or(|ch| !ch.is_whitespace()));
    for separator in separators {
        facts.push(
            inside
                .get(separator + 1..)
                .and_then(|suffix| suffix.chars().next())
                .is_some_and(char::is_whitespace),
        );
    }
    facts.push(
        inside
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_whitespace()),
    );
    facts
}

fn binary_boundaries(text: &str, start: usize, end: usize) -> (bool, bool) {
    let left = text[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'));
    let right = text[end..]
        .chars()
        .next()
        .is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'));
    (left, right)
}

fn binary_operator_is_in_malformed_comparison_run(text: &str, start: usize, end: usize) -> bool {
    let comparison_punctuation = |ch: char| matches!(ch, '=' | '!' | '<' | '>');
    text[..start]
        .chars()
        .next_back()
        .is_some_and(comparison_punctuation)
        || text[end..]
            .chars()
            .next()
            .is_some_and(comparison_punctuation)
}

fn operator_precedence(operator: ParsedBinaryOperator) -> usize {
    match operator {
        ParsedBinaryOperator::Or => 1,
        ParsedBinaryOperator::And => 2,
        ParsedBinaryOperator::Equal
        | ParsedBinaryOperator::NotEqual
        | ParsedBinaryOperator::Less
        | ParsedBinaryOperator::LessEqual
        | ParsedBinaryOperator::Greater
        | ParsedBinaryOperator::GreaterEqual
        | ParsedBinaryOperator::Is
        | ParsedBinaryOperator::Does
        | ParsedBinaryOperator::Returns
        | ParsedBinaryOperator::FailsWith => 3,
        ParsedBinaryOperator::Add | ParsedBinaryOperator::Subtract => 4,
        ParsedBinaryOperator::Multiply | ParsedBinaryOperator::Divide => 5,
    }
}

fn projected_decoded_text(value: &str) -> String {
    let mut decoded = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            decoded.push(ch);
            continue;
        }
        let Some(escaped) = chars.next() else {
            decoded.push('\\');
            break;
        };
        match escaped {
            '"' => decoded.push('"'),
            '\\' => decoded.push('\\'),
            'n' => decoded.push('\n'),
            'r' => decoded.push('\r'),
            't' => decoded.push('\t'),
            other => {
                decoded.push('\\');
                decoded.push(other);
            }
        }
    }
    decoded
}

fn retained_decoded_text(value: &str) -> String {
    let mut decoded = String::with_capacity(value.len());
    let mut positions = value.char_indices().peekable();
    while let Some((_, current)) = positions.next() {
        if current != '\\' {
            decoded.push(current);
            continue;
        }
        let Some((_, escaped)) = positions.next() else {
            decoded.push('\\');
            continue;
        };
        match escaped {
            'n' => decoded.push('\n'),
            'r' => decoded.push('\r'),
            't' => decoded.push('\t'),
            '"' => decoded.push('"'),
            '\\' => decoded.push('\\'),
            other => {
                decoded.extend(['\\', other]);
            }
        }
    }
    decoded
}

fn projected_text_escape_events(text: &str, span: &Span) -> Vec<(ParsedSourceRange, String)> {
    let interior = &text[1..text.len() - 1];
    let mut events = Vec::new();
    let mut chars = interior.char_indices();
    while let Some((index, ch)) = chars.next() {
        if ch != '\\' {
            continue;
        }
        let end = chars
            .next()
            .map_or(index + 1, |(next, escaped)| next + escaped.len_utf8());
        events.push((
            source_range_at(span, 1 + index, end - index),
            interior[index..end].to_string(),
        ));
    }
    events
}

fn retained_text_escape_events(text: &str, span: &Span) -> Vec<(ParsedSourceRange, String)> {
    canonical_lexical_events(text, span)
        .into_iter()
        .filter(|event| event.spelling.starts_with('\\'))
        .map(|event| (event.range, event.spelling))
        .collect()
}

fn projected_delimiter_payload(
    kind: CanonicalDelimiterKind,
    span: &Span,
    text: &str,
    open: usize,
    close: usize,
) -> Vec<CanonicalPayloadEvent> {
    vec![
        payload_value(
            CanonicalPayloadField::DelimiterPair,
            CanonicalPayloadEventValue::DelimiterPair {
                kind,
                open: source_range_at(span, open, 1),
                close: source_range_at(span, close, 1),
            },
        ),
        payload_value(
            CanonicalPayloadField::DelimiterNestingParent,
            CanonicalPayloadEventValue::Parent,
        ),
        payload_value(
            CanonicalPayloadField::DelimiterSemanticGaps,
            CanonicalPayloadEventValue::Ranges(projected_semantic_gap_ranges(
                text, span, open, close,
            )),
        ),
    ]
}

fn projected_payload_events(
    kind: &CanonicalExpressionKind,
    text: &str,
    span: &Span,
    child_count: usize,
) -> Vec<CanonicalPayloadEvent> {
    let mut events = Vec::new();
    match kind {
        CanonicalExpressionKind::Unit => events.push(payload_value(
            CanonicalPayloadField::UnitPosition,
            CanonicalPayloadEventValue::Position(span.clone()),
        )),
        CanonicalExpressionKind::Identifier(value) => events.extend([
            payload_token(CanonicalPayloadField::IdentifierToken, span, 0, text),
            payload_value(
                CanonicalPayloadField::IdentifierValue,
                CanonicalPayloadEventValue::Text(value.clone()),
            ),
        ]),
        CanonicalExpressionKind::UIntLiteral(value) => events.extend([
            payload_token(CanonicalPayloadField::UIntDigitsToken, span, 0, text),
            payload_value(
                CanonicalPayloadField::UIntValue,
                CanonicalPayloadEventValue::UInt(*value),
            ),
        ]),
        CanonicalExpressionKind::IntLiteral(value) => events.extend([
            payload_token(CanonicalPayloadField::IntSignToken, span, 0, &text[..1]),
            payload_token(CanonicalPayloadField::IntDigitsToken, span, 1, &text[1..]),
            payload_value(
                CanonicalPayloadField::IntValue,
                CanonicalPayloadEventValue::Int(*value),
            ),
            payload_value(
                CanonicalPayloadField::IntSignedLiteral,
                CanonicalPayloadEventValue::Bool(text.starts_with('-')),
            ),
        ]),
        CanonicalExpressionKind::BoolLiteral(value) => events.extend([
            payload_token(CanonicalPayloadField::BoolToken, span, 0, text),
            payload_value(
                CanonicalPayloadField::BoolValue,
                CanonicalPayloadEventValue::Bool(*value),
            ),
        ]),
        CanonicalExpressionKind::TextLiteral(value) => {
            let escapes = projected_text_escape_events(text, span);
            events.extend([
                payload_token(CanonicalPayloadField::TextOpenQuote, span, 0, "\""),
                payload_token(
                    CanonicalPayloadField::TextCloseQuote,
                    span,
                    text.len() - 1,
                    "\"",
                ),
                payload_value(
                    CanonicalPayloadField::TextRawContent,
                    CanonicalPayloadEventValue::Range(source_range_at(span, 1, text.len() - 2)),
                ),
                payload_value(
                    CanonicalPayloadField::TextEscapeEvents,
                    CanonicalPayloadEventValue::Tokens(escapes),
                ),
                payload_value(
                    CanonicalPayloadField::TextDecodedValue,
                    CanonicalPayloadEventValue::Text(projected_decoded_text(value)),
                ),
                payload_value(
                    CanonicalPayloadField::TextTerminated,
                    CanonicalPayloadEventValue::Bool(text.ends_with('"')),
                ),
            ]);
        }
        CanonicalExpressionKind::Field { field, .. } => {
            let dot = find_top_level_dot(text).expect("canonical field dot");
            let field_start = text.rfind(field).expect("canonical field token");
            events.extend([
                payload_value(
                    CanonicalPayloadField::FieldBaseEdge,
                    CanonicalPayloadEventValue::ChildOrdinal(0),
                ),
                payload_token(CanonicalPayloadField::FieldDotToken, span, dot, "."),
                payload_token(
                    CanonicalPayloadField::FieldNameToken,
                    span,
                    field_start,
                    field,
                ),
                payload_value(
                    CanonicalPayloadField::FieldValue,
                    CanonicalPayloadEventValue::Text(field.clone()),
                ),
            ]);
        }
        CanonicalExpressionKind::ElementPlace { index, .. } => {
            let open = terminal_element_open(text).expect("canonical element open");
            let close = text.len() - 1;
            let index_text = text[open + 1..close].trim();
            let index_start = open + 1 + text[open + 1..close].find(index_text).unwrap_or_default();
            events.extend(projected_delimiter_payload(
                CanonicalDelimiterKind::Element,
                span,
                text,
                open,
                close,
            ));
            events.extend([
                payload_value(
                    CanonicalPayloadField::ElementBaseEdge,
                    CanonicalPayloadEventValue::ChildOrdinal(0),
                ),
                payload_token(CanonicalPayloadField::ElementOpenBracket, span, open, "["),
                payload_token(CanonicalPayloadField::ElementCloseBracket, span, close, "]"),
                payload_token(
                    CanonicalPayloadField::ElementIndexToken,
                    span,
                    index_start,
                    index_text,
                ),
                payload_value(
                    CanonicalPayloadField::ElementIndexValue,
                    CanonicalPayloadEventValue::UInt(*index),
                ),
                payload_value(
                    CanonicalPayloadField::ElementPlaceRole,
                    CanonicalPayloadEventValue::Bool(true),
                ),
            ]);
        }
        CanonicalExpressionKind::Group(_) => {
            events.extend(projected_delimiter_payload(
                CanonicalDelimiterKind::Parenthesis,
                span,
                text,
                0,
                text.len() - 1,
            ));
            events.push(payload_value(
                CanonicalPayloadField::GroupValueEdge,
                CanonicalPayloadEventValue::ChildOrdinal(0),
            ));
        }
        CanonicalExpressionKind::ListLiteral(values) => {
            let inside = &text[1..text.len() - 1];
            events.extend(projected_delimiter_payload(
                CanonicalDelimiterKind::List,
                span,
                text,
                0,
                text.len() - 1,
            ));
            events.extend([
                payload_value(
                    CanonicalPayloadField::DelimiterSeparators,
                    CanonicalPayloadEventValue::Tokens(projected_top_level_separator_tokens(
                        inside,
                        &offset_span(span, 1),
                        ',',
                    )),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateEmpty,
                    CanonicalPayloadEventValue::Bool(values.is_empty()),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateTrailing,
                    CanonicalPayloadEventValue::Bool(inside.trim_end().ends_with(',')),
                ),
                payload_value(
                    CanonicalPayloadField::ListElementEdges,
                    CanonicalPayloadEventValue::ChildOrdinals((0..child_count).collect()),
                ),
            ]);
        }
        CanonicalExpressionKind::RecordLiteral { name, fields } => {
            let open = text.find('{').expect("canonical record open");
            if !text.ends_with('}') {
                return Vec::new();
            }
            let inside = &text[open + 1..text.len() - 1];
            events.extend(projected_delimiter_payload(
                CanonicalDelimiterKind::Record,
                span,
                text,
                open,
                text.len() - 1,
            ));
            let name_tokens = if name.is_empty() {
                Vec::new()
            } else {
                vec![(
                    source_range_at(
                        span,
                        text[..open].find(name).unwrap_or_default(),
                        name.len(),
                    ),
                    name.clone(),
                )]
            };
            let field_ranges = split_top_level_ranges_quoted(inside, ',');
            let mut field_tokens = Vec::new();
            let mut colon_tokens = Vec::new();
            for (ordinal, range) in field_ranges.into_iter().enumerate() {
                let raw = &inside[range.clone()];
                if let Some(colon) = find_top_level_char(raw, ':') {
                    let field = fields
                        .get(ordinal)
                        .map(|(field, _)| field.as_str())
                        .unwrap_or_else(|| raw[..colon].trim());
                    let start =
                        open + 1 + range.start + raw[..colon].find(field).unwrap_or_default();
                    field_tokens
                        .push((source_range_at(span, start, field.len()), field.to_string()));
                    colon_tokens.push((
                        source_range_at(span, open + 1 + range.start + colon, 1),
                        ":".to_string(),
                    ));
                }
            }
            events.extend([
                payload_value(
                    CanonicalPayloadField::RecordNameToken,
                    CanonicalPayloadEventValue::Tokens(name_tokens),
                ),
                payload_value(
                    CanonicalPayloadField::RecordFieldTokens,
                    CanonicalPayloadEventValue::Tokens(field_tokens),
                ),
                payload_value(
                    CanonicalPayloadField::RecordColonTokens,
                    CanonicalPayloadEventValue::Tokens(colon_tokens),
                ),
                payload_value(
                    CanonicalPayloadField::DelimiterSeparators,
                    CanonicalPayloadEventValue::Tokens(projected_top_level_separator_tokens(
                        inside,
                        &offset_span(span, open + 1),
                        ',',
                    )),
                ),
                payload_value(
                    CanonicalPayloadField::RecordValueEdges,
                    CanonicalPayloadEventValue::ChildOrdinals((0..child_count).collect()),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateEmpty,
                    CanonicalPayloadEventValue::Bool(fields.is_empty()),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateTrailing,
                    CanonicalPayloadEventValue::Bool(inside.trim_end().ends_with(',')),
                ),
            ]);
        }
        CanonicalExpressionKind::Call { arguments, .. } => {
            let open = find_top_level_open_paren(text).expect("canonical call open");
            let inside = &text[open + 1..text.len() - 1];
            events.extend(projected_delimiter_payload(
                CanonicalDelimiterKind::Parenthesis,
                span,
                text,
                open,
                text.len() - 1,
            ));
            events.extend([
                payload_value(
                    CanonicalPayloadField::DelimiterSeparators,
                    CanonicalPayloadEventValue::Tokens(projected_top_level_separator_tokens(
                        inside,
                        &offset_span(span, open + 1),
                        ',',
                    )),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateEmpty,
                    CanonicalPayloadEventValue::Bool(arguments.is_empty()),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateTrailing,
                    CanonicalPayloadEventValue::Bool(inside.trim_end().ends_with(',')),
                ),
                payload_value(
                    CanonicalPayloadField::CallCalleeEdge,
                    CanonicalPayloadEventValue::ChildOrdinal(0),
                ),
                payload_value(
                    CanonicalPayloadField::CallArgumentEdges,
                    CanonicalPayloadEventValue::ChildOrdinals((1..child_count).collect()),
                ),
                payload_value(
                    CanonicalPayloadField::CallAdjacency,
                    CanonicalPayloadEventValue::Bools(projected_call_adjacency_facts(text, open)),
                ),
                payload_value(
                    CanonicalPayloadField::CallCloseState,
                    CanonicalPayloadEventValue::Bool(true),
                ),
                payload_value(
                    CanonicalPayloadField::CallTrailingState,
                    CanonicalPayloadEventValue::Bool(!inside.trim_end().ends_with(',')),
                ),
            ]);
        }
        CanonicalExpressionKind::Binary { operator, .. } => {
            let (_, start, end) =
                top_level_binary_operator(text).expect("canonical binary operator");
            let operator_text = &text[start..end];
            let operator_tokens =
                canonical_lexical_events(operator_text, &offset_span(span, start))
                    .into_iter()
                    .map(|event| (event.range, event.spelling))
                    .collect();
            let (left_boundary, right_boundary) = binary_boundaries(text, start, end);
            events.extend([
                payload_value(
                    CanonicalPayloadField::BinaryOperator,
                    CanonicalPayloadEventValue::Operator(*operator),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryOperatorTokens,
                    CanonicalPayloadEventValue::Tokens(operator_tokens),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryOperatorRange,
                    CanonicalPayloadEventValue::Range(source_range_at(span, start, end - start)),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryPrecedence,
                    CanonicalPayloadEventValue::Usize(operator_precedence(*operator)),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryAssociativity,
                    CanonicalPayloadEventValue::Associativity(CanonicalAssociativity::Left),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryLeftBoundary,
                    CanonicalPayloadEventValue::Bool(left_boundary),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryRightBoundary,
                    CanonicalPayloadEventValue::Bool(right_boundary),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryReductionOrder,
                    CanonicalPayloadEventValue::ChildOrdinals(vec![0, 1]),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryChildRoles,
                    CanonicalPayloadEventValue::ChildOrdinals(vec![0, 1]),
                ),
            ]);
        }
        CanonicalExpressionKind::Permission { permission, .. } => {
            let keyword = permission.as_str();
            let value_start = text.len()
                - keyword_rest(text, keyword)
                    .expect("canonical permission value")
                    .len();
            events.extend([
                payload_token(CanonicalPayloadField::PermissionKeyword, span, 0, keyword),
                payload_value(
                    CanonicalPayloadField::PermissionDiscriminant,
                    CanonicalPayloadEventValue::Permission(*permission),
                ),
                payload_value(
                    CanonicalPayloadField::PermissionGap,
                    CanonicalPayloadEventValue::Range(source_range_at(
                        span,
                        keyword.len(),
                        value_start - keyword.len(),
                    )),
                ),
                payload_value(
                    CanonicalPayloadField::PermissionValueEdge,
                    CanonicalPayloadEventValue::ChildOrdinal(0),
                ),
            ]);
        }
        CanonicalExpressionKind::Try {
            failure_root,
            failure_variant,
            ..
        } => {
            let rest = keyword_rest(text, "try").expect("canonical try value");
            let value_start = text.len() - rest.len();
            let wrapper_start = find_top_level_phrase(rest, " or fail ");
            let mut relation = Vec::new();
            let mut roots = Vec::new();
            let mut dots = Vec::new();
            let mut variants = Vec::new();
            if let Some(start) = wrapper_start {
                let absolute = value_start + start;
                relation.extend([
                    (source_range_at(span, absolute + 1, 2), "or".to_string()),
                    (source_range_at(span, absolute + 4, 4), "fail".to_string()),
                ]);
                let failure_start = absolute + " or fail ".len();
                if let Some(root) = failure_root {
                    roots.push((
                        source_range_at(span, failure_start, root.len()),
                        root.clone(),
                    ));
                }
                if let Some(dot) = text[failure_start..].find('.') {
                    dots.push((
                        source_range_at(span, failure_start + dot, 1),
                        ".".to_string(),
                    ));
                    if let Some(variant) = failure_variant {
                        variants.push((
                            source_range_at(span, failure_start + dot + 1, variant.len()),
                            variant.clone(),
                        ));
                    }
                }
            }
            events.extend([
                payload_token(CanonicalPayloadField::TryKeyword, span, 0, "try"),
                payload_value(
                    CanonicalPayloadField::TryValueEdge,
                    CanonicalPayloadEventValue::ChildOrdinal(0),
                ),
                payload_value(
                    CanonicalPayloadField::TryWrapperRelation,
                    CanonicalPayloadEventValue::Tokens(relation),
                ),
                payload_value(
                    CanonicalPayloadField::TryFailureRootToken,
                    CanonicalPayloadEventValue::Tokens(roots),
                ),
                payload_value(
                    CanonicalPayloadField::TryDotToken,
                    CanonicalPayloadEventValue::Tokens(dots),
                ),
                payload_value(
                    CanonicalPayloadField::TryFailureVariantToken,
                    CanonicalPayloadEventValue::Tokens(variants),
                ),
                payload_value(
                    CanonicalPayloadField::TryWrapperKind,
                    CanonicalPayloadEventValue::WrapperKind(if wrapper_start.is_some() {
                        CanonicalTryWrapperKind::Wrap
                    } else {
                        CanonicalTryWrapperKind::Propagate
                    }),
                ),
            ]);
        }
        CanonicalExpressionKind::Unsupported => {}
    }
    events
}

fn retained_payload_events(
    kind: CanonicalCommonNodeKind,
    text: &str,
    span: &Span,
    child_count: usize,
) -> Vec<CanonicalPayloadEvent> {
    let mut events = Vec::new();
    match kind {
        CanonicalCommonNodeKind::Unit => events.push(payload_value(
            CanonicalPayloadField::UnitPosition,
            CanonicalPayloadEventValue::Position(span.clone()),
        )),
        CanonicalCommonNodeKind::Identifier => {
            events.push(payload_token(
                CanonicalPayloadField::IdentifierToken,
                span,
                0,
                text,
            ));
            events.push(payload_value(
                CanonicalPayloadField::IdentifierValue,
                CanonicalPayloadEventValue::Text(text.to_string()),
            ));
        }
        CanonicalCommonNodeKind::UIntLiteral => {
            events.push(payload_token(
                CanonicalPayloadField::UIntDigitsToken,
                span,
                0,
                text,
            ));
            events.push(payload_value(
                CanonicalPayloadField::UIntValue,
                CanonicalPayloadEventValue::UInt(text.parse().expect("successful UInt payload")),
            ));
        }
        CanonicalCommonNodeKind::IntLiteral => {
            events.push(payload_token(
                CanonicalPayloadField::IntSignToken,
                span,
                0,
                &text[..1],
            ));
            events.push(payload_token(
                CanonicalPayloadField::IntDigitsToken,
                span,
                1,
                &text[1..],
            ));
            events.push(payload_value(
                CanonicalPayloadField::IntValue,
                CanonicalPayloadEventValue::Int(text.parse().expect("successful Int payload")),
            ));
            events.push(payload_value(
                CanonicalPayloadField::IntSignedLiteral,
                CanonicalPayloadEventValue::Bool(true),
            ));
        }
        CanonicalCommonNodeKind::BoolLiteral => {
            events.push(payload_token(
                CanonicalPayloadField::BoolToken,
                span,
                0,
                text,
            ));
            events.push(payload_value(
                CanonicalPayloadField::BoolValue,
                CanonicalPayloadEventValue::Bool(text == "true"),
            ));
        }
        CanonicalCommonNodeKind::TextLiteral => {
            events.push(payload_token(
                CanonicalPayloadField::TextOpenQuote,
                span,
                0,
                "\"",
            ));
            events.push(payload_token(
                CanonicalPayloadField::TextCloseQuote,
                span,
                text.len() - 1,
                "\"",
            ));
            events.push(payload_value(
                CanonicalPayloadField::TextRawContent,
                CanonicalPayloadEventValue::Range(source_range_at(span, 1, text.len() - 2)),
            ));
            let escapes = retained_text_escape_events(text, span);
            events.push(payload_value(
                CanonicalPayloadField::TextEscapeEvents,
                CanonicalPayloadEventValue::Tokens(escapes),
            ));
            events.push(payload_value(
                CanonicalPayloadField::TextDecodedValue,
                CanonicalPayloadEventValue::Text(retained_decoded_text(&text[1..text.len() - 1])),
            ));
            events.push(payload_value(
                CanonicalPayloadField::TextTerminated,
                CanonicalPayloadEventValue::Bool(true),
            ));
        }
        CanonicalCommonNodeKind::Field => {
            let dot = find_top_level_dot(text).expect("successful field dot");
            let field = text[dot + 1..].trim();
            let field_start = text.rfind(field).expect("field token");
            events.extend([
                payload_value(
                    CanonicalPayloadField::FieldBaseEdge,
                    CanonicalPayloadEventValue::ChildOrdinal(0),
                ),
                payload_token(CanonicalPayloadField::FieldDotToken, span, dot, "."),
                payload_token(
                    CanonicalPayloadField::FieldNameToken,
                    span,
                    field_start,
                    field,
                ),
                payload_value(
                    CanonicalPayloadField::FieldValue,
                    CanonicalPayloadEventValue::Text(field.to_string()),
                ),
            ]);
        }
        CanonicalCommonNodeKind::ElementPlace => {
            let open = terminal_element_open(text).expect("successful element open");
            let close = text.len() - 1;
            let index_text = text[open + 1..close].trim();
            let index_start = open + 1 + text[open + 1..close].find(index_text).unwrap_or_default();
            events.extend(retained_delimiter_payload(
                CanonicalDelimiterKind::Element,
                span,
                text,
                open,
                close,
            ));
            events.extend([
                payload_value(
                    CanonicalPayloadField::ElementBaseEdge,
                    CanonicalPayloadEventValue::ChildOrdinal(0),
                ),
                payload_token(CanonicalPayloadField::ElementOpenBracket, span, open, "["),
                payload_token(CanonicalPayloadField::ElementCloseBracket, span, close, "]"),
                payload_token(
                    CanonicalPayloadField::ElementIndexToken,
                    span,
                    index_start,
                    index_text,
                ),
                payload_value(
                    CanonicalPayloadField::ElementIndexValue,
                    CanonicalPayloadEventValue::UInt(
                        index_text.parse().expect("successful element index"),
                    ),
                ),
                payload_value(
                    CanonicalPayloadField::ElementPlaceRole,
                    CanonicalPayloadEventValue::Bool(true),
                ),
            ]);
        }
        CanonicalCommonNodeKind::Group => {
            events.extend(retained_delimiter_payload(
                CanonicalDelimiterKind::Parenthesis,
                span,
                text,
                0,
                text.len() - 1,
            ));
            events.push(payload_value(
                CanonicalPayloadField::GroupValueEdge,
                CanonicalPayloadEventValue::ChildOrdinal(0),
            ));
        }
        CanonicalCommonNodeKind::ListLiteral => {
            events.extend(retained_delimiter_payload(
                CanonicalDelimiterKind::List,
                span,
                text,
                0,
                text.len() - 1,
            ));
            let inside = &text[1..text.len() - 1];
            let separators =
                retained_top_level_separator_tokens(inside, &offset_span(span, 1), ',');
            events.extend([
                payload_value(
                    CanonicalPayloadField::DelimiterSeparators,
                    CanonicalPayloadEventValue::Tokens(separators),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateEmpty,
                    CanonicalPayloadEventValue::Bool(child_count == 0),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateTrailing,
                    CanonicalPayloadEventValue::Bool(inside.trim_end().ends_with(',')),
                ),
                payload_value(
                    CanonicalPayloadField::ListElementEdges,
                    CanonicalPayloadEventValue::ChildOrdinals((0..child_count).collect()),
                ),
            ]);
        }
        CanonicalCommonNodeKind::RecordLiteral => {
            let open = text.find('{').expect("successful record open");
            if !text.ends_with('}') {
                return Vec::new();
            }
            let inside = &text[open + 1..text.len() - 1];
            events.extend(retained_delimiter_payload(
                CanonicalDelimiterKind::Record,
                span,
                text,
                open,
                text.len() - 1,
            ));
            let name = text[..open].trim();
            let name_tokens = if name.is_empty() {
                Vec::new()
            } else {
                vec![(
                    source_range_at(
                        span,
                        text[..open].find(name).unwrap_or_default(),
                        name.len(),
                    ),
                    name.to_string(),
                )]
            };
            let field_ranges = split_top_level_ranges_quoted(inside, ',');
            let mut fields = Vec::new();
            let mut colons = Vec::new();
            for range in field_ranges {
                let raw = &inside[range.clone()];
                if let Some(colon) = find_top_level_char(raw, ':') {
                    let field = raw[..colon].trim();
                    let field_start =
                        open + 1 + range.start + raw[..colon].find(field).unwrap_or_default();
                    fields.push((
                        source_range_at(span, field_start, field.len()),
                        field.to_string(),
                    ));
                    colons.push((
                        source_range_at(span, open + 1 + range.start + colon, 1),
                        ":".to_string(),
                    ));
                }
            }
            events.extend([
                payload_value(
                    CanonicalPayloadField::RecordNameToken,
                    CanonicalPayloadEventValue::Tokens(name_tokens),
                ),
                payload_value(
                    CanonicalPayloadField::RecordFieldTokens,
                    CanonicalPayloadEventValue::Tokens(fields),
                ),
                payload_value(
                    CanonicalPayloadField::RecordColonTokens,
                    CanonicalPayloadEventValue::Tokens(colons),
                ),
                payload_value(
                    CanonicalPayloadField::DelimiterSeparators,
                    CanonicalPayloadEventValue::Tokens(retained_top_level_separator_tokens(
                        inside,
                        &offset_span(span, open + 1),
                        ',',
                    )),
                ),
                payload_value(
                    CanonicalPayloadField::RecordValueEdges,
                    CanonicalPayloadEventValue::ChildOrdinals((0..child_count).collect()),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateEmpty,
                    CanonicalPayloadEventValue::Bool(child_count == 0),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateTrailing,
                    CanonicalPayloadEventValue::Bool(inside.trim_end().ends_with(',')),
                ),
            ]);
        }
        CanonicalCommonNodeKind::Call => {
            let open = find_top_level_open_paren(text).expect("successful call open");
            let inside = &text[open + 1..text.len() - 1];
            events.extend(retained_delimiter_payload(
                CanonicalDelimiterKind::Parenthesis,
                span,
                text,
                open,
                text.len() - 1,
            ));
            events.extend([
                payload_value(
                    CanonicalPayloadField::DelimiterSeparators,
                    CanonicalPayloadEventValue::Tokens(retained_top_level_separator_tokens(
                        inside,
                        &offset_span(span, open + 1),
                        ',',
                    )),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateEmpty,
                    CanonicalPayloadEventValue::Bool(child_count == 1),
                ),
                payload_value(
                    CanonicalPayloadField::AggregateTrailing,
                    CanonicalPayloadEventValue::Bool(inside.trim_end().ends_with(',')),
                ),
                payload_value(
                    CanonicalPayloadField::CallCalleeEdge,
                    CanonicalPayloadEventValue::ChildOrdinal(0),
                ),
                payload_value(
                    CanonicalPayloadField::CallArgumentEdges,
                    CanonicalPayloadEventValue::ChildOrdinals((1..child_count).collect()),
                ),
                payload_value(
                    CanonicalPayloadField::CallAdjacency,
                    CanonicalPayloadEventValue::Bools(retained_call_adjacency_facts(text, open)),
                ),
                payload_value(
                    CanonicalPayloadField::CallCloseState,
                    CanonicalPayloadEventValue::Bool(true),
                ),
                payload_value(
                    CanonicalPayloadField::CallTrailingState,
                    CanonicalPayloadEventValue::Bool(!inside.trim_end().ends_with(',')),
                ),
            ]);
        }
        CanonicalCommonNodeKind::Binary => {
            let (operator, start, end) =
                top_level_binary_operator(text).expect("successful binary operator");
            let operator_text = &text[start..end];
            let operator_tokens =
                canonical_lexical_events(operator_text, &offset_span(span, start))
                    .into_iter()
                    .map(|event| (event.range, event.spelling))
                    .collect();
            let (left_boundary, right_boundary) = binary_boundaries(text, start, end);
            events.extend([
                payload_value(
                    CanonicalPayloadField::BinaryOperator,
                    CanonicalPayloadEventValue::Operator(operator),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryOperatorTokens,
                    CanonicalPayloadEventValue::Tokens(operator_tokens),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryOperatorRange,
                    CanonicalPayloadEventValue::Range(source_range_at(span, start, end - start)),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryPrecedence,
                    CanonicalPayloadEventValue::Usize(operator_precedence(operator)),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryAssociativity,
                    CanonicalPayloadEventValue::Associativity(CanonicalAssociativity::Left),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryLeftBoundary,
                    CanonicalPayloadEventValue::Bool(left_boundary),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryRightBoundary,
                    CanonicalPayloadEventValue::Bool(right_boundary),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryReductionOrder,
                    CanonicalPayloadEventValue::ChildOrdinals(vec![0, 1]),
                ),
                payload_value(
                    CanonicalPayloadField::BinaryChildRoles,
                    CanonicalPayloadEventValue::ChildOrdinals(vec![0, 1]),
                ),
            ]);
        }
        CanonicalCommonNodeKind::Permission => {
            let (keyword, permission) = if keyword_rest(text, "borrow").is_some() {
                ("borrow", ParamPermission::Borrow)
            } else if keyword_rest(text, "change").is_some() {
                ("change", ParamPermission::Change)
            } else {
                ("consume", ParamPermission::Consume)
            };
            let value_start =
                text.len() - keyword_rest(text, keyword).expect("permission value").len();
            events.extend([
                payload_token(CanonicalPayloadField::PermissionKeyword, span, 0, keyword),
                payload_value(
                    CanonicalPayloadField::PermissionDiscriminant,
                    CanonicalPayloadEventValue::Permission(permission),
                ),
                payload_value(
                    CanonicalPayloadField::PermissionGap,
                    CanonicalPayloadEventValue::Range(source_range_at(
                        span,
                        keyword.len(),
                        value_start - keyword.len(),
                    )),
                ),
                payload_value(
                    CanonicalPayloadField::PermissionValueEdge,
                    CanonicalPayloadEventValue::ChildOrdinal(0),
                ),
            ]);
        }
        CanonicalCommonNodeKind::Try => {
            let rest = keyword_rest(text, "try").expect("try value");
            let value_start = text.len() - rest.len();
            let wrapper_start = find_top_level_phrase(rest, " or fail ");
            let wrapper_kind = if wrapper_start.is_some() {
                CanonicalTryWrapperKind::Wrap
            } else {
                CanonicalTryWrapperKind::Propagate
            };
            let mut relation_tokens = Vec::new();
            let mut root_token = Vec::new();
            let mut dot_token = Vec::new();
            let mut variant_token = Vec::new();
            if let Some(wrapper_start) = wrapper_start {
                let absolute = value_start + wrapper_start;
                relation_tokens.push((source_range_at(span, absolute + 1, 2), "or".to_string()));
                relation_tokens.push((source_range_at(span, absolute + 4, 4), "fail".to_string()));
                let failure_start = absolute + " or fail ".len();
                let failure = text[failure_start..].trim();
                let leading =
                    text[failure_start..].len() - text[failure_start..].trim_start().len();
                let absolute_failure = failure_start + leading;
                let (root, variant) = failure.split_once('.').unwrap_or((failure, ""));
                if !is_type_identifier(root) || !is_value_identifier(variant) {
                    return Vec::new();
                }
                if !root.is_empty() {
                    root_token.push((
                        source_range_at(span, absolute_failure, root.len()),
                        root.to_string(),
                    ));
                }
                if let Some(dot) = failure.find('.') {
                    dot_token.push((
                        source_range_at(span, absolute_failure + dot, 1),
                        ".".to_string(),
                    ));
                    if !variant.is_empty() {
                        variant_token.push((
                            source_range_at(span, absolute_failure + dot + 1, variant.len()),
                            variant.to_string(),
                        ));
                    }
                }
            }
            events.extend([
                payload_token(CanonicalPayloadField::TryKeyword, span, 0, "try"),
                payload_value(
                    CanonicalPayloadField::TryValueEdge,
                    CanonicalPayloadEventValue::ChildOrdinal(0),
                ),
                payload_value(
                    CanonicalPayloadField::TryWrapperRelation,
                    CanonicalPayloadEventValue::Tokens(relation_tokens),
                ),
                payload_value(
                    CanonicalPayloadField::TryFailureRootToken,
                    CanonicalPayloadEventValue::Tokens(root_token),
                ),
                payload_value(
                    CanonicalPayloadField::TryDotToken,
                    CanonicalPayloadEventValue::Tokens(dot_token),
                ),
                payload_value(
                    CanonicalPayloadField::TryFailureVariantToken,
                    CanonicalPayloadEventValue::Tokens(variant_token),
                ),
                payload_value(
                    CanonicalPayloadField::TryWrapperKind,
                    CanonicalPayloadEventValue::WrapperKind(wrapper_kind),
                ),
            ]);
        }
        CanonicalCommonNodeKind::Unsupported => {}
    }
    events
}

fn range_contains(parent: &ParsedSourceRange, child: &ParsedSourceRange) -> bool {
    parent.start.file == child.start.file
        && parent.start.line == child.start.line
        && parent.start.column <= child.start.column
        && child.start.column + child.byte_len <= parent.start.column + parent.byte_len
}

fn malformed_event_facts(
    occurrence: &CanonicalOccurrenceIdentity,
    nodes: &[CanonicalNodeEvidence],
) -> Vec<CanonicalMalformedSealFact> {
    nodes
        .iter()
        .enumerate()
        .filter_map(|(preorder, node)| match &node.completion {
            CanonicalCompletionEvent::Complete => None,
            CanonicalCompletionEvent::Unsupported(event) => {
                let node_identity = occurrence_node_identity(occurrence, preorder);
                let identity = CanonicalMalformedEventIdentity(source_owner_child_identity(
                    13,
                    &occurrence.0,
                    preorder,
                ));
                Some(vec![
                    CanonicalMalformedSealFact {
                        field: CanonicalMalformedSealField::Status,
                        value: CanonicalMalformedSealValue::Unsupported,
                    },
                    CanonicalMalformedSealFact {
                        field: CanonicalMalformedSealField::Node,
                        value: CanonicalMalformedSealValue::Node(node_identity),
                    },
                    CanonicalMalformedSealFact {
                        field: CanonicalMalformedSealField::Cause,
                        value: CanonicalMalformedSealValue::Cause(event.cause),
                    },
                    CanonicalMalformedSealFact {
                        field: CanonicalMalformedSealField::ProducingEvent,
                        value: CanonicalMalformedSealValue::ProducingEvent(
                            identity,
                            event.producing_event.clone(),
                        ),
                    },
                    CanonicalMalformedSealFact {
                        field: CanonicalMalformedSealField::OffendingRange,
                        value: CanonicalMalformedSealValue::Range(event.offending.clone()),
                    },
                    CanonicalMalformedSealFact {
                        field: CanonicalMalformedSealField::ConsumedRange,
                        value: CanonicalMalformedSealValue::Range(event.consumed.clone()),
                    },
                    CanonicalMalformedSealFact {
                        field: CanonicalMalformedSealField::ExpectedEvidence,
                        value: CanonicalMalformedSealValue::Expected(event.expected.clone()),
                    },
                    CanonicalMalformedSealFact {
                        field: CanonicalMalformedSealField::ActualEvidence,
                        value: CanonicalMalformedSealValue::Actual(event.actual.clone()),
                    },
                ])
            }
        })
        .flatten()
        .collect()
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
        occurrence: projected_occurrence.clone(),
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
        occurrence: authority_occurrence.clone(),
        role: authority_role(assignment.role),
        intent: authority_intent(assignment.intent),
        assignment_node: &assignment.expression_node_id,
        predicate_recognized: assignment.predicate_recognized,
        ordinal,
        nodes: &retained_nodes,
        tokens: &expression.canonical_tokens,
    });
    let payload_projection = payload_facts(
        &projected_occurrence,
        &projected_nodes,
        &expression.canonical_tokens,
    );
    let payload_authority = payload_facts(
        &authority_occurrence,
        &retained_nodes,
        &expression.canonical_tokens,
    );
    let malformed_projection = malformed_event_facts(&projected_occurrence, &projected_nodes);
    let malformed_authority = malformed_event_facts(&authority_occurrence, &retained_nodes);
    CanonicalOccurrenceSeal {
        projection,
        authority,
        payload_projection,
        payload_authority,
        malformed_projection,
        malformed_authority,
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

fn retained_parser_block_relationship(text: &str, span: &Span) -> ParsedBlockRelationship {
    let events = canonical_lexical_events(text, span);
    if events.len() == 1 && events[0].spelling == "}" {
        ParsedBlockRelationship::Closes
    } else if events.last().is_some_and(|event| event.spelling == "{") {
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
    let block = StatementBlockContext {
        relationship: block_relationship,
        depth_before: block_depth_before,
        depth_after: block_depth_after,
    };
    let (core_kind, core_status, core_expression_kind, core_reason) =
        parser_core_shape(line.text.trim(), &kind);
    let canonical_extra_occurrences =
        parse_statement_relationship_occurrences(line.text.trim(), &line.span, &source_node_id);
    let canonical_assignments =
        retained_statement_assignments(line.text.trim(), &kind, &canonical_extra_occurrences);
    let canonical_statement_projection = projected_statement_events(
        line.text.trim(),
        &line.span,
        &source_node_id,
        &kind,
        &canonical_extra_occurrences,
        block,
    );
    let canonical_statement_authority = retained_statement_events(
        line.text.trim(),
        &line.span,
        &source_node_id,
        &kind,
        &canonical_extra_occurrences,
        &canonical_assignments,
    );
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
        canonical_statement_projection,
        canonical_statement_authority,
    }
}

fn statement_fact(
    field: CanonicalStatementEventField,
    value: CanonicalStatementEventValue,
) -> CanonicalStatementEventFact {
    CanonicalStatementEventFact { field, value }
}

fn contract_statement_events(
    section_name: &str,
    section_span: &Span,
    line: &SectionLine,
    expression: &ParsedExpression,
    assignment: &CanonicalOccurrenceAssignmentEvent,
) -> (
    Vec<CanonicalStatementEventFact>,
    Vec<CanonicalStatementEventFact>,
) {
    let kind = if section_name == "needs" {
        CanonicalStatementKindEvent::NeedsPredicate
    } else {
        CanonicalStatementKindEvent::EnsuresPredicate
    };
    let projected_root = expression.canonical.node_id.clone();
    let authority_root = assignment.expression_node_id.clone();
    let common_prefix = |root: ParserSyntaxNodeId,
                         keyword: CanonicalStatementEventValue,
                         relationship: CanonicalStatementEventValue| {
        vec![
            statement_fact(
                CanonicalStatementEventField::Kind,
                CanonicalStatementEventValue::Kind(kind),
            ),
            statement_fact(
                CanonicalStatementEventField::Line,
                CanonicalStatementEventValue::Range(ParsedSourceRange {
                    start: line.span.clone(),
                    byte_len: line.text.len(),
                }),
            ),
            statement_fact(
                CanonicalStatementEventField::Statement,
                CanonicalStatementEventValue::Text(expression.canonical.node_id.as_str().into()),
            ),
            statement_fact(CanonicalStatementEventField::Keyword, keyword),
            statement_fact(
                CanonicalStatementEventField::RelationshipToken,
                relationship,
            ),
            statement_fact(
                CanonicalStatementEventField::ValueRoot,
                CanonicalStatementEventValue::Root {
                    ordinal: 0,
                    node: root.clone(),
                },
            ),
            statement_fact(
                CanonicalStatementEventField::OrderedRoots,
                CanonicalStatementEventValue::Roots(vec![(0, root)]),
            ),
            statement_fact(
                CanonicalStatementEventField::BlockDepthBefore,
                CanonicalStatementEventValue::Usize(0),
            ),
            statement_fact(
                CanonicalStatementEventField::BlockDepthAfter,
                CanonicalStatementEventValue::Usize(0),
            ),
            statement_fact(
                CanonicalStatementEventField::BlockRelationship,
                CanonicalStatementEventValue::BlockRelationship(ParsedBlockRelationship::None),
            ),
        ]
    };
    let projection = common_prefix(
        projected_root,
        CanonicalStatementEventValue::Token {
            slot: 0,
            range: source_range_at(section_span, 0, section_name.len()),
            spelling: section_name.to_string(),
        },
        CanonicalStatementEventValue::Token {
            slot: 1,
            range: source_range_at(section_span, section_name.len(), 1),
            spelling: ":".to_string(),
        },
    );
    let header = format!("{section_name}:");
    let retained_tokens = canonical_lexical_events(&header, section_span);
    let retained_keyword = retained_tokens
        .first()
        .expect("contract section keyword token must exist");
    let retained_colon = retained_tokens
        .get(1)
        .expect("contract section colon token must exist");
    let authority = common_prefix(
        authority_root,
        CanonicalStatementEventValue::Token {
            slot: 0,
            range: retained_keyword.range.clone(),
            spelling: retained_keyword.spelling.clone(),
        },
        CanonicalStatementEventValue::Token {
            slot: 1,
            range: retained_colon.range.clone(),
            spelling: retained_colon.spelling.clone(),
        },
    );
    (projection, authority)
}

fn statement_token_fact(
    field: CanonicalStatementEventField,
    slot: usize,
    span: &Span,
    start: usize,
    spelling: &str,
) -> CanonicalStatementEventFact {
    statement_fact(
        field,
        CanonicalStatementEventValue::Token {
            slot,
            range: source_range_at(span, start, spelling.len()),
            spelling: spelling.to_string(),
        },
    )
}

fn statement_roots_from_projection(
    kind: &ParsedBodyStatementKind,
    extra: &[ParsedExpression],
) -> Vec<ParserSyntaxNodeId> {
    if !extra.is_empty() {
        return extra
            .iter()
            .map(|expression| expression.canonical.node_id.clone())
            .collect();
    }
    match kind {
        ParsedBodyStatementKind::Return(expression) => vec![expression.canonical.node_id.clone()],
        ParsedBodyStatementKind::Binding {
            value: Some(expression),
            ..
        } => vec![expression.canonical.node_id.clone()],
        ParsedBodyStatementKind::Binding { value: None, .. } => Vec::new(),
        ParsedBodyStatementKind::Other { expressions } => expressions
            .iter()
            .map(|expression| expression.canonical.node_id.clone())
            .collect(),
    }
}

fn projected_statement_kind(
    text: &str,
    kind: &ParsedBodyStatementKind,
) -> CanonicalStatementKindEvent {
    match kind {
        ParsedBodyStatementKind::Return(_) => CanonicalStatementKindEvent::Return,
        ParsedBodyStatementKind::Binding { mutable: false, .. } => {
            CanonicalStatementKindEvent::ImmutableBinding
        }
        ParsedBodyStatementKind::Binding { mutable: true, .. } => {
            CanonicalStatementKindEvent::MutableBinding
        }
        ParsedBodyStatementKind::Other { .. } => {
            if projected_keyword(text, "set") {
                CanonicalStatementKindEvent::Set
            } else if projected_keyword(text, "save") {
                CanonicalStatementKindEvent::Save
            } else if projected_keyword(text, "fail") {
                CanonicalStatementKindEvent::Fail
            } else if projected_keyword(text, "expect") {
                CanonicalStatementKindEvent::Expect
            } else if projected_keyword(text, "if") {
                CanonicalStatementKindEvent::If
            } else if projected_keyword(text, "while") {
                CanonicalStatementKindEvent::While
            } else if projected_keyword(text, "for each") {
                CanonicalStatementKindEvent::ForEach
            } else if projected_keyword(text, "for index")
                && find_top_level_phrase(text, " through ").is_some()
            {
                CanonicalStatementKindEvent::ForIndexThrough
            } else if projected_keyword(text, "for index") {
                CanonicalStatementKindEvent::ForIndexUntil
            } else if text == "loop {" {
                CanonicalStatementKindEvent::UnconditionalLoop
            } else if text == "}" {
                CanonicalStatementKindEvent::BlockClose
            } else {
                CanonicalStatementKindEvent::FreeExpression
            }
        }
    }
}

fn retained_statement_kind(text: &str) -> CanonicalStatementKindEvent {
    if keyword_rest(text, "return").is_some() || text == "return" {
        CanonicalStatementKindEvent::Return
    } else if keyword_rest(text, "let").is_some() {
        CanonicalStatementKindEvent::ImmutableBinding
    } else if keyword_rest(text, "change").is_some() {
        CanonicalStatementKindEvent::MutableBinding
    } else if keyword_rest(text, "set").is_some() {
        CanonicalStatementKindEvent::Set
    } else if keyword_rest(text, "save").is_some() {
        CanonicalStatementKindEvent::Save
    } else if keyword_rest(text, "fail").is_some() {
        CanonicalStatementKindEvent::Fail
    } else if keyword_rest(text, "expect").is_some() {
        CanonicalStatementKindEvent::Expect
    } else if keyword_rest(text, "if").is_some() {
        CanonicalStatementKindEvent::If
    } else if keyword_rest(text, "while").is_some() {
        CanonicalStatementKindEvent::While
    } else if keyword_rest(text, "for each").is_some() {
        CanonicalStatementKindEvent::ForEach
    } else if keyword_rest(text, "for index").is_some()
        && find_top_level_phrase(text, " through ").is_some()
    {
        CanonicalStatementKindEvent::ForIndexThrough
    } else if keyword_rest(text, "for index").is_some() {
        CanonicalStatementKindEvent::ForIndexUntil
    } else if text == "loop {" {
        CanonicalStatementKindEvent::UnconditionalLoop
    } else if text == "}" {
        CanonicalStatementKindEvent::BlockClose
    } else {
        CanonicalStatementKindEvent::FreeExpression
    }
}

fn append_projected_form_facts(
    facts: &mut Vec<CanonicalStatementEventFact>,
    form: CanonicalStatementKindEvent,
    text: &str,
    span: &Span,
    roots: &[ParserSyntaxNodeId],
) {
    let keyword = match form {
        CanonicalStatementKindEvent::Return => Some("return"),
        CanonicalStatementKindEvent::ImmutableBinding => Some("let"),
        CanonicalStatementKindEvent::MutableBinding => Some("change"),
        CanonicalStatementKindEvent::Set => Some("set"),
        CanonicalStatementKindEvent::Save => Some("save"),
        CanonicalStatementKindEvent::Fail => Some("fail"),
        CanonicalStatementKindEvent::Expect => Some("expect"),
        CanonicalStatementKindEvent::If => Some("if"),
        CanonicalStatementKindEvent::While => Some("while"),
        CanonicalStatementKindEvent::ForEach
        | CanonicalStatementKindEvent::ForIndexUntil
        | CanonicalStatementKindEvent::ForIndexThrough => Some("for"),
        CanonicalStatementKindEvent::UnconditionalLoop => Some("loop"),
        CanonicalStatementKindEvent::BlockClose => Some("}"),
        CanonicalStatementKindEvent::NeedsPredicate
        | CanonicalStatementKindEvent::EnsuresPredicate
        | CanonicalStatementKindEvent::FreeExpression => None,
    };
    if let Some(keyword) = keyword {
        let start = text.find(keyword).unwrap_or_default();
        facts.push(statement_token_fact(
            CanonicalStatementEventField::Keyword,
            0,
            span,
            start,
            keyword,
        ));
    }
    let root = |ordinal: usize| {
        roots
            .get(ordinal)
            .cloned()
            .map(|node| CanonicalStatementEventValue::Root { ordinal, node })
    };
    match form {
        CanonicalStatementKindEvent::Return
        | CanonicalStatementKindEvent::Fail
        | CanonicalStatementKindEvent::Expect
        | CanonicalStatementKindEvent::FreeExpression
        | CanonicalStatementKindEvent::If
        | CanonicalStatementKindEvent::While => {
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::ValueRoot,
                    value,
                ));
            }
        }
        CanonicalStatementKindEvent::ImmutableBinding
        | CanonicalStatementKindEvent::MutableBinding => {
            let keyword = if form == CanonicalStatementKindEvent::ImmutableBinding {
                "let"
            } else {
                "change"
            };
            let rest = text[keyword.len()..].trim_start();
            let rest_start = text.len() - rest.len();
            let binder = rest
                .split(|ch: char| ch == ':' || ch == '=' || ch.is_whitespace())
                .next()
                .unwrap_or_default();
            facts.push(statement_token_fact(
                CanonicalStatementEventField::Binder,
                1,
                span,
                rest_start,
                binder,
            ));
            facts.push(statement_fact(
                CanonicalStatementEventField::BinderRelationship,
                CanonicalStatementEventValue::TokenReference(1),
            ));
            if let Some(colon) = find_top_level_char(text, ':') {
                facts.push(statement_token_fact(
                    CanonicalStatementEventField::TypeBoundary,
                    2,
                    span,
                    colon,
                    ":",
                ));
            }
            if let Some(equals) = find_top_level_char(text, '=') {
                facts.push(statement_token_fact(
                    CanonicalStatementEventField::AssignmentToken,
                    3,
                    span,
                    equals,
                    "=",
                ));
            }
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::ValueRoot,
                    value,
                ));
            }
        }
        CanonicalStatementKindEvent::Set => {
            if let Some(equals) = find_top_level_char(text, '=') {
                facts.push(statement_token_fact(
                    CanonicalStatementEventField::AssignmentToken,
                    1,
                    span,
                    equals,
                    "=",
                ));
            }
            if let Some(value) = root(1) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::TargetRoot,
                    value,
                ));
            }
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::ValueRoot,
                    value,
                ));
            }
        }
        CanonicalStatementKindEvent::Save => {
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::ValueRoot,
                    value,
                ));
            }
            if let Some(at) = find_top_level_phrase(text, " in ") {
                facts.push(statement_token_fact(
                    CanonicalStatementEventField::RelationshipToken,
                    1,
                    span,
                    at + 1,
                    "in",
                ));
                let destination = text[at + 4..].trim();
                let start = text.len() - destination.len();
                facts.push(statement_token_fact(
                    CanonicalStatementEventField::DestinationToken,
                    2,
                    span,
                    start,
                    destination,
                ));
            }
        }
        CanonicalStatementKindEvent::ForEach => {
            facts.push(statement_fact(
                CanonicalStatementEventField::PhraseTokens,
                CanonicalStatementEventValue::Tokens(vec![
                    (0, source_range_at(span, 0, 3), "for".to_string()),
                    (1, source_range_at(span, 4, 4), "each".to_string()),
                ]),
            ));
            if let Some(at) = find_top_level_phrase(text, " in ") {
                let binder = text["for each".len()..at].trim();
                let binder_start = text[..at].rfind(binder).unwrap_or("for each".len());
                facts.push(statement_token_fact(
                    CanonicalStatementEventField::Binder,
                    2,
                    span,
                    binder_start,
                    binder,
                ));
                facts.push(statement_fact(
                    CanonicalStatementEventField::BinderRelationship,
                    CanonicalStatementEventValue::TokenReference(2),
                ));
                facts.push(statement_token_fact(
                    CanonicalStatementEventField::RelationshipToken,
                    3,
                    span,
                    at + 1,
                    "in",
                ));
            }
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::ValueRoot,
                    value,
                ));
            }
        }
        CanonicalStatementKindEvent::ForIndexUntil
        | CanonicalStatementKindEvent::ForIndexThrough => {
            facts.push(statement_fact(
                CanonicalStatementEventField::PhraseTokens,
                CanonicalStatementEventValue::Tokens(vec![
                    (0, source_range_at(span, 0, 3), "for".to_string()),
                    (1, source_range_at(span, 4, 5), "index".to_string()),
                ]),
            ));
            if let Some(from) = find_top_level_phrase(text, " from ") {
                let binder = text["for index".len()..from].trim();
                let binder_start = text[..from].rfind(binder).unwrap_or("for index".len());
                facts.push(statement_token_fact(
                    CanonicalStatementEventField::Binder,
                    2,
                    span,
                    binder_start,
                    binder,
                ));
                facts.push(statement_fact(
                    CanonicalStatementEventField::BinderRelationship,
                    CanonicalStatementEventValue::TokenReference(2),
                ));
                facts.push(statement_token_fact(
                    CanonicalStatementEventField::RelationshipToken,
                    3,
                    span,
                    from + 1,
                    "from",
                ));
            }
            let bound_spelling = if form == CanonicalStatementKindEvent::ForIndexThrough {
                "through"
            } else {
                "until"
            };
            if let Some(at) = find_top_level_phrase(text, &format!(" {bound_spelling} ")) {
                facts.push(statement_token_fact(
                    CanonicalStatementEventField::RelationshipToken,
                    4,
                    span,
                    at + 1,
                    bound_spelling,
                ));
            }
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::StartRoot,
                    value,
                ));
            }
            if let Some(value) = root(1) {
                facts.push(statement_fact(CanonicalStatementEventField::EndRoot, value));
            }
        }
        CanonicalStatementKindEvent::UnconditionalLoop
        | CanonicalStatementKindEvent::BlockClose => {}
        CanonicalStatementKindEvent::NeedsPredicate
        | CanonicalStatementKindEvent::EnsuresPredicate => unreachable!(),
    }
}

#[derive(Clone, Copy)]
struct StatementBlockContext {
    relationship: ParsedBlockRelationship,
    depth_before: usize,
    depth_after: usize,
}

fn projected_statement_events(
    text: &str,
    span: &Span,
    source_node_id: &ParserSyntaxNodeId,
    kind: &ParsedBodyStatementKind,
    extra: &[ParsedExpression],
    block: StatementBlockContext,
) -> Vec<CanonicalStatementEventFact> {
    let form = projected_statement_kind(text, kind);
    let roots = statement_roots_from_projection(kind, extra);
    let mut facts = vec![
        statement_fact(
            CanonicalStatementEventField::Kind,
            CanonicalStatementEventValue::Kind(form),
        ),
        statement_fact(
            CanonicalStatementEventField::Line,
            CanonicalStatementEventValue::Range(source_range_at(span, 0, text.len())),
        ),
        statement_fact(
            CanonicalStatementEventField::Statement,
            CanonicalStatementEventValue::Text(source_node_id.as_str().to_string()),
        ),
    ];
    append_projected_form_facts(&mut facts, form, text, span, &roots);
    facts.push(statement_fact(
        CanonicalStatementEventField::OrderedRoots,
        CanonicalStatementEventValue::Roots(roots.iter().cloned().enumerate().collect()),
    ));
    facts.push(statement_fact(
        CanonicalStatementEventField::BlockDepthBefore,
        CanonicalStatementEventValue::Usize(block.depth_before),
    ));
    facts.push(statement_fact(
        CanonicalStatementEventField::BlockDepthAfter,
        CanonicalStatementEventValue::Usize(block.depth_after),
    ));
    facts.push(statement_fact(
        CanonicalStatementEventField::BlockRelationship,
        CanonicalStatementEventValue::BlockRelationship(block.relationship),
    ));
    if block.relationship == ParsedBlockRelationship::Opens {
        let open = text.rfind('{').unwrap_or(text.len().saturating_sub(1));
        facts.push(statement_token_fact(
            CanonicalStatementEventField::BlockOpenToken,
            31,
            span,
            open,
            "{",
        ));
        facts.push(statement_fact(
            CanonicalStatementEventField::BlockOwner,
            CanonicalStatementEventValue::Bool(true),
        ));
    } else if block.relationship == ParsedBlockRelationship::Closes {
        facts.push(statement_token_fact(
            CanonicalStatementEventField::BlockCloseToken,
            31,
            span,
            0,
            "}",
        ));
        facts.push(statement_fact(
            CanonicalStatementEventField::BlockOwner,
            CanonicalStatementEventValue::Bool(true),
        ));
    } else if block.depth_before > 0 {
        facts.push(statement_fact(
            CanonicalStatementEventField::BlockOwner,
            CanonicalStatementEventValue::Bool(true),
        ));
    }
    if roots.is_empty() {
        facts.push(statement_fact(
            CanonicalStatementEventField::ExpressionAbsent,
            CanonicalStatementEventValue::Bool(true),
        ));
    }
    facts
}

fn retained_token_value(
    events: &[CanonicalLexicalTokenEvent],
    slot: usize,
    spelling: &str,
    skip: usize,
) -> CanonicalStatementEventValue {
    let event = events
        .iter()
        .filter(|event| event.spelling == spelling)
        .nth(skip)
        .unwrap_or_else(|| panic!("retained statement token `{spelling}` must exist"));
    CanonicalStatementEventValue::Token {
        slot,
        range: event.range.clone(),
        spelling: event.spelling.clone(),
    }
}

fn retained_token_fact(
    facts: &mut Vec<CanonicalStatementEventFact>,
    field: CanonicalStatementEventField,
    events: &[CanonicalLexicalTokenEvent],
    slot: usize,
    spelling: &str,
    skip: usize,
) {
    facts.push(statement_fact(
        field,
        retained_token_value(events, slot, spelling, skip),
    ));
}

fn append_retained_form_facts(
    facts: &mut Vec<CanonicalStatementEventFact>,
    form: CanonicalStatementKindEvent,
    text: &str,
    span: &Span,
    roots: &[ParserSyntaxNodeId],
) {
    let events = canonical_lexical_events(text, span);
    let keyword = match form {
        CanonicalStatementKindEvent::Return => Some("return"),
        CanonicalStatementKindEvent::ImmutableBinding => Some("let"),
        CanonicalStatementKindEvent::MutableBinding => Some("change"),
        CanonicalStatementKindEvent::Set => Some("set"),
        CanonicalStatementKindEvent::Save => Some("save"),
        CanonicalStatementKindEvent::Fail => Some("fail"),
        CanonicalStatementKindEvent::Expect => Some("expect"),
        CanonicalStatementKindEvent::If => Some("if"),
        CanonicalStatementKindEvent::While => Some("while"),
        CanonicalStatementKindEvent::ForEach
        | CanonicalStatementKindEvent::ForIndexUntil
        | CanonicalStatementKindEvent::ForIndexThrough => Some("for"),
        CanonicalStatementKindEvent::UnconditionalLoop => Some("loop"),
        CanonicalStatementKindEvent::BlockClose => Some("}"),
        CanonicalStatementKindEvent::NeedsPredicate
        | CanonicalStatementKindEvent::EnsuresPredicate
        | CanonicalStatementKindEvent::FreeExpression => None,
    };
    if let Some(keyword) = keyword {
        retained_token_fact(
            facts,
            CanonicalStatementEventField::Keyword,
            &events,
            0,
            keyword,
            0,
        );
    }
    let root = |ordinal: usize| {
        roots
            .get(ordinal)
            .cloned()
            .map(|node| CanonicalStatementEventValue::Root { ordinal, node })
    };
    match form {
        CanonicalStatementKindEvent::Return
        | CanonicalStatementKindEvent::Fail
        | CanonicalStatementKindEvent::Expect
        | CanonicalStatementKindEvent::FreeExpression
        | CanonicalStatementKindEvent::If
        | CanonicalStatementKindEvent::While => {
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::ValueRoot,
                    value,
                ));
            }
        }
        CanonicalStatementKindEvent::ImmutableBinding
        | CanonicalStatementKindEvent::MutableBinding => {
            let binder = events
                .get(1)
                .expect("binding retained binder event must exist");
            facts.push(statement_fact(
                CanonicalStatementEventField::Binder,
                CanonicalStatementEventValue::Token {
                    slot: 1,
                    range: binder.range.clone(),
                    spelling: binder.spelling.clone(),
                },
            ));
            facts.push(statement_fact(
                CanonicalStatementEventField::BinderRelationship,
                CanonicalStatementEventValue::TokenReference(1),
            ));
            if events.iter().any(|event| event.spelling == ":") {
                retained_token_fact(
                    facts,
                    CanonicalStatementEventField::TypeBoundary,
                    &events,
                    2,
                    ":",
                    0,
                );
            }
            if events.iter().any(|event| event.spelling == "=") {
                retained_token_fact(
                    facts,
                    CanonicalStatementEventField::AssignmentToken,
                    &events,
                    3,
                    "=",
                    0,
                );
            }
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::ValueRoot,
                    value,
                ));
            }
        }
        CanonicalStatementKindEvent::Set => {
            if events.iter().any(|event| event.spelling == "=") {
                retained_token_fact(
                    facts,
                    CanonicalStatementEventField::AssignmentToken,
                    &events,
                    1,
                    "=",
                    0,
                );
            }
            if let Some(value) = root(1) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::TargetRoot,
                    value,
                ));
            }
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::ValueRoot,
                    value,
                ));
            }
        }
        CanonicalStatementKindEvent::Save => {
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::ValueRoot,
                    value,
                ));
            }
            if let Some((in_index, relationship)) = events
                .iter()
                .enumerate()
                .find(|(_, event)| event.spelling == "in")
            {
                facts.push(statement_fact(
                    CanonicalStatementEventField::RelationshipToken,
                    CanonicalStatementEventValue::Token {
                        slot: 1,
                        range: relationship.range.clone(),
                        spelling: relationship.spelling.clone(),
                    },
                ));
                if let Some(destination) = events.get(in_index + 1) {
                    facts.push(statement_fact(
                        CanonicalStatementEventField::DestinationToken,
                        CanonicalStatementEventValue::Token {
                            slot: 2,
                            range: destination.range.clone(),
                            spelling: destination.spelling.clone(),
                        },
                    ));
                }
            }
        }
        CanonicalStatementKindEvent::ForEach => {
            let phrase = events
                .iter()
                .take(2)
                .enumerate()
                .map(|(slot, event)| (slot, event.range.clone(), event.spelling.clone()))
                .collect();
            facts.push(statement_fact(
                CanonicalStatementEventField::PhraseTokens,
                CanonicalStatementEventValue::Tokens(phrase),
            ));
            let binder = events.get(2).expect("for-each retained binder must exist");
            facts.push(statement_fact(
                CanonicalStatementEventField::Binder,
                CanonicalStatementEventValue::Token {
                    slot: 2,
                    range: binder.range.clone(),
                    spelling: binder.spelling.clone(),
                },
            ));
            facts.push(statement_fact(
                CanonicalStatementEventField::BinderRelationship,
                CanonicalStatementEventValue::TokenReference(2),
            ));
            retained_token_fact(
                facts,
                CanonicalStatementEventField::RelationshipToken,
                &events,
                3,
                "in",
                0,
            );
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::ValueRoot,
                    value,
                ));
            }
        }
        CanonicalStatementKindEvent::ForIndexUntil
        | CanonicalStatementKindEvent::ForIndexThrough => {
            let phrase = events
                .iter()
                .take(2)
                .enumerate()
                .map(|(slot, event)| (slot, event.range.clone(), event.spelling.clone()))
                .collect();
            facts.push(statement_fact(
                CanonicalStatementEventField::PhraseTokens,
                CanonicalStatementEventValue::Tokens(phrase),
            ));
            let binder = events.get(2).expect("for-index retained binder must exist");
            facts.push(statement_fact(
                CanonicalStatementEventField::Binder,
                CanonicalStatementEventValue::Token {
                    slot: 2,
                    range: binder.range.clone(),
                    spelling: binder.spelling.clone(),
                },
            ));
            facts.push(statement_fact(
                CanonicalStatementEventField::BinderRelationship,
                CanonicalStatementEventValue::TokenReference(2),
            ));
            retained_token_fact(
                facts,
                CanonicalStatementEventField::RelationshipToken,
                &events,
                3,
                "from",
                0,
            );
            let bound = if form == CanonicalStatementKindEvent::ForIndexThrough {
                "through"
            } else {
                "until"
            };
            retained_token_fact(
                facts,
                CanonicalStatementEventField::RelationshipToken,
                &events,
                4,
                bound,
                0,
            );
            if let Some(value) = root(0) {
                facts.push(statement_fact(
                    CanonicalStatementEventField::StartRoot,
                    value,
                ));
            }
            if let Some(value) = root(1) {
                facts.push(statement_fact(CanonicalStatementEventField::EndRoot, value));
            }
        }
        CanonicalStatementKindEvent::UnconditionalLoop
        | CanonicalStatementKindEvent::BlockClose => {}
        CanonicalStatementKindEvent::NeedsPredicate
        | CanonicalStatementKindEvent::EnsuresPredicate => unreachable!(),
    }
}

fn retained_statement_events(
    text: &str,
    span: &Span,
    source_node_id: &ParserSyntaxNodeId,
    _kind: &ParsedBodyStatementKind,
    _extra: &[ParsedExpression],
    assignments: &[CanonicalOccurrenceAssignmentEvent],
) -> Vec<CanonicalStatementEventFact> {
    let form = retained_statement_kind(text);
    let roots = assignments
        .iter()
        .map(|assignment| assignment.expression_node_id.clone())
        .collect::<Vec<_>>();
    let mut facts = vec![
        statement_fact(
            CanonicalStatementEventField::Kind,
            CanonicalStatementEventValue::Kind(form),
        ),
        statement_fact(
            CanonicalStatementEventField::Line,
            CanonicalStatementEventValue::Range(ParsedSourceRange {
                start: span.clone(),
                byte_len: text.len(),
            }),
        ),
        statement_fact(
            CanonicalStatementEventField::Statement,
            CanonicalStatementEventValue::Text(source_node_id.as_str().to_string()),
        ),
    ];
    // The retained channel starts from the parser's token stream and assignment events,
    // independently of the projected statement facts.
    append_retained_form_facts(&mut facts, form, text, span, &roots);
    facts.push(statement_fact(
        CanonicalStatementEventField::OrderedRoots,
        CanonicalStatementEventValue::Roots(roots.iter().cloned().enumerate().collect()),
    ));
    if roots.is_empty() {
        facts.push(statement_fact(
            CanonicalStatementEventField::ExpressionAbsent,
            CanonicalStatementEventValue::Bool(true),
        ));
    }
    facts
}

fn append_retained_block_events(
    facts: &mut Vec<CanonicalStatementEventFact>,
    text: &str,
    span: &Span,
    block: StatementBlockContext,
) {
    facts.extend([
        statement_fact(
            CanonicalStatementEventField::BlockDepthBefore,
            CanonicalStatementEventValue::Usize(block.depth_before),
        ),
        statement_fact(
            CanonicalStatementEventField::BlockDepthAfter,
            CanonicalStatementEventValue::Usize(block.depth_after),
        ),
        statement_fact(
            CanonicalStatementEventField::BlockRelationship,
            CanonicalStatementEventValue::BlockRelationship(block.relationship),
        ),
    ]);
    let lexical_events = canonical_lexical_events(text, span);
    match block.relationship {
        ParsedBlockRelationship::Opens => {
            let open = lexical_events
                .iter()
                .rfind(|token| token.spelling == "{")
                .expect("retained block opener token");
            facts.push(statement_fact(
                CanonicalStatementEventField::BlockOpenToken,
                CanonicalStatementEventValue::Token {
                    slot: 31,
                    range: open.range.clone(),
                    spelling: open.spelling.clone(),
                },
            ));
            facts.push(statement_fact(
                CanonicalStatementEventField::BlockOwner,
                CanonicalStatementEventValue::Bool(true),
            ));
        }
        ParsedBlockRelationship::Closes => {
            let close = lexical_events
                .iter()
                .find(|token| token.spelling == "}")
                .expect("retained block close token");
            facts.push(statement_fact(
                CanonicalStatementEventField::BlockCloseToken,
                CanonicalStatementEventValue::Token {
                    slot: 31,
                    range: close.range.clone(),
                    spelling: close.spelling.clone(),
                },
            ));
            facts.push(statement_fact(
                CanonicalStatementEventField::BlockOwner,
                CanonicalStatementEventValue::Bool(true),
            ));
        }
        ParsedBlockRelationship::None if block.depth_before > 0 => {
            facts.push(statement_fact(
                CanonicalStatementEventField::BlockOwner,
                CanonicalStatementEventValue::Bool(true),
            ));
        }
        ParsedBlockRelationship::None => {}
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
    if keyword_rest(text, "set").is_some() && index == 0 {
        (
            CanonicalExpressionRoleEvent::SetValue,
            CanonicalExpressionIntentEvent::SetValue,
            false,
        )
    } else if keyword_rest(text, "set").is_some() {
        (
            CanonicalExpressionRoleEvent::Other,
            CanonicalExpressionIntentEvent::Other,
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
        CanonicalExpressionKind::ElementPlace { base, .. } => {
            validate_child(base, "element-base", identities)?;
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
        CanonicalExpressionKind::Try { value, .. } => {
            validate_child(value, "try-value", identities)?;
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
    } else if text != "}" && text != "loop {" && !text.ends_with(':') {
        Some((text, 0))
    } else {
        None
    };
    candidate
        .filter(|(expression, _)| !expression.trim().is_empty())
        .map(|(expression, offset)| {
            let mut parsed = parse_expression_syntax(
                expression,
                offset_span(span, offset),
                source_node_id.child("expression-0"),
            );
            if keyword_rest(text, "if").is_some() || keyword_rest(text, "while").is_some() {
                apply_predicate_completion(&mut parsed, expression, &offset_span(span, offset));
            }
            vec![parsed]
        })
        .unwrap_or_default()
}

fn parse_statement_relationship_occurrences(
    text: &str,
    span: &Span,
    source_node_id: &ParserSyntaxNodeId,
) -> Vec<ParsedExpression> {
    if let Some(rest) = keyword_rest(text, "set") {
        let rest_offset = text.len() - rest.len();
        if let Some(equals) = find_top_level_char(rest, '=') {
            let target_raw = &rest[..equals];
            let value_raw = &rest[equals + 1..];
            let target = target_raw.trim();
            let value = value_raw.trim();
            if !target.is_empty() && !value.is_empty() {
                let target_leading = target_raw.len() - target_raw.trim_start().len();
                let value_leading = value_raw.len() - value_raw.trim_start().len();
                return vec![
                    parse_expression_syntax(
                        value,
                        offset_span(span, rest_offset + equals + 1 + value_leading),
                        source_node_id.child("canonical-occurrence-expression-0"),
                    ),
                    parse_expression_syntax(
                        target,
                        offset_span(span, rest_offset + target_leading),
                        source_node_id.child("canonical-occurrence-expression-1"),
                    ),
                ];
            }
        }
    }
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
            events.push(CanonicalLexicalTokenEvent {
                range: ParsedSourceRange {
                    start: offset_span(&span, start),
                    byte_len: 1,
                },
                spelling: "\"".to_string(),
            });
            index += 1;
            while index < text.len() {
                let next = text[index..]
                    .chars()
                    .next()
                    .expect("text token cursor must remain on a character boundary");
                if next == '\\' {
                    let escape_start = index;
                    index += 1;
                    if index < text.len() {
                        let escaped = text[index..]
                            .chars()
                            .next()
                            .expect("escape cursor must remain on a character boundary");
                        index += escaped.len_utf8();
                    }
                    events.push(CanonicalLexicalTokenEvent {
                        range: ParsedSourceRange {
                            start: offset_span(&span, escape_start),
                            byte_len: index - escape_start,
                        },
                        spelling: text[escape_start..index].to_string(),
                    });
                } else if next == '"' {
                    events.push(CanonicalLexicalTokenEvent {
                        range: ParsedSourceRange {
                            start: offset_span(&span, index),
                            byte_len: 1,
                        },
                        spelling: "\"".to_string(),
                    });
                    index += 1;
                    break;
                } else {
                    index += next.len_utf8();
                }
            }
            continue;
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
            spelling: text[start..index].to_string(),
        });
    }
    events
}

const CANONICAL_MAX_DELIMITER_DEPTH: usize = 16;

fn completion_range(span: &Span, start: usize, byte_len: usize) -> ParsedSourceRange {
    ParsedSourceRange {
        start: offset_span(span, start),
        byte_len,
    }
}

fn completion_token_kind(ch: char) -> CanonicalLexicalTokenKind {
    match ch {
        '"' => CanonicalLexicalTokenKind::TextQuote,
        '(' => CanonicalLexicalTokenKind::ParenthesisOpen,
        ')' => CanonicalLexicalTokenKind::ParenthesisClose,
        '[' => CanonicalLexicalTokenKind::ListOpen,
        ']' => CanonicalLexicalTokenKind::ListClose,
        '{' => CanonicalLexicalTokenKind::RecordOpen,
        '}' => CanonicalLexicalTokenKind::RecordClose,
        ',' => CanonicalLexicalTokenKind::Comma,
        '.' => CanonicalLexicalTokenKind::Dot,
        '<' | '>' | '=' | '!' => CanonicalLexicalTokenKind::ComparisonOperator,
        value if value.is_ascii_digit() => CanonicalLexicalTokenKind::IntegerLiteral,
        value if value.is_ascii_alphabetic() || value == '_' => {
            CanonicalLexicalTokenKind::Identifier
        }
        _ => CanonicalLexicalTokenKind::Other,
    }
}

fn completion_close_kind(open: char) -> CanonicalLexicalTokenKind {
    match open {
        '(' => CanonicalLexicalTokenKind::ParenthesisClose,
        '[' => CanonicalLexicalTokenKind::ListClose,
        '{' => CanonicalLexicalTokenKind::RecordClose,
        _ => CanonicalLexicalTokenKind::Other,
    }
}

fn completion_actual_token(
    text: &str,
    span: &Span,
    start: usize,
) -> CanonicalActualLexicalEvidence {
    text.get(start..)
        .and_then(|rest| rest.chars().next())
        .map_or(CanonicalActualLexicalEvidence::EndOfInput, |ch| {
            let byte_len = if ch.is_ascii_alphanumeric() || ch == '_' {
                text[start..]
                    .chars()
                    .take_while(|next| next.is_ascii_alphanumeric() || *next == '_')
                    .map(char::len_utf8)
                    .sum()
            } else {
                ch.len_utf8()
            };
            CanonicalActualLexicalEvidence::Token {
                kind: completion_token_kind(ch),
                range: completion_range(span, start, byte_len),
                spelling: text[start..start + byte_len].to_string(),
            }
        })
}

fn malformed_completion(
    cause: CanonicalMalformedCause,
    text: &str,
    span: &Span,
    offending: ParsedSourceRange,
    expected: CanonicalExpectedLexicalEvidence,
    actual: CanonicalActualLexicalEvidence,
) -> CanonicalCompletionEvent {
    malformed_completion_with_producer(
        cause,
        text,
        span,
        offending.clone(),
        offending,
        expected,
        actual,
    )
}

fn malformed_completion_with_producer(
    cause: CanonicalMalformedCause,
    text: &str,
    span: &Span,
    producing_event: ParsedSourceRange,
    offending: ParsedSourceRange,
    expected: CanonicalExpectedLexicalEvidence,
    actual: CanonicalActualLexicalEvidence,
) -> CanonicalCompletionEvent {
    let actual_end = malformed_actual_range(&actual).map_or(0, |range| {
        range.start.column.saturating_sub(span.column) + range.byte_len
    });
    let consumed_end = (offending.start.column.saturating_sub(span.column) + offending.byte_len)
        .max(producing_event.start.column.saturating_sub(span.column) + producing_event.byte_len)
        .max(actual_end)
        .min(text.len());
    CanonicalCompletionEvent::Unsupported(Box::new(CanonicalMalformedEvent {
        cause,
        producing_event,
        offending,
        consumed: completion_range(span, 0, consumed_end),
        expected,
        actual,
    }))
}

fn projected_delimiter_completion(text: &str, span: &Span) -> Option<CanonicalCompletionEvent> {
    let mut stack = Vec::new();
    let mut quoted = false;
    let mut escaped = false;
    let mut quote_start = 0usize;
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
            '"' => {
                quoted = true;
                quote_start = index;
            }
            '(' | '[' | '{' => {
                stack.push((ch, index));
                if stack.len() > CANONICAL_MAX_DELIMITER_DEPTH {
                    let offending = completion_range(span, index, ch.len_utf8());
                    return Some(malformed_completion(
                        CanonicalMalformedCause::DelimiterDepthExceeded,
                        text,
                        span,
                        offending,
                        CanonicalExpectedLexicalEvidence::MaximumDelimiterDepth(
                            CANONICAL_MAX_DELIMITER_DEPTH,
                        ),
                        CanonicalActualLexicalEvidence::DelimiterDepth(stack.len()),
                    ));
                }
            }
            ')' | ']' | '}' => {
                let expected_open = match ch {
                    ')' => '(',
                    ']' => '[',
                    '}' => '{',
                    _ => unreachable!(),
                };
                if stack.last().map(|(open, _)| *open) == Some(expected_open) {
                    stack.pop();
                } else {
                    let offending = completion_range(span, index, ch.len_utf8());
                    let expected = stack.last().map_or(
                        CanonicalExpectedLexicalEvidence::Operand,
                        |(open, _)| {
                            CanonicalExpectedLexicalEvidence::Token(completion_close_kind(*open))
                        },
                    );
                    return Some(malformed_completion(
                        CanonicalMalformedCause::MismatchedDelimiter,
                        text,
                        span,
                        offending.clone(),
                        expected,
                        CanonicalActualLexicalEvidence::Token {
                            kind: completion_token_kind(ch),
                            range: offending,
                            spelling: ch.to_string(),
                        },
                    ));
                }
            }
            _ => {}
        }
    }
    if quoted {
        let offending = completion_range(span, quote_start, text.len() - quote_start);
        return Some(malformed_completion_with_producer(
            CanonicalMalformedCause::UnterminatedTextLiteral,
            text,
            span,
            completion_range(span, quote_start, 1),
            offending,
            CanonicalExpectedLexicalEvidence::Token(CanonicalLexicalTokenKind::TextQuote),
            CanonicalActualLexicalEvidence::EndOfInput,
        ));
    }
    stack.last().map(|(open, open_index)| {
        let offending = completion_range(span, text.len(), 0);
        malformed_completion_with_producer(
            CanonicalMalformedCause::MissingDelimiter,
            text,
            span,
            completion_range(span, *open_index, 1),
            offending,
            CanonicalExpectedLexicalEvidence::Token(completion_close_kind(*open)),
            CanonicalActualLexicalEvidence::EndOfInput,
        )
    })
}

fn retained_delimiter_completion(text: &str, span: &Span) -> Option<CanonicalCompletionEvent> {
    let mut opens: Vec<(u8, usize)> = Vec::new();
    let mut cursor = 0usize;
    let bytes = text.as_bytes();
    let mut text_open = None;
    while cursor < bytes.len() {
        if let Some(open) = text_open {
            if bytes[cursor] == b'\\' {
                cursor = (cursor + 2).min(bytes.len());
                continue;
            }
            if bytes[cursor] == b'"' {
                text_open = None;
            }
            cursor += 1;
            if cursor == bytes.len() && text_open.is_some() {
                let offending = completion_range(span, open, bytes.len() - open);
                return Some(malformed_completion_with_producer(
                    CanonicalMalformedCause::UnterminatedTextLiteral,
                    text,
                    span,
                    completion_range(span, open, 1),
                    offending,
                    CanonicalExpectedLexicalEvidence::Token(CanonicalLexicalTokenKind::TextQuote),
                    CanonicalActualLexicalEvidence::EndOfInput,
                ));
            }
            continue;
        }
        match bytes[cursor] {
            b'"' => text_open = Some(cursor),
            b'(' | b'[' | b'{' => {
                opens.push((bytes[cursor], cursor));
                if opens.len() > CANONICAL_MAX_DELIMITER_DEPTH {
                    let offending = completion_range(span, cursor, 1);
                    return Some(malformed_completion(
                        CanonicalMalformedCause::DelimiterDepthExceeded,
                        text,
                        span,
                        offending,
                        CanonicalExpectedLexicalEvidence::MaximumDelimiterDepth(
                            CANONICAL_MAX_DELIMITER_DEPTH,
                        ),
                        CanonicalActualLexicalEvidence::DelimiterDepth(opens.len()),
                    ));
                }
            }
            b')' | b']' | b'}' => {
                let wanted = match bytes[cursor] {
                    b')' => b'(',
                    b']' => b'[',
                    b'}' => b'{',
                    _ => unreachable!(),
                };
                if opens.last().map(|(open, _)| *open) == Some(wanted) {
                    opens.pop();
                } else {
                    let spelling = (bytes[cursor] as char).to_string();
                    let offending = completion_range(span, cursor, 1);
                    let expected = opens.last().map_or(
                        CanonicalExpectedLexicalEvidence::Operand,
                        |(open, _)| {
                            CanonicalExpectedLexicalEvidence::Token(completion_close_kind(
                                *open as char,
                            ))
                        },
                    );
                    return Some(malformed_completion(
                        CanonicalMalformedCause::MismatchedDelimiter,
                        text,
                        span,
                        offending.clone(),
                        expected,
                        CanonicalActualLexicalEvidence::Token {
                            kind: completion_token_kind(bytes[cursor] as char),
                            range: offending,
                            spelling,
                        },
                    ));
                }
            }
            _ => {}
        }
        cursor += 1;
    }
    if let Some(open) = text_open {
        let offending = completion_range(span, open, text.len() - open);
        return Some(malformed_completion_with_producer(
            CanonicalMalformedCause::UnterminatedTextLiteral,
            text,
            span,
            completion_range(span, open, 1),
            offending,
            CanonicalExpectedLexicalEvidence::Token(CanonicalLexicalTokenKind::TextQuote),
            CanonicalActualLexicalEvidence::EndOfInput,
        ));
    }
    opens.last().map(|(open, open_index)| {
        malformed_completion_with_producer(
            CanonicalMalformedCause::MissingDelimiter,
            text,
            span,
            completion_range(span, *open_index, 1),
            completion_range(span, text.len(), 0),
            CanonicalExpectedLexicalEvidence::Token(completion_close_kind(*open as char)),
            CanonicalActualLexicalEvidence::EndOfInput,
        )
    })
}

fn projected_out_of_range_integer(text: &str) -> Option<(usize, usize)> {
    let mut quoted = false;
    let mut escaped = false;
    let mut index = 0usize;
    while index < text.len() {
        let ch = text[index..].chars().next()?;
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            index += ch.len_utf8();
            continue;
        }
        if ch == '"' {
            quoted = true;
            index += 1;
            continue;
        }
        let signed = ch == '-'
            && text[index + ch.len_utf8()..]
                .chars()
                .next()
                .is_some_and(|next| next.is_ascii_digit());
        if ch.is_ascii_digit() || signed {
            let start = index;
            index += ch.len_utf8();
            while text[index..]
                .chars()
                .next()
                .is_some_and(|next| next.is_ascii_digit())
            {
                index += 1;
            }
            let before = text[..start].chars().next_back();
            let after = text[index..].chars().next();
            let identifier_continue = |value: char| value.is_ascii_alphanumeric() || value == '_';
            let bounded = before.is_none_or(|value| !identifier_continue(value))
                && after.is_none_or(|value| !identifier_continue(value));
            if bounded && text[start..index].parse::<i64>().is_err() {
                return Some((start, index - start));
            }
            continue;
        }
        index += ch.len_utf8();
    }
    None
}

fn retained_out_of_range_integer(text: &str, span: &Span) -> Option<(usize, usize)> {
    let events = canonical_lexical_events(text, span);
    for (index, event) in events.iter().enumerate() {
        let (start, spelling) = if event.spelling == "-"
            && events
                .get(index + 1)
                .is_some_and(|next| next.spelling.chars().all(|ch| ch.is_ascii_digit()))
        {
            let digits = &events[index + 1];
            (
                event.range.start.column.saturating_sub(span.column),
                format!("-{}", digits.spelling),
            )
        } else if event.spelling.chars().all(|ch| ch.is_ascii_digit()) {
            (
                event.range.start.column.saturating_sub(span.column),
                event.spelling.clone(),
            )
        } else {
            continue;
        };
        if spelling.parse::<i64>().is_err() {
            return Some((start, spelling.len()));
        }
    }
    None
}

fn malformed_comparison_range(text: &str) -> Option<(usize, usize)> {
    let mut quoted = false;
    let mut escaped = false;
    text.char_indices().find_map(|(index, ch)| {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            return None;
        }
        if ch == '"' {
            quoted = true;
            return None;
        }
        if matches!(ch, '<' | '>' | '=' | '!') {
            if text[..index]
                .chars()
                .next_back()
                .is_some_and(|previous| matches!(previous, '<' | '>' | '=' | '!'))
            {
                return None;
            }
            let len = text[index..]
                .chars()
                .take_while(|next| matches!(next, '<' | '>' | '=' | '!'))
                .map(char::len_utf8)
                .sum::<usize>();
            (!matches!(
                &text[index..index + len],
                "<" | "<=" | ">" | ">=" | "==" | "!="
            ))
            .then_some((index, len))
        } else {
            None
        }
    })
}

fn retained_malformed_comparison_range(text: &str, span: &Span) -> Option<(usize, usize)> {
    let events = canonical_lexical_events(text, span);
    let mut index = 0usize;
    while index < events.len() {
        if !events[index]
            .spelling
            .chars()
            .all(|ch| matches!(ch, '<' | '>' | '=' | '!'))
        {
            index += 1;
            continue;
        }
        let start = events[index].range.start.column.saturating_sub(span.column);
        let mut spelling = events[index].spelling.clone();
        let mut end = start + events[index].range.byte_len;
        index += 1;
        while index < events.len()
            && events[index].range.start.column.saturating_sub(span.column) == end
            && events[index]
                .spelling
                .chars()
                .all(|ch| matches!(ch, '<' | '>' | '=' | '!'))
        {
            spelling.push_str(&events[index].spelling);
            end += events[index].range.byte_len;
            index += 1;
        }
        if !matches!(spelling.as_str(), "<" | "<=" | ">" | ">=" | "==" | "!=") {
            return Some((start, end - start));
        }
    }
    None
}

fn projected_missing_operand(text: &str) -> Option<(usize, usize)> {
    let trimmed = text.trim_end();
    [
        "fails with",
        "returns",
        "does",
        "and",
        "or",
        "==",
        "!=",
        "<=",
        ">=",
        "<",
        ">",
        "+",
        "-",
        "*",
        "/",
        "is",
    ]
    .iter()
    .find_map(|operator| {
        let prefix = trimmed.strip_suffix(operator)?;
        let start = prefix.len();
        let bounded = !operator.chars().all(|ch| ch.is_ascii_alphabetic())
            || start == 0
            || prefix.chars().next_back().is_some_and(char::is_whitespace);
        bounded.then_some((start, operator.len()))
    })
}

fn retained_missing_operand(text: &str, span: &Span) -> Option<(usize, usize)> {
    let events = canonical_lexical_events(text, span);
    let last = events.last()?;
    let operator = matches!(
        last.spelling.as_str(),
        "==" | "!="
            | "<="
            | ">="
            | "<"
            | ">"
            | "+"
            | "-"
            | "*"
            | "/"
            | "is"
            | "does"
            | "returns"
            | "and"
            | "or"
            | "with"
    );
    if !operator {
        return None;
    }
    let start = if last.spelling == "with"
        && events
            .get(events.len().saturating_sub(2))
            .is_some_and(|event| event.spelling == "fails")
    {
        events[events.len() - 2]
            .range
            .start
            .column
            .saturating_sub(span.column)
    } else {
        last.range.start.column.saturating_sub(span.column)
    };
    Some((start, text.trim_end().len() - start))
}

fn malformed_field_offset(text: &str) -> Option<usize> {
    let mut quoted = false;
    let mut escaped = false;
    text.char_indices().find_map(|(index, ch)| {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            return None;
        }
        if ch == '"' {
            quoted = true;
            return None;
        }
        if ch != '.' {
            return None;
        }
        text[index + 1..]
            .chars()
            .next()
            .is_none_or(|next| !(next.is_ascii_lowercase() || next == '_'))
            .then_some(index)
    })
}

fn retained_malformed_field_offset(text: &str, span: &Span) -> Option<usize> {
    let events = canonical_lexical_events(text, span);
    events.iter().enumerate().find_map(|(index, event)| {
        if event.spelling != "." {
            return None;
        }
        let valid = events.get(index + 1).is_some_and(|next| {
            next.spelling
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_lowercase() || first == '_')
                && next
                    .spelling
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        });
        (!valid).then_some(event.range.start.column.saturating_sub(span.column))
    })
}

fn projected_completion_event(
    kind: &CanonicalExpressionKind,
    text: &str,
    span: &Span,
) -> CanonicalCompletionEvent {
    if let Some(issue) = projected_delimiter_completion(text, span) {
        return issue;
    }
    if let Some((start, len)) = projected_out_of_range_integer(text) {
        let offending = completion_range(span, start, len);
        return malformed_completion(
            CanonicalMalformedCause::IntegerLiteralOutOfRange,
            text,
            span,
            offending.clone(),
            CanonicalExpectedLexicalEvidence::Int64Value,
            CanonicalActualLexicalEvidence::Token {
                kind: CanonicalLexicalTokenKind::IntegerLiteral,
                range: offending,
                spelling: text[start..start + len].to_string(),
            },
        );
    }
    if let Some((operator_start, operator_len)) = projected_missing_operand(text) {
        return malformed_completion_with_producer(
            CanonicalMalformedCause::MissingOperand,
            text,
            span,
            completion_range(span, operator_start, operator_len),
            completion_range(span, text.trim_end().len(), 0),
            CanonicalExpectedLexicalEvidence::Operand,
            CanonicalActualLexicalEvidence::EndOfInput,
        );
    }
    if let Some((start, len)) = malformed_comparison_range(text) {
        let offending = completion_range(span, start, len);
        return malformed_completion(
            CanonicalMalformedCause::InvalidComparisonOperator,
            text,
            span,
            offending.clone(),
            CanonicalExpectedLexicalEvidence::ComparisonOperator,
            CanonicalActualLexicalEvidence::Token {
                kind: CanonicalLexicalTokenKind::ComparisonOperator,
                range: offending,
                spelling: text[start..start + len].to_string(),
            },
        );
    }
    if let Some(start) = malformed_field_offset(text) {
        return malformed_completion(
            CanonicalMalformedCause::MalformedFieldPlace,
            text,
            span,
            completion_range(span, start, 1),
            CanonicalExpectedLexicalEvidence::Identifier,
            completion_actual_token(text, span, start),
        );
    }
    if matches!(kind, CanonicalExpressionKind::Unsupported) {
        let leading = text.len() - text.trim_start().len();
        let actual = completion_actual_token(text, span, leading);
        let byte_len = malformed_actual_range(&actual).map_or(0, |range| range.byte_len);
        return malformed_completion(
            CanonicalMalformedCause::InvalidOperandStarter,
            text,
            span,
            completion_range(span, leading, byte_len),
            CanonicalExpectedLexicalEvidence::Operand,
            actual,
        );
    }
    CanonicalCompletionEvent::Complete
}

fn retained_completion_event(
    kind: CanonicalCommonNodeKind,
    text: &str,
    span: &Span,
) -> CanonicalCompletionEvent {
    if let Some(issue) = retained_delimiter_completion(text, span) {
        return issue;
    }
    if let Some((start, len)) = retained_out_of_range_integer(text, span) {
        let offending = completion_range(span, start, len);
        return malformed_completion(
            CanonicalMalformedCause::IntegerLiteralOutOfRange,
            text,
            span,
            offending.clone(),
            CanonicalExpectedLexicalEvidence::Int64Value,
            CanonicalActualLexicalEvidence::Token {
                kind: CanonicalLexicalTokenKind::IntegerLiteral,
                range: offending,
                spelling: text[start..start + len].to_string(),
            },
        );
    }
    if let Some((operator_start, operator_len)) = retained_missing_operand(text, span) {
        return malformed_completion_with_producer(
            CanonicalMalformedCause::MissingOperand,
            text,
            span,
            completion_range(span, operator_start, operator_len),
            completion_range(span, text.trim_end().len(), 0),
            CanonicalExpectedLexicalEvidence::Operand,
            CanonicalActualLexicalEvidence::EndOfInput,
        );
    }
    if let Some((start, len)) = retained_malformed_comparison_range(text, span) {
        let offending = completion_range(span, start, len);
        return malformed_completion(
            CanonicalMalformedCause::InvalidComparisonOperator,
            text,
            span,
            offending.clone(),
            CanonicalExpectedLexicalEvidence::ComparisonOperator,
            CanonicalActualLexicalEvidence::Token {
                kind: CanonicalLexicalTokenKind::ComparisonOperator,
                range: offending,
                spelling: text[start..start + len].to_string(),
            },
        );
    }
    if let Some(start) = retained_malformed_field_offset(text, span) {
        return malformed_completion(
            CanonicalMalformedCause::MalformedFieldPlace,
            text,
            span,
            completion_range(span, start, 1),
            CanonicalExpectedLexicalEvidence::Identifier,
            completion_actual_token(text, span, start),
        );
    }
    if kind == CanonicalCommonNodeKind::Unsupported {
        let start = text.len() - text.trim_start().len();
        let actual = completion_actual_token(text, span, start);
        let byte_len = malformed_actual_range(&actual).map_or(0, |range| range.byte_len);
        return malformed_completion(
            CanonicalMalformedCause::InvalidOperandStarter,
            text,
            span,
            completion_range(span, start, byte_len),
            CanonicalExpectedLexicalEvidence::Operand,
            actual,
        );
    }
    CanonicalCompletionEvent::Complete
}

fn projected_predicate_list_completion(
    text: &str,
    span: &Span,
) -> Option<CanonicalCompletionEvent> {
    let open = text
        .char_indices()
        .find_map(|(index, ch)| (ch == '[').then_some(index))?;
    let close = matching_delimiter_quoted(text, open, '[', ']')?;
    let inside = &text[open + 1..close];
    if inside.trim_end().ends_with(',') {
        let comma = open + 1 + inside.rfind(',')?;
        return Some(malformed_completion(
            CanonicalMalformedCause::ListTrailingComma,
            text,
            span,
            completion_range(span, comma, 1),
            CanonicalExpectedLexicalEvidence::TextListElement,
            CanonicalActualLexicalEvidence::Token {
                kind: CanonicalLexicalTokenKind::ListClose,
                range: completion_range(span, close, 1),
                spelling: "]".to_string(),
            },
        ));
    }
    for range in split_top_level_ranges_quoted(inside, ',') {
        let raw = &inside[range.clone()];
        let value = raw.trim();
        let leading = raw.len() - raw.trim_start().len();
        let absolute = open + 1 + range.start + leading;
        if value.starts_with('"') {
            let mut escaped = false;
            let end = value.char_indices().skip(1).find_map(|(index, ch)| {
                if escaped {
                    escaped = false;
                    None
                } else if ch == '\\' {
                    escaped = true;
                    None
                } else {
                    (ch == '"').then_some(index)
                }
            });
            let Some(end) = end else {
                continue;
            };
            let trailing = value[end + 1..].trim_start();
            if !trailing.is_empty() {
                let offset = absolute + value.len() - trailing.len();
                return Some(malformed_completion(
                    CanonicalMalformedCause::ListElementSeparator,
                    text,
                    span,
                    completion_range(span, offset, trailing.chars().next()?.len_utf8()),
                    CanonicalExpectedLexicalEvidence::ListSeparatorOrClose,
                    completion_actual_token(text, span, offset),
                ));
            }
        } else if !value.is_empty() {
            return Some(malformed_completion(
                CanonicalMalformedCause::ListNonTextElement,
                text,
                span,
                completion_range(span, absolute, value.chars().next()?.len_utf8()),
                CanonicalExpectedLexicalEvidence::TextListElement,
                completion_actual_token(text, span, absolute),
            ));
        }
    }
    None
}

fn retained_predicate_list_completion(text: &str, span: &Span) -> Option<CanonicalCompletionEvent> {
    let bytes = text.as_bytes();
    let mut open = None;
    let mut index = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    while index < bytes.len() {
        if quoted {
            if escaped {
                escaped = false;
            } else if bytes[index] == b'\\' {
                escaped = true;
            } else if bytes[index] == b'"' {
                quoted = false;
            }
            index += 1;
            continue;
        }
        match bytes[index] {
            b'"' => quoted = true,
            b'[' => {
                open = Some(index);
                break;
            }
            _ => {}
        }
        index += 1;
    }
    let open = open?;
    let close = matching_delimiter_quoted(text, open, '[', ']')?;
    let inside = &text[open + 1..close];
    let trimmed = inside.trim_end();
    if trimmed.as_bytes().last() == Some(&b',') {
        let comma = open + 1 + trimmed.len() - 1;
        return Some(malformed_completion(
            CanonicalMalformedCause::ListTrailingComma,
            text,
            span,
            completion_range(span, comma, 1),
            CanonicalExpectedLexicalEvidence::TextListElement,
            CanonicalActualLexicalEvidence::Token {
                kind: CanonicalLexicalTokenKind::ListClose,
                range: completion_range(span, close, 1),
                spelling: "]".to_string(),
            },
        ));
    }
    for range in split_top_level_ranges_quoted(inside, ',') {
        let raw = &inside[range.clone()];
        let leading = raw.len() - raw.trim_start().len();
        let value = raw.trim();
        let absolute = open + 1 + range.start + leading;
        if value.is_empty() {
            continue;
        }
        if value.as_bytes().first() != Some(&b'"') {
            return Some(malformed_completion(
                CanonicalMalformedCause::ListNonTextElement,
                text,
                span,
                completion_range(span, absolute, value.chars().next()?.len_utf8()),
                CanonicalExpectedLexicalEvidence::TextListElement,
                completion_actual_token(text, span, absolute),
            ));
        }
        let mut cursor = 1usize;
        let mut closed = None;
        while cursor < value.len() {
            if value.as_bytes()[cursor] == b'\\' {
                cursor += 1;
                if cursor < value.len() {
                    let escaped = value[cursor..].chars().next()?;
                    cursor += escaped.len_utf8();
                }
            } else if value.as_bytes()[cursor] == b'"' {
                closed = Some(cursor);
                break;
            } else {
                let ch = value[cursor..].chars().next()?;
                cursor += ch.len_utf8();
            }
        }
        if let Some(closed) = closed {
            let after = value[closed + 1..].trim_start();
            if !after.is_empty() {
                let offset = absolute + value.len() - after.len();
                return Some(malformed_completion(
                    CanonicalMalformedCause::ListElementSeparator,
                    text,
                    span,
                    completion_range(span, offset, after.chars().next()?.len_utf8()),
                    CanonicalExpectedLexicalEvidence::ListSeparatorOrClose,
                    completion_actual_token(text, span, offset),
                ));
            }
        }
    }
    None
}

fn apply_predicate_completion(expression: &mut ParsedExpression, text: &str, span: &Span) {
    if let Some(completion) = projected_predicate_list_completion(text, span) {
        expression.canonical.completion = completion;
        expression.canonical.payload.clear();
    }
    if let Some(completion) = retained_predicate_list_completion(text, span) {
        expression.canonical_event.completion = completion;
        expression.canonical_event.lexical_status = CanonicalCommonLexicalStatus::Unsupported;
        expression.canonical_event.payload.clear();
    }
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
    text: &str,
) -> CanonicalExpressionBuild {
    let own_projected_completion = projected_completion_event(&kind, text, &range.start);
    let projected_completion =
        if matches!(own_projected_completion, CanonicalCompletionEvent::Complete) {
            projected_child_completion(&kind).unwrap_or(own_projected_completion)
        } else {
            own_projected_completion
        };
    let own_retained_completion = retained_completion_event(event_kind, text, &range.start);
    let retained_completion =
        if matches!(own_retained_completion, CanonicalCompletionEvent::Complete) {
            children
                .iter()
                .find_map(|child| match &child.event.completion {
                    CanonicalCompletionEvent::Unsupported(event) => {
                        Some(CanonicalCompletionEvent::Unsupported(event.clone()))
                    }
                    CanonicalCompletionEvent::Complete => None,
                })
                .unwrap_or(own_retained_completion)
        } else {
            own_retained_completion
        };
    let projected_payload = if matches!(projected_completion, CanonicalCompletionEvent::Complete) {
        projected_payload_events(&kind, text, &range.start, children.len())
    } else {
        Vec::new()
    };
    let retained_payload = if matches!(retained_completion, CanonicalCompletionEvent::Complete) {
        retained_payload_events(event_kind, text, &range.start, children.len())
    } else {
        Vec::new()
    };
    CanonicalExpressionBuild {
        expression: CanonicalExpression {
            node_id,
            range: range.clone(),
            kind,
            payload: projected_payload,
            completion: projected_completion,
        },
        event: CanonicalReductionEvent {
            range,
            kind: event_kind,
            children,
            delimiter_depth_before: delimiter_depth,
            delimiter_depth_after: delimiter_depth,
            lexical_status: if matches!(
                retained_completion,
                CanonicalCompletionEvent::Unsupported(_)
            ) {
                CanonicalCommonLexicalStatus::Unsupported
            } else {
                CanonicalCommonLexicalStatus::Complete
            },
            payload: retained_payload,
            completion: retained_completion,
        },
    }
}

fn projected_child_completion(kind: &CanonicalExpressionKind) -> Option<CanonicalCompletionEvent> {
    let children: Vec<&CanonicalExpression> = match kind {
        CanonicalExpressionKind::Field { base, .. }
        | CanonicalExpressionKind::ElementPlace { base, .. }
        | CanonicalExpressionKind::Permission { value: base, .. }
        | CanonicalExpressionKind::Try { value: base, .. }
        | CanonicalExpressionKind::Group(base) => vec![base],
        CanonicalExpressionKind::ListLiteral(values) => values.iter().collect(),
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            fields.iter().map(|(_, value)| value).collect()
        }
        CanonicalExpressionKind::Call { callee, arguments } => {
            std::iter::once(callee.as_ref()).chain(arguments).collect()
        }
        CanonicalExpressionKind::Binary { left, right, .. } => vec![left, right],
        CanonicalExpressionKind::Unit
        | CanonicalExpressionKind::Identifier(_)
        | CanonicalExpressionKind::UIntLiteral(_)
        | CanonicalExpressionKind::IntLiteral(_)
        | CanonicalExpressionKind::BoolLiteral(_)
        | CanonicalExpressionKind::TextLiteral(_)
        | CanonicalExpressionKind::Unsupported => Vec::new(),
    };
    children
        .into_iter()
        .find_map(|child| match &child.completion {
            CanonicalCompletionEvent::Unsupported(event) => {
                Some(CanonicalCompletionEvent::Unsupported(event.clone()))
            }
            CanonicalCompletionEvent::Complete => projected_child_completion(&child.kind),
        })
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
            text,
        );
    }

    if let Some(rest) = keyword_rest(text, "try") {
        let value_offset = text.len() - rest.len();
        let Ok((value_text, failure_root, failure_variant)) = split_try_wrapper(rest) else {
            return canonical_expression_build(
                node_id,
                range,
                CanonicalExpressionKind::Unsupported,
                CanonicalCommonNodeKind::Unsupported,
                Vec::new(),
                delimiter_depth,
                text,
            );
        };
        let value = parse_canonical_expression(
            value_text,
            &offset_span(&span, value_offset),
            node_id.child("try-value"),
            delimiter_depth,
        );
        let children = vec![reduction_child(
            CanonicalCommonChildRole::TryValue,
            0,
            &value,
        )];
        return canonical_expression_build(
            node_id,
            range,
            CanonicalExpressionKind::Try {
                value: Box::new(value.expression),
                failure_root,
                failure_variant,
            },
            CanonicalCommonNodeKind::Try,
            children,
            delimiter_depth,
            text,
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
                text,
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
            text,
        );
    }

    if let Some((operator, start, end)) = top_level_binary_operator(text) {
        if binary_operator_is_in_malformed_comparison_run(text, start, end) {
            return canonical_expression_build(
                node_id,
                range,
                CanonicalExpressionKind::Unsupported,
                CanonicalCommonNodeKind::Unsupported,
                Vec::new(),
                delimiter_depth,
                text,
            );
        }
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
            text,
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
            text,
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
            text,
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
            text,
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
            text,
        );
    }

    if let Some(open) = terminal_element_open(text) {
        let index_text = text[open + 1..text.len() - 1].trim();
        if !text[..open].trim().is_empty()
            && !index_text.is_empty()
            && index_text.chars().all(|ch| ch.is_ascii_digit())
            && let Ok(index) = index_text.parse::<u64>()
        {
            let base = parse_canonical_expression(
                &text[..open],
                &span,
                node_id.child("element-base"),
                delimiter_depth,
            );
            let children = vec![reduction_child(
                CanonicalCommonChildRole::ElementBase,
                0,
                &base,
            )];
            return canonical_expression_build(
                node_id,
                range,
                CanonicalExpressionKind::ElementPlace {
                    base: Box::new(base.expression),
                    index,
                },
                CanonicalCommonNodeKind::ElementPlace,
                children,
                delimiter_depth,
                text,
            );
        }
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
            text,
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
            text,
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
            text,
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
            text,
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
                text,
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
        text,
    )
}

fn split_try_wrapper(text: &str) -> Result<(&str, Option<String>, Option<String>), &'static str> {
    let Some(wrapper_start) = find_top_level_phrase(text, " or fail ") else {
        let value = text.trim_end();
        return (!value.is_empty())
            .then_some((value, None, None))
            .ok_or("missing_try_value_v0");
    };
    let value = text[..wrapper_start].trim_end();
    let failure = text[wrapper_start + " or fail ".len()..].trim();
    let Some((root, variant)) = failure.split_once('.') else {
        return Err("malformed_try_failure_path_v0");
    };
    if value.is_empty() || !is_type_identifier(root) || !is_value_identifier(variant) {
        return Err("malformed_try_failure_path_v0");
    }
    Ok((value, Some(root.to_string()), Some(variant.to_string())))
}

fn find_top_level_phrase(text: &str, phrase: &str) -> Option<usize> {
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
            _ if depth == 0 && text[index..].starts_with(phrase) => return Some(index),
            _ => {}
        }
    }
    None
}

fn terminal_element_open(text: &str) -> Option<usize> {
    if !text.ends_with(']') {
        return None;
    }
    text.char_indices()
        .filter(|(_, ch)| *ch == '[')
        .filter_map(|(open, _)| {
            (open > 0 && matching_delimiter_quoted(text, open, '[', ']') == Some(text.len() - 1))
                .then_some(open)
        })
        .next_back()
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
        CanonicalExpressionRole, CanonicalItemOwner, CanonicalMalformedSealFact,
        CanonicalMalformedSealField, CanonicalMalformedSealValue, CanonicalNodeIdentity,
        CanonicalOccurrenceIdentity, CanonicalOccurrenceSeal, CanonicalPayloadSealFact,
        CanonicalPayloadValue, CanonicalPredicateRecognition, CanonicalReductionIdentity,
        CanonicalSealFact, CanonicalSectionOwner, CanonicalSemanticFile, CanonicalSourceBlob,
        CanonicalSourceOwnerFact, CanonicalSourceOwnerSeal, CanonicalSourceRevision,
        CanonicalStatementBlockIdentity, CanonicalStatementOwner, CanonicalStatementSeal,
        CanonicalStatementSealFact, CanonicalStatementSealValue, CanonicalTokenIdentity,
        build_occurrence_seal, executable_call_nodes, parse_source, parse_source_at_index,
        source_owner_fact_matches, validate_canonical_expression, validate_occurrence_seal,
        validate_occurrence_seal_ignoring_one_fact,
        validate_occurrence_seal_ignoring_one_payload_fact, validate_retained_body_syntax,
        validate_source_owner_seal, validate_statement_seal,
    };
    use crate::ast::{
        CanonicalActualLexicalEvidence, CanonicalAssociativity, CanonicalCommonChildRole,
        CanonicalCommonLexicalStatus, CanonicalCommonNodeKind, CanonicalDelimiterKind,
        CanonicalExpectedLexicalEvidence, CanonicalExpressionKind, CanonicalMalformedCause,
        CanonicalPayloadField, CanonicalStatementEventField, CanonicalStatementKindEvent,
        CanonicalTryWrapperKind, Item, ParsedBinaryOperator, ParsedBlockRelationship,
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

    const F1_COMMON_KINDS: [CanonicalCommonNodeKind; 16] = [
        CanonicalCommonNodeKind::Unit,
        CanonicalCommonNodeKind::Identifier,
        CanonicalCommonNodeKind::Field,
        CanonicalCommonNodeKind::ElementPlace,
        CanonicalCommonNodeKind::UIntLiteral,
        CanonicalCommonNodeKind::IntLiteral,
        CanonicalCommonNodeKind::BoolLiteral,
        CanonicalCommonNodeKind::TextLiteral,
        CanonicalCommonNodeKind::ListLiteral,
        CanonicalCommonNodeKind::RecordLiteral,
        CanonicalCommonNodeKind::Call,
        CanonicalCommonNodeKind::Permission,
        CanonicalCommonNodeKind::Try,
        CanonicalCommonNodeKind::Binary,
        CanonicalCommonNodeKind::Group,
        CanonicalCommonNodeKind::Unsupported,
    ];

    const F1_CHILD_ROLES: [CanonicalCommonChildRole; 11] = [
        CanonicalCommonChildRole::FieldBase,
        CanonicalCommonChildRole::ElementBase,
        CanonicalCommonChildRole::ListElement,
        CanonicalCommonChildRole::RecordFieldValue,
        CanonicalCommonChildRole::CallCallee,
        CanonicalCommonChildRole::CallArgument,
        CanonicalCommonChildRole::PermissionValue,
        CanonicalCommonChildRole::TryValue,
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
    return items[0]
    return try source(value)
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
    return items[0]
    return try source(value)
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
        CommonKindRow,
        ChildRoleRow,
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
        common_variant_rejections: usize,
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
        let common_kinds = if matches!(sabotage, Some(F1Sabotage::CommonKindRow)) {
            &F1_COMMON_KINDS[..F1_COMMON_KINDS.len() - 1]
        } else {
            &F1_COMMON_KINDS[..]
        };
        let child_roles = if matches!(sabotage, Some(F1Sabotage::ChildRoleRow)) {
            &F1_CHILD_ROLES[..F1_CHILD_ROLES.len() - 1]
        } else {
            &F1_CHILD_ROLES[..]
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
        let mut common_variant_rejections = 0usize;
        for kind in common_kinds {
            let (seal, index) = parsed
                .occurrence_seals
                .iter()
                .enumerate()
                .find_map(|(seal, occurrence)| {
                    occurrence
                        .projection
                        .iter()
                        .position(|fact| {
                            matches!(fact, CanonicalSealFact::Kind(_, actual) if actual == kind)
                        })
                        .map(|index| (seal, index))
                })
                .unwrap_or_else(|| panic!("F1 corpus lacks {kind:?}"));
            let mut corrupted = parsed.occurrence_seals[seal].clone();
            corrupted.projection[index] = corrupted_f1_fact(&corrupted.projection[index]);
            common_variant_rejections += usize::from(validate_occurrence_seal(&corrupted).is_err());
        }
        for role in child_roles {
            let (seal, index) = parsed
                .occurrence_seals
                .iter()
                .enumerate()
                .find_map(|(seal, occurrence)| {
                    occurrence
                        .projection
                        .iter()
                        .position(|fact| {
                            matches!(
                                fact,
                                CanonicalSealFact::ChildRole(_, Some(actual)) if actual == role
                            )
                        })
                        .map(|index| (seal, index))
                })
                .unwrap_or_else(|| panic!("F1 corpus lacks {role:?}"));
            let mut corrupted = parsed.occurrence_seals[seal].clone();
            corrupted.projection[index] = corrupted_f1_fact(&corrupted.projection[index]);
            common_variant_rejections += usize::from(validate_occurrence_seal(&corrupted).is_err());
        }
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
        if !matches!(sabotage, Some(F1Sabotage::ProducerArm)) {
            corrupted_expression.canonical_event.kind = CanonicalCommonNodeKind::Unsupported;
            corrupted_expression.canonical_event.lexical_status =
                CanonicalCommonLexicalStatus::Unsupported;
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
            kind_count: common_kinds
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
            child_role_count: child_roles
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
            common_variant_rejections,
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
            && evidence.kind_count == 16
            && evidence.role_count == 13
            && evidence.child_role_count == 11
            && evidence.lexical_status_count == 2
            // F3 adds the independently owned `set` target occurrence while
            // retaining every F1 identity and topology obligation.
            && evidence.node_count == 55
            && evidence.token_count == 79
            && evidence.single_rejections == 216
            && evidence.pair_rejections == 630
            && evidence.cross_occurrence_rejections == 16
            && evidence.common_variant_rejections == 27
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
            F1Sabotage::CommonKindRow,
            F1Sabotage::ChildRoleRow,
        ] {
            assert!(
                !complete_f1_evidence(&f1_evidence(Some(sabotage))),
                "{sabotage:?} sabotage stayed green"
            );
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum F2SealField {
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

    const F2_SEAL_FIELDS: [F2SealField; 64] = [
        F2SealField::UnitPosition,
        F2SealField::IdentifierToken,
        F2SealField::IdentifierValue,
        F2SealField::UIntDigitsToken,
        F2SealField::UIntValue,
        F2SealField::IntSignToken,
        F2SealField::IntDigitsToken,
        F2SealField::IntValue,
        F2SealField::IntSignedLiteral,
        F2SealField::BoolToken,
        F2SealField::BoolValue,
        F2SealField::TextOpenQuote,
        F2SealField::TextCloseQuote,
        F2SealField::TextRawContent,
        F2SealField::TextEscapeEvents,
        F2SealField::TextDecodedValue,
        F2SealField::TextTerminated,
        F2SealField::FieldBaseEdge,
        F2SealField::FieldDotToken,
        F2SealField::FieldNameToken,
        F2SealField::FieldValue,
        F2SealField::ElementBaseEdge,
        F2SealField::ElementOpenBracket,
        F2SealField::ElementCloseBracket,
        F2SealField::ElementIndexToken,
        F2SealField::ElementIndexValue,
        F2SealField::ElementPlaceRole,
        F2SealField::DelimiterPair,
        F2SealField::DelimiterNestingParent,
        F2SealField::DelimiterSemanticGaps,
        F2SealField::DelimiterSeparators,
        F2SealField::AggregateEmpty,
        F2SealField::AggregateTrailing,
        F2SealField::GroupValueEdge,
        F2SealField::ListElementEdges,
        F2SealField::RecordNameToken,
        F2SealField::RecordFieldTokens,
        F2SealField::RecordColonTokens,
        F2SealField::RecordValueEdges,
        F2SealField::CallCalleeEdge,
        F2SealField::CallArgumentEdges,
        F2SealField::CallAdjacency,
        F2SealField::CallCloseState,
        F2SealField::CallTrailingState,
        F2SealField::BinaryOperator,
        F2SealField::BinaryOperatorTokens,
        F2SealField::BinaryOperatorRange,
        F2SealField::BinaryPrecedence,
        F2SealField::BinaryAssociativity,
        F2SealField::BinaryLeftBoundary,
        F2SealField::BinaryRightBoundary,
        F2SealField::BinaryReductionOrder,
        F2SealField::BinaryChildRoles,
        F2SealField::PermissionKeyword,
        F2SealField::PermissionDiscriminant,
        F2SealField::PermissionGap,
        F2SealField::PermissionValueEdge,
        F2SealField::TryKeyword,
        F2SealField::TryValueEdge,
        F2SealField::TryWrapperRelation,
        F2SealField::TryFailureRootToken,
        F2SealField::TryDotToken,
        F2SealField::TryFailureVariantToken,
        F2SealField::TryWrapperKind,
    ];

    const F2_SOURCE_A: &str = r#"# alpha
task payload(value: UInt, other: UInt) -> UInt {
  does:
    return
    return value
    return value
    return 7
    return 7
    return -7
    return 7 - 2
    return true
    return false
    return "hé\"llo"
    return "\é"
    return "🙂\éß"
    return target.field
    return items[0]
    return (value)
    return [value, 7,]
    return []
    return Thing { first: value, second: 7, }
    return {}
    return call(value, 7,)
    return call()
    return call(value)
    return call(value)
    return (call(value))
    return call("a, b", nested(value, other), [value, other], value)
    return value * other
    return value / other
    return value + other
    return value - other
    return value == other
    return value != other
    return value < other
    return value <= other
    return value > other
    return value >= other
    return value is other
    return value does other
    return value returns other
    return value fails with other
    return value and other
    return value or other
    return borrow value
    return change value
    return consume value
    return try source(value) or fail PayloadError.context
    return try source(value)
}
"#;

    const F2_SOURCE_B: &str = r#"# bravo
task payload(value: UInt, other: UInt) -> UInt {
  does:
    return
    return value
    return value
    return 7
    return 7
    return -7
    return 7 - 2
    return true
    return false
    return "hé\"llo"
    return "\é"
    return "🙂\éß"
    return target.field
    return items[0]
    return (value)
    return [value, 7,]
    return []
    return Thing { first: value, second: 7, }
    return {}
    return call(value, 7,)
    return call()
    return call(value)
    return call(value)
    return (call(value))
    return call("a, b", nested(value, other), [value, other], value)
    return value * other
    return value / other
    return value + other
    return value - other
    return value == other
    return value != other
    return value < other
    return value <= other
    return value > other
    return value >= other
    return value is other
    return value does other
    return value returns other
    return value fails with other
    return value and other
    return value or other
    return borrow value
    return change value
    return consume value
    return try source(value) or fail PayloadError.context
    return try source(value)
}
"#;

    fn f2_field(field: CanonicalPayloadField) -> F2SealField {
        use CanonicalPayloadField as P;
        match field {
            P::UnitPosition => F2SealField::UnitPosition,
            P::IdentifierToken => F2SealField::IdentifierToken,
            P::IdentifierValue => F2SealField::IdentifierValue,
            P::UIntDigitsToken => F2SealField::UIntDigitsToken,
            P::UIntValue => F2SealField::UIntValue,
            P::IntSignToken => F2SealField::IntSignToken,
            P::IntDigitsToken => F2SealField::IntDigitsToken,
            P::IntValue => F2SealField::IntValue,
            P::IntSignedLiteral => F2SealField::IntSignedLiteral,
            P::BoolToken => F2SealField::BoolToken,
            P::BoolValue => F2SealField::BoolValue,
            P::TextOpenQuote => F2SealField::TextOpenQuote,
            P::TextCloseQuote => F2SealField::TextCloseQuote,
            P::TextRawContent => F2SealField::TextRawContent,
            P::TextEscapeEvents => F2SealField::TextEscapeEvents,
            P::TextDecodedValue => F2SealField::TextDecodedValue,
            P::TextTerminated => F2SealField::TextTerminated,
            P::FieldBaseEdge => F2SealField::FieldBaseEdge,
            P::FieldDotToken => F2SealField::FieldDotToken,
            P::FieldNameToken => F2SealField::FieldNameToken,
            P::FieldValue => F2SealField::FieldValue,
            P::ElementBaseEdge => F2SealField::ElementBaseEdge,
            P::ElementOpenBracket => F2SealField::ElementOpenBracket,
            P::ElementCloseBracket => F2SealField::ElementCloseBracket,
            P::ElementIndexToken => F2SealField::ElementIndexToken,
            P::ElementIndexValue => F2SealField::ElementIndexValue,
            P::ElementPlaceRole => F2SealField::ElementPlaceRole,
            P::DelimiterPair => F2SealField::DelimiterPair,
            P::DelimiterNestingParent => F2SealField::DelimiterNestingParent,
            P::DelimiterSemanticGaps => F2SealField::DelimiterSemanticGaps,
            P::DelimiterSeparators => F2SealField::DelimiterSeparators,
            P::AggregateEmpty => F2SealField::AggregateEmpty,
            P::AggregateTrailing => F2SealField::AggregateTrailing,
            P::GroupValueEdge => F2SealField::GroupValueEdge,
            P::ListElementEdges => F2SealField::ListElementEdges,
            P::RecordNameToken => F2SealField::RecordNameToken,
            P::RecordFieldTokens => F2SealField::RecordFieldTokens,
            P::RecordColonTokens => F2SealField::RecordColonTokens,
            P::RecordValueEdges => F2SealField::RecordValueEdges,
            P::CallCalleeEdge => F2SealField::CallCalleeEdge,
            P::CallArgumentEdges => F2SealField::CallArgumentEdges,
            P::CallAdjacency => F2SealField::CallAdjacency,
            P::CallCloseState => F2SealField::CallCloseState,
            P::CallTrailingState => F2SealField::CallTrailingState,
            P::BinaryOperator => F2SealField::BinaryOperator,
            P::BinaryOperatorTokens => F2SealField::BinaryOperatorTokens,
            P::BinaryOperatorRange => F2SealField::BinaryOperatorRange,
            P::BinaryPrecedence => F2SealField::BinaryPrecedence,
            P::BinaryAssociativity => F2SealField::BinaryAssociativity,
            P::BinaryLeftBoundary => F2SealField::BinaryLeftBoundary,
            P::BinaryRightBoundary => F2SealField::BinaryRightBoundary,
            P::BinaryReductionOrder => F2SealField::BinaryReductionOrder,
            P::BinaryChildRoles => F2SealField::BinaryChildRoles,
            P::PermissionKeyword => F2SealField::PermissionKeyword,
            P::PermissionDiscriminant => F2SealField::PermissionDiscriminant,
            P::PermissionGap => F2SealField::PermissionGap,
            P::PermissionValueEdge => F2SealField::PermissionValueEdge,
            P::TryKeyword => F2SealField::TryKeyword,
            P::TryValueEdge => F2SealField::TryValueEdge,
            P::TryWrapperRelation => F2SealField::TryWrapperRelation,
            P::TryFailureRootToken => F2SealField::TryFailureRootToken,
            P::TryDotToken => F2SealField::TryDotToken,
            P::TryFailureVariantToken => F2SealField::TryFailureVariantToken,
            P::TryWrapperKind => F2SealField::TryWrapperKind,
        }
    }

    fn corrupt_payload_fact(
        fact: &CanonicalPayloadSealFact,
        foreign: &CanonicalPayloadSealFact,
    ) -> CanonicalPayloadSealFact {
        let mut fact = fact.clone();
        fact.identity.0 = corrupt_owner_identity(&fact.identity.0);
        fact.value = match &fact.value {
            CanonicalPayloadValue::Position(value) => {
                let mut value = value.clone();
                value.column += 1;
                CanonicalPayloadValue::Position(value)
            }
            CanonicalPayloadValue::Token(identity, range, spelling) => {
                CanonicalPayloadValue::Token(
                    corrupt_token(identity),
                    range.clone(),
                    spelling.clone(),
                )
            }
            CanonicalPayloadValue::Tokens(values) => {
                let mut values = values.clone();
                if let Some((identity, _, _)) = values.first_mut() {
                    *identity = corrupt_token(identity);
                    CanonicalPayloadValue::Tokens(values)
                } else {
                    foreign.value.clone()
                }
            }
            CanonicalPayloadValue::Range(value) => {
                CanonicalPayloadValue::Range(corrupt_range(value))
            }
            CanonicalPayloadValue::Ranges(values) => {
                let mut values = values.clone();
                if let Some(value) = values.first_mut() {
                    *value = corrupt_range(value);
                    CanonicalPayloadValue::Ranges(values)
                } else {
                    foreign.value.clone()
                }
            }
            CanonicalPayloadValue::Text(value) => CanonicalPayloadValue::Text(format!("{value}!")),
            CanonicalPayloadValue::UInt(value) => CanonicalPayloadValue::UInt(value + 1),
            CanonicalPayloadValue::Int(value) => CanonicalPayloadValue::Int(value + 1),
            CanonicalPayloadValue::Bool(value) => CanonicalPayloadValue::Bool(!value),
            CanonicalPayloadValue::Usize(value) => CanonicalPayloadValue::Usize(value + 1),
            CanonicalPayloadValue::Bools(values) => {
                let mut values = values.clone();
                if let Some(value) = values.first_mut() {
                    *value = !*value;
                } else {
                    values.push(true);
                }
                CanonicalPayloadValue::Bools(values)
            }
            CanonicalPayloadValue::Node(value) => CanonicalPayloadValue::Node(corrupt_node(value)),
            CanonicalPayloadValue::Nodes(values) => {
                let mut values = values.clone();
                if values.len() > 1 {
                    values.reverse();
                } else if let Some(value) = values.first_mut() {
                    *value = corrupt_node(value);
                } else {
                    values.push(corrupt_node(&fact.node));
                }
                CanonicalPayloadValue::Nodes(values)
            }
            CanonicalPayloadValue::OptionalNode(value) => CanonicalPayloadValue::OptionalNode(
                value
                    .as_ref()
                    .map_or_else(|| Some(corrupt_node(&fact.node)), |_| None),
            ),
            CanonicalPayloadValue::DelimiterPair {
                kind,
                open,
                open_range,
                close,
                close_range,
            } => CanonicalPayloadValue::DelimiterPair {
                kind: if *kind == CanonicalDelimiterKind::Parenthesis {
                    CanonicalDelimiterKind::List
                } else {
                    CanonicalDelimiterKind::Parenthesis
                },
                open: corrupt_token(open),
                open_range: open_range.clone(),
                close: close.clone(),
                close_range: close_range.clone(),
            },
            CanonicalPayloadValue::Operator(value) => {
                CanonicalPayloadValue::Operator(if *value == ParsedBinaryOperator::Add {
                    ParsedBinaryOperator::Subtract
                } else {
                    ParsedBinaryOperator::Add
                })
            }
            CanonicalPayloadValue::Permission(value) => CanonicalPayloadValue::Permission(
                if *value == crate::ast::ParamPermission::Borrow {
                    crate::ast::ParamPermission::Consume
                } else {
                    crate::ast::ParamPermission::Borrow
                },
            ),
            CanonicalPayloadValue::Associativity(_) => {
                CanonicalPayloadValue::Associativity(CanonicalAssociativity::Right)
            }
            CanonicalPayloadValue::WrapperKind(value) => {
                CanonicalPayloadValue::WrapperKind(if *value == CanonicalTryWrapperKind::Wrap {
                    CanonicalTryWrapperKind::Propagate
                } else {
                    CanonicalTryWrapperKind::Wrap
                })
            }
        };
        fact
    }

    fn foreign_payload_fact(
        field: F2SealField,
        base: &CanonicalPayloadSealFact,
        foreign: &[CanonicalOccurrenceSeal],
    ) -> CanonicalPayloadSealFact {
        let mut replacement = foreign
            .iter()
            .flat_map(|seal| &seal.payload_projection)
            .find(|fact| f2_field(fact.field) == field && fact != &base)
            .cloned()
            .unwrap_or_else(|| panic!("foreign corpus lacks {field:?}"));
        replacement.node = base.node.clone();
        replacement
    }

    fn mutate_f2_projection(
        seal: &CanonicalOccurrenceSeal,
        foreign: &[CanonicalOccurrenceSeal],
        index: usize,
        mutation: ProjectionMutation,
    ) -> CanonicalOccurrenceSeal {
        let mut mutated = seal.clone();
        let field = f2_field(mutated.payload_projection[index].field);
        let replacement = foreign_payload_fact(field, &mutated.payload_projection[index], foreign);
        match mutation {
            ProjectionMutation::Corrupt => {
                mutated.payload_projection[index] =
                    corrupt_payload_fact(&mutated.payload_projection[index], &replacement);
            }
            ProjectionMutation::Missing => {
                mutated.payload_projection.remove(index);
            }
            ProjectionMutation::Duplicate => {
                mutated
                    .payload_projection
                    .insert(index, mutated.payload_projection[index].clone());
            }
            ProjectionMutation::Reordered => {
                if mutated.payload_projection.len() == 1 {
                    mutated.payload_projection.insert(0, replacement);
                } else {
                    let other = if index + 1 == mutated.payload_projection.len() {
                        index - 1
                    } else {
                        index + 1
                    };
                    mutated.payload_projection.swap(index, other);
                }
            }
            ProjectionMutation::Extra => mutated.payload_projection.push(replacement),
            ProjectionMutation::Substituted => mutated.payload_projection[index] = replacement,
        }
        mutated
    }

    #[derive(Debug, Clone, Copy)]
    enum CombinedField {
        F1(F1SealField),
        F2(F2SealField),
    }

    #[derive(Debug, Clone, Copy)]
    struct CombinedLocation {
        seal: usize,
        index: usize,
        field: CombinedField,
    }

    fn f2_semantic_examples_are_exact(seals: &[CanonicalOccurrenceSeal]) -> bool {
        let unsupported = seals.iter().flat_map(|seal| &seal.projection).any(|fact| {
            matches!(
                fact,
                CanonicalSealFact::Kind(_, CanonicalCommonNodeKind::Unsupported)
            )
        });
        if unsupported {
            return false;
        }
        let payloads = seals
            .iter()
            .flat_map(|seal| &seal.payload_projection)
            .collect::<Vec<_>>();
        let expected_operators = [
            (ParsedBinaryOperator::Multiply, 5, ["*", ""]),
            (ParsedBinaryOperator::Divide, 5, ["/", ""]),
            (ParsedBinaryOperator::Add, 4, ["+", ""]),
            (ParsedBinaryOperator::Subtract, 4, ["-", ""]),
            (ParsedBinaryOperator::Equal, 3, ["==", ""]),
            (ParsedBinaryOperator::NotEqual, 3, ["!=", ""]),
            (ParsedBinaryOperator::Less, 3, ["<", ""]),
            (ParsedBinaryOperator::LessEqual, 3, ["<=", ""]),
            (ParsedBinaryOperator::Greater, 3, [">", ""]),
            (ParsedBinaryOperator::GreaterEqual, 3, [">=", ""]),
            (ParsedBinaryOperator::Is, 3, ["is", ""]),
            (ParsedBinaryOperator::Does, 3, ["does", ""]),
            (ParsedBinaryOperator::Returns, 3, ["returns", ""]),
            (ParsedBinaryOperator::FailsWith, 3, ["fails", "with"]),
            (ParsedBinaryOperator::And, 2, ["and", ""]),
            (ParsedBinaryOperator::Or, 1, ["or", ""]),
        ];
        let operator_nodes = payloads
            .iter()
            .filter_map(|fact| match fact.value {
                CanonicalPayloadValue::Operator(value) => Some((value, &fact.node)),
                _ => None,
            })
            .collect::<Vec<_>>();
        let operator_kinds = operator_nodes
            .iter()
            .map(|(operator, _)| *operator)
            .collect::<std::collections::BTreeSet<_>>();
        let operators_exact = expected_operators.iter().all(|(operator, precedence, tokens)| {
            let Some((_, node)) = operator_nodes
                .iter()
                .find(|(actual, _)| actual == operator)
            else {
                return false;
            };
            let on_node = |field| {
                payloads
                    .iter()
                    .find(|fact| &fact.node == *node && fact.field == field)
                    .map(|fact| &fact.value)
            };
            let expected_tokens = tokens
                .iter()
                .filter(|token| !token.is_empty())
                .copied()
                .collect::<Vec<_>>();
            matches!(
                on_node(CanonicalPayloadField::BinaryPrecedence),
                Some(CanonicalPayloadValue::Usize(actual)) if actual == precedence
            ) && matches!(
                on_node(CanonicalPayloadField::BinaryAssociativity),
                Some(CanonicalPayloadValue::Associativity(CanonicalAssociativity::Left))
            ) && matches!(
                on_node(CanonicalPayloadField::BinaryOperatorTokens),
                Some(CanonicalPayloadValue::Tokens(actual))
                    if actual.iter().map(|(_, _, spelling)| spelling.as_str()).collect::<Vec<_>>() == expected_tokens
            ) && matches!(
                on_node(CanonicalPayloadField::BinaryLeftBoundary),
                Some(CanonicalPayloadValue::Bool(true))
            ) && matches!(
                on_node(CanonicalPayloadField::BinaryRightBoundary),
                Some(CanonicalPayloadValue::Bool(true))
            ) && matches!(
                on_node(CanonicalPayloadField::BinaryReductionOrder),
                Some(CanonicalPayloadValue::Nodes(actual)) if actual.len() == 2 && actual[0] != actual[1]
            ) && matches!(
                on_node(CanonicalPayloadField::BinaryChildRoles),
                Some(CanonicalPayloadValue::Nodes(actual)) if actual.len() == 2 && actual[0] != actual[1]
            )
        });
        let permissions = payloads
            .iter()
            .filter_map(|fact| match fact.value {
                CanonicalPayloadValue::Permission(value) => Some(value.as_str()),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        let wrappers = payloads
            .iter()
            .filter_map(|fact| match fact.value {
                CanonicalPayloadValue::WrapperKind(value) => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();
        let delimiter_kinds = payloads
            .iter()
            .filter_map(|fact| match fact.value {
                CanonicalPayloadValue::DelimiterPair { kind, .. } => Some(kind),
                _ => None,
            })
            .collect::<Vec<_>>();
        let utf8_escape_exact = payloads
            .iter()
            .find(|fact| {
                fact.field == CanonicalPayloadField::TextDecodedValue
                    && matches!(&fact.value, CanonicalPayloadValue::Text(value) if value == "\\é")
            })
            .is_some_and(|decoded| {
                payloads.iter().any(|fact| {
                    fact.node == decoded.node
                        && fact.field == CanonicalPayloadField::TextEscapeEvents
                        && matches!(
                            &fact.value,
                            CanonicalPayloadValue::Tokens(tokens)
                                if tokens.len() == 1 && tokens[0].2 == "\\é"
                        )
                })
            });
        let outer_call_topology_exact = payloads
            .iter()
            .find(|fact| {
                fact.field == CanonicalPayloadField::CallArgumentEdges
                    && matches!(&fact.value, CanonicalPayloadValue::Nodes(nodes) if nodes.len() == 4)
            })
            .is_some_and(|arguments| {
                let on_node = |field| {
                    payloads
                        .iter()
                        .find(|fact| fact.node == arguments.node && fact.field == field)
                        .map(|fact| &fact.value)
                };
                matches!(
                    on_node(CanonicalPayloadField::DelimiterSeparators),
                    Some(CanonicalPayloadValue::Tokens(tokens)) if tokens.len() == 3
                ) && matches!(
                    on_node(CanonicalPayloadField::DelimiterSemanticGaps),
                    Some(CanonicalPayloadValue::Ranges(gaps)) if gaps.len() == 3
                ) && matches!(
                    on_node(CanonicalPayloadField::CallAdjacency),
                    Some(CanonicalPayloadValue::Bools(adjacency))
                        if adjacency.len() == 6 && adjacency.iter().all(|adjacent| *adjacent)
                )
            });
        operators_exact
            && operator_kinds.len() == 16
            && permissions == ["borrow", "change", "consume"].into_iter().collect()
            && wrappers.contains(&CanonicalTryWrapperKind::Wrap)
            && wrappers.contains(&CanonicalTryWrapperKind::Propagate)
            && delimiter_kinds.contains(&CanonicalDelimiterKind::Parenthesis)
            && delimiter_kinds.contains(&CanonicalDelimiterKind::List)
            && delimiter_kinds.contains(&CanonicalDelimiterKind::Record)
            && delimiter_kinds.contains(&CanonicalDelimiterKind::Element)
            && utf8_escape_exact
            && outer_call_topology_exact
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::DelimiterNestingParent
                    && matches!(fact.value, CanonicalPayloadValue::OptionalNode(Some(_)))
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::AggregateEmpty
                    && matches!(fact.value, CanonicalPayloadValue::Bool(true))
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::AggregateTrailing
                    && matches!(fact.value, CanonicalPayloadValue::Bool(true))
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::CallAdjacency
                    && matches!(&fact.value, CanonicalPayloadValue::Bools(values) if values.contains(&true) && values.contains(&false))
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::TextDecodedValue
                    && matches!(&fact.value, CanonicalPayloadValue::Text(value) if value == "hé\"llo")
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::IntSignedLiteral
                    && matches!(fact.value, CanonicalPayloadValue::Bool(true))
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::FieldValue
                    && matches!(&fact.value, CanonicalPayloadValue::Text(value) if value == "field")
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::ElementIndexValue
                    && matches!(fact.value, CanonicalPayloadValue::UInt(0))
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::RecordNameToken
                    && matches!(&fact.value, CanonicalPayloadValue::Tokens(tokens) if tokens.iter().any(|(_, _, spelling)| spelling == "Thing"))
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::TryWrapperRelation
                    && matches!(&fact.value, CanonicalPayloadValue::Tokens(tokens) if tokens.iter().map(|(_, _, spelling)| spelling.as_str()).collect::<Vec<_>>() == ["or", "fail"])
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::TryFailureRootToken
                    && matches!(&fact.value, CanonicalPayloadValue::Tokens(tokens) if tokens.iter().any(|(_, _, spelling)| spelling == "PayloadError"))
            })
            && payloads.iter().any(|fact| {
                fact.field == CanonicalPayloadField::TryFailureVariantToken
                    && matches!(&fact.value, CanonicalPayloadValue::Tokens(tokens) if tokens.iter().any(|(_, _, spelling)| spelling == "context"))
            })
            && payloads.iter().all(|fact| {
                !matches!(&fact.value, CanonicalPayloadValue::Text(value) if value.ends_with('!'))
            })
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct F2Evidence {
        deterministic: bool,
        public_compatible: bool,
        production_valid: bool,
        field_count: usize,
        single_rejections: usize,
        cumulative_pair_rejections: usize,
        foreign_rejections: usize,
        semantics_exact: bool,
        producer_corruption_rejected: bool,
        shared_comutation_caught: bool,
    }

    #[derive(Debug, Clone, Copy)]
    enum F2Sabotage {
        ProducerArm,
        ValidatorArm,
        CatalogueRow,
        MutationOperator,
        PairMutation,
        CrossOccurrence,
    }

    fn f2_evidence(sabotage: Option<F2Sabotage>) -> F2Evidence {
        let parsed = parse_source("f2.hum", F2_SOURCE_A);
        let repeated = parse_source("f2.hum", F2_SOURCE_A);
        let foreign = parse_source("f2.hum", F2_SOURCE_B);
        let f1_foreign = parse_source("f1.hum", F1_SOURCE_B);
        assert_eq!(F2_SOURCE_A.len(), F2_SOURCE_B.len());
        assert_eq!(parsed.file, repeated.file);
        assert_eq!(parsed.diagnostics, repeated.diagnostics);
        let mut f2_locations = std::collections::BTreeMap::new();
        for (seal, occurrence) in parsed.occurrence_seals.iter().enumerate() {
            for (index, fact) in occurrence.payload_projection.iter().enumerate() {
                f2_locations
                    .entry(f2_field(fact.field))
                    .or_insert((seal, index));
            }
        }
        assert_eq!(f2_locations.len(), F2_SEAL_FIELDS.len());
        let mut production_seals = parsed.occurrence_seals.clone();
        if matches!(sabotage, Some(F2Sabotage::ProducerArm)) {
            let producer_location = f2_locations[&F2SealField::IdentifierValue];
            production_seals[producer_location.0]
                .payload_authority
                .remove(producer_location.1);
        }
        let catalogue = if matches!(sabotage, Some(F2Sabotage::CatalogueRow)) {
            &F2_SEAL_FIELDS[..F2_SEAL_FIELDS.len() - 1]
        } else {
            &F2_SEAL_FIELDS[..]
        };
        let mutation_operators = if matches!(sabotage, Some(F2Sabotage::MutationOperator)) {
            &PROJECTION_MUTATIONS[..PROJECTION_MUTATIONS.len() - 1]
        } else {
            &PROJECTION_MUTATIONS[..]
        };
        let mut single_rejections = 0usize;
        let mut foreign_rejections = 0usize;
        for &field in catalogue {
            let (seal, index) = f2_locations[&field];
            for &mutation in mutation_operators {
                let mutated = mutate_f2_projection(
                    &production_seals[seal],
                    &foreign.occurrence_seals,
                    index,
                    mutation,
                );
                let rejected = if matches!(sabotage, Some(F2Sabotage::ValidatorArm))
                    && field == F2SealField::IdentifierValue
                    && matches!(mutation, ProjectionMutation::Corrupt)
                {
                    validate_occurrence_seal_ignoring_one_payload_fact(&mutated, index).is_err()
                } else if matches!(sabotage, Some(F2Sabotage::CrossOccurrence))
                    && field == F2SealField::TryWrapperKind
                    && matches!(mutation, ProjectionMutation::Substituted)
                {
                    let mut shared_foreign = mutated;
                    shared_foreign.payload_authority[index] =
                        shared_foreign.payload_projection[index].clone();
                    validate_occurrence_seal(&shared_foreign).is_err()
                } else {
                    validate_occurrence_seal(&mutated).is_err()
                };
                single_rejections += usize::from(rejected);
                if matches!(mutation, ProjectionMutation::Substituted) {
                    foreign_rejections += usize::from(rejected);
                }
            }
        }
        let f1_base_seal = production_seals
            .iter()
            .enumerate()
            .max_by_key(|(_, seal)| seal.projection.len())
            .map(|(index, _)| index)
            .expect("F2 corpus occurrence");
        let f1_representatives = f1_representatives(&production_seals[f1_base_seal]);
        let mut combined = F1_SEAL_FIELDS
            .iter()
            .map(|field| CombinedLocation {
                seal: f1_base_seal,
                index: f1_representatives[field],
                field: CombinedField::F1(*field),
            })
            .collect::<Vec<_>>();
        combined.extend(F2_SEAL_FIELDS.iter().map(|field| {
            let (seal, index) = f2_locations[field];
            CombinedLocation {
                seal,
                index,
                field: CombinedField::F2(*field),
            }
        }));
        assert_eq!(combined.len(), 100);
        let mut cumulative_pair_rejections = 0usize;
        let pair_limit = if matches!(sabotage, Some(F2Sabotage::PairMutation)) {
            4_949
        } else {
            4_950
        };
        'pairs: for left in 0..combined.len() {
            for right in left + 1..combined.len() {
                if cumulative_pair_rejections == pair_limit {
                    break 'pairs;
                }
                let mut seals = production_seals.clone();
                for location in [combined[left], combined[right]] {
                    match location.field {
                        CombinedField::F1(field) => {
                            seals[location.seal].projection[location.index] = foreign_fact(
                                field,
                                &seals[location.seal].projection[location.index],
                                &f1_foreign.occurrence_seals,
                            );
                        }
                        CombinedField::F2(field) => {
                            seals[location.seal].payload_projection[location.index] =
                                foreign_payload_fact(
                                    field,
                                    &seals[location.seal].payload_projection[location.index],
                                    &foreign.occurrence_seals,
                                );
                        }
                    }
                }
                cumulative_pair_rejections += usize::from(
                    seals
                        .iter()
                        .any(|seal| validate_occurrence_seal(seal).is_err()),
                );
            }
        }
        let producer_location = f2_locations[&F2SealField::IdentifierValue];
        let mut producer_corruption = parsed.occurrence_seals[producer_location.0].clone();
        producer_corruption.payload_authority[producer_location.1] = corrupt_payload_fact(
            &producer_corruption.payload_authority[producer_location.1],
            &foreign.occurrence_seals[producer_location.0].payload_authority[producer_location.1],
        );
        let text_location = f2_locations[&F2SealField::TextDecodedValue];
        let mut shared = parsed.occurrence_seals.clone();
        let replacement = CanonicalPayloadValue::Text("shared-corrupt!".to_string());
        shared[text_location.0].payload_projection[text_location.1].value = replacement.clone();
        shared[text_location.0].payload_authority[text_location.1].value = replacement;
        F2Evidence {
            deterministic: parsed.occurrence_seals == repeated.occurrence_seals,
            public_compatible: parsed.file == repeated.file
                && parsed.diagnostics == repeated.diagnostics,
            production_valid: production_seals
                .iter()
                .all(|seal| validate_occurrence_seal(seal).is_ok()),
            field_count: catalogue.len(),
            single_rejections,
            cumulative_pair_rejections,
            foreign_rejections,
            semantics_exact: f2_semantic_examples_are_exact(&parsed.occurrence_seals),
            producer_corruption_rejected: validate_occurrence_seal(&producer_corruption).is_err(),
            shared_comutation_caught: validate_occurrence_seal(&shared[text_location.0]).is_ok()
                && !f2_semantic_examples_are_exact(&shared),
        }
    }

    #[test]
    fn successful_canonical_expression_payloads_are_complete_and_load_bearing() {
        let first = f2_evidence(None);
        let second = f2_evidence(None);
        assert_eq!(first, second, "fresh F2 inventories must be deterministic");
        assert_eq!(first.field_count, 64);
        assert_eq!(first.single_rejections, 384);
        assert_eq!(first.cumulative_pair_rejections, 4_950);
        assert_eq!(first.foreign_rejections, 64);
        assert!(first.deterministic);
        assert!(first.public_compatible);
        assert!(first.production_valid);
        assert!(first.semantics_exact);
        assert!(first.producer_corruption_rejected);
        assert!(first.shared_comutation_caught);
        for sabotage in [
            F2Sabotage::ProducerArm,
            F2Sabotage::ValidatorArm,
            F2Sabotage::CatalogueRow,
            F2Sabotage::MutationOperator,
            F2Sabotage::PairMutation,
            F2Sabotage::CrossOccurrence,
        ] {
            assert_ne!(
                f2_evidence(Some(sabotage)),
                first,
                "{sabotage:?} must change load-bearing F2 evidence"
            );
        }
    }

    #[test]
    fn f2_utf8_text_escape_payload_is_boundary_safe() {
        let parsed = parse_source(
            "f2-utf8-escape.hum",
            r#"task utf8_escape() -> Text {
  does:
    return "\é"
    return "🙂\éß"
}
"#,
        );
        assert!(parsed.diagnostics.is_empty());
        assert!(
            parsed
                .occurrence_seals
                .iter()
                .all(|seal| validate_occurrence_seal(seal).is_ok())
        );
        for expected in ["\\é", "🙂\\éß"] {
            let decoded = parsed
                .occurrence_seals
                .iter()
                .flat_map(|seal| &seal.payload_projection)
                .find(|fact| {
                    fact.field == CanonicalPayloadField::TextDecodedValue
                        && matches!(&fact.value, CanonicalPayloadValue::Text(value) if value == expected)
                })
                .unwrap_or_else(|| panic!("missing decoded UTF-8 Text payload {expected:?}"));
            let escapes = parsed
                .occurrence_seals
                .iter()
                .flat_map(|seal| &seal.payload_projection)
                .find(|fact| {
                    fact.node == decoded.node
                        && fact.field == CanonicalPayloadField::TextEscapeEvents
                })
                .expect("UTF-8 Text escape events");
            assert!(matches!(
                &escapes.value,
                CanonicalPayloadValue::Tokens(tokens)
                    if tokens.len() == 1 && tokens[0].2 == "\\é"
            ));
        }
    }

    #[test]
    fn f2_malformed_try_is_unsupported_without_authority_mismatch() {
        let parsed = parse_source(
            "f2-malformed-try.hum",
            r#"task malformed() -> UInt {
  does:
    return try source() or fail payloadError.context
}
"#,
        );
        assert!(
            parsed
                .occurrence_seals
                .iter()
                .all(|seal| validate_occurrence_seal(seal).is_ok())
        );
        let malformed = parsed
            .occurrence_seals
            .iter()
            .find(|seal| {
                seal.projection.iter().any(|fact| {
                    matches!(
                        fact,
                        CanonicalSealFact::Kind(_, CanonicalCommonNodeKind::Unsupported)
                    )
                })
            })
            .expect("malformed try occurrence");
        assert!(malformed.payload_projection.is_empty());
        assert!(malformed.payload_authority.is_empty());
        assert_eq!(malformed.projection, malformed.authority);
        assert!(malformed.projection.iter().any(|fact| {
            matches!(
                fact,
                CanonicalSealFact::LexicalStatus(_, CanonicalCommonLexicalStatus::Unsupported)
            )
        }));
    }

    #[test]
    fn f2_topology_channels_ignore_nested_and_quoted_tokens() {
        let parsed = parse_source(
            "f2-topology.hum",
            r#"task topology(value: UInt, other: UInt) -> UInt {
  does:
    return call("a, b", nested(value, other), [value, other], value)
}
"#,
        );
        let (seal_index, fact_index) = parsed
            .occurrence_seals
            .iter()
            .enumerate()
            .find_map(|(seal_index, seal)| {
                seal.payload_projection
                    .iter()
                    .position(|fact| {
                        fact.field == CanonicalPayloadField::CallArgumentEdges
                            && matches!(&fact.value, CanonicalPayloadValue::Nodes(nodes) if nodes.len() == 4)
                    })
                    .map(|fact_index| (seal_index, fact_index))
            })
            .expect("outer four-argument call");
        let seal = &parsed.occurrence_seals[seal_index];
        let call_node = &seal.payload_projection[fact_index].node;
        let fact = |field| {
            seal.payload_projection
                .iter()
                .find(|fact| &fact.node == call_node && fact.field == field)
                .unwrap_or_else(|| panic!("missing outer-call payload {field:?}"))
        };
        assert!(matches!(
            &fact(CanonicalPayloadField::DelimiterSeparators).value,
            CanonicalPayloadValue::Tokens(tokens) if tokens.len() == 3
        ));
        assert!(matches!(
            &fact(CanonicalPayloadField::DelimiterSemanticGaps).value,
            CanonicalPayloadValue::Ranges(gaps) if gaps.len() == 3
        ));
        assert!(matches!(
            &fact(CanonicalPayloadField::CallAdjacency).value,
            CanonicalPayloadValue::Bools(adjacency)
                if adjacency.len() == 6 && adjacency.iter().all(|adjacent| *adjacent)
        ));

        let topology_index = seal
            .payload_projection
            .iter()
            .position(|fact| {
                &fact.node == call_node && fact.field == CanonicalPayloadField::CallAdjacency
            })
            .expect("outer-call adjacency");
        let mut projected_corruption = seal.clone();
        projected_corruption.payload_projection[topology_index]
            .identity
            .0 = corrupt_owner_identity(
            &projected_corruption.payload_projection[topology_index]
                .identity
                .0,
        );
        assert!(validate_occurrence_seal(&projected_corruption).is_err());
        let mut retained_corruption = seal.clone();
        retained_corruption.payload_authority[topology_index]
            .identity
            .0 = corrupt_owner_identity(
            &retained_corruption.payload_authority[topology_index]
                .identity
                .0,
        );
        assert!(validate_occurrence_seal(&retained_corruption).is_err());
    }

    const F3_MALFORMED_FIELDS: [CanonicalMalformedSealField; 8] = [
        CanonicalMalformedSealField::Status,
        CanonicalMalformedSealField::Node,
        CanonicalMalformedSealField::Cause,
        CanonicalMalformedSealField::ProducingEvent,
        CanonicalMalformedSealField::OffendingRange,
        CanonicalMalformedSealField::ConsumedRange,
        CanonicalMalformedSealField::ExpectedEvidence,
        CanonicalMalformedSealField::ActualEvidence,
    ];

    const F3_STATEMENT_FIELDS: [CanonicalStatementEventField; 24] = [
        CanonicalStatementEventField::Kind,
        CanonicalStatementEventField::Section,
        CanonicalStatementEventField::Line,
        CanonicalStatementEventField::Statement,
        CanonicalStatementEventField::Keyword,
        CanonicalStatementEventField::PhraseTokens,
        CanonicalStatementEventField::Binder,
        CanonicalStatementEventField::BinderRelationship,
        CanonicalStatementEventField::TypeBoundary,
        CanonicalStatementEventField::AssignmentToken,
        CanonicalStatementEventField::RelationshipToken,
        CanonicalStatementEventField::TargetRoot,
        CanonicalStatementEventField::ValueRoot,
        CanonicalStatementEventField::DestinationToken,
        CanonicalStatementEventField::StartRoot,
        CanonicalStatementEventField::EndRoot,
        CanonicalStatementEventField::OrderedRoots,
        CanonicalStatementEventField::BlockOwner,
        CanonicalStatementEventField::BlockDepthBefore,
        CanonicalStatementEventField::BlockDepthAfter,
        CanonicalStatementEventField::BlockRelationship,
        CanonicalStatementEventField::BlockOpenToken,
        CanonicalStatementEventField::BlockCloseToken,
        CanonicalStatementEventField::ExpressionAbsent,
    ];

    fn f3_malformed_source() -> String {
        let deep = format!("{}value{}", "(".repeat(17), ")".repeat(17));
        format!(
            r#"task malformed(value: UInt) -> UInt {{
  needs:
    ["ok" "bad"]
    ["ok",]
    [7]
  does:
    return "oops
    return call(value
    return [value)
    return {deep}
    return value +
    return value === other
    return @value
    return value.
    return 9223372036854775808
}}
"#
        )
    }

    const F3_STATEMENT_SOURCE: &str = r#"task relationships(value: UInt, other: UInt) -> UInt {
  needs:
    value
  ensures:
    value
  does:
    return value
    let item: UInt = value
    change mutable: UInt = other
    set target = value
    save value in items
    fail value
    expect value
    free_call(value)
    if value {
      return value
    }
    while value {
      set target = value
    }
    for each element in values {
      return element
    }
    for index index_a from 0 until 4 {
      return index_a
    }
    for index index_b from 0 through 4 {
      return index_b
    }
    loop {
      return value
    }
}
"#;

    fn corrupt_malformed_fact(fact: &CanonicalMalformedSealFact) -> CanonicalMalformedSealFact {
        let mut fact = fact.clone();
        fact.value = match &fact.value {
            CanonicalMalformedSealValue::Unsupported => {
                CanonicalMalformedSealValue::Cause(CanonicalMalformedCause::MissingOperand)
            }
            CanonicalMalformedSealValue::Node(identity) => CanonicalMalformedSealValue::Node(
                CanonicalNodeIdentity(corrupt_owner_identity(&identity.0)),
            ),
            CanonicalMalformedSealValue::Cause(cause) => CanonicalMalformedSealValue::Cause(
                if *cause == CanonicalMalformedCause::MissingOperand {
                    CanonicalMalformedCause::InvalidOperandStarter
                } else {
                    CanonicalMalformedCause::MissingOperand
                },
            ),
            CanonicalMalformedSealValue::ProducingEvent(identity, range) => {
                CanonicalMalformedSealValue::ProducingEvent(
                    super::CanonicalMalformedEventIdentity(corrupt_owner_identity(&identity.0)),
                    range.clone(),
                )
            }
            CanonicalMalformedSealValue::Range(range) => {
                CanonicalMalformedSealValue::Range(corrupt_range(range))
            }
            CanonicalMalformedSealValue::Expected(value) => CanonicalMalformedSealValue::Expected(
                if matches!(value, CanonicalExpectedLexicalEvidence::Operand) {
                    CanonicalExpectedLexicalEvidence::Identifier
                } else {
                    CanonicalExpectedLexicalEvidence::Operand
                },
            ),
            CanonicalMalformedSealValue::Actual(value) => CanonicalMalformedSealValue::Actual(
                if matches!(value, CanonicalActualLexicalEvidence::EndOfInput) {
                    CanonicalActualLexicalEvidence::DelimiterDepth(99)
                } else {
                    CanonicalActualLexicalEvidence::EndOfInput
                },
            ),
        };
        fact
    }

    fn corrupt_statement_fact(fact: &CanonicalStatementSealFact) -> CanonicalStatementSealFact {
        let mut fact = fact.clone();
        fact.value = match &fact.value {
            CanonicalStatementSealValue::Kind(kind) => {
                CanonicalStatementSealValue::Kind(if *kind == CanonicalStatementKindEvent::Return {
                    CanonicalStatementKindEvent::UnconditionalLoop
                } else {
                    CanonicalStatementKindEvent::Return
                })
            }
            CanonicalStatementSealValue::Section(section) => CanonicalStatementSealValue::Section(
                CanonicalSectionOwner(corrupt_owner_identity(&section.0)),
            ),
            CanonicalStatementSealValue::Statement(statement) => {
                CanonicalStatementSealValue::Statement(CanonicalStatementOwner(
                    corrupt_owner_identity(&statement.0),
                ))
            }
            CanonicalStatementSealValue::Range(range) => {
                CanonicalStatementSealValue::Range(corrupt_range(range))
            }
            CanonicalStatementSealValue::Token(identity, range, spelling) => {
                CanonicalStatementSealValue::Token(
                    super::CanonicalStatementTokenIdentity(corrupt_owner_identity(&identity.0)),
                    range.clone(),
                    spelling.clone(),
                )
            }
            CanonicalStatementSealValue::Tokens(values) => {
                let mut values = values.clone();
                if let Some((identity, _, _)) = values.first_mut() {
                    identity.0 = corrupt_owner_identity(&identity.0);
                }
                CanonicalStatementSealValue::Tokens(values)
            }
            CanonicalStatementSealValue::TokenReference(identity) => {
                CanonicalStatementSealValue::TokenReference(super::CanonicalStatementTokenIdentity(
                    corrupt_owner_identity(&identity.0),
                ))
            }
            CanonicalStatementSealValue::Root(identity, ordinal, node) => {
                CanonicalStatementSealValue::Root(
                    corrupt_occurrence(identity),
                    *ordinal,
                    node.clone(),
                )
            }
            CanonicalStatementSealValue::Roots(values) => {
                let mut values = values.clone();
                if let Some((identity, _, _)) = values.first_mut() {
                    *identity = corrupt_occurrence(identity);
                }
                CanonicalStatementSealValue::Roots(values)
            }
            CanonicalStatementSealValue::Block(identity) => CanonicalStatementSealValue::Block(
                CanonicalStatementBlockIdentity(corrupt_owner_identity(&identity.0)),
            ),
            CanonicalStatementSealValue::Usize(value) => {
                CanonicalStatementSealValue::Usize(value + 1)
            }
            CanonicalStatementSealValue::Bool(value) => CanonicalStatementSealValue::Bool(!value),
            CanonicalStatementSealValue::BlockRelationship(value) => {
                CanonicalStatementSealValue::BlockRelationship(match value {
                    ParsedBlockRelationship::None => ParsedBlockRelationship::Opens,
                    ParsedBlockRelationship::Opens => ParsedBlockRelationship::Closes,
                    ParsedBlockRelationship::Closes => ParsedBlockRelationship::None,
                })
            }
        };
        fact
    }

    fn foreign_malformed_fact(
        base: &CanonicalMalformedSealFact,
        foreign: &[CanonicalOccurrenceSeal],
    ) -> CanonicalMalformedSealFact {
        foreign
            .iter()
            .flat_map(|seal| &seal.malformed_projection)
            .find(|fact| fact.field == base.field && *fact != base)
            .cloned()
            .unwrap_or_else(|| corrupt_malformed_fact(base))
    }

    fn foreign_statement_fact(
        base: &CanonicalStatementSealFact,
        foreign: &[CanonicalStatementSeal],
    ) -> CanonicalStatementSealFact {
        foreign
            .iter()
            .flat_map(|seal| &seal.projection)
            .find(|fact| fact.field == base.field && *fact != base)
            .cloned()
            .unwrap_or_else(|| corrupt_statement_fact(base))
    }

    fn mutate_malformed_projection(
        seal: &CanonicalOccurrenceSeal,
        foreign: &[CanonicalOccurrenceSeal],
        index: usize,
        mutation: ProjectionMutation,
    ) -> CanonicalOccurrenceSeal {
        let mut seal = seal.clone();
        let replacement = foreign_malformed_fact(&seal.malformed_projection[index], foreign);
        match mutation {
            ProjectionMutation::Corrupt => {
                seal.malformed_projection[index] =
                    corrupt_malformed_fact(&seal.malformed_projection[index]);
            }
            ProjectionMutation::Missing => {
                seal.malformed_projection.remove(index);
            }
            ProjectionMutation::Duplicate => {
                seal.malformed_projection
                    .insert(index, seal.malformed_projection[index].clone());
            }
            ProjectionMutation::Reordered => {
                let other = if index + 1 == seal.malformed_projection.len() {
                    index - 1
                } else {
                    index + 1
                };
                seal.malformed_projection.swap(index, other);
            }
            ProjectionMutation::Extra => seal.malformed_projection.push(replacement),
            ProjectionMutation::Substituted => seal.malformed_projection[index] = replacement,
        }
        seal
    }

    fn mutate_statement_projection(
        seal: &CanonicalStatementSeal,
        foreign: &[CanonicalStatementSeal],
        index: usize,
        mutation: ProjectionMutation,
    ) -> CanonicalStatementSeal {
        let mut seal = seal.clone();
        let replacement = foreign_statement_fact(&seal.projection[index], foreign);
        match mutation {
            ProjectionMutation::Corrupt => {
                seal.projection[index] = corrupt_statement_fact(&seal.projection[index]);
            }
            ProjectionMutation::Missing => {
                seal.projection.remove(index);
            }
            ProjectionMutation::Duplicate => {
                seal.projection
                    .insert(index, seal.projection[index].clone());
            }
            ProjectionMutation::Reordered => {
                let other = if index + 1 == seal.projection.len() {
                    index - 1
                } else {
                    index + 1
                };
                seal.projection.swap(index, other);
            }
            ProjectionMutation::Extra => seal.projection.push(replacement),
            ProjectionMutation::Substituted => seal.projection[index] = replacement,
        }
        seal
    }

    #[derive(Debug, Clone, Copy)]
    enum F3CombinedLocation {
        F1 {
            seal: usize,
            index: usize,
            field: F1SealField,
        },
        F2 {
            seal: usize,
            index: usize,
            field: F2SealField,
        },
        Malformed {
            seal: usize,
            index: usize,
        },
        Statement {
            seal: usize,
            index: usize,
        },
    }

    #[derive(Debug, Clone, Copy)]
    enum F3Sabotage {
        ProducerArm,
        ValidatorArm,
        CatalogueRow,
        MutationOperator,
        PairCase,
        SubstitutionCase,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct F3Evidence {
        deterministic: bool,
        production_valid: bool,
        complete_is_absence: bool,
        malformed_cause_count: usize,
        statement_kind_count: usize,
        field_count: usize,
        single_rejections: usize,
        foreign_rejections: usize,
        cumulative_pair_rejections: usize,
        semantic_comutation_rejections: usize,
    }

    fn semantic_comutation_rejected(
        result: Result<(), &'static str>,
        label: &'static str,
    ) -> usize {
        assert!(result.is_err(), "{label} coherent co-mutation stayed green");
        1
    }

    fn f3_evidence(sabotage: Option<F3Sabotage>) -> F3Evidence {
        let malformed_source = f3_malformed_source();
        let malformed = parse_source("f3-malformed.hum", &malformed_source);
        let repeated = parse_source("f3-malformed.hum", &malformed_source);
        let foreign_malformed_source = malformed_source.replacen("malformed", "malformex", 1);
        let foreign_malformed = parse_source("f3-malformed.hum", &foreign_malformed_source);
        let statements = parse_source("f3-statements.hum", F3_STATEMENT_SOURCE);
        let foreign_statement_source =
            F3_STATEMENT_SOURCE.replacen("relationships", "relationshixs", 1);
        let foreign_statements = parse_source("f3-statements.hum", &foreign_statement_source);
        let control_flow = parse_source(
            "examples/control_flow.hum",
            include_str!("../examples/control_flow.hum"),
        );
        let f2 = parse_source("f3-f2.hum", F2_SOURCE_A);
        let foreign_f2 = parse_source("f3-f2.hum", F2_SOURCE_B);
        let foreign_f1 = parse_source("f3-f1.hum", F1_SOURCE_B);

        let mut malformed_locations = std::collections::BTreeMap::new();
        for (seal, occurrence) in malformed.occurrence_seals.iter().enumerate() {
            for (index, fact) in occurrence.malformed_projection.iter().enumerate() {
                malformed_locations
                    .entry(fact.field)
                    .or_insert((seal, index));
            }
        }
        let mut statement_locations = std::collections::BTreeMap::new();
        for (seal, statement) in statements.statement_seals.iter().enumerate() {
            for (index, fact) in statement.projection.iter().enumerate() {
                statement_locations
                    .entry(fact.field)
                    .or_insert((seal, index));
            }
        }
        let malformed_catalogue = if matches!(sabotage, Some(F3Sabotage::CatalogueRow)) {
            &F3_MALFORMED_FIELDS[..F3_MALFORMED_FIELDS.len() - 1]
        } else {
            &F3_MALFORMED_FIELDS[..]
        };
        let statement_catalogue = &F3_STATEMENT_FIELDS[..];
        let mutations = if matches!(sabotage, Some(F3Sabotage::MutationOperator)) {
            &PROJECTION_MUTATIONS[..PROJECTION_MUTATIONS.len() - 1]
        } else {
            &PROJECTION_MUTATIONS[..]
        };
        let mut single_rejections = 0usize;
        let mut foreign_rejections = 0usize;
        for field in malformed_catalogue {
            let (seal, index) = malformed_locations[field];
            for mutation in mutations {
                let rejected = validate_occurrence_seal(&mutate_malformed_projection(
                    &malformed.occurrence_seals[seal],
                    &foreign_malformed.occurrence_seals,
                    index,
                    *mutation,
                ))
                .is_err();
                let credited = !(matches!(sabotage, Some(F3Sabotage::ValidatorArm))
                    && *field == CanonicalMalformedSealField::Status
                    && matches!(mutation, ProjectionMutation::Corrupt))
                    && rejected;
                single_rejections += usize::from(credited);
                if matches!(mutation, ProjectionMutation::Substituted) {
                    let credited = !(matches!(sabotage, Some(F3Sabotage::SubstitutionCase))
                        && *field == CanonicalMalformedSealField::Status)
                        && rejected;
                    foreign_rejections += usize::from(credited);
                }
            }
        }
        for field in statement_catalogue {
            let (seal, index) = statement_locations[field];
            for mutation in mutations {
                let rejected = validate_statement_seal(&mutate_statement_projection(
                    &statements.statement_seals[seal],
                    &foreign_statements.statement_seals,
                    index,
                    *mutation,
                ))
                .is_err();
                single_rejections += usize::from(rejected);
                if matches!(mutation, ProjectionMutation::Substituted) {
                    foreign_rejections += usize::from(rejected);
                }
            }
        }

        let f1_base = f2
            .occurrence_seals
            .iter()
            .enumerate()
            .max_by_key(|(_, seal)| seal.projection.len())
            .map(|(index, _)| index)
            .expect("F3 cumulative F1 occurrence");
        let f1_representatives = f1_representatives(&f2.occurrence_seals[f1_base]);
        let mut f2_locations = std::collections::BTreeMap::new();
        for (seal, occurrence) in f2.occurrence_seals.iter().enumerate() {
            for (index, fact) in occurrence.payload_projection.iter().enumerate() {
                f2_locations
                    .entry(f2_field(fact.field))
                    .or_insert((seal, index));
            }
        }
        let mut combined = F1_SEAL_FIELDS
            .iter()
            .map(|field| F3CombinedLocation::F1 {
                seal: f1_base,
                index: f1_representatives[field],
                field: *field,
            })
            .collect::<Vec<_>>();
        combined.extend(F2_SEAL_FIELDS.iter().map(|field| {
            let (seal, index) = f2_locations[field];
            F3CombinedLocation::F2 {
                seal,
                index,
                field: *field,
            }
        }));
        combined.extend(malformed_catalogue.iter().map(|field| {
            let (seal, index) = malformed_locations[field];
            F3CombinedLocation::Malformed { seal, index }
        }));
        combined.extend(statement_catalogue.iter().map(|field| {
            let (seal, index) = statement_locations[field];
            F3CombinedLocation::Statement { seal, index }
        }));
        let expected_pair_count = combined.len() * (combined.len() - 1) / 2;
        let pair_limit = if matches!(sabotage, Some(F3Sabotage::PairCase)) {
            expected_pair_count - 1
        } else {
            expected_pair_count
        };
        let mut cumulative_pair_rejections = 0usize;
        'pairs: for left in 0..combined.len() {
            for right in left + 1..combined.len() {
                if cumulative_pair_rejections == pair_limit {
                    break 'pairs;
                }
                let mut f2_seals = f2.occurrence_seals.clone();
                let mut malformed_seals = malformed.occurrence_seals.clone();
                let mut statement_seals = statements.statement_seals.clone();
                for location in [combined[left], combined[right]] {
                    match location {
                        F3CombinedLocation::F1 { seal, index, field } => {
                            f2_seals[seal].projection[index] = foreign_fact(
                                field,
                                &f2_seals[seal].projection[index],
                                &foreign_f1.occurrence_seals,
                            );
                        }
                        F3CombinedLocation::F2 { seal, index, field } => {
                            f2_seals[seal].payload_projection[index] = foreign_payload_fact(
                                field,
                                &f2_seals[seal].payload_projection[index],
                                &foreign_f2.occurrence_seals,
                            );
                        }
                        F3CombinedLocation::Malformed { seal, index } => {
                            malformed_seals[seal].malformed_projection[index] =
                                foreign_malformed_fact(
                                    &malformed_seals[seal].malformed_projection[index],
                                    &foreign_malformed.occurrence_seals,
                                );
                        }
                        F3CombinedLocation::Statement { seal, index } => {
                            statement_seals[seal].projection[index] = foreign_statement_fact(
                                &statement_seals[seal].projection[index],
                                &foreign_statements.statement_seals,
                            );
                        }
                    }
                }
                let rejected = f2_seals
                    .iter()
                    .any(|seal| validate_occurrence_seal(seal).is_err())
                    || malformed_seals
                        .iter()
                        .any(|seal| validate_occurrence_seal(seal).is_err())
                    || statement_seals
                        .iter()
                        .any(|seal| validate_statement_seal(seal).is_err());
                cumulative_pair_rejections += usize::from(rejected);
            }
        }

        let mut semantic_comutation_rejections = 0usize;
        let semantic_seal = |cause| {
            malformed
                .occurrence_seals
                .iter()
                .find(|seal| {
                    seal.malformed_projection.iter().any(|fact| {
                        matches!(fact.value, CanonicalMalformedSealValue::Cause(actual) if actual == cause)
                    })
                })
                .expect("F3 malformed semantic case")
                .clone()
        };
        let mut cause = semantic_seal(CanonicalMalformedCause::InvalidComparisonOperator);
        for facts in [
            &mut cause.malformed_projection,
            &mut cause.malformed_authority,
        ] {
            facts[2].value =
                CanonicalMalformedSealValue::Cause(CanonicalMalformedCause::MissingOperand);
            facts[6].value =
                CanonicalMalformedSealValue::Expected(CanonicalExpectedLexicalEvidence::Operand);
            facts[7].value =
                CanonicalMalformedSealValue::Actual(CanonicalActualLexicalEvidence::EndOfInput);
        }
        semantic_comutation_rejections +=
            semantic_comutation_rejected(validate_occurrence_seal(&cause), "cause/evidence");
        let mut depth = semantic_seal(CanonicalMalformedCause::DelimiterDepthExceeded);
        for facts in [
            &mut depth.malformed_projection,
            &mut depth.malformed_authority,
        ] {
            facts[6].value = CanonicalMalformedSealValue::Expected(
                CanonicalExpectedLexicalEvidence::MaximumDelimiterDepth(17),
            );
            facts[7].value = CanonicalMalformedSealValue::Actual(
                CanonicalActualLexicalEvidence::DelimiterDepth(18),
            );
        }
        semantic_comutation_rejections +=
            semantic_comutation_rejected(validate_occurrence_seal(&depth), "maximum depth");
        let mut partial = semantic_seal(CanonicalMalformedCause::MissingOperand);
        for facts in [
            &mut partial.malformed_projection,
            &mut partial.malformed_authority,
        ] {
            facts[3] = corrupt_malformed_fact(&facts[3]);
        }
        semantic_comutation_rejections +=
            semantic_comutation_rejected(validate_occurrence_seal(&partial), "partial reduction");

        let statement_with = |kind| {
            statements
                .statement_seals
                .iter()
                .find(|seal| {
                    matches!(
                        seal.projection.first().map(|fact| &fact.value),
                        Some(CanonicalStatementSealValue::Kind(actual)) if *actual == kind
                    )
                })
                .expect("F3 statement semantic case")
                .clone()
        };
        let mut binder = statement_with(CanonicalStatementKindEvent::ForEach);
        let binder_index = binder
            .projection
            .iter()
            .position(|fact| fact.field == CanonicalStatementEventField::Binder)
            .expect("binder fact");
        let reference_index = binder
            .projection
            .iter()
            .position(|fact| fact.field == CanonicalStatementEventField::BinderRelationship)
            .expect("binder relationship");
        for facts in [&mut binder.projection, &mut binder.authority] {
            let corrupted_identity = {
                let CanonicalStatementSealValue::Token(identity, _, _) =
                    &mut facts[binder_index].value
                else {
                    unreachable!()
                };
                identity.0 = corrupt_owner_identity(&identity.0);
                identity.0.clone()
            };
            let CanonicalStatementSealValue::TokenReference(reference) =
                &mut facts[reference_index].value
            else {
                unreachable!()
            };
            reference.0 = corrupted_identity;
        }
        semantic_comutation_rejections +=
            semantic_comutation_rejected(validate_statement_seal(&binder), "binder relationship");

        let mut relation = statement_with(CanonicalStatementKindEvent::ForIndexUntil);
        for facts in [&mut relation.projection, &mut relation.authority] {
            let CanonicalStatementSealValue::Kind(kind) = &mut facts[0].value else {
                unreachable!()
            };
            *kind = CanonicalStatementKindEvent::ForIndexThrough;
            let fact = facts
                .iter_mut()
                .find(|fact| {
                    fact.field == CanonicalStatementEventField::RelationshipToken
                        && matches!(&fact.value, CanonicalStatementSealValue::Token(_, _, spelling) if spelling == "until")
                })
                .expect("until token");
            let CanonicalStatementSealValue::Token(_, range, spelling) = &mut fact.value else {
                unreachable!()
            };
            range.byte_len = "through".len();
            *spelling = "through".to_string();
        }
        semantic_comutation_rejections +=
            semantic_comutation_rejected(validate_statement_seal(&relation), "loop relationship");

        let mut bounds = statement_with(CanonicalStatementKindEvent::ForIndexUntil);
        for facts in [&mut bounds.projection, &mut bounds.authority] {
            let start = facts
                .iter()
                .position(|fact| fact.field == CanonicalStatementEventField::StartRoot)
                .unwrap();
            let end = facts
                .iter()
                .position(|fact| fact.field == CanonicalStatementEventField::EndRoot)
                .unwrap();
            let start_value = facts[start].value.clone();
            facts[start].value = facts[end].value.clone();
            facts[end].value = start_value;
        }
        semantic_comutation_rejections +=
            semantic_comutation_rejected(validate_statement_seal(&bounds), "loop bounds");

        let mut block = statement_with(CanonicalStatementKindEvent::BlockClose);
        for facts in [&mut block.projection, &mut block.authority] {
            let fact = facts
                .iter_mut()
                .find(|fact| fact.field == CanonicalStatementEventField::BlockOwner)
                .unwrap();
            let CanonicalStatementSealValue::Block(identity) = &mut fact.value else {
                unreachable!()
            };
            identity.0 = corrupt_owner_identity(&identity.0);
        }
        semantic_comutation_rejections +=
            semantic_comutation_rejected(validate_statement_seal(&block), "block owner");

        let mut producer_statements = statements.statement_seals.clone();
        if matches!(sabotage, Some(F3Sabotage::ProducerArm)) {
            producer_statements[0].authority[0] =
                corrupt_statement_fact(&producer_statements[0].authority[0]);
        }
        let causes = malformed
            .occurrence_seals
            .iter()
            .flat_map(|seal| &seal.malformed_projection)
            .filter_map(|fact| match fact.value {
                CanonicalMalformedSealValue::Cause(cause) => Some(cause),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        let kinds = statements
            .statement_seals
            .iter()
            .filter_map(
                |seal| match seal.projection.first().map(|fact| &fact.value) {
                    Some(CanonicalStatementSealValue::Kind(kind)) => Some(*kind),
                    _ => None,
                },
            )
            .collect::<std::collections::BTreeSet<_>>();
        F3Evidence {
            deterministic: malformed.occurrence_seals == repeated.occurrence_seals,
            production_valid: malformed
                .occurrence_seals
                .iter()
                .all(|seal| validate_occurrence_seal(seal).is_ok())
                && producer_statements
                    .iter()
                    .all(|seal| validate_statement_seal(seal).is_ok())
                && control_flow
                    .statement_seals
                    .iter()
                    .all(|seal| validate_statement_seal(seal).is_ok()),
            complete_is_absence: f2.occurrence_seals.iter().all(|seal| {
                seal.malformed_projection.is_empty() && seal.malformed_authority.is_empty()
            }),
            malformed_cause_count: causes.len(),
            statement_kind_count: kinds.len(),
            field_count: malformed_catalogue.len() + statement_catalogue.len(),
            single_rejections,
            foreign_rejections,
            cumulative_pair_rejections,
            semantic_comutation_rejections,
        }
    }

    fn complete_f3_evidence(evidence: &F3Evidence) -> bool {
        evidence.deterministic
            && evidence.production_valid
            && evidence.complete_is_absence
            && evidence.malformed_cause_count == 12
            && evidence.statement_kind_count == 17
            && evidence.field_count == 32
            && evidence.single_rejections == 192
            && evidence.foreign_rejections == 32
            && evidence.cumulative_pair_rejections == 8_646
            && evidence.semantic_comutation_rejections == 7
    }

    fn f3_reviewer_regressions() -> usize {
        let loop_program = parse_source(
            "f3-loop-phrases.hum",
            r#"task loop_phrases(value: UInt) -> UInt {
  does:
    for index quoted from 0 until parse(" through ") {
      return quoted
    }
    for index actual from 0 through 4 {
      return actual
    }
    return value
}
"#,
        );
        let loop_kinds = loop_program
            .statement_seals
            .iter()
            .filter_map(|seal| {
                let line = super::statement_source_line(seal)?;
                line.starts_with("for index").then(|| {
                    seal.projection.iter().find_map(|fact| match fact.value {
                        CanonicalStatementSealValue::Kind(kind) => Some(kind),
                        _ => None,
                    })
                })?
            })
            .collect::<Vec<_>>();
        assert_eq!(
            loop_kinds,
            [
                CanonicalStatementKindEvent::ForIndexUntil,
                CanonicalStatementKindEvent::ForIndexThrough,
            ],
            "quoted through text must not select the through relationship"
        );
        assert!(
            loop_program
                .statement_seals
                .iter()
                .all(|seal| validate_statement_seal(seal).is_ok()),
            "top-level loop phrase controls must remain valid"
        );

        let statements = parse_source("f3-statements.hum", F3_STATEMENT_SOURCE);
        let mut free_root = statements
            .statement_seals
            .iter()
            .find(|seal| {
                super::statement_source_line(seal)
                    .is_some_and(|line| line.trim() == "free_call(value)")
            })
            .expect("rooted free-expression statement")
            .clone();
        for facts in [&mut free_root.projection, &mut free_root.authority] {
            facts.retain(|fact| fact.field != CanonicalStatementEventField::ValueRoot);
            let ordered = facts
                .iter_mut()
                .find(|fact| fact.field == CanonicalStatementEventField::OrderedRoots)
                .expect("free-expression ordered roots");
            ordered.value = CanonicalStatementSealValue::Roots(Vec::new());
        }
        assert!(
            validate_statement_seal(&free_root).is_err(),
            "coherent rooted-free-expression erasure stayed green"
        );

        let mut for_each = statements
            .statement_seals
            .iter()
            .find(|seal| {
                super::statement_source_line(seal)
                    .is_some_and(|line| line.starts_with("for each element"))
            })
            .expect("for-each statement")
            .clone();
        for facts in [&mut for_each.projection, &mut for_each.authority] {
            facts.retain(|fact| {
                !matches!(
                    fact.field,
                    CanonicalStatementEventField::Binder
                        | CanonicalStatementEventField::BinderRelationship
                )
            });
        }
        assert!(
            validate_statement_seal(&for_each).is_err(),
            "coherent for-each binder erasure stayed green"
        );

        let blocks = parse_source(
            "f3-block-authority.hum",
            r#"task block_authority(value: UInt) -> UInt {
  does:
    if value {
      return value
    }
    if value {
      return value
    }
    return value
}
"#,
        );
        let nested = blocks
            .statement_seals
            .iter()
            .find(|seal| {
                super::statement_source_line(seal).is_some_and(|line| line.trim() == "return value")
                    && seal.projection.iter().any(|fact| {
                        fact.field == CanonicalStatementEventField::BlockDepthBefore
                            && fact.value == CanonicalStatementSealValue::Usize(1)
                    })
            })
            .expect("nested return statement")
            .clone();
        let mut depth = nested.clone();
        for facts in [&mut depth.projection, &mut depth.authority] {
            for fact in facts.iter_mut().filter(|fact| {
                matches!(
                    fact.field,
                    CanonicalStatementEventField::BlockDepthBefore
                        | CanonicalStatementEventField::BlockDepthAfter
                )
            }) {
                let CanonicalStatementSealValue::Usize(value) = &mut fact.value else {
                    unreachable!()
                };
                *value += 1;
            }
        }
        assert!(
            validate_statement_seal(&depth).is_err(),
            "coherent nested block-depth rewrite stayed green"
        );
        let sibling = blocks
            .statement_seals
            .iter()
            .filter(|seal| {
                super::statement_source_line(seal).is_some_and(|line| line.trim() == "return value")
                    && seal.projection.iter().any(|fact| {
                        fact.field == CanonicalStatementEventField::BlockDepthBefore
                            && fact.value == CanonicalStatementSealValue::Usize(1)
                    })
            })
            .nth(1)
            .expect("same-shaped sibling return statement");
        let sibling_owner = sibling
            .projection
            .iter()
            .find(|fact| fact.field == CanonicalStatementEventField::BlockOwner)
            .expect("sibling block owner")
            .value
            .clone();
        let mut owner = nested;
        for facts in [&mut owner.projection, &mut owner.authority] {
            facts
                .iter_mut()
                .find(|fact| fact.field == CanonicalStatementEventField::BlockOwner)
                .expect("nested block owner")
                .value = sibling_owner.clone();
        }
        assert!(
            validate_statement_seal(&owner).is_err(),
            "same-shaped sibling block-owner substitution stayed green"
        );

        let utf8 = parse_source(
            "f3-utf8-malformed.hum",
            "task malformed_utf8(value: UInt) -> UInt {\n  does:\n    return \"é\" === value\n}\n",
        );
        assert!(
            utf8.occurrence_seals
                .iter()
                .all(|seal| validate_occurrence_seal(seal).is_ok()),
            "UTF-8 byte ranges must bind malformed evidence to source"
        );
        let mut equal_length = utf8
            .occurrence_seals
            .iter()
            .find(|seal| {
                seal.malformed_projection.iter().any(|fact| {
                    matches!(
                        &fact.value,
                        CanonicalMalformedSealValue::Actual(
                            CanonicalActualLexicalEvidence::Token { spelling, .. }
                        ) if spelling == "==="
                    )
                })
            })
            .expect("UTF-8 invalid comparison occurrence")
            .clone();
        for facts in [
            &mut equal_length.malformed_projection,
            &mut equal_length.malformed_authority,
        ] {
            for fact in facts {
                if let CanonicalMalformedSealValue::Actual(CanonicalActualLexicalEvidence::Token {
                    spelling,
                    ..
                }) = &mut fact.value
                    && spelling == "==="
                {
                    *spelling = "!==".to_string();
                }
            }
        }
        assert!(
            validate_occurrence_seal(&equal_length).is_err(),
            "equal-length malformed token substitution stayed green"
        );

        let unsupported = parse_source(
            "f3-unsupported-completion.hum",
            "task unsupported_completion(value: UInt) -> UInt {\n  does:\n    return foo bar\n}\n",
        );
        let seal = unsupported
            .occurrence_seals
            .iter()
            .find(|seal| {
                seal.projection.iter().any(|fact| {
                    matches!(
                        fact,
                        CanonicalSealFact::Kind(_, CanonicalCommonNodeKind::Unsupported)
                    )
                })
            })
            .expect("unsupported common node");
        assert!(
            !seal.malformed_projection.is_empty() && validate_occurrence_seal(seal).is_ok(),
            "unsupported common nodes require exact malformed evidence"
        );
        let mut forced_complete = seal.clone();
        forced_complete.malformed_projection.clear();
        forced_complete.malformed_authority.clear();
        for facts in [
            &mut forced_complete.projection,
            &mut forced_complete.authority,
        ] {
            for fact in facts {
                if let CanonicalSealFact::LexicalStatus(_, status) = fact {
                    *status = CanonicalCommonLexicalStatus::Complete;
                }
            }
        }
        assert!(
            validate_occurrence_seal(&forced_complete).is_err(),
            "Unsupported plus forced Complete stayed green"
        );
        6
    }

    #[test]
    fn parser_completion_and_statement_relationships_are_complete_and_load_bearing() {
        let first = f3_evidence(None);
        let second = f3_evidence(None);
        assert_eq!(first, second, "fresh F3 inventories must be deterministic");
        assert!(complete_f3_evidence(&first), "{first:#?}");
        assert_eq!(f3_reviewer_regressions(), 6);
        for sabotage in [
            F3Sabotage::ProducerArm,
            F3Sabotage::ValidatorArm,
            F3Sabotage::CatalogueRow,
            F3Sabotage::MutationOperator,
            F3Sabotage::PairCase,
            F3Sabotage::SubstitutionCase,
        ] {
            assert!(
                !complete_f3_evidence(&f3_evidence(Some(sabotage))),
                "{sabotage:?} sabotage stayed green"
            );
        }
    }

    #[test]
    fn f3_real_corpora_cover_completion_and_statement_domains() {
        let malformed = parse_source("f3-malformed.hum", &f3_malformed_source());
        assert!(
            malformed
                .occurrence_seals
                .iter()
                .all(|seal| validate_occurrence_seal(seal).is_ok())
        );
        let causes = malformed
            .occurrence_seals
            .iter()
            .flat_map(|seal| &seal.malformed_projection)
            .filter_map(|fact| match fact.value {
                CanonicalMalformedSealValue::Cause(cause) => Some(cause),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(causes.len(), 12, "missing F3 malformed causes: {causes:#?}");

        let statements = parse_source("f3-statements.hum", F3_STATEMENT_SOURCE);
        assert!(
            statements
                .statement_seals
                .iter()
                .all(|seal| validate_statement_seal(seal).is_ok())
        );
        let kinds = statements
            .statement_seals
            .iter()
            .flat_map(|seal| &seal.projection)
            .filter_map(|fact| match fact.value {
                CanonicalStatementSealValue::Kind(kind) => Some(kind),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(kinds.len(), 17, "missing F3 statement kinds: {kinds:#?}");
        let fields = statements
            .statement_seals
            .iter()
            .flat_map(|seal| &seal.projection)
            .map(|fact| fact.field)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            fields,
            F3_STATEMENT_FIELDS.into_iter().collect(),
            "statement field catalogue is incomplete"
        );
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
