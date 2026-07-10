use std::collections::{BTreeMap, BTreeSet};

use crate::app_entry;
use crate::ast::{App, Item, Program, Section, SectionLine, Task};
use crate::core_body;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Span};
use crate::graph::is_meaningful_line_text;
use crate::node_id;
use crate::typed_failure;

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
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CapabilityAnalysis {
    pub diagnostics: Vec<Diagnostic>,
    pub routes: Vec<CapabilityRouteFact>,
}

#[derive(Debug, Clone)]
struct Requirement {
    origin_task: String,
    declaration_span: Span,
    route_tasks: Vec<String>,
    route_spans: Vec<Span>,
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
}

struct TaskNode<'a> {
    task: &'a Task,
    capabilities: BTreeMap<SourceCapability, Span>,
    unknown_capabilities: Vec<(String, Span)>,
    calls: Vec<CallEdge>,
}

struct TaskGraph<'a> {
    tasks: BTreeMap<String, TaskNode<'a>>,
    order: Vec<String>,
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
                mapping_status: "reserved_until_session_z_v0",
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
                mapping_status: "reserved_until_session_aa_v0",
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
                mapping_status: "reserved_until_session_ad_v0",
            },
        }
    }
}

pub(crate) fn analyze(program: &Program) -> CapabilityAnalysis {
    let Some(entry) = app_entry::analyze(program).entry else {
        return CapabilityAnalysis::default();
    };
    analyze_app(entry.app, entry.task)
}

pub(crate) fn diagnostics(program: &Program) -> Vec<Diagnostic> {
    analyze(program).diagnostics
}

pub(crate) fn entry_diagnostics(program: &Program, entry_name: &str) -> Vec<Diagnostic> {
    let Some((task, items)) = find_task_context(program, entry_name) else {
        return Vec::new();
    };
    let graph = build_task_graph(items);
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
        return vec![diagnostic];
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
    vec![diagnostic]
}

pub(crate) fn is_capability_diagnostic(diagnostic: &Diagnostic) -> bool {
    matches!(
        diagnostic.code,
        DiagnosticCode::UNKNOWN_SOURCE_CAPABILITY
            | DiagnosticCode::MISSING_CALLER_CAPABILITY
            | DiagnosticCode::APP_CAPABILITY_MISMATCH
            | DiagnosticCode::ENTRY_CAPABILITY_BYPASS
    )
}

fn analyze_app(app: &App, start: &Task) -> CapabilityAnalysis {
    let graph = build_task_graph(&app.items);
    let closures = compute_closures(&graph);
    let (app_capabilities, app_unknown) = declarations(&app.sections);
    let entry_span = start_declaration_span(app).unwrap_or_else(|| start.span.clone());
    let mut diagnostics = Vec::new();
    let mut routes = Vec::new();

    for (capability, span) in app_unknown {
        diagnostics.push(unknown_capability_diagnostic(
            &capability,
            &span,
            "app",
            &app.name,
            &app.span,
        ));
        routes.push(unknown_route(
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
                capability,
                span,
                &node.task.span,
                Some(app),
                Some(node.task),
            ));
        }
        for (capability, declaration_span) in &node.capabilities {
            routes.push(task_budget_route(
                app,
                node.task,
                *capability,
                declaration_span,
            ));
        }
    }

    for (capability, declaration_span) in &app_capabilities {
        routes.push(app_budget_route(
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
                    app,
                    caller.task,
                    callee.task,
                    *capability,
                    &call.span,
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
            if !app_covers && task_route_complete {
                diagnostics.push(app_mismatch_diagnostic(
                    app,
                    start,
                    &entry_span,
                    *capability,
                    requirement,
                ));
            }
            routes.push(app_closure_route(
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
    }
}

fn build_task_graph(items: &[Item]) -> TaskGraph<'_> {
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
                },
            );
        }
    }
    let known_names = tasks.keys().cloned().collect::<BTreeSet<_>>();
    for task_name in &order {
        let task = tasks[task_name].task;
        let calls = task
            .section("does")
            .map(core_body::analyze_does_section)
            .into_iter()
            .flat_map(|body| body.statements)
            .flat_map(|statement| {
                let Some(expression) = typed_failure::statement_expression(&statement) else {
                    return Vec::new();
                };
                let expression_offset = statement.text.find(expression).unwrap_or(0);
                typed_failure::calls_in_expression(expression)
                    .into_iter()
                    .filter(|call| known_names.contains(&call.callee))
                    .map(|call| {
                        let byte_offset = expression_offset + call.source_offset;
                        let column_offset = statement.text[..byte_offset].chars().count();
                        CallEdge {
                            callee: call.callee,
                            span: Span {
                                file: statement.span.file.clone(),
                                line: statement.span.line,
                                column: statement.span.column + column_offset,
                            },
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        tasks.get_mut(task_name).expect("task exists").calls = calls;
    }
    TaskGraph { tasks, order }
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
    Requirement {
        origin_task: requirement.origin_task.clone(),
        declaration_span: requirement.declaration_span.clone(),
        route_tasks,
        route_spans,
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
    app: &App,
    task: &Task,
    capability: SourceCapability,
    declaration_span: &Span,
) -> CapabilityRouteFact {
    route_fact(
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
    app: &App,
    start: &Task,
    entry_span: &Span,
    capability: SourceCapability,
    declaration_span: &Span,
) -> CapabilityRouteFact {
    route_fact(
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
    app: &App,
    caller: &Task,
    callee: &Task,
    capability: SourceCapability,
    call_span: &Span,
    requirement: &Requirement,
    covered: bool,
) -> CapabilityRouteFact {
    let spec = capability.spec();
    route_fact(
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
    )
}

#[allow(clippy::too_many_arguments)]
fn app_closure_route(
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

fn unknown_route(
    capability: &str,
    span: &Span,
    owner_task_span: &Span,
    app: Option<&App>,
    task: Option<&Task>,
) -> CapabilityRouteFact {
    let sandbox_bypass = is_sandbox_bypass(capability);
    CapabilityRouteFact {
        id: node_id::span("capability-policy", span, &format!("unknown-{capability}")),
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
    }
}
