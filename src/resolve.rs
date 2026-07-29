use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{
    CanonicalExpression, CanonicalExpressionKind, CanonicalStatementEventField,
    CanonicalStatementEventValue, CanonicalStatementKindEvent, Item, Param, ParamPermission,
    ParsedBinaryOperator, ParsedBodyStatement, ParsedBodyStatementKind, Program, Section, Task,
    TypeSyntaxKind,
};
use crate::core_body;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticOccurrence, DiagnosticOccurrenceSet, Severity, Span,
};
use crate::graph::is_meaningful_line_text;
use crate::predicate;
use crate::version;
use crate::writable_field_alias;

pub const RESOLVE_REPORT_SCHEMA: &str = "hum.resolve.v0";
pub const RESOLVE_MODE: &str = "source_analysis_only_no_type_or_borrow_check";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalBackendResolverView {
    pub(crate) function_definition_identity: String,
    pub(crate) left_use_identity: String,
    pub(crate) left_definition_identity: String,
    pub(crate) right_use_identity: String,
    pub(crate) right_definition_identity: String,
    pub(crate) use_order_identity: String,
    pub(crate) definition_order_identity: String,
    pub(crate) distinct_binding_status: &'static str,
}

pub(crate) fn canonical_backend_resolver_view(
    program: &Program,
    diagnostics: &[Diagnostic],
    task: &Task,
    core: &crate::core_lower::CanonicalCheckedAddCoreView,
) -> Result<CanonicalBackendResolverView, &'static str> {
    let report = build_report(program, diagnostics);
    let function_definition_identity = semantic_task_definition_identity(program, task);
    let callable_scope = report
        .scopes
        .iter()
        .find(|scope| {
            scope.scope_kind == "callable"
                && scope.owner_semantic_identity == function_definition_identity
        })
        .ok_or("canonical_backend_callable_scope_absent_v0")?;
    let parameter_definition = |ordinal: usize| {
        report
            .definitions
            .iter()
            .find(|definition| {
                definition.scope_id == callable_scope.id
                    && definition.definition_kind == "parameter"
                    && definition.semantic_origin == format!("parameter-position:{ordinal}")
                    && definition.status == "defined_v0"
            })
            .map(semantic_definition_identity)
            .ok_or("canonical_backend_parameter_definition_absent_v0")
    };
    let expected_left_definition = parameter_definition(0)?;
    let expected_right_definition = parameter_definition(1)?;
    let resolve_operand =
        |node_identity: &str, expected_child_path: &[usize], expected_target: &str| {
            report
                .references
                .iter()
                .find(|reference| {
                    reference.canonical_node_id.as_deref() == Some(node_identity)
                        && reference.canonical_child_path.as_deref() == Some(expected_child_path)
                        && reference.resolution_status == "resolved_v0"
                        && reference.resolved_definition_semantic_identity.as_deref()
                            == Some(expected_target)
                })
                .map(|reference| {
                    (
                        reference.semantic_identity.clone(),
                        reference
                            .resolved_definition_semantic_identity
                            .clone()
                            .expect("resolved canonical reference has a semantic target"),
                    )
                })
                .ok_or("canonical_backend_operand_binding_absent_v0")
        };
    let (left_use_identity, left_definition_identity) =
        resolve_operand(&core.left_node_identity, &[0, 0], &expected_left_definition)?;
    let (right_use_identity, right_definition_identity) = resolve_operand(
        &core.right_node_identity,
        &[0, 1],
        &expected_right_definition,
    )?;
    if left_definition_identity == right_definition_identity {
        return Err("canonical_backend_operand_definitions_not_distinct_v0");
    }
    let use_order_identity =
        format!("resolver-use-order:left={left_use_identity}|right={right_use_identity}");
    let definition_order_identity = format!(
        "resolver-definition-order:left={left_definition_identity}|right={right_definition_identity}"
    );
    let mut view = CanonicalBackendResolverView {
        function_definition_identity: function_definition_identity.clone(),
        left_use_identity: left_use_identity.clone(),
        left_definition_identity: left_definition_identity.clone(),
        right_use_identity: right_use_identity.clone(),
        right_definition_identity: right_definition_identity.clone(),
        use_order_identity: use_order_identity.clone(),
        definition_order_identity: definition_order_identity.clone(),
        distinct_binding_status: "distinct_parameter_bindings_v0",
    };
    crate::ir_readiness::apply_c1_resolver_producer_corruption(&mut view);
    if view.function_definition_identity != function_definition_identity
        || view.left_use_identity != left_use_identity
        || view.left_definition_identity != left_definition_identity
        || view.right_use_identity != right_use_identity
        || view.right_definition_identity != right_definition_identity
        || view.use_order_identity != use_order_identity
        || view.definition_order_identity != definition_order_identity
        || view.distinct_binding_status != "distinct_parameter_bindings_v0"
    {
        return Err("canonical_backend_resolver_producer_corruption_v0");
    }
    Ok(view)
}

struct ResolveReport {
    files: usize,
    items: usize,
    source_errors: usize,
    source_warnings: usize,
    scopes: Vec<ResolveScope>,
    definitions: Vec<ResolveDefinition>,
    references: Vec<ResolveReference>,
    call_occurrences: Vec<ResolveCallOccurrenceSummary>,
    item_node_by_definition_id: BTreeMap<String, String>,
    diagnostics: Vec<ResolverDiagnostic>,
    diagnostic_occurrences: DiagnosticOccurrenceSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolveReadinessSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub files: usize,
    pub items: usize,
    pub source_errors: usize,
    pub source_warnings: usize,
    pub scopes: usize,
    pub definitions: usize,
    pub references: usize,
    pub resolved_references: usize,
    pub unresolved_references: usize,
    pub external_references: usize,
    pub duplicate_definitions: usize,
    pub mutable_place_errors: usize,
    pub resolver_errors: usize,
    pub resolver_warnings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveDefinitionSummary {
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub definition_kind: &'static str,
    pub scope_id: String,
    pub mutable: bool,
    pub state_kind: &'static str,
    pub source_span: Span,
    pub status: &'static str,
    pub(crate) semantic_identity: String,
    pub(crate) semantic_origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveScopeSummary {
    pub id: String,
    pub parent_scope_id: Option<String>,
    pub scope_kind: &'static str,
    pub owner_kind: &'static str,
    pub owner_name: String,
    pub source_span: Option<Span>,
    pub(crate) semantic_identity: String,
    pub(crate) owner_semantic_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveReferenceSummary {
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub reference_kind: &'static str,
    pub scope_id: String,
    pub source_span: Span,
    pub resolution_status: &'static str,
    pub resolved_definition_id: Option<String>,
    pub reason: Option<&'static str>,
    pub(crate) canonical_node_id: Option<String>,
    pub(crate) canonical_child_position: Option<String>,
    pub(crate) canonical_child_path: Option<Vec<usize>>,
    pub(crate) scope_semantic_identity: String,
    pub(crate) semantic_identity: String,
    pub(crate) resolved_definition_semantic_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ResolverCallOccurrenceId {
    reference_id: String,
    owner_definition_id: String,
    target_definition_id: String,
    position: CanonicalExpressionPosition,
}

impl ResolverCallOccurrenceId {
    pub(crate) fn stable_key(&self) -> String {
        format!(
            "{}|owner={}|target={}|{}",
            self.reference_id,
            self.owner_definition_id,
            self.target_definition_id,
            self.position.stable_component(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolveCallIdentifierUse {
    name: String,
    resolved_definition_id: Option<String>,
    ordinal: usize,
    consumed: bool,
}

impl ResolveCallIdentifierUse {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn consumed(&self) -> bool {
        self.consumed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolveCallOccurrenceSummary {
    pub reference_id: String,
    pub owner_definition_id: String,
    pub target_definition_id: String,
    pub exact_call_span: Span,
    resolver_occurrence_id: ResolverCallOccurrenceId,
    source: String,
    identifier_uses: Vec<ResolveCallIdentifierUse>,
    canonical_call_node_id: String,
    canonical_callee_node_id: String,
    canonical_argument_node_ids: Vec<String>,
}

impl ResolveCallOccurrenceSummary {
    pub(crate) fn resolver_occurrence_id(&self) -> &ResolverCallOccurrenceId {
        &self.resolver_occurrence_id
    }

    pub(crate) fn statement_index(&self) -> usize {
        self.resolver_occurrence_id.position.statement_index
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn identifier_uses(&self) -> impl Iterator<Item = &ResolveCallIdentifierUse> {
        self.identifier_uses.iter()
    }

    pub(crate) fn relationship_key(&self) -> String {
        self.resolver_occurrence_id.stable_key()
    }

    pub(crate) fn resolver_reference_identity(&self) -> &str {
        &self.resolver_occurrence_id.reference_id
    }

    pub(crate) fn relationship_route(&self) -> Vec<String> {
        vec![
            format!("resolver_call_occurrence={}", self.relationship_key()),
            format!("resolver_call_reference={}", self.reference_id),
            format!("resolver_call_owner={}", self.owner_definition_id),
            format!("resolver_call_target={}", self.target_definition_id),
            format!(
                "resolver_call_position={}",
                self.resolver_occurrence_id.position.stable_component()
            ),
        ]
    }

    pub(crate) fn canonical_call_node_id(&self) -> &str {
        &self.canonical_call_node_id
    }

    pub(crate) fn canonical_callee_node_id(&self) -> &str {
        &self.canonical_callee_node_id
    }

    pub(crate) fn canonical_argument_node_ids(&self) -> &[String] {
        &self.canonical_argument_node_ids
    }

    #[cfg(test)]
    pub(crate) fn replace_canonical_call_node_for_test(&mut self, replacement: &str) {
        self.canonical_call_node_id = replacement.to_string();
    }

    #[cfg(test)]
    pub(crate) fn replace_resolver_reference_identity_for_test(&mut self, replacement: &str) {
        self.resolver_occurrence_id.reference_id = replacement.to_string();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CanonicalExpressionPosition {
    statement_node_id: String,
    statement_index: usize,
    root_ordinal: usize,
    child_path: Vec<usize>,
    node_id: String,
}

#[cfg(test)]
#[derive(Clone)]
struct TenB2ResolverBoundaryCorruption {
    call_node_id: String,
    replacement_child_path: Vec<usize>,
}

#[cfg(test)]
thread_local! {
    static TEN_B2_RESOLVER_BOUNDARY_CORRUPTION:
        std::cell::RefCell<Option<TenB2ResolverBoundaryCorruption>> =
            const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn with_ten_b2_resolver_boundary_corruption<T>(
    corruption: TenB2ResolverBoundaryCorruption,
    run: impl FnOnce() -> T,
) -> T {
    TEN_B2_RESOLVER_BOUNDARY_CORRUPTION.with(|slot| {
        assert!(
            slot.replace(Some(corruption)).is_none(),
            "10B.2 resolver boundary corruption must not nest"
        );
    });
    let result = run();
    TEN_B2_RESOLVER_BOUNDARY_CORRUPTION.with(|slot| {
        assert!(
            slot.replace(None).is_some(),
            "10B.2 resolver boundary corruption must execute"
        );
    });
    result
}

impl CanonicalExpressionPosition {
    fn stable_component(&self) -> String {
        let child_path = self
            .child_path
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(".");
        format!(
            "statement-{}:{}:root-{}:path-{}:node-{}",
            self.statement_index,
            self.statement_node_id,
            self.root_ordinal,
            child_path,
            self.node_id
        )
    }
}

struct CanonicalCallNode<'a> {
    expression: &'a CanonicalExpression,
    callee: &'a CanonicalExpression,
    arguments: &'a [CanonicalExpression],
    position: CanonicalExpressionPosition,
}

#[derive(Debug, Clone)]
struct ResolveScope {
    id: String,
    parent_scope_id: Option<String>,
    scope_kind: &'static str,
    owner_kind: &'static str,
    owner_name: String,
    source_span: Option<Span>,
    semantic_identity: String,
    owner_semantic_identity: String,
}

#[derive(Debug, Clone)]
struct ResolveDefinition {
    id: String,
    name: String,
    normalized_name: String,
    definition_kind: &'static str,
    scope_id: String,
    mutable: bool,
    state_kind: &'static str,
    source_span: Span,
    status: &'static str,
    duplicate_of: Option<String>,
    semantic_identity: String,
    semantic_origin: String,
}

#[derive(Debug, Clone)]
struct ResolveReference {
    id: String,
    name: String,
    normalized_name: String,
    reference_kind: &'static str,
    scope_id: String,
    mutable_required: bool,
    source_span: Span,
    resolution_status: &'static str,
    resolved_definition_id: Option<String>,
    reason: Option<&'static str>,
    canonical_node_id: Option<String>,
    canonical_child_position: Option<String>,
    canonical_child_path: Option<Vec<usize>>,
    scope_semantic_identity: String,
    semantic_identity: String,
    resolved_definition_semantic_identity: Option<String>,
}

struct CallableResolveInput<'a> {
    item: &'a Item,
    owner_kind: &'static str,
    owner_name: &'a str,
    owner_definition_id: String,
    params: &'a [Param],
    sections: &'a [Section],
    span: &'a Span,
    resolve_canonical_body: bool,
}

#[derive(Debug, Clone)]
struct ResolverDiagnostic {
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    code: DiagnosticCode,
    severity: Severity,
    title: &'static str,
    message: String,
    source_span: Span,
    help: &'static str,
    reference_id: Option<String>,
    definition_id: Option<String>,
    reason: &'static str,
}

#[derive(Clone)]
struct DefinitionRef {
    index: usize,
}

struct PendingReferenceInput<'a> {
    name: &'a str,
    reference_kind: &'static str,
    mutable_required: bool,
    external_if_unresolved: bool,
    span: &'a Span,
}

struct DefinitionInput<'a> {
    name: &'a str,
    definition_kind: &'static str,
    mutable: bool,
    state_kind: &'static str,
    span: &'a Span,
    defer_duplicate_diagnostic: bool,
    semantic_origin: String,
    semantic_shape: Option<String>,
}

struct ScopeInput<'a> {
    parent_scope_id: Option<&'a str>,
    scope_kind: &'static str,
    owner_kind: &'static str,
    owner_name: &'a str,
    span: Option<&'a Span>,
    preferred_id: String,
    semantic_origin: String,
}

struct ResolverContext<'program> {
    program: &'program Program,
    scopes: Vec<ResolveScope>,
    definitions: Vec<ResolveDefinition>,
    references: Vec<ResolveReference>,
    call_occurrences: Vec<ResolveCallOccurrenceSummary>,
    diagnostics: Vec<ResolverDiagnostic>,
    scope_parents: BTreeMap<String, Option<String>>,
    scope_semantic_identities: BTreeMap<String, String>,
    definitions_by_scope_name: BTreeMap<(String, String), DefinitionRef>,
    scope_serial: usize,
    definition_serial: usize,
    reference_serial: usize,
    callable_receiver_definition_ids: BTreeSet<String>,
    callable_parameter_definition_ids: BTreeSet<String>,
    task_definition_ids_by_name: BTreeMap<String, Vec<String>>,
    item_node_by_definition_id: BTreeMap<String, String>,
}

pub fn resolve_has_errors(program: &Program, source_diagnostics: &[Diagnostic]) -> bool {
    let report = build_report(program, source_diagnostics);
    report.source_errors > 0 || report.error_count() > 0
}

pub fn resolve_readiness_summary(
    program: &Program,
    source_diagnostics: &[Diagnostic],
) -> ResolveReadinessSummary {
    let report = build_report(program, source_diagnostics);
    ResolveReadinessSummary {
        schema: RESOLVE_REPORT_SCHEMA,
        status: report.status(),
        mode: RESOLVE_MODE,
        files: report.files,
        items: report.items,
        source_errors: report.source_errors,
        source_warnings: report.source_warnings,
        scopes: report.scopes.len(),
        definitions: report.definitions.len(),
        references: report.references.len(),
        resolved_references: report.resolved_references(),
        unresolved_references: report.unresolved_references(),
        external_references: report.external_references(),
        duplicate_definitions: report.duplicate_definitions(),
        mutable_place_errors: report.mutable_place_errors(),
        resolver_errors: report.error_count(),
        resolver_warnings: report.warning_count(),
    }
}

pub fn resolve_definition_summaries(
    program: &Program,
    source_diagnostics: &[Diagnostic],
) -> Vec<ResolveDefinitionSummary> {
    let report = build_report(program, source_diagnostics);
    report
        .definitions
        .iter()
        .map(|definition| ResolveDefinitionSummary {
            id: definition.id.clone(),
            name: definition.name.clone(),
            normalized_name: definition.normalized_name.clone(),
            definition_kind: definition.definition_kind,
            scope_id: definition.scope_id.clone(),
            mutable: definition.mutable,
            state_kind: definition.state_kind,
            source_span: definition.source_span.clone(),
            status: definition.status,
            semantic_identity: definition.semantic_identity.clone(),
            semantic_origin: definition.semantic_origin.clone(),
        })
        .collect()
}

pub fn resolve_scope_summaries(
    program: &Program,
    source_diagnostics: &[Diagnostic],
) -> Vec<ResolveScopeSummary> {
    build_report(program, source_diagnostics)
        .scopes
        .iter()
        .map(|scope| ResolveScopeSummary {
            id: scope.id.clone(),
            parent_scope_id: scope.parent_scope_id.clone(),
            scope_kind: scope.scope_kind,
            owner_kind: scope.owner_kind,
            owner_name: scope.owner_name.clone(),
            source_span: scope.source_span.clone(),
            semantic_identity: scope.semantic_identity.clone(),
            owner_semantic_identity: scope.owner_semantic_identity.clone(),
        })
        .collect()
}

pub fn resolve_reference_summaries(
    program: &Program,
    source_diagnostics: &[Diagnostic],
) -> Vec<ResolveReferenceSummary> {
    build_report(program, source_diagnostics)
        .references
        .iter()
        .map(|reference| ResolveReferenceSummary {
            id: reference.id.clone(),
            name: reference.name.clone(),
            normalized_name: reference.normalized_name.clone(),
            reference_kind: reference.reference_kind,
            scope_id: reference.scope_id.clone(),
            source_span: reference.source_span.clone(),
            resolution_status: reference.resolution_status,
            resolved_definition_id: reference.resolved_definition_id.clone(),
            reason: reference.reason,
            canonical_node_id: reference.canonical_node_id.clone(),
            canonical_child_position: reference.canonical_child_position.clone(),
            canonical_child_path: reference.canonical_child_path.clone(),
            scope_semantic_identity: reference.scope_semantic_identity.clone(),
            semantic_identity: reference.semantic_identity.clone(),
            resolved_definition_semantic_identity: reference
                .resolved_definition_semantic_identity
                .clone(),
        })
        .collect()
}

pub(crate) fn resolve_call_occurrence_summaries(
    program: &Program,
    source_diagnostics: &[Diagnostic],
) -> Vec<ResolveCallOccurrenceSummary> {
    build_report(program, source_diagnostics).call_occurrences
}

fn resolver_call_reference_identity(
    owner_definition_id: &str,
    target_definition_id: &str,
    position: &CanonicalExpressionPosition,
) -> String {
    format!(
        "resolver-call-reference|owner={owner_definition_id}|target={target_definition_id}|{}",
        position.stable_component()
    )
}

fn unresolved_call_target_identity(
    owner_definition_id: &str,
    resolution_status: &str,
    position: &CanonicalExpressionPosition,
) -> String {
    format!(
        "resolver-call-target|owner={owner_definition_id}|status={resolution_status}|{}",
        position.stable_component()
    )
}

fn semantic_definition_identity(definition: &ResolveDefinition) -> String {
    definition.semantic_identity.clone()
}

fn item_definition_semantic_shape(item: &Item) -> String {
    fn type_shape(syntax: &crate::ast::TypeSyntax) -> String {
        match &syntax.kind {
            TypeSyntaxKind::Named { name } => format!("named:{name}"),
            TypeSyntaxKind::Result { value, .. } => format!("result({})", type_shape(value)),
            TypeSyntaxKind::Callable(callable) => format!(
                "callable({})->{}",
                callable
                    .inputs
                    .iter()
                    .map(type_shape)
                    .collect::<Vec<_>>()
                    .join(","),
                type_shape(&callable.result)
            ),
            TypeSyntaxKind::CallableCandidate { .. } => "callable-candidate".to_string(),
            TypeSyntaxKind::Other => "other".to_string(),
        }
    }

    fn expression_shape(expression: &CanonicalExpression) -> String {
        match &expression.kind {
            CanonicalExpressionKind::Unit => "unit".to_string(),
            CanonicalExpressionKind::Identifier(_) => "identifier".to_string(),
            CanonicalExpressionKind::Field { base, .. } => {
                format!("field({})", expression_shape(base))
            }
            CanonicalExpressionKind::ElementPlace { base, .. } => {
                format!("element({})", expression_shape(base))
            }
            CanonicalExpressionKind::UIntLiteral(_) => "uint".to_string(),
            CanonicalExpressionKind::IntLiteral(_) => "int".to_string(),
            CanonicalExpressionKind::BoolLiteral(_) => "bool".to_string(),
            CanonicalExpressionKind::TextLiteral(_) => "text".to_string(),
            CanonicalExpressionKind::ListLiteral(values) => format!(
                "list({})",
                values
                    .iter()
                    .map(expression_shape)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            CanonicalExpressionKind::RecordLiteral { fields, .. } => format!(
                "record({})",
                fields
                    .iter()
                    .map(|(_, value)| expression_shape(value))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            CanonicalExpressionKind::Call { callee, arguments } => format!(
                "call({};{})",
                expression_shape(callee),
                arguments
                    .iter()
                    .map(expression_shape)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            CanonicalExpressionKind::Binary {
                operator,
                left,
                right,
            } => format!(
                "binary({operator:?};{};{})",
                expression_shape(left),
                expression_shape(right)
            ),
            CanonicalExpressionKind::Permission { permission, value } => {
                format!("permission({permission:?};{})", expression_shape(value))
            }
            CanonicalExpressionKind::Try { value, .. } => {
                format!("try({})", expression_shape(value))
            }
            CanonicalExpressionKind::Group(value) => {
                format!("group({})", expression_shape(value))
            }
            CanonicalExpressionKind::Unsupported => "unsupported".to_string(),
        }
    }

    fn statement_shape(statement: &ParsedBodyStatement) -> String {
        let roots = parsed_statement_roots(statement)
            .into_iter()
            .map(expression_shape)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{}|{}|{}",
            statement.core_kind, statement.block_depth_before, roots
        )
    }

    match item {
        Item::App(app) => format!(
            "app|sections={}|children={}",
            app.sections.len(),
            app.items
                .iter()
                .map(Item::kind)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Item::Type(item) => format!(
            "type|fields={}|sections={}",
            item.fields.len(),
            item.sections.len()
        ),
        Item::Store(item) => format!("store|sections={}", item.sections.len()),
        Item::Task(task) => format!(
            "task|params={}|result={}|effects={}|body={}",
            task.params
                .iter()
                .map(|param| format!("{:?}:{}", param.permission, type_shape(&param.type_syntax)))
                .collect::<Vec<_>>()
                .join(","),
            task.result_syntax
                .as_ref()
                .map(type_shape)
                .unwrap_or_else(|| "unit".to_string()),
            task.effect_syntax.len(),
            task.section("does")
                .into_iter()
                .flat_map(|section| section.body_syntax.iter().flatten())
                .map(statement_shape)
                .collect::<Vec<_>>()
                .join(";")
        ),
        Item::Test(test) => format!(
            "test|params={}|modifiers={}|sections={}",
            test.params
                .iter()
                .map(|param| format!("{:?}:{}", param.permission, type_shape(&param.type_syntax)))
                .collect::<Vec<_>>()
                .join(","),
            test.modifiers.len(),
            test.sections.len()
        ),
    }
}

pub(crate) fn diagnostic_occurrence_set(
    program: &Program,
    source_diagnostics: &[Diagnostic],
) -> DiagnosticOccurrenceSet {
    build_report(program, source_diagnostics).diagnostic_occurrences
}

pub(crate) fn semantic_app_identity(program: &Program, target: &crate::ast::App) -> String {
    semantic_item_identity(
        program,
        |item| matches!(item, Item::App(app) if std::ptr::eq(app, target)),
    )
    .unwrap_or_else(|| panic!("app is not part of the resolver-owned program tree"))
}

pub(crate) fn semantic_task_identity(program: &Program, target: &Task) -> String {
    semantic_item_identity(
        program,
        |item| matches!(item, Item::Task(task) if std::ptr::eq(task, target)),
    )
    .unwrap_or_else(|| panic!("task is not part of the resolver-owned program tree"))
}

pub(crate) fn semantic_task_definition_identity(program: &Program, target: &Task) -> String {
    let task_node = semantic_task_identity(program, target);
    semantic_definition_identities_by_item(program)
        .remove(&task_node)
        .unwrap_or_else(|| panic!("task semantic node lacks a resolver definition"))
}

pub(crate) fn resolver_task_definition_id(program: &Program, target: &Task) -> String {
    let task_node = semantic_task_identity(program, target);
    let report = build_report(program, &[]);
    report
        .item_node_by_definition_id
        .iter()
        .find_map(|(definition_id, semantic_node)| {
            (semantic_node == &task_node).then(|| definition_id.clone())
        })
        .unwrap_or_else(|| panic!("task semantic node lacks a resolver definition"))
}

fn semantic_definition_identities_by_item(program: &Program) -> BTreeMap<String, String> {
    let report = build_report(program, &[]);
    report
        .item_node_by_definition_id
        .iter()
        .filter_map(|(definition_id, semantic_node)| {
            report
                .definitions
                .iter()
                .find(|definition| &definition.id == definition_id)
                .map(|definition| {
                    (
                        semantic_node.clone(),
                        semantic_definition_identity(definition),
                    )
                })
        })
        .collect()
}

pub(crate) fn semantic_item_identity_for(program: &Program, target: &Item) -> String {
    semantic_item_identity(program, |item| std::ptr::eq(item, target))
        .unwrap_or_else(|| panic!("item is not part of the resolver-owned program tree"))
}

pub(crate) fn semantic_item_definition_identity_for(program: &Program, target: &Item) -> String {
    let item_node = semantic_item_identity_for(program, target);
    semantic_definition_identities_by_item(program)
        .remove(&item_node)
        .unwrap_or_else(|| panic!("item semantic node lacks a resolver definition"))
}

fn semantic_item_identity(
    program: &Program,
    mut matches_target: impl FnMut(&Item) -> bool,
) -> Option<String> {
    fn visit(
        items: &[Item],
        path: &mut Vec<usize>,
        matches_target: &mut impl FnMut(&Item) -> bool,
    ) -> Option<Vec<usize>> {
        for (index, item) in items.iter().enumerate() {
            path.push(index);
            if matches_target(item) {
                return Some(path.clone());
            }
            if let Item::App(app) = item
                && let Some(found) = visit(&app.items, path, matches_target)
            {
                return Some(found);
            }
            path.pop();
        }
        None
    }

    for (file_index, file) in program.files.iter().enumerate() {
        let mut path = Vec::new();
        if let Some(path) = visit(&file.items, &mut path, &mut matches_target) {
            return Some(format!(
                "resolver-item:file-{file_index}:path-{}",
                path.iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(".")
            ));
        }
    }
    None
}

pub(crate) fn diagnostic_occurrence_set_from_source(
    program: &Program,
    source_diagnostics: &[Diagnostic],
    source_occurrences: &DiagnosticOccurrenceSet,
) -> DiagnosticOccurrenceSet {
    let occurrences = build_report_with_source(program, source_diagnostics, source_occurrences)
        .diagnostic_occurrences;
    let relationships = parser_precedence_relationships(&occurrences);
    let consumptions = consume_parser_precedence(&occurrences, &relationships, relationships.len())
        .expect("parser/resolver precedence must consume exact producer relationships");
    assert_eq!(consumptions.len(), relationships.len());
    occurrences
}

pub(crate) fn parser_precedence_relationships(
    occurrences: &DiagnosticOccurrenceSet,
) -> Vec<crate::diagnostic::DiagnosticPrecedenceRelationship> {
    let facts = occurrences.occurrences().collect::<Vec<_>>();
    let mut relationships = Vec::new();
    for dominant in &facts {
        let Some(node) = route_value(dominant.relationship_route(), "parser_semantic_node=") else {
            continue;
        };
        for suppressed in &facts {
            if route_value(suppressed.relationship_route(), "resolver_semantic_node=") != Some(node)
                || crate::diagnostic_catalog::exact_precedence_spec(
                    dominant.cause_key(),
                    suppressed.cause_key(),
                    "parser_blocks_resolver_semantic_node_v0",
                    "resolve",
                    "resolve",
                )
                .is_none()
            {
                continue;
            }
            let relationship_id = format!(
                "parser-resolver-precedence:{node}:{}:{}",
                dominant.cause_key().ordinal(),
                suppressed.cause_key().ordinal()
            );
            relationships.push(
                crate::diagnostic::DiagnosticPrecedenceRelationship::producer_owned(
                    "parser_over_resolver_v0",
                    "resolve",
                    "resolve",
                    relationship_id.clone(),
                    dominant,
                    suppressed,
                    [
                        dominant.semantic_origin().to_string(),
                        suppressed.semantic_origin().to_string(),
                    ],
                    vec![relationship_id, format!("semantic_node={node}")],
                )
                .expect("resolver must seal exact parser precedence"),
            );
        }
    }
    relationships
}

fn route_value<'a>(route: &'a [String], prefix: &str) -> Option<&'a str> {
    route.iter().find_map(|entry| entry.strip_prefix(prefix))
}

pub(crate) fn consume_parser_precedence(
    occurrences: &DiagnosticOccurrenceSet,
    relationships: &[crate::diagnostic::DiagnosticPrecedenceRelationship],
    expected_count: usize,
) -> Result<
    Vec<crate::diagnostic::DiagnosticPrecedenceConsumption>,
    crate::diagnostic::DiagnosticInvariantError,
> {
    occurrences.consume_precedence_relationships("resolve", relationships, expected_count)
}

pub fn resolve_text(program: &Program, source_diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, source_diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum resolver report ({RESOLVE_REPORT_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {RESOLVE_MODE}\n"));
    out.push_str(&format!(
        "summary: files={} items={} scopes={} definitions={} references={} resolved={} unresolved={} external={} duplicate_definitions={} mutable_place_errors={} resolver_errors={} source_errors={}\n",
        report.files,
        report.items,
        report.scopes.len(),
        report.definitions.len(),
        report.references.len(),
        report.resolved_references(),
        report.unresolved_references(),
        report.external_references(),
        report.duplicate_definitions(),
        report.mutable_place_errors(),
        report.error_count(),
        report.source_errors
    ));

    if report.diagnostics.is_empty() {
        out.push_str("diagnostics: none\n");
    } else {
        out.push_str("diagnostics:\n");
        for diagnostic in &report.diagnostics {
            out.push_str(&format!(
                "  {}:{}:{}: {}[{}]: {}\n",
                diagnostic.source_span.file,
                diagnostic.source_span.line,
                diagnostic.source_span.column,
                diagnostic.severity.as_str(),
                diagnostic.code.as_str(),
                diagnostic.message
            ));
            out.push_str(&format!("    help: {}\n", diagnostic.help));
        }
    }

    out.push_str(&predicate::analyze_program(program).place_facts_text());
    out.push_str("non_claims:\n");
    for non_claim in NON_CLAIMS {
        out.push_str(&format!("  - {non_claim}\n"));
    }

    out
}

pub fn resolve_json(program: &Program, source_diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, source_diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", RESOLVE_REPORT_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", RESOLVE_MODE, true);
    push_summary(&mut out, 2, &report, true);
    push_scopes(&mut out, 2, &report.scopes, true);
    push_definitions(&mut out, 2, &report.definitions, true);
    push_references(&mut out, 2, &report.references, true);
    push_diagnostics(&mut out, 2, &report.diagnostics, true);
    push_indent(&mut out, 2);
    push_json_string(&mut out, "predicate_place_facts");
    out.push_str(": ");
    out.push_str(&predicate::analyze_program(program).place_facts_json());
    out.push_str(",\n");
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, source_diagnostics: &[Diagnostic]) -> ResolveReport {
    build_report_with_source(
        program,
        source_diagnostics,
        &DiagnosticOccurrenceSet::default(),
    )
}

fn build_report_with_source(
    program: &Program,
    source_diagnostics: &[Diagnostic],
    source_occurrences: &DiagnosticOccurrenceSet,
) -> ResolveReport {
    retain_legacy_call_projection_api_compatibility();
    let mut context = ResolverContext::new(program);
    let mut file_scopes = Vec::new();
    for (file_index, file) in program.files.iter().enumerate() {
        let file_scope = context.add_scope(ScopeInput {
            parent_scope_id: None,
            scope_kind: "file",
            owner_kind: "file",
            owner_name: &file.path,
            span: file.items.first().map(|item| item.span()),
            preferred_id: format!("file_{file_index}_{}", id_fragment(&file.path)),
            semantic_origin: format!("semantic-file:{file_index}"),
        });
        context.collect_item_definitions(&file.items, &file_scope, file_index, &[]);
        file_scopes.push(file_scope);
    }
    for (file_index, (file, file_scope)) in program.files.iter().zip(&file_scopes).enumerate() {
        context.resolve_items(&file.items, file_scope, file_index, &[]);
    }

    let source_errors = source_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let source_warnings = source_diagnostics.len().saturating_sub(source_errors);

    let mut diagnostic_occurrences = source_occurrences.inherited();
    for diagnostic in &context.diagnostics {
        let mut route = vec![format!("resolver_reason={}", diagnostic.reason)];
        if let Some(reference_id) = &diagnostic.reference_id {
            route.push(format!("resolver_reference={reference_id}"));
        }
        if let Some(definition_id) = &diagnostic.definition_id {
            route.push(format!("resolver_definition={definition_id}"));
        }
        let semantic_origin = diagnostic
            .definition_id
            .as_ref()
            .and_then(|definition_id| {
                context
                    .item_node_by_definition_id
                    .get(definition_id)
                    .cloned()
            })
            .or_else(|| {
                diagnostic
                    .reference_id
                    .as_ref()
                    .map(|reference_id| format!("resolver-reference:{reference_id}"))
            })
            .or_else(|| diagnostic.definition_id.clone())
            .unwrap_or_else(|| panic!("resolver diagnostic lacks semantic definition/reference"));
        route.push(format!("resolver_semantic_node={semantic_origin}"));
        let registered = DiagnosticOccurrence::registered_cause(
            diagnostic.cause_key,
            Diagnostic::error(
                diagnostic.code,
                diagnostic.message.clone(),
                Some(diagnostic.source_span.clone()),
            )
            .with_help(diagnostic.help),
            semantic_origin,
            route,
        )
        .expect("resolver diagnostic cause must be registered");
        diagnostic_occurrences
            .insert_owned(registered)
            .expect("resolver diagnostic occurrences must be unique");
    }

    ResolveReport {
        files: program.files.len(),
        items: count_items(program),
        source_errors,
        source_warnings,
        scopes: context.scopes,
        definitions: context.definitions,
        references: context.references,
        call_occurrences: context.call_occurrences,
        item_node_by_definition_id: context.item_node_by_definition_id,
        diagnostics: context.diagnostics,
        diagnostic_occurrences,
    }
}

fn retain_legacy_call_projection_api_compatibility() {
    let _projection: fn(&[ParsedBodyStatement]) -> Vec<crate::parser::ParsedExecutableCallNode> =
        crate::parser::executable_call_nodes;
    let _statement_index: fn(&crate::parser::ParsedCallPosition) -> usize =
        crate::parser::ParsedCallPosition::statement_index;
    let _stable_component: fn(&crate::parser::ParsedCallPosition) -> String =
        crate::parser::ParsedCallPosition::stable_component;
}

impl<'program> ResolverContext<'program> {
    fn new(program: &'program Program) -> Self {
        Self {
            program,
            scopes: Vec::new(),
            definitions: Vec::new(),
            references: Vec::new(),
            call_occurrences: Vec::new(),
            diagnostics: Vec::new(),
            scope_parents: BTreeMap::new(),
            scope_semantic_identities: BTreeMap::new(),
            definitions_by_scope_name: BTreeMap::new(),
            scope_serial: 0,
            definition_serial: 0,
            reference_serial: 0,
            callable_receiver_definition_ids: BTreeSet::new(),
            callable_parameter_definition_ids: BTreeSet::new(),
            task_definition_ids_by_name: BTreeMap::new(),
            item_node_by_definition_id: BTreeMap::new(),
        }
    }

    fn add_scope(&mut self, input: ScopeInput<'_>) -> String {
        let ScopeInput {
            parent_scope_id,
            scope_kind,
            owner_kind,
            owner_name,
            span,
            preferred_id,
            semantic_origin,
        } = input;
        let current = self.scope_serial;
        self.scope_serial += 1;
        let base = if preferred_id.is_empty() {
            format!("scope_{scope_kind}_{}", id_fragment(owner_name))
        } else {
            preferred_id
        };
        let source_identity = span.map_or_else(
            || "generated".to_string(),
            |span| {
                format!(
                    "{}_{}_{}",
                    id_fragment(&portable_path(&span.file)),
                    span.line,
                    span.column
                )
            },
        );
        let id = format!("{base}_{source_identity}_{current}");
        let parent = parent_scope_id.map(str::to_string);
        let parent_semantic_identity = parent_scope_id
            .and_then(|parent| self.scope_semantic_identities.get(parent))
            .cloned()
            .unwrap_or_else(|| "resolver-root".to_string());
        let owner_semantic_identity = semantic_origin;
        let semantic_identity = format!(
            "resolver-scope|parent={parent_semantic_identity}|owner={owner_semantic_identity}|kind={scope_kind}|owner-kind={owner_kind}"
        );
        self.scope_parents.insert(id.clone(), parent.clone());
        self.scope_semantic_identities
            .insert(id.clone(), semantic_identity.clone());
        self.scopes.push(ResolveScope {
            id: id.clone(),
            parent_scope_id: parent,
            scope_kind,
            owner_kind,
            owner_name: portable_path(owner_name.trim()),
            source_span: span.map(portable_span),
            semantic_identity,
            owner_semantic_identity,
        });
        id
    }

    fn collect_item_definitions(
        &mut self,
        items: &[Item],
        scope_id: &str,
        file_index: usize,
        parent_path: &[usize],
    ) {
        for (item_index, item) in items.iter().enumerate() {
            let mut path = parent_path.to_vec();
            path.push(item_index);
            let semantic_origin = format!(
                "resolver-item:file-{file_index}:path-{}",
                path.iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(".")
            );
            let (definition_kind, mutable, state_kind) = match item {
                Item::App(_) => ("app", false, "item"),
                Item::Type(_) => ("type", false, "type"),
                Item::Store(_) => ("store", true, "store"),
                Item::Task(_) => ("task", false, "callable"),
                Item::Test(_) => ("test", false, "callable"),
            };
            let definition_id = self.add_definition(
                scope_id,
                DefinitionInput {
                    name: item.name(),
                    definition_kind,
                    mutable,
                    state_kind,
                    span: item.span(),
                    defer_duplicate_diagnostic: false,
                    semantic_origin: semantic_origin.clone(),
                    semantic_shape: Some(item_definition_semantic_shape(item)),
                },
            );
            if let Some(definition_id) = definition_id {
                self.item_node_by_definition_id
                    .insert(definition_id.clone(), semantic_origin);
                if let Item::Task(task) = item {
                    self.task_definition_ids_by_name
                        .entry(name_key(&task.name))
                        .or_default()
                        .push(definition_id.clone());
                    if task.params.iter().any(|param| {
                        matches!(
                            param.type_syntax.kind,
                            TypeSyntaxKind::Callable(_) | TypeSyntaxKind::CallableCandidate { .. }
                        )
                    }) {
                        self.callable_receiver_definition_ids.insert(definition_id);
                    }
                }
            }
        }
    }

    fn semantic_item_definition_identity(&self, file_index: usize, item_path: &[usize]) -> String {
        let item_node = format!(
            "resolver-item:file-{file_index}:path-{}",
            item_path
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(".")
        );
        let definition_id = self
            .item_node_by_definition_id
            .iter()
            .find_map(|(definition_id, node)| (node == &item_node).then_some(definition_id))
            .unwrap_or_else(|| panic!("missing resolver definition for semantic node {item_node}"));
        let definition = self
            .definitions
            .iter()
            .find(|definition| &definition.id == definition_id)
            .expect("semantic item definition must be registered");
        semantic_definition_identity(definition)
    }

    fn resolve_items(
        &mut self,
        items: &[Item],
        scope_id: &str,
        file_index: usize,
        parent_path: &[usize],
    ) {
        for (item_index, item) in items.iter().enumerate() {
            let mut item_path = parent_path.to_vec();
            item_path.push(item_index);
            match item {
                Item::App(app) => {
                    let owner_semantic_identity =
                        self.semantic_item_definition_identity(file_index, &item_path);
                    let app_scope = self.add_scope(ScopeInput {
                        parent_scope_id: Some(scope_id),
                        scope_kind: "app",
                        owner_kind: "app",
                        owner_name: &app.name,
                        span: Some(&app.span),
                        preferred_id: format!("app_{}_scope", id_fragment(&app.name)),
                        semantic_origin: owner_semantic_identity,
                    });
                    self.collect_item_definitions(&app.items, &app_scope, file_index, &item_path);
                    self.resolve_items(&app.items, &app_scope, file_index, &item_path);
                }
                Item::Type(type_def) => {
                    let owner_semantic_identity =
                        self.semantic_item_definition_identity(file_index, &item_path);
                    let type_scope = self.add_scope(ScopeInput {
                        parent_scope_id: Some(scope_id),
                        scope_kind: "type",
                        owner_kind: "type",
                        owner_name: &type_def.name,
                        span: Some(&type_def.span),
                        preferred_id: format!("type_{}_scope", id_fragment(&type_def.name)),
                        semantic_origin: owner_semantic_identity,
                    });
                    for (field_index, field) in type_def.fields.iter().enumerate() {
                        self.add_definition(
                            &type_scope,
                            DefinitionInput {
                                name: &field.name,
                                definition_kind: "field",
                                mutable: false,
                                state_kind: "field",
                                span: &field.span,
                                defer_duplicate_diagnostic: false,
                                semantic_origin: format!("field-position:{field_index}"),
                                semantic_shape: Some(format!("field-position:{field_index}")),
                            },
                        );
                    }
                }
                Item::Store(_) => {}
                Item::Task(task) => {
                    let owner_definition_id =
                        self.semantic_item_definition_identity(file_index, &item_path);
                    self.resolve_callable(
                        scope_id,
                        CallableResolveInput {
                            item,
                            owner_kind: "task",
                            owner_name: &task.name,
                            owner_definition_id,
                            params: &task.params,
                            sections: &task.sections,
                            span: &task.span,
                            resolve_canonical_body: true,
                        },
                    );
                }
                Item::Test(test) => {
                    let owner_definition_id =
                        self.semantic_item_definition_identity(file_index, &item_path);
                    self.resolve_callable(
                        scope_id,
                        CallableResolveInput {
                            item,
                            owner_kind: "test",
                            owner_name: &test.name,
                            owner_definition_id,
                            params: &test.params,
                            sections: &test.sections,
                            span: &test.span,
                            resolve_canonical_body: false,
                        },
                    );
                }
            }
        }
    }

    fn resolve_callable(&mut self, parent_scope_id: &str, input: CallableResolveInput<'_>) {
        let CallableResolveInput {
            item,
            owner_kind,
            owner_name,
            owner_definition_id,
            params,
            sections,
            span,
            resolve_canonical_body,
        } = input;
        let callable_scope = self.add_scope(ScopeInput {
            parent_scope_id: Some(parent_scope_id),
            scope_kind: "callable",
            owner_kind,
            owner_name,
            span: Some(span),
            preferred_id: format!("{}_{}_scope", owner_kind, id_fragment(owner_name)),
            semantic_origin: owner_definition_id.clone(),
        });
        for (param_index, param) in params.iter().enumerate() {
            let definition_id = self.add_definition(
                &callable_scope,
                DefinitionInput {
                    name: &param.name,
                    definition_kind: "parameter",
                    mutable: parameter_is_mutable(param),
                    state_kind: parameter_state_kind(param),
                    span: &param.span,
                    defer_duplicate_diagnostic: false,
                    semantic_origin: format!("parameter-position:{param_index}"),
                    semantic_shape: Some(format!("parameter-position:{param_index}")),
                },
            );
            if matches!(
                param.type_syntax.kind,
                TypeSyntaxKind::Callable(_) | TypeSyntaxKind::CallableCandidate { .. }
            ) && let Some(definition_id) = definition_id
            {
                self.callable_parameter_definition_ids.insert(definition_id);
            }
        }

        self.resolve_declared_sections(&callable_scope, sections);

        if let Some(section) = find_section(sections, "does") {
            let body = core_body::analyze_does_section(
                self.program
                    .canonical_core_expectation(item, section)
                    .expect("live resolver item must have parser authority"),
            );
            let canonical_body = section
                .body_syntax
                .iter()
                .flatten()
                .cloned()
                .collect::<Vec<_>>();
            if resolve_canonical_body {
                self.resolve_canonical_call_occurrences(
                    &callable_scope,
                    &owner_definition_id,
                    &canonical_body,
                );
            }
            let statement_scopes = self.resolve_does_section(&callable_scope, section, &body);
            if resolve_canonical_body {
                self.resolve_canonical_callable_references(
                    &callable_scope,
                    &canonical_body,
                    &statement_scopes,
                );
            }
        }
    }

    fn resolve_canonical_callable_references(
        &mut self,
        root_scope_id: &str,
        statements: &[ParsedBodyStatement],
        statement_scopes: &[String],
    ) {
        for (statement_index, statement) in statements.iter().enumerate() {
            let scope_id = statement_scopes
                .get(statement_index)
                .map_or(root_scope_id, String::as_str);
            for (root_ordinal, expression) in
                parsed_statement_roots(statement).into_iter().enumerate()
            {
                self.resolve_canonical_callable_expression(
                    scope_id,
                    expression,
                    false,
                    &CanonicalExpressionPosition {
                        statement_node_id: statement.source_node_id.as_str().to_string(),
                        statement_index,
                        root_ordinal,
                        child_path: vec![root_ordinal],
                        node_id: expression.node_id.as_str().to_string(),
                    },
                );
            }
        }
    }

    fn resolve_canonical_callable_expression(
        &mut self,
        scope_id: &str,
        expression: &CanonicalExpression,
        argument_position: bool,
        position: &CanonicalExpressionPosition,
    ) {
        match &expression.kind {
            CanonicalExpressionKind::Identifier(name) => {
                let resolved = self
                    .resolve_definition(scope_id, &name_key(name), "callable_value_ref")
                    .map(|definition| definition.id.clone());
                if argument_position
                    || resolved.as_ref().is_some_and(|definition_id| {
                        self.callable_parameter_definition_ids
                            .contains(definition_id)
                    })
                {
                    self.add_canonical_reference(
                        scope_id,
                        PendingReferenceInput {
                            name,
                            reference_kind: if argument_position {
                                "callable_argument_ref"
                            } else {
                                "callable_value_ref"
                            },
                            mutable_required: false,
                            external_if_unresolved: false,
                            span: &expression.range.start,
                        },
                        expression,
                        position,
                    );
                }
            }
            CanonicalExpressionKind::Call { callee, arguments } => {
                let mut callable_argument_positions = false;
                let callee_position = child_position(position, callee, 0);
                if let CanonicalExpressionKind::Identifier(name) = &callee.kind {
                    let normalized = name_key(name);
                    let target = self
                        .resolve_definition(scope_id, &normalized, "callable_callee_ref")
                        .map(|definition| definition.id.clone())
                        .or_else(|| {
                            self.unique_cross_file_callable_receiver(scope_id, &normalized)
                        });
                    if let Some(target) = target {
                        let direct_callable_receiver =
                            self.callable_receiver_definition_ids.contains(&target);
                        let indirect_callable_parameter =
                            self.callable_parameter_definition_ids.contains(&target);
                        callable_argument_positions =
                            direct_callable_receiver || indirect_callable_parameter;
                        if direct_callable_receiver || indirect_callable_parameter {
                            self.add_canonical_reference(
                                scope_id,
                                PendingReferenceInput {
                                    name,
                                    reference_kind: "callable_callee_ref",
                                    mutable_required: false,
                                    external_if_unresolved: false,
                                    span: &callee.range.start,
                                },
                                callee,
                                &callee_position,
                            );
                        }
                    }
                }
                for (argument_index, argument) in arguments.iter().enumerate() {
                    self.resolve_canonical_callable_expression(
                        scope_id,
                        argument,
                        callable_argument_positions,
                        &child_position(position, argument, argument_index + 1),
                    );
                }
            }
            CanonicalExpressionKind::Permission { value, .. }
            | CanonicalExpressionKind::Group(value) => {
                self.resolve_canonical_callable_expression(
                    scope_id,
                    value,
                    argument_position,
                    &child_position(position, value, 0),
                );
            }
            CanonicalExpressionKind::Try {
                value,
                failure_root,
                failure_variant,
            } => {
                let value_position = child_position(position, value, 0);
                let (semantic_value, semantic_position) = canonical_try_semantic_value(
                    value,
                    failure_root,
                    failure_variant,
                    &value_position,
                );
                self.resolve_canonical_callable_expression(
                    scope_id,
                    semantic_value,
                    argument_position,
                    &semantic_position,
                );
            }
            CanonicalExpressionKind::Field { base, .. }
            | CanonicalExpressionKind::ElementPlace { base, .. } => {
                self.resolve_canonical_callable_expression(
                    scope_id,
                    base,
                    argument_position,
                    &child_position(position, base, 0),
                );
            }
            CanonicalExpressionKind::ListLiteral(values) => {
                for (index, value) in values.iter().enumerate() {
                    self.resolve_canonical_callable_expression(
                        scope_id,
                        value,
                        argument_position,
                        &child_position(position, value, index),
                    );
                }
            }
            CanonicalExpressionKind::RecordLiteral { fields, .. } => {
                for (index, (_name, value)) in fields.iter().enumerate() {
                    self.resolve_canonical_callable_expression(
                        scope_id,
                        value,
                        argument_position,
                        &child_position(position, value, index),
                    );
                }
            }
            CanonicalExpressionKind::Binary { left, right, .. } => {
                for (index, value) in [left.as_ref(), right.as_ref()].into_iter().enumerate() {
                    self.resolve_canonical_callable_expression(
                        scope_id,
                        value,
                        argument_position,
                        &child_position(position, value, index),
                    );
                }
            }
            CanonicalExpressionKind::Unit
            | CanonicalExpressionKind::UIntLiteral(_)
            | CanonicalExpressionKind::IntLiteral(_)
            | CanonicalExpressionKind::BoolLiteral(_)
            | CanonicalExpressionKind::TextLiteral(_)
            | CanonicalExpressionKind::Unsupported => {}
        }
    }

    fn resolve_canonical_value_expression(
        &mut self,
        scope_id: &str,
        expression: &CanonicalExpression,
        position: &CanonicalExpressionPosition,
        path_root: bool,
    ) {
        match &expression.kind {
            CanonicalExpressionKind::Identifier(name) => {
                self.add_canonical_reference(
                    scope_id,
                    PendingReferenceInput {
                        name,
                        reference_kind: if path_root {
                            "path_root_ref"
                        } else {
                            "name_ref"
                        },
                        mutable_required: false,
                        external_if_unresolved: path_root && is_external_root(name),
                        span: &expression.range.start,
                    },
                    expression,
                    position,
                );
            }
            CanonicalExpressionKind::Field { base, .. }
            | CanonicalExpressionKind::ElementPlace { base, .. } => {
                self.resolve_canonical_value_expression(
                    scope_id,
                    base,
                    &child_position(position, base, 0),
                    true,
                );
            }
            CanonicalExpressionKind::Call {
                callee: _,
                arguments,
            } => {
                for (index, argument) in arguments.iter().enumerate() {
                    self.resolve_canonical_value_expression(
                        scope_id,
                        argument,
                        &child_position(position, argument, index + 1),
                        false,
                    );
                }
            }
            CanonicalExpressionKind::Permission { value, .. }
            | CanonicalExpressionKind::Group(value) => self.resolve_canonical_value_expression(
                scope_id,
                value,
                &child_position(position, value, 0),
                path_root,
            ),
            CanonicalExpressionKind::Try {
                value,
                failure_root,
                failure_variant,
            } => {
                let value_position = child_position(position, value, 0);
                let (semantic_value, semantic_position) = canonical_try_semantic_value(
                    value,
                    failure_root,
                    failure_variant,
                    &value_position,
                );
                self.resolve_canonical_value_expression(
                    scope_id,
                    semantic_value,
                    &semantic_position,
                    path_root,
                );
            }
            CanonicalExpressionKind::ListLiteral(values) => {
                for (index, value) in values.iter().enumerate() {
                    self.resolve_canonical_value_expression(
                        scope_id,
                        value,
                        &child_position(position, value, index),
                        false,
                    );
                }
            }
            CanonicalExpressionKind::RecordLiteral { fields, .. } => {
                for (index, (_field, value)) in fields.iter().enumerate() {
                    self.resolve_canonical_value_expression(
                        scope_id,
                        value,
                        &child_position(position, value, index),
                        false,
                    );
                }
            }
            CanonicalExpressionKind::Binary {
                operator,
                left,
                right,
            } => {
                self.resolve_canonical_value_expression(
                    scope_id,
                    left,
                    &child_position(position, left, 0),
                    false,
                );
                if *operator != ParsedBinaryOperator::Is {
                    self.resolve_canonical_value_expression(
                        scope_id,
                        right,
                        &child_position(position, right, 1),
                        false,
                    );
                }
            }
            CanonicalExpressionKind::Unit
            | CanonicalExpressionKind::UIntLiteral(_)
            | CanonicalExpressionKind::IntLiteral(_)
            | CanonicalExpressionKind::BoolLiteral(_)
            | CanonicalExpressionKind::TextLiteral(_)
            | CanonicalExpressionKind::Unsupported => {}
        }
    }

    fn unique_cross_file_callable_receiver(
        &self,
        scope_id: &str,
        normalized_name: &str,
    ) -> Option<String> {
        let matches = self
            .task_definition_ids_by_name
            .get(normalized_name)?
            .iter()
            .filter(|definition_id| {
                self.callable_receiver_definition_ids
                    .contains(*definition_id)
                    && self.definition_is_in_another_file(scope_id, definition_id)
            })
            .collect::<Vec<_>>();
        (matches.len() == 1).then(|| matches[0].clone())
    }

    fn definition_is_in_another_file(&self, scope_id: &str, definition_id: &str) -> bool {
        let Some(scope_file) = self
            .scopes
            .iter()
            .find(|scope| scope.id == scope_id)
            .and_then(|scope| scope.source_span.as_ref())
            .map(|span| portable_path(&span.file))
        else {
            return false;
        };
        self.definitions
            .iter()
            .find(|definition| definition.id == definition_id)
            .is_some_and(|definition| portable_path(&definition.source_span.file) != scope_file)
    }

    fn resolve_declared_sections(&mut self, scope_id: &str, sections: &[Section]) {
        for section in sections {
            let (reference_kind, definition_kind, mutable, state_kind) = match section.name.as_str()
            {
                "uses" => (
                    "declared_use",
                    "declared_use_permission",
                    false,
                    "external_state",
                ),
                "changes" => (
                    "declared_change",
                    "declared_change_permission",
                    true,
                    "external_state",
                ),
                _ => continue,
            };
            for (line_index, line) in section.lines.iter().enumerate() {
                if !is_meaningful_line_text(&line.text) {
                    continue;
                }
                let Some(name) = declared_name_from_line(&line.text) else {
                    continue;
                };
                self.add_reference(
                    scope_id,
                    PendingReferenceInput {
                        name: &name,
                        reference_kind,
                        mutable_required: false,
                        external_if_unresolved: true,
                        span: &line.span,
                    },
                );
                self.add_definition(
                    scope_id,
                    DefinitionInput {
                        name: &name,
                        definition_kind,
                        mutable,
                        state_kind,
                        span: &line.span,
                        defer_duplicate_diagnostic: false,
                        semantic_origin: format!(
                            "declared-section:{}:line-position:{line_index}",
                            section.name
                        ),
                        semantic_shape: Some(format!(
                            "declared-{definition_kind}-position:{line_index}"
                        )),
                    },
                );
            }
        }
    }

    fn resolve_canonical_call_occurrences(
        &mut self,
        scope_id: &str,
        owner_definition_id: &str,
        statements: &[ParsedBodyStatement],
    ) {
        for call in canonical_call_nodes(statements) {
            #[cfg(test)]
            let call = {
                let mut call = call;
                TEN_B2_RESOLVER_BOUNDARY_CORRUPTION.with(|slot| {
                    if let Some(corruption) = slot.borrow().as_ref()
                        && call.expression.node_id.as_str() == corruption.call_node_id
                    {
                        call.position.child_path = corruption.replacement_child_path.clone();
                    }
                });
                call
            };
            self.add_canonical_executable_call(scope_id, owner_definition_id, &call);
        }
    }

    fn resolve_does_section(
        &mut self,
        root_scope_id: &str,
        section: &Section,
        body: &core_body::BodyGrammarReport,
    ) -> Vec<String> {
        let existing_names = self
            .definitions_by_scope_name
            .keys()
            .filter(|(scope_id, _name)| scope_id == root_scope_id)
            .map(|(_scope_id, name)| name.clone())
            .collect::<BTreeSet<_>>();
        let alias_analysis =
            writable_field_alias::analyze_with_existing_names(&body.statements, &existing_names);
        let mut active_scopes = vec![root_scope_id.to_string()];
        let mut statement_scopes = Vec::with_capacity(body.statements.len());
        let mut record_literal_depth = 0usize;
        for (statement_index, (statement, parsed_statement)) in body
            .statements
            .iter()
            .zip(section.body_syntax.iter().flatten())
            .enumerate()
        {
            if statement.kind == "block_close" {
                if record_literal_depth > 0 {
                    record_literal_depth -= 1;
                } else if active_scopes.len() > 1 {
                    active_scopes.pop();
                }
                statement_scopes.push(
                    active_scopes
                        .last()
                        .cloned()
                        .unwrap_or_else(|| root_scope_id.to_string()),
                );
                continue;
            }

            let current_scope = active_scopes
                .last()
                .cloned()
                .unwrap_or_else(|| root_scope_id.to_string());
            statement_scopes.push(current_scope.clone());
            self.resolve_canonical_statement_references(
                &current_scope,
                parsed_statement,
                statement_index,
            );

            match canonical_statement_kind(parsed_statement) {
                Some(CanonicalStatementKindEvent::ImmutableBinding) => {
                    if let Some((name, span)) = canonical_statement_token(
                        parsed_statement,
                        CanonicalStatementEventField::Binder,
                    ) {
                        let writable_alias =
                            writable_field_alias::candidate_name(statement).is_some();
                        let alias_rebinding = alias_analysis.issues.iter().any(|issue| {
                            issue.index == statement_index
                                && matches!(
                                    issue.cause,
                                    writable_field_alias::AliasCause::Rebinding
                                        | writable_field_alias::AliasCause::BindingRebinding
                                        | writable_field_alias::AliasCause::OwnerRebinding
                                        | writable_field_alias::AliasCause::RebindsItsOwner
                                )
                        });
                        self.add_definition(
                            &current_scope,
                            DefinitionInput {
                                name,
                                definition_kind: if writable_alias {
                                    "writable_field_alias"
                                } else {
                                    "let_binding"
                                },
                                mutable: writable_alias,
                                state_kind: if writable_alias {
                                    "writable_field_alias"
                                } else {
                                    "immutable_value"
                                },
                                span,
                                defer_duplicate_diagnostic: alias_rebinding,
                                semantic_origin: format!(
                                    "statement-node:{}",
                                    parsed_statement.source_node_id.as_str()
                                ),
                                semantic_shape: Some(format!(
                                    "statement-node:{}",
                                    parsed_statement.source_node_id.as_str()
                                )),
                            },
                        );
                    }
                }
                Some(CanonicalStatementKindEvent::MutableBinding) => {
                    if let Some((name, span)) = canonical_statement_token(
                        parsed_statement,
                        CanonicalStatementEventField::Binder,
                    ) {
                        self.add_definition(
                            &current_scope,
                            DefinitionInput {
                                name,
                                definition_kind: "mutable_binding",
                                mutable: true,
                                state_kind: "mutable_local",
                                span,
                                defer_duplicate_diagnostic: false,
                                semantic_origin: format!(
                                    "statement-node:{}",
                                    parsed_statement.source_node_id.as_str()
                                ),
                                semantic_shape: Some(format!(
                                    "statement-node:{}",
                                    parsed_statement.source_node_id.as_str()
                                )),
                            },
                        );
                    }
                }
                Some(CanonicalStatementKindEvent::ForEach) => {
                    let block_scope = self.open_block_scope(
                        root_scope_id,
                        &current_scope,
                        "for_each",
                        statement_index,
                        &statement.span,
                        parsed_statement.source_node_id.as_str(),
                    );
                    if let Some((name, span)) = canonical_statement_token(
                        parsed_statement,
                        CanonicalStatementEventField::Binder,
                    ) {
                        self.add_definition(
                            &block_scope,
                            DefinitionInput {
                                name,
                                definition_kind: "for_each_binding",
                                mutable: false,
                                state_kind: "immutable_value",
                                span,
                                defer_duplicate_diagnostic: false,
                                semantic_origin: format!(
                                    "statement-node:{}",
                                    parsed_statement.source_node_id.as_str()
                                ),
                                semantic_shape: Some(format!(
                                    "statement-node:{}",
                                    parsed_statement.source_node_id.as_str()
                                )),
                            },
                        );
                    }
                    active_scopes.push(block_scope);
                }
                Some(
                    CanonicalStatementKindEvent::ForIndexUntil
                    | CanonicalStatementKindEvent::ForIndexThrough,
                ) => {
                    let block_scope = self.open_block_scope(
                        root_scope_id,
                        &current_scope,
                        "for_index",
                        statement_index,
                        &statement.span,
                        parsed_statement.source_node_id.as_str(),
                    );
                    if let Some((name, span)) = canonical_statement_token(
                        parsed_statement,
                        CanonicalStatementEventField::Binder,
                    ) {
                        self.add_definition(
                            &block_scope,
                            DefinitionInput {
                                name,
                                definition_kind: "for_index_binding",
                                mutable: false,
                                state_kind: "immutable_value",
                                span,
                                defer_duplicate_diagnostic: false,
                                semantic_origin: format!(
                                    "statement-node:{}",
                                    parsed_statement.source_node_id.as_str()
                                ),
                                semantic_shape: Some(format!(
                                    "statement-node:{}",
                                    parsed_statement.source_node_id.as_str()
                                )),
                            },
                        );
                    }
                    active_scopes.push(block_scope);
                }
                Some(
                    kind @ (CanonicalStatementKindEvent::If
                    | CanonicalStatementKindEvent::While
                    | CanonicalStatementKindEvent::UnconditionalLoop),
                ) => {
                    let block_kind = match kind {
                        CanonicalStatementKindEvent::If => "if",
                        CanonicalStatementKindEvent::While => "while",
                        CanonicalStatementKindEvent::UnconditionalLoop => "loop",
                        _ => unreachable!("matched block kind"),
                    };
                    let block_scope = self.open_block_scope(
                        root_scope_id,
                        &current_scope,
                        block_kind,
                        statement_index,
                        &statement.span,
                        parsed_statement.source_node_id.as_str(),
                    );
                    active_scopes.push(block_scope);
                }
                _ => {}
            }

            if statement.expression_kind == Some("record_literal_start") {
                record_literal_depth += 1;
            }
        }
        assert_eq!(
            body.statements.len(),
            section.body_syntax.iter().flatten().count(),
            "validated Core statements must preserve parser statement cardinality"
        );
        statement_scopes
    }

    fn open_block_scope(
        &mut self,
        root_scope_id: &str,
        parent_scope_id: &str,
        block_kind: &'static str,
        statement_index: usize,
        span: &Span,
        statement_node_id: &str,
    ) -> String {
        self.add_scope(ScopeInput {
            parent_scope_id: Some(parent_scope_id),
            scope_kind: block_kind,
            owner_kind: "block",
            owner_name: block_kind,
            span: Some(span),
            preferred_id: format!(
                "{}_block_{}_{}_scope",
                root_scope_id, statement_index, block_kind
            ),
            semantic_origin: format!("statement-node:{statement_node_id}"),
        })
    }

    fn add_canonical_executable_call(
        &mut self,
        scope_id: &str,
        owner_definition_id: &str,
        call: &CanonicalCallNode<'_>,
    ) {
        let CanonicalExpressionKind::Identifier(callee_name) = &call.callee.kind else {
            unreachable!("canonical executable call has identifier callee")
        };
        let public_reference_id = self
            .add_canonical_reference(
                scope_id,
                PendingReferenceInput {
                    name: callee_name,
                    reference_kind: "callee_ref",
                    mutable_required: false,
                    external_if_unresolved: true,
                    span: &call.callee.range.start,
                },
                call.callee,
                &child_position(&call.position, call.callee, 0),
            )
            .expect("canonical executable call must produce a resolver reference");
        let reference = self
            .references
            .iter()
            .find(|reference| reference.id == public_reference_id)
            .cloned()
            .expect("new resolver call reference must be registered");
        let target_definition_id = reference
            .resolved_definition_semantic_identity
            .clone()
            .unwrap_or_else(|| {
                if reference.resolution_status == "builtin_reference_v0" {
                    format!("builtin_{}", reference.normalized_name)
                } else {
                    unresolved_call_target_identity(
                        owner_definition_id,
                        reference.resolution_status,
                        &call.position,
                    )
                }
            });
        let mut canonical_identifier_uses = Vec::new();
        for argument in call.arguments {
            collect_canonical_call_identifier_uses(argument, false, &mut canonical_identifier_uses);
        }
        let identifier_uses = canonical_identifier_uses
            .into_iter()
            .enumerate()
            .map(|(ordinal, identifier)| ResolveCallIdentifierUse {
                name: identifier.name.to_string(),
                resolved_definition_id: self
                    .resolve_definition(scope_id, &name_key(identifier.name), "value_ref")
                    .map(semantic_definition_identity),
                ordinal,
                consumed: identifier.consumed,
            })
            .collect();
        let call_reference_id = resolver_call_reference_identity(
            owner_definition_id,
            &target_definition_id,
            &call.position,
        );
        let resolver_occurrence_id = ResolverCallOccurrenceId {
            reference_id: call_reference_id.clone(),
            owner_definition_id: owner_definition_id.to_string(),
            target_definition_id: target_definition_id.clone(),
            position: call.position.clone(),
        };
        self.call_occurrences.push(ResolveCallOccurrenceSummary {
            reference_id: call_reference_id,
            owner_definition_id: owner_definition_id.to_string(),
            target_definition_id,
            exact_call_span: portable_span(&call.expression.range.start),
            resolver_occurrence_id,
            source: render_canonical_expression(call.expression),
            identifier_uses,
            canonical_call_node_id: call.expression.node_id.as_str().to_string(),
            canonical_callee_node_id: call.callee.node_id.as_str().to_string(),
            canonical_argument_node_ids: call
                .arguments
                .iter()
                .map(|argument| argument.node_id.as_str().to_string())
                .collect(),
        });
    }

    fn resolve_canonical_statement_references(
        &mut self,
        scope_id: &str,
        statement: &ParsedBodyStatement,
        statement_index: usize,
    ) {
        for (root_ordinal, expression) in parsed_statement_roots(statement).into_iter().enumerate()
        {
            let position = CanonicalExpressionPosition {
                statement_node_id: statement.source_node_id.as_str().to_string(),
                statement_index,
                root_ordinal,
                child_path: vec![root_ordinal],
                node_id: expression.node_id.as_str().to_string(),
            };
            self.resolve_canonical_value_expression(scope_id, expression, &position, false);
        }

        match canonical_statement_kind(statement) {
            Some(CanonicalStatementKindEvent::Set) => {
                if let Some(target) =
                    canonical_statement_root(statement, CanonicalStatementEventField::TargetRoot)
                {
                    self.add_canonical_reference(
                        scope_id,
                        PendingReferenceInput {
                            name: canonical_place_root_name(target),
                            reference_kind: "mutation_target",
                            mutable_required: true,
                            external_if_unresolved: false,
                            span: &target.range.start,
                        },
                        target,
                        &canonical_root_position(statement, target, statement_index),
                    );
                }
            }
            Some(CanonicalStatementKindEvent::Save) => {
                if let Some(value) =
                    canonical_statement_root(statement, CanonicalStatementEventField::ValueRoot)
                {
                    self.add_canonical_reference(
                        scope_id,
                        PendingReferenceInput {
                            name: canonical_place_root_name(value),
                            reference_kind: "store_write_value",
                            mutable_required: false,
                            external_if_unresolved: false,
                            span: &value.range.start,
                        },
                        value,
                        &canonical_root_position(statement, value, statement_index),
                    );
                }
                if let Some((destination, span)) = canonical_statement_token(
                    statement,
                    CanonicalStatementEventField::DestinationToken,
                ) {
                    self.add_reference(
                        scope_id,
                        PendingReferenceInput {
                            name: destination,
                            reference_kind: "store_write_target",
                            mutable_required: true,
                            external_if_unresolved: false,
                            span,
                        },
                    );
                }
            }
            _ => {}
        }
    }

    fn add_definition(&mut self, scope_id: &str, input: DefinitionInput<'_>) -> Option<String> {
        let normalized_name = name_key(input.name);
        if normalized_name.is_empty() {
            return None;
        }
        let key = (scope_id.to_string(), normalized_name.clone());
        let duplicate = self.definitions_by_scope_name.get(&key).cloned();
        let duplicate_id = duplicate
            .as_ref()
            .map(|existing| self.definitions[existing.index].id.clone());
        let id = format!(
            "def_{}_{}_{}",
            self.definition_serial, input.definition_kind, normalized_name
        );
        self.definition_serial += 1;
        let scope_semantic_identity = self
            .scope_semantic_identities
            .get(scope_id)
            .unwrap_or_else(|| panic!("definition scope `{scope_id}` lacks semantic authority"));
        let semantic_shape = input
            .semantic_shape
            .unwrap_or_else(|| format!("definition-kind:{}", input.definition_kind));
        let semantic_origin = input.semantic_origin;
        let semantic_identity = format!(
            "resolver-definition|scope={scope_semantic_identity}|kind={}|origin={}|shape={semantic_shape}",
            input.definition_kind, semantic_origin,
        );
        let status = if duplicate.is_some() && input.defer_duplicate_diagnostic {
            "duplicate_definition_deferred_to_ownership_v0"
        } else if duplicate.is_some() {
            "duplicate_definition_v0"
        } else {
            "defined_v0"
        };
        let definition_index = self.definitions.len();
        self.definitions.push(ResolveDefinition {
            id: id.clone(),
            name: input.name.trim().to_string(),
            normalized_name: normalized_name.clone(),
            definition_kind: input.definition_kind,
            scope_id: scope_id.to_string(),
            mutable: input.mutable,
            state_kind: input.state_kind,
            source_span: portable_span(input.span),
            status,
            duplicate_of: duplicate_id.clone(),
            semantic_identity,
            semantic_origin,
        });
        if duplicate.is_none() {
            self.definitions_by_scope_name.insert(
                key,
                DefinitionRef {
                    index: definition_index,
                },
            );
        } else if !input.defer_duplicate_diagnostic {
            self.add_duplicate_definition_diagnostic(
                input.name,
                input.span,
                &id,
                duplicate_id.as_deref(),
            );
        }
        self.add_read_before_declare_diagnostics(&normalized_name, input.span, &id);
        Some(id)
    }

    fn add_reference(
        &mut self,
        scope_id: &str,
        input: PendingReferenceInput<'_>,
    ) -> Option<String> {
        let normalized_name = name_key(input.name);
        if normalized_name.is_empty() {
            return None;
        }
        let lexical_resolved = self
            .resolve_definition(scope_id, &normalized_name, input.reference_kind)
            .map(|definition| {
                (
                    definition.id.clone(),
                    definition.mutable,
                    definition.definition_kind,
                )
            });
        let cross_file_resolved = if lexical_resolved.is_none() {
            match input.reference_kind {
                "callee_ref" | "callable_callee_ref" => self
                    .unique_cross_file_callable_receiver(scope_id, &normalized_name)
                    .and_then(|definition_id| {
                        self.definitions
                            .iter()
                            .find(|definition| definition.id == definition_id)
                            .map(|definition| {
                                (
                                    definition.id.clone(),
                                    definition.mutable,
                                    definition.definition_kind,
                                )
                            })
                    }),
                "callable_argument_ref" => self
                    .task_definition_ids_by_name
                    .get(&normalized_name)
                    .map(|definitions| {
                        definitions
                            .iter()
                            .filter(|definition_id| {
                                self.definition_is_in_another_file(scope_id, definition_id)
                            })
                            .collect::<Vec<_>>()
                    })
                    .filter(|definitions| definitions.len() == 1)
                    .and_then(|definitions| {
                        let definition_id = definitions[0];
                        self.definitions
                            .iter()
                            .find(|definition| &definition.id == definition_id)
                            .map(|definition| {
                                (
                                    definition.id.clone(),
                                    definition.mutable,
                                    definition.definition_kind,
                                )
                            })
                    }),
                _ => None,
            }
        } else {
            None
        };
        let resolved = lexical_resolved.or(cross_file_resolved.clone());
        let builtin_name = input.name.trim();
        let builtin_callee = input.reference_kind == "callee_ref"
            && matches!(
                builtin_name,
                "stdout_write" | "clock_replay_tick" | "files_read_text"
            );
        let app_local_callee = input.reference_kind == "callee_ref"
            && self.scope_is_within_app_boundary(scope_id)
            && !builtin_callee;
        let external =
            (input.external_if_unresolved || is_external_root(input.name)) && !app_local_callee;
        let id = format!(
            "ref_{}_{}_{}",
            self.reference_serial, input.reference_kind, normalized_name
        );
        self.reference_serial += 1;

        let (resolution_status, resolved_definition_id, reason) = if builtin_callee {
            (
                "builtin_reference_v0",
                None,
                Some(match builtin_name {
                    "stdout_write" => "session_z_stdout_builtin_v0",
                    "clock_replay_tick" => "session_aa_runner_replay_builtin_v0",
                    "files_read_text" => "session_ad_exact_file_read_builtin_v0",
                    _ => unreachable!("pinned builtin"),
                }),
            )
        } else if let Some((definition_id, mutable, definition_kind)) = resolved {
            if input.mutable_required && !mutable && definition_kind != "parameter" {
                (
                    "resolved_immutable_place_v0",
                    Some(definition_id),
                    Some("target_is_not_mutable"),
                )
            } else {
                (
                    if cross_file_resolved.is_some() {
                        "cross_file_callable_candidate_v0"
                    } else {
                        "resolved_v0"
                    },
                    Some(definition_id),
                    None,
                )
            }
        } else if external {
            (
                "external_reference_v0",
                None,
                Some("outside_current_source_or_capability_v0"),
            )
        } else {
            ("unresolved_v0", None, Some("name_not_in_visible_scope"))
        };
        let resolved_definition_semantic_identity = resolved_definition_id
            .as_ref()
            .and_then(|definition_id| {
                self.definitions
                    .iter()
                    .find(|definition| &definition.id == definition_id)
            })
            .map(semantic_definition_identity);
        let scope_semantic_identity = self
            .scope_semantic_identities
            .get(scope_id)
            .unwrap_or_else(|| panic!("reference scope `{scope_id}` lacks semantic authority"));
        let semantic_identity = format!(
            "resolver-reference|scope={scope_semantic_identity}|kind={}|binding={normalized_name}|status={resolution_status}",
            input.reference_kind
        );

        self.references.push(ResolveReference {
            id: id.clone(),
            name: input.name.trim().to_string(),
            normalized_name,
            reference_kind: input.reference_kind,
            scope_id: scope_id.to_string(),
            mutable_required: input.mutable_required,
            source_span: portable_span(input.span),
            resolution_status,
            resolved_definition_id: resolved_definition_id.clone(),
            reason,
            canonical_node_id: None,
            canonical_child_position: None,
            canonical_child_path: None,
            scope_semantic_identity: scope_semantic_identity.clone(),
            semantic_identity,
            resolved_definition_semantic_identity,
        });

        match resolution_status {
            "unresolved_v0" => {
                self.add_unresolved_name_diagnostic(input.name, input.span, &id, app_local_callee);
            }
            "resolved_immutable_place_v0" => {
                self.add_immutable_place_diagnostic(
                    input.name,
                    input.span,
                    &id,
                    resolved_definition_id.as_deref(),
                );
            }
            _ => {}
        }

        Some(id)
    }

    fn add_canonical_reference(
        &mut self,
        scope_id: &str,
        input: PendingReferenceInput<'_>,
        expression: &CanonicalExpression,
        position: &CanonicalExpressionPosition,
    ) -> Option<String> {
        let reference_id = self.add_reference(scope_id, input)?;
        let scope_semantic_identity = self
            .scope_semantic_identities
            .get(scope_id)
            .expect("canonical reference scope semantic authority")
            .clone();
        let reference = self
            .references
            .iter_mut()
            .find(|reference| reference.id == reference_id)
            .expect("new canonical resolver reference must be registered");
        reference.canonical_node_id = Some(expression.node_id.as_str().to_string());
        reference.canonical_child_position = Some(position.stable_component());
        reference.canonical_child_path = Some(position.child_path.clone());
        reference.semantic_identity = format!(
            "resolver-reference|scope={}|kind={}|position={}",
            scope_semantic_identity,
            reference.reference_kind,
            position.stable_component()
        );
        Some(reference_id)
    }

    fn resolve_definition(
        &self,
        scope_id: &str,
        normalized_name: &str,
        reference_kind: &str,
    ) -> Option<&ResolveDefinition> {
        let mut cursor = Some(scope_id.to_string());
        while let Some(current_scope) = cursor {
            let key = (current_scope.clone(), normalized_name.to_string());
            if let Some(definition) = self
                .definitions_by_scope_name
                .get(&key)
                .map(|definition| &self.definitions[definition.index])
                .filter(|definition| {
                    !matches!(reference_kind, "callee_ref" | "callable_callee_ref")
                        || matches!(
                            definition.definition_kind,
                            "task" | "test" | "type" | "parameter"
                        )
                })
            {
                return Some(definition);
            }
            if reference_kind == "callee_ref" && self.is_app_scope(&current_scope) {
                return None;
            }
            cursor = self
                .scope_parents
                .get(&current_scope)
                .and_then(|parent| parent.clone());
        }
        None
    }

    fn is_app_scope(&self, scope_id: &str) -> bool {
        self.scopes
            .iter()
            .any(|scope| scope.id == scope_id && scope.scope_kind == "app")
    }

    fn scope_is_within_app_boundary(&self, scope_id: &str) -> bool {
        let mut cursor = Some(scope_id.to_string());
        while let Some(current_scope) = cursor {
            if self.is_app_scope(&current_scope) {
                return true;
            }
            cursor = self
                .scope_parents
                .get(&current_scope)
                .and_then(|parent| parent.clone());
        }
        false
    }

    fn add_unresolved_name_diagnostic(
        &mut self,
        name: &str,
        span: &Span,
        reference_id: &str,
        app_local_callee: bool,
    ) {
        self.diagnostics.push(ResolverDiagnostic {
            cause_key: crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(78),
            code: DiagnosticCode::UNRESOLVED_NAME,
            severity: Severity::Error,
            title: DiagnosticCode::UNRESOLVED_NAME.title(),
            message: format!("name `{}` is not visible in this scope", name.trim()),
            source_span: portable_span(span),
            help: if app_local_callee {
                "Declare or nest the helper task inside the current app; app-local calls cannot resolve file-level helpers."
            } else {
                "Declare the name before use, add a matching item, or list an external dependency under `uses:` when it is intentional."
            },
            reference_id: Some(reference_id.to_string()),
            definition_id: None,
            reason: "unresolved_name_v0",
        });
    }

    fn add_duplicate_definition_diagnostic(
        &mut self,
        name: &str,
        span: &Span,
        definition_id: &str,
        duplicate_of: Option<&str>,
    ) {
        self.diagnostics.push(ResolverDiagnostic {
            cause_key: crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(79),
            code: DiagnosticCode::DUPLICATE_NAME_IN_SCOPE,
            severity: Severity::Error,
            title: DiagnosticCode::DUPLICATE_NAME_IN_SCOPE.title(),
            message: format!("name `{}` is already defined in this scope", name.trim()),
            source_span: portable_span(span),
            help: "Rename one binding or move it into a narrower block so each scope has one definition for the name.",
            reference_id: duplicate_of.map(str::to_string),
            definition_id: Some(definition_id.to_string()),
            reason: "duplicate_name_in_scope_v0",
        });
    }

    fn add_immutable_place_diagnostic(
        &mut self,
        name: &str,
        span: &Span,
        reference_id: &str,
        definition_id: Option<&str>,
    ) {
        self.diagnostics.push(ResolverDiagnostic {
            cause_key: crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(80),
            code: DiagnosticCode::SET_TARGET_IMMUTABLE,
            severity: Severity::Error,
            title: DiagnosticCode::SET_TARGET_IMMUTABLE.title(),
            message: format!("cannot mutate immutable place `{}`", name.trim()),
            source_span: portable_span(span),
            help: "Declare the local with `change`, target a declared `changes:` permission, or keep the value immutable.",
            reference_id: Some(reference_id.to_string()),
            definition_id: definition_id.map(str::to_string),
            reason: "set_target_immutable_v0",
        });
    }

    fn add_read_before_declare_diagnostics(
        &mut self,
        normalized_name: &str,
        definition_span: &Span,
        definition_id: &str,
    ) {
        let mut seen = BTreeSet::new();
        for reference in &self.references {
            if reference.normalized_name != normalized_name
                || reference.resolution_status != "unresolved_v0"
                || reference.source_span.file != portable_path(&definition_span.file)
                || reference.source_span.line >= definition_span.line
                || !seen.insert(reference.id.clone())
            {
                continue;
            }
            self.diagnostics.push(ResolverDiagnostic {
                cause_key: crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(81),
                code: DiagnosticCode::READ_BEFORE_DECLARE,
                severity: Severity::Error,
                title: DiagnosticCode::READ_BEFORE_DECLARE.title(),
                message: format!(
                    "name `{}` is read before its declaration",
                    reference.name.trim()
                ),
                source_span: reference.source_span.clone(),
                help: "Move the declaration above the read or pass the value in through an explicit parameter.",
                reference_id: Some(reference.id.clone()),
                definition_id: Some(definition_id.to_string()),
                reason: "read_before_declare_v0",
            });
        }
    }
}

impl ResolveReport {
    fn status(&self) -> &'static str {
        if self.source_errors > 0 {
            "blocked_by_source_errors"
        } else if self.error_count() > 0 {
            "checked_resolver_with_errors_v0"
        } else {
            "checked_resolver_v0"
        }
    }

    fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .count()
    }

    fn warning_count(&self) -> usize {
        self.diagnostics.len().saturating_sub(self.error_count())
    }

    fn resolved_references(&self) -> usize {
        self.references
            .iter()
            .filter(|reference| reference.resolution_status == "resolved_v0")
            .count()
    }

    fn unresolved_references(&self) -> usize {
        self.references
            .iter()
            .filter(|reference| reference.resolution_status == "unresolved_v0")
            .count()
    }

    fn external_references(&self) -> usize {
        self.references
            .iter()
            .filter(|reference| reference.resolution_status == "external_reference_v0")
            .count()
    }

    fn duplicate_definitions(&self) -> usize {
        self.definitions
            .iter()
            .filter(|definition| definition.status == "duplicate_definition_v0")
            .count()
    }

    fn mutable_place_errors(&self) -> usize {
        self.references
            .iter()
            .filter(|reference| reference.resolution_status == "resolved_immutable_place_v0")
            .count()
    }
}

fn parsed_statement_roots(statement: &ParsedBodyStatement) -> Vec<&CanonicalExpression> {
    if !statement.canonical_extra_occurrences.is_empty() {
        return statement
            .canonical_extra_occurrences
            .iter()
            .map(|expression| &expression.canonical)
            .collect();
    }
    match &statement.kind {
        ParsedBodyStatementKind::Return(expression) => vec![&expression.canonical],
        ParsedBodyStatementKind::Binding { value, .. } => value
            .iter()
            .map(|expression| &expression.canonical)
            .collect(),
        ParsedBodyStatementKind::Other { expressions } => expressions
            .iter()
            .map(|expression| &expression.canonical)
            .collect(),
    }
}

fn canonical_call_nodes(statements: &[ParsedBodyStatement]) -> Vec<CanonicalCallNode<'_>> {
    let mut calls = Vec::new();
    for (statement_index, statement) in statements.iter().enumerate() {
        for (root_ordinal, expression) in parsed_statement_roots(statement).into_iter().enumerate()
        {
            let position = CanonicalExpressionPosition {
                statement_node_id: statement.source_node_id.as_str().to_string(),
                statement_index,
                root_ordinal,
                child_path: vec![root_ordinal],
                node_id: expression.node_id.as_str().to_string(),
            };
            collect_canonical_calls(expression, position, &mut calls);
        }
    }
    calls
}

fn collect_canonical_calls<'a>(
    expression: &'a CanonicalExpression,
    position: CanonicalExpressionPosition,
    calls: &mut Vec<CanonicalCallNode<'a>>,
) {
    match &expression.kind {
        CanonicalExpressionKind::Call { callee, arguments } => {
            if matches!(&callee.kind, CanonicalExpressionKind::Identifier(name) if is_executable_callee(name))
            {
                calls.push(CanonicalCallNode {
                    expression,
                    callee,
                    arguments,
                    position: position.clone(),
                });
            }
            collect_canonical_calls(callee, child_position(&position, callee, 0), calls);
            for (index, argument) in arguments.iter().enumerate() {
                collect_canonical_calls(
                    argument,
                    child_position(&position, argument, index + 1),
                    calls,
                );
            }
        }
        CanonicalExpressionKind::Field { base, .. }
        | CanonicalExpressionKind::ElementPlace { base, .. }
        | CanonicalExpressionKind::Permission { value: base, .. }
        | CanonicalExpressionKind::Try { value: base, .. }
        | CanonicalExpressionKind::Group(base) => {
            collect_canonical_calls(base, child_position(&position, base, 0), calls);
        }
        CanonicalExpressionKind::ListLiteral(values) => {
            for (index, value) in values.iter().enumerate() {
                collect_canonical_calls(value, child_position(&position, value, index), calls);
            }
        }
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            for (index, (_field, value)) in fields.iter().enumerate() {
                collect_canonical_calls(value, child_position(&position, value, index), calls);
            }
        }
        CanonicalExpressionKind::Binary { left, right, .. } => {
            collect_canonical_calls(left, child_position(&position, left, 0), calls);
            collect_canonical_calls(right, child_position(&position, right, 1), calls);
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

fn child_position(
    parent: &CanonicalExpressionPosition,
    child: &CanonicalExpression,
    ordinal: usize,
) -> CanonicalExpressionPosition {
    let mut child_path = parent.child_path.clone();
    child_path.push(ordinal);
    CanonicalExpressionPosition {
        statement_node_id: parent.statement_node_id.clone(),
        statement_index: parent.statement_index,
        root_ordinal: parent.root_ordinal,
        child_path,
        node_id: child.node_id.as_str().to_string(),
    }
}

fn canonical_try_semantic_value<'a>(
    value: &'a CanonicalExpression,
    failure_root: &Option<String>,
    failure_variant: &Option<String>,
    value_position: &CanonicalExpressionPosition,
) -> (&'a CanonicalExpression, CanonicalExpressionPosition) {
    if failure_root.is_none()
        && failure_variant.is_none()
        && let CanonicalExpressionKind::Binary {
            operator: ParsedBinaryOperator::Or,
            left,
            right,
        } = &value.kind
        && matches!(&right.kind, CanonicalExpressionKind::Identifier(name) if name == "fail")
    {
        return (left, child_position(value_position, left, 0));
    }
    (value, value_position.clone())
}

fn canonical_root_position(
    statement: &ParsedBodyStatement,
    root: &CanonicalExpression,
    statement_index: usize,
) -> CanonicalExpressionPosition {
    let root_ordinal = parsed_statement_roots(statement)
        .iter()
        .position(|candidate| candidate.node_id == root.node_id)
        .expect("canonical statement root must be present");
    CanonicalExpressionPosition {
        statement_node_id: statement.source_node_id.as_str().to_string(),
        statement_index,
        root_ordinal,
        child_path: vec![root_ordinal],
        node_id: root.node_id.as_str().to_string(),
    }
}

fn canonical_statement_kind(
    statement: &ParsedBodyStatement,
) -> Option<CanonicalStatementKindEvent> {
    statement
        .canonical_statement_authority
        .iter()
        .find_map(|fact| match (&fact.field, &fact.value) {
            (CanonicalStatementEventField::Kind, CanonicalStatementEventValue::Kind(kind)) => {
                Some(*kind)
            }
            _ => None,
        })
}

fn canonical_statement_token(
    statement: &ParsedBodyStatement,
    field: CanonicalStatementEventField,
) -> Option<(&str, &Span)> {
    statement
        .canonical_statement_authority
        .iter()
        .find_map(|fact| match (&fact.field, &fact.value) {
            (
                candidate,
                CanonicalStatementEventValue::Token {
                    range, spelling, ..
                },
            ) if *candidate == field => Some((spelling.as_str(), &range.start)),
            _ => None,
        })
}

fn canonical_statement_root(
    statement: &ParsedBodyStatement,
    field: CanonicalStatementEventField,
) -> Option<&CanonicalExpression> {
    let node_id = statement
        .canonical_statement_authority
        .iter()
        .find_map(|fact| match (&fact.field, &fact.value) {
            (candidate, CanonicalStatementEventValue::Root { node, .. }) if *candidate == field => {
                Some(node)
            }
            _ => None,
        })?;
    parsed_statement_roots(statement)
        .into_iter()
        .find(|expression| expression.node_id == *node_id)
}

fn canonical_place_root_name(expression: &CanonicalExpression) -> &str {
    match &expression.kind {
        CanonicalExpressionKind::Identifier(name) => name,
        CanonicalExpressionKind::Field { base, .. }
        | CanonicalExpressionKind::ElementPlace { base, .. }
        | CanonicalExpressionKind::Permission { value: base, .. }
        | CanonicalExpressionKind::Try { value: base, .. }
        | CanonicalExpressionKind::Group(base) => canonical_place_root_name(base),
        _ => "",
    }
}

struct CanonicalCallIdentifierUse<'a> {
    name: &'a str,
    consumed: bool,
}

fn collect_canonical_call_identifier_uses<'a>(
    expression: &'a CanonicalExpression,
    consumed: bool,
    uses: &mut Vec<CanonicalCallIdentifierUse<'a>>,
) {
    match &expression.kind {
        CanonicalExpressionKind::Identifier(name) => {
            uses.push(CanonicalCallIdentifierUse { name, consumed });
        }
        CanonicalExpressionKind::Permission { permission, value } => {
            collect_canonical_call_identifier_uses(
                value,
                consumed || *permission == ParamPermission::Consume,
                uses,
            );
        }
        CanonicalExpressionKind::Field { base, .. }
        | CanonicalExpressionKind::ElementPlace { base, .. }
        | CanonicalExpressionKind::Try { value: base, .. }
        | CanonicalExpressionKind::Group(base) => {
            collect_canonical_call_identifier_uses(base, consumed, uses);
        }
        CanonicalExpressionKind::ListLiteral(values) => {
            for value in values {
                collect_canonical_call_identifier_uses(value, consumed, uses);
            }
        }
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            for (_field, value) in fields {
                collect_canonical_call_identifier_uses(value, consumed, uses);
            }
        }
        CanonicalExpressionKind::Binary { left, right, .. } => {
            collect_canonical_call_identifier_uses(left, consumed, uses);
            collect_canonical_call_identifier_uses(right, consumed, uses);
        }
        CanonicalExpressionKind::Call { .. }
        | CanonicalExpressionKind::Unit
        | CanonicalExpressionKind::UIntLiteral(_)
        | CanonicalExpressionKind::IntLiteral(_)
        | CanonicalExpressionKind::BoolLiteral(_)
        | CanonicalExpressionKind::TextLiteral(_)
        | CanonicalExpressionKind::Unsupported => {}
    }
}

fn is_executable_callee(name: &str) -> bool {
    !matches!(
        name,
        "borrow" | "change" | "consume" | "expect" | "fail" | "if" | "return" | "try" | "while"
    )
}

fn render_canonical_expression(expression: &CanonicalExpression) -> String {
    match &expression.kind {
        CanonicalExpressionKind::Unit => "()".to_string(),
        CanonicalExpressionKind::Identifier(name) => name.clone(),
        CanonicalExpressionKind::Field { base, field } => {
            format!("{}.{}", render_canonical_expression(base), field)
        }
        CanonicalExpressionKind::ElementPlace { base, index } => {
            format!("{}[{index}]", render_canonical_expression(base))
        }
        CanonicalExpressionKind::UIntLiteral(value) => value.to_string(),
        CanonicalExpressionKind::IntLiteral(value) => value.to_string(),
        CanonicalExpressionKind::BoolLiteral(value) => value.to_string(),
        CanonicalExpressionKind::TextLiteral(value) => format!("{value:?}"),
        CanonicalExpressionKind::ListLiteral(values) => format!(
            "[{}]",
            values
                .iter()
                .map(render_canonical_expression)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CanonicalExpressionKind::RecordLiteral { name, fields } => format!(
            "{name} {{ {} }}",
            fields
                .iter()
                .map(|(field, value)| format!("{field}: {}", render_canonical_expression(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CanonicalExpressionKind::Call { callee, arguments } => format!(
            "{}({})",
            render_canonical_expression(callee),
            arguments
                .iter()
                .map(render_canonical_expression)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CanonicalExpressionKind::Permission { permission, value } => format!(
            "{} {}",
            match permission {
                ParamPermission::Borrow => "borrow",
                ParamPermission::Change => "change",
                ParamPermission::Consume => "consume",
            },
            render_canonical_expression(value)
        ),
        CanonicalExpressionKind::Try {
            value,
            failure_root,
            failure_variant,
        } => {
            let mut rendered = format!("try {}", render_canonical_expression(value));
            if let Some(root) = failure_root {
                rendered.push_str(" as ");
                rendered.push_str(root);
                if let Some(variant) = failure_variant {
                    rendered.push('.');
                    rendered.push_str(variant);
                }
            }
            rendered
        }
        CanonicalExpressionKind::Binary {
            operator,
            left,
            right,
        } => format!(
            "{} {} {}",
            render_canonical_expression(left),
            render_binary_operator(*operator),
            render_canonical_expression(right)
        ),
        CanonicalExpressionKind::Group(value) => {
            format!("({})", render_canonical_expression(value))
        }
        CanonicalExpressionKind::Unsupported => String::new(),
    }
}

fn render_binary_operator(operator: ParsedBinaryOperator) -> &'static str {
    match operator {
        ParsedBinaryOperator::Multiply => "*",
        ParsedBinaryOperator::Divide => "/",
        ParsedBinaryOperator::Add => "+",
        ParsedBinaryOperator::Subtract => "-",
        ParsedBinaryOperator::Equal => "==",
        ParsedBinaryOperator::NotEqual => "!=",
        ParsedBinaryOperator::Less => "<",
        ParsedBinaryOperator::LessEqual => "<=",
        ParsedBinaryOperator::Greater => ">",
        ParsedBinaryOperator::GreaterEqual => ">=",
        ParsedBinaryOperator::Is => "is",
        ParsedBinaryOperator::Does => "does",
        ParsedBinaryOperator::Returns => "returns",
        ParsedBinaryOperator::FailsWith => "fails with",
        ParsedBinaryOperator::And => "and",
        ParsedBinaryOperator::Or => "or",
    }
}

fn declared_name_from_line(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Some((key, value)) = text.split_once(':')
        && matches!(
            key.trim(),
            "triple" | "record" | "target" | "requires" | "denies"
        )
    {
        return Some(value.trim().to_string());
    }
    if let Some((root, _field)) = text.split_once('.') {
        return Some(root.trim().to_string());
    }
    Some(text.to_string())
}

fn parameter_is_mutable(param: &Param) -> bool {
    matches!(
        param.permission,
        ParamPermission::Change | ParamPermission::Consume
    )
}

fn parameter_state_kind(param: &Param) -> &'static str {
    match param.permission {
        ParamPermission::Borrow => "borrow_parameter",
        ParamPermission::Change => "change_parameter",
        ParamPermission::Consume => "consume_parameter",
    }
}

fn is_external_root(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
}

fn find_section<'a>(sections: &'a [Section], name: &str) -> Option<&'a Section> {
    sections.iter().find(|section| section.name == name)
}

fn count_items(program: &Program) -> usize {
    program
        .files
        .iter()
        .map(|file| count_items_in(&file.items))
        .sum()
}

fn count_items_in(items: &[Item]) -> usize {
    items
        .iter()
        .map(|item| {
            1 + match item {
                Item::App(app) => count_items_in(&app.items),
                _ => 0,
            }
        })
        .sum()
}

fn name_key(name: &str) -> String {
    snake_identifier(name)
}

fn id_fragment(text: &str) -> String {
    let mut fragment = snake_identifier(text);
    if fragment.is_empty() {
        fragment.push_str("root");
    }
    if fragment.len() > 64 {
        fragment.truncate(64);
        fragment = fragment.trim_matches('_').to_string();
    }
    fragment
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
    out.trim_matches('_').to_string()
}

fn portable_span(span: &Span) -> Span {
    Span {
        file: portable_path(&span.file),
        line: span.line,
        column: span.column,
    }
}

fn portable_path(path: &str) -> String {
    path.replace('\\', "/")
}

const NON_CLAIMS: &[&str] = &[
    "no type checking",
    "no borrow checking",
    "no lifetime inference",
    "no effect checking",
    "no module import resolution",
    "no executable semantics",
    "no optimizer authority",
];

fn push_summary(out: &mut String, indent: usize, report: &ResolveReport, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "summary");
    out.push_str(": {\n");
    push_usize_field(out, indent + 2, "files", report.files, true);
    push_usize_field(out, indent + 2, "items", report.items, true);
    push_usize_field(out, indent + 2, "source_errors", report.source_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "source_warnings",
        report.source_warnings,
        true,
    );
    push_usize_field(out, indent + 2, "scopes", report.scopes.len(), true);
    push_usize_field(
        out,
        indent + 2,
        "definitions",
        report.definitions.len(),
        true,
    );
    push_usize_field(out, indent + 2, "references", report.references.len(), true);
    push_usize_field(
        out,
        indent + 2,
        "resolved_references",
        report.resolved_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unresolved_references",
        report.unresolved_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "external_references",
        report.external_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "duplicate_definitions",
        report.duplicate_definitions(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "mutable_place_errors",
        report.mutable_place_errors(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        report.error_count(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_warnings",
        report.warning_count(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_scopes(out: &mut String, indent: usize, scopes: &[ResolveScope], comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "scopes");
    out.push_str(": [");
    if !scopes.is_empty() {
        out.push('\n');
        for (index, scope) in scopes.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            push_scope(out, indent + 2, scope);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_scope(out: &mut String, indent: usize, scope: &ResolveScope) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &scope.id, true);
    push_optional_string_field(
        out,
        indent + 2,
        "parent_scope_id",
        scope.parent_scope_id.as_deref(),
        true,
    );
    push_string_field(out, indent + 2, "scope_kind", scope.scope_kind, true);
    push_string_field(out, indent + 2, "owner_kind", scope.owner_kind, true);
    push_string_field(out, indent + 2, "owner_name", &scope.owner_name, true);
    push_optional_span_field(
        out,
        indent + 2,
        "source_span",
        scope.source_span.as_ref(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
}

fn push_definitions(
    out: &mut String,
    indent: usize,
    definitions: &[ResolveDefinition],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "definitions");
    out.push_str(": [");
    if !definitions.is_empty() {
        out.push('\n');
        for (index, definition) in definitions.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            push_definition(out, indent + 2, definition);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_definition(out: &mut String, indent: usize, definition: &ResolveDefinition) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &definition.id, true);
    push_string_field(out, indent + 2, "name", &definition.name, true);
    push_string_field(
        out,
        indent + 2,
        "normalized_name",
        &definition.normalized_name,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "definition_kind",
        definition.definition_kind,
        true,
    );
    push_string_field(out, indent + 2, "scope_id", &definition.scope_id, true);
    push_bool_field(out, indent + 2, "mutable", definition.mutable, true);
    push_string_field(out, indent + 2, "state_kind", definition.state_kind, true);
    push_span_field(
        out,
        indent + 2,
        "source_span",
        &definition.source_span,
        true,
    );
    push_string_field(out, indent + 2, "status", definition.status, true);
    push_optional_string_field(
        out,
        indent + 2,
        "duplicate_of",
        definition.duplicate_of.as_deref(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
}

fn push_references(out: &mut String, indent: usize, references: &[ResolveReference], comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "references");
    out.push_str(": [");
    if !references.is_empty() {
        out.push('\n');
        for (index, reference) in references.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            push_reference(out, indent + 2, reference);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_reference(out: &mut String, indent: usize, reference: &ResolveReference) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &reference.id, true);
    push_string_field(out, indent + 2, "name", &reference.name, true);
    push_string_field(
        out,
        indent + 2,
        "normalized_name",
        &reference.normalized_name,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "reference_kind",
        reference.reference_kind,
        true,
    );
    push_string_field(out, indent + 2, "scope_id", &reference.scope_id, true);
    push_bool_field(
        out,
        indent + 2,
        "mutable_required",
        reference.mutable_required,
        true,
    );
    push_span_field(out, indent + 2, "source_span", &reference.source_span, true);
    push_string_field(
        out,
        indent + 2,
        "resolution_status",
        reference.resolution_status,
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "resolved_definition_id",
        reference.resolved_definition_id.as_deref(),
        true,
    );
    push_optional_string_field(out, indent + 2, "reason", reference.reason, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_diagnostics(
    out: &mut String,
    indent: usize,
    diagnostics: &[ResolverDiagnostic],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "diagnostics");
    out.push_str(": [");
    if !diagnostics.is_empty() {
        out.push('\n');
        for (index, diagnostic) in diagnostics.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            push_resolver_diagnostic(out, indent + 2, diagnostic);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_resolver_diagnostic(out: &mut String, indent: usize, diagnostic: &ResolverDiagnostic) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "code", diagnostic.code.as_str(), true);
    push_string_field(out, indent + 2, "title", diagnostic.title, true);
    push_string_field(
        out,
        indent + 2,
        "severity",
        diagnostic.severity.as_str(),
        true,
    );
    push_string_field(out, indent + 2, "message", &diagnostic.message, true);
    push_span_field(
        out,
        indent + 2,
        "source_span",
        &diagnostic.source_span,
        true,
    );
    push_string_field(out, indent + 2, "help", diagnostic.help, true);
    push_optional_string_field(
        out,
        indent + 2,
        "reference_id",
        diagnostic.reference_id.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "definition_id",
        diagnostic.definition_id.as_deref(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
}

fn push_string_array(out: &mut String, indent: usize, key: &str, values: &[&str], comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(out, value);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_optional_span_field(
    out: &mut String,
    indent: usize,
    key: &str,
    span: Option<&Span>,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    if let Some(span) = span {
        push_span_object(out, span);
    } else {
        out.push_str("null");
    }
    push_comma_newline(out, comma);
}

fn push_span_field(out: &mut String, indent: usize, key: &str, span: &Span, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_span_object(out, span);
    push_comma_newline(out, comma);
}

fn push_span_object(out: &mut String, span: &Span) {
    out.push('{');
    out.push_str("\"file\": ");
    push_json_string(out, &span.file);
    out.push_str(&format!(
        ", \"line\": {}, \"column\": {}",
        span.line, span.column
    ));
    out.push('}');
}

fn push_optional_string_field(
    out: &mut String,
    indent: usize,
    key: &str,
    value: Option<&str>,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    match value {
        Some(value) => push_json_string(out, value),
        None => out.push_str("null"),
    }
    push_comma_newline(out, comma);
}

fn push_string_field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_json_string(out, value);
    push_comma_newline(out, comma);
}

fn push_bool_field(out: &mut String, indent: usize, key: &str, value: bool, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    out.push_str(if value { "true" } else { "false" });
    push_comma_newline(out, comma);
}

fn push_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    out.push_str(&value.to_string());
    push_comma_newline(out, comma);
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn push_comma_newline(out: &mut String, comma: bool) {
    if comma {
        out.push(',');
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use crate::ast::{Item, Program};
    use crate::parser::parse_source;

    use super::{
        TenB2ResolverBoundaryCorruption, diagnostic_occurrence_set_from_source,
        parser_precedence_relationships, resolve_call_occurrence_summaries, resolve_json,
        resolve_text, with_ten_b2_resolver_boundary_corruption,
    };

    fn ten_b2_resolver_boundary_observation(program: &Program) -> Vec<String> {
        fn semantic_origin(identity: &str) -> &str {
            identity
                .rsplit("|origin=")
                .next()
                .and_then(|rest| rest.split("|shape=").next())
                .expect("resolver identity carries one structural origin")
        }
        resolve_call_occurrence_summaries(program, &[])
            .iter()
            .map(|call| {
                format!(
                    "owner={}|reference={}|target={}|position={}",
                    semantic_origin(&call.owner_definition_id),
                    call.reference_id,
                    semantic_origin(&call.target_definition_id),
                    call.resolver_occurrence_id.position.stable_component(),
                )
            })
            .collect()
    }

    #[test]
    fn ten_b2_resolver_consumer_boundary_corruption_is_load_bearing() {
        const OWNER: &str = "resolver-item:file-0:path-1";
        const TARGET: &str = "resolver-item:file-0:path-0";
        const REFERENCE_PREFIX: &str = concat!(
            "resolver-call-reference|owner=resolver-definition|scope=resolver-scope|",
            "parent=resolver-root|owner=semantic-file:0|kind=file|owner-kind=file|",
            "kind=task|origin=resolver-item:file-0:path-1|shape=task|",
            "params=Borrow:named:UInt|result=named:UInt|effects=0|",
            "body=return|0|binary(Add;binary(Add;call(identifier;identifier);",
            "call(identifier;identifier));call(identifier;call(identifier;identifier)))|",
            "target=resolver-definition|scope=resolver-scope|parent=resolver-root|",
            "owner=semantic-file:0|kind=file|owner-kind=file|kind=task|",
            "origin=resolver-item:file-0:path-0|shape=task|params=Borrow:named:UInt|",
            "result=named:UInt|effects=0|body=return|0|identifier|",
        );
        fn expected_observation(position: &str) -> String {
            format!(
                "owner={OWNER}|reference={REFERENCE_PREFIX}{position}|target={TARGET}|position={position}"
            )
        }
        let source = r#"task leaf(value: UInt) -> UInt {
  does:
    return value
}

task caller(value: UInt) -> UInt {
  does:
    return leaf(value) + leaf(value) + leaf(leaf(value))
}
"#;
        let parsed = parse_source("ten-b2-resolver-boundary.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let expected_clean = [
            "statement-0:parser-body:resolver-item:file-0:path-1:statement-0:root-0:path-0.0.0:node-parser-body:resolver-item:file-0:path-1:statement-0:expression-0:binary-left:binary-left",
            "statement-0:parser-body:resolver-item:file-0:path-1:statement-0:root-0:path-0.0.1:node-parser-body:resolver-item:file-0:path-1:statement-0:expression-0:binary-left:binary-right",
            "statement-0:parser-body:resolver-item:file-0:path-1:statement-0:root-0:path-0.1:node-parser-body:resolver-item:file-0:path-1:statement-0:expression-0:binary-right",
            "statement-0:parser-body:resolver-item:file-0:path-1:statement-0:root-0:path-0.1.1:node-parser-body:resolver-item:file-0:path-1:statement-0:expression-0:binary-right:call-argument-0",
        ]
        .map(expected_observation)
        .to_vec();
        let expected_corrupted = [
            "statement-0:parser-body:resolver-item:file-0:path-1:statement-0:root-0:path-9:node-parser-body:resolver-item:file-0:path-1:statement-0:expression-0:binary-left:binary-left",
            "statement-0:parser-body:resolver-item:file-0:path-1:statement-0:root-0:path-0.0.1:node-parser-body:resolver-item:file-0:path-1:statement-0:expression-0:binary-left:binary-right",
            "statement-0:parser-body:resolver-item:file-0:path-1:statement-0:root-0:path-0.1:node-parser-body:resolver-item:file-0:path-1:statement-0:expression-0:binary-right",
            "statement-0:parser-body:resolver-item:file-0:path-1:statement-0:root-0:path-0.1.1:node-parser-body:resolver-item:file-0:path-1:statement-0:expression-0:binary-right:call-argument-0",
        ]
        .map(expected_observation)
        .to_vec();
        for _ in 0..2 {
            assert_eq!(
                ten_b2_resolver_boundary_observation(&program),
                expected_clean
            );
            let corrupted = with_ten_b2_resolver_boundary_corruption(
                TenB2ResolverBoundaryCorruption {
                    call_node_id: "parser-body:resolver-item:file-0:path-1:statement-0:expression-0:binary-left:binary-left".to_string(),
                    replacement_child_path: vec![9],
                },
                || ten_b2_resolver_boundary_observation(&program),
            );
            assert_eq!(corrupted, expected_corrupted);
        }
    }

    #[test]
    fn resolver_mints_distinct_repeated_sibling_and_nested_call_occurrences() {
        let parsed = parse_source(
            "resolver-call-occurrences.hum",
            r#"task leaf(value: UInt) -> UInt {
  does:
    return value
}

task caller(value: UInt) -> UInt {
  does:
    return leaf(value) + leaf(value) + leaf(leaf(value))
}
"#,
        );
        let program = Program {
            files: vec![parsed.file],
        };
        let calls = resolve_call_occurrence_summaries(&program, &[])
            .into_iter()
            .filter(|call| call.exact_call_span.line == 8)
            .collect::<Vec<_>>();
        assert_eq!(calls.len(), 4);
        assert_eq!(
            calls
                .iter()
                .map(|call| call.relationship_key())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
        );
        assert!(
            calls
                .iter()
                .all(|call| call.owner_definition_id == calls[0].owner_definition_id)
        );
        assert!(
            calls
                .iter()
                .all(|call| call.target_definition_id == calls[0].target_definition_id)
        );
        assert_eq!(
            calls
                .iter()
                .map(|call| call.exact_call_span.column)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
        );
    }

    #[test]
    fn resolver_rejects_retained_section_text_corruption_before_summary() {
        let parsed = parse_source(
            "resolver-call-text-sabotage.hum",
            r#"task leaf(value: UInt) -> UInt {
  does:
    return value
}

task caller(value: UInt) -> UInt {
  does:
    return leaf(value) + leaf(value) + leaf(leaf(consume value))
}
"#,
        );
        for replacement in [
            "",
            "return unrelated_value",
            "return fabricated(value) + source_text_only(value)",
        ] {
            let mut file = parsed.file.clone();
            let Item::Task(caller) = &mut file.items[1] else {
                panic!("caller task")
            };
            caller
                .sections
                .iter_mut()
                .find(|section| section.name == "does")
                .expect("does section")
                .lines[0]
                .text = replacement.to_string();
            let program = Program { files: vec![file] };
            let item = &program.files[0].items[1];
            let Item::Task(caller) = item else {
                panic!("caller task")
            };
            let does = caller.section("does").expect("does section");
            let expectation = program
                .canonical_core_expectation(item, does)
                .expect("corrupt projection remains live and locatable");
            assert!(matches!(
                crate::core_body::try_analyze_does_section(expectation),
                Err("canonical_core_section_projection_mismatch_v0")
            ));
        }
    }

    #[test]
    fn resolver_rejects_earlier_statement_text_corruption_before_later_summary() {
        let parsed = parse_source(
            "resolver-earlier-text-sabotage.hum",
            r#"task first(value: UInt) -> UInt {
  does:
    return value
}

task later(value: UInt) -> UInt {
  does:
    return first(value)
}
"#,
        );
        for replacement in ["return 0", "return value + extra_one + extra_two"] {
            let mut file = parsed.file.clone();
            let Item::Task(first) = &mut file.items[0] else {
                panic!("first task")
            };
            first
                .sections
                .iter_mut()
                .find(|section| section.name == "does")
                .expect("does section")
                .lines[0]
                .text = replacement.to_string();

            let program = Program { files: vec![file] };
            let item = &program.files[0].items[0];
            let Item::Task(first) = item else {
                panic!("first task")
            };
            let does = first.section("does").expect("does section");
            let expectation = program
                .canonical_core_expectation(item, does)
                .expect("corrupt projection remains live and locatable");
            assert!(matches!(
                crate::core_body::try_analyze_does_section(expectation),
                Err("canonical_core_section_projection_mismatch_v0")
            ));
        }
    }

    #[test]
    fn resolver_does_not_mint_a_call_for_the_return_unit_syntax() {
        let parsed = parse_source(
            "resolver-return-unit.hum",
            r#"task caller -> Unit {
  does:
    return ()
}
"#,
        );
        let program = Program {
            files: vec![parsed.file],
        };
        assert!(resolve_call_occurrence_summaries(&program, &[]).is_empty());
    }

    #[test]
    fn parser_precedence_is_consumed_for_a_genuine_shared_item_node() {
        let parsed = parse_source(
            "parser-resolver-precedence.hum",
            r#"task Bad Name(value: UInt) -> UInt {
  does:
    return value
}

task Bad Name(value: UInt) -> UInt {
  does:
    return value
}
"#,
        );
        let source_occurrences = parsed.diagnostic_occurrences.clone();
        let diagnostics = parsed.diagnostics.clone();
        let program = Program {
            files: vec![parsed.file],
        };
        let occurrences =
            diagnostic_occurrence_set_from_source(&program, &diagnostics, &source_occurrences);
        let relationships = parser_precedence_relationships(&occurrences);
        assert_eq!(relationships.len(), 1);
        let application = relationships[0].application();
        assert_eq!(application.rule_id, "parser_over_resolver_v0");
        assert_eq!(application.applying_stage, "resolve");
        assert!(
            application
                .relationship_route
                .iter()
                .any(|entry| entry == "semantic_node=resolver-item:file-0:path-1")
        );
    }

    #[test]
    fn json_resolves_items_permissions_and_local_places() {
        let source = r#"type WorkItem {
  title: Text
}

store work_items: list WorkItem {
  why:
    remember work
}

task remember_work_item(title: Text) -> WorkItem {
  uses:
    clock

  changes:
    work_items

  does:
    let item = WorkItem {
      title: title
    }
    save item in work_items
    return item
}
"#;
        let parsed = parse_source("resolve.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = resolve_json(&program, &[]);

        assert!(json.contains("\"schema\": \"hum.resolve.v0\""));
        assert!(json.contains("\"mode\": \"source_analysis_only_no_type_or_borrow_check\""));
        assert!(json.contains("\"definition_kind\": \"store\""));
        assert!(json.contains("\"definition_kind\": \"declared_change_permission\""));
        assert!(json.contains("\"reference_kind\": \"declared_change\""));
        assert!(json.contains("\"reference_kind\": \"store_write_target\""));
        assert!(json.contains("\"reference_kind\": \"store_write_value\""));
        assert!(json.contains("\"resolution_status\": \"resolved_v0\""));
        assert!(json.contains("\"normalized_name\": \"work_items\""));
        assert!(json.contains("\"normalized_name\": \"item\""));
        assert!(json.contains("\"no type checking\""));
        assert!(json.contains("\"no borrow checking\""));
    }

    #[test]
    fn json_reports_unresolved_duplicates_read_before_declare_and_immutable_set() {
        let source = r#"task bad_names() -> UInt {
  does:
    return later
    let later: UInt = 0
    let frozen: UInt = 1
    let frozen: UInt = 2
    set frozen = 3
    return missing
}
"#;
        let parsed = parse_source("bad.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = resolve_json(&program, &[]);

        assert!(json.contains("\"status\": \"checked_resolver_with_errors_v0\""));
        assert!(json.contains("\"code\": \"H0601\""));
        assert!(json.contains("\"code\": \"H0602\""));
        assert!(json.contains("\"code\": \"H0603\""));
        assert!(json.contains("\"code\": \"H0604\""));
        assert!(json.contains("\"resolution_status\": \"resolved_immutable_place_v0\""));
        assert!(json.contains("\"status\": \"duplicate_definition_v0\""));
    }

    #[test]
    fn text_report_summarizes_without_execution_claims() {
        let source = r#"task ok(value: UInt) -> UInt {
  does:
    return value
}
"#;
        let parsed = parse_source("ok.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let text = resolve_text(&program, &[]);

        assert!(text.contains("Hum resolver report (hum.resolve.v0)"));
        assert!(text.contains("summary: files=1 items=1"));
        assert!(text.contains("diagnostics: none"));
        assert!(text.contains("no executable semantics"));
    }

    #[test]
    fn writable_alias_rebinding_defers_duplicate_blame_to_ownership() {
        let parsed = parse_source(
            "fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum"),
        );
        let program = Program {
            files: vec![parsed.file],
        };
        let json = resolve_json(&program, &[]);

        assert!(json.contains("\"resolver_errors\": 0"));
        assert!(json.contains("\"duplicate_definition_deferred_to_ownership_v0\""));
        assert!(!json.contains("\"code\": \"H0602\""));
    }
}
