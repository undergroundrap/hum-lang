use std::collections::{BTreeMap, BTreeSet};

use crate::core_body::BodyStatement;
use crate::diagnostic::Span;
use crate::field_place;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AliasBinding {
    pub name: String,
    pub source_place: String,
    pub owner_root: String,
    pub binding_index: usize,
    pub binding_span: Span,
    pub last_use_index: usize,
    pub last_use_span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AliasIssueKind {
    Overlap,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AliasCause {
    DirectWriteOverlap,
    DirectReadOverlap,
    OwnerWideWriteOverlap,
    SecondWritableAliasOverlap,
    ShapeOutsideDirectFieldSlice,
    RebindsItsOwner,
    BindingRebinding,
    BindingInsideControlFlow,
    OwnerRebinding,
    LifetimeCrossesControlFlow,
    UseInsideControlFlow,
    Rebinding,
    Storage,
    UnsupportedUse,
    PermissionWrapper,
    AliasToAliasBinding,
    NestedOrElementUse,
    PassedToCall,
    Escape,
    OutsideTaskBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AliasCauseInvariantError {
    Missing,
    Unknown,
    Substituted,
    ContradictoryKind,
    ProducerSealMismatch,
    RegistryMismatch,
}

impl AliasCause {
    pub(crate) const fn key(self) -> crate::diagnostic_catalog::DiagnosticCauseKey {
        use AliasCause as Cause;
        crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(match self {
            Cause::DirectWriteOverlap => 116,
            Cause::Escape => 117,
            Cause::DirectReadOverlap => 158,
            Cause::OwnerWideWriteOverlap => 159,
            Cause::SecondWritableAliasOverlap => 160,
            Cause::ShapeOutsideDirectFieldSlice => 163,
            Cause::RebindsItsOwner => 164,
            Cause::BindingRebinding => 165,
            Cause::BindingInsideControlFlow => 166,
            Cause::OwnerRebinding => 167,
            Cause::LifetimeCrossesControlFlow => 168,
            Cause::UseInsideControlFlow => 169,
            Cause::Rebinding => 170,
            Cause::Storage => 171,
            Cause::UnsupportedUse => 172,
            Cause::PermissionWrapper => 173,
            Cause::AliasToAliasBinding => 174,
            Cause::NestedOrElementUse => 175,
            Cause::PassedToCall => 176,
            Cause::OutsideTaskBody => 177,
        })
    }

    pub(crate) const fn reason(self) -> &'static str {
        use AliasCause as Cause;
        match self {
            Cause::DirectWriteOverlap => "direct_write_overlaps_live_writable_alias_v0",
            Cause::DirectReadOverlap => "direct_read_overlaps_live_writable_alias_v0",
            Cause::OwnerWideWriteOverlap => "owner_wide_write_overlaps_live_writable_alias_v0",
            Cause::SecondWritableAliasOverlap => "second_writable_alias_overlaps_live_alias_v0",
            Cause::ShapeOutsideDirectFieldSlice => {
                "writable_alias_shape_outside_direct_field_slice_v0"
            }
            Cause::RebindsItsOwner => "writable_alias_rebinds_its_owner_v0",
            Cause::BindingRebinding => "writable_alias_binding_rebinding_v0",
            Cause::BindingInsideControlFlow => "writable_alias_binding_inside_control_flow_v0",
            Cause::OwnerRebinding => "writable_alias_owner_rebinding_v0",
            Cause::LifetimeCrossesControlFlow => "writable_alias_lifetime_crosses_control_flow_v0",
            Cause::UseInsideControlFlow => "writable_alias_use_inside_control_flow_v0",
            Cause::Rebinding => "writable_alias_rebinding_v0",
            Cause::Storage => "writable_alias_storage_v0",
            Cause::UnsupportedUse => "unsupported_writable_alias_use_v0",
            Cause::PermissionWrapper => "writable_alias_permission_wrapper_v0",
            Cause::AliasToAliasBinding => "writable_alias_to_alias_binding_v0",
            Cause::NestedOrElementUse => "writable_alias_nested_or_element_use_v0",
            Cause::PassedToCall => "writable_alias_passed_to_call_v0",
            Cause::Escape => "writable_alias_escape_v0",
            Cause::OutsideTaskBody => "writable_alias_outside_task_body_v0",
        }
    }

    #[cfg(test)]
    fn from_key(key: crate::diagnostic_catalog::DiagnosticCauseKey) -> Option<Self> {
        const ALL: [AliasCause; 20] = [
            AliasCause::DirectWriteOverlap,
            AliasCause::DirectReadOverlap,
            AliasCause::OwnerWideWriteOverlap,
            AliasCause::SecondWritableAliasOverlap,
            AliasCause::ShapeOutsideDirectFieldSlice,
            AliasCause::RebindsItsOwner,
            AliasCause::BindingRebinding,
            AliasCause::BindingInsideControlFlow,
            AliasCause::OwnerRebinding,
            AliasCause::LifetimeCrossesControlFlow,
            AliasCause::UseInsideControlFlow,
            AliasCause::Rebinding,
            AliasCause::Storage,
            AliasCause::UnsupportedUse,
            AliasCause::PermissionWrapper,
            AliasCause::AliasToAliasBinding,
            AliasCause::NestedOrElementUse,
            AliasCause::PassedToCall,
            AliasCause::Escape,
            AliasCause::OutsideTaskBody,
        ];
        ALL.into_iter().find(|cause| cause.key() == key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AliasIssue {
    pub kind: AliasIssueKind,
    pub cause: AliasCause,
    pub index: usize,
    pub alias_name: String,
    pub source_place: String,
    pub binding_span: Span,
    pub last_use_span: Span,
    pub conflict_place: String,
    pub conflict_span: Span,
    cause_seal: String,
}

impl AliasIssue {
    pub(crate) const fn reason(&self) -> &'static str {
        self.cause.reason()
    }

    pub(crate) fn validate_cause(&self) -> Result<(), AliasCauseInvariantError> {
        validate_typed_cause(self, Some(self.cause))
    }
}

fn validate_typed_cause(
    issue: &AliasIssue,
    supplied: Option<AliasCause>,
) -> Result<(), AliasCauseInvariantError> {
    let supplied = supplied.ok_or(AliasCauseInvariantError::Missing)?;
    if supplied != issue.cause {
        return Err(AliasCauseInvariantError::Substituted);
    }
    let expected_kind = match supplied {
        AliasCause::DirectWriteOverlap
        | AliasCause::DirectReadOverlap
        | AliasCause::OwnerWideWriteOverlap
        | AliasCause::SecondWritableAliasOverlap => AliasIssueKind::Overlap,
        _ => AliasIssueKind::Unsupported,
    };
    if issue.kind != expected_kind {
        return Err(AliasCauseInvariantError::ContradictoryKind);
    }
    if issue.cause_seal != alias_cause_seal(issue) {
        return Err(AliasCauseInvariantError::ProducerSealMismatch);
    }
    let spec = crate::diagnostic_catalog::diagnostic_cause_for_key(supplied.key())
        .ok_or(AliasCauseInvariantError::Unknown)?;
    let expected_code = match issue.kind {
        AliasIssueKind::Overlap => crate::diagnostic::DiagnosticCode::WRITABLE_ALIAS_OVERLAP,
        AliasIssueKind::Unsupported => {
            crate::diagnostic::DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS
        }
    };
    if spec.code != expected_code || spec.reason != supplied.reason() {
        return Err(AliasCauseInvariantError::RegistryMismatch);
    }
    Ok(())
}

fn alias_cause_seal(issue: &AliasIssue) -> String {
    format!(
        "alias-cause:{}:{:?}:{}:{}:{}:{}",
        issue.cause.key().ordinal(),
        issue.kind,
        issue.index,
        issue.alias_name,
        issue.source_place,
        issue.conflict_place,
    )
}

fn seal_alias_issues(issues: &mut [AliasIssue]) {
    for issue in issues {
        issue.cause_seal = alias_cause_seal(issue);
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct AliasAnalysis {
    pub bindings: Vec<AliasBinding>,
    pub issues: Vec<AliasIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AliasSyntax {
    pub name: String,
    pub source_place: String,
    pub owner_root: String,
}

#[derive(Debug, Clone)]
struct AliasCandidate {
    name: String,
    source_text: String,
}

pub(crate) fn candidate_name(statement: &BodyStatement) -> Option<String> {
    alias_candidate(statement).map(|candidate| candidate.name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Control,
    Literal,
}

#[derive(Debug, Clone)]
struct PlaceAccess {
    place: String,
    cause: AliasCause,
}

#[cfg(test)]
pub(crate) fn analyze(statements: &[BodyStatement]) -> AliasAnalysis {
    analyze_with_existing_names(statements, &BTreeSet::new())
}

pub(crate) fn analyze_with_existing_names(
    statements: &[BodyStatement],
    existing_names: &BTreeSet<String>,
) -> AliasAnalysis {
    let control_depths = control_depths(statements);
    let mut bindings = Vec::new();
    let mut issues = BTreeMap::<(usize, String), AliasIssue>::new();
    let mut seen_names = existing_names.clone();

    for (index, statement) in statements.iter().enumerate() {
        let statement_binding_name = binding_name(statement).map(str::to_string);
        let binding_name_already_visible = statement_binding_name
            .as_ref()
            .is_some_and(|name| seen_names.contains(name));
        if let Some(name) = statement_binding_name {
            seen_names.insert(name);
        }
        let Some(candidate) = alias_candidate(statement) else {
            continue;
        };
        let Some(syntax) = exact_binding(statement) else {
            issues
                .entry((index, candidate.name.clone()))
                .or_insert_with(|| AliasIssue {
                    kind: AliasIssueKind::Unsupported,
                    cause: AliasCause::ShapeOutsideDirectFieldSlice,
                    index,
                    alias_name: candidate.name,
                    source_place: candidate.source_text.clone(),
                    binding_span: statement.span.clone(),
                    last_use_span: statement.span.clone(),
                    conflict_place: candidate.source_text,
                    conflict_span: statement.span.clone(),
                    cause_seal: String::new(),
                });
            continue;
        };
        if syntax.name == syntax.owner_root {
            issues
                .entry((index, syntax.name.clone()))
                .or_insert_with(|| {
                    unsupported_issue(
                        index,
                        &syntax,
                        &statement.span,
                        &statement.span,
                        AliasCause::RebindsItsOwner,
                    )
                });
            continue;
        }
        if binding_name_already_visible {
            issues
                .entry((index, syntax.name.clone()))
                .or_insert_with(|| {
                    unsupported_issue(
                        index,
                        &syntax,
                        &statement.span,
                        &statement.span,
                        AliasCause::BindingRebinding,
                    )
                });
            continue;
        }
        if control_depths[index] > 0 {
            issues
                .entry((index, syntax.name.clone()))
                .or_insert_with(|| {
                    unsupported_issue(
                        index,
                        &syntax,
                        &statement.span,
                        &statement.span,
                        AliasCause::BindingInsideControlFlow,
                    )
                });
            continue;
        }

        bindings.push(AliasBinding {
            name: syntax.name,
            source_place: syntax.source_place,
            owner_root: syntax.owner_root,
            binding_index: index,
            binding_span: statement.span.clone(),
            last_use_index: index,
            last_use_span: statement.span.clone(),
        });
    }

    for binding in &mut bindings {
        for (index, statement) in statements
            .iter()
            .enumerate()
            .skip(binding.binding_index + 1)
        {
            if statement_references_alias(statement, &binding.name) {
                binding.last_use_index = index;
                binding.last_use_span = statement.span.clone();
            }
        }
    }

    for binding in &bindings {
        for index in (binding.binding_index + 1)..=binding.last_use_index {
            let statement = &statements[index];
            if binding_name(statement).is_some_and(|name| name == binding.owner_root) {
                issues
                    .entry((index, binding.name.clone()))
                    .or_insert_with(|| AliasIssue {
                        kind: AliasIssueKind::Unsupported,
                        cause: AliasCause::OwnerRebinding,
                        index,
                        alias_name: binding.name.clone(),
                        source_place: binding.source_place.clone(),
                        binding_span: binding.binding_span.clone(),
                        last_use_span: binding.last_use_span.clone(),
                        conflict_place: binding.owner_root.clone(),
                        conflict_span: statement.span.clone(),
                        cause_seal: String::new(),
                    });
                continue;
            }
            if let Some(cause) =
                unsupported_alias_use(statement, &binding.name, control_depths[index])
            {
                issues
                    .entry((index, binding.name.clone()))
                    .or_insert_with(|| AliasIssue {
                        kind: AliasIssueKind::Unsupported,
                        cause,
                        index,
                        alias_name: binding.name.clone(),
                        source_place: binding.source_place.clone(),
                        binding_span: binding.binding_span.clone(),
                        last_use_span: binding.last_use_span.clone(),
                        conflict_place: binding.name.clone(),
                        conflict_span: statement.span.clone(),
                        cause_seal: String::new(),
                    });
                continue;
            }

            if is_control_header(statement) {
                issues
                    .entry((index, binding.name.clone()))
                    .or_insert_with(|| AliasIssue {
                        kind: AliasIssueKind::Unsupported,
                        cause: AliasCause::LifetimeCrossesControlFlow,
                        index,
                        alias_name: binding.name.clone(),
                        source_place: binding.source_place.clone(),
                        binding_span: binding.binding_span.clone(),
                        last_use_span: binding.last_use_span.clone(),
                        conflict_place: statement.text.clone(),
                        conflict_span: statement.span.clone(),
                        cause_seal: String::new(),
                    });
                continue;
            }

            if let Some(access) = statement_place_accesses(statement, &binding.name)
                .into_iter()
                .find(|access| places_overlap(&binding.source_place, &access.place))
            {
                issues
                    .entry((index, binding.name.clone()))
                    .or_insert_with(|| AliasIssue {
                        kind: AliasIssueKind::Overlap,
                        cause: access.cause,
                        index,
                        alias_name: binding.name.clone(),
                        source_place: binding.source_place.clone(),
                        binding_span: binding.binding_span.clone(),
                        last_use_span: binding.last_use_span.clone(),
                        conflict_place: access.place,
                        conflict_span: statement.span.clone(),
                        cause_seal: String::new(),
                    });
            }
        }
    }

    let mut issues = issues.into_values().collect::<Vec<_>>();
    issues.sort_by_key(|issue| {
        (
            issue.index,
            match issue.kind {
                AliasIssueKind::Unsupported => 0,
                AliasIssueKind::Overlap => 1,
            },
            issue.alias_name.clone(),
        )
    });
    seal_alias_issues(&mut issues);
    AliasAnalysis { bindings, issues }
}

pub(crate) fn analyze_item(
    statements: &[BodyStatement],
    task_body: bool,
    existing_names: &BTreeSet<String>,
) -> AliasAnalysis {
    let mut analysis = analyze_with_existing_names(statements, existing_names);
    if task_body {
        return analysis;
    }
    for binding in &analysis.bindings {
        if analysis.issues.iter().any(|issue| {
            issue.alias_name == binding.name && issue.kind == AliasIssueKind::Unsupported
        }) {
            continue;
        }
        analysis.issues.push(AliasIssue {
            kind: AliasIssueKind::Unsupported,
            cause: AliasCause::OutsideTaskBody,
            index: binding.binding_index,
            alias_name: binding.name.clone(),
            source_place: binding.source_place.clone(),
            binding_span: binding.binding_span.clone(),
            last_use_span: binding.last_use_span.clone(),
            conflict_place: binding.name.clone(),
            conflict_span: binding.binding_span.clone(),
            cause_seal: String::new(),
        });
    }
    analysis.issues.sort_by_key(|issue| {
        (
            issue.index,
            match issue.kind {
                AliasIssueKind::Unsupported => 0,
                AliasIssueKind::Overlap => 1,
            },
            issue.alias_name.clone(),
        )
    });
    seal_alias_issues(&mut analysis.issues);
    analysis
}

pub(crate) fn exact_binding(statement: &BodyStatement) -> Option<AliasSyntax> {
    exact_binding_text(&statement.text)
}

pub(crate) fn exact_binding_text(text: &str) -> Option<AliasSyntax> {
    let rest = strip_keyword(text.trim(), "let")?;
    let (left, initializer) = rest.split_once('=')?;
    let left = left.trim();
    if left.contains(':') || !is_value_ident(left) {
        return None;
    }
    let source_place = strip_keyword(initializer.trim(), "change")?;
    let (owner_root, _field) = field_place::split_field_place(source_place)?;
    Some(AliasSyntax {
        name: left.to_string(),
        source_place: source_place.to_string(),
        owner_root: owner_root.to_string(),
    })
}

pub(crate) fn overlap_message(issue: &AliasIssue) -> String {
    format!(
        "writable alias `{}` to `{}` overlaps access `{}` while the alias is live",
        issue.alias_name, issue.source_place, issue.conflict_place
    )
}

pub(crate) fn unsupported_message(issue: &AliasIssue) -> String {
    format!(
        "writable alias `{}` uses a form outside the straight-line direct-field slice",
        issue.alias_name
    )
}

pub(crate) fn issue_help(task_name: &str, issue: &AliasIssue) -> String {
    match issue.kind {
        AliasIssueKind::Overlap => format!(
            "Fix task `{task_name}`: writable alias `{}` binds `{}` at {}:{}:{} and remains live through its last syntactic use at {}:{}:{}; access `{}` at {}:{}:{} may name the same storage, so the alias and conflicting access are not known independent. Use a definitely distinct direct field, or move this access after the alias's last use.",
            issue.alias_name,
            issue.source_place,
            issue.binding_span.file,
            issue.binding_span.line,
            issue.binding_span.column,
            issue.last_use_span.file,
            issue.last_use_span.line,
            issue.last_use_span.column,
            issue.conflict_place,
            issue.conflict_span.file,
            issue.conflict_span.line,
            issue.conflict_span.column,
        ),
        AliasIssueKind::Unsupported => format!(
            "Fix task `{task_name}`: writable alias `{}` binds `{}` at {}:{}:{}, but `{}` at {}:{}:{} is outside Session V's direct-field, straight-line, non-escaping slice ({}). Keep creation and use in one straight-line body, read or write through the alias locally, and copy the field value instead of passing, returning, storing, nesting, or rebinding the alias.",
            issue.alias_name,
            issue.source_place,
            issue.binding_span.file,
            issue.binding_span.line,
            issue.binding_span.column,
            issue.conflict_place,
            issue.conflict_span.file,
            issue.conflict_span.line,
            issue.conflict_span.column,
            issue.reason(),
        ),
    }
}

pub(crate) fn authority_message(binding: &AliasBinding) -> String {
    format!(
        "writable alias `{}` cannot acquire mutation authority for `{}`",
        binding.name, binding.source_place
    )
}

pub(crate) fn authority_help(
    task_name: &str,
    binding: &AliasBinding,
    owner_permission: &str,
) -> String {
    format!(
        "Fix task `{task_name}`: writable alias `{}` binds `{}` at {}:{}:{}, but owner `{}` has `{owner_permission}` authority. Bind the owner with `change` or pass it with `change`/`consume`; `borrow` and immutable `let` owners cannot supply a writable field alias.",
        binding.name,
        binding.source_place,
        binding.binding_span.file,
        binding.binding_span.line,
        binding.binding_span.column,
        binding.owner_root,
    )
}

fn unsupported_issue(
    index: usize,
    syntax: &AliasSyntax,
    binding_span: &Span,
    conflict_span: &Span,
    cause: AliasCause,
) -> AliasIssue {
    AliasIssue {
        kind: AliasIssueKind::Unsupported,
        cause,
        index,
        alias_name: syntax.name.clone(),
        source_place: syntax.source_place.clone(),
        binding_span: binding_span.clone(),
        last_use_span: conflict_span.clone(),
        conflict_place: syntax.source_place.clone(),
        conflict_span: conflict_span.clone(),
        cause_seal: String::new(),
    }
}

fn alias_candidate(statement: &BodyStatement) -> Option<AliasCandidate> {
    if !matches!(statement.kind, "let_binding" | "mutable_binding") {
        return None;
    }
    let (left, initializer) = statement.text.split_once('=')?;
    let source_text = strip_keyword(initializer.trim(), "change")?.to_string();
    let keyword = if statement.kind == "let_binding" {
        "let"
    } else {
        "change"
    };
    let left = strip_keyword(left.trim(), keyword)?;
    let name = left
        .split_once(':')
        .map_or(left, |(name, _annotation)| name)
        .trim();
    Some(AliasCandidate {
        name: if name.is_empty() {
            "unknown_alias".to_string()
        } else {
            name.to_string()
        },
        source_text,
    })
}

fn control_depths(statements: &[BodyStatement]) -> Vec<usize> {
    let mut stack = Vec::<BlockKind>::new();
    let mut depths = Vec::with_capacity(statements.len());
    for statement in statements {
        if statement.kind == "block_close" {
            stack.pop();
            depths.push(
                stack
                    .iter()
                    .filter(|kind| **kind == BlockKind::Control)
                    .count(),
            );
            continue;
        }
        depths.push(
            stack
                .iter()
                .filter(|kind| **kind == BlockKind::Control)
                .count(),
        );
        if is_control_header(statement) {
            stack.push(BlockKind::Control);
        } else if statement.expression_kind == Some("record_literal_start") {
            stack.push(BlockKind::Literal);
        }
    }
    depths
}

fn is_control_header(statement: &BodyStatement) -> bool {
    matches!(
        statement.kind,
        "if_header" | "while_header" | "loop_header" | "for_each_header" | "for_index_header"
    )
}

fn unsupported_alias_use(
    statement: &BodyStatement,
    alias_name: &str,
    control_depth: usize,
) -> Option<AliasCause> {
    if !statement_references_alias(statement, alias_name) {
        return None;
    }
    if control_depth > 0 || is_control_header(statement) {
        return Some(AliasCause::UseInsideControlFlow);
    }
    if matches!(statement.kind, "return" | "fail") {
        return Some(AliasCause::Escape);
    }
    if binding_name(statement).is_some_and(|name| name == alias_name) {
        return Some(AliasCause::Rebinding);
    }
    if statement.kind == "save_in_store" {
        return Some(AliasCause::Storage);
    }
    if !matches!(
        statement.kind,
        "let_binding"
            | "mutable_binding"
            | "set_place"
            | "record_field_initializer"
            | "test_expectation"
    ) {
        return Some(AliasCause::UnsupportedUse);
    }
    let expression = expression_text(statement).unwrap_or_default();
    let expression_mentions_alias = contains_root_reference(expression, alias_name);
    if expression_mentions_alias
        && ["borrow", "consume"].iter().any(|permission| {
            strip_keyword(expression, permission)
                .is_some_and(|source| first_resource(source) == alias_name)
        })
    {
        return Some(AliasCause::PermissionWrapper);
    }
    if expression_mentions_alias
        && strip_keyword(expression, "change")
            .is_some_and(|source| first_resource(source) == alias_name)
    {
        return Some(AliasCause::AliasToAliasBinding);
    }
    if expression_mentions_alias
        && (nested_alias_reference(expression, alias_name)
            || set_target(statement)
                .is_some_and(|target| target != alias_name && first_resource(target) == alias_name))
    {
        return Some(AliasCause::NestedOrElementUse);
    }
    if expression_mentions_alias
        && (statement.kind == "record_field_initializer"
            || expression.trim().starts_with('[')
            || expression.trim().starts_with('{'))
    {
        return Some(AliasCause::Storage);
    }
    if expression_mentions_alias && expression.contains('(') && expression.contains(')') {
        return Some(AliasCause::PassedToCall);
    }
    None
}

fn statement_references_alias(statement: &BodyStatement, alias_name: &str) -> bool {
    if binding_name(statement).is_some_and(|name| name == alias_name) {
        return true;
    }
    if set_target(statement).is_some_and(|target| first_resource(target) == alias_name) {
        return true;
    }
    expression_text(statement).map_or_else(
        || contains_root_reference(&statement.text, alias_name),
        |expression| contains_root_reference(expression, alias_name),
    )
}

fn statement_place_accesses(statement: &BodyStatement, alias_name: &str) -> Vec<PlaceAccess> {
    let mut accesses = Vec::new();
    if let Some(target) = set_target(statement)
        && first_resource(target) != alias_name
    {
        accesses.push(PlaceAccess {
            place: target.to_string(),
            cause: if target.contains('.') {
                AliasCause::DirectWriteOverlap
            } else {
                AliasCause::OwnerWideWriteOverlap
            },
        });
    }

    if let Some(expression) = expression_text(statement) {
        let second_alias = alias_candidate(statement).is_some();
        let direct_callee = direct_call_callee(expression);
        for place in place_references(expression) {
            if direct_callee == Some(place.as_str()) {
                continue;
            }
            if first_resource(&place) == alias_name {
                continue;
            }
            accesses.push(PlaceAccess {
                place,
                cause: if second_alias {
                    AliasCause::SecondWritableAliasOverlap
                } else {
                    AliasCause::DirectReadOverlap
                },
            });
        }
    }
    accesses
}

fn direct_call_callee(text: &str) -> Option<&str> {
    let (callee, _arguments) = text.trim().split_once('(')?;
    let callee = callee.trim();
    (!callee.is_empty()
        && callee
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.')))
    .then_some(callee)
}

fn places_overlap(source_place: &str, access_place: &str) -> bool {
    let Some((source_root, source_field)) = field_place::split_field_place(source_place) else {
        return true;
    };
    let access_root = first_resource(access_place);
    if access_root != source_root {
        return false;
    }
    if access_place == source_root {
        return true;
    }
    if let Some((_root, access_field)) = field_place::split_field_place(access_place) {
        return access_field == source_field;
    }
    true
}

fn expression_text(statement: &BodyStatement) -> Option<&str> {
    match statement.kind {
        "return" => strip_keyword(&statement.text, "return"),
        "fail" => strip_keyword(&statement.text, "fail"),
        "let_binding" | "mutable_binding" | "set_place" => statement
            .text
            .split_once('=')
            .map(|(_left, value)| value.trim()),
        "if_header" => header_body(&statement.text, "if"),
        "while_header" => header_body(&statement.text, "while"),
        "for_each_header" => header_body(&statement.text, "for each"),
        "for_index_header" => header_body(&statement.text, "for index"),
        "record_field_initializer" => statement
            .text
            .split_once(':')
            .map(|(_field, value)| value.trim()),
        "test_expectation" => strip_keyword(&statement.text, "expect"),
        _ => None,
    }
}

fn set_target(statement: &BodyStatement) -> Option<&str> {
    if statement.kind != "set_place" {
        return None;
    }
    let rest = strip_keyword(&statement.text, "set")?;
    let (target, _value) = rest.split_once('=')?;
    let target = target.trim();
    (!target.is_empty()).then_some(target)
}

fn binding_name(statement: &BodyStatement) -> Option<&str> {
    if !matches!(statement.kind, "let_binding" | "mutable_binding") {
        return None;
    }
    let keyword = if statement.kind == "let_binding" {
        "let"
    } else {
        "change"
    };
    let rest = strip_keyword(&statement.text, keyword)?;
    let (left, _value) = rest.split_once('=')?;
    let name = left
        .split_once(':')
        .map_or(left, |(name, _annotation)| name)
        .trim();
    (!name.is_empty()).then_some(name)
}

fn contains_root_reference(text: &str, root: &str) -> bool {
    place_references(text)
        .into_iter()
        .any(|place| first_resource(&place) == root)
}

fn nested_alias_reference(text: &str, alias_name: &str) -> bool {
    place_references(text)
        .into_iter()
        .any(|place| first_resource(&place) == alias_name && place.trim() != alias_name)
}

fn place_references(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    for ch in text.chars() {
        if ch == '"' {
            in_string = !in_string;
            if !current.is_empty() {
                push_reference_token(&mut tokens, &mut current);
            }
            continue;
        }
        if in_string {
            continue;
        }
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '[' | ']') {
            current.push(ch);
        } else if !current.is_empty() {
            push_reference_token(&mut tokens, &mut current);
        }
    }
    if !current.is_empty() {
        push_reference_token(&mut tokens, &mut current);
    }
    tokens
}

fn push_reference_token(tokens: &mut Vec<String>, current: &mut String) {
    let token = std::mem::take(current);
    if matches!(
        token.as_str(),
        "let" | "change" | "borrow" | "consume" | "set" | "return" | "fail" | "true" | "false"
    ) || token.chars().all(|ch| ch.is_ascii_digit())
        || token
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
    {
        return;
    }
    if token
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase() || ch == '_')
    {
        tokens.push(token);
    }
}

fn first_resource(text: &str) -> &str {
    text.split(['.', '[']).next().unwrap_or(text).trim()
}

fn header_body<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = strip_keyword(text, keyword)?;
    rest.strip_suffix('{').map(str::trim)
}

fn strip_keyword<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    if text == keyword {
        return Some("");
    }
    text.strip_prefix(keyword)
        .and_then(|rest| rest.strip_prefix(char::is_whitespace))
        .map(str::trim)
}

fn is_value_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first == '_')
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::{
        AliasCause, AliasCauseInvariantError, AliasIssueKind, analyze, analyze_item,
        analyze_with_existing_names, exact_binding_text, validate_typed_cause,
    };
    use crate::core_body::BodyStatement;
    use crate::diagnostic::Span;

    #[test]
    fn typed_alias_cause_is_sealed_and_rendering_independent() {
        let analysis = analyze(&[
            statement(1, "let alias = change point.x", "let_binding"),
            statement(2, "set point.x = 7", "set_place"),
            statement(3, "set alias = 9", "set_place"),
        ]);
        let issue = &analysis.issues[0];
        issue.validate_cause().expect("sealed producer cause");
        let selected_cause = issue.cause;
        let mut rendered_reason = issue.reason().to_string();
        rendered_reason.push_str("_adversarial_rendering_change");
        assert_eq!(issue.cause, selected_cause);
        assert_ne!(rendered_reason, issue.reason());
        issue
            .validate_cause()
            .expect("rendering cannot select semantic cause");

        assert_eq!(
            validate_typed_cause(issue, None),
            Err(AliasCauseInvariantError::Missing)
        );
        assert_eq!(
            validate_typed_cause(issue, Some(AliasCause::DirectReadOverlap)),
            Err(AliasCauseInvariantError::Substituted)
        );

        let mut contradictory = issue.clone();
        contradictory.kind = AliasIssueKind::Unsupported;
        assert_eq!(
            contradictory.validate_cause(),
            Err(AliasCauseInvariantError::ContradictoryKind)
        );

        let mut cause_rewrite = issue.clone();
        cause_rewrite.cause = AliasCause::DirectReadOverlap;
        assert_eq!(
            cause_rewrite.validate_cause(),
            Err(AliasCauseInvariantError::ProducerSealMismatch)
        );

        assert_eq!(
            AliasCause::from_key(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(65_535)
            ),
            None
        );
    }
    use std::collections::BTreeSet;

    fn statement(line: usize, text: &str, kind: &'static str) -> BodyStatement {
        BodyStatement {
            span: Span::new("alias-test.hum", line, 5),
            text: text.to_string(),
            kind,
            status: "recognized_v0",
            expression_kind: None,
            reason: None,
        }
    }

    #[test]
    fn parses_only_exact_unannotated_direct_field_aliases() {
        let alias = exact_binding_text("let alias_to_x = change point.x").unwrap();
        assert_eq!(alias.name, "alias_to_x");
        assert_eq!(alias.source_place, "point.x");
        assert!(exact_binding_text("let alias: UInt = change point.x").is_none());
        assert!(exact_binding_text("let alias = change point.x.deep").is_none());
        assert!(exact_binding_text("let alias = change points[0]").is_none());
    }

    #[test]
    fn rejects_same_field_access_only_while_alias_is_syntactically_live() {
        let statements = vec![
            statement(1, "let alias_to_x = change point.x", "let_binding"),
            statement(2, "set point.x = 7", "set_place"),
            statement(3, "set alias_to_x = 9", "set_place"),
            statement(4, "set point.x = 11", "set_place"),
        ];
        let analysis = analyze(&statements);
        assert_eq!(analysis.bindings[0].last_use_index, 2);
        assert_eq!(analysis.issues.len(), 1);
        assert_eq!(analysis.issues[0].kind, AliasIssueKind::Overlap);
        assert_eq!(analysis.issues[0].index, 1);
    }

    #[test]
    fn accepts_distinct_live_fields_and_sequential_same_field_aliases() {
        let statements = vec![
            statement(1, "let x_alias = change point.x", "let_binding"),
            statement(2, "let y_alias = change point.y", "let_binding"),
            statement(3, "set y_alias = 4", "set_place"),
            statement(4, "set x_alias = 3", "set_place"),
            statement(5, "let next_x = change point.x", "let_binding"),
            statement(6, "set next_x = 8", "set_place"),
        ];
        let analysis = analyze(&statements);
        assert_eq!(analysis.bindings.len(), 3);
        assert!(analysis.issues.is_empty());
    }

    #[test]
    fn rejects_overlapping_second_alias_when_first_has_later_use() {
        let statements = vec![
            statement(1, "let first = change point.x", "let_binding"),
            statement(2, "let second = change point.x", "let_binding"),
            statement(3, "set first = 3", "set_place"),
        ];
        let analysis = analyze(&statements);
        assert_eq!(analysis.issues.len(), 1);
        assert_eq!(analysis.issues[0].kind, AliasIssueKind::Overlap);
        assert_eq!(analysis.issues[0].index, 1);
    }

    #[test]
    fn rejects_escape_and_control_flow_crossing_as_unsupported() {
        let escape = analyze(&[
            statement(1, "let alias = change point.x", "let_binding"),
            statement(2, "return alias", "return"),
        ]);
        assert_eq!(escape.issues[0].kind, AliasIssueKind::Unsupported);
        assert_eq!(escape.issues[0].reason(), "writable_alias_escape_v0");

        let control = analyze(&[
            statement(1, "let alias = change point.x", "let_binding"),
            statement(2, "if ready {", "if_header"),
            statement(3, "}", "block_close"),
            statement(4, "set alias = 9", "set_place"),
        ]);
        assert_eq!(control.issues[0].kind, AliasIssueKind::Unsupported);
        assert_eq!(
            control.issues[0].reason(),
            "writable_alias_lifetime_crosses_control_flow_v0"
        );
    }

    #[test]
    fn ignores_alias_spelling_inside_text_literals() {
        let analysis = analyze(&[
            statement(1, "let alias = change point.x", "let_binding"),
            statement(2, "let text: Text = \"alias point.x\"", "let_binding"),
            statement(3, "set alias = 9", "set_place"),
        ]);
        assert!(analysis.issues.is_empty());
    }

    #[test]
    fn rejects_direct_reads_and_owner_wide_access_while_live() {
        for conflicting in [
            statement(2, "let copy = point.x", "let_binding"),
            statement(2, "let copy = point", "let_binding"),
            statement(2, "set point = replacement", "set_place"),
        ] {
            let analysis = analyze(&[
                statement(1, "let alias = change point.x", "let_binding"),
                conflicting,
                statement(3, "set alias = 9", "set_place"),
            ]);
            assert_eq!(analysis.issues.len(), 1);
            assert_eq!(analysis.issues[0].kind, AliasIssueKind::Overlap);
            assert_eq!(analysis.issues[0].index, 1);
        }
    }

    #[test]
    fn accepts_distinct_direct_reads_and_writes_while_live() {
        let analysis = analyze(&[
            statement(1, "let alias = change point.x", "let_binding"),
            statement(2, "let copy = point.y", "let_binding"),
            statement(3, "set point.y = 7", "set_place"),
            statement(4, "set alias = copy", "set_place"),
        ]);
        assert!(analysis.issues.is_empty());
    }

    #[test]
    fn rejects_binding_or_using_an_alias_in_control_flow() {
        let nested_binding = analyze(&[
            statement(1, "if ready {", "if_header"),
            statement(2, "let alias = change point.x", "let_binding"),
            statement(3, "}", "block_close"),
        ]);
        assert_eq!(nested_binding.issues[0].kind, AliasIssueKind::Unsupported);
        assert_eq!(
            nested_binding.issues[0].reason(),
            "writable_alias_binding_inside_control_flow_v0"
        );

        let nested_use = analyze(&[
            statement(1, "let alias = change point.x", "let_binding"),
            statement(2, "if ready {", "if_header"),
            statement(3, "set alias = 9", "set_place"),
            statement(4, "}", "block_close"),
        ]);
        assert!(nested_use.issues.iter().any(|issue| {
            issue.kind == AliasIssueKind::Unsupported
                && matches!(
                    issue.reason(),
                    "writable_alias_lifetime_crosses_control_flow_v0"
                        | "writable_alias_use_inside_control_flow_v0"
                )
        }));
    }

    #[test]
    fn rejects_alias_owner_rebinding_only_while_the_alias_is_live() {
        let self_rebinding = analyze(&[statement(1, "let point = change point.x", "let_binding")]);
        assert_eq!(self_rebinding.issues[0].kind, AliasIssueKind::Unsupported);
        assert_eq!(
            self_rebinding.issues[0].reason(),
            "writable_alias_rebinds_its_owner_v0"
        );

        let live_owner_rebinding = analyze(&[
            statement(1, "let alias = change point.x", "let_binding"),
            statement(2, "let point = replacement", "let_binding"),
            statement(3, "set alias = 9", "set_place"),
        ]);
        assert_eq!(
            live_owner_rebinding.issues[0].reason(),
            "writable_alias_owner_rebinding_v0"
        );

        let owner_rebinding_after_last_use = analyze(&[
            statement(1, "let alias = change point.x", "let_binding"),
            statement(2, "set alias = 9", "set_place"),
            statement(3, "let point = replacement", "let_binding"),
        ]);
        assert!(owner_rebinding_after_last_use.issues.is_empty());
    }

    #[test]
    fn rejects_alias_binding_that_shadows_an_existing_name() {
        let existing = BTreeSet::from(["other".to_string()]);
        let parameter_collision = analyze_with_existing_names(
            &[statement(1, "let other = change point.x", "let_binding")],
            &existing,
        );
        assert_eq!(
            parameter_collision.issues[0].reason(),
            "writable_alias_binding_rebinding_v0"
        );

        let local_collision = analyze(&[
            statement(1, "change other: UInt = 0", "mutable_binding"),
            statement(2, "let other = change point.x", "let_binding"),
        ]);
        assert_eq!(
            local_collision.issues[0].reason(),
            "writable_alias_binding_rebinding_v0"
        );
    }

    #[test]
    fn rejects_permission_wrapped_alias_uses() {
        for permission in ["borrow", "consume"] {
            let analysis = analyze(&[
                statement(1, "let alias = change point.x", "let_binding"),
                statement(
                    2,
                    &format!("let wrapped = {permission} alias"),
                    "let_binding",
                ),
            ]);
            assert_eq!(analysis.issues[0].kind, AliasIssueKind::Unsupported);
            assert_eq!(
                analysis.issues[0].reason(),
                "writable_alias_permission_wrapper_v0"
            );
        }
    }

    #[test]
    fn rejects_alias_to_alias_nested_list_passing_storage_and_rebinding() {
        let cases = [
            vec![
                statement(1, "let alias = change point.x", "let_binding"),
                statement(2, "let next = change alias.x", "let_binding"),
            ],
            vec![statement(
                1,
                "let alias = change point.x.deep",
                "let_binding",
            )],
            vec![statement(1, "let alias = change points[0]", "let_binding")],
            vec![
                statement(1, "let alias = change point.x", "let_binding"),
                statement(2, "let sent = send(alias)", "let_binding"),
            ],
            vec![
                statement(1, "let alias = change point.x", "let_binding"),
                statement(2, "let borrowed = borrow alias", "let_binding"),
            ],
            vec![
                statement(1, "let alias = change point.x", "let_binding"),
                statement(2, "let moved = consume alias", "let_binding"),
            ],
            vec![
                statement(1, "let alias = change point.x", "let_binding"),
                statement(2, "save alias in cache", "save_in_store"),
            ],
            vec![
                statement(1, "let alias = change point.x", "let_binding"),
                statement(2, "let alias = 7", "let_binding"),
            ],
        ];
        for statements in cases {
            let analysis = analyze(&statements);
            assert!(
                analysis
                    .issues
                    .iter()
                    .any(|issue| issue.kind == AliasIssueKind::Unsupported),
                "expected unsupported issue for {statements:?}"
            );
        }
    }

    #[test]
    fn accepts_arbitrary_rhs_when_only_the_set_target_is_the_alias() {
        for write in [
            statement(2, "set alias = make_value()", "set_place"),
            statement(2, "set alias = [1]", "set_place"),
        ] {
            let analysis = analyze(&[
                statement(1, "let alias = change point.x", "let_binding"),
                write,
            ]);
            assert!(analysis.issues.is_empty());
        }
    }

    #[test]
    fn does_not_treat_a_same_named_call_callee_as_owner_access() {
        let analysis = analyze(&[
            statement(1, "let alias = change point.x", "let_binding"),
            statement(2, "let value = point()", "let_binding"),
            statement(3, "set alias = value", "set_place"),
        ]);
        assert!(analysis.issues.is_empty());
    }

    #[test]
    fn preserves_unsupported_precedence_for_another_alias_at_the_same_site() {
        let analysis = analyze(&[
            statement(1, "let x_alias = change point.x", "let_binding"),
            statement(2, "let other_alias = change other.x", "let_binding"),
            statement(3, "set point.x = other_alias.deep", "set_place"),
            statement(4, "set x_alias = 1", "set_place"),
        ]);
        assert!(
            analysis
                .issues
                .iter()
                .any(|issue| { issue.index == 2 && issue.kind == AliasIssueKind::Unsupported })
        );
        assert_eq!(analysis.issues[0].kind, AliasIssueKind::Unsupported);
    }

    #[test]
    fn rejects_the_alias_form_outside_task_bodies() {
        let analysis = analyze_item(
            &[
                statement(1, "let alias = change point.x", "let_binding"),
                statement(2, "set alias = 9", "set_place"),
            ],
            false,
            &BTreeSet::new(),
        );
        assert_eq!(analysis.issues[0].kind, AliasIssueKind::Unsupported);
        assert_eq!(
            analysis.issues[0].reason(),
            "writable_alias_outside_task_body_v0"
        );
    }
}
