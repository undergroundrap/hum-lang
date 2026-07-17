use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::sync::Arc;

use crate::app_entry;
use crate::ast::{
    App, CanonicalExpression, CanonicalExpressionKind, Item, ParamPermission, ParsedBinaryOperator,
    ParsedBlockRelationship, ParsedBodyStatement, ParsedBodyStatementKind, Program, Section, Task,
};
use crate::callable::{self, CallableAnalysis};
use crate::capability_root::{self, FilePolicyFact, OutputPolicyFact, ReplayPolicyFact};
use crate::core_body::{self, BodyStatement};
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticOccurrence, DiagnosticOccurrenceCollector,
    DiagnosticOccurrenceSet, Span,
};
use crate::file_read::{
    FileLocalityAdapter, FileLocalityError, FileReadAdapter, HostFileLocalityAdapter,
    HostFileReadAdapter,
};
use crate::graph::is_meaningful_line_text;
use crate::native_path::{ValidatedNativePath, validate_native_path};
use crate::operator_grant::{GrantDecision, OperatorGrantPolicy};
use crate::ownership_check;
use crate::predicate::{self, RecognitionStatus};
use crate::return_dependency;
use crate::type_check;
use crate::typed_failure::{self, FailureVariant};
use crate::writable_field_alias::{self, AliasAnalysis, AliasBinding, AliasIssueKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    Success(String),
    AppSuccess,
    Failure(String),
    AppFailure(String),
    ContractViolation,
    PreflightRejected,
    Trap(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub outcome: RunOutcome,
    pub diagnostics: Vec<Diagnostic>,
    pub authority_events: Vec<AuthorityAuditEvent>,
}

pub(crate) const OUTPUT_LIMIT_BYTES: usize = 1024 * 1024;
const DIAGNOSTIC_PREFLIGHT_REJECTED: &str = "diagnostic-preflight-rejected-v0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityAuditEvent {
    pub event_id: String,
    pub event_sequence: usize,
    pub request_id: String,
    pub source_policy_id: String,
    pub event_kind: &'static str,
    pub authority_surface: &'static str,
    pub capability_id: &'static str,
    pub grant_kind: &'static str,
    pub grant_scope: &'static str,
    pub grant_strength: &'static str,
    pub grant_lifetime: &'static str,
    pub app_name: Option<String>,
    pub task: String,
    pub call_span: Span,
    pub source_route: Vec<String>,
    pub source_route_spans: Vec<Span>,
    pub source_task_authorized: bool,
    pub source_app_authorized: bool,
    pub operator_allow_present: bool,
    pub operator_deny_present: bool,
    pub effective_decision: &'static str,
    pub decision_reason: &'static str,
    pub adapter_called: bool,
    pub byte_count: usize,
    pub replay_index: Option<usize>,
    pub replay_tick: Option<i64>,
    pub native_path_identity: Option<OsString>,
    pub native_path_matched: Option<bool>,
    pub locality_status: Option<&'static str>,
    pub result: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OutputAdapterError;

pub(crate) trait OutputAdapter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), OutputAdapterError>;
}

pub(crate) struct StdoutOutputAdapter;

impl OutputAdapter for StdoutOutputAdapter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), OutputAdapterError> {
        io::stdout()
            .lock()
            .write_all(bytes)
            .map_err(|_| OutputAdapterError)
    }
}

#[cfg(test)]
struct DeniedOutputAdapter;

#[cfg(test)]
impl OutputAdapter for DeniedOutputAdapter {
    fn write(&mut self, _bytes: &[u8]) -> Result<(), OutputAdapterError> {
        Err(OutputAdapterError)
    }
}

struct OutputRuntime<'a> {
    adapter: &'a mut dyn OutputAdapter,
    successful_bytes: usize,
}

pub(crate) trait ReplayAdapter {
    fn next_tick(&mut self) -> Option<i64>;
}

pub(crate) struct RunnerReplayAdapter {
    ticks: VecDeque<i64>,
}

pub(crate) struct RunAdapters<'a> {
    output: &'a mut dyn OutputAdapter,
    replay: &'a mut dyn ReplayAdapter,
    file_locality: &'a mut dyn FileLocalityAdapter,
    file: &'a mut dyn FileReadAdapter,
}

impl RunnerReplayAdapter {
    pub(crate) fn new(ticks: Vec<i64>) -> Self {
        Self {
            ticks: ticks.into(),
        }
    }
}

impl ReplayAdapter for RunnerReplayAdapter {
    fn next_tick(&mut self) -> Option<i64> {
        self.ticks.pop_front()
    }
}

struct ReplayRuntime<'a> {
    adapter: &'a mut dyn ReplayAdapter,
    consumed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Unit,
    Int(i64),
    Bool(bool),
    Text(String),
    Record(BTreeMap<String, Value>),
    List(Vec<Value>),
    Variant(String),
    Path(ValidatedNativePath),
    Callable { target_definition_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Evaluated {
    Value(Value),
    Failure(FailureValue),
    ContractViolation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Flow {
    Continue,
    Return {
        value: Value,
        root: Option<String>,
        span: Span,
    },
    Fail(FailureValue),
    ContractViolation,
}

#[derive(Debug, Clone)]
struct ExecLine {
    text: String,
    location: String,
    span: Span,
    parsed: ParsedBodyStatement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimePermission {
    Borrow,
    Change,
    Consume,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RuntimePlace {
    Root(String),
    Field { root: String, field: String },
    Element { root: String, index: usize },
}

impl RuntimePlace {
    fn from_canonical(expression: &CanonicalExpression) -> Option<Self> {
        let (root, field, index) = expression.direct_place()?;
        Some(match (field, index) {
            (Some(field), None) => Self::Field {
                root: root.to_string(),
                field: field.to_string(),
            },
            (None, Some(index)) => Self::Element {
                root: root.to_string(),
                index,
            },
            (None, None) => Self::Root(root.to_string()),
            (Some(_), Some(_)) => return None,
        })
    }

    fn root(&self) -> &str {
        match self {
            Self::Root(root) | Self::Field { root, .. } | Self::Element { root, .. } => root,
        }
    }

    fn display(&self) -> String {
        match self {
            Self::Root(root) => root.clone(),
            Self::Field { root, field } => format!("{root}.{field}"),
            Self::Element { root, index } => format!("{root}[{index}]"),
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeBinding {
    value: Value,
    definition_id: Option<String>,
    permission: RuntimePermission,
    writable: bool,
    moved_at: Option<Span>,
    moved_by: Option<String>,
    linear: bool,
    view: Option<RuntimeView>,
    writable_alias_source: Option<RuntimePlace>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeViewKind {
    Field,
    Element,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeViewInvalidationKind {
    FieldWrite,
    ListAppend,
}

#[derive(Debug, Clone)]
struct RuntimeViewInvalidation {
    span: Span,
    kind: RuntimeViewInvalidationKind,
}

#[derive(Debug, Clone)]
struct RuntimeView {
    kind: RuntimeViewKind,
    source_place: RuntimePlace,
    bound_at: Span,
    invalidated_by: Option<RuntimeViewInvalidation>,
}

impl RuntimeBinding {
    fn parameter(value: Value, permission: ParamPermission, definition_id: Option<String>) -> Self {
        Self {
            value,
            definition_id,
            permission: RuntimePermission::from(permission),
            writable: permission != ParamPermission::Borrow,
            moved_at: None,
            moved_by: None,
            linear: false,
            view: None,
            writable_alias_source: None,
        }
    }

    fn local(value: Value, linear: bool) -> Self {
        Self {
            value,
            definition_id: None,
            permission: RuntimePermission::Local,
            writable: false,
            moved_at: None,
            moved_by: None,
            linear,
            view: None,
            writable_alias_source: None,
        }
    }

    fn mutable_local(value: Value, linear: bool) -> Self {
        Self {
            value,
            definition_id: None,
            permission: RuntimePermission::Local,
            writable: true,
            moved_at: None,
            moved_by: None,
            linear,
            view: None,
            writable_alias_source: None,
        }
    }

    fn writable_alias(source_place: RuntimePlace) -> Self {
        Self {
            value: Value::Unit,
            definition_id: None,
            permission: RuntimePermission::Local,
            writable: true,
            moved_at: None,
            moved_by: None,
            linear: false,
            view: None,
            writable_alias_source: Some(source_place),
        }
    }

    fn view(
        value: Value,
        kind: RuntimeViewKind,
        source_place: RuntimePlace,
        bound_at: Span,
    ) -> Self {
        Self {
            value,
            definition_id: None,
            permission: RuntimePermission::Local,
            writable: false,
            moved_at: None,
            moved_by: None,
            linear: false,
            view: Some(RuntimeView {
                kind,
                source_place,
                bound_at,
                invalidated_by: None,
            }),
            writable_alias_source: None,
        }
    }
}

impl From<ParamPermission> for RuntimePermission {
    fn from(permission: ParamPermission) -> Self {
        match permission {
            ParamPermission::Borrow => RuntimePermission::Borrow,
            ParamPermission::Change => RuntimePermission::Change,
            ParamPermission::Consume => RuntimePermission::Consume,
        }
    }
}

type Env = BTreeMap<String, RuntimeBinding>;

#[derive(Debug, Clone)]
struct ActiveIteration {
    root: String,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FailureValue {
    root: FailureVariant,
    root_origin: Span,
    steps: Vec<FailureStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FailureStep {
    Propagate {
        span: Span,
        callee: String,
    },
    Wrap {
        outer: FailureVariant,
        span: Span,
        callee: String,
    },
}

impl FailureValue {
    fn root(root: FailureVariant, root_origin: Span) -> Self {
        Self {
            root,
            root_origin,
            steps: Vec::new(),
        }
    }

    fn propagate(mut self, span: Span, callee: String) -> Self {
        self.steps.push(FailureStep::Propagate { span, callee });
        self
    }

    fn wrap(mut self, outer: FailureVariant, span: Span, callee: String) -> Self {
        self.steps.push(FailureStep::Wrap {
            outer,
            span,
            callee,
        });
        self
    }

    fn identity(&self) -> String {
        self.steps
            .iter()
            .rev()
            .find_map(|step| match step {
                FailureStep::Wrap { outer, .. } => Some(outer.identity()),
                FailureStep::Propagate { .. } => None,
            })
            .unwrap_or_else(|| self.root.identity())
    }

    fn render(&self) -> String {
        let identities = self
            .steps
            .iter()
            .rev()
            .filter_map(|step| match step {
                FailureStep::Wrap { outer, .. } => Some(outer.identity()),
                FailureStep::Propagate { .. } => None,
            })
            .chain(std::iter::once(self.root.identity()))
            .collect::<Vec<_>>();
        let mut out = format!("failure: {}", identities[0]);
        let mut wrap_index = 0;
        for step in self.steps.iter().rev() {
            match step {
                FailureStep::Propagate { span, callee } => out.push_str(&format!(
                    "\n  propagated at {} while calling `{callee}`",
                    location(span)
                )),
                FailureStep::Wrap {
                    outer: _,
                    span,
                    callee,
                } => {
                    out.push_str(&format!(
                        "\n  wrapped at {} while calling `{callee}`",
                        location(span)
                    ));
                    wrap_index += 1;
                    out.push_str(&format!("\ncaused by: {}", identities[wrap_index]));
                }
            }
        }
        out.push_str(&format!(
            "\n  originated at {}",
            location(&self.root_origin)
        ));
        out
    }
}

#[cfg(test)]
pub fn run_program(program: &Program, entry: Option<&str>, raw_args: &[String]) -> RunReport {
    let mut adapter = DeniedOutputAdapter;
    let mut replay = RunnerReplayAdapter::new(Vec::new());
    let raw_args = raw_args.iter().map(OsString::from).collect::<Vec<_>>();
    run_program_with_adapters(
        program,
        entry,
        &raw_args,
        &OperatorGrantPolicy::default(),
        &mut adapter,
        &mut replay,
    )
}

#[cfg(test)]
pub(crate) fn run_program_with_output(
    program: &Program,
    entry: Option<&str>,
    raw_args: &[String],
    grant_policy: &OperatorGrantPolicy,
    output_adapter: &mut dyn OutputAdapter,
) -> RunReport {
    let mut replay = RunnerReplayAdapter::new(Vec::new());
    let raw_args = raw_args.iter().map(OsString::from).collect::<Vec<_>>();
    run_program_with_adapters(
        program,
        entry,
        &raw_args,
        grant_policy,
        output_adapter,
        &mut replay,
    )
}

#[cfg(test)]
pub(crate) fn run_program_with_adapters(
    program: &Program,
    entry: Option<&str>,
    raw_args: &[OsString],
    grant_policy: &OperatorGrantPolicy,
    output_adapter: &mut dyn OutputAdapter,
    replay_adapter: &mut dyn ReplayAdapter,
) -> RunReport {
    let occurrences = match runtime_occurrence_authority(program) {
        Ok(occurrences) => occurrences,
        Err(message) => {
            return RunReport {
                outcome: RunOutcome::Trap(message),
                diagnostics: Vec::new(),
                authority_events: Vec::new(),
            };
        }
    };
    run_program_with_occurrences_and_adapters(
        program,
        &occurrences,
        entry,
        raw_args,
        grant_policy,
        output_adapter,
        replay_adapter,
    )
}

pub(crate) fn run_program_with_occurrences_and_adapters(
    program: &Program,
    diagnostic_occurrences: &DiagnosticOccurrenceSet,
    entry: Option<&str>,
    raw_args: &[OsString],
    grant_policy: &OperatorGrantPolicy,
    output_adapter: &mut dyn OutputAdapter,
    replay_adapter: &mut dyn ReplayAdapter,
) -> RunReport {
    let mut file_adapter = HostFileReadAdapter;
    let mut locality_adapter = HostFileLocalityAdapter;
    run_program_with_occurrences_and_file_adapters(
        program,
        diagnostic_occurrences,
        entry,
        raw_args,
        grant_policy,
        RunAdapters {
            output: output_adapter,
            replay: replay_adapter,
            file_locality: &mut locality_adapter,
            file: &mut file_adapter,
        },
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_program_with_occurrences_and_test_adapters(
    program: &Program,
    diagnostic_occurrences: &DiagnosticOccurrenceSet,
    entry: Option<&str>,
    raw_args: &[OsString],
    grant_policy: &OperatorGrantPolicy,
    output_adapter: &mut dyn OutputAdapter,
    replay_adapter: &mut dyn ReplayAdapter,
    file_locality_adapter: &mut dyn FileLocalityAdapter,
    file_adapter: &mut dyn FileReadAdapter,
) -> RunReport {
    run_program_with_occurrences_and_file_adapters(
        program,
        diagnostic_occurrences,
        entry,
        raw_args,
        grant_policy,
        RunAdapters {
            output: output_adapter,
            replay: replay_adapter,
            file_locality: file_locality_adapter,
            file: file_adapter,
        },
    )
}

#[cfg(all(test, windows))]
pub(crate) fn run_program_with_file_adapters(
    program: &Program,
    entry: Option<&str>,
    raw_args: &[OsString],
    grant_policy: &OperatorGrantPolicy,
    adapters: RunAdapters<'_>,
) -> RunReport {
    let occurrences = match runtime_occurrence_authority(program) {
        Ok(occurrences) => occurrences,
        Err(message) => {
            return RunReport {
                outcome: RunOutcome::Trap(message),
                diagnostics: Vec::new(),
                authority_events: Vec::new(),
            };
        }
    };
    run_program_with_occurrences_and_file_adapters(
        program,
        &occurrences,
        entry,
        raw_args,
        grant_policy,
        adapters,
    )
}

fn run_program_with_occurrences_and_file_adapters(
    program: &Program,
    diagnostic_occurrences: &DiagnosticOccurrenceSet,
    entry: Option<&str>,
    raw_args: &[OsString],
    grant_policy: &OperatorGrantPolicy,
    adapters: RunAdapters<'_>,
) -> RunReport {
    let output_policies = output_policy_map(capability_root::output_policy_facts(program));
    let replay_policies = replay_policy_map(capability_root::replay_policy_facts(program));
    let file_policies = file_policy_map(capability_root::file_policy_facts(program));
    let predicate_analysis = predicate::analyze_program(program);
    let callable_analysis = callable::analyze_program(program);
    let diagnostic_occurrence_collector =
        match DiagnosticOccurrenceCollector::from_authority(diagnostic_occurrences) {
            Ok(collector) => collector,
            Err(error) => {
                return RunReport {
                    outcome: RunOutcome::Trap(format!("diagnostic invariant failure: {error:?}")),
                    diagnostics: Vec::new(),
                    authority_events: Vec::new(),
                };
            }
        };
    let interpreter = Interpreter {
        program,
        callable_analysis,
        predicate_analysis,
        diagnostics: RefCell::new(Vec::new()),
        diagnostic_occurrences: RefCell::new(diagnostic_occurrence_collector),
        active_iterations: RefCell::new(Vec::new()),
        active_app: RefCell::new(None),
        grant_policy,
        output: RefCell::new(OutputRuntime {
            adapter: adapters.output,
            successful_bytes: 0,
        }),
        replay: RefCell::new(ReplayRuntime {
            adapter: adapters.replay,
            consumed: 0,
        }),
        file_adapter: RefCell::new(adapters.file),
        file_locality: RefCell::new(adapters.file_locality),
        output_policies,
        replay_policies,
        file_policies,
        output_call_cursors: RefCell::new(BTreeMap::new()),
        replay_call_cursors: RefCell::new(BTreeMap::new()),
        file_call_cursors: RefCell::new(BTreeMap::new()),
        output_task_call_cursors: RefCell::new(BTreeMap::new()),
        active_task_route: RefCell::new(Vec::new()),
        active_task_definition_ids: RefCell::new(Vec::new()),
        active_call_route: RefCell::new(Vec::new()),
        active_callable_applications: RefCell::new(Vec::new()),
        authority_events: RefCell::new(Vec::new()),
    };
    let outcome = match interpreter.run(entry, raw_args) {
        Ok((TaskResult::Returned(value), false)) => match display_value(&value) {
            Ok(display) => RunOutcome::Success(display),
            Err(message) => RunOutcome::Trap(message),
        },
        Ok((TaskResult::Returned(Value::Unit), true)) => RunOutcome::AppSuccess,
        Ok((TaskResult::Returned(_), true)) => RunOutcome::Trap(
            "app start returned a non-Unit value after static checking".to_string(),
        ),
        Ok((TaskResult::Failed(value), false)) => RunOutcome::Failure(value.render()),
        Ok((TaskResult::Failed(value), true)) => RunOutcome::AppFailure(value.render()),
        Ok((TaskResult::ContractViolation, _)) => RunOutcome::ContractViolation,
        Err(message) if message == DIAGNOSTIC_PREFLIGHT_REJECTED => RunOutcome::PreflightRejected,
        Err(message) => RunOutcome::Trap(message),
    };
    let diagnostics = interpreter.diagnostics.into_inner();
    let authority_events = interpreter.authority_events.into_inner();
    RunReport {
        outcome,
        diagnostics,
        authority_events,
    }
}

#[cfg(test)]
fn runtime_occurrence_authority(program: &Program) -> Result<DiagnosticOccurrenceSet, String> {
    let mut source_occurrences = app_entry::diagnostic_occurrence_set(program);
    source_occurrences
        .extend_owned(&crate::path_boundary::diagnostic_occurrence_set(program))
        .map_err(|error| format!("diagnostic invariant failure: {error:?}"))?;
    source_occurrences
        .extend_owned(&capability_root::diagnostic_occurrence_set(program))
        .map_err(|error| format!("diagnostic invariant failure: {error:?}"))?;
    let diagnostics = source_occurrences
        .normalized_occurrences()
        .into_iter()
        .map(|occurrence| occurrence.diagnostic().clone())
        .collect::<Vec<_>>();
    let mut occurrences = crate::profile_check::diagnostic_occurrence_set_from_source(
        program,
        &diagnostics,
        &source_occurrences,
    )
    .map_err(|error| format!("diagnostic invariant failure: {error:?}"))?;
    for occurrence in callable::diagnostic_occurrences(program) {
        occurrences
            .insert_owned(occurrence)
            .map_err(|error| format!("diagnostic invariant failure: {error:?}"))?;
    }
    Ok(occurrences)
}

fn occurrence_public_order(
    left: &DiagnosticOccurrence,
    right: &DiagnosticOccurrence,
) -> std::cmp::Ordering {
    fn display_site(occurrence: &DiagnosticOccurrence) -> (&str, usize, usize) {
        occurrence
            .diagnostic()
            .span
            .as_ref()
            .map_or(("", usize::MAX, usize::MAX), |span| {
                (span.file.as_str(), span.line, span.column)
            })
    }
    display_site(left)
        .cmp(&display_site(right))
        .then_with(|| left.semantic_origin().cmp(right.semantic_origin()))
        .then_with(|| left.id().cmp(right.id()))
}

fn task_reachability_mask(program: &Program, reachable_tasks: &[&Task]) -> Vec<bool> {
    let reachable = reachable_tasks
        .iter()
        .map(|task| crate::resolve::semantic_task_identity(program, task))
        .collect::<BTreeSet<_>>();
    fn collect(
        program: &Program,
        items: &[Item],
        reachable: &BTreeSet<String>,
        out: &mut Vec<bool>,
    ) {
        for item in items {
            match item {
                Item::App(app) => collect(program, &app.items, reachable, out),
                Item::Task(task) => out.push(
                    reachable.contains(&crate::resolve::semantic_task_identity(program, task)),
                ),
                Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
            }
        }
    }
    let mut mask = Vec::new();
    for file in &program.files {
        collect(program, &file.items, &reachable, &mut mask);
    }
    mask
}

fn runtime_type_scope(program: &Program, reachable_tasks: &[&Task]) -> Program {
    fn replace_param_type(param: &mut crate::ast::Param) {
        param.ty = "Int".to_string();
        param.type_syntax.kind = crate::ast::TypeSyntaxKind::Named {
            name: "Int".to_string(),
        };
    }

    fn retain_reachable_signatures(
        items: &mut [Item],
        reachability: &mut impl Iterator<Item = bool>,
    ) {
        for item in items {
            match item {
                Item::App(app) => retain_reachable_signatures(&mut app.items, reachability),
                Item::Task(task) => {
                    let reachable = reachability.next().expect("task reachability mask");
                    for param in &mut task.params {
                        if !reachable
                            || matches!(
                                param.type_syntax.kind,
                                crate::ast::TypeSyntaxKind::Callable(_)
                                    | crate::ast::TypeSyntaxKind::CallableCandidate { .. }
                            )
                        {
                            replace_param_type(param);
                        }
                    }
                    if !reachable {
                        if task.result.is_some() {
                            task.result = Some("Int".to_string());
                        }
                        if let Some(result) = &mut task.result_syntax {
                            result.kind = crate::ast::TypeSyntaxKind::Named {
                                name: "Int".to_string(),
                            };
                        }
                    }
                }
                Item::Test(test) => {
                    for param in &mut test.params {
                        replace_param_type(param);
                    }
                }
                Item::Type(type_def) => {
                    for field in &mut type_def.fields {
                        field.ty = "Int".to_string();
                    }
                }
                Item::Store(store) => store.ty = "Int".to_string(),
            }
        }
    }

    let mut scoped = program.clone();
    let mut reachability = task_reachability_mask(program, reachable_tasks).into_iter();
    for file in &mut scoped.files {
        retain_reachable_signatures(&mut file.items, &mut reachability);
    }
    assert!(reachability.next().is_none());
    scoped
}

fn output_policy_map(
    facts: Vec<OutputPolicyFact>,
) -> BTreeMap<(String, usize), Vec<OutputPolicyFact>> {
    let mut policies = BTreeMap::<(String, usize), Vec<OutputPolicyFact>>::new();
    for fact in facts {
        policies
            .entry((fact.task.clone(), fact.call_span.line))
            .or_default()
            .push(fact);
    }
    for facts in policies.values_mut() {
        facts.sort_by_key(|fact| fact.call_span.column);
    }
    policies
}

fn replay_policy_map(
    facts: Vec<ReplayPolicyFact>,
) -> BTreeMap<(String, usize), Vec<ReplayPolicyFact>> {
    let mut policies = BTreeMap::<(String, usize), Vec<ReplayPolicyFact>>::new();
    for fact in facts {
        policies
            .entry((fact.task.clone(), fact.call_span.line))
            .or_default()
            .push(fact);
    }
    for facts in policies.values_mut() {
        facts.sort_by_key(|fact| fact.call_span.column);
    }
    policies
}

fn file_policy_map(facts: Vec<FilePolicyFact>) -> BTreeMap<(String, usize), Vec<FilePolicyFact>> {
    let mut policies = BTreeMap::<(String, usize), Vec<FilePolicyFact>>::new();
    for fact in facts {
        policies
            .entry((fact.task.clone(), fact.call_span.line))
            .or_default()
            .push(fact);
    }
    for facts in policies.values_mut() {
        facts.sort_by_key(|fact| fact.call_span.column);
    }
    policies
}

fn output_failure(variant: &str, span: Span) -> FailureValue {
    FailureValue::root(
        FailureVariant {
            root: "OutputError".to_string(),
            variant: variant.to_string(),
        },
        span,
    )
}

fn replay_failure(variant: &str, span: Span) -> FailureValue {
    FailureValue::root(
        FailureVariant {
            root: "ReplayClockError".to_string(),
            variant: variant.to_string(),
        },
        span,
    )
}

fn file_failure(variant: &str, span: Span) -> FailureValue {
    FailureValue::root(
        FailureVariant {
            root: "FileReadError".to_string(),
            variant: variant.to_string(),
        },
        span,
    )
}

#[allow(clippy::too_many_arguments)]
fn output_audit_event(
    event_id: String,
    event_sequence: usize,
    request_id: String,
    policy: &OutputPolicyFact,
    event_kind: &'static str,
    decision: GrantDecision,
    operator_allow_present: bool,
    operator_deny_present: bool,
    adapter_called: bool,
    byte_count: usize,
    result: &'static str,
) -> AuthorityAuditEvent {
    AuthorityAuditEvent {
        event_id,
        event_sequence,
        request_id,
        source_policy_id: policy.policy_id.clone(),
        event_kind,
        authority_surface: "hum_run_cli",
        capability_id: "stdout.write",
        grant_kind: "output_stream",
        grant_scope: "app_stdout",
        grant_strength: "write",
        grant_lifetime: "one_run",
        app_name: policy.app_name.clone(),
        task: policy.task.clone(),
        call_span: policy.call_span.clone(),
        source_route: policy.source_route.clone(),
        source_route_spans: policy.source_route_spans.clone(),
        source_task_authorized: true,
        source_app_authorized: policy.app_name.is_some(),
        operator_allow_present,
        operator_deny_present,
        effective_decision: decision.effective(),
        decision_reason: decision.reason(),
        adapter_called,
        byte_count,
        replay_index: None,
        replay_tick: None,
        native_path_identity: None,
        native_path_matched: None,
        locality_status: None,
        result,
    }
}

#[allow(clippy::too_many_arguments)]
fn replay_audit_event(
    event_id: String,
    event_sequence: usize,
    request_id: String,
    policy: &ReplayPolicyFact,
    event_kind: &'static str,
    decision: GrantDecision,
    operator_allow_present: bool,
    operator_deny_present: bool,
    adapter_called: bool,
    replay_index: usize,
    replay_tick: Option<i64>,
    result: &'static str,
) -> AuthorityAuditEvent {
    AuthorityAuditEvent {
        event_id,
        event_sequence,
        request_id,
        source_policy_id: policy.policy_id.clone(),
        event_kind,
        authority_surface: "hum_run_cli",
        capability_id: "clock.replay",
        grant_kind: "replay_input",
        grant_scope: "runner_tick_sequence",
        grant_strength: "read_ordered",
        grant_lifetime: "one_run",
        app_name: policy.app_name.clone(),
        task: policy.task.clone(),
        call_span: policy.call_span.clone(),
        source_route: policy.source_route.clone(),
        source_route_spans: policy.source_route_spans.clone(),
        source_task_authorized: true,
        source_app_authorized: policy.app_name.is_some(),
        operator_allow_present,
        operator_deny_present,
        effective_decision: decision.effective(),
        decision_reason: decision.reason(),
        adapter_called,
        byte_count: 0,
        replay_index: Some(replay_index),
        replay_tick,
        native_path_identity: None,
        native_path_matched: None,
        locality_status: None,
        result,
    }
}

#[allow(clippy::too_many_arguments)]
fn file_audit_event(
    event_id: String,
    event_sequence: usize,
    request_id: String,
    policy: &FilePolicyFact,
    event_kind: &'static str,
    decision: GrantDecision,
    operator_allow_present: bool,
    operator_deny_present: bool,
    adapter_called: bool,
    byte_count: usize,
    native_path_identity: OsString,
    native_path_matched: bool,
    locality_status: &'static str,
    result: &'static str,
) -> AuthorityAuditEvent {
    AuthorityAuditEvent {
        event_id,
        event_sequence,
        request_id,
        source_policy_id: policy.policy_id.clone(),
        event_kind,
        authority_surface: "hum_run_cli",
        capability_id: "files.read",
        grant_kind: "file",
        grant_scope: "exact_native_path",
        grant_strength: "read",
        grant_lifetime: "one_run",
        app_name: policy.app_name.clone(),
        task: policy.task.clone(),
        call_span: policy.call_span.clone(),
        source_route: policy.source_route.clone(),
        source_route_spans: policy.source_route_spans.clone(),
        source_task_authorized: true,
        source_app_authorized: policy.app_name.is_some(),
        operator_allow_present,
        operator_deny_present,
        effective_decision: decision.effective(),
        decision_reason: decision.reason(),
        adapter_called,
        byte_count,
        replay_index: None,
        replay_tick: None,
        native_path_identity: Some(native_path_identity),
        native_path_matched: Some(native_path_matched),
        locality_status: Some(locality_status),
        result,
    }
}

struct Interpreter<'program, 'output> {
    program: &'program Program,
    callable_analysis: Arc<CallableAnalysis>,
    predicate_analysis: Arc<predicate::PredicateAnalysis>,
    diagnostics: RefCell<Vec<Diagnostic>>,
    diagnostic_occurrences: RefCell<DiagnosticOccurrenceCollector>,
    active_iterations: RefCell<Vec<ActiveIteration>>,
    active_app: RefCell<Option<&'program App>>,
    grant_policy: &'output OperatorGrantPolicy,
    output: RefCell<OutputRuntime<'output>>,
    replay: RefCell<ReplayRuntime<'output>>,
    file_adapter: RefCell<&'output mut dyn FileReadAdapter>,
    file_locality: RefCell<&'output mut dyn FileLocalityAdapter>,
    output_policies: BTreeMap<(String, usize), Vec<OutputPolicyFact>>,
    replay_policies: BTreeMap<(String, usize), Vec<ReplayPolicyFact>>,
    file_policies: BTreeMap<(String, usize), Vec<FilePolicyFact>>,
    output_call_cursors: RefCell<BTreeMap<(String, usize, String), usize>>,
    replay_call_cursors: RefCell<BTreeMap<(String, usize, String), usize>>,
    file_call_cursors: RefCell<BTreeMap<(String, usize, String), usize>>,
    output_task_call_cursors: RefCell<BTreeMap<(String, usize, String, String), usize>>,
    active_task_route: RefCell<Vec<String>>,
    active_task_definition_ids: RefCell<Vec<String>>,
    active_call_route: RefCell<Vec<Span>>,
    active_callable_applications: RefCell<Vec<String>>,
    authority_events: RefCell<Vec<AuthorityAuditEvent>>,
}

enum TaskResult {
    Returned(Value),
    Failed(FailureValue),
    ContractViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContractKind {
    Needs,
    Ensures,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpressionContext {
    Body,
    Contract,
}

impl<'program, 'output> Interpreter<'program, 'output> {
    fn run(
        &self,
        entry: Option<&str>,
        raw_args: &[OsString],
    ) -> Result<(TaskResult, bool), String> {
        let (task, app_mode) = self.entry_task(entry)?;
        if self.emit_exact_occurrences(
            crate::path_boundary::diagnostic_occurrence_set(self.program)
                .occurrences()
                .cloned()
                .collect(),
        )? {
            return Err(DIAGNOSTIC_PREFLIGHT_REJECTED.to_string());
        }
        if self.emit_exact_occurrences(
            capability_root::diagnostic_occurrence_set(self.program)
                .occurrences()
                .cloned()
                .collect(),
        )? {
            return Err(DIAGNOSTIC_PREFLIGHT_REJECTED.to_string());
        }
        let reachable_tasks = self.reachable_type_tasks(task);
        if self.emit_exact_occurrences(self.reachable_type_occurrences(&reachable_tasks))? {
            return Err(DIAGNOSTIC_PREFLIGHT_REJECTED.to_string());
        }
        self.preflight_reachable_typed_failures(&reachable_tasks)?;
        let callable_occurrences = callable::diagnostic_occurrences(self.program);
        if self.emit_exact_occurrences(callable_occurrences)? {
            return Err(DIAGNOSTIC_PREFLIGHT_REJECTED.to_string());
        }
        self.preflight_reachable_predicates(&reachable_tasks)?;
        let ownership_blockers = self.reachable_ownership_blockers(&reachable_tasks)?;
        if let Some(first) = ownership_blockers.first() {
            let first_projection = first.occurrence().diagnostic().clone();
            for blocker in &ownership_blockers {
                blocker
                    .validate()
                    .map_err(|error| format!("diagnostic invariant failure: {error:?}"))?;
                let occurrence = blocker.occurrence();
                self.emit_occurrence(occurrence, occurrence.diagnostic().clone())?;
            }
            return Err(format!(
                "{} {}",
                first_projection.code.as_str(),
                first_projection.code.title()
            ));
        }
        let args = self.parse_args(task, raw_args, app_mode)?;
        self.execute_task(task, args)
            .map(|result| (result, app_mode))
    }

    fn emit_occurrence(
        &self,
        occurrence: &DiagnosticOccurrence,
        public_projection: Diagnostic,
    ) -> Result<(), String> {
        let diagnostic = self
            .diagnostic_occurrences
            .borrow_mut()
            .consume_exact(occurrence, public_projection)
            .map_err(|error| format!("diagnostic invariant failure: {error:?}"))?;
        self.diagnostics.borrow_mut().push(diagnostic);
        Ok(())
    }

    fn emit_exact_occurrences(
        &self,
        mut occurrences: Vec<DiagnosticOccurrence>,
    ) -> Result<bool, String> {
        occurrences.sort_by(occurrence_public_order);
        for occurrence in &occurrences {
            self.emit_occurrence(occurrence, occurrence.diagnostic().clone())?;
        }
        Ok(!occurrences.is_empty())
    }

    fn reachable_type_occurrences(&self, reachable_tasks: &[&Task]) -> Vec<DiagnosticOccurrence> {
        // Keep the existing public-diagnostic helper available to its static
        // consumers without using it to choose private runtime identity.
        let _public_projection_compatibility = type_check::unknown_type_diagnostics_for_tasks;
        let scoped = runtime_type_scope(self.program, reachable_tasks);
        type_check::diagnostic_occurrence_set(&scoped, &[])
            .occurrences()
            .filter(|occurrence| {
                occurrence.cause_key()
                    == crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(82)
            })
            .cloned()
            .collect()
    }

    fn reachable_ownership_blockers(
        &self,
        reachable_tasks: &[&Task],
    ) -> Result<Vec<ownership_check::OwnershipRuntimeBlocker>, String> {
        let reachable = reachable_tasks
            .iter()
            .map(|task| crate::resolve::semantic_task_identity(self.program, task))
            .collect::<BTreeSet<_>>();
        ownership_check::runtime_use_after_move_blockers(self.program, &reachable)
            .map_err(|error| format!("diagnostic invariant failure: {error:?}"))
    }

    fn reachable_type_tasks(&self, entry: &'program Task) -> Vec<&'program Task> {
        let mut pending = vec![entry];
        let mut visited = BTreeSet::new();
        let mut reachable = Vec::new();
        while let Some(task) = pending.pop() {
            let identity = (task.span.file.clone(), task.span.line, task.span.column);
            if !visited.insert(identity) {
                continue;
            }
            reachable.push(task);
            for target_definition_id in self
                .callable_analysis
                .callable_argument_target_definition_ids(task)
            {
                if let Some(target) = self.task_by_definition_id(target_definition_id) {
                    pending.push(target);
                }
            }
            let Some(does) = task.section("does") else {
                continue;
            };
            let body = core_body::analyze_does_section(does);
            for statement in &body.statements {
                let mut resolver_owned_callable_occurrence = false;
                if let Some(application) = self
                    .callable_analysis
                    .direct_application(task, &statement.span)
                {
                    resolver_owned_callable_occurrence = true;
                    if let Some(receiver) =
                        self.task_by_definition_id(&application.receiver_definition_id)
                    {
                        pending.push(receiver);
                    }
                    if let Some(target) =
                        self.task_by_definition_id(&application.target_definition_id)
                    {
                        pending.push(target);
                    }
                }
                for receiver_definition_id in self
                    .callable_analysis
                    .callable_callee_target_definition_ids(task, &statement.span)
                {
                    resolver_owned_callable_occurrence = true;
                    if let Some(receiver) = self.task_by_definition_id(receiver_definition_id) {
                        pending.push(receiver);
                    }
                }
                if resolver_owned_callable_occurrence {
                    continue;
                }
                let Some(expression) = typed_failure::statement_expression_node(statement) else {
                    continue;
                };
                for call in typed_failure::calls_in_canonical(expression) {
                    if let Some(callee) = self.find_task(&call.callee) {
                        pending.push(callee);
                    }
                }
            }
        }
        reachable
    }

    fn entry_task(&self, entry: Option<&str>) -> Result<(&'program Task, bool), String> {
        if let Some(name) = entry {
            return self
                .find_task(name)
                .map(|task| (task, false))
                .ok_or_else(|| format!("entry task `{name}` was not found"));
        }

        let analysis = app_entry::analyze(self.program);
        if let Some(diagnostic) = analysis.diagnostic {
            let code = diagnostic.code.as_str();
            let title = diagnostic.code.title();
            let occurrence = analysis.diagnostic_occurrence.ok_or_else(|| {
                "diagnostic invariant failure: app-entry diagnostic has no occurrence".to_string()
            })?;
            self.emit_occurrence(&occurrence, diagnostic)?;
            return Err(format!("{code} {title}"));
        }
        if let Some(entry) = analysis.entry {
            self.active_app.replace(Some(entry.app));
            return Ok((entry.task, true));
        }

        let mut tasks = Vec::new();
        for file in &self.program.files {
            collect_tasks(&file.items, &mut tasks);
        }
        match tasks.as_slice() {
            [task] => Ok((*task, false)),
            [] => Err("no task is available to run".to_string()),
            _ => Err("multiple tasks are available; pass `--entry <task>`".to_string()),
        }
    }

    fn find_task(&self, name: &str) -> Option<&'program Task> {
        if let Some(app) = *self.active_app.borrow() {
            return find_task_in_items(&app.items, name);
        }
        self.program
            .files
            .iter()
            .find_map(|file| find_task_in_items(&file.items, name))
    }

    fn parse_args(
        &self,
        task: &Task,
        raw_args: &[OsString],
        app_mode: bool,
    ) -> Result<Vec<Value>, String> {
        if raw_args.len() != task.params.len() {
            return Err(format!(
                "task `{}` expects {} argument(s), got {}",
                task.name,
                task.params.len(),
                raw_args.len()
            ));
        }

        let path_parameters = task
            .params
            .iter()
            .filter(|parameter| parameter.ty.trim() == "Path")
            .count();
        if path_parameters > 1 {
            return Err(format!(
                "task `{}` declares more than one Path parameter; structural app entry accepts at most one",
                task.name
            ));
        }

        task.params
            .iter()
            .zip(raw_args)
            .map(|(param, raw)| parse_arg(&param.ty, raw, app_mode))
            .collect()
    }

    fn execute_task(&self, task: &Task, args: Vec<Value>) -> Result<TaskResult, String> {
        self.active_task_route.borrow_mut().push(task.name.clone());
        let definition_id = self
            .callable_analysis
            .definition_id_for_task(task)
            .map(str::to_string)
            .unwrap_or_else(|| format!("runtime-task:{}:{}", task.name, task.span.line));
        self.active_task_definition_ids
            .borrow_mut()
            .push(definition_id);
        let result = self.execute_task_body(task, args);
        self.active_task_definition_ids.borrow_mut().pop();
        let popped = self.active_task_route.borrow_mut().pop();
        debug_assert_eq!(popped.as_deref(), Some(task.name.as_str()));
        result
    }

    fn current_task(&self) -> Option<&'program Task> {
        let id = self.active_task_definition_ids.borrow().last()?.clone();
        self.task_by_definition_id(&id)
    }

    fn task_by_definition_id(&self, definition_id: &str) -> Option<&'program Task> {
        let mut tasks = Vec::new();
        for file in &self.program.files {
            collect_tasks(&file.items, &mut tasks);
        }
        tasks
            .into_iter()
            .find(|task| self.callable_analysis.definition_id_for_task(task) == Some(definition_id))
    }

    fn execute_task_body(&self, task: &Task, args: Vec<Value>) -> Result<TaskResult, String> {
        let Some(does) = task.section("does") else {
            return Err(format!("task `{}` has no `does:` section", task.name));
        };
        let body = core_body::analyze_does_section(does);
        let mut existing_names = task
            .params
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<BTreeSet<_>>();
        for section_name in ["uses", "changes"] {
            let Some(section) = task.section(section_name) else {
                continue;
            };
            existing_names.extend(
                section
                    .lines
                    .iter()
                    .filter(|line| is_meaningful_line_text(&line.text))
                    .map(|line| declaration_root(line.text.trim())),
            );
        }
        let alias_analysis =
            writable_field_alias::analyze_with_existing_names(&body.statements, &existing_names);
        self.preflight_writable_aliases(task, &body.statements, &alias_analysis)?;

        let mut env = Env::new();
        for (param, value) in task.params.iter().zip(args) {
            let definition_id = self
                .callable_analysis
                .definition_id_for_span(&param.span)
                .map(str::to_string);
            env.insert(
                param.name.clone(),
                RuntimeBinding::parameter(value, param.permission, definition_id),
            );
        }

        if !self.evaluate_contract_section(task, "needs", ContractKind::Needs, &mut env)? {
            return Ok(TaskResult::ContractViolation);
        }

        self.capture_old_contract_values(task, &mut env)?;

        let lines = executable_lines(does);
        match self.eval_block(&lines, 0, lines.len(), &mut env, &task.name)? {
            Flow::Return { value, root, span } => {
                self.ensure_return_dependency(task, root.as_deref(), &span)?;
                self.finish_success(task, value, &env)
            }
            Flow::Fail(value) => Ok(TaskResult::Failed(value)),
            Flow::Continue => {
                self.ensure_linear_closed_on_exit(&env, &task.name, "fallthrough", &task.span)?;
                self.finish_success(task, Value::Unit, &env)
            }
            Flow::ContractViolation => Ok(TaskResult::ContractViolation),
        }
    }

    fn preflight_writable_aliases(
        &self,
        task: &Task,
        statements: &[BodyStatement],
        analysis: &AliasAnalysis,
    ) -> Result<(), String> {
        for binding in &analysis.bindings {
            let borrowed_owner = task.params.iter().any(|param| {
                param.name == binding.owner_root && param.permission == ParamPermission::Borrow
            });
            if borrowed_owner {
                return Err(self.writable_alias_authority_trap(
                    task,
                    binding,
                    DiagnosticCode::BORROW_PARAMETER_MUTATION,
                    "borrow",
                ));
            }
        }

        if let Some(issue) = analysis.issues.first() {
            let (code, message) = match issue.kind {
                AliasIssueKind::Overlap => (
                    DiagnosticCode::WRITABLE_ALIAS_OVERLAP,
                    writable_field_alias::overlap_message(issue),
                ),
                AliasIssueKind::Unsupported => (
                    DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS,
                    writable_field_alias::unsupported_message(issue),
                ),
            };
            let diagnostic = Diagnostic::error(code, message, Some(issue.conflict_span.clone()))
                .with_help(writable_field_alias::issue_help(&task.name, issue));
            self.diagnostics.borrow_mut().push(diagnostic);
            return Err(format!("{} {}", code.as_str(), code.title()));
        }

        for binding in &analysis.bindings {
            let permission = task
                .params
                .iter()
                .find(|param| param.name == binding.owner_root)
                .map(|param| param.permission);
            match permission {
                Some(ParamPermission::Change | ParamPermission::Consume) => continue,
                Some(ParamPermission::Borrow) => {
                    return Err(self.writable_alias_authority_trap(
                        task,
                        binding,
                        DiagnosticCode::BORROW_PARAMETER_MUTATION,
                        "borrow",
                    ));
                }
                None => {}
            }

            let owner = statements
                .iter()
                .enumerate()
                .take(binding.binding_index)
                .rev()
                .find_map(|(_index, statement)| {
                    statement
                        .binding_name()
                        .and_then(|name| (name == binding.owner_root).then_some(statement.kind))
                });
            if owner == Some("mutable_binding") {
                continue;
            }
            let authority = if owner == Some("let_binding") {
                "immutable let"
            } else {
                "unknown"
            };
            return Err(self.writable_alias_authority_trap(
                task,
                binding,
                DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS,
                authority,
            ));
        }
        Ok(())
    }

    fn preflight_reachable_typed_failures(&self, reachable_tasks: &[&Task]) -> Result<(), String> {
        let analysis = typed_failure::analyze_program(self.program);
        let occurrences = reachable_tasks
            .iter()
            .flat_map(|task| {
                task.section("does")
                    .map(core_body::analyze_does_section)
                    .into_iter()
                    .flat_map(|body| {
                        body.statements
                            .into_iter()
                            .enumerate()
                            .filter_map(|(index, _statement)| {
                                analysis
                                    .fact(task, index)
                                    .and_then(|fact| fact.occurrence.clone())
                            })
                    })
            })
            .collect::<Vec<_>>();
        if self.emit_exact_occurrences(occurrences)? {
            return Err(DIAGNOSTIC_PREFLIGHT_REJECTED.to_string());
        }
        Ok(())
    }

    fn writable_alias_authority_trap(
        &self,
        task: &Task,
        binding: &AliasBinding,
        code: DiagnosticCode,
        authority: &str,
    ) -> String {
        let diagnostic = Diagnostic::error(
            code,
            writable_field_alias::authority_message(binding),
            Some(binding.binding_span.clone()),
        )
        .with_help(writable_field_alias::authority_help(
            &task.name, binding, authority,
        ));
        self.diagnostics.borrow_mut().push(diagnostic);
        format!("{} {}", code.as_str(), code.title())
    }

    fn ensure_return_dependency(
        &self,
        task: &Task,
        returned_root: Option<&str>,
        span: &Span,
    ) -> Result<(), String> {
        let Some(dependency) = task
            .result
            .as_deref()
            .and_then(return_dependency::parse_return_dependency)
        else {
            return Ok(());
        };
        let source = dependency.source.as_str();
        let source_is_parameter = return_dependency::is_bare_source_name(source)
            && task.params.iter().any(|param| param.name == source);
        if !source_is_parameter || returned_root != Some(source) {
            return Err(self.return_dependency_trap(&task.name, source, span));
        }
        Ok(())
    }

    fn finish_success(&self, task: &Task, value: Value, env: &Env) -> Result<TaskResult, String> {
        let mut exit_env = env.clone();
        exit_env.insert(
            "result".to_string(),
            RuntimeBinding::local(value.clone(), false),
        );
        if !self.evaluate_contract_section(task, "ensures", ContractKind::Ensures, &mut exit_env)? {
            return Ok(TaskResult::ContractViolation);
        }
        Ok(TaskResult::Returned(value))
    }

    fn capture_old_contract_values(&self, task: &Task, env: &mut Env) -> Result<(), String> {
        for fact in self.predicate_analysis.facts_for_task(task).filter(|fact| {
            fact.section == "ensures" && fact.status == RecognitionStatus::RecognizedTyped
        }) {
            for place in old_places(&fact.canonical) {
                let inner = crate::core_expr::canonical_text(&place);
                let key = format!("old({inner})");
                if env.contains_key(&key) {
                    continue;
                }
                let value =
                    match self.eval_canonical_at(&place, env, &task.name, &fact.line_span)? {
                        Evaluated::Value(value) => value,
                        Evaluated::Failure(value) => {
                            return Err(format!(
                                "old capture of `{inner}` produced failure {}",
                                value.identity()
                            ));
                        }
                        Evaluated::ContractViolation => {
                            return Err(format!(
                                "old capture of `{inner}` hit a contract violation"
                            ));
                        }
                    };
                env.insert(key, RuntimeBinding::local(value, false));
            }
        }
        Ok(())
    }

    fn preflight_reachable_predicates(&self, reachable_tasks: &[&Task]) -> Result<(), String> {
        let occurrences = reachable_tasks
            .iter()
            .flat_map(|task| self.predicate_analysis.facts_for_task(task))
            .filter_map(|fact| fact.diagnostic_occurrence())
            .collect::<Vec<_>>();
        if !occurrences.is_empty() {
            for occurrence in occurrences {
                self.emit_occurrence(&occurrence, occurrence.diagnostic().clone())?;
            }
            return Err(DIAGNOSTIC_PREFLIGHT_REJECTED.to_string());
        }
        Ok(())
    }

    fn evaluate_contract_section(
        &self,
        task: &Task,
        section_name: &str,
        kind: ContractKind,
        env: &mut Env,
    ) -> Result<bool, String> {
        let Some(section) = task.section(section_name) else {
            return Ok(true);
        };

        for line in &section.lines {
            let text = line.text.trim();
            if !is_meaningful_line_text(text) {
                continue;
            }

            let Some(fact) = self
                .predicate_analysis
                .fact_for_line(task, section_name, line)
            else {
                continue;
            };
            if fact.status == RecognitionStatus::NonExecutableProse {
                self.diagnostics.borrow_mut().push(
                    Diagnostic::warning(
                        DiagnosticCode::UNCHECKED_PROSE_CONTRACT,
                        format!("unchecked prose {} contract: {text}", kind.section_name()),
                        Some(line.span.clone()),
                    )
                    .with_help(
                        "Predicate v2 checks one typed comparison over eligible places, including exact Text/List Text equality and contract-only `list_count`; prose remains visible but unchecked.",
                    ),
                );
                continue;
            }
            let value = match self.eval_canonical_with_context(
                &fact.canonical,
                env,
                &task.name,
                &line.span,
                ExpressionContext::Contract,
            )? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(failure) => {
                    return Err(format!(
                        "typed predicate produced failure {}",
                        failure.identity()
                    ));
                }
                Evaluated::ContractViolation => {
                    return Err("typed predicate evaluation hit a contract violation".to_string());
                }
            };
            if as_bool(&value)? {
                continue;
            }

            let (code, message, help) = match kind {
                ContractKind::Needs => (
                    DiagnosticCode::NEEDS_CONTRACT_VIOLATION,
                    format!("caller did not satisfy needs: {text}"),
                    format!(
                        "The caller must pass arguments that make this predicate true before task `{}` runs.",
                        task.name
                    ),
                ),
                ContractKind::Ensures => (
                    DiagnosticCode::ENSURES_CONTRACT_VIOLATION,
                    format!("task `{}` did not satisfy ensures: {text}", task.name),
                    "Fix the task body or change the contract; task blame means the caller met entry conditions but the implementation broke its promise."
                        .to_string(),
                ),
            };
            self.diagnostics
                .borrow_mut()
                .push(Diagnostic::error(code, message, Some(line.span.clone())).with_help(help));
            return Ok(false);
        }

        Ok(true)
    }

    fn eval_block(
        &self,
        lines: &[ExecLine],
        start: usize,
        end: usize,
        env: &mut Env,
        task_name: &str,
    ) -> Result<Flow, String> {
        let mut index = start;
        while index < end {
            let line = &lines[index];
            let text = line.text.as_str();

            if line.parsed.block_relationship == ParsedBlockRelationship::Closes {
                return Ok(Flow::Continue);
            }

            if line.parsed.core_kind == "if_header" {
                let close = matching_close(lines, index)?;
                let condition = statement_expression(&line.parsed).ok_or_else(|| {
                    format!("{}: if header has no expression fact", line.location)
                })?;
                match self.eval_canonical_at(condition, env, task_name, &line.span)? {
                    Evaluated::Value(value) if as_bool(&value)? => {
                        let flow = self.eval_block(lines, index + 1, close, env, task_name)?;
                        if flow != Flow::Continue {
                            return Ok(flow);
                        }
                    }
                    Evaluated::Value(_) => {}
                    Evaluated::Failure(value) => return Ok(Flow::Fail(value)),
                    Evaluated::ContractViolation => return Ok(Flow::ContractViolation),
                }
                index = close + 1;
                continue;
            }

            if line.parsed.core_kind == "for_each_header" {
                let close = matching_close(lines, index)?;
                let name = line.parsed.facts.loop_binding.as_ref().ok_or_else(|| {
                    format!("{}: malformed `for each` binding fact", line.location)
                })?;
                let collection_node = statement_expression(&line.parsed).ok_or_else(|| {
                    format!("{}: for-each header has no expression fact", line.location)
                })?;
                let collection =
                    match self.eval_canonical_at(collection_node, env, task_name, &line.span)? {
                        Evaluated::Value(value) => value,
                        Evaluated::Failure(value) => return Ok(Flow::Fail(value)),
                        Evaluated::ContractViolation => return Ok(Flow::ContractViolation),
                    };
                let Value::List(values) = collection else {
                    return Err(format!(
                        "{}: `for each` requires a list value",
                        line.location
                    ));
                };

                let active_iteration = collection_node
                    .direct_place()
                    .map(|(root, _, _)| root)
                    .map(|root| self.push_active_iteration(root.to_string(), line.span.clone()));
                let name = name.name.as_str();
                let previous = env.get(name).cloned();
                for value in values {
                    env.insert(name.to_string(), RuntimeBinding::local(value, false));
                    let flow = match self.eval_block(lines, index + 1, close, env, task_name) {
                        Ok(flow) => flow,
                        Err(error) => {
                            restore_binding(env, name, previous.clone());
                            if active_iteration.is_some() {
                                self.pop_active_iteration();
                            }
                            return Err(error);
                        }
                    };
                    if flow != Flow::Continue {
                        restore_binding(env, name, previous);
                        if active_iteration.is_some() {
                            self.pop_active_iteration();
                        }
                        return Ok(flow);
                    }
                }
                restore_binding(env, name, previous);
                if active_iteration.is_some() {
                    self.pop_active_iteration();
                }
                index = close + 1;
                continue;
            }

            if let ParsedBodyStatementKind::Return(expression) = &line.parsed.kind {
                return match self.eval_canonical_at(
                    &expression.canonical,
                    env,
                    task_name,
                    &line.span,
                )? {
                    Evaluated::Value(value) => {
                        let root =
                            return_dependency::visible_view_source_root(&expression.canonical);
                        if let Some(root) = root.as_deref()
                            && !is_linear_binding(env, root)
                        {
                            self.mark_moved(env, root, &line.span, "return");
                        }
                        self.ensure_linear_closed_on_exit(env, task_name, "return", &line.span)?;
                        Ok(Flow::Return {
                            value,
                            root,
                            span: line.span.clone(),
                        })
                    }
                    Evaluated::Failure(value) => {
                        self.ensure_linear_closed_on_exit(env, task_name, "fail", &line.span)?;
                        Ok(Flow::Fail(value))
                    }
                    Evaluated::ContractViolation => Ok(Flow::ContractViolation),
                };
            }

            if line.parsed.core_kind == "fail" {
                let expression = statement_expression(&line.parsed).ok_or_else(|| {
                    format!("{}: typed fail has no expression fact", line.location)
                })?;
                let rendered = crate::core_expr::canonical_text(expression);
                let Some(variant) = typed_failure::parse_failure_variant(&rendered) else {
                    return Err(format!(
                        "{}: typed `fail` requires `ErrorRoot.variant`",
                        line.location
                    ));
                };
                self.ensure_linear_closed_on_exit(env, task_name, "fail", &line.span)?;
                return Ok(Flow::Fail(FailureValue::root(variant, line.span.clone())));
            }

            if matches!(&line.parsed.kind, ParsedBodyStatementKind::Binding { .. }) {
                if let Some(flow) = self.eval_binding(&line.parsed, env, &line.span, task_name)? {
                    return Ok(flow);
                }
                index += 1;
                continue;
            }

            if line.parsed.core_kind == "set_place" {
                let target = line
                    .parsed
                    .facts
                    .set_target
                    .as_ref()
                    .ok_or_else(|| format!("{}: set has no target fact", line.location))?;
                let value = statement_expression(&line.parsed)
                    .ok_or_else(|| format!("{}: set has no expression fact", line.location))?;
                if let Some(flow) =
                    self.eval_set(&target.canonical, value, env, &line.span, task_name)?
                {
                    return Ok(flow);
                }
                index += 1;
                continue;
            }

            return Err(format!(
                "{}: unsupported executable line `{text}`",
                line.location
            ));
        }
        Ok(Flow::Continue)
    }

    fn eval_binding(
        &self,
        binding: &ParsedBodyStatement,
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Option<Flow>, String> {
        let ParsedBodyStatementKind::Binding {
            mutable,
            name: parsed_name,
            value: parsed_value,
        } = &binding.kind
        else {
            return Err("internal binding evaluator received a non-binding fact".to_string());
        };
        let parsed_value = parsed_value
            .as_ref()
            .ok_or_else(|| "binding fact is missing an initializer".to_string())?;
        let name = parsed_name
            .as_ref()
            .map(|name| name.name.as_str())
            .ok_or_else(|| "binding fact is missing a name".to_string())?;
        let annotation = binding.facts.binding_annotation.as_deref();
        if !*mutable && let Some(alias) = writable_field_alias::exact_binding_from_parser(binding) {
            let source_place = RuntimePlace::Field {
                root: alias.owner_root,
                field: alias.source_field,
            };
            self.ensure_can_set(env, &source_place, span, task_name)?;
            env.insert(
                name.to_string(),
                RuntimeBinding::writable_alias(source_place),
            );
            return Ok(None);
        }
        if !*mutable
            && let CanonicalExpressionKind::Permission { permission, value } =
                &parsed_value.canonical.kind
            && matches!(permission, ParamPermission::Borrow)
        {
            let source_place = RuntimePlace::from_canonical(value)
                .ok_or_else(|| "borrowed view source is not a canonical place".to_string())?;
            let kind = if matches!(value.kind, CanonicalExpressionKind::Element { .. }) {
                RuntimeViewKind::Element
            } else {
                RuntimeViewKind::Field
            };
            let value = match self.eval_canonical(value, env, task_name)? {
                Evaluated::Value(value) => value,
                Evaluated::Failure(value) => return Ok(Some(Flow::Fail(value))),
                Evaluated::ContractViolation => return Ok(Some(Flow::ContractViolation)),
            };
            env.insert(
                name.to_string(),
                RuntimeBinding::view(value, kind, source_place, span.clone()),
            );
            return Ok(None);
        }
        let linear = annotation.is_some_and(is_linear_resource_type);
        match self.eval_canonical_at(&parsed_value.canonical, env, task_name, span)? {
            Evaluated::Value(value) => {
                let binding = if *mutable {
                    RuntimeBinding::mutable_local(value, linear)
                } else {
                    RuntimeBinding::local(value, linear)
                };
                env.insert(name.to_string(), binding);
                Ok(None)
            }
            Evaluated::Failure(value) => Ok(Some(Flow::Fail(value))),
            Evaluated::ContractViolation => Ok(Some(Flow::ContractViolation)),
        }
    }

    fn eval_set(
        &self,
        target: &CanonicalExpression,
        parsed_value: &CanonicalExpression,
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Option<Flow>, String> {
        let place = RuntimePlace::from_canonical(target)
            .ok_or_else(|| "set target is not a canonical place".to_string())?;
        let effective_place = self.resolve_writable_alias_place(env, &place)?;
        self.ensure_can_set(env, &effective_place, span, task_name)?;
        match self.eval_canonical_at(parsed_value, env, task_name, span)? {
            Evaluated::Value(value) => {
                self.write_place(env, &effective_place, value)?;
                invalidate_field_views(env, &effective_place, span);
                Ok(None)
            }
            Evaluated::Failure(value) => Ok(Some(Flow::Fail(value))),
            Evaluated::ContractViolation => Ok(Some(Flow::ContractViolation)),
        }
    }

    fn eval_canonical(
        &self,
        expression: &CanonicalExpression,
        env: &mut Env,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        self.eval_canonical_with_context(
            expression,
            env,
            task_name,
            &expression.range.start,
            ExpressionContext::Body,
        )
    }

    fn eval_canonical_at(
        &self,
        expression: &CanonicalExpression,
        env: &mut Env,
        task_name: &str,
        span: &Span,
    ) -> Result<Evaluated, String> {
        self.eval_canonical_with_context(expression, env, task_name, span, ExpressionContext::Body)
    }

    fn eval_canonical_with_context(
        &self,
        expression: &CanonicalExpression,
        env: &mut Env,
        task_name: &str,
        span: &Span,
        context: ExpressionContext,
    ) -> Result<Evaluated, String> {
        match &expression.kind {
            CanonicalExpressionKind::Unit => Ok(Evaluated::Value(Value::Unit)),
            CanonicalExpressionKind::UIntLiteral(value) => i64::try_from(*value)
                .map(Value::Int)
                .map(Evaluated::Value)
                .map_err(|_| format!("integer overflow while evaluating `{value}`")),
            CanonicalExpressionKind::IntLiteral(value) => Ok(Evaluated::Value(Value::Int(*value))),
            CanonicalExpressionKind::BoolLiteral(value) => {
                Ok(Evaluated::Value(Value::Bool(*value)))
            }
            CanonicalExpressionKind::TextLiteral(value) => {
                Ok(Evaluated::Value(Value::Text(value.clone())))
            }
            CanonicalExpressionKind::Identifier(name) => {
                if env.contains_key(name) {
                    self.read_value(env, name, span, task_name)
                        .map(Evaluated::Value)
                } else {
                    Err(format!("unknown expression `{name}`"))
                }
            }
            CanonicalExpressionKind::Field { base, field } => {
                if let CanonicalExpressionKind::Identifier(root) = &base.kind
                    && root
                        .chars()
                        .next()
                        .is_some_and(|ch| ch.is_ascii_uppercase())
                {
                    return Ok(Evaluated::Value(Value::Variant(format!("{root}.{field}"))));
                }
                let base_text = crate::core_expr::canonical_text(base);
                let value =
                    match self.eval_canonical_with_context(base, env, task_name, span, context)? {
                        Evaluated::Value(value) => value,
                        other => return Ok(other),
                    };
                let Value::Record(fields) = value else {
                    return Err(format!("`{base_text}` is not a record"));
                };
                fields
                    .get(field)
                    .cloned()
                    .map(Evaluated::Value)
                    .ok_or_else(|| format!("record `{base_text}` has no field `{field}`"))
            }
            CanonicalExpressionKind::Element { base, index } => {
                let base_text = crate::core_expr::canonical_text(base);
                let value =
                    match self.eval_canonical_with_context(base, env, task_name, span, context)? {
                        Evaluated::Value(value) => value,
                        other => return Ok(other),
                    };
                let Value::List(values) = value else {
                    return Err(format!("{base_text} is not a list"));
                };
                values
                    .get(*index)
                    .cloned()
                    .map(Evaluated::Value)
                    .ok_or_else(|| format!("list {base_text} has no element at index {index}"))
            }
            CanonicalExpressionKind::ListLiteral(values) => {
                let mut result = Vec::new();
                for value in values {
                    match self.eval_canonical_with_context(value, env, task_name, span, context)? {
                        Evaluated::Value(value) => result.push(value),
                        other => return Ok(other),
                    }
                }
                Ok(Evaluated::Value(Value::List(result)))
            }
            CanonicalExpressionKind::RecordLiteral { fields, .. } => {
                let mut result = BTreeMap::new();
                for (field, value) in fields {
                    match self.eval_canonical_with_context(value, env, task_name, span, context)? {
                        Evaluated::Value(value) => {
                            result.insert(field.clone(), value);
                        }
                        other => return Ok(other),
                    }
                }
                Ok(Evaluated::Value(Value::Record(result)))
            }
            CanonicalExpressionKind::Permission { value, .. }
            | CanonicalExpressionKind::Group(value) => {
                self.eval_canonical_with_context(value, env, task_name, span, context)
            }
            CanonicalExpressionKind::Try { call, wrapper } => {
                match self.eval_canonical_with_context(call, env, task_name, span, context)? {
                    Evaluated::Value(value) => Ok(Evaluated::Value(value)),
                    Evaluated::Failure(failure) => {
                        let callee = canonical_callee_name(call).unwrap_or("<unknown>");
                        let failure = if let Some(wrapper) = wrapper {
                            let wrapper = typed_failure::parse_failure_variant(wrapper)
                                .ok_or_else(|| {
                                    "checked typed-failure wrapper is invalid".to_string()
                                })?;
                            failure.wrap(wrapper, span.clone(), callee.to_string())
                        } else {
                            failure.propagate(span.clone(), callee.to_string())
                        };
                        Ok(Evaluated::Failure(failure))
                    }
                    Evaluated::ContractViolation => Ok(Evaluated::ContractViolation),
                }
            }
            CanonicalExpressionKind::Binary {
                operator,
                left,
                right,
                ..
            } => {
                let left_value =
                    match self.eval_canonical_with_context(left, env, task_name, span, context)? {
                        Evaluated::Value(value) => value,
                        other => return Ok(other),
                    };
                if *operator == ParsedBinaryOperator::Or && as_bool(&left_value)? {
                    return Ok(Evaluated::Value(Value::Bool(true)));
                }
                if *operator == ParsedBinaryOperator::And && !as_bool(&left_value)? {
                    return Ok(Evaluated::Value(Value::Bool(false)));
                }
                let right_value =
                    match self.eval_canonical_with_context(right, env, task_name, span, context)? {
                        Evaluated::Value(value) => value,
                        other => return Ok(other),
                    };
                apply_binary_operator(*operator, left_value, right_value, expression)
                    .map(Evaluated::Value)
            }
            CanonicalExpressionKind::Call { callee, arguments } => {
                self.eval_canonical_call(callee, arguments, env, span, task_name, context)
            }
            CanonicalExpressionKind::Unsupported => Err(format!(
                "unknown expression `{}`",
                crate::core_expr::canonical_text(expression)
            )),
        }
    }

    fn eval_canonical_call(
        &self,
        callee: &CanonicalExpression,
        arguments: &[CanonicalExpression],
        env: &mut Env,
        span: &Span,
        task_name: &str,
        context: ExpressionContext,
    ) -> Result<Evaluated, String> {
        let callee_name = canonical_identifier(callee)
            .ok_or_else(|| "checked call callee is not a direct identifier".to_string())?;
        if callee_name == "stdout_write" {
            return self.eval_stdout_write(arguments, env, span, task_name);
        }
        if callee_name == "clock_replay_tick" {
            return self.eval_clock_replay_tick(arguments, span, task_name);
        }
        if callee_name == "files_read_text" {
            return self.eval_files_read_text(arguments, env, span, task_name);
        }
        if return_dependency::is_closed_view_deriving_operation(callee_name) {
            return self.eval_slice_until(arguments, env, span, task_name);
        }
        if callee_name == "list_append" {
            return self.eval_list_append(arguments, env, span, task_name);
        }
        if callee_name == "old" {
            if context != ExpressionContext::Contract {
                return Err(
                    "old(...) is available only in `ensures:` over parameters or parameter fields readable when the task starts"
                        .to_string(),
                );
            }
            if arguments.len() != 1 {
                return Err("old expects one place".to_string());
            }
            let key = format!("old({})", crate::core_expr::canonical_text(&arguments[0]));
            return env
                .get(&key)
                .map(|binding| Evaluated::Value(binding.value.clone()))
                .ok_or_else(|| {
                    format!(
                        "`{key}` was not captured at task entry; old(...) is available only in `ensures:` over parameters or parameter fields readable when the task starts"
                    )
                });
        }
        if callee_name == "list_len" {
            return self.eval_list_len(arguments, env, span, task_name);
        }
        if callee_name == "list_count" {
            if context != ExpressionContext::Contract {
                return Err("list_count is contract-only Predicate v2 vocabulary".to_string());
            }
            let [list, text] = arguments else {
                return Err("list_count expects two arguments".to_string());
            };
            let list = match self.eval_canonical_with_context(
                list,
                env,
                task_name,
                span,
                ExpressionContext::Contract,
            )? {
                Evaluated::Value(value) => value,
                other => return Ok(other),
            };
            let text = match self.eval_canonical_with_context(
                text,
                env,
                task_name,
                span,
                ExpressionContext::Contract,
            )? {
                Evaluated::Value(value) => value,
                other => return Ok(other),
            };
            let Value::List(values) = list else {
                return Err("typed list_count fact evaluated a non-list".to_string());
            };
            let Value::Text(needle) = text else {
                return Err("typed list_count fact evaluated a non-Text match".to_string());
            };
            return Ok(Evaluated::Value(Value::Int(
                values
                    .iter()
                    .filter(|value| matches!(value, Value::Text(item) if item == &needle))
                    .count() as i64,
            )));
        }

        let active_callable_application =
            self.active_callable_applications.borrow().last().cloned();
        if let Some(current_task) = self.current_task()
            && let Some(application_id) = active_callable_application.as_deref()
            && let Some(application) = self.callable_analysis.indirect_application_with_id(
                current_task,
                span,
                application_id,
            )
        {
            let target_definition_id = match self
                .binding_by_definition_id(env, &application.callable_parameter_definition_id)
                .map(|binding| &binding.value)
            {
                Some(Value::Callable {
                    target_definition_id,
                }) if target_definition_id == &application.target_definition_id => {
                    target_definition_id.clone()
                }
                _ => {
                    return Err(
                        "runtime callable identity disagrees with checked application fact"
                            .to_string(),
                    );
                }
            };
            let value = self.read_value_by_definition_id(
                env,
                &application.ordinary_parameter_definition_id,
                span,
                task_name,
            )?;
            let target = self
                .task_by_definition_id(&target_definition_id)
                .ok_or_else(|| "checked callable target definition is unavailable".to_string())?;
            return task_result_to_evaluated(self.execute_task(target, vec![value])?);
        }
        if let Some(current_task) = self.current_task()
            && let Some(application) = self
                .callable_analysis
                .direct_application(current_task, span)
        {
            let value = match &application.ordinary_argument {
                callable::OrdinaryArgumentFact::UIntLiteral(value) => {
                    Value::Int(i64::try_from(*value).map_err(|_| {
                        "checked UInt callable argument exceeds runtime Int".to_string()
                    })?)
                }
                callable::OrdinaryArgumentFact::Definition { definition_id, .. } => {
                    self.read_value_by_definition_id(env, definition_id, span, task_name)?
                }
            };
            let receiver = self
                .task_by_definition_id(&application.receiver_definition_id)
                .ok_or_else(|| "checked callable receiver definition is unavailable".to_string())?;
            let callable = Value::Callable {
                target_definition_id: application.target_definition_id.clone(),
            };
            self.active_callable_applications
                .borrow_mut()
                .push(application.id.clone());
            let result = self.execute_task(receiver, vec![callable, value]);
            self.active_callable_applications.borrow_mut().pop();
            return task_result_to_evaluated(result?);
        }

        let Some(task) = self.find_task(callee_name) else {
            return Err(format!("task `{callee_name}` was not found"));
        };
        if arguments.len() != task.params.len() {
            return Err(format!(
                "task `{}` expects {} argument(s), got {}",
                task.name,
                task.params.len(),
                arguments.len()
            ));
        }
        let mut values = Vec::new();
        for argument in arguments {
            if let CanonicalExpressionKind::Permission {
                permission: ParamPermission::Consume,
                value,
            } = &argument.kind
            {
                let root = crate::core_expr::canonical_text(value);
                let value = self.read_consume_value(env, &root, span, task_name)?;
                self.mark_moved(env, &root, span, &task.name);
                values.push(value);
                continue;
            }
            match self.eval_canonical(argument, env, task_name)? {
                Evaluated::Value(value) => values.push(value),
                other => return Ok(other),
            }
        }
        let route_call_span = self.next_output_route_call_span(task_name, &task.name, span);
        if let Some(call_span) = &route_call_span {
            self.active_call_route.borrow_mut().push(call_span.clone());
        }
        let result = self.execute_task(task, values);
        if route_call_span.is_some() {
            self.active_call_route.borrow_mut().pop();
        }
        task_result_to_evaluated(result?)
    }

    fn eval_stdout_write(
        &self,
        args: &[CanonicalExpression],
        env: &mut Env,
        statement_span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        if args.len() != 1 {
            return Err(format!(
                "stdout_write expects exactly 1 Text argument, got {}",
                args.len()
            ));
        }
        let value = match self.eval_canonical(&args[0], env, task_name)? {
            Evaluated::Value(value) => value,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let Value::Text(text) = value else {
            return Err("stdout_write expects a Text argument".to_string());
        };
        let policy = self.next_output_policy(task_name, statement_span)?;
        let decision = self.grant_policy.stdout_write_decision();
        let request_id = self.record_output_decision(&policy, decision, text.len());

        if decision != GrantDecision::Allowed {
            self.record_output_exercise(
                &request_id,
                &policy,
                decision,
                false,
                text.len(),
                "denied_before_adapter_v0",
            );
            return Ok(Evaluated::Failure(output_failure(
                "denied",
                policy.call_span,
            )));
        }

        let mut output = self.output.borrow_mut();
        let Some(next_total) = output.successful_bytes.checked_add(text.len()) else {
            drop(output);
            self.record_output_exercise(
                &request_id,
                &policy,
                decision,
                false,
                text.len(),
                "limit_rejected_before_adapter_v0",
            );
            return Ok(Evaluated::Failure(output_failure(
                "limit_exceeded",
                policy.call_span,
            )));
        };
        if next_total > OUTPUT_LIMIT_BYTES {
            drop(output);
            self.record_output_exercise(
                &request_id,
                &policy,
                decision,
                false,
                text.len(),
                "limit_rejected_before_adapter_v0",
            );
            return Ok(Evaluated::Failure(output_failure(
                "limit_exceeded",
                policy.call_span,
            )));
        }
        let write_result = output.adapter.write(text.as_bytes());
        if write_result.is_ok() {
            output.successful_bytes = next_total;
        }
        drop(output);
        match write_result {
            Ok(()) => {
                self.record_output_exercise(
                    &request_id,
                    &policy,
                    decision,
                    true,
                    text.len(),
                    "write_succeeded_v0",
                );
                Ok(Evaluated::Value(Value::Unit))
            }
            Err(OutputAdapterError) => {
                self.record_output_exercise(
                    &request_id,
                    &policy,
                    decision,
                    true,
                    text.len(),
                    "write_failed_v0",
                );
                Ok(Evaluated::Failure(output_failure(
                    "write_failed",
                    policy.call_span,
                )))
            }
        }
    }

    fn eval_clock_replay_tick(
        &self,
        args: &[CanonicalExpression],
        statement_span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        if !args.is_empty() {
            return Err("clock_replay_tick expects no arguments".to_string());
        }
        let policy = self.next_replay_policy(task_name, statement_span)?;
        let decision = self.grant_policy.clock_replay_decision();
        let replay_index = self.replay.borrow().consumed;
        let request_id = self.record_replay_decision(&policy, decision, replay_index);

        if decision != GrantDecision::Allowed {
            self.record_replay_exercise(
                &request_id,
                &policy,
                decision,
                false,
                replay_index,
                None,
                "denied_before_adapter_v0",
            );
            return Ok(Evaluated::Failure(replay_failure(
                "denied",
                policy.call_span,
            )));
        }

        let tick = self.replay.borrow_mut().adapter.next_tick();
        match tick {
            Some(tick) => {
                self.replay.borrow_mut().consumed += 1;
                self.record_replay_exercise(
                    &request_id,
                    &policy,
                    decision,
                    true,
                    replay_index,
                    Some(tick),
                    "tick_consumed_v0",
                );
                Ok(Evaluated::Value(Value::Int(tick)))
            }
            None => {
                self.record_replay_exercise(
                    &request_id,
                    &policy,
                    decision,
                    true,
                    replay_index,
                    None,
                    "sequence_exhausted_v0",
                );
                Ok(Evaluated::Failure(replay_failure(
                    "exhausted",
                    policy.call_span,
                )))
            }
        }
    }

    fn eval_files_read_text(
        &self,
        args: &[CanonicalExpression],
        env: &mut Env,
        statement_span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        if args.len() != 1 {
            return Err(format!(
                "files_read_text expects exactly 1 Path argument, got {}",
                args.len()
            ));
        }
        let value = match self.eval_canonical(&args[0], env, task_name)? {
            Evaluated::Value(value) => value,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let Value::Path(path) = value else {
            return Err("files_read_text expects an opaque Path argument".to_string());
        };

        let policy = self.next_file_policy(task_name, statement_span)?;
        let decision = self.grant_policy.files_read_decision();
        let grant_matches = self
            .grant_policy
            .files_read_grant()
            .is_some_and(|grant| grant == &path);
        let request_id = self.record_file_decision(&policy, decision, &path, grant_matches);

        if decision != GrantDecision::Allowed {
            self.record_file_exercise(
                &request_id,
                &policy,
                decision,
                &path,
                grant_matches,
                false,
                0,
                "denied_before_candidate_access_v0",
            );
            return Ok(Evaluated::Failure(file_failure("denied", policy.call_span)));
        }
        if !grant_matches {
            self.record_file_exercise(
                &request_id,
                &policy,
                decision,
                &path,
                false,
                false,
                0,
                "outside_exact_native_grant_before_candidate_access_v0",
            );
            return Ok(Evaluated::Failure(file_failure(
                "outside_grant",
                policy.call_span,
            )));
        }

        let revalidated = match self.file_locality.borrow_mut().revalidate(&path) {
            Ok(path) => path,
            Err(error) => {
                let (variant, reason) = match error {
                    FileLocalityError::Unavailable => (
                        "unavailable",
                        "unsupported_or_unclassified_host_before_candidate_access_v0",
                    ),
                    FileLocalityError::UnsafePath => (
                        "unsafe_path",
                        "lexical_revalidation_failed_before_candidate_access_v0",
                    ),
                };
                self.record_file_exercise(
                    &request_id,
                    &policy,
                    decision,
                    &path,
                    true,
                    false,
                    0,
                    reason,
                );
                return Ok(Evaluated::Failure(file_failure(variant, policy.call_span)));
            }
        };
        if !revalidated.is_fixed_local() {
            self.record_file_exercise(
                &request_id,
                &policy,
                decision,
                &revalidated,
                true,
                false,
                0,
                "fixed_local_v0_not_proven_before_candidate_access_v0",
            );
            return Ok(Evaluated::Failure(file_failure(
                "unavailable",
                policy.call_span,
            )));
        }

        let result = self
            .file_adapter
            .borrow_mut()
            .read_text(revalidated.as_os_str());
        match result {
            Ok(text) => {
                self.record_file_exercise(
                    &request_id,
                    &policy,
                    decision,
                    &revalidated,
                    true,
                    true,
                    text.len(),
                    "exact_utf8_read_succeeded_v0",
                );
                Ok(Evaluated::Value(Value::Text(text)))
            }
            Err(error) => {
                self.record_file_exercise(
                    &request_id,
                    &policy,
                    decision,
                    &revalidated,
                    true,
                    true,
                    0,
                    error.result_reason(),
                );
                Ok(Evaluated::Failure(file_failure(
                    error.variant(),
                    policy.call_span,
                )))
            }
        }
    }

    fn next_file_policy(
        &self,
        task_name: &str,
        statement_span: &Span,
    ) -> Result<FilePolicyFact, String> {
        let key = (task_name.to_string(), statement_span.line);
        let policies = self.file_policies.get(&key).ok_or_else(|| {
            format!(
                "{}: files_read_text has no checked source-policy fact",
                location(statement_span)
            )
        })?;
        let mut expected_route = self
            .active_app
            .borrow()
            .map(|app| vec![app.name.clone()])
            .unwrap_or_default();
        expected_route.extend(self.active_task_route.borrow().iter().cloned());
        let active_call_route = self.active_call_route.borrow().clone();
        let matching = policies
            .iter()
            .filter(|policy| {
                policy.source_route == expected_route
                    && policy.source_route_spans.len() == active_call_route.len() + 1
                    && same_span_identities(
                        &policy.source_route_spans[..active_call_route.len()],
                        &active_call_route,
                    )
                    && policy.source_route_spans.last().is_some_and(|span| {
                        same_source_file(&span.file, &statement_span.file)
                            && span.line == statement_span.line
                    })
            })
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Err(format!(
                "{}: files_read_text has no checked source-policy fact for route `{}`",
                location(statement_span),
                expected_route.join(" -> ")
            ));
        }
        let route_key = active_call_route
            .iter()
            .map(span_identity_key)
            .collect::<Vec<_>>()
            .join("->");
        let cursor_key = (
            task_name.to_string(),
            statement_span.line,
            format!("{}|{route_key}", expected_route.join("->")),
        );
        let mut cursors = self.file_call_cursors.borrow_mut();
        let cursor = cursors.entry(cursor_key).or_default();
        let policy = matching[*cursor % matching.len()].clone();
        *cursor += 1;
        Ok(policy.clone())
    }

    fn next_replay_policy(
        &self,
        task_name: &str,
        statement_span: &Span,
    ) -> Result<ReplayPolicyFact, String> {
        let key = (task_name.to_string(), statement_span.line);
        let policies = self.replay_policies.get(&key).ok_or_else(|| {
            format!(
                "{}: clock_replay_tick has no checked source-policy fact",
                location(statement_span)
            )
        })?;
        let mut expected_route = self
            .active_app
            .borrow()
            .map(|app| vec![app.name.clone()])
            .unwrap_or_default();
        expected_route.extend(self.active_task_route.borrow().iter().cloned());
        let active_call_route = self.active_call_route.borrow().clone();
        let matching = policies
            .iter()
            .filter(|policy| {
                policy.source_route == expected_route
                    && policy.source_route_spans.len() == active_call_route.len() + 1
                    && same_span_identities(
                        &policy.source_route_spans[..active_call_route.len()],
                        &active_call_route,
                    )
                    && policy.source_route_spans.last().is_some_and(|span| {
                        same_source_file(&span.file, &statement_span.file)
                            && span.line == statement_span.line
                    })
            })
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Err(format!(
                "{}: clock_replay_tick has no checked source-policy fact for route `{}`",
                location(statement_span),
                expected_route.join(" -> ")
            ));
        }
        let route_key = active_call_route
            .iter()
            .map(span_identity_key)
            .collect::<Vec<_>>()
            .join("->");
        let cursor_key = (
            task_name.to_string(),
            statement_span.line,
            format!("{}|{route_key}", expected_route.join("->")),
        );
        let mut cursors = self.replay_call_cursors.borrow_mut();
        let cursor = cursors.entry(cursor_key).or_default();
        let policy = matching[*cursor % matching.len()].clone();
        *cursor += 1;
        Ok(policy.clone())
    }

    fn next_output_policy(
        &self,
        task_name: &str,
        statement_span: &Span,
    ) -> Result<OutputPolicyFact, String> {
        let key = (task_name.to_string(), statement_span.line);
        let policies = self.output_policies.get(&key).ok_or_else(|| {
            format!(
                "{}: stdout_write has no checked source-policy fact",
                location(statement_span)
            )
        })?;
        let mut expected_route = self
            .active_app
            .borrow()
            .map(|app| vec![app.name.clone()])
            .unwrap_or_default();
        expected_route.extend(self.active_task_route.borrow().iter().cloned());
        let active_call_route = self.active_call_route.borrow().clone();
        let matching = policies
            .iter()
            .filter(|policy| {
                policy.source_route == expected_route
                    && policy.source_route_spans.len() == active_call_route.len() + 1
                    && same_span_identities(
                        &policy.source_route_spans[..active_call_route.len()],
                        &active_call_route,
                    )
                    && policy.source_route_spans.last().is_some_and(|span| {
                        same_source_file(&span.file, &statement_span.file)
                            && span.line == statement_span.line
                    })
            })
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Err(format!(
                "{}: stdout_write has no checked source-policy fact for route `{}`",
                location(statement_span),
                expected_route.join(" -> ")
            ));
        }
        let route_key = active_call_route
            .iter()
            .map(span_identity_key)
            .collect::<Vec<_>>()
            .join("->");
        let cursor_key = (
            task_name.to_string(),
            statement_span.line,
            format!("{}|{route_key}", expected_route.join("->")),
        );
        let mut cursors = self.output_call_cursors.borrow_mut();
        let cursor = cursors.entry(cursor_key).or_default();
        let policy = matching[*cursor % matching.len()].clone();
        *cursor += 1;
        Ok(policy.clone())
    }

    fn next_output_route_call_span(
        &self,
        caller: &str,
        callee: &str,
        statement_span: &Span,
    ) -> Option<Span> {
        let mut expected_prefix = (*self.active_app.borrow())
            .map(|app| vec![app.name.clone()])
            .unwrap_or_default();
        expected_prefix.extend(self.active_task_route.borrow().iter().cloned());
        expected_prefix.push(callee.to_string());
        let active_call_route = self.active_call_route.borrow().clone();
        let call_index = active_call_route.len();
        let mut candidates = self
            .output_policies
            .values()
            .flatten()
            .filter(|policy| {
                policy.source_route.starts_with(&expected_prefix)
                    && policy.source_route_spans.len() > call_index
                    && same_span_identities(
                        &policy.source_route_spans[..call_index],
                        &active_call_route,
                    )
                    && same_source_file(
                        &policy.source_route_spans[call_index].file,
                        &statement_span.file,
                    )
                    && policy.source_route_spans[call_index].line == statement_span.line
            })
            .map(|policy| policy.source_route_spans[call_index].clone())
            .collect::<Vec<_>>();
        candidates.extend(
            self.replay_policies
                .values()
                .flatten()
                .filter(|policy| {
                    policy.source_route.starts_with(&expected_prefix)
                        && policy.source_route_spans.len() > call_index
                        && same_span_identities(
                            &policy.source_route_spans[..call_index],
                            &active_call_route,
                        )
                        && same_source_file(
                            &policy.source_route_spans[call_index].file,
                            &statement_span.file,
                        )
                        && policy.source_route_spans[call_index].line == statement_span.line
                })
                .map(|policy| policy.source_route_spans[call_index].clone()),
        );
        candidates.extend(
            self.file_policies
                .values()
                .flatten()
                .filter(|policy| {
                    policy.source_route.starts_with(&expected_prefix)
                        && policy.source_route_spans.len() > call_index
                        && same_span_identities(
                            &policy.source_route_spans[..call_index],
                            &active_call_route,
                        )
                        && same_source_file(
                            &policy.source_route_spans[call_index].file,
                            &statement_span.file,
                        )
                        && policy.source_route_spans[call_index].line == statement_span.line
                })
                .map(|policy| policy.source_route_spans[call_index].clone()),
        );
        candidates.sort_by_key(span_identity_key);
        candidates.dedup_by(|left, right| same_span_identity(left, right));
        if candidates.is_empty() {
            return None;
        }
        let route_key = active_call_route
            .iter()
            .map(span_identity_key)
            .collect::<Vec<_>>()
            .join("->");
        let key = (
            caller.to_string(),
            statement_span.line,
            callee.to_string(),
            route_key,
        );
        let mut cursors = self.output_task_call_cursors.borrow_mut();
        let cursor = cursors.entry(key).or_default();
        let span = candidates[*cursor % candidates.len()].clone();
        *cursor += 1;
        Some(span)
    }

    fn record_output_decision(
        &self,
        policy: &OutputPolicyFact,
        decision: GrantDecision,
        byte_count: usize,
    ) -> String {
        let mut events = self.authority_events.borrow_mut();
        let ordinal = events
            .iter()
            .filter(|event| event.event_kind == "operator_decision")
            .count()
            + 1;
        let request_id = format!("{}:request-{ordinal}", policy.policy_id);
        let event_sequence = events.len() + 1;
        events.push(output_audit_event(
            format!("{request_id}:decision"),
            event_sequence,
            request_id.clone(),
            policy,
            "operator_decision",
            decision,
            self.grant_policy.allows_stdout_write(),
            self.grant_policy.denies_stdout_write(),
            false,
            byte_count,
            "decision_recorded_v0",
        ));
        request_id
    }

    #[allow(clippy::too_many_arguments)]
    fn record_output_exercise(
        &self,
        request_id: &str,
        policy: &OutputPolicyFact,
        decision: GrantDecision,
        adapter_called: bool,
        byte_count: usize,
        result: &'static str,
    ) {
        let mut events = self.authority_events.borrow_mut();
        let event_sequence = events.len() + 1;
        events.push(output_audit_event(
            format!("{request_id}:exercise"),
            event_sequence,
            request_id.to_string(),
            policy,
            "operation_exercise",
            decision,
            self.grant_policy.allows_stdout_write(),
            self.grant_policy.denies_stdout_write(),
            adapter_called,
            byte_count,
            result,
        ));
    }

    fn record_replay_decision(
        &self,
        policy: &ReplayPolicyFact,
        decision: GrantDecision,
        replay_index: usize,
    ) -> String {
        let mut events = self.authority_events.borrow_mut();
        let ordinal = events
            .iter()
            .filter(|event| {
                event.event_kind == "operator_decision" && event.capability_id == "clock.replay"
            })
            .count()
            + 1;
        let request_id = format!("{}:request-{ordinal}", policy.policy_id);
        let event_sequence = events.len() + 1;
        events.push(replay_audit_event(
            format!("{request_id}:decision"),
            event_sequence,
            request_id.clone(),
            policy,
            "operator_decision",
            decision,
            self.grant_policy.allows_clock_replay(),
            self.grant_policy.denies_clock_replay(),
            false,
            replay_index,
            None,
            "decision_recorded_v0",
        ));
        request_id
    }

    #[allow(clippy::too_many_arguments)]
    fn record_replay_exercise(
        &self,
        request_id: &str,
        policy: &ReplayPolicyFact,
        decision: GrantDecision,
        adapter_called: bool,
        replay_index: usize,
        replay_tick: Option<i64>,
        result: &'static str,
    ) {
        let mut events = self.authority_events.borrow_mut();
        let event_sequence = events.len() + 1;
        events.push(replay_audit_event(
            format!("{request_id}:exercise"),
            event_sequence,
            request_id.to_string(),
            policy,
            "operation_exercise",
            decision,
            self.grant_policy.allows_clock_replay(),
            self.grant_policy.denies_clock_replay(),
            adapter_called,
            replay_index,
            replay_tick,
            result,
        ));
    }

    fn record_file_decision(
        &self,
        policy: &FilePolicyFact,
        decision: GrantDecision,
        path: &ValidatedNativePath,
        native_path_matched: bool,
    ) -> String {
        let mut events = self.authority_events.borrow_mut();
        let ordinal = events
            .iter()
            .filter(|event| {
                event.event_kind == "operator_decision" && event.capability_id == "files.read"
            })
            .count()
            + 1;
        let request_id = format!("{}:request-{ordinal}", policy.policy_id);
        let event_sequence = events.len() + 1;
        events.push(file_audit_event(
            format!("{request_id}:decision"),
            event_sequence,
            request_id.clone(),
            policy,
            "operator_decision",
            decision,
            self.grant_policy.allows_files_read(),
            self.grant_policy.denies_files_read(),
            false,
            0,
            path.as_os_str().to_os_string(),
            native_path_matched,
            path.locality(),
            "decision_recorded_v0",
        ));
        request_id
    }

    #[allow(clippy::too_many_arguments)]
    fn record_file_exercise(
        &self,
        request_id: &str,
        policy: &FilePolicyFact,
        decision: GrantDecision,
        path: &ValidatedNativePath,
        native_path_matched: bool,
        adapter_called: bool,
        byte_count: usize,
        result: &'static str,
    ) {
        let mut events = self.authority_events.borrow_mut();
        let event_sequence = events.len() + 1;
        events.push(file_audit_event(
            format!("{request_id}:exercise"),
            event_sequence,
            request_id.to_string(),
            policy,
            "operation_exercise",
            decision,
            self.grant_policy.allows_files_read(),
            self.grant_policy.denies_files_read(),
            adapter_called,
            byte_count,
            path.as_os_str().to_os_string(),
            native_path_matched,
            path.locality(),
            result,
        ));
    }

    fn eval_list_append(
        &self,
        args: &[CanonicalExpression],
        env: &mut Env,
        span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        if args.len() != 2 {
            return Err(format!(
                "list_append expects 2 argument(s), got {}",
                args.len()
            ));
        }
        let CanonicalExpressionKind::Permission {
            permission: ParamPermission::Change,
            value: list_value,
        } = &args[0].kind
        else {
            return Err("list_append first argument must be `change list`".to_string());
        };
        let list_place = RuntimePlace::from_canonical(list_value)
            .ok_or_else(|| "list_append first argument must be a canonical place".to_string())?;
        let root = list_place.root().to_string();
        if let Some(loop_span) = self.active_iteration_for(&root) {
            return Err(self.iteration_mutation_trap(task_name, &root, span, &loop_span));
        }
        self.ensure_can_set(env, &list_place, span, task_name)?;
        let item = match self.eval_canonical(&args[1], env, task_name)? {
            Evaluated::Value(value) => value,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let Some(binding) = env.get_mut(&root) else {
            return Err(format!("cannot append to unknown list `{root}`"));
        };
        let Value::List(values) = &mut binding.value else {
            return Err(format!("`{root}` is not a list"));
        };
        values.push(item);
        binding.moved_at = None;
        binding.moved_by = None;
        invalidate_element_views_for_growth(env, &root, span);
        Ok(Evaluated::Value(Value::Unit))
    }

    fn eval_slice_until(
        &self,
        args: &[CanonicalExpression],
        env: &mut Env,
        _span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        if args.len() != 2 {
            return Err(format!(
                "slice_until expects 2 argument(s), got {}",
                args.len()
            ));
        }
        let source_expr = permission_value(&args[0]);
        let separator_expr = permission_value(&args[1]);
        let source = match self.eval_canonical(source_expr, env, task_name)? {
            Evaluated::Value(value) => as_text(&value)?.to_string(),
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let separator = match self.eval_canonical(separator_expr, env, task_name)? {
            Evaluated::Value(value) => as_text(&value)?.to_string(),
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let prefix = if separator.is_empty() {
            ""
        } else {
            source
                .split_once(separator.as_str())
                .map_or(source.as_str(), |(head, _tail)| head)
        };
        Ok(Evaluated::Value(Value::Text(prefix.to_string())))
    }

    fn eval_list_len(
        &self,
        args: &[CanonicalExpression],
        env: &mut Env,
        _span: &Span,
        task_name: &str,
    ) -> Result<Evaluated, String> {
        if args.len() != 1 {
            return Err(format!(
                "list_len expects 1 argument(s), got {}",
                args.len()
            ));
        }
        let list_expr = permission_value(&args[0]);
        let value = match self.eval_canonical(list_expr, env, task_name)? {
            Evaluated::Value(value) => value,
            Evaluated::Failure(value) => return Ok(Evaluated::Failure(value)),
            Evaluated::ContractViolation => return Ok(Evaluated::ContractViolation),
        };
        let Value::List(values) = value else {
            return Err(format!(
                "list_len expects a list, got {}",
                crate::core_expr::canonical_text(list_expr)
            ));
        };
        Ok(Evaluated::Value(Value::Int(values.len() as i64)))
    }

    fn push_active_iteration(&self, root: String, span: Span) {
        self.active_iterations
            .borrow_mut()
            .push(ActiveIteration { root, span });
    }

    fn pop_active_iteration(&self) {
        self.active_iterations.borrow_mut().pop();
    }

    fn active_iteration_for(&self, root: &str) -> Option<Span> {
        self.active_iterations
            .borrow()
            .iter()
            .rev()
            .find(|iteration| iteration.root == root)
            .map(|iteration| iteration.span.clone())
    }

    fn read_value(
        &self,
        env: &Env,
        name: &str,
        span: &Span,
        task_name: &str,
    ) -> Result<Value, String> {
        let root = name;
        let Some(binding) = env.get(root) else {
            return Err(format!("unknown expression `{name}`"));
        };
        if let Some(move_span) = &binding.moved_at {
            return Err(use_after_move_invariant(task_name, root, span, move_span));
        }
        if let Some(view) = &binding.view
            && let Some(invalidation) = &view.invalidated_by
        {
            return Err(self.stale_view_trap(task_name, root, view, span, invalidation));
        }
        if let Some(source_place) = binding.writable_alias_source.clone() {
            return self.read_direct_field_value(env, &source_place, span, task_name);
        }
        Ok(binding.value.clone())
    }

    fn binding_by_definition_id<'env>(
        &self,
        env: &'env Env,
        definition_id: &str,
    ) -> Option<&'env RuntimeBinding> {
        env.values()
            .find(|binding| binding.definition_id.as_deref() == Some(definition_id))
    }

    fn read_value_by_definition_id(
        &self,
        env: &Env,
        definition_id: &str,
        span: &Span,
        task_name: &str,
    ) -> Result<Value, String> {
        let (name, _binding) = env
            .iter()
            .find(|(_name, binding)| binding.definition_id.as_deref() == Some(definition_id))
            .ok_or_else(|| "checked runtime binding definition is unavailable".to_string())?;
        self.read_value(env, name, span, task_name)
    }

    fn read_direct_field_value(
        &self,
        env: &Env,
        place: &RuntimePlace,
        span: &Span,
        task_name: &str,
    ) -> Result<Value, String> {
        let RuntimePlace::Field { root, field } = place else {
            return Err(format!(
                "unsupported writable alias source `{}`",
                place.display()
            ));
        };
        let value = self.read_value(env, root, span, task_name)?;
        let Value::Record(fields) = value else {
            return Err(format!("`{root}` is not a record"));
        };
        fields
            .get(field)
            .cloned()
            .ok_or_else(|| format!("record `{root}` has no field `{field}`"))
    }

    fn resolve_writable_alias_place(
        &self,
        env: &Env,
        place: &RuntimePlace,
    ) -> Result<RuntimePlace, String> {
        let root = place.root();
        let Some(binding) = env.get(root) else {
            return Ok(place.clone());
        };
        let Some(source_place) = &binding.writable_alias_source else {
            return Ok(place.clone());
        };
        if !matches!(place, RuntimePlace::Root(_)) {
            return Err(format!(
                "writable alias `{root}` supports only direct local reads and writes"
            ));
        }
        Ok(source_place.clone())
    }

    fn read_consume_value(
        &self,
        env: &Env,
        name: &str,
        span: &Span,
        task_name: &str,
    ) -> Result<Value, String> {
        let root = name;
        let Some(binding) = env.get(root) else {
            return Err(format!("unknown expression `{name}`"));
        };
        if let Some(move_span) = &binding.moved_at {
            if binding.linear {
                return Err(self.linear_double_consume_trap(
                    task_name,
                    root,
                    span,
                    move_span,
                    binding.moved_by.as_deref().unwrap_or("consume"),
                ));
            }
            return Err(use_after_move_invariant(task_name, root, span, move_span));
        }
        Ok(binding.value.clone())
    }

    fn ensure_can_set(
        &self,
        env: &Env,
        place: &RuntimePlace,
        span: &Span,
        task_name: &str,
    ) -> Result<(), String> {
        let root = place.root();
        let Some(binding) = env.get(root) else {
            return Err(format!("cannot set unknown place `{root}`"));
        };
        if let Some(move_span) = &binding.moved_at {
            return Err(use_after_move_invariant(task_name, root, span, move_span));
        }
        if binding.permission == RuntimePermission::Borrow {
            return Err(self.borrow_mutation_trap(task_name, &place.display(), root, span));
        }
        if !binding.writable {
            return Err(format!("cannot set immutable place `{root}`"));
        }
        Ok(())
    }

    fn write_place(&self, env: &mut Env, place: &RuntimePlace, value: Value) -> Result<(), String> {
        let root = place.root();
        let Some(binding) = env.get_mut(root) else {
            return Err(format!("cannot set unknown place `{}`", place.display()));
        };
        if let RuntimePlace::Field { field, .. } = place {
            let Value::Record(fields) = &mut binding.value else {
                return Err(format!("`{root}` is not a record"));
            };
            if !fields.contains_key(field) {
                return Err(format!("record `{root}` has no field `{field}`"));
            }
            fields.insert(field.to_string(), value);
        } else if matches!(place, RuntimePlace::Root(_)) {
            binding.value = value;
        } else {
            return Err(format!("unsupported set place `{}`", place.display()));
        }
        binding.moved_at = None;
        binding.moved_by = None;
        Ok(())
    }

    fn mark_moved(&self, env: &mut Env, root: &str, span: &Span, action: &str) {
        if let Some(binding) = env.get_mut(root)
            && matches!(
                binding.permission,
                RuntimePermission::Local | RuntimePermission::Consume
            )
        {
            binding.moved_at.get_or_insert_with(|| span.clone());
            if binding.moved_by.is_none() {
                binding.moved_by = Some(action.to_string());
            }
        }
    }

    fn ensure_linear_closed_on_exit(
        &self,
        env: &Env,
        task_name: &str,
        exit_kind: &str,
        span: &Span,
    ) -> Result<(), String> {
        if let Some((root, _binding)) = env
            .iter()
            .find(|(_root, binding)| binding.linear && binding.moved_at.is_none())
        {
            return Err(self.linear_not_consumed_trap(task_name, root, exit_kind, span));
        }
        Ok(())
    }

    fn borrow_mutation_trap(
        &self,
        task_name: &str,
        place: &str,
        root: &str,
        span: &Span,
    ) -> String {
        let message = if place == root {
            format!("borrowed parameter `{root}` cannot be written")
        } else {
            format!("borrowed parameter `{root}` cannot write `{place}`")
        };
        let diagnostic = Diagnostic::error(
                DiagnosticCode::BORROW_PARAMETER_MUTATION,
                message,
                Some(span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: mark `{root}` as `change`, copy it into a `change` local, or remove the `set`."
            ));
        self.diagnostics.borrow_mut().push(diagnostic);
        format!(
            "{} {}",
            DiagnosticCode::BORROW_PARAMETER_MUTATION.as_str(),
            DiagnosticCode::BORROW_PARAMETER_MUTATION.title()
        )
    }

    fn iteration_mutation_trap(
        &self,
        task_name: &str,
        root: &str,
        mutation_span: &Span,
        loop_span: &Span,
    ) -> String {
        self.diagnostics.borrow_mut().push(
            Diagnostic::error(
                DiagnosticCode::ITERATION_MUTATION_CONFLICT,
                format!("cannot structurally mutate `{root}` while it is being iterated"),
                Some(mutation_span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: `list_append` mutates `{root}` at {}:{}:{} during `for each` started at {}:{}:{}; collect changes after the loop or iterate over a separate list.",
                mutation_span.file,
                mutation_span.line,
                mutation_span.column,
                loop_span.file,
                loop_span.line,
                loop_span.column
            )),
        );
        format!(
            "{} {}",
            DiagnosticCode::ITERATION_MUTATION_CONFLICT.as_str(),
            DiagnosticCode::ITERATION_MUTATION_CONFLICT.title()
        )
    }

    fn stale_view_trap(
        &self,
        task_name: &str,
        view_name: &str,
        view: &RuntimeView,
        use_span: &Span,
        invalidation: &RuntimeViewInvalidation,
    ) -> String {
        let (message, help) = match (view.kind, invalidation.kind) {
            (RuntimeViewKind::Field, RuntimeViewInvalidationKind::FieldWrite) => (
                format!(
                    "field view {view_name} was used after {} changed",
                    view.source_place.display()
                ),
                format!(
                    "Fix task {task_name}: {view_name} borrowed {} at {}:{}:{}, but {} was written at {}:{}:{} before this use; re-borrow after the write or bind a value copy before the write.",
                    view.source_place.display(),
                    view.bound_at.file,
                    view.bound_at.line,
                    view.bound_at.column,
                    view.source_place.display(),
                    invalidation.span.file,
                    invalidation.span.line,
                    invalidation.span.column
                ),
            ),
            (RuntimeViewKind::Element, RuntimeViewInvalidationKind::ListAppend) => {
                let root = view.source_place.root();
                (
                    format!("element view {view_name} was used after {root} grew"),
                    format!(
                        "Fix task {task_name}: {view_name} borrowed {} at {}:{}:{}, but list_append grew {root} at {}:{}:{} before this use; re-borrow after the append or copy the element value before the append.",
                        view.source_place.display(),
                        view.bound_at.file,
                        view.bound_at.line,
                        view.bound_at.column,
                        invalidation.span.file,
                        invalidation.span.line,
                        invalidation.span.column
                    ),
                )
            }
            (RuntimeViewKind::Field, RuntimeViewInvalidationKind::ListAppend)
            | (RuntimeViewKind::Element, RuntimeViewInvalidationKind::FieldWrite) => (
                format!("view {view_name} was used after its source changed"),
                format!(
                    "Fix task {task_name}: {view_name} borrowed {} at {}:{}:{}, but the source changed at {}:{}:{} before this use; re-borrow after the change or copy the value first.",
                    view.source_place.display(),
                    view.bound_at.file,
                    view.bound_at.line,
                    view.bound_at.column,
                    invalidation.span.file,
                    invalidation.span.line,
                    invalidation.span.column
                ),
            ),
        };
        let diagnostic = Diagnostic::error(
            DiagnosticCode::STALE_FIELD_VIEW,
            message,
            Some(use_span.clone()),
        )
        .with_help(help);
        self.diagnostics.borrow_mut().push(diagnostic);
        format!(
            "{} {}",
            DiagnosticCode::STALE_FIELD_VIEW.as_str(),
            DiagnosticCode::STALE_FIELD_VIEW.title()
        )
    }
    fn return_dependency_trap(&self, task_name: &str, source: &str, span: &Span) -> String {
        let diagnostic = Diagnostic::error(
                DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER,
                format!("returned view does not visibly depend on parameter `{source}`"),
                Some(span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: returned-view `from` source `{source}` must name a task parameter, and returns must visibly return that parameter or a closed-set view derivation such as `slice_until(source, separator)`; locals, internal references, and non-closed derivation chains remain rejected."
            ));
        self.diagnostics.borrow_mut().push(diagnostic);
        format!(
            "{} {}",
            DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER.as_str(),
            DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER.title()
        )
    }

    fn linear_not_consumed_trap(
        &self,
        task_name: &str,
        root: &str,
        exit_kind: &str,
        span: &Span,
    ) -> String {
        let diagnostic = Diagnostic::error(
                DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED,
                format!("linear resource `{root}` reached {exit_kind} without being consumed"),
                Some(span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: consume `{root}` exactly once with commit, rollback, close, or transfer before leaving this path."
            ));
        self.diagnostics.borrow_mut().push(diagnostic);
        format!(
            "{} {}",
            DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED.as_str(),
            DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED.title()
        )
    }

    fn linear_double_consume_trap(
        &self,
        task_name: &str,
        root: &str,
        span: &Span,
        move_span: &Span,
        moved_by: &str,
    ) -> String {
        let diagnostic = Diagnostic::error(
                DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE,
                format!("linear resource `{root}` was consumed twice"),
                Some(span.clone()),
            )
            .with_help(format!(
                "Fix task `{task_name}`: `{root}` was already consumed by {moved_by} at {}:{}:{}; keep exactly one commit, rollback, close, or transfer on each path.",
                move_span.file, move_span.line, move_span.column
            ));
        self.diagnostics.borrow_mut().push(diagnostic);
        format!(
            "{} {}",
            DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE.as_str(),
            DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE.title()
        )
    }
}

impl ContractKind {
    fn section_name(self) -> &'static str {
        match self {
            ContractKind::Needs => "needs",
            ContractKind::Ensures => "ensures",
        }
    }
}

fn old_places(expression: &CanonicalExpression) -> Vec<CanonicalExpression> {
    fn collect(expression: &CanonicalExpression, out: &mut Vec<CanonicalExpression>) {
        match &expression.kind {
            CanonicalExpressionKind::Call { callee, arguments }
                if callee.direct_identifier() == Some("old") =>
            {
                if let [place] = arguments.as_slice()
                    && !out.iter().any(|found| found.node_id == place.node_id)
                {
                    out.push(place.clone());
                }
            }
            CanonicalExpressionKind::Call { arguments, .. } => {
                for argument in arguments {
                    collect(argument, out);
                }
            }
            CanonicalExpressionKind::Binary { left, right, .. } => {
                collect(left, out);
                collect(right, out);
            }
            CanonicalExpressionKind::Group(inner) => collect(inner, out),
            _ => {}
        }
    }
    let mut places = Vec::new();
    collect(expression, &mut places);
    places
}

fn executable_lines(section: &Section) -> Vec<ExecLine> {
    section
        .lines
        .iter()
        .zip(&section.body_syntax)
        .filter_map(|(line, parsed)| {
            let text = line.text.trim();
            is_meaningful_line_text(text).then(|| ExecLine {
                text: text.to_string(),
                location: format!("{}:{}:{}", line.span.file, line.span.line, line.span.column),
                span: line.span.clone(),
                parsed: parsed
                    .clone()
                    .expect("meaningful does line must retain parser-owned statement syntax"),
            })
        })
        .collect()
}

fn collect_tasks<'a>(items: &'a [Item], tasks: &mut Vec<&'a Task>) {
    for item in items {
        match item {
            Item::Task(task) => tasks.push(task),
            Item::App(app) => collect_tasks(&app.items, tasks),
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn find_task_in_items<'a>(items: &'a [Item], name: &str) -> Option<&'a Task> {
    for item in items {
        match item {
            Item::Task(task) if task.name == name => return Some(task),
            Item::App(app) => {
                if let Some(task) = find_task_in_items(&app.items, name) {
                    return Some(task);
                }
            }
            Item::Task(_) | Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
    None
}

fn parse_arg(ty: &str, raw: &OsStr, app_mode: bool) -> Result<Value, String> {
    if ty.trim() == "Path" {
        if !app_mode {
            return Err(
                "opaque Path arguments can be constructed only by structural app entry; direct `--entry` is forbidden"
                    .to_string(),
            );
        }
        return validate_native_path(raw).map(Value::Path).map_err(|issue| {
            format!(
                "structural app Path argument rejected because {}; reason={}; no host access was attempted",
                issue.description(),
                issue.reason()
            )
        });
    }
    let raw = raw.to_str().ok_or_else(|| {
        format!(
            "non-Unicode argument is valid only for an opaque Path parameter, not `{}`",
            ty.trim()
        )
    })?;
    match ty.trim() {
        "Int" => parse_int_literal(raw).map(Value::Int),
        "UInt" => {
            let value = parse_int_literal(raw)?;
            if value < 0 {
                Err(format!("UInt argument `{raw}` must not be negative"))
            } else {
                Ok(Value::Int(value))
            }
        }
        "Bool" => parse_bool(raw).map(Value::Bool),
        "Text" => Ok(Value::Text(raw.to_string())),
        ty if ty.starts_with("List ") => {
            let element_ty = ty.trim_start_matches("List ").trim();
            parse_list_arg(element_ty, raw)
        }
        other => parse_record_arg(other, raw),
    }
}

fn parse_list_arg(element_ty: &str, raw: &str) -> Result<Value, String> {
    let raw = raw.trim();
    let inside = raw
        .strip_prefix('[')
        .and_then(|text| text.strip_suffix(']'))
        .ok_or_else(|| format!("list argument `{raw}` must use `[a, b]` syntax"))?;
    let mut values = Vec::new();
    for item in split_arguments(inside) {
        values.push(parse_list_element(element_ty, item.trim())?);
    }
    Ok(Value::List(values))
}

fn parse_list_element(element_ty: &str, raw: &str) -> Result<Value, String> {
    match element_ty {
        "Bool" => parse_bool(raw).map(Value::Bool),
        "Int" | "UInt" | "Text" => parse_arg(element_ty, OsStr::new(raw), false),
        other => parse_record_arg(other, raw),
    }
}

fn parse_record_arg(ty: &str, raw: &str) -> Result<Value, String> {
    let raw = raw.trim();
    let inside = raw
        .strip_prefix('{')
        .and_then(|text| text.strip_suffix('}'))
        .ok_or_else(|| format!("record argument for `{ty}` must use `{{field: value}}` syntax"))?;

    let mut fields = BTreeMap::new();
    for field in split_arguments(inside) {
        let (name, value_text) = field
            .split_once(':')
            .ok_or_else(|| format!("record field `{field}` is missing `:`"))?;
        let name = name.trim();
        if !is_record_field_name(name) {
            return Err(format!(
                "record field name `{name}` is not a valid Hum field name"
            ));
        }
        fields.insert(
            name.to_string(),
            parse_cli_literal_value(value_text.trim())?,
        );
    }

    Ok(Value::Record(fields))
}

fn parse_cli_literal_value(raw: &str) -> Result<Value, String> {
    let raw = raw.trim();
    if raw == "true" || raw == "false" {
        return parse_bool(raw).map(Value::Bool);
    }
    if let Ok(value) = parse_int_literal(raw) {
        return Ok(Value::Int(value));
    }
    if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
        return Ok(Value::Text(raw[1..raw.len() - 1].to_string()));
    }
    if raw.starts_with('[') && raw.ends_with(']') {
        let inside = &raw[1..raw.len() - 1];
        let mut values = Vec::new();
        for item in split_arguments(inside) {
            values.push(parse_cli_literal_value(item)?);
        }
        return Ok(Value::List(values));
    }
    if raw.starts_with('{') && raw.ends_with('}') {
        return parse_record_arg("record", raw);
    }

    Err(format!("unsupported CLI literal `{raw}`"))
}

fn is_record_field_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first == '_')
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn parse_bool(raw: &str) -> Result<bool, String> {
    match raw.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("Bool argument `{other}` must be `true` or `false`")),
    }
}

fn parse_int_literal(text: &str) -> Result<i64, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("empty integer literal".to_string());
    }
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty() || !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(format!("`{text}` is not an integer literal"));
    }
    text.parse::<i64>()
        .map_err(|_| format!("integer literal `{text}` is outside Int range"))
}

fn as_int(value: &Value) -> Result<i64, String> {
    match value {
        Value::Int(value) => Ok(*value),
        other => Err(format!("expected Int value, got {}", value_kind(other))),
    }
}

fn as_bool(value: &Value) -> Result<bool, String> {
    match value {
        Value::Bool(value) => Ok(*value),
        other => Err(format!("expected Bool value, got {}", value_kind(other))),
    }
}

fn apply_binary_operator(
    operator: ParsedBinaryOperator,
    left: Value,
    right: Value,
    expression: &CanonicalExpression,
) -> Result<Value, String> {
    let source = crate::core_expr::canonical_text(expression);
    let value = match operator {
        ParsedBinaryOperator::Equal => Value::Bool(left == right),
        ParsedBinaryOperator::NotEqual => Value::Bool(left != right),
        ParsedBinaryOperator::Less => Value::Bool(as_int(&left)? < as_int(&right)?),
        ParsedBinaryOperator::LessEqual => Value::Bool(as_int(&left)? <= as_int(&right)?),
        ParsedBinaryOperator::Greater => Value::Bool(as_int(&left)? > as_int(&right)?),
        ParsedBinaryOperator::GreaterEqual => Value::Bool(as_int(&left)? >= as_int(&right)?),
        ParsedBinaryOperator::And => Value::Bool(as_bool(&left)? && as_bool(&right)?),
        ParsedBinaryOperator::Or => Value::Bool(as_bool(&left)? || as_bool(&right)?),
        ParsedBinaryOperator::Add
        | ParsedBinaryOperator::Subtract
        | ParsedBinaryOperator::Multiply
        | ParsedBinaryOperator::Divide => {
            let left = as_int(&left)?;
            let right = as_int(&right)?;
            let result = checked_integer_operator(operator, left, right).map_err(|reason| {
                if reason == "division_by_zero_v0" {
                    format!("division by zero while evaluating `{source}`")
                } else {
                    format!("integer overflow while evaluating `{source}`")
                }
            })?;
            Value::Int(result)
        }
        ParsedBinaryOperator::Is
        | ParsedBinaryOperator::Does
        | ParsedBinaryOperator::Returns
        | ParsedBinaryOperator::FailsWith => {
            return Err(format!(
                "unsupported executable operator `{}`",
                crate::core_expr::operator_spelling(operator)
            ));
        }
    };
    Ok(value)
}

fn checked_integer_operator(
    operator: ParsedBinaryOperator,
    left: i64,
    right: i64,
) -> Result<i64, &'static str> {
    match operator {
        ParsedBinaryOperator::Add => left.checked_add(right),
        ParsedBinaryOperator::Subtract => left.checked_sub(right),
        ParsedBinaryOperator::Multiply => left.checked_mul(right),
        ParsedBinaryOperator::Divide if right == 0 => return Err("division_by_zero_v0"),
        ParsedBinaryOperator::Divide => left.checked_div(right),
        _ => return Err("not_integer_operator_v0"),
    }
    .ok_or("integer_overflow_v0")
}

fn canonical_identifier(expression: &CanonicalExpression) -> Option<&str> {
    match &expression.kind {
        CanonicalExpressionKind::Identifier(name) => Some(name),
        _ => None,
    }
}

fn canonical_callee_name(expression: &CanonicalExpression) -> Option<&str> {
    match &expression.kind {
        CanonicalExpressionKind::Call { callee, .. } => canonical_identifier(callee),
        _ => None,
    }
}

fn permission_value(expression: &CanonicalExpression) -> &CanonicalExpression {
    match &expression.kind {
        CanonicalExpressionKind::Permission { value, .. } => value,
        _ => expression,
    }
}

fn statement_expression(statement: &ParsedBodyStatement) -> Option<&CanonicalExpression> {
    match &statement.kind {
        ParsedBodyStatementKind::Return(expression) => Some(&expression.canonical),
        ParsedBodyStatementKind::Binding { value, .. } => {
            value.as_ref().map(|expression| &expression.canonical)
        }
        ParsedBodyStatementKind::Other { expressions } => {
            expressions.first().map(|expression| &expression.canonical)
        }
    }
}

fn task_result_to_evaluated(result: TaskResult) -> Result<Evaluated, String> {
    Ok(match result {
        TaskResult::Returned(value) => Evaluated::Value(value),
        TaskResult::Failed(value) => Evaluated::Failure(value),
        TaskResult::ContractViolation => Evaluated::ContractViolation,
    })
}

fn as_text(value: &Value) -> Result<&str, String> {
    match value {
        Value::Text(value) => Ok(value),
        other => Err(format!("expected Text value, got {}", value_kind(other))),
    }
}

fn display_value(value: &Value) -> Result<String, String> {
    match value {
        Value::Unit => Ok("()".to_string()),
        Value::Int(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Text(value) => Ok(value.clone()),
        Value::Variant(value) => Ok(value.clone()),
        Value::Path(_) => Err("opaque Path values have no display surface".to_string()),
        Value::Callable { .. } => {
            Err("runtime callable handles have no display surface".to_string())
        }
        Value::List(values) => {
            let body = values
                .iter()
                .map(display_value)
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            Ok(format!("[{body}]"))
        }
        Value::Record(fields) => {
            let body = fields
                .iter()
                .map(|(name, value)| display_value(value).map(|value| format!("{name}: {value}")))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            Ok(format!("{{{body}}}"))
        }
    }
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Unit => "Unit",
        Value::Int(_) => "Int",
        Value::Bool(_) => "Bool",
        Value::Text(_) => "Text",
        Value::Record(_) => "record",
        Value::List(_) => "list",
        Value::Variant(_) => "variant",
        Value::Path(_) => "opaque Path",
        Value::Callable { .. } => "runtime callable handle",
    }
}

fn use_after_move_invariant(
    task_name: &str,
    root: &str,
    use_span: &Span,
    move_span: &Span,
) -> String {
    format!(
        "diagnostic invariant failure: ownership preflight allowed post-move access to `{root}` in task `{task_name}` at {}; move was recorded at {}",
        location(use_span),
        location(move_span),
    )
}

fn location(span: &Span) -> String {
    format!(
        "{}:{}:{}",
        span.file.replace('\\', "/"),
        span.line,
        span.column
    )
}

fn same_source_file(left: &str, right: &str) -> bool {
    crate::node_id::source_path_identity(left) == crate::node_id::source_path_identity(right)
}

fn same_span_identity(left: &Span, right: &Span) -> bool {
    same_source_file(&left.file, &right.file)
        && left.line == right.line
        && left.column == right.column
}

fn same_span_identities(left: &[Span], right: &[Span]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| same_span_identity(left, right))
}

fn span_identity_key(span: &Span) -> String {
    format!(
        "{}:{}:{}",
        crate::node_id::source_path_identity(&span.file),
        span.line,
        span.column
    )
}

fn invalidate_field_views(env: &mut Env, place: &RuntimePlace, span: &Span) {
    if !matches!(place, RuntimePlace::Field { .. }) {
        return;
    }
    for binding in env.values_mut() {
        if let Some(view) = &mut binding.view
            && view.kind == RuntimeViewKind::Field
            && &view.source_place == place
            && view.invalidated_by.is_none()
        {
            view.invalidated_by = Some(RuntimeViewInvalidation {
                span: span.clone(),
                kind: RuntimeViewInvalidationKind::FieldWrite,
            });
        }
    }
}

fn invalidate_element_views_for_growth(env: &mut Env, root: &str, span: &Span) {
    for binding in env.values_mut() {
        if let Some(view) = &mut binding.view
            && view.kind == RuntimeViewKind::Element
            && view.source_place.root() == root
            && view.invalidated_by.is_none()
        {
            view.invalidated_by = Some(RuntimeViewInvalidation {
                span: span.clone(),
                kind: RuntimeViewInvalidationKind::ListAppend,
            });
        }
    }
}

fn is_linear_binding(env: &Env, root: &str) -> bool {
    env.get(root).is_some_and(|binding| binding.linear)
}

fn is_linear_resource_type(type_text: &str) -> bool {
    type_tokens(type_text)
        .into_iter()
        .any(|token| token.eq_ignore_ascii_case("Transaction") || token.ends_with("Transaction"))
}

fn type_tokens(type_text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in type_text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn declaration_root(text: &str) -> String {
    text.split(|ch: char| ch == '.' || ch == '[' || ch.is_whitespace() || ch == ',')
        .find(|part| !part.is_empty())
        .unwrap_or(text)
        .trim()
        .to_string()
}

fn restore_binding(env: &mut Env, name: &str, previous: Option<RuntimeBinding>) {
    if let Some(binding) = previous {
        env.insert(name.to_string(), binding);
    } else {
        env.remove(name);
    }
}

fn matching_close(lines: &[ExecLine], open: usize) -> Result<usize, String> {
    let opening = &lines[open].parsed;
    if opening.block_relationship != ParsedBlockRelationship::Opens {
        return Err(format!(
            "{}: statement is not a parser-owned block opener",
            lines[open].location
        ));
    }
    for (index, line) in lines.iter().enumerate().skip(open + 1) {
        if line.parsed.block_relationship == ParsedBlockRelationship::Closes
            && line.parsed.block_depth_after == opening.block_depth_before
        {
            return Ok(index);
        }
    }
    Err(format!(
        "{}: block is missing closing `}}`",
        lines[open].location
    ))
}

fn split_arguments(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut depth = 0isize;
    let mut in_string = false;
    for (index, ch) in text.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' | '[' | '{' if !in_string => depth += 1,
            ')' | ']' | '}' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                let part = text[start..index].trim();
                if !part.is_empty() {
                    parts.push(part);
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    let part = text[start..].trim();
    if !part.is_empty() {
        parts.push(part);
    }
    parts
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};

    use crate::ast::Program;
    use crate::check;
    use crate::diagnostic::{
        DiagnosticCode, DiagnosticInvariantError, DiagnosticOccurrenceCollector,
        DiagnosticOccurrenceSet, Severity,
    };
    use crate::file_read::FileReadAdapterError;
    #[cfg(windows)]
    use crate::file_read::{FileLocalityAdapter, FileLocalityError, FileReadAdapter};
    #[cfg(windows)]
    use crate::native_path::ValidatedNativePath;
    use crate::operator_grant::OperatorGrantPolicy;
    use crate::parser;

    use super::{
        OUTPUT_LIMIT_BYTES, OutputAdapter, OutputAdapterError, ReplayAdapter, RunAdapters,
        RunOutcome, RuntimePlace, run_program, run_program_with_adapters,
        run_program_with_occurrences_and_adapters, run_program_with_occurrences_and_file_adapters,
        run_program_with_output, runtime_occurrence_authority,
    };
    #[cfg(windows)]
    use super::{RunReport, Value, parse_arg, run_program_with_file_adapters};

    #[derive(Default)]
    struct RecordingOutput {
        writes: Vec<Vec<u8>>,
        fail_on_call: Option<usize>,
        calls: usize,
    }

    impl OutputAdapter for RecordingOutput {
        fn write(&mut self, bytes: &[u8]) -> Result<(), OutputAdapterError> {
            let call = self.calls;
            self.calls += 1;
            if self.fail_on_call == Some(call) {
                return Err(OutputAdapterError);
            }
            self.writes.push(bytes.to_vec());
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingReplay {
        ticks: std::collections::VecDeque<i64>,
        calls: usize,
    }

    impl RecordingReplay {
        fn new(ticks: &[i64]) -> Self {
            Self {
                ticks: ticks.iter().copied().collect(),
                calls: 0,
            }
        }
    }

    impl ReplayAdapter for RecordingReplay {
        fn next_tick(&mut self) -> Option<i64> {
            self.calls += 1;
            self.ticks.pop_front()
        }
    }

    #[derive(Default)]
    struct CountingFileRead {
        calls: usize,
    }

    impl super::FileReadAdapter for CountingFileRead {
        fn read_text(&mut self, _path: &OsStr) -> Result<String, FileReadAdapterError> {
            self.calls += 1;
            Err(FileReadAdapterError::IoFailed)
        }
    }

    #[derive(Default)]
    struct CountingLocality {
        calls: usize,
    }

    impl super::FileLocalityAdapter for CountingLocality {
        fn revalidate(
            &mut self,
            _path: &crate::native_path::ValidatedNativePath,
        ) -> Result<crate::native_path::ValidatedNativePath, super::FileLocalityError> {
            self.calls += 1;
            Err(super::FileLocalityError::Unavailable)
        }
    }

    type CorruptedMovedStateProbe = Result<
        (
            Vec<(&'static str, String)>,
            Vec<crate::diagnostic::Diagnostic>,
        ),
        String,
    >;

    fn probe_corrupted_moved_state_branches<'output>(
        program: &crate::ast::Program,
        diagnostic_occurrences: &crate::diagnostic::DiagnosticOccurrenceSet,
        grant_policy: &'output OperatorGrantPolicy,
        adapters: RunAdapters<'output>,
    ) -> CorruptedMovedStateProbe {
        let collector = crate::diagnostic::DiagnosticOccurrenceCollector::from_authority(
            diagnostic_occurrences,
        )
        .map_err(|error| format!("diagnostic invariant failure: {error:?}"))?;
        let interpreter = super::Interpreter {
            program,
            callable_analysis: crate::callable::analyze_program(program),
            predicate_analysis: crate::predicate::analyze_program(program),
            diagnostics: std::cell::RefCell::new(Vec::new()),
            diagnostic_occurrences: std::cell::RefCell::new(collector),
            active_iterations: std::cell::RefCell::new(Vec::new()),
            active_app: std::cell::RefCell::new(None),
            grant_policy,
            output: std::cell::RefCell::new(super::OutputRuntime {
                adapter: adapters.output,
                successful_bytes: 0,
            }),
            replay: std::cell::RefCell::new(super::ReplayRuntime {
                adapter: adapters.replay,
                consumed: 0,
            }),
            file_adapter: std::cell::RefCell::new(adapters.file),
            file_locality: std::cell::RefCell::new(adapters.file_locality),
            output_policies: std::collections::BTreeMap::new(),
            replay_policies: std::collections::BTreeMap::new(),
            file_policies: std::collections::BTreeMap::new(),
            output_call_cursors: std::cell::RefCell::new(std::collections::BTreeMap::new()),
            replay_call_cursors: std::cell::RefCell::new(std::collections::BTreeMap::new()),
            file_call_cursors: std::cell::RefCell::new(std::collections::BTreeMap::new()),
            output_task_call_cursors: std::cell::RefCell::new(std::collections::BTreeMap::new()),
            active_task_route: std::cell::RefCell::new(Vec::new()),
            active_task_definition_ids: std::cell::RefCell::new(Vec::new()),
            active_call_route: std::cell::RefCell::new(Vec::new()),
            active_callable_applications: std::cell::RefCell::new(Vec::new()),
            authority_events: std::cell::RefCell::new(Vec::new()),
        };
        let move_span = crate::diagnostic::Span::new("corrupted-after-preflight.hum", 11, 5);
        let use_span = crate::diagnostic::Span::new("corrupted-after-preflight.hum", 12, 5);
        let mut binding = super::RuntimeBinding::mutable_local(super::Value::Int(7), false);
        binding.moved_at = Some(move_span);
        binding.moved_by = Some("test-only post-preflight corruption".to_string());
        let mut env = super::Env::new();
        env.insert("value".to_string(), binding);
        let results = vec![
            (
                "read_value",
                interpreter
                    .read_value(&env, "value", &use_span, "corrupted_probe")
                    .expect_err("corrupted read must hit the defensive moved-state branch"),
            ),
            (
                "read_consume_value",
                interpreter
                    .read_consume_value(&env, "value", &use_span, "corrupted_probe")
                    .expect_err("corrupted consume must hit the defensive moved-state branch"),
            ),
            (
                "ensure_can_set",
                interpreter
                    .ensure_can_set(
                        &env,
                        &RuntimePlace::Root("value".to_string()),
                        &use_span,
                        "corrupted_probe",
                    )
                    .expect_err("corrupted set must hit the defensive moved-state branch"),
            ),
        ];
        let diagnostics = interpreter.diagnostics.borrow().clone();
        Ok((results, diagnostics))
    }

    #[cfg(windows)]
    struct RecordingFileRead {
        result: Result<String, FileReadAdapterError>,
        calls: usize,
        paths: Vec<OsString>,
    }

    #[cfg(windows)]
    impl RecordingFileRead {
        fn success(text: &str) -> Self {
            Self {
                result: Ok(text.to_string()),
                calls: 0,
                paths: Vec::new(),
            }
        }

        fn failure(error: FileReadAdapterError) -> Self {
            Self {
                result: Err(error),
                calls: 0,
                paths: Vec::new(),
            }
        }
    }

    #[cfg(windows)]
    impl FileReadAdapter for RecordingFileRead {
        fn read_text(&mut self, path: &OsStr) -> Result<String, FileReadAdapterError> {
            self.calls += 1;
            self.paths.push(path.to_os_string());
            self.result.clone()
        }
    }

    #[cfg(windows)]
    #[derive(Debug, Clone, Copy)]
    enum InjectedLocality {
        Fixed,
        Unavailable,
        Unsafe,
    }

    #[cfg(windows)]
    struct RecordingLocality {
        result: InjectedLocality,
        calls: usize,
    }

    #[cfg(windows)]
    impl FileLocalityAdapter for RecordingLocality {
        fn revalidate(
            &mut self,
            path: &ValidatedNativePath,
        ) -> Result<ValidatedNativePath, FileLocalityError> {
            self.calls += 1;
            match self.result {
                InjectedLocality::Fixed => Ok(path.fixed_local_for_test()),
                InjectedLocality::Unavailable => Err(FileLocalityError::Unavailable),
                InjectedLocality::Unsafe => Err(FileLocalityError::UnsafePath),
            }
        }
    }

    fn allowed_stdout() -> OperatorGrantPolicy {
        let mut policy = OperatorGrantPolicy::default();
        policy.allow("stdout.write").expect("exact allow");
        policy
    }

    fn allowed_replay_and_stdout() -> OperatorGrantPolicy {
        let mut policy = allowed_stdout();
        policy.allow("clock.replay").expect("exact replay allow");
        policy
    }

    #[cfg(windows)]
    fn exact_file_policy(path: &OsStr, deny: bool) -> OperatorGrantPolicy {
        let mut policy = allowed_stdout();
        let mut grant = OsString::from("files.read=");
        grant.push(path);
        policy.allow_os(&grant).expect("exact native file allow");
        if deny {
            policy.deny("files.read").expect("exact file deny");
        }
        policy
    }

    #[cfg(windows)]
    fn integrated_policy(path: &OsStr) -> OperatorGrantPolicy {
        let mut policy = exact_file_policy(path, false);
        policy.allow("clock.replay").expect("exact replay allow");
        policy
    }

    #[cfg(windows)]
    fn run_integrated_fixture(
        path: &OsStr,
        file_text: &str,
        ticks: &[i64],
        policy: &OperatorGrantPolicy,
    ) -> (
        RunReport,
        RecordingOutput,
        RecordingReplay,
        RecordingLocality,
        RecordingFileRead,
    ) {
        let program = fixture_program(
            "examples/probes/integrated_local_app.hum",
            include_str!("../examples/probes/integrated_local_app.hum"),
        );
        let mut output = RecordingOutput::default();
        let mut replay = RecordingReplay::new(ticks);
        let mut locality = RecordingLocality {
            result: InjectedLocality::Fixed,
            calls: 0,
        };
        let mut files = RecordingFileRead::success(file_text);
        let report = run_program_with_file_adapters(
            &program,
            None,
            &[path.to_os_string()],
            policy,
            RunAdapters {
                output: &mut output,
                replay: &mut replay,
                file_locality: &mut locality,
                file: &mut files,
            },
        );
        (report, output, replay, locality, files)
    }

    #[cfg(windows)]
    #[test]
    fn integrated_local_app_is_repeatable_for_complete_inputs() {
        let path = OsString::from(format!("C:{}session-ae{}input.txt", '\\', '\\'));
        let text = "Hum reads exact UTF-8: lambda=λ\n";
        let policy = integrated_policy(&path);

        let first = run_integrated_fixture(&path, text, &[7], &policy);
        let second = run_integrated_fixture(&path, text, &[7], &policy);

        assert_eq!(first.0, second.0);
        assert_eq!(first.0.outcome, RunOutcome::AppSuccess);
        assert!(first.0.diagnostics.is_empty());
        assert_eq!(
            first.1.writes,
            vec![text.as_bytes().to_vec(), b"seven".to_vec()]
        );
        assert_eq!(first.1.calls, 2);
        assert_eq!(first.2.calls, 1);
        assert_eq!(first.3.calls, 1);
        assert_eq!(first.4.calls, 1);
        assert_eq!(first.4.paths, vec![path.clone()]);
        assert_eq!(second.1.writes, first.1.writes);
        assert_eq!(second.2.calls, first.2.calls);
        assert_eq!(second.3.calls, first.3.calls);
        assert_eq!(second.4.calls, first.4.calls);

        let changed_tick = run_integrated_fixture(&path, text, &[8], &policy);
        assert_eq!(changed_tick.0.outcome, RunOutcome::AppSuccess);
        assert_eq!(
            changed_tick.1.writes,
            vec![text.as_bytes().to_vec(), b"other".to_vec()]
        );

        let changed_bytes = run_integrated_fixture(&path, "changed input\n", &[7], &policy);
        assert_eq!(changed_bytes.0.outcome, RunOutcome::AppSuccess);
        assert_eq!(
            changed_bytes.1.writes,
            vec![b"changed input\n".to_vec(), b"seven".to_vec()]
        );
    }

    #[cfg(windows)]
    #[test]
    fn integrated_local_app_missing_file_keeps_outer_to_root_cause() {
        let program = fixture_program(
            "examples/probes/integrated_local_app.hum",
            include_str!("../examples/probes/integrated_local_app.hum"),
        );
        let path = OsString::from(format!("C:{}session-ae{}missing.txt", '\\', '\\'));
        let mut output = RecordingOutput::default();
        let mut replay = RecordingReplay::new(&[7]);
        let mut locality = RecordingLocality {
            result: InjectedLocality::Fixed,
            calls: 0,
        };
        let mut files = RecordingFileRead::failure(FileReadAdapterError::NotFound);
        let report = run_program_with_file_adapters(
            &program,
            None,
            std::slice::from_ref(&path),
            &integrated_policy(&path),
            RunAdapters {
                output: &mut output,
                replay: &mut replay,
                file_locality: &mut locality,
                file: &mut files,
            },
        );
        let RunOutcome::AppFailure(chain) = report.outcome else {
            panic!("expected typed app failure, got {:?}", report.outcome);
        };
        assert!(chain.contains("failure: IntegratedAppError.file"));
        assert!(chain.contains("caused by: FileReadError.not_found"));
        assert!(chain.contains("while calling `files_read_text`"));
        assert!(chain.contains("originated at examples/probes/integrated_local_app.hum:33:22"));
        assert!(!chain.contains("runtime trap"));
        assert_eq!(locality.calls, 1);
        assert_eq!(files.calls, 1);
        assert_eq!(replay.calls, 0);
        assert_eq!(output.calls, 0);
    }

    #[cfg(windows)]
    #[test]
    fn integrated_local_app_exact_denies_precede_their_adapters() {
        let path = OsString::from(format!("C:{}session-ae{}input.txt", '\\', '\\'));
        let text = "must remain adapter-bounded";

        let mut file_denied = integrated_policy(&path);
        file_denied.deny("files.read").expect("exact file deny");
        let file = run_integrated_fixture(&path, text, &[7], &file_denied);
        assert!(
            matches!(&file.0.outcome, RunOutcome::AppFailure(chain) if chain.contains("FileReadError.denied"))
        );
        assert_eq!(
            (file.1.calls, file.2.calls, file.3.calls, file.4.calls),
            (0, 0, 0, 0)
        );

        let mut replay_denied = integrated_policy(&path);
        replay_denied
            .deny("clock.replay")
            .expect("exact replay deny");
        let replay = run_integrated_fixture(&path, text, &[7], &replay_denied);
        assert!(
            matches!(&replay.0.outcome, RunOutcome::AppFailure(chain) if chain.contains("ReplayClockError.denied"))
        );
        assert_eq!(
            (
                replay.1.calls,
                replay.2.calls,
                replay.3.calls,
                replay.4.calls
            ),
            (0, 0, 1, 1)
        );

        let mut output_denied = integrated_policy(&path);
        output_denied
            .deny("stdout.write")
            .expect("exact output deny");
        let output = run_integrated_fixture(&path, text, &[7], &output_denied);
        assert!(
            matches!(&output.0.outcome, RunOutcome::AppFailure(chain) if chain.contains("OutputError.denied"))
        );
        assert_eq!(
            (
                output.1.calls,
                output.2.calls,
                output.3.calls,
                output.4.calls
            ),
            (0, 1, 1, 1)
        );

        for report in [&file.0, &replay.0, &output.0] {
            assert!(report.authority_events.iter().any(|event| {
                event.operator_allow_present
                    && event.operator_deny_present
                    && event.effective_decision == "deny"
                    && !event.adapter_called
            }));
        }
    }

    #[cfg(windows)]
    #[test]
    fn opaque_native_path_entry_runs_fixed_output_without_candidate_access() {
        let program = fixture_program(
            "examples/probes/opaque_native_path.hum",
            include_str!("../examples/probes/opaque_native_path.hum"),
        );
        let mut output = RecordingOutput::default();
        let mut replay = RecordingReplay::new(&[]);
        let args = vec![OsString::from(format!(
            "C:{}hum-session-ab{}definitely-missing.bin",
            char::from(92),
            char::from(92)
        ))];
        let report = run_program_with_adapters(
            &program,
            None,
            &args,
            &allowed_stdout(),
            &mut output,
            &mut replay,
        );
        assert_eq!(report.outcome, RunOutcome::AppSuccess);
        assert_eq!(output.writes, vec![b"path accepted".to_vec()]);
        assert_eq!(output.calls, 1);
        assert_eq!(replay.calls, 0);

        let path_source = include_str!("native_path.rs");
        let grant_source = include_str!("operator_grant.rs");
        for forbidden in [
            concat!(".meta", "data("),
            concat!("File::", "open("),
            concat!("canonical", "ize("),
            concat!("read_", "to_string("),
            concat!("read_", "to_end("),
        ] {
            assert!(!path_source.contains(forbidden));
            assert!(!grant_source.contains(forbidden));
        }
    }

    #[cfg(windows)]
    #[test]
    fn runtime_path_value_preserves_non_string_code_units() {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};

        let units = vec![
            u16::from(b'C'),
            u16::from(b':'),
            u16::from(b'\\'),
            u16::from(b'o'),
            u16::from(b'p'),
            u16::from(b'a'),
            u16::from(b'q'),
            u16::from(b'u'),
            u16::from(b'e'),
            u16::from(b'\\'),
            0xd800,
        ];
        let raw = OsString::from_wide(&units);
        let value = parse_arg("Path", &raw, true).expect("runner-owned native Path");
        let Value::Path(path) = value else {
            panic!("expected opaque Path value");
        };
        assert_eq!(path.as_os_str().encode_wide().collect::<Vec<_>>(), units);
        assert!(matches!(
            path.locality(),
            "fixed_local_v0" | "locality_unclassified"
        ));
        let direct = format!("C:{}opaque", char::from(92));
        assert!(
            parse_arg("Path", OsStr::new(&direct), false)
                .unwrap_err()
                .contains("structural app entry")
        );
    }

    #[cfg(windows)]
    #[test]
    fn exact_file_read_writes_checked_utf8_and_joins_forensic_events() {
        let program = fixture_program(
            "examples/probes/exact_file_read.hum",
            include_str!("../examples/probes/exact_file_read.hum"),
        );
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/file_read/session_ad_utf8.txt")
            .into_os_string();
        let text = "Hum reads exact UTF-8: lambda=λ\n";
        let mut output = RecordingOutput::default();
        let mut replay = RecordingReplay::new(&[]);
        let mut locality = RecordingLocality {
            result: InjectedLocality::Fixed,
            calls: 0,
        };
        let mut files = RecordingFileRead::success(text);
        let report = run_program_with_file_adapters(
            &program,
            None,
            std::slice::from_ref(&path),
            &exact_file_policy(&path, false),
            RunAdapters {
                output: &mut output,
                replay: &mut replay,
                file_locality: &mut locality,
                file: &mut files,
            },
        );

        assert_eq!(report.outcome, RunOutcome::AppSuccess);
        assert_eq!(output.writes, vec![text.as_bytes().to_vec()]);
        assert_eq!(output.calls, 1);
        assert_eq!(locality.calls, 1);
        assert_eq!(files.calls, 1);
        assert_eq!(files.paths.as_slice(), std::slice::from_ref(&path));
        let file_events = report
            .authority_events
            .iter()
            .filter(|event| event.capability_id == "files.read")
            .collect::<Vec<_>>();
        assert_eq!(file_events.len(), 2);
        assert_eq!(file_events[0].request_id, file_events[1].request_id);
        assert_eq!(file_events[0].native_path_identity, Some(path.clone()));
        assert_eq!(file_events[0].native_path_matched, Some(true));
        assert_eq!(file_events[1].locality_status, Some("fixed_local_v0"));
        assert_eq!(file_events[1].byte_count, text.len());
        assert!(file_events[1].adapter_called);
        assert_eq!(
            file_events[1].source_route,
            ["exact_file_read_probe", "run_tool"]
        );
        assert_eq!(file_events[1].source_route_spans.len(), 1);
    }

    #[cfg(windows)]
    #[test]
    fn file_authority_precedence_rejects_before_locality_or_candidate_adapter() {
        let program = fixture_program(
            "examples/probes/exact_file_read.hum",
            include_str!("../examples/probes/exact_file_read.hum"),
        );
        let input = OsString::from(format!("C:{}session-ad{}input.txt", '\\', '\\'));
        let other = OsString::from(format!("C:{}session-ad{}other.txt", '\\', '\\'));

        let cases = [
            (allowed_stdout(), "FileReadError.denied"),
            (exact_file_policy(&input, true), "FileReadError.denied"),
            (
                exact_file_policy(&other, false),
                "FileReadError.outside_grant",
            ),
        ];
        for (policy, expected) in cases {
            let mut output = RecordingOutput::default();
            let mut replay = RecordingReplay::new(&[]);
            let mut locality = RecordingLocality {
                result: InjectedLocality::Fixed,
                calls: 0,
            };
            let mut files = RecordingFileRead::success("must not read");
            let report = run_program_with_file_adapters(
                &program,
                None,
                std::slice::from_ref(&input),
                &policy,
                RunAdapters {
                    output: &mut output,
                    replay: &mut replay,
                    file_locality: &mut locality,
                    file: &mut files,
                },
            );
            let RunOutcome::AppFailure(chain) = report.outcome else {
                panic!("expected typed app failure, got {:?}", report.outcome);
            };
            assert!(chain.contains("failure: FileAppError.file"));
            assert!(chain.contains(&format!("caused by: {expected}")));
            assert!(chain.contains("while calling `files_read_text`"));
            assert!(!chain.contains("runtime trap"));
            assert_eq!(locality.calls, 0);
            assert_eq!(files.calls, 0);
            assert_eq!(output.calls, 0);
            let exercise = report
                .authority_events
                .iter()
                .find(|event| {
                    event.capability_id == "files.read" && event.event_kind == "operation_exercise"
                })
                .expect("file exercise");
            assert!(!exercise.adapter_called);
        }
    }

    #[cfg(windows)]
    #[test]
    fn locality_and_every_file_adapter_failure_are_typed_and_causal() {
        let program = fixture_program(
            "examples/probes/exact_file_read.hum",
            include_str!("../examples/probes/exact_file_read.hum"),
        );
        let input = OsString::from(format!("C:{}session-ad{}input.txt", '\\', '\\'));

        for (locality_result, adapter_error, expected, expected_file_calls) in [
            (
                InjectedLocality::Unavailable,
                FileReadAdapterError::IoFailed,
                "FileReadError.unavailable",
                0,
            ),
            (
                InjectedLocality::Unsafe,
                FileReadAdapterError::IoFailed,
                "FileReadError.unsafe_path",
                0,
            ),
            (
                InjectedLocality::Fixed,
                FileReadAdapterError::UnsafePath,
                "FileReadError.unsafe_path",
                1,
            ),
            (
                InjectedLocality::Fixed,
                FileReadAdapterError::NotFound,
                "FileReadError.not_found",
                1,
            ),
            (
                InjectedLocality::Fixed,
                FileReadAdapterError::NotFile,
                "FileReadError.not_file",
                1,
            ),
            (
                InjectedLocality::Fixed,
                FileReadAdapterError::TooLarge,
                "FileReadError.too_large",
                1,
            ),
            (
                InjectedLocality::Fixed,
                FileReadAdapterError::InvalidUtf8,
                "FileReadError.invalid_utf8",
                1,
            ),
            (
                InjectedLocality::Fixed,
                FileReadAdapterError::IoFailed,
                "FileReadError.io_failed",
                1,
            ),
        ] {
            let mut output = RecordingOutput::default();
            let mut replay = RecordingReplay::new(&[]);
            let mut locality = RecordingLocality {
                result: locality_result,
                calls: 0,
            };
            let mut files = RecordingFileRead::failure(adapter_error);
            let report = run_program_with_file_adapters(
                &program,
                None,
                std::slice::from_ref(&input),
                &exact_file_policy(&input, false),
                RunAdapters {
                    output: &mut output,
                    replay: &mut replay,
                    file_locality: &mut locality,
                    file: &mut files,
                },
            );
            let RunOutcome::AppFailure(chain) = report.outcome else {
                panic!("expected typed app failure, got {:?}", report.outcome);
            };
            assert!(chain.contains("failure: FileAppError.file"));
            assert!(chain.contains(&format!("caused by: {expected}")));
            assert!(chain.contains("originated at examples/probes/exact_file_read.hum:31:22"));
            assert_eq!(locality.calls, 1);
            assert_eq!(files.calls, expected_file_calls);
            assert_eq!(output.calls, 0);
        }
    }

    #[test]
    fn runner_replay_consumes_ordered_ticks_and_records_forensic_events() {
        let program = fixture_program(
            "examples/probes/runner_replay_clock.hum",
            include_str!("../examples/probes/runner_replay_clock.hum"),
        );
        let mut output = RecordingOutput::default();
        let mut replay = RecordingReplay::new(&[1, 7]);
        let report = run_program_with_adapters(
            &program,
            None,
            &[],
            &allowed_replay_and_stdout(),
            &mut output,
            &mut replay,
        );
        assert_eq!(report.outcome, RunOutcome::AppSuccess);
        assert_eq!(output.writes, vec![b"seven".to_vec()]);
        assert_eq!(replay.calls, 2);
        let replay_events = report
            .authority_events
            .iter()
            .filter(|event| event.capability_id == "clock.replay")
            .collect::<Vec<_>>();
        assert_eq!(replay_events.len(), 4);
        assert_eq!(replay_events[1].replay_index, Some(0));
        assert_eq!(replay_events[1].replay_tick, Some(1));
        assert_eq!(replay_events[3].replay_index, Some(1));
        assert_eq!(replay_events[3].replay_tick, Some(7));
        assert_ne!(
            replay_events[0].source_policy_id,
            replay_events[2].source_policy_id
        );
        assert_eq!(replay_events[0].source_route_spans[0].line, 49);
        assert_eq!(replay_events[2].source_route_spans[0].line, 50);
        assert!(replay_events.iter().all(|event| {
            event.source_route == ["runner_replay_clock_probe", "run_tool", "read_tick"]
                && event.source_route_spans.len() == 2
                && event.source_policy_id.contains("clock-replay")
        }));
    }

    #[test]
    fn runner_replay_denials_do_not_call_adapter_or_grant_from_values() {
        let program = fixture_program(
            "examples/probes/runner_replay_clock.hum",
            include_str!("../examples/probes/runner_replay_clock.hum"),
        );
        for (reason, policy) in [
            ("default_empty_grant_set_v0", allowed_stdout()),
            ("exact_deny_overrides_allow_v0", {
                let mut policy = allowed_replay_and_stdout();
                policy.deny("clock.replay").expect("exact replay deny");
                policy
            }),
        ] {
            let mut output = RecordingOutput::default();
            let mut replay = RecordingReplay::new(&[1, 7]);
            let report =
                run_program_with_adapters(&program, None, &[], &policy, &mut output, &mut replay);
            assert!(matches!(
                report.outcome,
                RunOutcome::AppFailure(ref chain)
                    if chain.contains("ReplayAppError.replay")
                        && chain.contains("ReplayClockError.denied")
            ));
            assert_eq!(replay.calls, 0, "{reason}");
            assert_eq!(output.calls, 0, "{reason}");
            let exercise = report
                .authority_events
                .iter()
                .find(|event| event.event_kind == "operation_exercise")
                .expect("replay exercise event");
            assert_eq!(exercise.decision_reason, reason);
            assert!(!exercise.adapter_called);
        }
    }

    #[test]
    fn runner_replay_exhaustion_is_typed_and_causal() {
        let program = fixture_program(
            "examples/probes/runner_replay_clock.hum",
            include_str!("../examples/probes/runner_replay_clock.hum"),
        );
        let mut output = RecordingOutput::default();
        let mut replay = RecordingReplay::new(&[1]);
        let report = run_program_with_adapters(
            &program,
            None,
            &[],
            &allowed_replay_and_stdout(),
            &mut output,
            &mut replay,
        );
        assert!(matches!(
            report.outcome,
            RunOutcome::AppFailure(ref chain)
                if chain.contains("ReplayAppError.replay")
                    && chain.contains("while calling `read_tick`")
                    && chain.contains("ReplayClockError.exhausted")
                    && chain.contains("originated at examples/probes/runner_replay_clock.hum:30:22")
        ));
        assert_eq!(replay.calls, 2);
        assert_eq!(output.calls, 0);
        let exhausted = report
            .authority_events
            .iter()
            .find(|event| event.result == "sequence_exhausted_v0")
            .expect("exhausted exercise event");
        assert!(exhausted.adapter_called);
        assert_eq!(exhausted.replay_index, Some(1));
        assert_eq!(exhausted.replay_tick, None);
    }

    #[test]
    fn runner_replay_changed_tick_changes_only_selected_literal() {
        let program = fixture_program(
            "examples/probes/runner_replay_clock.hum",
            include_str!("../examples/probes/runner_replay_clock.hum"),
        );
        let run = |ticks: &[i64]| {
            let mut output = RecordingOutput::default();
            let mut replay = RecordingReplay::new(ticks);
            let report = run_program_with_adapters(
                &program,
                None,
                &[],
                &allowed_replay_and_stdout(),
                &mut output,
                &mut replay,
            );
            (report.outcome, output.writes)
        };
        assert_eq!(run(&[1, 7]), run(&[1, 7]));
        assert_eq!(run(&[1, 7]).1, vec![b"seven".to_vec()]);
        assert_eq!(run(&[1, 8]).1, vec![b"other".to_vec()]);
    }

    #[test]
    fn runner_replay_runtime_has_no_host_clock_symbols() {
        let source = include_str!("run.rs");
        assert!(!source.contains(concat!("System", "Time")));
        assert!(!source.contains(concat!("std::", "time")));
        assert!(!source.contains(concat!("Instant", "::now")));
    }

    #[test]
    fn runner_replay_path_separator_variants_keep_policy_identity_and_routes() {
        let source = include_str!("../examples/probes/runner_replay_clock.hum");
        let run = |path: &str| {
            let program = fixture_program(path, source);
            let mut output = RecordingOutput::default();
            let mut replay = RecordingReplay::new(&[1, 7]);
            let report = run_program_with_adapters(
                &program,
                None,
                &[],
                &allowed_replay_and_stdout(),
                &mut output,
                &mut replay,
            );
            assert_eq!(report.outcome, RunOutcome::AppSuccess);
            report
                .authority_events
                .into_iter()
                .filter(|event| {
                    event.capability_id == "clock.replay" && event.event_kind == "operator_decision"
                })
                .map(|event| {
                    (
                        event.source_policy_id,
                        event
                            .source_route_spans
                            .iter()
                            .map(|span| (span.line, span.column))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        };
        let backward_spelling = "examples/probes/runner_replay_clock.hum"
            .chars()
            .map(|character| {
                if character == '/' {
                    char::from(92)
                } else {
                    character
                }
            })
            .collect::<String>();
        assert_eq!(
            run("examples/probes/runner_replay_clock.hum"),
            run(&backward_spelling)
        );
    }

    #[test]
    fn bounded_stdout_writes_exact_utf8_and_records_joined_events() {
        let program = fixture_program(
            "examples/probes/bounded_stdout.hum",
            include_str!("../examples/probes/bounded_stdout.hum"),
        );
        let mut output = RecordingOutput::default();
        let report = run_program_with_output(
            &program,
            None,
            &["Hum λ".to_string()],
            &allowed_stdout(),
            &mut output,
        );
        assert_eq!(report.outcome, RunOutcome::AppSuccess);
        assert_eq!(output.writes, vec!["Hum λ".as_bytes()]);
        assert_eq!(output.calls, 1);
        assert_eq!(report.authority_events.len(), 2);
        assert_eq!(report.authority_events[0].event_sequence, 1);
        assert_eq!(report.authority_events[1].event_sequence, 2);
        assert_eq!(
            report.authority_events[0].request_id,
            report.authority_events[1].request_id
        );
        assert_eq!(report.authority_events[0].authority_surface, "hum_run_cli");
        assert_eq!(
            report.authority_events[0].app_name.as_deref(),
            Some("bounded_stdout_probe")
        );
        assert_eq!(
            report.authority_events[0].source_route,
            ["bounded_stdout_probe", "run_tool", "emit"]
        );
        assert_eq!(report.authority_events[0].source_route_spans.len(), 2);
        assert!(report.authority_events[0].source_task_authorized);
        assert!(report.authority_events[0].source_app_authorized);
        assert!(report.authority_events[0].operator_allow_present);
        assert!(!report.authority_events[0].operator_deny_present);
        assert_eq!(report.authority_events[0].effective_decision, "allow");
        assert_eq!(report.authority_events[1].result, "write_succeeded_v0");
        assert!(report.authority_events[1].adapter_called);
        assert!(
            report.authority_events[0]
                .source_policy_id
                .contains("source-capability-output-operation-stdout-write")
        );
    }

    #[test]
    fn bounded_stdout_path_separator_variants_keep_policy_identity_and_route() {
        let source = include_str!("../examples/probes/bounded_stdout.hum");
        let forward = fixture_program("examples/probes/bounded_stdout.hum", source);
        let separator = char::from(92).to_string();
        let backward_path = ["examples", "probes", "bounded_stdout.hum"].join(&separator);
        let backward = fixture_program(&backward_path, source);
        let mut forward_output = RecordingOutput::default();
        let forward_report = run_program_with_output(
            &forward,
            None,
            &["same".to_string()],
            &allowed_stdout(),
            &mut forward_output,
        );
        let mut backward_output = RecordingOutput::default();
        let backward_report = run_program_with_output(
            &backward,
            None,
            &["same".to_string()],
            &allowed_stdout(),
            &mut backward_output,
        );
        assert_eq!(forward_report.outcome, RunOutcome::AppSuccess);
        assert_eq!(backward_report.outcome, RunOutcome::AppSuccess);
        assert_eq!(forward_output.writes, vec![b"same".to_vec()]);
        assert_eq!(backward_output.writes, forward_output.writes);
        let forward_decision = &forward_report.authority_events[0];
        let backward_decision = &backward_report.authority_events[0];
        assert_eq!(
            forward_decision.source_policy_id,
            backward_decision.source_policy_id
        );
        assert_eq!(
            forward_decision.source_route_spans,
            backward_decision.source_route_spans
        );
        assert_eq!(
            forward_decision.source_route,
            backward_decision.source_route
        );
    }

    #[test]
    fn bounded_stdout_default_and_explicit_denial_never_call_adapter() {
        let program = fixture_program(
            "examples/probes/bounded_stdout.hum",
            include_str!("../examples/probes/bounded_stdout.hum"),
        );
        for (expected_reason, mut policy) in [
            ("default_empty_grant_set_v0", OperatorGrantPolicy::default()),
            ("exact_deny_overrides_allow_v0", {
                let mut policy = allowed_stdout();
                policy.deny("stdout.write").expect("exact deny");
                policy
            }),
        ] {
            let mut output = RecordingOutput::default();
            let report = run_program_with_output(
                &program,
                None,
                &["blocked".to_string()],
                &policy,
                &mut output,
            );
            assert!(matches!(
                report.outcome,
                RunOutcome::AppFailure(ref chain) if chain.contains("OutputError.denied")
            ));
            assert!(output.writes.is_empty());
            assert_eq!(output.calls, 0);
            assert_eq!(report.authority_events[0].decision_reason, expected_reason);
            assert_eq!(
                report.authority_events[0].operator_allow_present,
                expected_reason == "exact_deny_overrides_allow_v0"
            );
            assert_eq!(
                report.authority_events[0].operator_deny_present,
                expected_reason == "exact_deny_overrides_allow_v0"
            );
            assert!(!report.authority_events[1].adapter_called);
            policy
                .allow("stdout.write")
                .expect("duplicate remains valid");
        }
    }

    #[test]
    fn bounded_stdout_write_failure_is_typed_and_opaque() {
        let program = fixture_program(
            "examples/probes/bounded_stdout.hum",
            include_str!("../examples/probes/bounded_stdout.hum"),
        );
        let mut output = RecordingOutput {
            fail_on_call: Some(0),
            ..Default::default()
        };
        let report = run_program_with_output(
            &program,
            None,
            &["fail".to_string()],
            &allowed_stdout(),
            &mut output,
        );
        assert!(matches!(
            report.outcome,
            RunOutcome::AppFailure(ref chain)
                if chain.contains("AppError.output")
                    && chain.contains("OutputError.write_failed")
                    && !chain.contains("host")
        ));
        assert!(output.writes.is_empty());
        assert_eq!(output.calls, 1);
        assert_eq!(report.authority_events[1].result, "write_failed_v0");
    }

    #[test]
    fn bounded_stdout_rolling_limit_rejects_before_second_adapter_call() {
        let source = r#"app limit_probe {
  uses:
    stdout.write
  starts with:
    run_tool
  task run_tool(first: Text, second: Text) -> Result Unit, OutputError {
    uses:
      stdout.write
    fails when:
      the rolling output limit is exceeded
    allocates:
      callee-defined allocation behavior
    does:
      let first_write = try stdout_write(first)
      let second_write = try stdout_write(second)
      return second_write
  }
}
"#;
        let program = fixture_program("limit.hum", source);
        let mut output = RecordingOutput::default();
        let first = "a".repeat(OUTPUT_LIMIT_BYTES);
        let report = run_program_with_output(
            &program,
            None,
            &[first.clone(), "b".to_string()],
            &allowed_stdout(),
            &mut output,
        );
        assert!(matches!(
            report.outcome,
            RunOutcome::AppFailure(ref chain) if chain.contains("OutputError.limit_exceeded")
        ));
        assert_eq!(output.writes, vec![first.into_bytes()]);
        assert_eq!(output.calls, 1);
        assert_eq!(report.authority_events.len(), 4);
        assert_eq!(
            report.authority_events[3].result,
            "limit_rejected_before_adapter_v0"
        );
        assert!(!report.authority_events[3].adapter_called);
    }

    #[test]
    fn bounded_stdout_keeps_prior_bytes_when_later_adapter_write_fails() {
        let source = r#"app partial_output_probe {
  uses:
    stdout.write
  starts with:
    run_tool
  task run_tool -> Result Unit, OutputError {
    uses:
      stdout.write
    fails when:
      an adapter write fails
    allocates:
      callee-defined allocation behavior
    does:
      let first = try stdout_write("first")
      let second = try stdout_write("second")
      return second
  }
}
"#;
        let program = fixture_program("partial.hum", source);
        let mut output = RecordingOutput {
            fail_on_call: Some(1),
            ..Default::default()
        };
        let report = run_program_with_output(&program, None, &[], &allowed_stdout(), &mut output);
        assert!(matches!(
            report.outcome,
            RunOutcome::AppFailure(ref chain) if chain.contains("OutputError.write_failed")
        ));
        assert_eq!(output.writes, vec![b"first".to_vec()]);
        assert_eq!(output.calls, 2);
    }

    #[test]
    fn bounded_stdout_audit_selects_complete_multiple_caller_routes_stably() {
        let source = r#"app replay_probe {
  uses:
    stdout.write
  starts with:
    run_tool
  task emit(text: Text) -> Result Unit, OutputError {
    uses:
      stdout.write
    fails when:
      output fails
    allocates:
      callee-defined allocation behavior
    does:
      let written = try stdout_write(text)
      return written
  }
  task left -> Result Unit, OutputError {
    uses:
      stdout.write
    fails when:
      left output fails
    allocates:
      callee-defined allocation behavior
    does:
      let written = try emit("L")
      return written
  }
  task right -> Result Unit, OutputError {
    uses:
      stdout.write
    fails when:
      right output fails
    allocates:
      callee-defined allocation behavior
    does:
      let written = try emit("R")
      return written
  }
  task run_tool -> Result Unit, OutputError {
    uses:
      stdout.write
    fails when:
      either output route fails
    allocates:
      callee-defined allocation behavior
    does:
      let left_done = try left()
      let right_done = try right()
      return right_done
  }
}
"#;
        let program = fixture_program("replay.hum", source);
        let mut first_output = RecordingOutput::default();
        let first =
            run_program_with_output(&program, None, &[], &allowed_stdout(), &mut first_output);
        let mut second_output = RecordingOutput::default();
        let second =
            run_program_with_output(&program, None, &[], &allowed_stdout(), &mut second_output);
        assert_eq!(first.outcome, RunOutcome::AppSuccess);
        assert_eq!(first_output.writes, vec![b"L".to_vec(), b"R".to_vec()]);
        assert_eq!(first_output.calls, 2);
        let first_decisions = first
            .authority_events
            .iter()
            .filter(|event| event.event_kind == "operator_decision")
            .collect::<Vec<_>>();
        let second_decisions = second
            .authority_events
            .iter()
            .filter(|event| event.event_kind == "operator_decision")
            .collect::<Vec<_>>();
        assert_eq!(first_decisions.len(), 2);
        assert_eq!(
            first_decisions[0].source_route,
            ["replay_probe", "run_tool", "left", "emit"]
        );
        assert_eq!(
            first_decisions[1].source_route,
            ["replay_probe", "run_tool", "right", "emit"]
        );
        assert_eq!(first_decisions[0].source_route_spans.len(), 3);
        assert_eq!(first_decisions[1].source_route_spans.len(), 3);
        assert_ne!(
            first_decisions[0].source_policy_id,
            first_decisions[1].source_policy_id
        );
        assert_eq!(
            first_decisions
                .iter()
                .map(|event| &event.source_policy_id)
                .collect::<Vec<_>>(),
            second_decisions
                .iter()
                .map(|event| &event.source_policy_id)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn bounded_stdout_audit_selects_the_executed_conditional_call_occurrence() {
        let source = r#"app conditional_probe {
  uses:
    stdout.write
  starts with:
    run_tool
  task emit(text: Text) -> Result Unit, OutputError {
    uses:
      stdout.write
    fails when:
      output fails
    allocates:
      callee-defined allocation behavior
    does:
      let written = try stdout_write(text)
      return written
  }
  task run_tool(take_first: Bool) -> Result Unit, OutputError {
    uses:
      stdout.write
    fails when:
      selected output fails
    allocates:
      callee-defined allocation behavior
    does:
      if take_first {
        let first = try emit("first")
      }
      if true {
        let second = try emit("second")
      }
      return
  }
}
"#;
        let second_call_line = source
            .lines()
            .position(|line| line.contains("try emit(\"second\")"))
            .expect("second call")
            + 1;
        let program = fixture_program("conditional.hum", source);
        let mut first_output = RecordingOutput::default();
        let first = run_program_with_output(
            &program,
            None,
            &["false".to_string()],
            &allowed_stdout(),
            &mut first_output,
        );
        let mut second_output = RecordingOutput::default();
        let second = run_program_with_output(
            &program,
            None,
            &["false".to_string()],
            &allowed_stdout(),
            &mut second_output,
        );
        assert_eq!(first.outcome, RunOutcome::AppSuccess);
        assert_eq!(first_output.writes, vec![b"second".to_vec()]);
        let first_decision = first
            .authority_events
            .iter()
            .find(|event| event.event_kind == "operator_decision")
            .expect("decision event");
        let second_decision = second
            .authority_events
            .iter()
            .find(|event| event.event_kind == "operator_decision")
            .expect("repeat decision event");
        assert_eq!(first_decision.source_route_spans[0].line, second_call_line);
        assert_eq!(
            first_decision.source_policy_id,
            second_decision.source_policy_id
        );
        assert_eq!(
            first_decision.source_route_spans,
            second_decision.source_route_spans
        );
    }

    #[test]
    fn runs_add_program() {
        let program = fixture_program(
            "examples/core/add.hum",
            include_str!("../examples/core/add.hum"),
        );
        let report = run_program(&program, Some("add"), &["2".to_string(), "3".to_string()]);
        assert_eq!(report.outcome, RunOutcome::Success("5".to_string()));
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn structural_app_runs_without_displaying_unit() {
        let program = fixture_program(
            "examples/probes/pure_app_entry.hum",
            include_str!("../examples/probes/pure_app_entry.hum"),
        );
        let report = run_program(&program, None, &["hello".to_string()]);
        assert_eq!(report.outcome, RunOutcome::AppSuccess);
        assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    }

    #[test]
    fn structural_app_keeps_typed_failure_distinct_from_legacy_display() {
        let program = fixture_program(
            "examples/probes/fallible_app_entry.hum",
            include_str!("../examples/probes/fallible_app_entry.hum"),
        );
        let report = run_program(&program, None, &["true".to_string()]);
        let RunOutcome::AppFailure(chain) = report.outcome else {
            panic!("expected app failure");
        };
        assert!(chain.contains("failure: LaunchError.requested"));
        assert!(chain.contains("originated at examples/probes/fallible_app_entry.hum:23:9"));
    }

    #[test]
    fn app_mode_rejects_external_same_name_but_entry_remains_direct_probe() {
        let program = fixture_program(
            "fixtures/app_entry/session_x_external_same_name_fail.hum",
            include_str!("../fixtures/app_entry/session_x_external_same_name_fail.hum"),
        );
        let app_report = run_program(&program, None, &[]);
        assert!(matches!(
            app_report.outcome,
            RunOutcome::Trap(ref message) if message == "H0614 app start task is not a child"
        ));
        assert_eq!(app_report.diagnostics.len(), 1);
        assert_eq!(
            app_report.diagnostics[0].code,
            DiagnosticCode::APP_START_NOT_CHILD
        );

        let direct_report = run_program(&program, Some("run_tool"), &[]);
        assert_eq!(direct_report.outcome, RunOutcome::Success("()".to_string()));
        assert!(direct_report.diagnostics.is_empty());
    }

    #[test]
    fn divide_zero_fails_needs_with_caller_blame() {
        let program = fixture_program(
            "examples/core/divide.hum",
            include_str!("../examples/core/divide.hum"),
        );
        let report = run_program(
            &program,
            Some("divide"),
            &["10".to_string(), "0".to_string()],
        );
        assert_eq!(report.outcome, RunOutcome::ContractViolation);
        assert_eq!(
            report.diagnostics.len(),
            1,
            "diagnostics: {:#?}",
            report.diagnostics
        );
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::NEEDS_CONTRACT_VIOLATION);
        assert_eq!(diagnostic.severity, Severity::Error);
        let rendered = diagnostic.render();
        assert!(
            rendered.contains("examples/core/divide.hum:12:"),
            "{rendered}"
        );
        assert!(
            rendered.contains("caller did not satisfy needs: b != 0"),
            "{rendered}"
        );
    }

    #[test]
    fn unchecked_task_can_return_typed_failure() {
        let source = r#"module tests.typed_failure

type MathError {
  code: Text
}

task fail_now() -> Result Int, MathError {
  fails when:
    divisor is zero

  does:
    fail MathError.divide_by_zero
}
"#;
        let program = fixture_program("typed_failure.hum", source);
        let report = run_program(&program, Some("fail_now"), &[]);
        assert_eq!(
            report.outcome,
            RunOutcome::Failure(
                "failure: MathError.divide_by_zero\n  originated at typed_failure.hum:12:5"
                    .to_string()
            )
        );
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn causal_failure_chain_keeps_outer_to_root_sites() {
        let program = fixture_program(
            "examples/probes/causal_failures.hum",
            include_str!("../examples/probes/causal_failures.hum"),
        );
        let report = run_program(&program, Some("outer_value"), &["true".to_string()]);
        let RunOutcome::Failure(chain) = report.outcome else {
            panic!("expected typed failure, got {:?}", report.outcome);
        };
        let expected = [
            "failure: OuterError.context",
            ":74:5 while calling `middle_value`",
            "caused by: MiddleError.context",
            ":59:5 while calling `root_value`",
            "caused by: RootError.origin",
            "originated at examples/probes/causal_failures.hum:27:7",
        ];
        for text in expected {
            assert!(chain.contains(text), "missing {text}: {chain}");
        }
    }

    #[test]
    fn causal_failure_renders_direct_origin_and_same_root_propagation_once() {
        let program = fixture_program(
            "examples/probes/causal_failures.hum",
            include_str!("../examples/probes/causal_failures.hum"),
        );

        let direct = run_program(&program, Some("root_value"), &["true".to_string()]);
        let RunOutcome::Failure(direct_chain) = direct.outcome else {
            panic!("expected direct typed failure, got {:?}", direct.outcome);
        };
        assert!(direct_chain.starts_with("failure: RootError.origin"));
        assert_eq!(direct_chain.matches("originated at").count(), 1);
        assert!(direct_chain.contains("causal_failures.hum:27:7"));

        let propagated = run_program(&program, Some("same_root"), &["true".to_string()]);
        let RunOutcome::Failure(propagated_chain) = propagated.outcome else {
            panic!(
                "expected propagated typed failure, got {:?}",
                propagated.outcome
            );
        };
        assert!(propagated_chain.starts_with("failure: RootError.origin"));
        assert_eq!(propagated_chain.matches("propagated at").count(), 1);
        assert_eq!(propagated_chain.matches("originated at").count(), 1);
        assert!(propagated_chain.contains("while calling `root_value`"));
    }

    #[test]
    fn implicit_fallible_call_rejects_with_shared_diagnostic() {
        let program = fixture_program(
            "fixtures/full_type_check/session_w_implicit_fallible_call_fail.hum",
            include_str!("../fixtures/full_type_check/session_w_implicit_fallible_call_fail.hum"),
        );
        let report = run_program(&program, Some("caller"), &[]);
        assert_eq!(report.outcome, RunOutcome::PreflightRejected);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY
        );
        assert!(
            report.diagnostics[0]
                .render()
                .contains("callee declared at")
        );
    }

    #[test]
    fn count_completed_counts_done_records_with_prose_warning() {
        let program = fixture_program(
            "examples/core/count_completed.hum",
            include_str!("../examples/core/count_completed.hum"),
        );
        let report = run_program(
            &program,
            Some("count_completed"),
            &["[{done:true},{done:false},{done:true}]".to_string()],
        );
        assert_eq!(report.outcome, RunOutcome::Success("2".to_string()));
        assert_eq!(
            report.diagnostics.len(),
            1,
            "diagnostics: {:#?}",
            report.diagnostics
        );
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::UNCHECKED_PROSE_CONTRACT);
        assert_eq!(diagnostic.severity, Severity::Warning);
        let rendered = diagnostic.render();
        assert!(
            rendered.contains(
                "unchecked prose ensures contract: result is at most the number of items"
            ),
            "{rendered}"
        );
    }

    #[test]
    fn wrong_add_fixture_fails_ensures_with_task_blame() {
        let program = fixture_program(
            "fixtures/run/wrong_add_contract.hum",
            include_str!("../fixtures/run/wrong_add_contract.hum"),
        );
        let report = run_program(&program, Some("add"), &["2".to_string(), "3".to_string()]);
        assert_eq!(report.outcome, RunOutcome::ContractViolation);
        assert_eq!(
            report.diagnostics.len(),
            1,
            "diagnostics: {:#?}",
            report.diagnostics
        );
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::ENSURES_CONTRACT_VIOLATION);
        assert_eq!(diagnostic.severity, Severity::Error);
        let rendered = diagnostic.render();
        assert!(
            rendered.contains("fixtures/run/wrong_add_contract.hum:8:"),
            "{rendered}"
        );
        assert!(
            rendered.contains("task `add` did not satisfy ensures: result == a + b"),
            "{rendered}"
        );
    }

    #[test]
    fn wrong_swap_fixture_fails_old_ensures_with_task_blame() {
        let program = fixture_program(
            "fixtures/run/session_t_wrong_swap_contract.hum",
            include_str!("../fixtures/run/session_t_wrong_swap_contract.hum"),
        );
        let report = run_program(&program, Some("wrong_swap_demo"), &[]);
        assert_eq!(report.outcome, RunOutcome::ContractViolation);
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::ENSURES_CONTRACT_VIOLATION);
        let rendered = diagnostic.render();
        assert!(
            rendered.contains("task `swap_xy` did not satisfy ensures: result.x == old(point.y)"),
            "{rendered}"
        );
    }

    #[test]
    fn complete_item_preservation_is_checked_not_prose() {
        let program = fixture_program(
            "fixtures/run/session_o_complete_item_field_place.hum",
            include_str!("../fixtures/run/session_o_complete_item_field_place.hum"),
        );
        let report = run_program(&program, Some("complete_item_demo"), &[]);
        assert_eq!(
            report.outcome,
            RunOutcome::Success("{done: true, title: hum}".to_string())
        );
        assert!(
            report.diagnostics.is_empty(),
            "preservation must be a checked old() contract with no prose warning: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn builder_list_len_and_exact_content_contracts_are_checked() {
        let program = fixture_program(
            "examples/probes/list_builder.hum",
            include_str!("../examples/probes/list_builder.hum"),
        );
        let report = run_program(&program, Some("builder_demo"), &[]);
        assert_eq!(
            report.outcome,
            RunOutcome::Success("[parse, check, run]".to_string())
        );
        assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    }

    #[test]
    fn spaced_old_is_malformed_executable_predicate() {
        let source = r#"module tests.spaced_old

task echo_amount(amount: UInt) -> UInt {
  why:
    pin the canonical-prefix rule for old

  ensures:
    result == old (amount)

  does:
    return amount
}
"#;
        let program = fixture_program("tests/spaced_old.hum", source);
        let report = run_program(&program, Some("echo_amount"), &["7".to_string()]);
        assert_eq!(report.outcome, RunOutcome::PreflightRejected);
        assert_eq!(report.diagnostics.len(), 1, "{:#?}", report.diagnostics);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::INVALID_EXECUTABLE_PREDICATE
        );
    }

    #[test]
    fn spaced_list_len_is_malformed_executable_predicate() {
        let source = r#"module tests.spaced_list_len

task make_items() -> List Text {
  why:
    pin the canonical-prefix rule for list_len

  ensures:
    list_len (result) == 2

  does:
    change items: List Text = []
    let added_a: Unit = list_append(change items, "a")
    let added_b: Unit = list_append(change items, "b")
    return items
}
"#;
        let program = fixture_program("tests/spaced_list_len.hum", source);
        let report = run_program(&program, Some("make_items"), &[]);
        assert_eq!(report.diagnostics.len(), 1, "{:#?}", report.diagnostics);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::INVALID_EXECUTABLE_PREDICATE
        );
    }

    #[test]
    fn list_len_is_reachable_from_task_bodies() {
        let source = r#"module tests.list_len_body

task count_items() -> Int {
  why:
    pin list_len as a documented body-reachable length read

  does:
    change items: List Text = []
    let added_a: Unit = list_append(change items, "a")
    let added_b: Unit = list_append(change items, "b")
    return list_len(items)
}
"#;
        let program = fixture_program("tests/list_len_body.hum", source);
        let report = run_program(&program, Some("count_items"), &[]);
        assert_eq!(report.outcome, RunOutcome::Success("2".to_string()));
    }

    #[test]
    fn old_in_task_body_traps_with_clear_message() {
        let source = r#"module tests.old_in_body

task echo_amount(amount: UInt) -> UInt {
  why:
    pin that old() stays contract-only vocabulary

  does:
    return old(amount)
}
"#;
        let program = fixture_program("tests/old_in_body.hum", source);
        let report = run_program(&program, Some("echo_amount"), &["7".to_string()]);
        let RunOutcome::Trap(message) = &report.outcome else {
            panic!("old() in a body must trap, got {:#?}", report.outcome);
        };
        assert!(
            message.contains("available only in `ensures:`"),
            "{message}"
        );
    }

    #[test]
    fn old_in_needs_is_semantically_rejected_before_evaluation() {
        let program = fixture_program(
            "fixtures/run/session_t_old_in_needs_prose.hum",
            include_str!("../fixtures/run/session_t_old_in_needs_prose.hum"),
        );
        let report = run_program(&program, Some("old_in_needs_demo"), &[]);
        assert_eq!(report.outcome, RunOutcome::PreflightRejected);
        assert_eq!(report.diagnostics.len(), 1);
        let rendered = report.diagnostics[0].render();
        assert!(
            rendered.contains("H0704") && rendered.contains("old_place_not_entry_readable_v2"),
            "{rendered}"
        );
    }

    #[test]
    fn task_call_expression_runs_called_task() {
        let source = r#"module tests.calls

task add(a: Int, b: Int) -> Int {
  does:
    return a + b
}

task add_one(value: Int) -> Int {
  does:
    return add(value, 1)
}
"#;
        let program = fixture_program("calls.hum", source);
        let report = run_program(&program, Some("add_one"), &["4".to_string()]);
        assert_eq!(report.outcome, RunOutcome::Success("5".to_string()));
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn integer_overflow_traps() {
        let source = r#"module tests.overflow

task overflow(value: Int) -> Int {
  does:
    return value + 1
}
"#;
        let program = fixture_program("overflow.hum", source);
        let report = run_program(&program, Some("overflow"), &[i64::MAX.to_string()]);
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("integer overflow while evaluating `value + 1`".to_string())
        );
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn division_by_zero_traps() {
        let source = r#"module tests.divide_trap

task unchecked_divide(a: Int, b: Int) -> Int {
  does:
    return a / b
}
"#;
        let program = fixture_program("divide_trap.hum", source);
        let report = run_program(
            &program,
            Some("unchecked_divide"),
            &["10".to_string(), "0".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("division by zero while evaluating `a / b`".to_string())
        );
        assert!(
            report.diagnostics.is_empty(),
            "diagnostics: {:#?}",
            report.diagnostics
        );
    }

    #[test]
    fn writable_field_alias_reads_and_writes_through_to_the_owner() {
        let program = fixture_program(
            "examples/probes/writable_field_aliases.hum",
            include_str!("../examples/probes/writable_field_aliases.hum"),
        );
        let report = run_program(
            &program,
            Some("swap_xy_with_aliases"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Success("{x: 2, y: 1}".to_string())
        );
        assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    }

    #[test]
    fn writable_field_alias_overlap_traps_with_shared_h0808_blame() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_program8_overlap_write_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_program8_overlap_write_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("overlapping_write"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0808 writable alias overlap".to_string())
        );
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::WRITABLE_ALIAS_OVERLAP
        );
        let rendered = report.diagnostics[0].render();
        assert!(rendered.contains("alias_to_x"), "{rendered}");
        assert!(rendered.contains("point.x"), "{rendered}");
        assert!(rendered.contains(":13:5"), "{rendered}");
        assert!(rendered.contains(":14:5"), "{rendered}");
        assert!(rendered.contains(":15:5"), "{rendered}");
        assert!(
            rendered.contains("definitely distinct direct field"),
            "{rendered}"
        );
    }

    #[test]
    fn writable_field_alias_escape_traps_with_h0809() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_escape_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_escape_fail.hum"),
        );
        let report = run_program(&program, Some("escaped_alias"), &["{x:1,y:2}".to_string()]);
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS
        );
    }

    #[test]
    fn writable_alias_to_alias_traps_with_shared_h0809_fact() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_to_alias_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_to_alias_fail.hum"),
        );
        let report = run_program(&program, Some("alias_to_alias"), &["{x:1,y:2}".to_string()]);
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert_eq!(report.diagnostics.len(), 1);
        let rendered = report.diagnostics[0].render();
        assert!(rendered.contains("writable_alias_to_alias_binding_v0"));
        assert!(rendered.contains("writable alias `first`"));
    }

    #[test]
    fn writable_field_alias_requires_visible_mutation_authority() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_borrowed_owner_alias_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_borrowed_owner_alias_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("borrowed_owner_alias"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0802 borrowed parameter written".to_string())
        );
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::BORROW_PARAMETER_MUTATION
        );
    }

    #[test]
    fn writable_field_alias_permission_wrapper_traps_with_h0809() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_permission_wrapper_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_permission_wrapper_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("permission_wrapped_alias"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert!(
            report.diagnostics[0]
                .render()
                .contains("writable_alias_permission_wrapper_v0")
        );
    }

    #[test]
    fn writable_field_alias_cannot_rebind_its_owner() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("alias_rebinds_owner"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert!(
            report.diagnostics[0]
                .render()
                .contains("writable_alias_rebinds_its_owner_v0")
        );
    }

    #[test]
    fn writable_field_alias_cannot_shadow_an_existing_name() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_name_collision_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_name_collision_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("alias_name_collision"),
            &["{x:1,y:2}".to_string(), "7".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert!(
            report.diagnostics[0]
                .render()
                .contains("writable_alias_binding_rebinding_v0")
        );
    }

    #[test]
    fn writable_field_alias_cannot_shadow_a_declared_permission() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_alias_declared_name_collision_fail.hum",
            include_str!(
                "../fixtures/ownership_check/session_v_alias_declared_name_collision_fail.hum"
            ),
        );
        let report = run_program(
            &program,
            Some("alias_declared_name_collision"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0809 unsupported writable alias".to_string())
        );
        assert!(
            report.diagnostics[0]
                .render()
                .contains("writable_alias_binding_rebinding_v0")
        );
    }

    #[test]
    fn writable_alias_authority_precedes_overlap_consistently() {
        let program = fixture_program(
            "fixtures/ownership_check/session_v_borrowed_owner_overlap_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_borrowed_owner_overlap_fail.hum"),
        );
        let report = run_program(
            &program,
            Some("borrowed_owner_overlap"),
            &["{x:1,y:2}".to_string()],
        );
        assert_eq!(
            report.outcome,
            RunOutcome::Trap("H0802 borrowed parameter written".to_string())
        );
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::BORROW_PARAMETER_MUTATION
        );
    }

    #[test]
    fn predicate_preflight_aggregates_independent_predicate_and_chain_rows() {
        let parsed = parser::parse_source(
            "fixtures/full_type_check/session_af_predicate_v2_boundary_fail.hum",
            include_str!("../fixtures/full_type_check/session_af_predicate_v2_boundary_fail.hum"),
        );
        assert_eq!(
            parsed
                .diagnostics
                .iter()
                .filter(|diagnostic| {
                    diagnostic.code == DiagnosticCode::CHAINED_COMPARISON_NOT_SUPPORTED
                })
                .count(),
            1
        );
        let program = Program {
            files: vec![parsed.file],
        };
        let report = run_program(&program, Some("malformed_boundaries"), &[]);
        assert_eq!(report.outcome, RunOutcome::PreflightRejected);
        assert_eq!(report.diagnostics.len(), 18);
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| {
                    diagnostic.code == DiagnosticCode::INVALID_EXECUTABLE_PREDICATE
                })
                .count(),
            18
        );
        assert!(
            report.diagnostics.iter().all(|diagnostic| {
                diagnostic.code == DiagnosticCode::INVALID_EXECUTABLE_PREDICATE
            })
        );
    }

    #[test]
    fn reachable_predicate_preflight_precedes_output_adapter() {
        let program = fixture_program(
            "fixtures/run/session_af_predicate_v2_reachable_callee_fail.hum",
            include_str!("../fixtures/run/session_af_predicate_v2_reachable_callee_fail.hum"),
        );
        let mut output = RecordingOutput::default();
        let report = run_program_with_output(&program, None, &[], &allowed_stdout(), &mut output);
        assert_eq!(report.outcome, RunOutcome::PreflightRejected);
        assert_eq!(output.calls, 0);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::INVALID_EXECUTABLE_PREDICATE
        );
        assert!(report.diagnostics[0].render().contains("result = 1"));
    }

    #[test]
    fn passed_callable_runtime_depends_on_callable_identity_and_ordinary_value() {
        let cases = [
            ("increment", 41, "42"),
            ("increment", 40, "41"),
            ("double", 41, "82"),
        ];
        for (target, value, expected) in cases {
            let source = format!(
                r#"task increment(value: UInt) -> UInt {{
  does:
    return value + 1
}}

task double(value: UInt) -> UInt {{
  does:
    return value * 2
}}

task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {{
  does:
    return transform(value)
}}

task run -> UInt {{
  does:
    return apply_once({target}, {value})
}}
"#
            );
            let program = fixture_program("passed_callable_runtime.hum", &source);
            assert_eq!(
                run_program(&program, Some("run"), &[]).outcome,
                RunOutcome::Success(expected.to_string())
            );
        }
    }

    #[test]
    fn callable_preflight_rejects_before_output_adapter() {
        let program = fixture_program(
            "fixtures/callable/session_al_wrong_input_fail.hum",
            include_str!("../fixtures/callable/session_al_wrong_input_fail.hum"),
        );
        let mut output = RecordingOutput::default();
        let report = run_program_with_output(
            &program,
            Some("run"),
            &[],
            &OperatorGrantPolicy::default(),
            &mut output,
        );
        assert_eq!(report.outcome, RunOutcome::PreflightRejected);
        assert_eq!(output.calls, 0);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::CALLABLE_SIGNATURE_MISMATCH
        );

        let escaped = fixture_program(
            "fixtures/callable/session_al_nested_callable_escape_fail.hum",
            include_str!("../fixtures/callable/session_al_nested_callable_escape_fail.hum"),
        );
        let mut output = RecordingOutput::default();
        let report = run_program_with_output(
            &escaped,
            Some("apply_once"),
            &[],
            &OperatorGrantPolicy::default(),
            &mut output,
        );
        assert_eq!(report.outcome, RunOutcome::PreflightRejected);
        assert_eq!(output.calls, 0);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::INVALID_CALLABLE_FORM
        );
    }

    #[test]
    fn session_aq_static_runtime_causes_are_consumed_once_before_adapters() {
        let program = fixture_program(
            "fixtures/diagnostics/session_aq_static_runtime_shared_cause_fail.hum",
            include_str!("../fixtures/diagnostics/session_aq_static_runtime_shared_cause_fail.hum"),
        );
        let authority = runtime_occurrence_authority(&program).expect("runtime authority");

        let typed_occurrence = crate::typed_failure::analyze_program(&program)
            .occurrences()
            .into_iter()
            .find(|occurrence| occurrence.code == DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY)
            .expect("typed-failure occurrence");
        let mut collector =
            DiagnosticOccurrenceCollector::from_authority(&authority).expect("typed authority");
        collector
            .consume_exact(&typed_occurrence, typed_occurrence.diagnostic().clone())
            .expect("one typed consumption");
        assert_eq!(
            collector.consume_exact(&typed_occurrence, typed_occurrence.diagnostic().clone()),
            Err(DiagnosticInvariantError::DuplicateOccurrence)
        );

        let mut output = RecordingOutput::default();
        let typed = run_program_with_output(
            &program,
            Some("typed_failure_probe"),
            &[],
            &OperatorGrantPolicy::default(),
            &mut output,
        );
        assert_eq!(typed.outcome, RunOutcome::PreflightRejected);
        assert_eq!(
            typed.diagnostics,
            vec![typed_occurrence.diagnostic().clone()]
        );
        assert_eq!(output.calls, 0);

        let ownership_program = fixture_program(
            "fixtures/diagnostics/session_aq_static_runtime_shared_ownership_fail.hum",
            include_str!(
                "../fixtures/diagnostics/session_aq_static_runtime_shared_ownership_fail.hum"
            ),
        );
        let ownership_authority =
            runtime_occurrence_authority(&ownership_program).expect("ownership authority");
        let ownership = ownership_authority
            .normalized_occurrences()
            .into_iter()
            .find(|occurrence| {
                occurrence.owning_stage() == "ownership_check"
                    && occurrence.code == DiagnosticCode::USE_AFTER_MOVE
            })
            .expect("ownership occurrence");
        let mut collector = DiagnosticOccurrenceCollector::from_authority(&ownership_authority)
            .expect("ownership authority");
        let projection = ownership.diagnostic().clone();
        collector
            .consume_exact(ownership, projection.clone())
            .expect("one ownership consumption");
        assert_eq!(
            collector.consume_exact(ownership, projection),
            Err(DiagnosticInvariantError::DuplicateOccurrence)
        );

        let mut output = RecordingOutput::default();
        let ownership_report = run_program_with_output(
            &ownership_program,
            Some("ownership_probe"),
            &[],
            &OperatorGrantPolicy::default(),
            &mut output,
        );
        assert_eq!(
            ownership_report.outcome,
            RunOutcome::Trap("H0801 use after move".to_string())
        );
        assert_eq!(ownership_report.diagnostics.len(), 1);
        assert_eq!(
            ownership_report.diagnostics[0].code,
            DiagnosticCode::USE_AFTER_MOVE
        );
        assert_eq!(output.calls, 0);
    }

    #[test]
    fn session_aq_execution_time_use_after_move_survivor_is_internal_invariant() {
        let misuse_program = fixture_program(
            "session_aq_unreachable_moved_state_branches.hum",
            r#"task take(consume value: Int) -> Int {
  does:
    return value
}

task read_after_move() -> Int {
  does:
    let value: Int = 7
    let taken: Int = take(consume value)
    return value
}

task consume_after_move() -> Int {
  does:
    let value: Int = 7
    let taken: Int = take(consume value)
    return take(consume value)
}

task set_after_move() -> Int {
  does:
    change value: Int = 7
    let taken: Int = take(consume value)
    set value = 8
    return value
}
"#,
        );
        let misuse_authority =
            runtime_occurrence_authority(&misuse_program).expect("misuse authority");
        for entry in ["read_after_move", "consume_after_move", "set_after_move"] {
            let mut output = RecordingOutput::default();
            let mut replay = RecordingReplay::default();
            let mut locality = CountingLocality::default();
            let mut files = CountingFileRead::default();
            let report = run_program_with_occurrences_and_file_adapters(
                &misuse_program,
                &misuse_authority,
                Some(entry),
                &[],
                &OperatorGrantPolicy::default(),
                RunAdapters {
                    output: &mut output,
                    replay: &mut replay,
                    file_locality: &mut locality,
                    file: &mut files,
                },
            );
            assert_eq!(
                report.outcome,
                RunOutcome::Trap("H0801 use after move".to_string()),
                "ownership preflight must intercept {entry} before execution"
            );
            assert_eq!(report.diagnostics.len(), 1, "{entry}");
            assert_eq!(report.diagnostics[0].code, DiagnosticCode::USE_AFTER_MOVE);
            assert_eq!(output.calls, 0, "{entry}");
            assert_eq!(replay.calls, 0, "{entry}");
            assert_eq!(files.calls, 0, "{entry}");
            assert_eq!(locality.calls, 0, "{entry}");
        }

        let clean_program = fixture_program(
            "session_aq_successful_ownership_preflight.hum",
            r#"task clean(consume value: Int) -> Int {
  does:
    return value
}
"#,
        );
        let clean_authority =
            runtime_occurrence_authority(&clean_program).expect("clean runtime authority");
        let policy = OperatorGrantPolicy::default();
        let mut output = RecordingOutput::default();
        let mut replay = RecordingReplay::default();
        let mut locality = CountingLocality::default();
        let mut files = CountingFileRead::default();
        let report = run_program_with_occurrences_and_file_adapters(
            &clean_program,
            &clean_authority,
            Some("clean"),
            &[OsString::from("7")],
            &policy,
            RunAdapters {
                output: &mut output,
                replay: &mut replay,
                file_locality: &mut locality,
                file: &mut files,
            },
        );
        assert_eq!(report.outcome, RunOutcome::Success("7".to_string()));
        assert!(report.diagnostics.is_empty());

        let (branches, diagnostics) = probe_corrupted_moved_state_branches(
            &clean_program,
            &clean_authority,
            &policy,
            RunAdapters {
                output: &mut output,
                replay: &mut replay,
                file_locality: &mut locality,
                file: &mut files,
            },
        )
        .expect("test-only post-preflight moved-state corruption probe");
        assert_eq!(
            branches
                .iter()
                .map(|(branch, _)| *branch)
                .collect::<Vec<_>>(),
            ["read_value", "read_consume_value", "ensure_can_set"]
        );
        for (branch, message) in branches {
            assert!(
                message.starts_with("diagnostic invariant failure:"),
                "{branch}"
            );
            assert!(
                message.contains("ownership preflight allowed post-move access"),
                "{branch}"
            );
            assert!(!message.contains("H0801"), "{branch}");
        }
        assert!(diagnostics.is_empty());
        assert_eq!(output.calls, 0);
        assert!(output.writes.is_empty());
        assert_eq!(replay.calls, 0);
        assert_eq!(files.calls, 0);
        assert_eq!(locality.calls, 0);
    }

    #[test]
    fn session_aq_reachable_second_ownership_occurrence_is_consumed_exactly() {
        let program = fixture_program(
            "fixtures/diagnostics/session_aq_reachable_second_ownership_occurrence_fail.hum",
            include_str!(
                "../fixtures/diagnostics/session_aq_reachable_second_ownership_occurrence_fail.hum"
            ),
        );
        let authority = runtime_occurrence_authority(&program).expect("runtime authority");
        let mut ownership = authority
            .normalized_occurrences()
            .into_iter()
            .filter(|occurrence| {
                occurrence.owning_stage() == "ownership_check"
                    && occurrence.code == DiagnosticCode::USE_AFTER_MOVE
            })
            .cloned()
            .collect::<Vec<_>>();
        ownership.sort_by_key(|occurrence| {
            occurrence
                .diagnostic()
                .span
                .as_ref()
                .expect("ownership primary span")
                .line
        });
        assert_eq!(ownership.len(), 2);
        let first = &ownership[0];
        let second = &ownership[1];
        assert_ne!(first.id(), second.id());
        assert_eq!(
            second.diagnostic().message,
            "value `second` was used after it was moved"
        );
        assert_eq!(
            second.diagnostic().help.as_deref(),
            Some(
                "Fix task `reachable_second`: `second` moved at fixtures/diagnostics/session_aq_reachable_second_ownership_occurrence_fail.hum:38:7; use it before that move or create a fresh owned value."
            )
        );
        assert_eq!(
            second.diagnostic().span.as_ref().map(|span| span.line),
            Some(40)
        );
        assert!(second.diagnostic().related_spans.is_empty());

        let mut substitution =
            DiagnosticOccurrenceCollector::from_authority(&authority).expect("authority");
        assert_eq!(
            substitution.consume_exact(first, second.diagnostic().clone()),
            Err(DiagnosticInvariantError::DiagnosticProjectionMismatch)
        );
        for field in 0..7 {
            let mut projection = second.diagnostic().clone();
            match field {
                0 => projection.code = DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE,
                1 => projection.severity = Severity::Warning,
                2 => projection.message.push_str(" changed"),
                3 => projection.help = Some("wrong ownership repair".to_string()),
                4 => projection.span.as_mut().expect("H0801 span").column += 1,
                5 => projection
                    .related_spans
                    .push(crate::diagnostic::RelatedSpan {
                        label: "wrong move site".to_string(),
                        span: crate::diagnostic::Span::new("wrong.hum", 1, 1),
                    }),
                _ => {
                    projection.related_spans = vec![
                        crate::diagnostic::RelatedSpan {
                            label: "second".to_string(),
                            span: crate::diagnostic::Span::new("wrong.hum", 2, 1),
                        },
                        crate::diagnostic::RelatedSpan {
                            label: "first".to_string(),
                            span: crate::diagnostic::Span::new("wrong.hum", 1, 1),
                        },
                    ];
                }
            }
            assert_eq!(
                DiagnosticOccurrenceCollector::from_authority(&authority)
                    .expect("ownership authority")
                    .consume_exact(second, projection),
                Err(DiagnosticInvariantError::DiagnosticProjectionMismatch),
                "H0801 public field mutation {field} must fail closed"
            );
        }

        let authoritative = authority.occurrences().cloned().collect::<Vec<_>>();
        let mut reports = Vec::new();
        for reverse in [false, true] {
            let mut reordered = DiagnosticOccurrenceSet::default();
            let ordered = if reverse {
                authoritative.iter().rev().cloned().collect::<Vec<_>>()
            } else {
                authoritative.clone()
            };
            for occurrence in ordered {
                reordered
                    .insert_owned(occurrence)
                    .expect("unique occurrence");
            }
            let mut output = RecordingOutput::default();
            let mut replay = RecordingReplay::new(&[]);
            let report = run_program_with_occurrences_and_adapters(
                &program,
                &reordered,
                None,
                &[],
                &allowed_stdout(),
                &mut output,
                &mut replay,
            );
            assert_eq!(output.calls, 0, "ownership preflight must precede output");
            assert_eq!(
                report.outcome,
                RunOutcome::Trap("H0801 use after move".to_string())
            );
            assert_eq!(report.diagnostics, vec![second.diagnostic().clone()]);
            reports.push(report);
        }
        assert_eq!(reports[0], reports[1]);
    }

    #[test]
    fn session_aq_behavioral_legacy_classifier_witness_fails_wrong_occurrence() {
        let program = fixture_program(
            "fixtures/diagnostics/session_aq_reachable_second_ownership_occurrence_fail.hum",
            include_str!(
                "../fixtures/diagnostics/session_aq_reachable_second_ownership_occurrence_fail.hum"
            ),
        );
        let authority = runtime_occurrence_authority(&program).expect("runtime authority");
        let mut ownership = authority
            .normalized_occurrences()
            .into_iter()
            .filter(|occurrence| {
                occurrence.owning_stage() == "ownership_check"
                    && occurrence.code == DiagnosticCode::USE_AFTER_MOVE
            })
            .cloned()
            .collect::<Vec<_>>();
        ownership.sort_by_key(|occurrence| {
            occurrence
                .diagnostic()
                .span
                .as_ref()
                .expect("ownership span")
                .line
        });
        let [earlier, later] = ownership.as_slice() else {
            panic!("two H0801 witness occurrences");
        };
        let reachable_task = program
            .files
            .iter()
            .find_map(|file| super::find_task_in_items(&file.items, "reachable_second"))
            .expect("reachable task");
        let reachable_identity = crate::resolve::semantic_task_identity(&program, reachable_task);
        let canonical = crate::ownership_check::runtime_use_after_move_blockers(
            &program,
            &std::collections::BTreeSet::from([reachable_identity]),
        )
        .expect("canonical producer blockers");
        assert_eq!(canonical.len(), 1);
        assert_eq!(canonical[0].occurrence().id(), later.id());
        assert_ne!(earlier.id(), later.id());
        assert_ne!(earlier.semantic_origin(), later.semantic_origin());
        assert_ne!(earlier.relationship_route(), later.relationship_route());
        assert_eq!(earlier.semantic_owner(), later.semantic_owner());

        let legacy_code_first = ownership
            .iter()
            .find(|occurrence| occurrence.code == later.code)
            .expect("legacy code match");
        let legacy_prefix_first = ownership
            .iter()
            .find(|occurrence| {
                occurrence.origin_kind() == later.origin_kind()
                    && occurrence.owning_stage() == later.owning_stage()
            })
            .expect("legacy prefix match");
        let legacy_projection_first = ownership
            .iter()
            .find(|occurrence| {
                occurrence.diagnostic().code == later.diagnostic().code
                    && occurrence.diagnostic().severity == later.diagnostic().severity
            })
            .expect("legacy projection match");
        for legacy in [
            ownership.first().expect("legacy first match"),
            legacy_code_first,
            legacy_prefix_first,
            legacy_projection_first,
        ] {
            assert_eq!(legacy.id(), earlier.id());
            assert_ne!(legacy.id(), canonical[0].occurrence().id());
            assert!(
                DiagnosticOccurrenceCollector::from_authority(&authority)
                    .expect("legacy witness authority")
                    .consume_exact(legacy, later.diagnostic().clone())
                    .is_err(),
                "any legacy selector must fail the canonical consumption assertion"
            );
        }
    }

    #[test]
    fn session_aq_runtime_producer_substitutions_fail_closed() {
        let typed_program = fixture_program(
            "fixtures/diagnostics/session_aq_static_runtime_shared_cause_fail.hum",
            include_str!("../fixtures/diagnostics/session_aq_static_runtime_shared_cause_fail.hum"),
        );
        let typed_authority =
            runtime_occurrence_authority(&typed_program).expect("typed authority");
        let typed = crate::typed_failure::analyze_program(&typed_program)
            .occurrences()
            .into_iter()
            .next()
            .expect("typed occurrence");
        let mut typed_projection = typed.diagnostic().clone();
        typed_projection.severity = Severity::Warning;
        assert_eq!(
            DiagnosticOccurrenceCollector::from_authority(&typed_authority)
                .expect("typed collector")
                .consume_exact(&typed, typed_projection),
            Err(DiagnosticInvariantError::DiagnosticProjectionMismatch)
        );

        let callable_program = fixture_program(
            "fixtures/callable/session_al_wrong_input_fail.hum",
            include_str!("../fixtures/callable/session_al_wrong_input_fail.hum"),
        );
        let callable_authority =
            runtime_occurrence_authority(&callable_program).expect("callable authority");
        let callable = crate::callable::diagnostic_occurrences(&callable_program)
            .into_iter()
            .next()
            .expect("callable occurrence");
        let mut callable_projection = callable.diagnostic().clone();
        callable_projection.related_spans.reverse();
        if callable_projection.related_spans.len() < 2 {
            callable_projection.message.push_str(" changed");
        }
        assert_eq!(
            DiagnosticOccurrenceCollector::from_authority(&callable_authority)
                .expect("callable collector")
                .consume_exact(&callable, callable_projection),
            Err(DiagnosticInvariantError::DiagnosticProjectionMismatch)
        );

        let type_program = fixture_program(
            "session_aq_reachable_unknown_type_fail.hum",
            "task bad(value: MissingType) -> Int {\n  does:\n    return 0\n}\n",
        );
        let type_authority = runtime_occurrence_authority(&type_program).expect("type authority");
        let type_occurrence = crate::type_check::diagnostic_occurrence_set(&type_program, &[])
            .occurrences()
            .find(|occurrence| {
                occurrence.cause_key()
                    == crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(82)
            })
            .cloned()
            .expect("type occurrence");
        let mut type_projection = type_occurrence.diagnostic().clone();
        type_projection.help = Some("wrong type repair".to_string());
        assert_eq!(
            DiagnosticOccurrenceCollector::from_authority(&type_authority)
                .expect("type collector")
                .consume_exact(&type_occurrence, type_projection),
            Err(DiagnosticInvariantError::DiagnosticProjectionMismatch)
        );
        let type_report = run_program(&type_program, Some("bad"), &[]);
        assert_eq!(type_report.outcome, RunOutcome::PreflightRejected);
        assert_eq!(
            type_report.diagnostics,
            vec![type_occurrence.diagnostic().clone()]
        );

        let predicate_program = fixture_program(
            "fixtures/run/session_af_predicate_v2_reachable_callee_fail.hum",
            include_str!("../fixtures/run/session_af_predicate_v2_reachable_callee_fail.hum"),
        );
        let predicate_authority =
            runtime_occurrence_authority(&predicate_program).expect("predicate authority");
        let predicate_occurrence = crate::predicate::analyze_program(&predicate_program)
            .facts()
            .iter()
            .find_map(crate::predicate::PredicateFact::diagnostic_occurrence)
            .expect("predicate occurrence");
        let mut predicate_projection = predicate_occurrence.diagnostic().clone();
        predicate_projection
            .span
            .as_mut()
            .expect("predicate span")
            .column += 1;
        assert_eq!(
            DiagnosticOccurrenceCollector::from_authority(&predicate_authority)
                .expect("predicate collector")
                .consume_exact(&predicate_occurrence, predicate_projection),
            Err(DiagnosticInvariantError::DiagnosticProjectionMismatch)
        );
    }

    #[test]
    fn session_aq_same_code_and_app_scope_occurrences_remain_exact() {
        let same_code = fixture_program(
            "fixtures/diagnostics/session_aq_same_code_distinct_occurrences_fail.hum",
            include_str!(
                "../fixtures/diagnostics/session_aq_same_code_distinct_occurrences_fail.hum"
            ),
        );
        let authority = runtime_occurrence_authority(&same_code).expect("same-code authority");
        let matching = authority
            .normalized_occurrences()
            .into_iter()
            .filter(|occurrence| occurrence.code == DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 2);
        assert_ne!(matching[0].id(), matching[1].id());
        assert!(
            matching[0].diagnostic().span.as_ref().unwrap().line
                < matching[1].diagnostic().span.as_ref().unwrap().line
        );

        let app_scope = fixture_program(
            "fixtures/diagnostics/session_aq_app_scope_reanalysis_fail.hum",
            include_str!("../fixtures/diagnostics/session_aq_app_scope_reanalysis_fail.hum"),
        );
        let first = crate::capability_root::diagnostic_occurrence_set(&app_scope);
        let second = crate::capability_root::diagnostic_occurrence_set(&app_scope);
        assert_eq!(first, second);
        assert_eq!(first.occurrences().count(), 1);
        let occurrence = first.occurrences().next().expect("capability occurrence");
        assert!(!occurrence.relationship_route().is_empty());
        assert!(occurrence.resolver_call_occurrence().is_some());
    }

    fn fixture_program(path: &str, source: &str) -> Program {
        let parsed = parser::parse_source(path, source);
        let mut diagnostics = parsed.diagnostics;
        diagnostics.extend(check::check_file(&parsed.file));
        let errors = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "fixture diagnostics: {errors:#?}");
        Program {
            files: vec![parsed.file],
        }
    }
}
