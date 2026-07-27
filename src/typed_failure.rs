use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::ast::{
    CanonicalActualLexicalEvidence, CanonicalCompletionEvent, CanonicalExpectedLexicalEvidence,
    CanonicalExpression, CanonicalExpressionKind, CanonicalLexicalTokenKind,
    CanonicalMalformedCause, CanonicalPayloadEventValue, CanonicalPayloadField,
    CanonicalTryWrapperKind, Item, ParsedBodyStatement, ParsedBodyStatementKind, Program, Task,
};
use crate::core_body::BodyStatement;
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOccurrence, Span};
use crate::graph::{hollow_contract_reason, is_meaningful_line_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskFailureSignature {
    pub name: String,
    pub success_type: Option<String>,
    pub error_root: Option<String>,
    pub span: Span,
    semantic_identity: Option<String>,
    resolver_definition_id: Option<String>,
    resolver_target_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FailureCatalog {
    tasks: BTreeMap<String, TaskFailureSignature>,
    tasks_by_resolver_target: BTreeMap<String, TaskFailureSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DirectCall {
    pub callee: String,
    pub source: String,
    pub source_offset: usize,
    canonical_node_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FailureVariant {
    pub root: String,
    pub variant: String,
}

impl FailureVariant {
    pub fn identity(&self) -> String {
        format!("{}.{}", self.root, self.variant)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TryExpression {
    pub call: DirectCall,
    pub wrapper: Option<FailureVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FailureFact {
    pub index: usize,
    pub status: &'static str,
    pub reason: Option<&'static str>,
    pub diagnostic_code: Option<DiagnosticCode>,
    cause: Option<TypedFailureCause>,
    pub form: &'static str,
    pub callee: Option<String>,
    pub callee_result_root: Option<String>,
    pub caller_result_root: Option<String>,
    pub wrapper_root: Option<String>,
    pub call_span: Span,
    pub callee_span: Option<Span>,
    callee_semantic_identity: Option<String>,
    callee_resolver_definition_id: Option<String>,
    callee_resolver_target_id: Option<String>,
    pub caller_span: Span,
    pub help: Option<String>,
    pub success_type: Option<String>,
    pub try_expression: Option<TryExpression>,
    pub occurrence: Option<DiagnosticOccurrence>,
    resolver_call: Option<crate::resolve::ResolveCallOccurrenceSummary>,
    canonical_statement_node_id: Option<String>,
    canonical_expression_node_id: Option<String>,
    canonical_call_node_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypedFailureCause {
    FallibleCallRequiresTry,
    UnwrappedFailureRootsMustMatch,
    FailureWrapperRootMustMatchCaller,
    TryRequiresFallibleCallee,
    DirectFailureRootMustMatchCaller,
    TryRequiresUnannotatedLetBinding,
    UnsupportedTryExpressionShape,
    TryCalleeNotKnown,
    TypedFailureRequiresFailsWhen,
}

impl TypedFailureCause {
    const fn key(self) -> crate::diagnostic_catalog::DiagnosticCauseKey {
        use TypedFailureCause as Cause;
        crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(match self {
            Cause::FallibleCallRequiresTry => 0,
            Cause::UnwrappedFailureRootsMustMatch => 1,
            Cause::FailureWrapperRootMustMatchCaller => 2,
            Cause::TryRequiresFallibleCallee => 3,
            Cause::DirectFailureRootMustMatchCaller => 4,
            Cause::TryRequiresUnannotatedLetBinding => 5,
            Cause::UnsupportedTryExpressionShape => 6,
            Cause::TryCalleeNotKnown => 7,
            Cause::TypedFailureRequiresFailsWhen => 8,
        })
    }

    const fn reason(self) -> &'static str {
        use TypedFailureCause as Cause;
        match self {
            Cause::FallibleCallRequiresTry => "fallible_call_requires_try_v0",
            Cause::UnwrappedFailureRootsMustMatch => "unwrapped_failure_roots_must_match_v0",
            Cause::FailureWrapperRootMustMatchCaller => "failure_wrapper_root_must_match_caller_v0",
            Cause::TryRequiresFallibleCallee => "try_requires_fallible_callee_v0",
            Cause::DirectFailureRootMustMatchCaller => "direct_failure_root_must_match_caller_v0",
            Cause::TryRequiresUnannotatedLetBinding => "try_requires_unannotated_let_binding_v0",
            Cause::UnsupportedTryExpressionShape => "unsupported_try_expression_shape_v0",
            Cause::TryCalleeNotKnown => "try_callee_not_known_v0",
            Cause::TypedFailureRequiresFailsWhen => "typed_failure_requires_fails_when_v0",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TaskFailureAnalysis {
    pub facts: BTreeMap<usize, FailureFact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypedFailureBindingError {
    BodySyntaxLengthMismatch,
    InvalidDirectFailureAuthority,
    MissingResolverCall,
    DuplicateResolverCall,
    ResolverCallOwnerMismatch,
    ResolverCallTargetMismatch,
    ResolverCallNodeMismatch,
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum TenB3TypedFailureBoundaryCorruption {
    WrapperRoot,
    WrapperRelation,
    DirectFailureMalformedCause,
}

#[cfg(test)]
struct TenB3TypedFailureBoundaryCorruptionState {
    task: &'static str,
    corruption: TenB3TypedFailureBoundaryCorruption,
    hits: usize,
}

#[cfg(test)]
thread_local! {
    static TEN_B3_TYPED_FAILURE_BOUNDARY_CORRUPTION:
        RefCell<Option<TenB3TypedFailureBoundaryCorruptionState>> =
            const { RefCell::new(None) };
}

#[cfg(test)]
fn with_ten_b3_typed_failure_boundary_corruption<T>(
    task: &'static str,
    corruption: TenB3TypedFailureBoundaryCorruption,
    run: impl FnOnce() -> T,
) -> T {
    TEN_B3_TYPED_FAILURE_BOUNDARY_CORRUPTION.with(|slot| {
        assert!(
            slot.replace(Some(TenB3TypedFailureBoundaryCorruptionState {
                task,
                corruption,
                hits: 0,
            }))
            .is_none(),
            "10B.3 typed-failure boundary corruption must not nest"
        );
    });
    let result = run();
    TEN_B3_TYPED_FAILURE_BOUNDARY_CORRUPTION.with(|slot| {
        let state = slot
            .replace(None)
            .expect("10B.3 typed-failure boundary corruption must remain installed");
        assert_eq!(
            state.hits, 1,
            "10B.3 typed-failure boundary corruption must alter exactly one production read"
        );
    });
    result
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ProgramFailureAnalysis {
    tasks: Vec<(usize, TaskFailureAnalysis)>,
}

thread_local! {
    static PROGRAM_ANALYSIS_CACHE: RefCell<Option<(usize, Program, Arc<ProgramFailureAnalysis>)>> = const { RefCell::new(None) };
}

pub(crate) fn analyze_program(program: &Program) -> Arc<ProgramFailureAnalysis> {
    PROGRAM_ANALYSIS_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let program_address = std::ptr::from_ref(program) as usize;
        if let Some((cached_address, cached_program, analysis)) = cache.as_ref()
            && *cached_address == program_address
            && cached_program == program
        {
            return Arc::clone(analysis);
        }
        let analysis = Arc::new(
            build_program_analysis(program)
                .expect("typed-failure canonical consumer authority must remain valid"),
        );
        *cache = Some((program_address, program.clone(), Arc::clone(&analysis)));
        analysis
    })
}

fn build_program_analysis(
    program: &Program,
) -> Result<ProgramFailureAnalysis, TypedFailureBindingError> {
    let resolver_calls = crate::resolve::resolve_call_occurrence_summaries(program, &[]);
    build_program_analysis_at_boundary(program, &resolver_calls)
}

fn build_program_analysis_at_boundary(
    program: &Program,
    resolver_calls: &[crate::resolve::ResolveCallOccurrenceSummary],
) -> Result<ProgramFailureAnalysis, TypedFailureBindingError> {
    let global_catalog = FailureCatalog::from_program(program);
    let mut analysis = ProgramFailureAnalysis::default();
    for file in &program.files {
        collect_program_analysis(
            program,
            &file.items,
            &global_catalog,
            resolver_calls,
            &mut analysis.tasks,
        )?;
    }
    Ok(analysis)
}

impl ProgramFailureAnalysis {
    pub(crate) fn task(&self, task: &Task) -> Option<&TaskFailureAnalysis> {
        let address = std::ptr::from_ref(task) as usize;
        self.tasks
            .iter()
            .find_map(|(candidate, analysis)| (*candidate == address).then_some(analysis))
    }

    pub(crate) fn fact(&self, task: &Task, index: usize) -> Option<&FailureFact> {
        self.task(task)?.facts.get(&index)
    }

    pub(crate) fn occurrences(&self) -> Vec<DiagnosticOccurrence> {
        self.tasks
            .iter()
            .map(|(_, analysis)| analysis)
            .flat_map(|analysis| analysis.facts.values())
            .filter_map(|fact| fact.occurrence.clone())
            .collect()
    }
}

fn collect_program_analysis(
    program: &Program,
    items: &[Item],
    catalog: &FailureCatalog,
    resolver_calls: &[crate::resolve::ResolveCallOccurrenceSummary],
    out: &mut Vec<(usize, TaskFailureAnalysis)>,
) -> Result<(), TypedFailureBindingError> {
    for item in items {
        match item {
            Item::Task(task) => {
                let statements = task
                    .section("does")
                    .map(|does| {
                        crate::core_body::analyze_does_section(
                            program
                                .canonical_core_expectation_for_task(task, does)
                                .expect("live failure task must have parser authority"),
                        )
                    })
                    .map(|body| body.statements)
                    .unwrap_or_default();
                if statements.len() != task.body_syntax.len() {
                    return Err(TypedFailureBindingError::BodySyntaxLengthMismatch);
                }
                let owner_definition_id =
                    crate::resolve::semantic_task_definition_identity(program, task);
                let task_resolver_calls = resolver_calls
                    .iter()
                    .filter(|call| call.owner_definition_id == owner_definition_id)
                    .cloned()
                    .collect::<Vec<_>>();
                #[cfg(test)]
                let corrupted_body_syntax =
                    ten_b3_typed_failure_statements_at_consumer_boundary(task);
                #[cfg(test)]
                let body_syntax = corrupted_body_syntax
                    .as_deref()
                    .unwrap_or(task.body_syntax.as_slice());
                #[cfg(not(test))]
                let body_syntax = task.body_syntax.as_slice();
                let mut task_analysis = analyze_task_with_resolver_calls(
                    task,
                    &statements,
                    body_syntax,
                    catalog,
                    &task_resolver_calls,
                )?;
                task_analysis.bind_producer_identity(program, task)?;
                out.push((std::ptr::from_ref(task) as usize, task_analysis));
            }
            Item::App(app) => {
                let scoped_catalog = FailureCatalog::from_items(program, &app.items);
                collect_program_analysis(
                    program,
                    &app.items,
                    &scoped_catalog,
                    resolver_calls,
                    out,
                )?;
            }
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
fn ten_b3_typed_failure_statements_at_consumer_boundary(
    task: &Task,
) -> Option<Vec<ParsedBodyStatement>> {
    TEN_B3_TYPED_FAILURE_BOUNDARY_CORRUPTION.with(|slot| {
        let mut slot = slot.borrow_mut();
        let state = slot.as_mut()?;
        if state.task != task.name {
            return None;
        }
        state.hits += 1;
        let mut statements = task.body_syntax.clone();
        let expression = statements
            .first_mut()
            .and_then(|statement| match &mut statement.kind {
                ParsedBodyStatementKind::Binding {
                    value: Some(value), ..
                } => Some(&mut value.canonical),
                ParsedBodyStatementKind::Other { expressions } => {
                    expressions.first_mut().map(|value| &mut value.canonical)
                }
                ParsedBodyStatementKind::Return(_) | ParsedBodyStatementKind::Binding { .. } => {
                    None
                }
            })
            .expect("10B.3 typed-failure corruption target expression");
        match state.corruption {
            TenB3TypedFailureBoundaryCorruption::WrapperRoot => {
                let CanonicalExpressionKind::Try { failure_root, .. } = &mut expression.kind else {
                    panic!("10B.3 wrapper-root corruption requires Try");
                };
                *failure_root = Some("SourceError".to_string());
                let payload = expression
                    .payload
                    .iter_mut()
                    .find(|payload| payload.field == CanonicalPayloadField::TryFailureRootToken)
                    .expect("10B.3 Try root payload");
                let CanonicalPayloadEventValue::Tokens(tokens) = &mut payload.value else {
                    panic!("10B.3 Try root payload must be tokens");
                };
                let [(_, spelling)] = tokens.as_mut_slice() else {
                    panic!("10B.3 wrapped Try must retain one root token");
                };
                *spelling = "SourceError".to_string();
            }
            TenB3TypedFailureBoundaryCorruption::WrapperRelation => {
                let payload = expression
                    .payload
                    .iter_mut()
                    .find(|payload| payload.field == CanonicalPayloadField::TryWrapperRelation)
                    .expect("10B.3 Try relation payload");
                let CanonicalPayloadEventValue::Tokens(tokens) = &mut payload.value else {
                    panic!("10B.3 Try relation payload must be tokens");
                };
                tokens[0].1 = "xor".to_string();
            }
            TenB3TypedFailureBoundaryCorruption::DirectFailureMalformedCause => {
                let CanonicalExpressionKind::Field { base, .. } = &mut expression.kind else {
                    panic!("10B.3 direct-failure corruption requires Field");
                };
                let CanonicalCompletionEvent::Unsupported(malformed) = &mut base.completion else {
                    panic!("10B.3 direct-failure root must retain malformed evidence");
                };
                malformed.cause = CanonicalMalformedCause::MissingOperand;
            }
        }
        Some(statements)
    })
}

impl TaskFailureAnalysis {
    fn bind_producer_identity(
        &mut self,
        program: &Program,
        task: &Task,
    ) -> Result<(), TypedFailureBindingError> {
        let task_identity = crate::resolve::semantic_task_identity(program, task);
        let owner_definition_id = crate::resolve::semantic_task_definition_identity(program, task);
        for (statement_index, fact) in &mut self.facts {
            let resolver_call = fact.resolver_call.as_ref();
            if let Some(resolver_call) = resolver_call {
                if resolver_call.owner_definition_id != owner_definition_id {
                    return Err(TypedFailureBindingError::ResolverCallOwnerMismatch);
                }
                if fact
                    .callee_resolver_target_id
                    .as_deref()
                    .is_some_and(|target| resolver_call.target_definition_id != target)
                {
                    return Err(TypedFailureBindingError::ResolverCallTargetMismatch);
                }
                if fact
                    .canonical_call_node_id
                    .as_deref()
                    .is_some_and(|node| resolver_call.canonical_call_node_id() != node)
                {
                    return Err(TypedFailureBindingError::ResolverCallNodeMismatch);
                }
            }
            let call_identity_required = fact.callee.is_some()
                && fact.canonical_call_node_id.is_some()
                && !matches!(
                    fact.cause,
                    Some(
                        TypedFailureCause::TryRequiresUnannotatedLetBinding
                            | TypedFailureCause::UnsupportedTryExpressionShape
                    )
                );
            if call_identity_required && resolver_call.is_none() {
                return Err(TypedFailureBindingError::MissingResolverCall);
            }
            let (Some(code), Some(typed_cause)) = (fact.diagnostic_code, fact.cause) else {
                continue;
            };
            let cause_key = typed_cause.key();
            let cause = crate::diagnostic_catalog::diagnostic_cause_for_key(cause_key)
                .expect("typed-failure producer cause must remain registered");
            let mut identity = DiagnosticOccurrence::typed_failure_identity(
                &task_identity,
                *statement_index,
                fact.form,
                fact.callee_semantic_identity.as_deref(),
                fact.callee_resolver_definition_id.as_deref(),
            );
            if let Some(resolver_call) = resolver_call {
                identity.extend_relationship_route(resolver_call.relationship_route());
            }
            if let Some(statement) = &fact.canonical_statement_node_id {
                identity.extend_relationship_route(vec![format!(
                    "typed_failure_statement_node={statement}"
                )]);
            }
            if let Some(expression) = &fact.canonical_expression_node_id {
                identity.extend_relationship_route(vec![format!(
                    "typed_failure_expression_node={expression}"
                )]);
            }
            if let Some(call) = &fact.canonical_call_node_id {
                identity.extend_relationship_route(vec![format!("typed_failure_call_node={call}")]);
            }
            let mut diagnostic =
                Diagnostic::error(code, diagnostic_message(fact), Some(fact.call_span.clone()));
            if let Some(help) = &fact.help {
                diagnostic = diagnostic.with_help(help.clone());
            }
            let mut occurrence = DiagnosticOccurrence::registered(cause, identity, diagnostic)
                .expect("typed-failure producer must seal structural identity");
            if let Some(resolver_call) = resolver_call {
                occurrence = occurrence
                    .with_resolver_call(resolver_call)
                    .expect("typed-failure occurrence must carry its exact resolver call");
            }
            fact.occurrence = Some(occurrence);
        }
        Ok(())
    }
}

impl FailureCatalog {
    pub fn from_program(program: &Program) -> Self {
        let mut catalog = Self::default();
        insert_session_z_builtins(&mut catalog.tasks);
        for file in &program.files {
            collect_signatures(program, &file.items, &mut catalog.tasks);
        }
        catalog.index_resolver_targets();
        catalog
    }

    pub fn from_items(program: &Program, items: &[Item]) -> Self {
        let mut catalog = Self::default();
        insert_session_z_builtins(&mut catalog.tasks);
        collect_signatures(program, items, &mut catalog.tasks);
        catalog.index_resolver_targets();
        catalog
    }

    pub fn task(&self, name: &str) -> Option<&TaskFailureSignature> {
        self.tasks.get(name)
    }

    fn task_for_resolver_target(
        &self,
        target_definition_id: &str,
    ) -> Option<&TaskFailureSignature> {
        self.tasks_by_resolver_target.get(target_definition_id)
    }

    fn index_resolver_targets(&mut self) {
        self.tasks_by_resolver_target = self
            .tasks
            .values()
            .filter_map(|signature| {
                signature
                    .resolver_target_id
                    .as_ref()
                    .map(|identity| (identity.clone(), signature.clone()))
            })
            .collect();
    }
}

fn insert_session_z_builtins(out: &mut BTreeMap<String, TaskFailureSignature>) {
    out.insert(
        "stdout_write".to_string(),
        TaskFailureSignature {
            name: "stdout_write".to_string(),
            success_type: Some("Unit".to_string()),
            error_root: Some("OutputError".to_string()),
            span: Span {
                file: "<builtin:stdout_write>".to_string(),
                line: 1,
                column: 1,
            },
            semantic_identity: Some("builtin-task:stdout_write".to_string()),
            resolver_definition_id: None,
            resolver_target_id: Some("builtin_stdout_write".to_string()),
        },
    );
    out.insert(
        "clock_replay_tick".to_string(),
        TaskFailureSignature {
            name: "clock_replay_tick".to_string(),
            success_type: Some("UInt".to_string()),
            error_root: Some("ReplayClockError".to_string()),
            span: Span {
                file: "<builtin:clock_replay_tick>".to_string(),
                line: 1,
                column: 1,
            },
            semantic_identity: Some("builtin-task:clock_replay_tick".to_string()),
            resolver_definition_id: None,
            resolver_target_id: Some("builtin_clock_replay_tick".to_string()),
        },
    );
    out.insert(
        "files_read_text".to_string(),
        TaskFailureSignature {
            name: "files_read_text".to_string(),
            success_type: Some("Text".to_string()),
            error_root: Some("FileReadError".to_string()),
            span: Span {
                file: "<builtin:files_read_text>".to_string(),
                line: 1,
                column: 1,
            },
            semantic_identity: Some("builtin-task:files_read_text".to_string()),
            resolver_definition_id: None,
            resolver_target_id: Some("builtin_files_read_text".to_string()),
        },
    );
}

#[cfg(test)]
pub(crate) fn analyze_task(
    task: &Task,
    statements: &[BodyStatement],
    catalog: &FailureCatalog,
) -> TaskFailureAnalysis {
    analyze_task_core(task, statements, &task.body_syntax, catalog, None)
        .expect("test task body syntax must align with validated Core statements")
}

fn analyze_task_with_resolver_calls(
    task: &Task,
    statements: &[BodyStatement],
    parsed_statements: &[ParsedBodyStatement],
    catalog: &FailureCatalog,
    resolver_calls: &[crate::resolve::ResolveCallOccurrenceSummary],
) -> Result<TaskFailureAnalysis, TypedFailureBindingError> {
    analyze_task_core(
        task,
        statements,
        parsed_statements,
        catalog,
        Some(resolver_calls),
    )
}

fn analyze_task_core(
    task: &Task,
    statements: &[BodyStatement],
    parsed_statements: &[ParsedBodyStatement],
    catalog: &FailureCatalog,
    resolver_calls: Option<&[crate::resolve::ResolveCallOccurrenceSummary]>,
) -> Result<TaskFailureAnalysis, TypedFailureBindingError> {
    if statements.len() != parsed_statements.len() {
        return Err(TypedFailureBindingError::BodySyntaxLengthMismatch);
    }
    let caller_root = task.result.as_deref().and_then(result_error_root);
    let has_failure_declaration = task.section("fails when").is_some_and(|section| {
        section
            .lines
            .iter()
            .any(|line| is_meaningful_failure_declaration(&line.text))
    });
    let mut facts = BTreeMap::new();

    for (index, (statement, parsed_statement)) in
        statements.iter().zip(parsed_statements).enumerate()
    {
        let statement_resolver_calls = resolver_calls
            .map(|calls| {
                calls
                    .iter()
                    .filter(|call| call.statement_index() == index)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let expression = canonical_statement_primary_expression(parsed_statement);
        if statement.kind == "fail" {
            if let Some(variant) = expression.and_then(canonical_failure_variant) {
                let mut fact = direct_fail_fact(
                    task,
                    statement,
                    index,
                    caller_root.as_deref(),
                    &variant,
                    has_failure_declaration,
                );
                bind_canonical_statement_identity(&mut fact, parsed_statement, expression);
                facts.insert(index, fact);
                continue;
            }
            if expression.is_some_and(canonical_is_sealed_direct_failure_shape) {
                return Err(TypedFailureBindingError::InvalidDirectFailureAuthority);
            }
        }

        let Some(expression) = expression else {
            continue;
        };
        if canonical_contains_try(expression) {
            let expected_call_node = canonical_direct_call_node_id(expression);
            let matching_calls = expected_call_node
                .as_deref()
                .map(|expected| {
                    statement_resolver_calls
                        .iter()
                        .filter(|call| call.canonical_call_node_id() == expected)
                        .copied()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let resolver_call = match matching_calls.as_slice() {
                [call] => Some(*call),
                [] if statement_resolver_calls.len() == 1 => Some(statement_resolver_calls[0]),
                _ => None,
            };
            let mut fact = analyze_try_expression(
                task,
                statement,
                index,
                expression,
                caller_root.as_deref(),
                catalog,
                has_failure_declaration,
                resolver_call,
                resolver_calls.is_some(),
            );
            bind_canonical_statement_identity(&mut fact, parsed_statement, Some(expression));
            if let Some(resolver_call) = resolver_call {
                bind_failure_fact_to_resolver_call(&mut fact, resolver_call)?;
            }
            facts.insert(index, fact);
            continue;
        }

        let resolved_fallible = if resolver_calls.is_some() {
            let mut resolved = None;
            for resolver_call in &statement_resolver_calls {
                let Some(callee) =
                    catalog.task_for_resolver_target(&resolver_call.target_definition_id)
                else {
                    continue;
                };
                let Some(callee_root) = callee.error_root.as_deref() else {
                    continue;
                };
                let canonical_call =
                    canonical_node_by_id(expression, resolver_call.canonical_call_node_id())
                        .ok_or(TypedFailureBindingError::ResolverCallNodeMismatch)?;
                let Some(call) = direct_call_from_resolver(resolver_call, canonical_call) else {
                    continue;
                };
                resolved = Some((call, callee, callee_root, *resolver_call));
                break;
            }
            resolved
        } else {
            None
        };
        let legacy_fallible = resolver_calls
            .is_none()
            .then(|| {
                canonical_direct_calls(expression)
                    .into_iter()
                    .find_map(|call| {
                        let callee = catalog.task(&call.callee)?;
                        let callee_root = callee.error_root.as_deref()?;
                        Some((call, callee, callee_root, None))
                    })
            })
            .flatten();
        if let Some((call, callee, callee_root, resolver_call)) = resolved_fallible
            .map(|(call, callee, root, resolver_call)| (call, callee, root, Some(resolver_call)))
            .or(legacy_fallible)
        {
            let mut fact = issue_fact(
                task,
                statement,
                index,
                "implicit_fallible_call",
                TypedFailureCause::FallibleCallRequiresTry,
                Some(&call),
                Some(callee),
                caller_root.as_deref(),
                None,
                Some(format!(
                    "Fix task `{}`: call `{}` at {} returns `Result ..., {callee_root}` from callee `{}` declared at {}, so the failure cannot be implicit. Write `try {}` when the caller also returns `{callee_root}`, or write `try {} or fail CallerError.context` using this caller's declared error root.",
                    task.name,
                    call.source,
                    location(&statement.span),
                    callee.name,
                    location(&callee.span),
                    call.source,
                    call.source,
                )),
            );
            bind_canonical_statement_identity(&mut fact, parsed_statement, Some(expression));
            if let Some(resolver_call) = resolver_call {
                bind_failure_fact_to_resolver_call(&mut fact, resolver_call)?;
            }
            facts.insert(index, fact);
        }
    }

    Ok(TaskFailureAnalysis { facts })
}

fn bind_failure_fact_to_resolver_call(
    fact: &mut FailureFact,
    resolver_call: &crate::resolve::ResolveCallOccurrenceSummary,
) -> Result<(), TypedFailureBindingError> {
    if fact.resolver_call.is_some() {
        return Err(TypedFailureBindingError::DuplicateResolverCall);
    }
    fact.call_span = resolver_call.exact_call_span.clone();
    if fact.canonical_call_node_id.is_none() {
        fact.canonical_call_node_id = Some(resolver_call.canonical_call_node_id().to_string());
    }
    fact.resolver_call = Some(resolver_call.clone());
    Ok(())
}

fn canonical_statement_primary_expression(
    statement: &ParsedBodyStatement,
) -> Option<&CanonicalExpression> {
    match &statement.kind {
        ParsedBodyStatementKind::Return(expression) => Some(&expression.canonical),
        ParsedBodyStatementKind::Binding { value, .. } => {
            value.as_ref().map(|expression| &expression.canonical)
        }
        ParsedBodyStatementKind::Other { expressions } => expressions
            .first()
            .map(|expression| &expression.canonical)
            .or_else(|| {
                statement
                    .canonical_extra_occurrences
                    .first()
                    .map(|expression| &expression.canonical)
            }),
    }
}

fn bind_canonical_statement_identity(
    fact: &mut FailureFact,
    statement: &ParsedBodyStatement,
    expression: Option<&CanonicalExpression>,
) {
    fact.canonical_statement_node_id = Some(statement.source_node_id.as_str().to_string());
    fact.canonical_expression_node_id =
        expression.map(|expression| expression.node_id.as_str().to_string());
    if fact.canonical_call_node_id.is_none() {
        fact.canonical_call_node_id = fact
            .try_expression
            .as_ref()
            .and_then(|parsed| parsed.call.canonical_node_id.clone());
    }
}

fn canonical_failure_variant(expression: &CanonicalExpression) -> Option<FailureVariant> {
    let CanonicalExpressionKind::Field { base, field } = &expression.kind else {
        return None;
    };
    let root = match &base.kind {
        CanonicalExpressionKind::Identifier(root) => root,
        CanonicalExpressionKind::Unsupported => {
            let CanonicalCompletionEvent::Unsupported(malformed) = &base.completion else {
                return None;
            };
            let CanonicalActualLexicalEvidence::Token {
                kind: CanonicalLexicalTokenKind::Identifier,
                range,
                spelling,
            } = &malformed.actual
            else {
                return None;
            };
            if malformed.cause != CanonicalMalformedCause::InvalidOperandStarter
                || malformed.expected != CanonicalExpectedLexicalEvidence::Operand
                || range != &base.range
            {
                return None;
            }
            spelling
        }
        _ => return None,
    };
    (is_type_ident(root) && is_value_ident(field)).then(|| FailureVariant {
        root: root.clone(),
        variant: field.clone(),
    })
}

fn canonical_is_sealed_direct_failure_shape(expression: &CanonicalExpression) -> bool {
    matches!(
        &expression.kind,
        CanonicalExpressionKind::Field { base, .. }
            if matches!(base.kind, CanonicalExpressionKind::Unsupported)
    )
}

fn canonical_try_expression(expression: &CanonicalExpression) -> Option<TryExpression> {
    let CanonicalExpressionKind::Try {
        value,
        failure_root,
        failure_variant,
    } = &expression.kind
    else {
        return None;
    };
    let keyword = canonical_payload_value(expression, CanonicalPayloadField::TryKeyword)?;
    let CanonicalPayloadEventValue::Token(keyword_range, keyword_spelling) = keyword else {
        return None;
    };
    if keyword_spelling != "try"
        || keyword_range.byte_len != 3
        || keyword_range.start != expression.range.start
    {
        return None;
    }
    if canonical_payload_value(expression, CanonicalPayloadField::TryValueEdge)
        != Some(&CanonicalPayloadEventValue::ChildOrdinal(0))
    {
        return None;
    }
    let relation = canonical_payload_tokens(expression, CanonicalPayloadField::TryWrapperRelation)?;
    let roots = canonical_payload_tokens(expression, CanonicalPayloadField::TryFailureRootToken)?;
    let dots = canonical_payload_tokens(expression, CanonicalPayloadField::TryDotToken)?;
    let variants =
        canonical_payload_tokens(expression, CanonicalPayloadField::TryFailureVariantToken)?;
    let wrapper_kind = canonical_payload_value(expression, CanonicalPayloadField::TryWrapperKind)?;
    let wrapper = match (failure_root, failure_variant) {
        (Some(root), Some(variant))
            if canonical_token_spellings(relation) == ["or", "fail"]
                && canonical_token_spellings(roots) == [root.as_str()]
                && canonical_token_spellings(dots) == ["."]
                && canonical_token_spellings(variants) == [variant.as_str()]
                && wrapper_kind
                    == &CanonicalPayloadEventValue::WrapperKind(CanonicalTryWrapperKind::Wrap)
                && [relation, roots, dots, variants].into_iter().flatten().all(
                    |(range, spelling)| {
                        range.byte_len == spelling.len()
                            && range.start.file == expression.range.start.file
                            && range.start.line == expression.range.start.line
                    },
                ) =>
        {
            Some(FailureVariant {
                root: root.clone(),
                variant: variant.clone(),
            })
        }
        (None, None)
            if relation.is_empty()
                && roots.is_empty()
                && dots.is_empty()
                && variants.is_empty()
                && wrapper_kind
                    == &CanonicalPayloadEventValue::WrapperKind(
                        CanonicalTryWrapperKind::Propagate,
                    ) =>
        {
            None
        }
        _ => return None,
    };
    Some(TryExpression {
        call: canonical_direct_call(value)?,
        wrapper,
    })
}

fn canonical_payload_value(
    expression: &CanonicalExpression,
    field: CanonicalPayloadField,
) -> Option<&CanonicalPayloadEventValue> {
    let mut matching = expression
        .payload
        .iter()
        .filter(|payload| payload.field == field);
    let value = &matching.next()?.value;
    matching.next().is_none().then_some(value)
}

fn canonical_payload_tokens(
    expression: &CanonicalExpression,
    field: CanonicalPayloadField,
) -> Option<&[(crate::ast::ParsedSourceRange, String)]> {
    let CanonicalPayloadEventValue::Tokens(tokens) = canonical_payload_value(expression, field)?
    else {
        return None;
    };
    Some(tokens)
}

fn canonical_token_spellings(tokens: &[(crate::ast::ParsedSourceRange, String)]) -> Vec<&str> {
    tokens
        .iter()
        .map(|(_, spelling)| spelling.as_str())
        .collect()
}

fn canonical_direct_call_node_id(expression: &CanonicalExpression) -> Option<String> {
    let CanonicalExpressionKind::Try { value, .. } = &expression.kind else {
        return canonical_direct_call(expression).and_then(|call| call.canonical_node_id);
    };
    canonical_direct_call(value).and_then(|call| call.canonical_node_id)
}

fn canonical_direct_call(expression: &CanonicalExpression) -> Option<DirectCall> {
    let CanonicalExpressionKind::Call { callee, arguments } = &expression.kind else {
        return None;
    };
    let CanonicalExpressionKind::Identifier(callee) = &callee.kind else {
        return None;
    };
    if !arguments.iter().all(canonical_ordinary_value_argument) {
        return None;
    }
    Some(DirectCall {
        callee: callee.clone(),
        source: String::new(),
        source_offset: 0,
        canonical_node_id: Some(expression.node_id.as_str().to_string()),
    })
}

fn direct_call_from_resolver(
    resolver_call: &crate::resolve::ResolveCallOccurrenceSummary,
    expression: &CanonicalExpression,
) -> Option<DirectCall> {
    let mut call = canonical_direct_call(expression)?;
    call.source = resolver_call.source().to_string();
    Some(call)
}

fn canonical_ordinary_value_argument(expression: &CanonicalExpression) -> bool {
    match &expression.kind {
        CanonicalExpressionKind::Identifier(_)
        | CanonicalExpressionKind::UIntLiteral(_)
        | CanonicalExpressionKind::IntLiteral(_)
        | CanonicalExpressionKind::BoolLiteral(_)
        | CanonicalExpressionKind::TextLiteral(_) => true,
        CanonicalExpressionKind::Field { base, .. }
        | CanonicalExpressionKind::ElementPlace { base, .. }
        | CanonicalExpressionKind::Group(base) => canonical_ordinary_value_argument(base),
        CanonicalExpressionKind::Unit
        | CanonicalExpressionKind::ListLiteral(_)
        | CanonicalExpressionKind::RecordLiteral { .. }
        | CanonicalExpressionKind::Call { .. }
        | CanonicalExpressionKind::Permission { .. }
        | CanonicalExpressionKind::Try { .. }
        | CanonicalExpressionKind::Binary { .. }
        | CanonicalExpressionKind::Unsupported => false,
    }
}

fn canonical_direct_calls(expression: &CanonicalExpression) -> Vec<DirectCall> {
    let mut calls = Vec::new();
    walk_canonical_expression(expression, &mut |node| {
        if let Some(call) = canonical_direct_call(node) {
            calls.push(call);
        }
    });
    calls
}

fn canonical_contains_try(expression: &CanonicalExpression) -> bool {
    let mut found = false;
    walk_canonical_expression(expression, &mut |node| {
        found |= matches!(node.kind, CanonicalExpressionKind::Try { .. });
    });
    found
}

fn canonical_node_by_id<'a>(
    expression: &'a CanonicalExpression,
    expected: &str,
) -> Option<&'a CanonicalExpression> {
    let mut found = None;
    walk_canonical_expression(expression, &mut |node| {
        if node.node_id.as_str() == expected {
            found = Some(node);
        }
    });
    found
}

fn walk_canonical_expression<'a>(
    expression: &'a CanonicalExpression,
    visit: &mut impl FnMut(&'a CanonicalExpression),
) {
    visit(expression);
    match &expression.kind {
        CanonicalExpressionKind::Field { base, .. }
        | CanonicalExpressionKind::ElementPlace { base, .. }
        | CanonicalExpressionKind::Permission { value: base, .. }
        | CanonicalExpressionKind::Try { value: base, .. }
        | CanonicalExpressionKind::Group(base) => walk_canonical_expression(base, visit),
        CanonicalExpressionKind::ListLiteral(values) => {
            for value in values {
                walk_canonical_expression(value, visit);
            }
        }
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            for (_, value) in fields {
                walk_canonical_expression(value, visit);
            }
        }
        CanonicalExpressionKind::Call { callee, arguments } => {
            walk_canonical_expression(callee, visit);
            for argument in arguments {
                walk_canonical_expression(argument, visit);
            }
        }
        CanonicalExpressionKind::Binary { left, right, .. } => {
            walk_canonical_expression(left, visit);
            walk_canonical_expression(right, visit);
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

pub(crate) fn parse_try_expression(text: &str) -> Option<TryExpression> {
    let rest = strip_keyword(text.trim(), "try")?;
    let (call_text, wrapper) = if let Some((call, wrapper)) = rest.split_once(" or fail ") {
        if wrapper.contains(" or fail ") {
            return None;
        }
        (call.trim(), Some(parse_failure_variant(wrapper.trim())?))
    } else {
        (rest.trim(), None)
    };
    let call = parse_direct_call(call_text)?;
    Some(TryExpression { call, wrapper })
}

pub(crate) fn parse_failure_variant(text: &str) -> Option<FailureVariant> {
    let (root, variant) = text.trim().split_once('.')?;
    if !is_type_ident(root) || !is_value_ident(variant) || variant.contains('.') {
        return None;
    }
    Some(FailureVariant {
        root: root.to_string(),
        variant: variant.to_string(),
    })
}

pub(crate) fn result_error_root(result: &str) -> Option<String> {
    result_parts(result).map(|(_success, error)| error)
}

pub(crate) fn result_success_type(result: &str) -> Option<String> {
    result_parts(result).map(|(success, _error)| success)
}

pub(crate) fn is_try_candidate(text: &str) -> bool {
    strip_keyword(text.trim(), "try").is_some()
}

pub(crate) fn is_meaningful_failure_declaration(text: &str) -> bool {
    is_meaningful_line_text(text) && hollow_contract_reason(text).is_none()
}

pub(crate) fn diagnostic_message(fact: &FailureFact) -> String {
    let relationship = match fact.diagnostic_code {
        Some(DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY) => {
            "a known fallible task call must use an explicit `try` form"
        }
        Some(DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION) => {
            "unwrapped propagation requires the caller and callee to declare the same error root"
        }
        Some(DiagnosticCode::FAILURE_WRAPPER_ROOT_MISMATCH) => {
            "a failure wrapper must use the caller's declared error root"
        }
        Some(DiagnosticCode::TRY_ON_INFALLIBLE_CALL) => {
            "`try` cannot be applied to a task without a declared `Result` error root"
        }
        Some(DiagnosticCode::DIRECT_FAILURE_ROOT_MISMATCH) => {
            "a direct `fail` value must use the task's declared error root"
        }
        Some(DiagnosticCode::UNSUPPORTED_TRY_EXPRESSION) => {
            "this `try` expression is outside Session W's two recognized forms"
        }
        Some(DiagnosticCode::MISSING_FAILURE_DECLARATION) => {
            "typed failure requires a meaningful `fails when:` declaration"
        }
        _ => "invalid typed-failure relationship",
    };
    format!(
        "{relationship}; call site {}, caller `{}` declared at {}{}",
        location(&fact.call_span),
        fact.caller_result_root
            .as_deref()
            .unwrap_or("non-Result task"),
        location(&fact.caller_span),
        fact.callee_span
            .as_ref()
            .map(|span| format!(", callee declared at {}", location(span)))
            .unwrap_or_default(),
    )
}

#[allow(clippy::too_many_arguments)]
fn analyze_try_expression(
    task: &Task,
    statement: &BodyStatement,
    index: usize,
    expression: &CanonicalExpression,
    caller_root: Option<&str>,
    catalog: &FailureCatalog,
    has_failure_declaration: bool,
    resolver_call: Option<&crate::resolve::ResolveCallOccurrenceSummary>,
    resolver_owned: bool,
) -> FailureFact {
    if statement.kind != "let_binding" || !is_exact_unannotated_binding(statement) {
        return unsupported_try_fact(
            task,
            statement,
            index,
            TypedFailureCause::TryRequiresUnannotatedLetBinding,
            None,
            caller_root,
        );
    }
    let Some(mut parsed) = canonical_try_expression(expression) else {
        return unsupported_try_fact(
            task,
            statement,
            index,
            TypedFailureCause::UnsupportedTryExpressionShape,
            resolver_call
                .map(|call| DirectCall {
                    callee: call
                        .source()
                        .split('(')
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                    source: call.source().to_string(),
                    source_offset: 0,
                    canonical_node_id: Some(call.canonical_call_node_id().to_string()),
                })
                .as_ref(),
            caller_root,
        );
    };
    if let Some(resolver_call) = resolver_call {
        parsed.call.source = resolver_call.source().to_string();
    }
    let callee = if resolver_owned {
        resolver_call.and_then(|call| catalog.task_for_resolver_target(&call.target_definition_id))
    } else {
        catalog.task(&parsed.call.callee)
    };
    let Some(callee) = callee else {
        return unsupported_try_fact(
            task,
            statement,
            index,
            TypedFailureCause::TryCalleeNotKnown,
            Some(&parsed.call),
            caller_root,
        );
    };
    let Some(callee_root) = callee.error_root.as_deref() else {
        return issue_fact(
            task,
            statement,
            index,
            "try_infallible_call",
            TypedFailureCause::TryRequiresFallibleCallee,
            Some(&parsed.call),
            Some(callee),
            caller_root,
            parsed.wrapper.as_ref(),
            Some(format!(
                "Fix task `{}`: `try` at {} calls infallible task `{}` declared at {}, whose result has no error root. Remove `try` and the wrapper; use `{}` directly.",
                task.name,
                location(&statement.span),
                callee.name,
                location(&callee.span),
                parsed.call.source,
            )),
        );
    };

    if let Some(wrapper) = &parsed.wrapper {
        if caller_root != Some(wrapper.root.as_str()) {
            return issue_fact(
                task,
                statement,
                index,
                "failure_wrap",
                TypedFailureCause::FailureWrapperRootMustMatchCaller,
                Some(&parsed.call),
                Some(callee),
                caller_root,
                Some(wrapper),
                Some(format!(
                    "Fix task `{}`: wrapper `{}` at {} uses root `{}`, but the caller declared error root `{}` at {}; the callee `{}` declared `{callee_root}` at {}. Change the wrapper to this caller's root, or change the caller result deliberately.",
                    task.name,
                    wrapper.identity(),
                    location(&statement.span),
                    wrapper.root,
                    caller_root.unwrap_or("none"),
                    location(&task.span),
                    callee.name,
                    location(&callee.span),
                )),
            );
        }
    } else if caller_root != Some(callee_root) {
        return issue_fact(
            task,
            statement,
            index,
            "failure_propagation",
            TypedFailureCause::UnwrappedFailureRootsMustMatch,
            Some(&parsed.call),
            Some(callee),
            caller_root,
            None,
            Some(format!(
                "Fix task `{}`: unwrapped `try` at {} propagates `{callee_root}` from callee `{}` declared at {}, but the caller declared error root `{}` at {}. Wrap with `or fail CallerError.context` using the caller root, or make the nominal roots equal.",
                task.name,
                location(&statement.span),
                callee.name,
                location(&callee.span),
                caller_root.unwrap_or("none"),
                location(&task.span),
            )),
        );
    }

    if !has_failure_declaration {
        return issue_fact(
            task,
            statement,
            index,
            if parsed.wrapper.is_some() {
                "failure_wrap"
            } else {
                "failure_propagation"
            },
            TypedFailureCause::TypedFailureRequiresFailsWhen,
            Some(&parsed.call),
            Some(callee),
            caller_root,
            parsed.wrapper.as_ref(),
            Some(format!(
                "Fix task `{}`: typed failure at {} can produce `{}`, but the caller header at {} has no meaningful `fails when:` declaration. Add a concrete failure condition naming this path.",
                task.name,
                location(&statement.span),
                parsed
                    .wrapper
                    .as_ref()
                    .map_or(callee_root.to_string(), |wrapper| wrapper.root.clone()),
                location(&task.span),
            )),
        );
    }

    FailureFact {
        index,
        status: if parsed.wrapper.is_some() {
            "accepted_causal_failure_wrap_v0"
        } else {
            "accepted_same_root_failure_propagation_v0"
        },
        reason: None,
        diagnostic_code: None,
        cause: None,
        form: if parsed.wrapper.is_some() {
            "failure_wrap"
        } else {
            "failure_propagation"
        },
        callee: Some(callee.name.clone()),
        callee_result_root: Some(callee_root.to_string()),
        caller_result_root: caller_root.map(str::to_string),
        wrapper_root: parsed.wrapper.as_ref().map(|wrapper| wrapper.root.clone()),
        call_span: call_span_in_statement(statement, &parsed.call)
            .unwrap_or_else(|| portable_span(&statement.span)),
        callee_span: Some(portable_span(&callee.span)),
        callee_semantic_identity: callee.semantic_identity.clone(),
        callee_resolver_definition_id: callee.resolver_definition_id.clone(),
        callee_resolver_target_id: callee.resolver_target_id.clone(),
        caller_span: portable_span(&task.span),
        help: None,
        success_type: callee.success_type.clone(),
        try_expression: Some(parsed),
        occurrence: None,
        resolver_call: None,
        canonical_statement_node_id: None,
        canonical_expression_node_id: Some(expression.node_id.as_str().to_string()),
        canonical_call_node_id: canonical_direct_call_node_id(expression),
    }
}

fn direct_fail_fact(
    task: &Task,
    statement: &BodyStatement,
    index: usize,
    caller_root: Option<&str>,
    variant: &FailureVariant,
    has_failure_declaration: bool,
) -> FailureFact {
    if caller_root != Some(variant.root.as_str()) {
        return issue_fact(
            task,
            statement,
            index,
            "direct_failure",
            TypedFailureCause::DirectFailureRootMustMatchCaller,
            None,
            None,
            caller_root,
            Some(variant),
            Some(format!(
                "Fix task `{}`: direct failure `{}` at {} uses root `{}`, but the caller declared error root `{}` at {}. Fail with a variant under the caller root, or change the caller result deliberately.",
                task.name,
                variant.identity(),
                location(&statement.span),
                variant.root,
                caller_root.unwrap_or("none"),
                location(&task.span),
            )),
        );
    }
    if !has_failure_declaration {
        return issue_fact(
            task,
            statement,
            index,
            "direct_failure",
            TypedFailureCause::TypedFailureRequiresFailsWhen,
            None,
            None,
            caller_root,
            Some(variant),
            Some(format!(
                "Fix task `{}`: direct failure `{}` at {} requires a meaningful `fails when:` declaration on the caller declared at {}.",
                task.name,
                variant.identity(),
                location(&statement.span),
                location(&task.span),
            )),
        );
    }
    FailureFact {
        index,
        status: "accepted_nominal_direct_failure_v0",
        reason: None,
        diagnostic_code: None,
        cause: None,
        form: "direct_failure",
        callee: None,
        callee_result_root: None,
        caller_result_root: caller_root.map(str::to_string),
        wrapper_root: Some(variant.root.clone()),
        call_span: portable_span(&statement.span),
        callee_span: None,
        callee_semantic_identity: None,
        callee_resolver_definition_id: None,
        callee_resolver_target_id: None,
        caller_span: portable_span(&task.span),
        help: None,
        success_type: None,
        try_expression: None,
        occurrence: None,
        resolver_call: None,
        canonical_statement_node_id: None,
        canonical_expression_node_id: None,
        canonical_call_node_id: None,
    }
}

fn unsupported_try_fact(
    task: &Task,
    statement: &BodyStatement,
    index: usize,
    cause: TypedFailureCause,
    call: Option<&DirectCall>,
    caller_root: Option<&str>,
) -> FailureFact {
    issue_fact(
        task,
        statement,
        index,
        "unsupported_try",
        cause,
        call,
        None,
        caller_root,
        None,
        Some(format!(
            "Fix task `{}`: `try` at {} is outside Session W's exact `let value = try named_call(...)` or `let value = try named_call(...) or fail CallerError.context` forms ({}). Use one direct named call with ordinary value arguments; `borrow`, `change`, `consume`, nested calls, operators, and other expression shapes remain unsupported.",
            task.name,
            location(&statement.span),
            cause.reason(),
        )),
    )
}

#[allow(clippy::too_many_arguments)]
fn issue_fact(
    task: &Task,
    statement: &BodyStatement,
    index: usize,
    form: &'static str,
    cause: TypedFailureCause,
    call: Option<&DirectCall>,
    callee: Option<&TaskFailureSignature>,
    caller_root: Option<&str>,
    wrapper: Option<&FailureVariant>,
    help: Option<String>,
) -> FailureFact {
    let cause_spec = crate::diagnostic_catalog::diagnostic_cause_for_key(cause.key())
        .expect("typed-failure cause enum must map to one registered cause");
    FailureFact {
        index,
        status: "rejected_typed_failure_relationship_v0",
        reason: Some(cause.reason()),
        diagnostic_code: Some(cause_spec.code),
        cause: Some(cause),
        form,
        callee: call.map(|call| call.callee.clone()),
        callee_result_root: callee.and_then(|callee| callee.error_root.clone()),
        caller_result_root: caller_root.map(str::to_string),
        wrapper_root: wrapper.map(|wrapper| wrapper.root.clone()),
        call_span: call
            .and_then(|call| call_span_in_statement(statement, call))
            .unwrap_or_else(|| portable_span(&statement.span)),
        callee_span: callee.map(|callee| portable_span(&callee.span)),
        callee_semantic_identity: callee.and_then(|callee| callee.semantic_identity.clone()),
        callee_resolver_definition_id: callee
            .and_then(|callee| callee.resolver_definition_id.clone()),
        callee_resolver_target_id: callee.and_then(|callee| callee.resolver_target_id.clone()),
        caller_span: portable_span(&task.span),
        help,
        success_type: callee.and_then(|callee| callee.success_type.clone()),
        try_expression: None,
        occurrence: None,
        resolver_call: None,
        canonical_statement_node_id: None,
        canonical_expression_node_id: None,
        canonical_call_node_id: call.and_then(|call| call.canonical_node_id.clone()),
    }
}

fn collect_signatures(
    program: &Program,
    items: &[Item],
    out: &mut BTreeMap<String, TaskFailureSignature>,
) {
    for item in items {
        match item {
            Item::Task(task) => {
                let semantic_identity = crate::resolve::semantic_task_identity(program, task);
                out.entry(task.name.clone())
                    .or_insert_with(|| TaskFailureSignature {
                        name: task.name.clone(),
                        success_type: task.result.as_deref().and_then(result_success_type),
                        error_root: task.result.as_deref().and_then(result_error_root),
                        span: portable_span(&task.span),
                        resolver_definition_id: Some(crate::resolve::resolver_task_definition_id(
                            program, task,
                        )),
                        resolver_target_id: Some(
                            crate::resolve::semantic_task_definition_identity(program, task),
                        ),
                        semantic_identity: Some(semantic_identity),
                    });
            }
            Item::App(app) => collect_signatures(program, &app.items, out),
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn result_parts(result: &str) -> Option<(String, String)> {
    let rest = strip_keyword(result.trim(), "Result")?;
    let (success, error) = rest.split_once(',')?;
    let success = success.trim();
    let error = error.trim();
    if success.is_empty() || !is_type_ident(error) {
        return None;
    }
    Some((success.to_string(), error.to_string()))
}

fn parse_direct_call(text: &str) -> Option<DirectCall> {
    let text = text.trim();
    let inside = text.strip_suffix(')')?;
    let (callee, args) = inside.split_once('(')?;
    let callee = callee.trim();
    if !is_value_ident(callee) || args.contains('(') || args.contains(')') {
        return None;
    }
    for argument in split_arguments(args) {
        let argument = argument.trim();
        if argument.is_empty()
            || ["borrow", "change", "consume", "try", "fail"]
                .iter()
                .any(|keyword| strip_keyword(argument, keyword).is_some())
            || !is_ordinary_value_argument(argument)
        {
            return None;
        }
    }
    Some(DirectCall {
        callee: callee.to_string(),
        source: text.to_string(),
        source_offset: 0,
        canonical_node_id: None,
    })
}

pub(crate) fn calls_in_expression(text: &str) -> Vec<DirectCall> {
    let bytes = text.as_bytes();
    let mut calls = Vec::new();
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
            index += 1;
            continue;
        }
        if !is_ident_start_byte(byte)
            || index.checked_sub(1).is_some_and(|previous| {
                is_ident_continue_byte(bytes[previous]) || bytes[previous] == b'.'
            })
        {
            index += 1;
            continue;
        }

        let start = index;
        index += 1;
        while index < bytes.len() && is_ident_continue_byte(bytes[index]) {
            index += 1;
        }
        let callee = &text[start..index];
        let mut open = index;
        while open < bytes.len() && bytes[open].is_ascii_whitespace() {
            open += 1;
        }
        if open >= bytes.len() || bytes[open] != b'(' || !is_value_ident(callee) {
            continue;
        }
        let end = matching_call_end(text, open).unwrap_or(bytes.len().saturating_sub(1));
        calls.push(DirectCall {
            callee: callee.to_string(),
            source: text[start..=end].trim().to_string(),
            source_offset: start,
            canonical_node_id: None,
        });
    }

    calls
}

pub(crate) fn call_span_in_statement(
    statement: &BodyStatement,
    expected: &DirectCall,
) -> Option<Span> {
    let expression = statement_expression(statement)?;
    let calls = calls_in_expression(expression);
    let call = if let Some(at_expected_offset) = calls.iter().find(|call| {
        call.source_offset == expected.source_offset
            && call.callee == expected.callee
            && call.source == expected.source
    }) {
        at_expected_offset.clone()
    } else {
        let exact = calls
            .iter()
            .filter(|call| call.callee == expected.callee && call.source == expected.source)
            .cloned()
            .collect::<Vec<_>>();
        if exact.len() == 1 {
            exact.into_iter().next()?
        } else {
            let by_callee = calls
                .into_iter()
                .filter(|call| call.callee == expected.callee)
                .collect::<Vec<_>>();
            if by_callee.len() != 1 {
                return None;
            }
            by_callee.into_iter().next()?
        }
    };
    let expression_offset = statement.text.find(expression)?;
    let byte_offset = expression_offset.checked_add(call.source_offset)?;
    let prefix = statement.text.get(..byte_offset)?;
    Some(Span::new(
        statement.span.file.clone(),
        statement.span.line,
        statement.span.column + prefix.chars().count(),
    ))
}

#[cfg(test)]
pub(crate) fn call_span_for_identifier_use(
    statement: &BodyStatement,
    identifier: &str,
) -> Option<Span> {
    let expression = statement_expression(statement)?;
    let identifier_offsets = identifier_offsets(expression, identifier);
    if identifier_offsets.is_empty() {
        return None;
    }
    let calls = calls_in_expression(expression);
    let mut owning_calls = Vec::new();
    for identifier_offset in identifier_offsets {
        let mut containing = calls
            .iter()
            .filter(|call| {
                let end = call.source_offset.saturating_add(call.source.len());
                call.source_offset <= identifier_offset && identifier_offset < end
            })
            .collect::<Vec<_>>();
        containing.sort_by_key(|call| call.source.len());
        let Some(innermost) = containing.first() else {
            continue;
        };
        if containing
            .get(1)
            .is_some_and(|next| next.source.len() == innermost.source.len())
        {
            return None;
        }
        if !owning_calls.iter().any(|call: &&DirectCall| {
            call.source_offset == innermost.source_offset && call.source == innermost.source
        }) {
            owning_calls.push(*innermost);
        }
    }
    if owning_calls.len() != 1 {
        return None;
    }
    call_span_in_statement(statement, owning_calls[0])
}

#[cfg(test)]
fn identifier_offsets(text: &str, identifier: &str) -> Vec<usize> {
    if !is_value_ident(identifier) {
        return Vec::new();
    }
    let bytes = text.as_bytes();
    let mut offsets = Vec::new();
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
            index += 1;
            continue;
        }
        if !is_ident_start_byte(byte) {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len() && is_ident_continue_byte(bytes[index]) {
            index += 1;
        }
        if &text[start..index] == identifier {
            offsets.push(start);
        }
    }
    offsets
}

fn matching_call_end(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate().skip(open) {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        if byte == b'"' {
            in_string = true;
            continue;
        }
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
fn contains_keyword_token(text: &str, keyword: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
            index += 1;
            continue;
        }
        if !is_ident_start_byte(byte)
            || index
                .checked_sub(1)
                .is_some_and(|previous| bytes[previous] == b'.')
        {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len() && is_ident_continue_byte(bytes[index]) {
            index += 1;
        }
        if &text[start..index] == keyword {
            return true;
        }
    }
    false
}

fn is_ident_start_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte == b'_'
}

fn is_ident_continue_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
}

fn is_ordinary_value_argument(text: &str) -> bool {
    let text = text.trim();
    if matches!(text, "true" | "false")
        || text.parse::<i64>().is_ok()
        || (text.starts_with('"') && text.ends_with('"') && text.len() >= 2)
    {
        return true;
    }
    text.split('.').all(is_value_ident)
}

pub(crate) fn statement_expression(statement: &BodyStatement) -> Option<&str> {
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

fn is_exact_unannotated_binding(statement: &BodyStatement) -> bool {
    let Some(rest) = strip_keyword(&statement.text, "let") else {
        return false;
    };
    let Some((left, _initializer)) = rest.split_once('=') else {
        return false;
    };
    is_value_ident(left.trim())
}

fn split_arguments(text: &str) -> Vec<&str> {
    if text.trim().is_empty() {
        Vec::new()
    } else {
        text.split(',').collect()
    }
}

fn header_body<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    strip_keyword(text, keyword)?
        .strip_suffix('{')
        .map(str::trim)
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

fn is_type_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_uppercase() && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn location(span: &Span) -> String {
    format!(
        "{}:{}:{}",
        span.file.replace('\\', "/"),
        span.line,
        span.column
    )
}

fn portable_span(span: &Span) -> Span {
    Span::new(span.file.replace('\\', "/"), span.line, span.column)
}

#[cfg(test)]
mod tests {
    use super::{
        FailureCatalog, TenB3TypedFailureBoundaryCorruption, TypedFailureBindingError,
        TypedFailureCause, analyze_program, analyze_task, analyze_task_with_resolver_calls,
        bind_failure_fact_to_resolver_call, build_program_analysis,
        build_program_analysis_at_boundary, call_span_for_identifier_use, call_span_in_statement,
        calls_in_expression, contains_keyword_token, is_meaningful_failure_declaration,
        is_try_candidate, parse_failure_variant, parse_try_expression, result_error_root,
        with_ten_b3_typed_failure_boundary_corruption,
    };
    use crate::ast::{Item, Program, Task};
    use crate::core_body::BodyStatement;
    use crate::diagnostic::DiagnosticCode;
    use crate::diagnostic::Span;
    use crate::parser::parse_source;

    const TEN_B3_TYPED_FAILURE_SOURCE: &str = r#"module tests.ten_b3.typed_failure

type SourceError {
  code: Text
}

type CallerError {
  source: SourceError
}

task source() -> Result UInt, SourceError {
  fails when:
    source unavailable
  does:
    fail SourceError.origin
}

task wrong_root() -> Result UInt, CallerError {
  fails when:
    the wrong nominal root is rejected
  does:
    fail SourceError.origin
}

task propagate() -> Result UInt, SourceError {
  fails when:
    source unavailable
  does:
    let value = try source()
    return value
}

task wrap() -> Result UInt, CallerError {
  fails when:
    wrapped source unavailable
  does:
    let value = try source() or fail CallerError.source
    return value
}

task implicit() -> Result UInt, SourceError {
  fails when:
    source unavailable
  does:
    let value = source()
    return value
}
"#;

    fn ten_b3_program_at_index(index: usize) -> Program {
        let parsed = crate::parser::parse_source_at_index(
            "ten-b3-typed-failure.hum",
            TEN_B3_TYPED_FAILURE_SOURCE,
            index,
        );
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        if index == 0 {
            Program {
                files: vec![parsed.file],
            }
        } else {
            let dummy =
                crate::parser::parse_source_at_index("ten-b3-dummy.hum", "module tests.dummy\n", 0);
            Program {
                files: vec![dummy.file, parsed.file],
            }
        }
    }

    fn ten_b3_task<'a>(program: &'a Program, name: &str) -> &'a Task {
        program
            .files
            .iter()
            .flat_map(|file| &file.items)
            .find_map(|item| match item {
                Item::Task(task) if task.name == name => Some(task),
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing task {name}"))
    }

    fn ten_b3_fact_observation(fact: &super::FailureFact) -> String {
        format!(
            "status={}|reason={}|code={}|form={}|callee={}|caller-root={}|wrapper-root={}|statement={}|expression={}|call={}|resolver-call={}",
            fact.status,
            fact.reason.unwrap_or("none"),
            fact.diagnostic_code
                .map(|code| code.as_str())
                .unwrap_or("none"),
            fact.form,
            fact.callee.as_deref().unwrap_or("none"),
            fact.caller_result_root.as_deref().unwrap_or("none"),
            fact.wrapper_root.as_deref().unwrap_or("none"),
            fact.canonical_statement_node_id
                .as_deref()
                .unwrap_or("none"),
            fact.canonical_expression_node_id
                .as_deref()
                .unwrap_or("none"),
            fact.canonical_call_node_id.as_deref().unwrap_or("none"),
            fact.resolver_call
                .as_ref()
                .map(|call| call.canonical_call_node_id())
                .unwrap_or("none"),
        )
    }

    #[test]
    fn typed_failure_cause_enum_is_closed_and_registry_exact() {
        use TypedFailureCause as Cause;
        let causes = [
            Cause::FallibleCallRequiresTry,
            Cause::UnwrappedFailureRootsMustMatch,
            Cause::FailureWrapperRootMustMatchCaller,
            Cause::TryRequiresFallibleCallee,
            Cause::DirectFailureRootMustMatchCaller,
            Cause::TryRequiresUnannotatedLetBinding,
            Cause::UnsupportedTryExpressionShape,
            Cause::TryCalleeNotKnown,
            Cause::TypedFailureRequiresFailsWhen,
        ];
        let mut keys = std::collections::BTreeSet::new();
        for cause in causes {
            let registered = crate::diagnostic_catalog::diagnostic_cause_for_key(cause.key())
                .expect("closed typed-failure cause");
            assert_eq!(registered.reason, cause.reason());
            assert_eq!(registered.semantic_owner, "typed_failure_analysis");
            assert!(keys.insert(registered.key));
        }
        assert_eq!(keys.len(), 9);
    }

    #[test]
    fn failure_declarations_reject_hollow_contract_text() {
        for placeholder in ["todo", "TODO: explain later", "tbd", "safe", "true"] {
            assert!(!is_meaningful_failure_declaration(placeholder));
        }
        assert!(is_meaningful_failure_declaration(
            "the root source rejects an invalid flag"
        ));
    }

    #[test]
    fn parses_only_the_session_w_try_shapes() {
        let propagate = parse_try_expression("try load(7)").expect("propagation");
        assert_eq!(propagate.call.callee, "load");
        assert!(propagate.wrapper.is_none());

        let wrap =
            parse_try_expression("try load(7) or fail OuterError.context").expect("wrapping");
        assert_eq!(
            wrap.wrapper.expect("wrapper").identity(),
            "OuterError.context"
        );

        assert!(parse_try_expression("try borrow load(7)").is_none());
        assert!(parse_try_expression("try load(inner())").is_none());
        assert!(parse_try_expression("try value + 1").is_none());
        assert!(parse_failure_variant("Error.case.more").is_none());
        assert_eq!(
            result_error_root("Result UInt, WorkError").as_deref(),
            Some("WorkError")
        );
    }

    #[test]
    fn exact_call_spans_and_identifier_ownership_fail_closed() {
        let exact = BodyStatement {
            span: Span::new("same-line.hum", 7, 5),
            text: "return first(\"safe\") + second(value)".to_string(),
            kind: "return",
            status: "accepted_v0",
            expression_kind: Some("compound"),
            reason: None,
        };
        let calls = calls_in_expression("first(\"safe\") + second(value)");
        assert_eq!(calls.len(), 2);
        let first = call_span_in_statement(&exact, &calls[0]).expect("first exact call");
        let second = call_span_in_statement(&exact, &calls[1]).expect("second exact call");
        assert_ne!(first.column, second.column);
        assert_eq!(call_span_for_identifier_use(&exact, "value"), Some(second));

        let ambiguous = BodyStatement {
            text: "return first(value) + second(value)".to_string(),
            ..exact.clone()
        };
        assert_eq!(call_span_for_identifier_use(&ambiguous, "value"), None);

        let duplicate_calls = BodyStatement {
            text: "return first(value) + first(value)".to_string(),
            ..exact.clone()
        };
        let duplicate_scan = calls_in_expression("first(value) + first(value)");
        assert_eq!(duplicate_scan.len(), 2);
        assert_ne!(
            call_span_in_statement(&duplicate_calls, &duplicate_scan[0]),
            call_span_in_statement(&duplicate_calls, &duplicate_scan[1])
        );

        let absent = BodyStatement {
            text: "return value".to_string(),
            ..exact
        };
        assert_eq!(call_span_for_identifier_use(&absent, "value"), None);
    }

    #[test]
    fn scans_nested_calls_and_bounds_the_try_keyword() {
        let calls = calls_in_expression("identity(source()) + source_list()");
        assert_eq!(
            calls
                .iter()
                .map(|call| call.callee.as_str())
                .collect::<Vec<_>>(),
            vec!["identity", "source", "source_list"]
        );
        assert_eq!(calls[1].source, "source()");
        assert!(
            !calls_in_expression("\"source()\"")
                .iter()
                .any(|call| call.callee == "source")
        );

        assert!(is_try_candidate("try source()"));
        assert!(!is_try_candidate("trying()"));
        assert!(!is_try_candidate("try_value()"));
        assert!(contains_keyword_token("value + try source()", "try"));
        assert!(!contains_keyword_token("trying()", "try"));
        assert!(!contains_keyword_token("object.try", "try"));
    }

    #[test]
    fn program_analysis_preserves_distinct_same_cause_occurrences_repeatably() {
        let source = r#"module tests.ao.same_code

type SourceError {
  code: Text
}

task source() -> Result UInt, SourceError {
  fails when:
    the source fails
  does:
    fail SourceError.origin
}

task first() -> Result UInt, SourceError {
  fails when:
    the first call fails
  does:
    let value = source()
    return value
}

task second() -> Result UInt, SourceError {
  fails when:
    the second call fails
  does:
    let value = source()
    return value
}
"#;
        let parsed = parse_source("session_ao_same_code.hum", source);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let first = analyze_program(&program);
        let second = analyze_program(&program);
        assert!(std::sync::Arc::ptr_eq(&first, &second));
        let occurrences = first
            .occurrences()
            .into_iter()
            .filter(|occurrence| occurrence.code == DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY)
            .collect::<Vec<_>>();
        assert_eq!(occurrences.len(), 2);
        assert_ne!(occurrences[0].id(), occurrences[1].id());
        assert_eq!(occurrences[0].cause_key(), occurrences[1].cause_key());
        assert_eq!(occurrences[0].semantic_owner(), "typed_failure_analysis");
        assert_eq!(occurrences[0].owning_stage(), "full_type_check");
        assert_ne!(
            occurrences[0].semantic_origin(),
            occurrences[1].semantic_origin()
        );
        for occurrence in occurrences {
            assert!(!occurrence.semantic_origin().contains(".hum"));
            assert!(
                occurrence
                    .relationship_route()
                    .iter()
                    .all(|entry| !entry.contains(".hum"))
            );
            assert!(occurrence.relationship_route().iter().any(|entry| {
                entry
                    .strip_prefix("semantic_site=definition=")
                    .is_some_and(|definition_id| definition_id.starts_with("def_"))
            }));
            assert!(occurrence.relationship_route().iter().any(|entry| {
                entry
                    .strip_prefix("resolver_call_reference=")
                    .is_some_and(|reference_id| {
                        reference_id.starts_with("resolver-call-reference|owner=")
                    })
            }));
        }
    }

    #[test]
    fn statement_analysis_cannot_mint_occurrence_before_resolver_binding() {
        let parsed = parse_source(
            "typed-failure-display.hum",
            r#"task source() -> Result UInt, SourceError {
  fails when:
    source unavailable
  does:
    return 1
}

task caller() -> UInt {
  does:
    return source()
}
"#,
        );
        let program = Program {
            files: vec![parsed.file],
        };
        let catalog = FailureCatalog::from_program(&program);
        let caller = program.files[0]
            .items
            .iter()
            .find_map(|item| match item {
                Item::Task(task) if task.name == "caller" => Some(task),
                _ => None,
            })
            .expect("caller task");
        let does = caller.section("does").expect("caller does");
        let body = crate::core_body::analyze_does_section(
            program
                .canonical_core_expectation_for_task(caller, does)
                .expect("caller expectation"),
        );
        let local = analyze_task(caller, &body.statements, &catalog);
        assert!(local.facts.values().all(|fact| fact.occurrence.is_none()));
        assert_eq!(
            analyze_program(&program)
                .occurrences()
                .into_iter()
                .filter(|occurrence| {
                    occurrence.code == DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY
                })
                .count(),
            1
        );
    }

    #[test]
    fn resolver_call_binding_rejects_missing_duplicate_ambiguous_and_wrong_identity() {
        let parsed = parse_source(
            "typed-failure-resolver-binding.hum",
            r#"type SourceError {
  code: Text
}

task source() -> Result UInt, SourceError {
  fails when:
    source unavailable
  does:
    fail SourceError.origin
}

task caller() -> Result UInt, SourceError {
  does:
    let value = try source()
    return value
}
"#,
        );
        let program = Program {
            files: vec![parsed.file],
        };
        let caller = program.files[0]
            .items
            .iter()
            .find_map(|item| match item {
                Item::Task(task) if task.name == "caller" => Some(task),
                _ => None,
            })
            .expect("caller task");
        let does = caller.section("does").expect("caller does");
        let statements = crate::core_body::analyze_does_section(
            program
                .canonical_core_expectation_for_task(caller, does)
                .expect("caller expectation"),
        )
        .statements;
        let catalog = FailureCatalog::from_program(&program);
        let owner = crate::resolve::semantic_task_definition_identity(&program, caller);
        let resolver_calls = crate::resolve::resolve_call_occurrence_summaries(&program, &[])
            .into_iter()
            .filter(|call| call.owner_definition_id == owner)
            .collect::<Vec<_>>();
        assert_eq!(resolver_calls.len(), 1);

        let mut baseline = analyze_task_with_resolver_calls(
            caller,
            &statements,
            &caller.body_syntax,
            &catalog,
            &resolver_calls,
        )
        .expect("baseline consumer binding");
        baseline
            .bind_producer_identity(&program, caller)
            .expect("exact resolver call binding");

        let mut missing = analyze_task_with_resolver_calls(
            caller,
            &statements,
            &caller.body_syntax,
            &catalog,
            &[],
        )
        .expect("missing call reaches consumer validation");
        assert_eq!(
            missing.bind_producer_identity(&program, caller),
            Err(TypedFailureBindingError::MissingResolverCall)
        );

        let ambiguous_calls = vec![resolver_calls[0].clone(), resolver_calls[0].clone()];
        let mut ambiguous = analyze_task_with_resolver_calls(
            caller,
            &statements,
            &caller.body_syntax,
            &catalog,
            &ambiguous_calls,
        )
        .expect("ambiguous calls reach consumer validation");
        assert_eq!(
            ambiguous.bind_producer_identity(&program, caller),
            Err(TypedFailureBindingError::MissingResolverCall)
        );

        let mut duplicate = analyze_task_with_resolver_calls(
            caller,
            &statements,
            &caller.body_syntax,
            &catalog,
            &resolver_calls,
        )
        .expect("duplicate binding baseline");
        let fact = duplicate.facts.values_mut().next().expect("typed fact");
        assert_eq!(
            bind_failure_fact_to_resolver_call(fact, &resolver_calls[0]),
            Err(TypedFailureBindingError::DuplicateResolverCall)
        );

        let mut wrong_owner = analyze_task_with_resolver_calls(
            caller,
            &statements,
            &caller.body_syntax,
            &catalog,
            &resolver_calls,
        )
        .expect("wrong owner baseline");
        wrong_owner
            .facts
            .values_mut()
            .next()
            .and_then(|fact| fact.resolver_call.as_mut())
            .expect("resolver call")
            .owner_definition_id
            .push_str(":wrong-owner");
        assert_eq!(
            wrong_owner.bind_producer_identity(&program, caller),
            Err(TypedFailureBindingError::ResolverCallOwnerMismatch)
        );

        let mut wrong_target = analyze_task_with_resolver_calls(
            caller,
            &statements,
            &caller.body_syntax,
            &catalog,
            &resolver_calls,
        )
        .expect("wrong target baseline");
        wrong_target
            .facts
            .values_mut()
            .next()
            .and_then(|fact| fact.resolver_call.as_mut())
            .expect("resolver call")
            .target_definition_id
            .push_str(":wrong-target");
        assert_eq!(
            wrong_target.bind_producer_identity(&program, caller),
            Err(TypedFailureBindingError::ResolverCallTargetMismatch)
        );
    }

    #[test]
    fn missing_failure_declaration_has_one_effect_owned_occurrence() {
        let source = r#"module tests.ao.h0907

type SourceError {
  code: Text
}

task source() -> Result UInt, SourceError {
  fails when:
    the source fails
  does:
    fail SourceError.origin
}

task caller() -> Result UInt, SourceError {
  does:
    let value = try source()
    return value
}
"#;
        let parsed = parse_source("session_ao_h0907.hum", source);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let occurrences = analyze_program(&program)
            .occurrences()
            .into_iter()
            .filter(|occurrence| occurrence.code == DiagnosticCode::MISSING_FAILURE_DECLARATION)
            .collect::<Vec<_>>();
        assert_eq!(occurrences.len(), 1);
        assert_eq!(occurrences[0].semantic_owner(), "typed_failure_analysis");
        assert_eq!(occurrences[0].owning_stage(), "effect_check");
        assert_eq!(
            occurrences[0].diagnostic().code,
            DiagnosticCode::MISSING_FAILURE_DECLARATION
        );
    }

    #[test]
    fn ten_b3_typed_failure_real_path_uses_structured_wrapper_and_resolver_call() {
        let program = ten_b3_program_at_index(0);
        let analysis = build_program_analysis(&program).expect("real production analysis");
        let source = ten_b3_task(&program, "source");
        let source_analysis = analysis.task(source).expect("direct failure consumer");
        let source_fact = source_analysis.facts.get(&0).expect("direct failure fact");

        let wrong_root = ten_b3_task(&program, "wrong_root");
        let wrong_root_analysis = analysis
            .task(wrong_root)
            .expect("wrong-root direct failure consumer");
        let wrong_root_fact = wrong_root_analysis
            .facts
            .get(&0)
            .expect("wrong-root direct failure fact");

        let propagate = ten_b3_task(&program, "propagate");
        let propagate_analysis = analysis.task(propagate).expect("propagation consumer");
        let propagate_fact = propagate_analysis.facts.get(&0).expect("propagation fact");

        let wrap = ten_b3_task(&program, "wrap");
        let wrap_analysis = analysis.task(wrap).expect("wrapper consumer");
        let wrap_fact = wrap_analysis.facts.get(&0).expect("wrapper fact");

        let implicit = ten_b3_task(&program, "implicit");
        let implicit_analysis = analysis.task(implicit).expect("implicit consumer");
        let implicit_fact = implicit_analysis.facts.get(&0).expect("implicit fact");

        let observations = [
            ten_b3_fact_observation(source_fact),
            ten_b3_fact_observation(wrong_root_fact),
            ten_b3_fact_observation(propagate_fact),
            ten_b3_fact_observation(wrap_fact),
            ten_b3_fact_observation(implicit_fact),
        ];
        assert_eq!(
            observations,
            [
                "status=accepted_nominal_direct_failure_v0|reason=none|code=none|form=direct_failure|callee=none|caller-root=SourceError|wrapper-root=SourceError|statement=parser-body:resolver-item:file-0:path-2:statement-0|expression=parser-body:resolver-item:file-0:path-2:statement-0:expression-0|call=none|resolver-call=none".to_string(),
                "status=rejected_typed_failure_relationship_v0|reason=direct_failure_root_must_match_caller_v0|code=H0905|form=direct_failure|callee=none|caller-root=CallerError|wrapper-root=SourceError|statement=parser-body:resolver-item:file-0:path-3:statement-0|expression=parser-body:resolver-item:file-0:path-3:statement-0:expression-0|call=none|resolver-call=none".to_string(),
                "status=accepted_same_root_failure_propagation_v0|reason=none|code=none|form=failure_propagation|callee=source|caller-root=SourceError|wrapper-root=none|statement=parser-body:resolver-item:file-0:path-4:statement-0|expression=parser-body:resolver-item:file-0:path-4:statement-0:expression-0|call=parser-body:resolver-item:file-0:path-4:statement-0:expression-0:try-value|resolver-call=parser-body:resolver-item:file-0:path-4:statement-0:expression-0:try-value".to_string(),
                "status=accepted_causal_failure_wrap_v0|reason=none|code=none|form=failure_wrap|callee=source|caller-root=CallerError|wrapper-root=CallerError|statement=parser-body:resolver-item:file-0:path-5:statement-0|expression=parser-body:resolver-item:file-0:path-5:statement-0:expression-0|call=parser-body:resolver-item:file-0:path-5:statement-0:expression-0:try-value|resolver-call=parser-body:resolver-item:file-0:path-5:statement-0:expression-0:try-value".to_string(),
                "status=rejected_typed_failure_relationship_v0|reason=fallible_call_requires_try_v0|code=H0901|form=implicit_fallible_call|callee=source|caller-root=SourceError|wrapper-root=none|statement=parser-body:resolver-item:file-0:path-6:statement-0|expression=parser-body:resolver-item:file-0:path-6:statement-0:expression-0|call=parser-body:resolver-item:file-0:path-6:statement-0:expression-0|resolver-call=parser-body:resolver-item:file-0:path-6:statement-0:expression-0".to_string(),
            ]
        );
    }

    #[test]
    fn ten_b3_typed_failure_canonical_corruption_and_substitution_fail_closed() {
        let program = ten_b3_program_at_index(0);
        let wrap = ten_b3_task(&program, "wrap");
        let clean = build_program_analysis(&program).expect("clean production consumer");
        assert_eq!(
            ten_b3_fact_observation(
                clean
                    .task(wrap)
                    .expect("clean wrapper consumer")
                    .facts
                    .get(&0)
                    .expect("clean wrapper fact")
            ),
            "status=accepted_causal_failure_wrap_v0|reason=none|code=none|form=failure_wrap|callee=source|caller-root=CallerError|wrapper-root=CallerError|statement=parser-body:resolver-item:file-0:path-5:statement-0|expression=parser-body:resolver-item:file-0:path-5:statement-0:expression-0|call=parser-body:resolver-item:file-0:path-5:statement-0:expression-0:try-value|resolver-call=parser-body:resolver-item:file-0:path-5:statement-0:expression-0:try-value"
        );

        for _ in 0..2 {
            let corrupted = with_ten_b3_typed_failure_boundary_corruption(
                "wrap",
                TenB3TypedFailureBoundaryCorruption::WrapperRoot,
                || build_program_analysis(&program),
            );
            assert_eq!(
                ten_b3_fact_observation(
                    corrupted
                        .expect("wrapper-root corruption reaches production consumer")
                        .task(wrap)
                        .expect("corrupted wrapper consumer")
                        .facts
                        .get(&0)
                        .expect("corrupted wrapper fact")
                ),
                "status=rejected_typed_failure_relationship_v0|reason=failure_wrapper_root_must_match_caller_v0|code=H0903|form=failure_wrap|callee=source|caller-root=CallerError|wrapper-root=SourceError|statement=parser-body:resolver-item:file-0:path-5:statement-0|expression=parser-body:resolver-item:file-0:path-5:statement-0:expression-0|call=parser-body:resolver-item:file-0:path-5:statement-0:expression-0:try-value|resolver-call=parser-body:resolver-item:file-0:path-5:statement-0:expression-0:try-value"
            );

            let relation = with_ten_b3_typed_failure_boundary_corruption(
                "wrap",
                TenB3TypedFailureBoundaryCorruption::WrapperRelation,
                || build_program_analysis(&program),
            )
            .expect("wrapper-marker corruption reaches production consumer");
            assert_eq!(
                ten_b3_fact_observation(
                    relation
                        .task(wrap)
                        .expect("marker-corrupted wrapper consumer")
                        .facts
                        .get(&0)
                        .expect("marker-corrupted wrapper fact")
                ),
                "status=rejected_typed_failure_relationship_v0|reason=unsupported_try_expression_shape_v0|code=H0906|form=unsupported_try|callee=source|caller-root=CallerError|wrapper-root=none|statement=parser-body:resolver-item:file-0:path-5:statement-0|expression=parser-body:resolver-item:file-0:path-5:statement-0:expression-0|call=parser-body:resolver-item:file-0:path-5:statement-0:expression-0:try-value|resolver-call=parser-body:resolver-item:file-0:path-5:statement-0:expression-0:try-value"
            );

            assert_eq!(
                with_ten_b3_typed_failure_boundary_corruption(
                    "wrong_root",
                    TenB3TypedFailureBoundaryCorruption::DirectFailureMalformedCause,
                    || build_program_analysis(&program),
                )
                .unwrap_err(),
                TypedFailureBindingError::InvalidDirectFailureAuthority
            );
        }

        let right = ten_b3_program_at_index(1);
        let right_call = crate::resolve::resolve_call_occurrence_summaries(&right, &[])
            .into_iter()
            .find(|call| call.source() == "source()")
            .expect("same-shaped foreign source call");
        let left_call = crate::resolve::resolve_call_occurrence_summaries(&program, &[])
            .into_iter()
            .find(|call| call.source() == "source()")
            .expect("left source call");
        assert_eq!(left_call.exact_call_span, right_call.exact_call_span);
        assert_eq!(left_call.source(), right_call.source());
        assert_ne!(
            left_call.canonical_call_node_id(),
            right_call.canonical_call_node_id()
        );

        for _ in 0..2 {
            let mut substituted = crate::resolve::resolve_call_occurrence_summaries(&program, &[]);
            substituted
                .iter_mut()
                .find(|call| call.source() == "source()")
                .expect("destination source call")
                .replace_canonical_call_node_for_test(right_call.canonical_call_node_id());
            assert_eq!(
                build_program_analysis_at_boundary(&program, &substituted).unwrap_err(),
                TypedFailureBindingError::ResolverCallNodeMismatch
            );
        }
    }

    #[test]
    fn ten_b3_typed_failure_source_audit_rejects_expression_text_authority() {
        let source = include_str!("typed_failure.rs");
        let start = source
            .find("fn analyze_task_core(")
            .expect("consumer start");
        let end = source[start..]
            .find("\nfn bind_failure_fact_to_resolver_call(")
            .map(|offset| start + offset)
            .expect("consumer end");
        let consumer = &source[start..end];
        for prohibited in [
            "statement_expression(",
            "calls_in_expression(",
            "parse_try_expression(",
            "render_canonical_expression(",
        ] {
            assert!(
                !consumer.contains(prohibited),
                "typed-failure production consumer restored prohibited authority: {prohibited}"
            );
        }
        assert!(consumer.contains("canonical_statement_primary_expression("));
        assert!(consumer.contains("resolver_call.canonical_call_node_id()"));
    }
}
