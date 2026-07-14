use std::collections::{BTreeMap, BTreeSet};

use crate::app_entry;
use crate::ast::{App, Item, Program, Section, SectionLine, Task};
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOccurrence, Span};
use crate::graph::is_meaningful_line_text;
use crate::node_id;

pub(crate) const CAPABILITY_IDS: &[&str] = &["stdout.write", "clock.replay", "files.read"];

const CAPABILITY_LIKE_ROOTS: &[&str] = &[
    "stdout",
    "stderr",
    "clock",
    "time",
    "files",
    "file",
    "filesystem",
    "network",
    "socket",
    "random",
    "crypto",
    "env",
    "process",
    "os",
    "registry",
    "device",
    "sensor",
    "storage",
    "database",
    "http",
    "ffi",
    "unsafe",
    "import",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SourceCapability {
    StdoutWrite,
    ClockReplay,
    FilesRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CapabilitySpec {
    pub id: &'static str,
    pub core_effect: &'static str,
    pub runtime_target_meaning: &'static str,
    pub grant_kind: &'static str,
    pub grant_scope: &'static str,
    pub grant_strength: &'static str,
    pub grant_lifetime: &'static str,
    pub severity_tier: &'static str,
    pub mapping_status: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct CapabilityRouteFact {
    pub id: String,
    pub owner_task_identity: Option<String>,
    pub owner_task_span: Span,
    pub primary_span: Span,
    pub check: &'static str,
    pub status: &'static str,
    pub reason: Option<&'static str>,
    pub diagnostic_code: Option<&'static str>,
    pub capability_id: String,
    pub core_effect: Option<&'static str>,
    pub runtime_target_meaning: Option<&'static str>,
    pub grant_kind: Option<&'static str>,
    pub grant_scope: Option<&'static str>,
    pub grant_strength: Option<&'static str>,
    pub grant_lifetime: Option<&'static str>,
    pub severity_tier: Option<&'static str>,
    pub mapping_status: Option<&'static str>,
    pub app_name: Option<String>,
    pub caller: Option<String>,
    pub callee: Option<String>,
    pub app_span: Option<Span>,
    pub caller_span: Option<Span>,
    pub callee_span: Option<Span>,
    pub declaration_span: Option<Span>,
    pub entry_span: Option<Span>,
    pub route_tasks: Vec<String>,
    pub route_spans: Vec<Span>,
    pub help: Option<String>,
    resolver_call: Option<crate::resolve::ResolveCallOccurrenceSummary>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CapabilityAnalysis {
    pub diagnostics: Vec<Diagnostic>,
    pub routes: Vec<CapabilityRouteFact>,
    pub(crate) diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputPolicyFact {
    pub policy_id: String,
    pub app_name: Option<String>,
    pub task: String,
    pub call_span: Span,
    pub source_route: Vec<String>,
    pub source_route_spans: Vec<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReplayPolicyFact {
    pub policy_id: String,
    pub app_name: Option<String>,
    pub task: String,
    pub call_span: Span,
    pub source_route: Vec<String>,
    pub source_route_spans: Vec<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FilePolicyFact {
    pub policy_id: String,
    pub app_name: Option<String>,
    pub task: String,
    pub call_span: Span,
    pub source_route: Vec<String>,
    pub source_route_spans: Vec<Span>,
}

#[derive(Debug, Clone)]
struct Requirement {
    origin_task: String,
    declaration_span: Span,
    route_tasks: Vec<String>,
    route_spans: Vec<Span>,
    route_calls: Vec<crate::resolve::ResolveCallOccurrenceSummary>,
}

#[derive(Debug, Clone)]
struct UnknownRequirement {
    capability: String,
    origin_task: String,
    declaration_span: Span,
    route_tasks: Vec<String>,
    route_spans: Vec<Span>,
}

#[derive(Debug, Clone)]
struct CallEdge {
    callee: String,
    span: Span,
    resolver_call: crate::resolve::ResolveCallOccurrenceSummary,
}

struct TaskNode<'a> {
    task: &'a Task,
    capabilities: BTreeMap<SourceCapability, Span>,
    unknown_capabilities: Vec<(String, Span)>,
    calls: Vec<CallEdge>,
    output_calls: Vec<crate::resolve::ResolveCallOccurrenceSummary>,
    replay_calls: Vec<crate::resolve::ResolveCallOccurrenceSummary>,
    file_calls: Vec<crate::resolve::ResolveCallOccurrenceSummary>,
}

struct TaskGraph<'a> {
    tasks: BTreeMap<String, TaskNode<'a>>,
    order: Vec<String>,
}

#[derive(Debug, Clone)]
struct ReachableOutputRoute {
    task: String,
    call_span: Span,
    resolver_call: crate::resolve::ResolveCallOccurrenceSummary,
    task_route: Vec<String>,
    call_route: Vec<Span>,
}

#[derive(Debug, Clone)]
struct ReachableReplayRoute {
    task: String,
    call_span: Span,
    resolver_call: crate::resolve::ResolveCallOccurrenceSummary,
    task_route: Vec<String>,
    call_route: Vec<Span>,
}

#[derive(Debug, Clone)]
struct ReachableFileRoute {
    task: String,
    call_span: Span,
    resolver_call: crate::resolve::ResolveCallOccurrenceSummary,
    task_route: Vec<String>,
    call_route: Vec<Span>,
}

#[derive(Debug, Clone)]
struct OutputRecursionIssue {
    caller: String,
    callee: String,
    call_span: Span,
    task_route: Vec<String>,
    call_route: Vec<Span>,
}

impl SourceCapability {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "stdout.write" => Some(Self::StdoutWrite),
            "clock.replay" => Some(Self::ClockReplay),
            "files.read" => Some(Self::FilesRead),
            _ => None,
        }
    }

    pub(crate) fn spec(self) -> CapabilitySpec {
        match self {
            Self::StdoutWrite => CapabilitySpec {
                id: "stdout.write",
                core_effect: "output",
                runtime_target_meaning: "bounded_bootstrap_stdout_adapter_reserved_os.stdio",
                grant_kind: "output_stream",
                grant_scope: "app_stdout",
                grant_strength: "write",
                grant_lifetime: "one_run",
                severity_tier: "ordinary_external_authority",
                mapping_status: "implemented_bounded_output_v0_reserved_os.stdio_mapping",
            },
            Self::ClockReplay => CapabilitySpec {
                id: "clock.replay",
                core_effect: "time",
                runtime_target_meaning: "ordered_runner_replay_input_no_host_clock",
                grant_kind: "replay_input",
                grant_scope: "runner_tick_sequence",
                grant_strength: "read_ordered",
                grant_lifetime: "one_run",
                severity_tier: "ordinary_external_authority",
                mapping_status: "implemented_runner_replay_input_v0_no_os.clock",
            },
            Self::FilesRead => CapabilitySpec {
                id: "files.read",
                core_effect: "file",
                runtime_target_meaning: "one_exact_native_path_via_os.filesystem_adapter",
                grant_kind: "file",
                grant_scope: "exact_native_path",
                grant_strength: "read",
                grant_lifetime: "one_run",
                severity_tier: "ordinary_external_authority",
                mapping_status: "implemented_hardened_exact_file_read_v0_reserved_os.filesystem",
            },
        }
    }
}

pub(crate) fn analyze(program: &Program) -> CapabilityAnalysis {
    let mut analysis = match app_entry::analyze(program).entry {
        Some(entry) => analyze_app(program, entry.app, entry.task),
        None => analyze_unrooted_operations(program),
    };
    seal_analysis(program, &mut analysis);
    analysis
}

pub(crate) fn diagnostics(program: &Program) -> Vec<Diagnostic> {
    analyze(program).diagnostics
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diagnostic_occurrence_set(
    program: &Program,
) -> crate::diagnostic::DiagnosticOccurrenceSet {
    analyze(program).diagnostic_occurrences
}

pub(crate) fn output_policy_facts(program: &Program) -> Vec<OutputPolicyFact> {
    analyze(program)
        .routes
        .into_iter()
        .filter(|route| route.check == "source_capability_output_operation")
        .filter_map(|route| {
            Some(OutputPolicyFact {
                policy_id: route.id,
                app_name: route.app_name,
                task: route.caller?,
                call_span: route.primary_span,
                source_route: route.route_tasks,
                source_route_spans: route.route_spans,
            })
        })
        .collect()
}

pub(crate) fn replay_policy_facts(program: &Program) -> Vec<ReplayPolicyFact> {
    analyze(program)
        .routes
        .into_iter()
        .filter(|route| route.check == "source_capability_replay_operation")
        .filter_map(|route| {
            Some(ReplayPolicyFact {
                policy_id: route.id,
                app_name: route.app_name,
                task: route.caller?,
                call_span: route.primary_span,
                source_route: route.route_tasks,
                source_route_spans: route.route_spans,
            })
        })
        .collect()
}

pub(crate) fn file_policy_facts(program: &Program) -> Vec<FilePolicyFact> {
    analyze(program)
        .routes
        .into_iter()
        .filter(|route| route.check == "source_capability_file_operation")
        .filter_map(|route| {
            Some(FilePolicyFact {
                policy_id: route.id,
                app_name: route.app_name,
                task: route.caller?,
                call_span: route.primary_span,
                source_route: route.route_tasks,
                source_route_spans: route.route_spans,
            })
        })
        .collect()
}

pub(crate) fn entry_diagnostics(program: &Program, entry_name: &str) -> Vec<Diagnostic> {
    let Some((task, items)) = find_task_context(program, entry_name) else {
        return Vec::new();
    };
    let graph = build_task_graph(program, items);
    let Some(node) = graph.tasks.get(&task.name) else {
        return Vec::new();
    };
    if let Some(requirement) = reachable_unknown_requirement(&graph, &node.task.name) {
        let origin = &graph.tasks[&requirement.origin_task].task;
        let mut diagnostic = unknown_capability_diagnostic(
            &requirement.capability,
            &requirement.declaration_span,
            "task",
            &origin.name,
            &origin.span,
        );
        for (index, span) in requirement.route_spans.iter().enumerate() {
            diagnostic = diagnostic.with_related_span(
                format!(
                    "direct-entry authority route call {} for `{}`",
                    index + 1,
                    requirement.capability
                ),
                span.clone(),
            );
        }
        return one_entry_diagnostic(
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(91),
            format!("entry={}:origin={}", task.name, requirement.origin_task),
            requirement.route_tasks.clone(),
            diagnostic,
        );
    }
    if let Some((origin_task, call_span)) =
        reachable_output_call(&graph, &node.task.name, &mut BTreeSet::new())
        && !graph.tasks[&origin_task]
            .capabilities
            .contains_key(&SourceCapability::StdoutWrite)
    {
        let origin = graph.tasks[&origin_task].task;
        return one_entry_diagnostic(
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(95),
            format!("entry={}:output_task={origin_task}", task.name),
            vec![task.name.clone(), origin_task],
            missing_output_source_diagnostic(None, origin, &call_span, false, true),
        );
    }
    if let Some((origin_task, call_span)) =
        reachable_replay_call(&graph, &node.task.name, &mut BTreeSet::new())
        && !graph.tasks[&origin_task]
            .capabilities
            .contains_key(&SourceCapability::ClockReplay)
    {
        let origin = graph.tasks[&origin_task].task;
        return one_entry_diagnostic(
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(97),
            format!("entry={}:replay_task={origin_task}", task.name),
            vec![task.name.clone(), origin_task],
            missing_replay_source_diagnostic(None, origin, &call_span, false, true),
        );
    }
    if let Some((origin_task, call_span)) =
        reachable_file_call(&graph, &node.task.name, &mut BTreeSet::new())
        && !graph.tasks[&origin_task]
            .capabilities
            .contains_key(&SourceCapability::FilesRead)
    {
        let origin = graph.tasks[&origin_task].task;
        return one_entry_diagnostic(
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(99),
            format!("entry={}:file_task={origin_task}", task.name),
            vec![task.name.clone(), origin_task],
            missing_file_source_diagnostic(None, origin, &call_span, false, true),
        );
    }
    let closures = compute_closures(&graph);
    let Some((capability, requirement)) = closures
        .get(&task.name)
        .and_then(|requirements| requirements.iter().next())
    else {
        return Vec::new();
    };
    let spec = capability.spec();
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::ENTRY_CAPABILITY_BYPASS,
        format!(
            "direct entry task `{}` requires source capability `{}`; `--entry` is a pure probe and cannot carry external authority",
            task.name, spec.id
        ),
        Some(task.span.clone()),
    )
    .with_related_span(
        format!(
            "capability `{}` declared by task `{}`",
            spec.id, requirement.origin_task
        ),
        requirement.declaration_span.clone(),
    )
    .with_help(format!(
        "Run the structural app without `--entry`, keep `{}` declared through every caller and the app maximum, or select a task whose direct-call closure is pure.",
        spec.id
    ));
    for (index, span) in requirement.route_spans.iter().enumerate() {
        diagnostic = diagnostic.with_related_span(
            format!("authority route call {} for `{}`", index + 1, spec.id),
            span.clone(),
        );
    }
    one_entry_diagnostic(
        crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(94),
        format!(
            "entry={}:capability={}",
            crate::resolve::semantic_task_identity(program, task),
            spec.id
        ),
        vec![
            format!(
                "entry_task_identity={}",
                crate::resolve::semantic_task_identity(program, task)
            ),
            format!("entry_capability={}", spec.id),
        ],
        diagnostic,
    )
}

fn one_entry_diagnostic(
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    semantic_origin: String,
    relationship_route: Vec<String>,
    diagnostic: Diagnostic,
) -> Vec<Diagnostic> {
    let (diagnostic, occurrence) = DiagnosticOccurrence::producer_diagnostic(
        cause_key,
        diagnostic,
        semantic_origin,
        relationship_route,
    )
    .expect("entry capability diagnostic must have producer-owned identity");
    occurrence
        .validate()
        .expect("entry capability occurrence must validate");
    vec![diagnostic]
}

fn seal_analysis(_program: &Program, analysis: &mut CapabilityAnalysis) {
    let diagnostic_routes = analysis
        .routes
        .iter()
        .enumerate()
        .filter(|(_, route)| route.diagnostic_code.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        analysis.diagnostics.len(),
        diagnostic_routes.len(),
        "every capability diagnostic must leave with one analyzer-owned route"
    );
    for (diagnostic, (route_index, route)) in analysis.diagnostics.iter_mut().zip(diagnostic_routes)
    {
        let cause_key = match route.check {
            "source_capability_vocabulary" => {
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(91)
            }
            "source_capability_caller_closure" => {
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(92)
            }
            "source_capability_start_closure" => {
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(93)
            }
            "source_capability_output_operation" => {
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(95)
            }
            "source_capability_replay_operation" => {
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(97)
            }
            "source_capability_file_operation" => {
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(99)
            }
            "source_capability_output_recursion" => {
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(96)
            }
            "source_capability_replay_recursion" => {
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(98)
            }
            other => panic!("unregistered capability diagnostic route `{other}`"),
        };
        let owner_task_identity = route
            .owner_task_identity
            .as_deref()
            .expect("every diagnostic capability route must have a resolver-owned task");
        let semantic_origin = format!(
            "capability-occurrence:{owner_task_identity}:{}:{}:{route_index}",
            route.check, route.capability_id
        );
        let mut relationship_route = vec![
            format!("capability_check={}", route.check),
            format!("capability_status={}", route.status),
            format!("capability_id={}", route.capability_id),
            format!("capability_route_ordinal={route_index}"),
            format!("owner_task_identity={owner_task_identity}"),
        ];
        relationship_route.extend(
            route
                .route_tasks
                .iter()
                .enumerate()
                .map(|(index, _)| format!("capability_route_task_ordinal={index}")),
        );
        if let Some(resolver_call) = &route.resolver_call {
            relationship_route.extend(resolver_call.relationship_route());
        }
        let (sealed, mut occurrence) = DiagnosticOccurrence::producer_diagnostic(
            cause_key,
            diagnostic.clone(),
            semantic_origin,
            relationship_route,
        )
        .expect("capability route must select one exact registered cause");
        if let Some(resolver_call) = &route.resolver_call {
            occurrence = occurrence
                .with_resolver_call(resolver_call)
                .expect("capability occurrence must carry its exact resolver call");
        }
        *diagnostic = sealed;
        analysis
            .diagnostic_occurrences
            .insert_owned(occurrence)
            .expect("capability diagnostic occurrences must be unique");
    }
}

pub(crate) fn is_capability_diagnostic(diagnostic: &Diagnostic) -> bool {
    is_capability_code(diagnostic.code)
}

pub(crate) fn is_capability_code(code: DiagnosticCode) -> bool {
    matches!(
        code,
        DiagnosticCode::UNKNOWN_SOURCE_CAPABILITY
            | DiagnosticCode::MISSING_CALLER_CAPABILITY
            | DiagnosticCode::APP_CAPABILITY_MISMATCH
            | DiagnosticCode::ENTRY_CAPABILITY_BYPASS
            | DiagnosticCode::OUTPUT_CAPABILITY_UNDECLARED
            | DiagnosticCode::OUTPUT_RECURSION_UNSUPPORTED
            | DiagnosticCode::REPLAY_CAPABILITY_UNDECLARED
            | DiagnosticCode::REPLAY_RECURSION_UNSUPPORTED
            | DiagnosticCode::FILE_CAPABILITY_UNDECLARED
    )
}

fn analyze_app(program: &Program, app: &App, start: &Task) -> CapabilityAnalysis {
    let graph = build_task_graph(program, &app.items);
    let output_recursion = output_reachable_recursion(&graph, &start.name);
    let replay_recursion = replay_reachable_recursion(&graph, &start.name);
    let reachable_output_routes = reachable_output_routes(&graph, &start.name);
    let reachable_replay_routes = reachable_replay_routes(&graph, &start.name);
    let reachable_file_routes = reachable_file_routes(&graph, &start.name);
    let closures = compute_closures(&graph);
    let (app_capabilities, app_unknown) = declarations(&app.sections);
    let entry_span = start_declaration_span(app).unwrap_or_else(|| start.span.clone());
    let mut diagnostics = Vec::new();
    let mut routes = Vec::new();
    let output_recursion_has_complete_authority = app_capabilities
        .contains_key(&SourceCapability::StdoutWrite)
        && !reachable_output_routes.is_empty()
        && reachable_output_routes.iter().all(|route| {
            route.task_route.iter().all(|task_name| {
                graph.tasks.get(task_name).is_some_and(|node| {
                    node.capabilities
                        .contains_key(&SourceCapability::StdoutWrite)
                })
            })
        });
    let replay_recursion_has_complete_authority = app_capabilities
        .contains_key(&SourceCapability::ClockReplay)
        && !reachable_replay_routes.is_empty()
        && reachable_replay_routes.iter().all(|route| {
            route.task_route.iter().all(|task_name| {
                graph.tasks.get(task_name).is_some_and(|node| {
                    node.capabilities
                        .contains_key(&SourceCapability::ClockReplay)
                })
            })
        });

    if let Some(issue) = output_recursion
        .as_ref()
        .filter(|_| output_recursion_has_complete_authority)
    {
        let caller = &graph.tasks[&issue.caller].task;
        let callee = &graph.tasks[&issue.callee].task;
        let mut diagnostic = Diagnostic::error(
            DiagnosticCode::OUTPUT_RECURSION_UNSUPPORTED,
            format!(
                "recursive call from task `{}` to `{}` can reach `stdout_write`, but Session Z requires a finite exact output route",
                issue.caller, issue.callee
            ),
            Some(issue.call_span.clone()),
        )
        .with_related_span(
            format!("recursive caller task `{}`", issue.caller),
            caller.span.clone(),
        )
        .with_related_span(
            format!("re-entered task `{}`", issue.callee),
            callee.span.clone(),
        )
        .with_related_span(
            format!("structural app `{}`", app.name),
            app.span.clone(),
        )
        .with_help(
            "Rewrite this output-bearing recursion as an explicit bounded loop or a non-recursive task chain so every output exercise has one finite auditable route.",
        );
        for (index, span) in issue.call_route.iter().enumerate() {
            diagnostic = diagnostic.with_related_span(
                format!("output-recursion route call {}", index + 1),
                span.clone(),
            );
        }
        diagnostics.push(diagnostic);
        let mut route_tasks = vec![app.name.clone()];
        route_tasks.extend(issue.task_route.clone());
        routes.push(route_fact(
            program,
            "source_capability_output_recursion",
            "rejected_output_reachable_recursion_v0",
            Some("finite_output_route_required_for_forensic_replay_v0"),
            Some(DiagnosticCode::OUTPUT_RECURSION_UNSUPPORTED.as_str()),
            SourceCapability::StdoutWrite.spec(),
            caller.span.clone(),
            issue.call_span.clone(),
            Some(app),
            Some(caller),
            Some(callee),
            None,
            Some(entry_span.clone()),
            route_tasks,
            issue.call_route.clone(),
            Some(
                "Replace output-bearing recursion with an explicit bounded loop or non-recursive task chain."
                    .to_string(),
            ),
        ));
    }

    if let Some(issue) = replay_recursion
        .as_ref()
        .filter(|_| replay_recursion_has_complete_authority)
    {
        let caller = &graph.tasks[&issue.caller].task;
        let callee = &graph.tasks[&issue.callee].task;
        let mut diagnostic = Diagnostic::error(
            DiagnosticCode::REPLAY_RECURSION_UNSUPPORTED,
            format!(
                "recursive call from task `{}` to `{}` can reach `clock_replay_tick`, but Session AA requires a finite exact replay route",
                issue.caller, issue.callee
            ),
            Some(issue.call_span.clone()),
        )
        .with_related_span(
            format!("recursive caller task `{}`", issue.caller),
            caller.span.clone(),
        )
        .with_related_span(
            format!("re-entered task `{}`", issue.callee),
            callee.span.clone(),
        )
        .with_related_span(
            format!("structural app `{}`", app.name),
            app.span.clone(),
        )
        .with_help(
            "Rewrite this replay-bearing recursion as an explicit bounded loop or a non-recursive task chain so every replay exercise has one finite auditable route.",
        );
        for (index, span) in issue.call_route.iter().enumerate() {
            diagnostic = diagnostic.with_related_span(
                format!("replay-recursion route call {}", index + 1),
                span.clone(),
            );
        }
        diagnostics.push(diagnostic);
        let mut route_tasks = vec![app.name.clone()];
        route_tasks.extend(issue.task_route.clone());
        routes.push(route_fact(
            program,
            "source_capability_replay_recursion",
            "rejected_replay_reachable_recursion_v0",
            Some("finite_replay_route_required_for_forensic_replay_v0"),
            Some(DiagnosticCode::REPLAY_RECURSION_UNSUPPORTED.as_str()),
            SourceCapability::ClockReplay.spec(),
            caller.span.clone(),
            issue.call_span.clone(),
            Some(app),
            Some(caller),
            Some(callee),
            None,
            Some(entry_span.clone()),
            route_tasks,
            issue.call_route.clone(),
            Some(
                "Replace replay-bearing recursion with an explicit bounded loop or non-recursive task chain."
                    .to_string(),
            ),
        ));
    }

    for (capability, span) in app_unknown {
        diagnostics.push(unknown_capability_diagnostic(
            &capability,
            &span,
            "app",
            &app.name,
            &app.span,
        ));
        routes.push(unknown_route(
            program,
            &capability,
            &span,
            &start.span,
            Some(app),
            Some(start),
        ));
    }

    for task_name in &graph.order {
        let node = &graph.tasks[task_name];
        for (capability, span) in &node.unknown_capabilities {
            diagnostics.push(unknown_capability_diagnostic(
                capability,
                span,
                "task",
                &node.task.name,
                &node.task.span,
            ));
            routes.push(unknown_route(
                program,
                capability,
                span,
                &node.task.span,
                Some(app),
                Some(node.task),
            ));
        }
        for (capability, declaration_span) in &node.capabilities {
            routes.push(task_budget_route(
                program,
                app,
                node.task,
                *capability,
                declaration_span,
            ));
        }
        for resolver_call in &node.output_calls {
            let task_covers = node
                .capabilities
                .contains_key(&SourceCapability::StdoutWrite);
            let app_covers = app_capabilities.contains_key(&SourceCapability::StdoutWrite);
            if !task_covers || !app_covers {
                diagnostics.push(missing_output_source_diagnostic(
                    Some(app),
                    node.task,
                    &resolver_call.exact_call_span,
                    task_covers,
                    app_covers,
                ));
            }
        }
        for resolver_call in &node.replay_calls {
            let task_covers = node
                .capabilities
                .contains_key(&SourceCapability::ClockReplay);
            let app_covers = app_capabilities.contains_key(&SourceCapability::ClockReplay);
            if !task_covers || !app_covers {
                diagnostics.push(missing_replay_source_diagnostic(
                    Some(app),
                    node.task,
                    &resolver_call.exact_call_span,
                    task_covers,
                    app_covers,
                ));
            }
        }
        for resolver_call in &node.file_calls {
            let task_covers = node.capabilities.contains_key(&SourceCapability::FilesRead);
            let app_covers = app_capabilities.contains_key(&SourceCapability::FilesRead);
            if !task_covers || !app_covers {
                diagnostics.push(missing_file_source_diagnostic(
                    Some(app),
                    node.task,
                    &resolver_call.exact_call_span,
                    task_covers,
                    app_covers,
                ));
            }
        }
    }

    for output_route in &reachable_output_routes {
        let node = &graph.tasks[&output_route.task];
        let task_covers = node
            .capabilities
            .contains_key(&SourceCapability::StdoutWrite);
        let app_covers = app_capabilities.contains_key(&SourceCapability::StdoutWrite);
        let mut route_tasks = vec![app.name.clone()];
        route_tasks.extend(output_route.task_route.clone());
        routes.push(output_operation_route(
            program,
            Some(app),
            node.task,
            Some(&entry_span),
            &output_route.resolver_call,
            node.capabilities.get(&SourceCapability::StdoutWrite),
            task_covers,
            app_covers,
            route_tasks,
            output_route.call_route.clone(),
        ));
    }

    for replay_route in &reachable_replay_routes {
        let node = &graph.tasks[&replay_route.task];
        let task_covers = node
            .capabilities
            .contains_key(&SourceCapability::ClockReplay);
        let app_covers = app_capabilities.contains_key(&SourceCapability::ClockReplay);
        let mut route_tasks = vec![app.name.clone()];
        route_tasks.extend(replay_route.task_route.clone());
        routes.push(replay_operation_route(
            program,
            Some(app),
            node.task,
            Some(&entry_span),
            &replay_route.resolver_call,
            node.capabilities.get(&SourceCapability::ClockReplay),
            task_covers,
            app_covers,
            route_tasks,
            replay_route.call_route.clone(),
        ));
    }

    for file_route in &reachable_file_routes {
        let node = &graph.tasks[&file_route.task];
        let task_covers = node.capabilities.contains_key(&SourceCapability::FilesRead);
        let app_covers = app_capabilities.contains_key(&SourceCapability::FilesRead);
        let mut route_tasks = vec![app.name.clone()];
        route_tasks.extend(file_route.task_route.clone());
        routes.push(file_operation_route(
            program,
            Some(app),
            node.task,
            Some(&entry_span),
            &file_route.resolver_call,
            node.capabilities.get(&SourceCapability::FilesRead),
            task_covers,
            app_covers,
            route_tasks,
            file_route.call_route.clone(),
        ));
    }

    for task_name in &graph.order {
        let node = &graph.tasks[task_name];
        for resolver_call in &node.output_calls {
            let call_span = &resolver_call.exact_call_span;
            let is_reachable = reachable_output_routes
                .iter()
                .any(|route| route.task == node.task.name && route.call_span == *call_span);
            if is_reachable {
                continue;
            }
            let task_covers = node
                .capabilities
                .contains_key(&SourceCapability::StdoutWrite);
            let app_covers = app_capabilities.contains_key(&SourceCapability::StdoutWrite);
            routes.push(output_operation_route(
                program,
                Some(app),
                node.task,
                Some(&entry_span),
                resolver_call,
                node.capabilities.get(&SourceCapability::StdoutWrite),
                task_covers,
                app_covers,
                vec![app.name.clone(), node.task.name.clone()],
                vec![call_span.clone()],
            ));
        }
        for resolver_call in &node.replay_calls {
            let call_span = &resolver_call.exact_call_span;
            let is_reachable = reachable_replay_routes
                .iter()
                .any(|route| route.task == node.task.name && route.call_span == *call_span);
            if is_reachable {
                continue;
            }
            let task_covers = node
                .capabilities
                .contains_key(&SourceCapability::ClockReplay);
            let app_covers = app_capabilities.contains_key(&SourceCapability::ClockReplay);
            routes.push(replay_operation_route(
                program,
                Some(app),
                node.task,
                Some(&entry_span),
                resolver_call,
                node.capabilities.get(&SourceCapability::ClockReplay),
                task_covers,
                app_covers,
                vec![app.name.clone(), node.task.name.clone()],
                vec![call_span.clone()],
            ));
        }
        for resolver_call in &node.file_calls {
            let call_span = &resolver_call.exact_call_span;
            let is_reachable = reachable_file_routes
                .iter()
                .any(|route| route.task == node.task.name && route.call_span == *call_span);
            if is_reachable {
                continue;
            }
            let task_covers = node.capabilities.contains_key(&SourceCapability::FilesRead);
            let app_covers = app_capabilities.contains_key(&SourceCapability::FilesRead);
            routes.push(file_operation_route(
                program,
                Some(app),
                node.task,
                Some(&entry_span),
                resolver_call,
                node.capabilities.get(&SourceCapability::FilesRead),
                task_covers,
                app_covers,
                vec![app.name.clone(), node.task.name.clone()],
                vec![call_span.clone()],
            ));
        }
    }

    for (capability, declaration_span) in &app_capabilities {
        routes.push(app_budget_route(
            program,
            app,
            start,
            &entry_span,
            *capability,
            declaration_span,
        ));
    }

    for task_name in &graph.order {
        let caller = &graph.tasks[task_name];
        for call in &caller.calls {
            let callee = &graph.tasks[&call.callee];
            let Some(callee_requirements) = closures.get(&callee.task.name) else {
                continue;
            };
            for (capability, requirement) in callee_requirements {
                let covered = caller.capabilities.contains_key(capability);
                let route = prepend_route(&caller.task.name, call, requirement);
                if !covered {
                    diagnostics.push(missing_caller_diagnostic(
                        caller.task,
                        callee.task,
                        *capability,
                        &call.span,
                        &route,
                    ));
                }
                routes.push(caller_route(
                    program,
                    app,
                    caller.task,
                    callee.task,
                    *capability,
                    &route,
                    covered,
                ));
            }
        }
    }

    if let Some(start_requirements) = closures.get(&start.name) {
        for (capability, requirement) in start_requirements {
            let app_covers = app_capabilities.contains_key(capability);
            let task_route_complete = route_is_declared(&graph, *capability, requirement);
            let operation_call_owns_missing_app = !app_covers
                && match capability {
                    SourceCapability::StdoutWrite => graph
                        .tasks
                        .values()
                        .any(|node| !node.output_calls.is_empty()),
                    SourceCapability::ClockReplay => graph
                        .tasks
                        .values()
                        .any(|node| !node.replay_calls.is_empty()),
                    SourceCapability::FilesRead => {
                        graph.tasks.values().any(|node| !node.file_calls.is_empty())
                    }
                };
            if !app_covers && task_route_complete && !operation_call_owns_missing_app {
                diagnostics.push(app_mismatch_diagnostic(
                    app,
                    start,
                    &entry_span,
                    *capability,
                    requirement,
                ));
            }
            routes.push(app_closure_route(
                program,
                app,
                start,
                &entry_span,
                *capability,
                requirement,
                app_covers,
                task_route_complete,
            ));
        }
    }

    CapabilityAnalysis {
        diagnostics,
        routes,
        diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet::default(),
    }
}

fn analyze_unrooted_operations(program: &Program) -> CapabilityAnalysis {
    let mut analysis = CapabilityAnalysis::default();
    for file in &program.files {
        let graph = build_task_graph(program, &file.items);
        for task_name in &graph.order {
            let node = &graph.tasks[task_name];
            for resolver_call in &node.output_calls {
                let call_span = &resolver_call.exact_call_span;
                let task_covers = node
                    .capabilities
                    .contains_key(&SourceCapability::StdoutWrite);
                analysis.diagnostics.push(missing_output_source_diagnostic(
                    None,
                    node.task,
                    call_span,
                    task_covers,
                    false,
                ));
                analysis.routes.push(output_operation_route(
                    program,
                    None,
                    node.task,
                    None,
                    resolver_call,
                    node.capabilities.get(&SourceCapability::StdoutWrite),
                    task_covers,
                    false,
                    vec![node.task.name.clone()],
                    vec![call_span.clone()],
                ));
            }
            for resolver_call in &node.replay_calls {
                let call_span = &resolver_call.exact_call_span;
                let task_covers = node
                    .capabilities
                    .contains_key(&SourceCapability::ClockReplay);
                analysis.diagnostics.push(missing_replay_source_diagnostic(
                    None,
                    node.task,
                    call_span,
                    task_covers,
                    false,
                ));
                analysis.routes.push(replay_operation_route(
                    program,
                    None,
                    node.task,
                    None,
                    resolver_call,
                    node.capabilities.get(&SourceCapability::ClockReplay),
                    task_covers,
                    false,
                    vec![node.task.name.clone()],
                    vec![call_span.clone()],
                ));
            }
            for resolver_call in &node.file_calls {
                let call_span = &resolver_call.exact_call_span;
                let task_covers = node.capabilities.contains_key(&SourceCapability::FilesRead);
                analysis.diagnostics.push(missing_file_source_diagnostic(
                    None,
                    node.task,
                    call_span,
                    task_covers,
                    false,
                ));
                analysis.routes.push(file_operation_route(
                    program,
                    None,
                    node.task,
                    None,
                    resolver_call,
                    node.capabilities.get(&SourceCapability::FilesRead),
                    task_covers,
                    false,
                    vec![node.task.name.clone()],
                    vec![call_span.clone()],
                ));
            }
        }
    }
    analysis
}

fn build_task_graph<'a>(program: &Program, items: &'a [Item]) -> TaskGraph<'a> {
    let mut tasks = BTreeMap::new();
    let mut order = Vec::new();
    for item in items {
        let Item::Task(task) = item else {
            continue;
        };
        let (capabilities, unknown_capabilities) = declarations(&task.sections);
        if !tasks.contains_key(&task.name) {
            order.push(task.name.clone());
            tasks.insert(
                task.name.clone(),
                TaskNode {
                    task,
                    capabilities,
                    unknown_capabilities,
                    calls: Vec::new(),
                    output_calls: Vec::new(),
                    replay_calls: Vec::new(),
                    file_calls: Vec::new(),
                },
            );
        }
    }
    let target_tasks = tasks
        .values()
        .map(|node| {
            (
                crate::resolve::semantic_task_definition_identity(program, node.task),
                node.task.name.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let resolver_calls = crate::resolve::resolve_call_occurrence_summaries(program, &[]);
    for task_name in &order {
        let task = tasks[task_name].task;
        let owner_definition_id = crate::resolve::semantic_task_definition_identity(program, task);
        let mut calls = Vec::new();
        let mut output_calls = Vec::new();
        let mut replay_calls = Vec::new();
        let mut file_calls = Vec::new();
        for resolver_call in resolver_calls
            .iter()
            .filter(|call| call.owner_definition_id == owner_definition_id)
        {
            match resolver_call.target_definition_id.as_str() {
                "builtin_stdout_write" => output_calls.push(resolver_call.clone()),
                "builtin_clock_replay_tick" => replay_calls.push(resolver_call.clone()),
                "builtin_files_read_text" => file_calls.push(resolver_call.clone()),
                target => {
                    if let Some(callee) = target_tasks.get(target) {
                        calls.push(CallEdge {
                            callee: callee.clone(),
                            span: resolver_call.exact_call_span.clone(),
                            resolver_call: resolver_call.clone(),
                        });
                    }
                }
            }
        }
        let node = tasks.get_mut(task_name).expect("task exists");
        node.calls = calls;
        node.output_calls = output_calls;
        node.replay_calls = replay_calls;
        node.file_calls = file_calls;
    }
    TaskGraph { tasks, order }
}

fn reachable_output_routes(graph: &TaskGraph<'_>, start: &str) -> Vec<ReachableOutputRoute> {
    let mut routes = Vec::new();
    let mut active = BTreeSet::new();
    let mut task_route = vec![start.to_string()];
    let mut call_route = Vec::new();
    collect_reachable_output_routes(
        graph,
        start,
        &mut active,
        &mut task_route,
        &mut call_route,
        &mut routes,
    );
    routes
}

fn output_reachable_recursion(graph: &TaskGraph<'_>, start: &str) -> Option<OutputRecursionIssue> {
    operation_reachable_recursion(graph, start, SourceCapability::StdoutWrite)
}

fn replay_reachable_recursion(graph: &TaskGraph<'_>, start: &str) -> Option<OutputRecursionIssue> {
    operation_reachable_recursion(graph, start, SourceCapability::ClockReplay)
}

fn operation_reachable_recursion(
    graph: &TaskGraph<'_>,
    start: &str,
    capability: SourceCapability,
) -> Option<OutputRecursionIssue> {
    find_operation_reachable_recursion(
        graph,
        start,
        capability,
        &mut vec![start.to_string()],
        &mut Vec::new(),
    )
}

fn find_operation_reachable_recursion(
    graph: &TaskGraph<'_>,
    task_name: &str,
    capability: SourceCapability,
    task_route: &mut Vec<String>,
    call_route: &mut Vec<Span>,
) -> Option<OutputRecursionIssue> {
    let node = graph.tasks.get(task_name)?;
    for call in &node.calls {
        if task_route.contains(&call.callee) {
            if task_reaches_operation(graph, &call.callee, capability, &mut BTreeSet::new()) {
                let mut recursive_tasks = task_route.clone();
                recursive_tasks.push(call.callee.clone());
                let mut recursive_calls = call_route.clone();
                recursive_calls.push(call.span.clone());
                return Some(OutputRecursionIssue {
                    caller: task_name.to_string(),
                    callee: call.callee.clone(),
                    call_span: call.span.clone(),
                    task_route: recursive_tasks,
                    call_route: recursive_calls,
                });
            }
            continue;
        }
        task_route.push(call.callee.clone());
        call_route.push(call.span.clone());
        let issue = find_operation_reachable_recursion(
            graph,
            &call.callee,
            capability,
            task_route,
            call_route,
        );
        call_route.pop();
        task_route.pop();
        if issue.is_some() {
            return issue;
        }
    }
    None
}

fn task_reaches_operation(
    graph: &TaskGraph<'_>,
    task_name: &str,
    capability: SourceCapability,
    visited: &mut BTreeSet<String>,
) -> bool {
    if !visited.insert(task_name.to_string()) {
        return false;
    }
    let Some(node) = graph.tasks.get(task_name) else {
        return false;
    };
    let has_direct_operation = match capability {
        SourceCapability::StdoutWrite => !node.output_calls.is_empty(),
        SourceCapability::ClockReplay => !node.replay_calls.is_empty(),
        SourceCapability::FilesRead => !node.file_calls.is_empty(),
    };
    has_direct_operation
        || node
            .calls
            .iter()
            .any(|call| task_reaches_operation(graph, &call.callee, capability, visited))
}

fn collect_reachable_output_routes(
    graph: &TaskGraph<'_>,
    task_name: &str,
    active: &mut BTreeSet<String>,
    task_route: &mut Vec<String>,
    call_route: &mut Vec<Span>,
    routes: &mut Vec<ReachableOutputRoute>,
) {
    if !active.insert(task_name.to_string()) {
        return;
    }
    let Some(node) = graph.tasks.get(task_name) else {
        active.remove(task_name);
        return;
    };
    for resolver_call in &node.output_calls {
        let call_span = &resolver_call.exact_call_span;
        let mut route_spans = call_route.clone();
        route_spans.push(call_span.clone());
        routes.push(ReachableOutputRoute {
            task: task_name.to_string(),
            call_span: call_span.clone(),
            resolver_call: resolver_call.clone(),
            task_route: task_route.clone(),
            call_route: route_spans,
        });
    }
    for call in &node.calls {
        task_route.push(call.callee.clone());
        call_route.push(call.span.clone());
        collect_reachable_output_routes(
            graph,
            &call.callee,
            active,
            task_route,
            call_route,
            routes,
        );
        call_route.pop();
        task_route.pop();
    }
    active.remove(task_name);
}

fn reachable_replay_routes(graph: &TaskGraph<'_>, start: &str) -> Vec<ReachableReplayRoute> {
    let mut routes = Vec::new();
    let mut active = BTreeSet::new();
    let mut task_route = vec![start.to_string()];
    let mut call_route = Vec::new();
    collect_reachable_replay_routes(
        graph,
        start,
        &mut active,
        &mut task_route,
        &mut call_route,
        &mut routes,
    );
    routes
}

fn collect_reachable_replay_routes(
    graph: &TaskGraph<'_>,
    task_name: &str,
    active: &mut BTreeSet<String>,
    task_route: &mut Vec<String>,
    call_route: &mut Vec<Span>,
    routes: &mut Vec<ReachableReplayRoute>,
) {
    if !active.insert(task_name.to_string()) {
        return;
    }
    let Some(node) = graph.tasks.get(task_name) else {
        active.remove(task_name);
        return;
    };
    for resolver_call in &node.replay_calls {
        let call_span = &resolver_call.exact_call_span;
        let mut route_spans = call_route.clone();
        route_spans.push(call_span.clone());
        routes.push(ReachableReplayRoute {
            task: task_name.to_string(),
            call_span: call_span.clone(),
            resolver_call: resolver_call.clone(),
            task_route: task_route.clone(),
            call_route: route_spans,
        });
    }
    for call in &node.calls {
        task_route.push(call.callee.clone());
        call_route.push(call.span.clone());
        collect_reachable_replay_routes(
            graph,
            &call.callee,
            active,
            task_route,
            call_route,
            routes,
        );
        call_route.pop();
        task_route.pop();
    }
    active.remove(task_name);
}

fn reachable_file_routes(graph: &TaskGraph<'_>, start: &str) -> Vec<ReachableFileRoute> {
    let mut routes = Vec::new();
    let mut active = BTreeSet::new();
    let mut task_route = vec![start.to_string()];
    let mut call_route = Vec::new();
    collect_reachable_file_routes(
        graph,
        start,
        &mut active,
        &mut task_route,
        &mut call_route,
        &mut routes,
    );
    routes
}

fn collect_reachable_file_routes(
    graph: &TaskGraph<'_>,
    task_name: &str,
    active: &mut BTreeSet<String>,
    task_route: &mut Vec<String>,
    call_route: &mut Vec<Span>,
    routes: &mut Vec<ReachableFileRoute>,
) {
    if !active.insert(task_name.to_string()) {
        return;
    }
    let Some(node) = graph.tasks.get(task_name) else {
        active.remove(task_name);
        return;
    };
    for resolver_call in &node.file_calls {
        let call_span = &resolver_call.exact_call_span;
        let mut route_spans = call_route.clone();
        route_spans.push(call_span.clone());
        routes.push(ReachableFileRoute {
            task: task_name.to_string(),
            call_span: call_span.clone(),
            resolver_call: resolver_call.clone(),
            task_route: task_route.clone(),
            call_route: route_spans,
        });
    }
    for call in &node.calls {
        task_route.push(call.callee.clone());
        call_route.push(call.span.clone());
        collect_reachable_file_routes(graph, &call.callee, active, task_route, call_route, routes);
        call_route.pop();
        task_route.pop();
    }
    active.remove(task_name);
}

fn reachable_unknown_requirement(
    graph: &TaskGraph<'_>,
    task_name: &str,
) -> Option<UnknownRequirement> {
    reachable_unknown_from(graph, task_name, &mut BTreeSet::new())
}

fn reachable_unknown_from(
    graph: &TaskGraph<'_>,
    task_name: &str,
    path: &mut BTreeSet<String>,
) -> Option<UnknownRequirement> {
    let node = graph.tasks.get(task_name)?;
    if let Some((capability, declaration_span)) = node.unknown_capabilities.first() {
        return Some(UnknownRequirement {
            capability: capability.clone(),
            origin_task: node.task.name.clone(),
            declaration_span: declaration_span.clone(),
            route_tasks: vec![node.task.name.clone()],
            route_spans: Vec::new(),
        });
    }
    if !path.insert(task_name.to_string()) {
        return None;
    }
    for call in &node.calls {
        if let Some(mut requirement) = reachable_unknown_from(graph, &call.callee, path) {
            requirement.route_tasks.insert(0, task_name.to_string());
            requirement.route_spans.insert(0, call.span.clone());
            path.remove(task_name);
            return Some(requirement);
        }
    }
    path.remove(task_name);
    None
}

fn reachable_output_call(
    graph: &TaskGraph<'_>,
    task_name: &str,
    path: &mut BTreeSet<String>,
) -> Option<(String, Span)> {
    let node = graph.tasks.get(task_name)?;
    if let Some(call) = node.output_calls.first() {
        return Some((node.task.name.clone(), call.exact_call_span.clone()));
    }
    if !path.insert(task_name.to_string()) {
        return None;
    }
    for call in &node.calls {
        if let Some(found) = reachable_output_call(graph, &call.callee, path) {
            path.remove(task_name);
            return Some(found);
        }
    }
    path.remove(task_name);
    None
}

fn reachable_replay_call(
    graph: &TaskGraph<'_>,
    task_name: &str,
    path: &mut BTreeSet<String>,
) -> Option<(String, Span)> {
    let node = graph.tasks.get(task_name)?;
    if let Some(call) = node.replay_calls.first() {
        return Some((node.task.name.clone(), call.exact_call_span.clone()));
    }
    if !path.insert(task_name.to_string()) {
        return None;
    }
    for call in &node.calls {
        if let Some(found) = reachable_replay_call(graph, &call.callee, path) {
            path.remove(task_name);
            return Some(found);
        }
    }
    path.remove(task_name);
    None
}

fn reachable_file_call(
    graph: &TaskGraph<'_>,
    task_name: &str,
    visited: &mut BTreeSet<String>,
) -> Option<(String, Span)> {
    if !visited.insert(task_name.to_string()) {
        return None;
    }
    let node = graph.tasks.get(task_name)?;
    if let Some(call) = node.file_calls.first() {
        return Some((task_name.to_string(), call.exact_call_span.clone()));
    }
    for call in &node.calls {
        if let Some(found) = reachable_file_call(graph, &call.callee, visited) {
            return Some(found);
        }
    }
    None
}

fn compute_closures(
    graph: &TaskGraph<'_>,
) -> BTreeMap<String, BTreeMap<SourceCapability, Requirement>> {
    let mut closures = BTreeMap::new();
    for task_name in &graph.order {
        let node = &graph.tasks[task_name];
        let direct = node
            .capabilities
            .iter()
            .map(|(capability, span)| {
                (
                    *capability,
                    Requirement {
                        origin_task: node.task.name.clone(),
                        declaration_span: span.clone(),
                        route_tasks: vec![node.task.name.clone()],
                        route_spans: Vec::new(),
                        route_calls: Vec::new(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        closures.insert(task_name.clone(), direct);
    }

    loop {
        let snapshot = closures.clone();
        let mut changed = false;
        for task_name in &graph.order {
            let node = &graph.tasks[task_name];
            for call in &node.calls {
                let Some(callee_requirements) = snapshot.get(&call.callee) else {
                    continue;
                };
                for (capability, requirement) in callee_requirements {
                    if closures[task_name].contains_key(capability) {
                        continue;
                    }
                    closures
                        .get_mut(task_name)
                        .expect("task closure exists")
                        .insert(*capability, prepend_route(task_name, call, requirement));
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    closures
}

fn prepend_route(caller: &str, call: &CallEdge, requirement: &Requirement) -> Requirement {
    let mut route_tasks = vec![caller.to_string()];
    route_tasks.extend(requirement.route_tasks.iter().cloned());
    let mut route_spans = vec![call.span.clone()];
    route_spans.extend(requirement.route_spans.iter().cloned());
    let mut route_calls = vec![call.resolver_call.clone()];
    route_calls.extend(requirement.route_calls.iter().cloned());
    Requirement {
        origin_task: requirement.origin_task.clone(),
        declaration_span: requirement.declaration_span.clone(),
        route_tasks,
        route_spans,
        route_calls,
    }
}

fn route_is_declared(
    graph: &TaskGraph<'_>,
    capability: SourceCapability,
    requirement: &Requirement,
) -> bool {
    requirement.route_tasks.iter().all(|task_name| {
        graph
            .tasks
            .get(task_name)
            .is_some_and(|task| task.capabilities.contains_key(&capability))
    })
}

fn declarations(sections: &[Section]) -> (BTreeMap<SourceCapability, Span>, Vec<(String, Span)>) {
    let mut capabilities = BTreeMap::new();
    let mut unknown = Vec::new();
    for line in meaningful_section_lines(sections, "uses") {
        let text = line.text.trim();
        if let Some(capability) = SourceCapability::parse(text) {
            capabilities
                .entry(capability)
                .or_insert_with(|| line.span.clone());
        } else if is_capability_like(text) {
            unknown.push((text.to_string(), line.span.clone()));
        }
    }
    (capabilities, unknown)
}

fn meaningful_section_lines<'a>(
    sections: &'a [Section],
    name: &str,
) -> impl Iterator<Item = &'a SectionLine> {
    sections
        .iter()
        .filter(move |section| section.name == name)
        .flat_map(|section| &section.lines)
        .filter(|line| is_meaningful_line_text(&line.text))
}

fn is_capability_like(text: &str) -> bool {
    let mut parts = text.split('.');
    let Some(root) = parts.next() else {
        return false;
    };
    let rest = parts.collect::<Vec<_>>();
    !rest.is_empty()
        && CAPABILITY_LIKE_ROOTS.contains(&root)
        && is_value_identifier(root)
        && rest
            .iter()
            .all(|part| *part == "*" || is_value_identifier(part))
}

fn start_declaration_span(app: &App) -> Option<Span> {
    meaningful_section_lines(&app.sections, "starts with")
        .next()
        .map(|line| line.span.clone())
}

fn unknown_capability_diagnostic(
    capability: &str,
    span: &Span,
    owner_kind: &str,
    owner_name: &str,
    owner_span: &Span,
) -> Diagnostic {
    let sandbox_bypass = is_sandbox_bypass(capability);
    Diagnostic::error(
        DiagnosticCode::UNKNOWN_SOURCE_CAPABILITY,
        format!(
            "{owner_kind} `{owner_name}` uses unknown executable source capability `{capability}`{}",
            if sandbox_bypass {
                "; sandbox-bypass authority is not an ordinary grant"
            } else {
                ""
            }
        ),
        Some(span.clone()),
    )
    .with_related_span(format!("{owner_kind} `{owner_name}` authority boundary"), owner_span.clone())
    .with_help(if sandbox_bypass {
        format!(
            "Remove `{capability}`; process, FFI, unsafe, and unrestricted-import authority require a separate sandbox-bypass severity tier and are not authorized in this work order. Ordinary exact IDs are `{}`.",
            CAPABILITY_IDS.join("`, `")
        )
    } else {
        format!(
            "Use exactly one pinned capability (`{}`), or keep an ordinary dependency under `uses:` without an external capability-family spelling.",
            CAPABILITY_IDS.join("`, `")
        )
    })
}

fn missing_caller_diagnostic(
    caller: &Task,
    callee: &Task,
    capability: SourceCapability,
    call_span: &Span,
    route: &Requirement,
) -> Diagnostic {
    let spec = capability.spec();
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::MISSING_CALLER_CAPABILITY,
        format!(
            "task `{}` calls `{}` whose closure requires `{}`, but the caller does not declare that authority budget",
            caller.name, callee.name, spec.id
        ),
        Some(call_span.clone()),
    )
    .with_related_span(format!("caller task `{}`", caller.name), caller.span.clone())
    .with_related_span(format!("callee task `{}`", callee.name), callee.span.clone())
    .with_related_span(
        format!(
            "origin declaration `{}` on task `{}`",
            spec.id, route.origin_task
        ),
        route.declaration_span.clone(),
    )
    .with_help(format!(
        "Add exact `{}` under task `{}`'s `uses:` section, or remove/restructure the call so this caller no longer reaches that authority budget.",
        spec.id, caller.name
    ));
    for (index, span) in route.route_spans.iter().skip(1).enumerate() {
        diagnostic = diagnostic.with_related_span(
            format!(
                "transitive call {} toward `{}`",
                index + 1,
                route.origin_task
            ),
            span.clone(),
        );
    }
    diagnostic
}

fn app_mismatch_diagnostic(
    app: &App,
    start: &Task,
    entry_span: &Span,
    capability: SourceCapability,
    requirement: &Requirement,
) -> Diagnostic {
    let spec = capability.spec();
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::APP_CAPABILITY_MISMATCH,
        format!(
            "app `{}` does not include `{}` in its maximum source authority, but start task `{}` reaches that capability",
            app.name, spec.id, start.name
        ),
        Some(entry_span.clone()),
    )
    .with_related_span(format!("app `{}` authority root", app.name), app.span.clone())
    .with_related_span(format!("start task `{}`", start.name), start.span.clone())
    .with_related_span(
        format!(
            "origin declaration `{}` on task `{}`",
            spec.id, requirement.origin_task
        ),
        requirement.declaration_span.clone(),
    )
    .with_help(format!(
        "Add exact `{}` under app `{}`'s `uses:` section, or remove that capability from the reachable start-task closure. Source declaration remains a maximum, not operator consent.",
        spec.id, app.name
    ));
    for (index, span) in requirement.route_spans.iter().enumerate() {
        diagnostic = diagnostic.with_related_span(
            format!("start closure call {} for `{}`", index + 1, spec.id),
            span.clone(),
        );
    }
    diagnostic
}

fn task_budget_route(
    program: &Program,
    app: &App,
    task: &Task,
    capability: SourceCapability,
    declaration_span: &Span,
) -> CapabilityRouteFact {
    route_fact(
        program,
        "source_capability_task_budget",
        "accepted_source_capability_budget_v0",
        None,
        None,
        capability.spec(),
        task.span.clone(),
        declaration_span.clone(),
        Some(app),
        Some(task),
        None,
        Some(declaration_span.clone()),
        None,
        vec![task.name.clone()],
        Vec::new(),
        None,
    )
}

fn app_budget_route(
    program: &Program,
    app: &App,
    start: &Task,
    entry_span: &Span,
    capability: SourceCapability,
    declaration_span: &Span,
) -> CapabilityRouteFact {
    route_fact(
        program,
        "source_capability_app_maximum",
        "accepted_app_capability_maximum_v0",
        None,
        None,
        capability.spec(),
        start.span.clone(),
        declaration_span.clone(),
        Some(app),
        Some(start),
        None,
        Some(declaration_span.clone()),
        Some(entry_span.clone()),
        vec![app.name.clone(), start.name.clone()],
        Vec::new(),
        None,
    )
}

fn caller_route(
    program: &Program,
    app: &App,
    caller: &Task,
    callee: &Task,
    capability: SourceCapability,
    requirement: &Requirement,
    covered: bool,
) -> CapabilityRouteFact {
    let spec = capability.spec();
    let call_span = requirement
        .route_spans
        .first()
        .expect("caller route begins with its canonical call occurrence");
    let mut fact = route_fact(
        program,
        "source_capability_caller_closure",
        if covered {
            "accepted_caller_capability_closure_v0"
        } else {
            "rejected_missing_caller_capability_v0"
        },
        (!covered).then_some("caller_does_not_cover_callee_capability_closure_v0"),
        (!covered).then_some(DiagnosticCode::MISSING_CALLER_CAPABILITY.as_str()),
        spec,
        caller.span.clone(),
        call_span.clone(),
        Some(app),
        Some(caller),
        Some(callee),
        Some(requirement.declaration_span.clone()),
        None,
        requirement.route_tasks.clone(),
        requirement.route_spans.clone(),
        (!covered).then(|| {
            format!(
                "Add exact `{}` under caller task `{}`'s `uses:` section.",
                spec.id, caller.name
            )
        }),
    );
    fact.resolver_call = requirement.route_calls.first().cloned();
    fact
}

#[allow(clippy::too_many_arguments)]
fn app_closure_route(
    program: &Program,
    app: &App,
    start: &Task,
    entry_span: &Span,
    capability: SourceCapability,
    requirement: &Requirement,
    app_covers: bool,
    task_route_complete: bool,
) -> CapabilityRouteFact {
    let spec = capability.spec();
    let (status, reason, diagnostic_code, help) = if !task_route_complete {
        (
            "not_checked_app_closure_blocked_by_caller_v0",
            Some("caller_capability_closure_must_be_repaired_first_v0"),
            None,
            Some(format!(
                "Repair the missing caller `{}` declaration before evaluating app `{}`'s maximum.",
                spec.id, app.name
            )),
        )
    } else if app_covers {
        ("accepted_app_capability_closure_v0", None, None, None)
    } else {
        (
            "rejected_app_capability_mismatch_v0",
            Some("app_does_not_cover_start_task_capability_closure_v0"),
            Some(DiagnosticCode::APP_CAPABILITY_MISMATCH.as_str()),
            Some(format!(
                "Add exact `{}` under app `{}`'s `uses:` section.",
                spec.id, app.name
            )),
        )
    };
    route_fact(
        program,
        "source_capability_start_closure",
        status,
        reason,
        diagnostic_code,
        spec,
        start.span.clone(),
        entry_span.clone(),
        Some(app),
        Some(start),
        None,
        Some(requirement.declaration_span.clone()),
        Some(entry_span.clone()),
        requirement.route_tasks.clone(),
        requirement.route_spans.clone(),
        help,
    )
}

fn missing_output_source_diagnostic(
    app: Option<&App>,
    task: &Task,
    call_span: &Span,
    task_covers: bool,
    app_covers: bool,
) -> Diagnostic {
    let (missing, verb) = match (task_covers, app_covers) {
        (false, false) => ("the task and app declarations", "do"),
        (false, true) => ("the task declaration", "does"),
        (true, false) => ("the app declaration", "does"),
        (true, true) => ("no declaration", "does"),
    };
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::OUTPUT_CAPABILITY_UNDECLARED,
        format!(
            "`stdout_write` requires exact `stdout.write` source authority, but {missing} {verb} not cover this call"
        ),
        Some(call_span.clone()),
    )
    .with_related_span(
        format!("calling task `{}` authority boundary", task.name),
        task.span.clone(),
    )
    .with_help(
        "Add exact `stdout.write` under the calling task and structural app `uses:` sections, preserving every caller closure, or remove the output call. Source declarations are budgets, not operator consent."
            .to_string(),
    );
    if let Some(app) = app {
        diagnostic = diagnostic.with_related_span(
            format!("app `{}` maximum authority", app.name),
            app.span.clone(),
        );
    }
    diagnostic
}

fn missing_replay_source_diagnostic(
    app: Option<&App>,
    task: &Task,
    call_span: &Span,
    task_covers: bool,
    app_covers: bool,
) -> Diagnostic {
    let (missing, verb) = match (task_covers, app_covers) {
        (false, false) => ("the task and app declarations", "do"),
        (false, true) => ("the task declaration", "does"),
        (true, false) => ("the app declaration", "does"),
        (true, true) => ("no declaration", "does"),
    };
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::REPLAY_CAPABILITY_UNDECLARED,
        format!(
            "`clock_replay_tick` requires exact `clock.replay` source authority, but {missing} {verb} not cover this call"
        ),
        Some(call_span.clone()),
    )
    .with_related_span(
        format!("calling task `{}` authority boundary", task.name),
        task.span.clone(),
    )
    .with_help(
        "Add exact `clock.replay` under the calling task and structural app `uses:` sections, preserving every caller closure, or remove the replay call. Source declarations are budgets, not operator consent, and replay values grant no authority."
            .to_string(),
    );
    if let Some(app) = app {
        diagnostic = diagnostic.with_related_span(
            format!("app `{}` maximum authority", app.name),
            app.span.clone(),
        );
    }
    diagnostic
}

fn missing_file_source_diagnostic(
    app: Option<&App>,
    task: &Task,
    call_span: &Span,
    task_covers: bool,
    app_covers: bool,
) -> Diagnostic {
    let (missing, verb) = match (task_covers, app_covers) {
        (false, false) => ("the task and app declarations", "do"),
        (false, true) => ("the task declaration", "does"),
        (true, false) => ("the app declaration", "does"),
        (true, true) => ("no declaration", "does"),
    };
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::FILE_CAPABILITY_UNDECLARED,
        format!(
            "`files_read_text` requires exact `files.read` source authority, but {missing} {verb} not cover this call"
        ),
        Some(call_span.clone()),
    )
    .with_related_span(
        format!("calling task `{}` authority boundary", task.name),
        task.span.clone(),
    )
    .with_help(
        "Add exact `files.read` under the calling task and structural app `uses:` sections, preserving every caller closure, or remove the file call. Source declarations are budgets, not operator consent; the operator must separately grant the exact native path."
            .to_string(),
    );
    if let Some(app) = app {
        diagnostic = diagnostic.with_related_span(
            format!("app `{}` maximum authority", app.name),
            app.span.clone(),
        );
    }
    diagnostic
}

#[allow(clippy::too_many_arguments)]
fn output_operation_route(
    program: &Program,
    app: Option<&App>,
    task: &Task,
    entry_span: Option<&Span>,
    resolver_call: &crate::resolve::ResolveCallOccurrenceSummary,
    declaration_span: Option<&Span>,
    task_covers: bool,
    app_covers: bool,
    route_tasks: Vec<String>,
    route_spans: Vec<Span>,
) -> CapabilityRouteFact {
    let call_span = &resolver_call.exact_call_span;
    let covered = task_covers && app_covers;
    let route_identity = format!(
        "{}-{}",
        route_tasks.join("-"),
        route_spans
            .iter()
            .map(|span| format!("{}-{}-{}", span.file, span.line, span.column))
            .collect::<Vec<_>>()
            .join("-")
    );
    let mut fact = route_fact(
        program,
        "source_capability_output_operation",
        if covered {
            "accepted_declared_output_operation_v0"
        } else {
            "rejected_missing_output_source_authority_v0"
        },
        (!covered).then_some("stdout_write_requires_task_and_app_source_authority_v0"),
        (!covered).then_some(DiagnosticCode::OUTPUT_CAPABILITY_UNDECLARED.as_str()),
        SourceCapability::StdoutWrite.spec(),
        task.span.clone(),
        call_span.clone(),
        app,
        Some(task),
        None,
        declaration_span.cloned(),
        entry_span.cloned(),
        route_tasks,
        route_spans,
        (!covered).then(|| {
            "Add exact `stdout.write` to both the task and app source authority budgets."
                .to_string()
        }),
    );
    fact.id = node_id::span(
        "capability-policy",
        call_span,
        &format!("source-capability-output-operation-stdout-write-{route_identity}"),
    );
    fact.resolver_call = Some(resolver_call.clone());
    fact
}

#[allow(clippy::too_many_arguments)]
fn replay_operation_route(
    program: &Program,
    app: Option<&App>,
    task: &Task,
    entry_span: Option<&Span>,
    resolver_call: &crate::resolve::ResolveCallOccurrenceSummary,
    declaration_span: Option<&Span>,
    task_covers: bool,
    app_covers: bool,
    route_tasks: Vec<String>,
    route_spans: Vec<Span>,
) -> CapabilityRouteFact {
    let call_span = &resolver_call.exact_call_span;
    let covered = task_covers && app_covers;
    let route_identity = format!(
        "{}-{}",
        route_tasks.join("-"),
        route_spans
            .iter()
            .map(|span| format!("{}-{}-{}", span.file, span.line, span.column))
            .collect::<Vec<_>>()
            .join("-")
    );
    let mut fact = route_fact(
        program,
        "source_capability_replay_operation",
        if covered {
            "accepted_declared_runner_replay_operation_v0"
        } else {
            "rejected_missing_replay_source_authority_v0"
        },
        (!covered).then_some("clock_replay_tick_requires_task_and_app_source_authority_v0"),
        (!covered).then_some(DiagnosticCode::REPLAY_CAPABILITY_UNDECLARED.as_str()),
        SourceCapability::ClockReplay.spec(),
        task.span.clone(),
        call_span.clone(),
        app,
        Some(task),
        None,
        declaration_span.cloned(),
        entry_span.cloned(),
        route_tasks,
        route_spans,
        (!covered).then(|| {
            "Add exact `clock.replay` to both the task and app source authority budgets."
                .to_string()
        }),
    );
    fact.id = node_id::span(
        "capability-policy",
        call_span,
        &format!("source-capability-replay-operation-clock-replay-{route_identity}"),
    );
    fact.resolver_call = Some(resolver_call.clone());
    fact
}

#[allow(clippy::too_many_arguments)]
fn file_operation_route(
    program: &Program,
    app: Option<&App>,
    task: &Task,
    entry_span: Option<&Span>,
    resolver_call: &crate::resolve::ResolveCallOccurrenceSummary,
    declaration_span: Option<&Span>,
    task_covers: bool,
    app_covers: bool,
    route_tasks: Vec<String>,
    route_spans: Vec<Span>,
) -> CapabilityRouteFact {
    let call_span = &resolver_call.exact_call_span;
    let covered = task_covers && app_covers;
    let route_identity = format!(
        "{}-{}",
        route_tasks.join("-"),
        route_spans
            .iter()
            .map(|span| format!("{}-{}-{}", span.file, span.line, span.column))
            .collect::<Vec<_>>()
            .join("-")
    );
    let mut fact = route_fact(
        program,
        "source_capability_file_operation",
        if covered {
            "accepted_declared_exact_file_read_operation_v0"
        } else {
            "rejected_missing_file_source_authority_v0"
        },
        (!covered).then_some("files_read_text_requires_task_and_app_source_authority_v0"),
        (!covered).then_some(DiagnosticCode::FILE_CAPABILITY_UNDECLARED.as_str()),
        SourceCapability::FilesRead.spec(),
        task.span.clone(),
        call_span.clone(),
        app,
        Some(task),
        None,
        declaration_span.cloned(),
        entry_span.cloned(),
        route_tasks,
        route_spans,
        (!covered).then(|| {
            "Add exact `files.read` to both the task and app source authority budgets.".to_string()
        }),
    );
    fact.id = node_id::span(
        "capability-policy",
        call_span,
        &format!("source-capability-file-operation-files-read-{route_identity}"),
    );
    fact.resolver_call = Some(resolver_call.clone());
    fact
}

fn unknown_route(
    program: &Program,
    capability: &str,
    span: &Span,
    owner_task_span: &Span,
    app: Option<&App>,
    task: Option<&Task>,
) -> CapabilityRouteFact {
    let sandbox_bypass = is_sandbox_bypass(capability);
    CapabilityRouteFact {
        id: node_id::span("capability-policy", span, &format!("unknown-{capability}")),
        owner_task_identity: task.map(|task| crate::resolve::semantic_task_identity(program, task)),
        owner_task_span: owner_task_span.clone(),
        primary_span: span.clone(),
        check: "source_capability_vocabulary",
        status: "rejected_unknown_source_capability_v0",
        reason: Some("capability_id_not_in_pinned_source_vocabulary_v0"),
        diagnostic_code: Some(DiagnosticCode::UNKNOWN_SOURCE_CAPABILITY.as_str()),
        capability_id: capability.to_string(),
        core_effect: None,
        runtime_target_meaning: None,
        grant_kind: sandbox_bypass.then_some("sandbox_bypass"),
        grant_scope: None,
        grant_strength: None,
        grant_lifetime: None,
        severity_tier: sandbox_bypass.then_some("sandbox_bypass_authority"),
        mapping_status: sandbox_bypass.then_some("forbidden_in_work_order_6_v0"),
        app_name: app.map(|app| app.name.clone()),
        caller: task.map(|task| task.name.clone()),
        callee: None,
        app_span: app.map(|app| app.span.clone()),
        caller_span: task.map(|task| task.span.clone()),
        callee_span: None,
        declaration_span: Some(span.clone()),
        entry_span: None,
        route_tasks: task.map(|task| vec![task.name.clone()]).unwrap_or_default(),
        route_spans: Vec::new(),
        help: Some(format!(
            "Use one of the exact pinned IDs: {}.",
            CAPABILITY_IDS.join(", ")
        )),
        resolver_call: None,
    }
}

fn is_sandbox_bypass(capability: &str) -> bool {
    capability
        .split('.')
        .next()
        .is_some_and(|root| matches!(root, "process" | "ffi" | "unsafe" | "import"))
}

#[allow(clippy::too_many_arguments)]
fn route_fact(
    program: &Program,
    check: &'static str,
    status: &'static str,
    reason: Option<&'static str>,
    diagnostic_code: Option<&'static str>,
    spec: CapabilitySpec,
    owner_task_span: Span,
    primary_span: Span,
    app: Option<&App>,
    caller: Option<&Task>,
    callee: Option<&Task>,
    declaration_span: Option<Span>,
    entry_span: Option<Span>,
    route_tasks: Vec<String>,
    route_spans: Vec<Span>,
    help: Option<String>,
) -> CapabilityRouteFact {
    CapabilityRouteFact {
        id: node_id::span(
            "capability-policy",
            &primary_span,
            &format!("{check}-{}", spec.id),
        ),
        owner_task_identity: caller
            .map(|task| crate::resolve::semantic_task_identity(program, task)),
        owner_task_span,
        primary_span,
        check,
        status,
        reason,
        diagnostic_code,
        capability_id: spec.id.to_string(),
        core_effect: Some(spec.core_effect),
        runtime_target_meaning: Some(spec.runtime_target_meaning),
        grant_kind: Some(spec.grant_kind),
        grant_scope: Some(spec.grant_scope),
        grant_strength: Some(spec.grant_strength),
        grant_lifetime: Some(spec.grant_lifetime),
        severity_tier: Some(spec.severity_tier),
        mapping_status: Some(spec.mapping_status),
        app_name: app.map(|app| app.name.clone()),
        caller: caller.map(|task| task.name.clone()),
        callee: callee.map(|task| task.name.clone()),
        app_span: app.map(|app| app.span.clone()),
        caller_span: caller.map(|task| task.span.clone()),
        callee_span: callee.map(|task| task.span.clone()),
        declaration_span,
        entry_span,
        route_tasks,
        route_spans,
        help,
        resolver_call: None,
    }
}

fn find_task_context<'a>(program: &'a Program, name: &str) -> Option<(&'a Task, &'a [Item])> {
    for file in &program.files {
        if let Some(found) = find_task_in_items(&file.items, name) {
            return Some(found);
        }
    }
    None
}

fn find_task_in_items<'a>(items: &'a [Item], name: &str) -> Option<(&'a Task, &'a [Item])> {
    for item in items {
        match item {
            Item::Task(task) if task.name == name => return Some((task, items)),
            Item::App(app) => {
                if let Some(found) = find_task_in_items(&app.items, name) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_value_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_lowercase())
        && chars.all(|ch| ch == '_' || ch.is_ascii_lowercase() || ch.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::parser;

    use super::{CapabilityAnalysis, SourceCapability, analyze, entry_diagnostics};

    fn program(source: &str) -> crate::ast::Program {
        let parsed = parser::parse_source("capability.hum", source);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        crate::ast::Program {
            files: vec![parsed.file],
        }
    }

    #[test]
    fn source_capability_specs_are_exact_and_one_run() {
        let stdout = SourceCapability::parse("stdout.write")
            .expect("stdout capability")
            .spec();
        assert_eq!(stdout.grant_scope, "app_stdout");
        assert_eq!(stdout.grant_strength, "write");
        assert_eq!(stdout.grant_lifetime, "one_run");
        assert_eq!(stdout.severity_tier, "ordinary_external_authority");
        assert!(SourceCapability::parse("stdout.*").is_none());
        assert!(SourceCapability::parse("process.run").is_none());
    }

    #[test]
    fn computes_transitive_source_closure_without_host_effects() {
        let program = program(
            r#"app tool {
  uses:
    clock.replay
  starts with:
    run_tool
  task helper -> Int {
    uses:
      clock.replay
    does:
      return 7
  }
  task run_tool -> Unit {
    uses:
      clock.replay
    does:
      let value = helper()
      return
  }
}
"#,
        );
        let analysis = analyze(&program);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:#?}",
            analysis.diagnostics
        );
        assert!(analysis.routes.iter().any(|route| {
            route.status == "accepted_caller_capability_closure_v0"
                && route.capability_id == "clock.replay"
                && route.route_tasks == ["run_tool", "helper"]
        }));
        assert!(analysis.routes.iter().any(|route| {
            route.status == "accepted_app_capability_closure_v0"
                && route.capability_id == "clock.replay"
                && route.route_tasks == ["run_tool"]
        }));
        let entry = entry_diagnostics(&program, "run_tool");
        assert_eq!(entry.len(), 1);
        assert_eq!(
            entry[0].code,
            crate::diagnostic::DiagnosticCode::ENTRY_CAPABILITY_BYPASS
        );
    }

    #[test]
    fn rejects_wildcards_and_keeps_caller_failure_precedence() {
        let wildcard = analyze(&program(
            r#"app tool {
  uses:
    stdout.*
  starts with:
    run_tool
  task run_tool -> Unit {
    does:
      return
  }
}
"#,
        ));
        assert_eq!(wildcard.diagnostics.len(), 1);
        assert_eq!(
            wildcard.diagnostics[0].code,
            crate::diagnostic::DiagnosticCode::UNKNOWN_SOURCE_CAPABILITY
        );

        let incomplete_closure = analyze(&program(
            r#"app tool {
  starts with:
    run_tool
  task helper -> Int {
    uses:
      files.read
    does:
      return 7
  }
  task run_tool -> Unit {
    does:
      let observed = helper()
      return
  }
}
"#,
        ));
        assert_eq!(incomplete_closure.diagnostics.len(), 1);
        assert_eq!(
            incomplete_closure.diagnostics[0].code,
            crate::diagnostic::DiagnosticCode::MISSING_CALLER_CAPABILITY
        );
        assert!(
            !incomplete_closure
                .routes
                .iter()
                .any(|route| { route.status == "rejected_app_capability_mismatch_v0" })
        );
    }

    #[test]
    fn direct_entry_rejects_transitive_unknown_authority_with_route() {
        let program = program(
            r#"app tool {
  starts with:
    run_tool
  task helper -> Int {
    uses:
      process.run
    does:
      return 7
  }
  task run_tool -> Unit {
    does:
      let observed = helper()
      return
  }
}
"#,
        );
        let diagnostics = entry_diagnostics(&program, "run_tool");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            crate::diagnostic::DiagnosticCode::UNKNOWN_SOURCE_CAPABILITY
        );
        assert_eq!(diagnostics[0].related_spans.len(), 2);
        assert!(
            diagnostics[0].related_spans[1]
                .label
                .contains("direct-entry authority route call 1")
        );
    }

    #[test]
    fn call_occurrence_policy_ids_are_unique_and_repeatable() {
        let program = program(
            r#"app tool {
  uses:
    stdout.write
  starts with:
    run_tool
  task left -> Int {
    uses:
      stdout.write
    does:
      return 1
  }
  task right -> Int {
    uses:
      stdout.write
    does:
      return 2
  }
  task run_tool -> Unit {
    uses:
      stdout.write
    does:
      let observed = left() + right() + left()
      return
  }
}
"#,
        );
        let occurrence_rows = |analysis: CapabilityAnalysis| {
            analysis
                .routes
                .into_iter()
                .filter(|route| {
                    route.status == "accepted_caller_capability_closure_v0"
                        && route.caller.as_deref() == Some("run_tool")
                        && route.capability_id == "stdout.write"
                })
                .map(|route| (route.id, route.primary_span.column))
                .collect::<Vec<_>>()
        };
        let first = occurrence_rows(analyze(&program));
        let second = occurrence_rows(analyze(&program));
        assert_eq!(first.len(), 3);
        assert_eq!(first, second);
        assert_eq!(
            first
                .iter()
                .map(|(id, _)| id)
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
        assert_eq!(
            first
                .iter()
                .map(|(_, column)| column)
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
        let resolver_keys = crate::resolve::resolve_call_occurrence_summaries(&program, &[])
            .into_iter()
            .map(|call| call.relationship_key())
            .collect::<BTreeSet<_>>();
        let capability_keys = analyze(&program)
            .routes
            .into_iter()
            .filter(|route| {
                route.status == "accepted_caller_capability_closure_v0"
                    && route.caller.as_deref() == Some("run_tool")
                    && route.capability_id == "stdout.write"
            })
            .map(|route| {
                route
                    .resolver_call
                    .expect("caller capability fact must carry resolver call")
                    .relationship_key()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(capability_keys.len(), 3);
        assert!(capability_keys.is_subset(&resolver_keys));
    }

    #[test]
    fn capability_occurrence_identity_uses_structural_items_not_display_names() {
        fn source(app: &str, start: &str, helper: &str) -> String {
            format!(
                r#"app {app} {{
  starts with:
    {start}
  task {helper} -> Int {{
    uses:
      stdout.write
    does:
      return 7
  }}
  task {start} -> Unit {{
    does:
      let observed = {helper}()
      return
  }}
}}"#
            )
        }

        let first_program = program(&source("tool", "run_tool", "helper"));
        let renamed_program = program(&source("renamed", "start", "worker"));
        let first = analyze(&first_program)
            .diagnostic_occurrences
            .occurrences()
            .find(|occurrence| {
                occurrence.code == crate::diagnostic::DiagnosticCode::MISSING_CALLER_CAPABILITY
            })
            .expect("first authority occurrence")
            .clone();
        let renamed = analyze(&renamed_program)
            .diagnostic_occurrences
            .occurrences()
            .find(|occurrence| {
                occurrence.code == crate::diagnostic::DiagnosticCode::MISSING_CALLER_CAPABILITY
            })
            .expect("renamed authority occurrence")
            .clone();
        assert_eq!(first.semantic_origin(), renamed.semantic_origin());
        assert_eq!(first.relationship_route(), renamed.relationship_route());
        assert!(
            first
                .relationship_route()
                .iter()
                .all(|part| !part.contains("run_tool") && !part.contains("helper"))
        );
    }
}
