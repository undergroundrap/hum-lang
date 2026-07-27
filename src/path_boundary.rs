use std::collections::BTreeMap;

use crate::ast::{
    App, CanonicalExpression, CanonicalExpressionKind, Item, ParamPermission, ParsedBodyStatement,
    ParsedBodyStatementKind, Program, Section, Task,
};
use crate::core_body;
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOccurrence, Span};
use crate::graph::is_meaningful_line_text;

pub(crate) fn diagnostics(program: &Program) -> Vec<Diagnostic> {
    analyze(program).diagnostics
}

#[derive(Default)]
struct PathAnalysis {
    diagnostics: Vec<Diagnostic>,
    diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet,
}

#[derive(Clone)]
struct PathResolverAuthority {
    definitions: Vec<crate::resolve::ResolveDefinitionSummary>,
    references: Vec<crate::resolve::ResolveReferenceSummary>,
    calls: Vec<crate::resolve::ResolveCallOccurrenceSummary>,
}

#[cfg(test)]
enum TenB3PathBoundaryCorruption {
    BuiltinCallOccurrence(String),
    ParameterDefinition,
    ParameterReference,
    RemoveSourcePathCall,
    SourceCallTarget,
    GroupOccurrence,
    PermissionOccurrence,
    PlaceRootEdge,
}

#[cfg(test)]
struct TenB3PathBoundaryCorruptionState {
    corruption: TenB3PathBoundaryCorruption,
    hits: usize,
}

#[cfg(test)]
thread_local! {
    static TEN_B3_PATH_BOUNDARY_CORRUPTION:
        std::cell::RefCell<Option<TenB3PathBoundaryCorruptionState>> =
            const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn with_ten_b3_path_boundary_corruption<T>(
    corruption: TenB3PathBoundaryCorruption,
    run: impl FnOnce() -> T,
) -> T {
    TEN_B3_PATH_BOUNDARY_CORRUPTION.with(|slot| {
        assert!(
            slot.replace(Some(TenB3PathBoundaryCorruptionState {
                corruption,
                hits: 0,
            }))
            .is_none(),
            "10B.3 Path boundary corruption must not nest"
        );
    });
    let result = run();
    TEN_B3_PATH_BOUNDARY_CORRUPTION.with(|slot| {
        let state = slot
            .replace(None)
            .expect("10B.3 Path boundary corruption must remain installed");
        assert_eq!(
            state.hits, 1,
            "10B.3 Path boundary corruption must alter exactly one production read"
        );
    });
    result
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diagnostic_occurrence_set(
    program: &Program,
) -> crate::diagnostic::DiagnosticOccurrenceSet {
    analyze(program).diagnostic_occurrences
}

fn analyze(program: &Program) -> PathAnalysis {
    let mut authority = PathResolverAuthority {
        definitions: crate::resolve::resolve_definition_summaries(program, &[]),
        references: crate::resolve::resolve_reference_summaries(program, &[]),
        calls: crate::resolve::resolve_call_occurrence_summaries(program, &[]),
    };
    #[cfg(test)]
    ten_b3_corrupt_path_resolver_boundary(&mut authority);
    analyze_at_resolver_boundary(program, &authority)
}

#[cfg(test)]
fn ten_b3_corrupt_path_resolver_boundary(authority: &mut PathResolverAuthority) {
    TEN_B3_PATH_BOUNDARY_CORRUPTION.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(state) = slot.as_mut() else {
            return;
        };
        match &state.corruption {
            TenB3PathBoundaryCorruption::BuiltinCallOccurrence(replacement) => {
                authority
                    .calls
                    .iter_mut()
                    .find(|call| call.target_definition_id == "builtin_files_read_text")
                    .expect("10B.3 builtin Path call occurrence")
                    .replace_canonical_call_node_for_test(replacement);
            }
            TenB3PathBoundaryCorruption::ParameterDefinition => {
                authority
                    .definitions
                    .iter_mut()
                    .find(|definition| {
                        definition.definition_kind == "parameter" && definition.name == "input"
                    })
                    .expect("10B.3 Path parameter definition")
                    .semantic_identity = "foreign-path-parameter-definition".to_string();
            }
            TenB3PathBoundaryCorruption::ParameterReference => {
                authority
                    .references
                    .iter_mut()
                    .find(|reference| {
                        reference.name == "input"
                            && reference.resolved_definition_semantic_identity.is_some()
                    })
                    .expect("10B.3 Path parameter reference")
                    .resolved_definition_semantic_identity =
                    Some("foreign-path-parameter-definition".to_string());
            }
            TenB3PathBoundaryCorruption::RemoveSourcePathCall => {
                let index = authority
                    .calls
                    .iter()
                    .position(|call| call.source().starts_with("run_tool("))
                    .expect("10B.3 source Path call occurrence");
                authority.calls.remove(index);
            }
            TenB3PathBoundaryCorruption::SourceCallTarget => {
                let harmless = authority
                    .definitions
                    .iter()
                    .find(|definition| {
                        definition.definition_kind == "task" && definition.name == "harmless"
                    })
                    .expect("10B.3 harmless task definition")
                    .clone();
                let call = authority
                    .calls
                    .iter_mut()
                    .find(|call| call.source().starts_with("run_tool("))
                    .expect("10B.3 source Path call occurrence");
                call.target_definition_id = harmless.semantic_identity;
                let reference = authority
                    .references
                    .iter_mut()
                    .find(|reference| {
                        reference.canonical_node_id.as_deref()
                            == Some(call.canonical_callee_node_id())
                    })
                    .expect("10B.3 source Path call reference");
                reference.resolved_definition_id = Some(harmless.id);
                reference.resolved_definition_semantic_identity =
                    Some(call.target_definition_id.clone());
            }
            TenB3PathBoundaryCorruption::GroupOccurrence
            | TenB3PathBoundaryCorruption::PermissionOccurrence
            | TenB3PathBoundaryCorruption::PlaceRootEdge => return,
        }
        state.hits += 1;
    });
}

fn analyze_at_resolver_boundary(
    program: &Program,
    authority: &PathResolverAuthority,
) -> PathAnalysis {
    let mut diagnostics = PathAnalysis::default();
    for file in &program.files {
        check_scope(program, &file.items, None, authority, &mut diagnostics);
    }
    diagnostics
}

fn check_scope(
    program: &Program,
    items: &[Item],
    selected: Option<&Task>,
    authority: &PathResolverAuthority,
    diagnostics: &mut PathAnalysis,
) {
    for item in items {
        let item_identity = crate::resolve::semantic_item_identity_for(program, item);
        match item {
            Item::App(app) => check_scope(
                program,
                &app.items,
                local_start_task(app),
                authority,
                diagnostics,
            ),
            Item::Type(type_def) => {
                if type_def.name == "Path" {
                    emit(
                        diagnostics,
                        crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(106),
                        "path_type",
                        item_identity.clone(),
                        invalid_declaration(
                            "type `Path` redeclares Hum's opaque runner-owned Path identity",
                            &type_def.span,
                            None,
                        ),
                    );
                } else if let Some((field_index, field)) = type_def
                    .fields
                    .iter()
                    .enumerate()
                    .find(|(_, field)| contains_path_type(&field.ty))
                {
                    emit(
                        diagnostics,
                        crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(106),
                        "path_field",
                        format!("{item_identity}:field-{field_index}"),
                        invalid_declaration(
                            "opaque Path cannot be stored in a type field",
                            &field.span,
                            Some(&type_def.span),
                        ),
                    );
                }
            }
            Item::Store(store) if contains_path_type(&store.ty) => {
                emit(
                    diagnostics,
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(106),
                    "path_store",
                    item_identity,
                    invalid_declaration(
                        "opaque Path cannot be stored in a store",
                        &store.span,
                        None,
                    ),
                );
            }
            Item::Task(task) => check_task_signature(
                program,
                task,
                &item_identity,
                selected,
                authority,
                diagnostics,
            ),
            Item::Test(test) => {
                if let Some((parameter_index, parameter)) = test
                    .params
                    .iter()
                    .enumerate()
                    .find(|(_, parameter)| contains_path_type(&parameter.ty))
                {
                    emit(
                        diagnostics,
                        crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(106),
                        "path_test_parameter",
                        format!("{item_identity}:parameter-{parameter_index}"),
                        invalid_declaration(
                            "opaque Path cannot be constructed as a test parameter",
                            &parameter.span,
                            Some(&test.span),
                        ),
                    );
                }
            }
            Item::Store(_) => {}
        }
    }

    let callable_paths = items
        .iter()
        .filter_map(|item| match item {
            Item::Task(task)
                if selected.is_some_and(|selected| std::ptr::eq(task, selected))
                    && task
                        .params
                        .iter()
                        .filter(|parameter| contains_path_type(&parameter.ty))
                        .count()
                        == 1 =>
            {
                Some((
                    crate::resolve::semantic_task_definition_identity(program, task),
                    task,
                ))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    if callable_paths.is_empty() {
        return;
    }
    for item in items {
        match item {
            Item::Task(task) => check_source_path_construction(
                program,
                item,
                SourceOwner {
                    kind: "task",
                    name: &task.name,
                    span: &task.span,
                    identity: crate::resolve::semantic_item_identity_for(program, item),
                },
                task.section("does"),
                &callable_paths,
                authority,
                diagnostics,
            ),
            Item::Test(test) => check_source_path_construction(
                program,
                item,
                SourceOwner {
                    kind: "test",
                    name: &test.name,
                    span: &test.span,
                    identity: crate::resolve::semantic_item_identity_for(program, item),
                },
                test.section("does"),
                &callable_paths,
                authority,
                diagnostics,
            ),
            _ => {}
        }
    }
}

fn local_start_task(app: &App) -> Option<&Task> {
    let section = app
        .sections
        .iter()
        .filter(|section| section.name == "starts with")
        .collect::<Vec<_>>();
    let [section] = section.as_slice() else {
        return None;
    };
    let lines = section
        .lines
        .iter()
        .filter(|line| is_meaningful_line_text(&line.text))
        .collect::<Vec<_>>();
    let [line] = lines.as_slice() else {
        return None;
    };
    let name = line.text.trim();
    app.items.iter().find_map(|item| match item {
        Item::Task(task) if task.name == name => Some(task),
        _ => None,
    })
}

fn check_task_signature(
    program: &Program,
    task: &Task,
    task_identity: &str,
    selected: Option<&Task>,
    authority: &PathResolverAuthority,
    diagnostics: &mut PathAnalysis,
) {
    if task.result.as_deref().is_some_and(contains_path_type) {
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(106),
            "path_return",
            format!("{task_identity}:result"),
            invalid_declaration(
                "opaque Path cannot be returned from a task",
                &task.span,
                None,
            ),
        );
        return;
    }
    let path_parameters = task
        .params
        .iter()
        .filter(|parameter| contains_path_type(&parameter.ty))
        .collect::<Vec<_>>();
    if path_parameters.is_empty() {
        return;
    }
    let is_selected = selected.is_some_and(|selected| std::ptr::eq(task, selected));
    if !is_selected {
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(106),
            "path_non_start_parameter",
            format!("{task_identity}:parameter-0"),
            invalid_declaration(
                "opaque Path is allowed only as the runner-constructed parameter of the structural app start task",
                &path_parameters[0].span,
                Some(&task.span),
            ),
        );
        return;
    }
    if path_parameters.len() > 1 {
        let mut diagnostic = invalid_declaration(
            "structural app start declares more than one opaque Path parameter",
            &path_parameters[1].span,
            Some(&task.span),
        )
        .with_related_span("first Path parameter", path_parameters[0].span.clone());
        for parameter in path_parameters.iter().skip(2) {
            diagnostic =
                diagnostic.with_related_span("additional Path parameter", parameter.span.clone());
        }
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(106),
            "path_parameter_count",
            format!("{task_identity}:parameter-1"),
            diagnostic,
        );
        return;
    }

    let parameter = path_parameters[0];
    let Some(parameter_definition) = authority.definitions.iter().find(|definition| {
        definition.definition_kind == "parameter"
            && definition.name == parameter.name
            && definition.source_span == parameter.span
    }) else {
        // A duplicate callable can be rejected by the resolver before it owns
        // parameter definitions. Path analysis must not panic or invent a
        // competing identity/diagnostic for that already-blocked callable.
        return;
    };
    let contract_use = ["needs", "ensures"].into_iter().find_map(|name| {
        task.section(name).and_then(|section| {
            section.lines.iter().find_map(|line| {
                let fact = crate::predicate::fact_for_line(program, task, name, line)?;
                (is_meaningful_line_text(&line.text)
                    && fact.reason == "opaque_path_inspection_owned_by_h0630"
                    && fact.places.iter().any(|place| {
                        place.resolution == "resolved_v0"
                            && place
                                .definition_id
                                .as_deref()
                                .is_some_and(|identity| identity == parameter_definition.id)
                    }))
                .then(|| (line, fact.semantic_line_identity().to_string()))
            })
        })
    });
    let body_use = task.section("does").and_then(|does| {
        let body = core_body::analyze_does_section(
            program
                .canonical_core_expectation_for_task(task, does)
                .expect("live Path task must have parser authority"),
        );
        #[cfg(test)]
        let corrupted_body_syntax =
            ten_b3_path_statements_at_consumer_boundary("path-parameter-consumption", task);
        #[cfg(test)]
        let body_syntax = corrupted_body_syntax
            .as_deref()
            .unwrap_or(task.body_syntax.as_slice());
        #[cfg(not(test))]
        let body_syntax = task.body_syntax.as_slice();
        assert_eq!(
            body.statements.len(),
            body_syntax.len(),
            "validated Core body and parser statements must stay aligned"
        );
        let task_identity = crate::resolve::semantic_task_identity(program, task);
        body.statements
            .into_iter()
            .zip(body_syntax)
            .enumerate()
            .find_map(|(index, (statement, parsed))| {
                (canonical_statement_roots(parsed).iter().any(|expression| {
                    canonical_resolved_identifier_nodes(
                        expression,
                        parameter_definition,
                        &authority.references,
                    )
                    .next()
                    .is_some()
                }) && !is_exact_file_read_consumption(parsed, parameter_definition, authority))
                .then(|| {
                    let source_identity = format!(
                        "{task_identity}:does-statement-{index}:{}",
                        parsed.source_node_id.as_str()
                    );
                    (statement.span, source_identity)
                })
            })
    });
    let source_use = contract_use
        .map(|(line, line_identity)| (line.span.clone(), line_identity))
        .or(body_use);
    if let Some((span, source_identity)) = source_use {
        emit_with_identity(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(107), "path_inspection",
            format!("path-inspection:{source_identity}"),
            vec![
                format!("path_task_identity={task_identity}"),
                format!("path_source_identity={source_identity}"),
            ],
            Diagnostic::error(
                DiagnosticCode::PATH_SOURCE_CONSTRUCTION,
                format!(
                    "task `{}` uses opaque Path parameter `{}` outside the exact hardened file-read boundary",
                    task.name, parameter.name
                ),
                Some(span),
            )
            .with_related_span("runner-owned Path parameter", parameter.span.clone())
            .with_related_span("structural app start task", task.span.clone())
            .with_help(
                "Pass the runner-owned Path only as the sole argument to `files_read_text`; do not display, compare, store, return, pass elsewhere, inspect, or transform it.",
            ),
        );
    }
}

#[cfg(test)]
fn ten_b3_path_statements_at_consumer_boundary(
    site: &str,
    task: &Task,
) -> Option<Vec<ParsedBodyStatement>> {
    TEN_B3_PATH_BOUNDARY_CORRUPTION.with(|slot| {
        let mut slot = slot.borrow_mut();
        let state = slot.as_mut()?;
        if site != "path-parameter-consumption" || task.name != "run_tool" {
            return None;
        }
        let wrapper = match state.corruption {
            TenB3PathBoundaryCorruption::GroupOccurrence => "group",
            TenB3PathBoundaryCorruption::PermissionOccurrence => "permission",
            TenB3PathBoundaryCorruption::PlaceRootEdge => "field",
            TenB3PathBoundaryCorruption::BuiltinCallOccurrence(_)
            | TenB3PathBoundaryCorruption::ParameterDefinition
            | TenB3PathBoundaryCorruption::ParameterReference
            | TenB3PathBoundaryCorruption::RemoveSourcePathCall
            | TenB3PathBoundaryCorruption::SourceCallTarget => return None,
        };
        state.hits += 1;
        let mut statements = task.body_syntax.clone();
        let ParsedBodyStatementKind::Binding {
            value: Some(value), ..
        } = &mut statements
            .first_mut()
            .expect("10B.3 Path body statement")
            .kind
        else {
            panic!("10B.3 Path corruption requires a binding");
        };
        let CanonicalExpressionKind::Try { value, .. } = &mut value.canonical.kind else {
            panic!("10B.3 Path corruption requires a Try");
        };
        let CanonicalExpressionKind::Call { arguments, .. } = &mut value.kind else {
            panic!("10B.3 Path corruption requires a Call");
        };
        let [argument] = arguments.as_mut_slice() else {
            panic!("10B.3 Path corruption requires one argument");
        };
        let retained = argument.clone();
        argument.kind = match wrapper {
            "group" => CanonicalExpressionKind::Group(Box::new(retained)),
            "permission" => CanonicalExpressionKind::Permission {
                permission: ParamPermission::Borrow,
                value: Box::new(retained),
            },
            "field" => CanonicalExpressionKind::Field {
                base: Box::new(retained),
                field: "foreign".to_string(),
            },
            _ => unreachable!("closed 10B.3 Path corruption"),
        };
        Some(statements)
    })
}

fn is_exact_file_read_consumption(
    statement: &ParsedBodyStatement,
    parameter_definition: &crate::resolve::ResolveDefinitionSummary,
    authority: &PathResolverAuthority,
) -> bool {
    let roots = canonical_statement_roots(statement);
    let calls = roots
        .iter()
        .flat_map(|root| canonical_calls(root))
        .filter_map(|expression| {
            exact_resolver_call_for_canonical_node(expression, authority)
                .map(|call| (expression, call))
        })
        .collect::<Vec<_>>();
    let [(call_node, call)] = calls.as_slice() else {
        return false;
    };
    if call.target_definition_id != "builtin_files_read_text" {
        return false;
    }
    let CanonicalExpressionKind::Call { arguments, .. } = &call_node.kind else {
        return false;
    };
    let CanonicalExpressionKind::Call { callee, .. } = &call_node.kind else {
        return false;
    };
    if call.canonical_callee_node_id() != callee.node_id.as_str()
        || call.canonical_argument_node_ids()
            != arguments
                .iter()
                .map(|argument| argument.node_id.as_str())
                .collect::<Vec<_>>()
    {
        return false;
    }
    let [argument] = arguments.as_slice() else {
        return false;
    };
    matches!(
        &argument.kind,
        CanonicalExpressionKind::Identifier(name) if name == &parameter_definition.name
    ) && canonical_resolved_identifier_nodes(argument, parameter_definition, &authority.references)
        .filter(|(_, reference)| {
            reference.resolved_definition_id.as_deref() == Some(parameter_definition.id.as_str())
                && reference.resolved_definition_semantic_identity.as_deref()
                    == Some(parameter_definition.semantic_identity.as_str())
                && reference.resolution_status == "resolved_v0"
        })
        .count()
        == 1
        && roots
            .iter()
            .flat_map(|root| {
                canonical_resolved_identifier_nodes(
                    root,
                    parameter_definition,
                    &authority.references,
                )
            })
            .count()
            == 1
}

fn exact_resolver_call_for_canonical_node<'a>(
    call_node: &CanonicalExpression,
    authority: &'a PathResolverAuthority,
) -> Option<&'a crate::resolve::ResolveCallOccurrenceSummary> {
    let CanonicalExpressionKind::Call { callee, .. } = &call_node.kind else {
        return None;
    };
    let mut calls = authority
        .calls
        .iter()
        .filter(|call| call.canonical_call_node_id() == call_node.node_id.as_str());
    let call = calls.next()?;
    if calls.next().is_some() {
        return None;
    }
    let mut references = authority.references.iter().filter(|reference| {
        reference.canonical_node_id.as_deref() == Some(callee.node_id.as_str())
    });
    let reference = references.next()?;
    let ordinary_resolution = reference.resolution_status == "resolved_v0"
        && reference.resolved_definition_semantic_identity.as_deref()
            == Some(call.target_definition_id.as_str());
    let builtin_file_read_resolution = reference.resolution_status == "builtin_reference_v0"
        && reference.reason == Some("session_ad_exact_file_read_builtin_v0")
        && call.target_definition_id == "builtin_files_read_text";
    if references.next().is_some() || !(ordinary_resolution || builtin_file_read_resolution) {
        return None;
    }
    Some(call)
}

fn canonical_statement_roots(statement: &ParsedBodyStatement) -> Vec<&CanonicalExpression> {
    if !statement.canonical_extra_occurrences.is_empty() {
        return statement
            .canonical_extra_occurrences
            .iter()
            .map(|expression| &expression.canonical)
            .collect();
    }
    match &statement.kind {
        crate::ast::ParsedBodyStatementKind::Return(expression) => vec![&expression.canonical],
        crate::ast::ParsedBodyStatementKind::Binding { value, .. } => value
            .iter()
            .map(|expression| &expression.canonical)
            .collect(),
        crate::ast::ParsedBodyStatementKind::Other { expressions } => expressions
            .iter()
            .map(|expression| &expression.canonical)
            .collect(),
    }
}

fn canonical_calls(expression: &CanonicalExpression) -> Vec<&CanonicalExpression> {
    let mut calls = Vec::new();
    walk_canonical(expression, &mut |node| {
        if matches!(node.kind, CanonicalExpressionKind::Call { .. }) {
            calls.push(node);
        }
    });
    calls
}

fn canonical_resolved_identifier_nodes<'a>(
    expression: &'a CanonicalExpression,
    definition: &'a crate::resolve::ResolveDefinitionSummary,
    references: &'a [crate::resolve::ResolveReferenceSummary],
) -> impl Iterator<
    Item = (
        &'a CanonicalExpression,
        &'a crate::resolve::ResolveReferenceSummary,
    ),
> {
    let mut nodes = Vec::new();
    walk_canonical(expression, &mut |node| {
        if !matches!(node.kind, CanonicalExpressionKind::Identifier(_)) {
            return;
        }
        for reference in references.iter().filter(|reference| {
            reference.canonical_node_id.as_deref() == Some(node.node_id.as_str())
                && reference.resolved_definition_id.as_deref() == Some(definition.id.as_str())
        }) {
            nodes.push((node, reference));
        }
    });
    nodes.into_iter()
}

fn walk_canonical<'a>(
    expression: &'a CanonicalExpression,
    visit: &mut impl FnMut(&'a CanonicalExpression),
) {
    visit(expression);
    match &expression.kind {
        CanonicalExpressionKind::Field { base, .. }
        | CanonicalExpressionKind::ElementPlace { base, .. }
        | CanonicalExpressionKind::Permission { value: base, .. }
        | CanonicalExpressionKind::Try { value: base, .. }
        | CanonicalExpressionKind::Group(base) => walk_canonical(base, visit),
        CanonicalExpressionKind::ListLiteral(values) => {
            for value in values {
                walk_canonical(value, visit);
            }
        }
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            for (_, value) in fields {
                walk_canonical(value, visit);
            }
        }
        CanonicalExpressionKind::Call { callee, arguments } => {
            walk_canonical(callee, visit);
            for argument in arguments {
                walk_canonical(argument, visit);
            }
        }
        CanonicalExpressionKind::Binary { left, right, .. } => {
            walk_canonical(left, visit);
            walk_canonical(right, visit);
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

struct SourceOwner<'a> {
    kind: &'static str,
    name: &'a str,
    span: &'a Span,
    identity: String,
}

fn check_source_path_construction(
    program: &Program,
    item: &Item,
    owner: SourceOwner<'_>,
    does: Option<&Section>,
    path_callees: &BTreeMap<String, &Task>,
    authority: &PathResolverAuthority,
    diagnostics: &mut PathAnalysis,
) {
    let Some(does) = does else {
        return;
    };
    let body = core_body::analyze_does_section(
        program
            .canonical_core_expectation(item, does)
            .expect("live Path source owner must have parser authority"),
    );
    let canonical_statements = does.body_syntax.iter().flatten().collect::<Vec<_>>();
    assert_eq!(
        body.statements.len(),
        canonical_statements.len(),
        "validated Path body and parser statements must stay aligned"
    );
    for (statement_index, (_statement, parsed)) in body
        .statements
        .into_iter()
        .zip(canonical_statements)
        .enumerate()
    {
        let Some((call_node, callee)) = canonical_statement_roots(parsed)
            .into_iter()
            .flat_map(canonical_calls)
            .find_map(|call_node| {
                path_callee_for_canonical_call(call_node, path_callees, authority)
                    .map(|callee| (call_node, callee))
            })
        else {
            continue;
        };
        let parameter = callee
            .params
            .iter()
            .find(|parameter| contains_path_type(&parameter.ty))
            .expect("Path callee parameter");
        let call_span = call_node.range.start.clone();
        let callee_identity = crate::resolve::semantic_task_identity(program, callee);
        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(107), "path_source_call",
            format!(
                "{}:does-statement-{statement_index}:call-{}:callee-{callee_identity}",
                owner.identity,
                call_node.node_id.as_str()
            ),
            Diagnostic::error(
                DiagnosticCode::PATH_SOURCE_CONSTRUCTION,
                format!(
                    "source call to `{}` attempts to construct opaque Path parameter `{}`",
                    callee.name, parameter.name
                ),
                Some(call_span),
            )
            .with_related_span("runner-owned Path parameter", parameter.span.clone())
            .with_related_span("structural app start task", callee.span.clone())
            .with_related_span(format!("calling {} `{}`", owner.kind, owner.name), owner.span.clone())
            .with_help(
                "Remove the source call. Only structural `hum run` app entry may construct the opaque Path from one native OS argument.",
            ),
        );
    }
}

fn path_callee_for_canonical_call<'a>(
    call_node: &CanonicalExpression,
    path_callees: &'a BTreeMap<String, &Task>,
    authority: &PathResolverAuthority,
) -> Option<&'a Task> {
    let CanonicalExpressionKind::Call { callee, .. } = &call_node.kind else {
        return None;
    };
    let reference_target = {
        let mut references = authority.references.iter().filter(|reference| {
            reference.canonical_node_id.as_deref() == Some(callee.node_id.as_str())
                && reference.resolution_status == "resolved_v0"
        });
        let reference = references.next();
        if references.next().is_some() {
            None
        } else {
            reference
                .and_then(|reference| reference.resolved_definition_semantic_identity.as_deref())
        }
    };
    let occurrence = {
        let mut calls = authority
            .calls
            .iter()
            .filter(|call| call.canonical_call_node_id() == call_node.node_id.as_str());
        let call = calls.next();
        if calls.next().is_some() { None } else { call }
    };
    let occurrence_target = occurrence.map(|call| call.target_definition_id.as_str());
    let target = reference_target
        .filter(|target| path_callees.contains_key(*target))
        .or_else(|| occurrence_target.filter(|target| path_callees.contains_key(*target)))?;
    path_callees.get(target).copied()
}

fn invalid_declaration(message: &str, span: &Span, owner: Option<&Span>) -> Diagnostic {
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::INVALID_PATH_BOUNDARY,
        message,
        Some(span.clone()),
    )
    .with_help(
        "Keep exactly one `Path` parameter only on the structural app start task; Path has no source construction, return, or storage surface in Session AB.",
    );
    if let Some(owner) = owner {
        diagnostic = diagnostic.with_related_span("owning declaration", owner.clone());
    }
    diagnostic
}

fn emit(
    analysis: &mut PathAnalysis,
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    node_role: &'static str,
    semantic_node: String,
    diagnostic: Diagnostic,
) {
    let semantic_origin = format!("path-boundary:{semantic_node}:role={node_role}");
    let route = vec![
        format!("path_semantic_node={semantic_node}"),
        format!("path_node_role={node_role}"),
    ];
    emit_with_identity(
        analysis,
        cause_key,
        node_role,
        semantic_origin,
        route,
        diagnostic,
    );
}

fn emit_with_identity(
    analysis: &mut PathAnalysis,
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    node_role: &'static str,
    semantic_origin: String,
    mut route: Vec<String>,
    diagnostic: Diagnostic,
) {
    route.push(format!("path_node_role={node_role}"));
    let (diagnostic, occurrence) =
        DiagnosticOccurrence::producer_diagnostic(cause_key, diagnostic, semantic_origin, route)
            .expect("Path diagnostic cause and producer identity must be registered");
    analysis
        .diagnostic_occurrences
        .insert_owned(occurrence)
        .expect("Path diagnostic occurrences must be unique");
    analysis.diagnostics.push(diagnostic);
}

fn contains_path_type(type_text: &str) -> bool {
    type_text
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|token| token == "Path")
}

#[cfg(test)]
mod tests {
    use crate::parser;

    use super::{
        PathResolverAuthority, TenB3PathBoundaryCorruption, analyze_at_resolver_boundary,
        diagnostics, with_ten_b3_path_boundary_corruption,
    };

    const TEN_B3_PATH_SOURCE: &str = r#"app file_probe {
  why:
    consume one opaque path
  uses:
    files.read
  starts with:
    run_tool
  task run_tool(input: Path) -> Result Unit, FileReadError {
    uses:
      files.read
    fails when:
      exact file reading fails
    allocates:
      one bounded file buffer
    does:
      let text = try files_read_text(input)
      return
  }
}"#;

    fn program(source: &str) -> crate::ast::Program {
        let parsed = parser::parse_source("path.hum", source);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        crate::ast::Program {
            files: vec![parsed.file],
        }
    }

    fn ten_b3_program_at_index(index: usize) -> crate::ast::Program {
        let parsed = parser::parse_source_at_index("ten-b3-path.hum", TEN_B3_PATH_SOURCE, index);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        if index == 0 {
            crate::ast::Program {
                files: vec![parsed.file],
            }
        } else {
            let dummy =
                parser::parse_source_at_index("ten-b3-dummy.hum", "module tests.dummy\n", 0);
            crate::ast::Program {
                files: vec![dummy.file, parsed.file],
            }
        }
    }

    fn resolver_authority(program: &crate::ast::Program) -> PathResolverAuthority {
        PathResolverAuthority {
            definitions: crate::resolve::resolve_definition_summaries(program, &[]),
            references: crate::resolve::resolve_reference_summaries(program, &[]),
            calls: crate::resolve::resolve_call_occurrence_summaries(program, &[]),
        }
    }

    fn path_observation(analysis: &super::PathAnalysis) -> Vec<String> {
        analysis
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let span = diagnostic.span.as_ref().expect("Path diagnostic span");
                format!(
                    "{}|{}|{}:{}:{}",
                    diagnostic.code.as_str(),
                    diagnostic.message,
                    span.file.replace('\\', "/"),
                    span.line,
                    span.column
                )
            })
            .collect()
    }

    #[test]
    fn accepts_one_inert_path_only_on_structural_start() {
        let program = program(
            r#"app path_probe {
  why:
    prove opaque path entry
  starts with:
    run_tool
  task run_tool(input: Path) -> Unit {
    ensures:
      input == input according to unchecked prose
    does:
      let label = "input"
      return
  }
}"#,
        );
        assert!(diagnostics(&program).is_empty());
    }

    #[test]
    fn accepts_only_exact_hardened_file_read_consumption_of_path() {
        let program = program(
            r#"app file_probe {
  why:
    consume one opaque path
  uses:
    files.read
  starts with:
    run_tool
  task run_tool(input: Path) -> Result Unit, FileReadError {
    uses:
      files.read
    fails when:
      exact file reading fails
    allocates:
      one bounded file buffer
    does:
      let text = try files_read_text(input)
      return
  }
}"#,
        );
        assert!(diagnostics(&program).is_empty());
    }

    #[test]
    fn rejects_multiple_storage_use_and_source_construction() {
        let multiple = program(
            r#"app path_probe {
  why:
    reject two paths
  starts with:
    run_tool
  task run_tool(left: Path, right: Path) -> Unit {
    does:
      return
  }
}"#,
        );
        let multiple_diagnostics = diagnostics(&multiple);
        assert_eq!(multiple_diagnostics[0].code.as_str(), "H0629");
        let multiple_task = match &multiple.files[0].items[0] {
            crate::ast::Item::App(app) => match &app.items[0] {
                crate::ast::Item::Task(task) => task,
                other => panic!("expected task, got {other:?}"),
            },
            other => panic!("expected app, got {other:?}"),
        };
        assert_eq!(multiple_task.params[0].span.column, 17);
        assert_eq!(multiple_task.params[1].span.column, 29);

        let used = program(
            r#"app path_probe {
  why:
    reject source use
  starts with:
    run_tool
  task run_tool(input: Path) -> Unit {
    does:
      let saved = input
      return
  }
}"#,
        );
        assert_eq!(diagnostics(&used)[0].code.as_str(), "H0630");

        let literal = program(
            r#"app path_probe {
  why:
    reject source construction
  starts with:
    run_tool
  task run_tool(input: Path) -> Unit {
    does:
      return
  }
  task misuse -> Unit {
    does:
      run_tool("source Text is not a Path")
      return
  }
}"#,
        );
        assert_eq!(diagnostics(&literal)[0].code.as_str(), "H0630");
    }

    #[test]
    fn rejects_path_use_in_contracts_and_test_expectations() {
        let contract = program(
            r#"app path_probe {
  why:
    reject contract comparison
  starts with:
    run_tool
  task run_tool(input: Path) -> Unit {
    ensures:
      input == input
    does:
      return
  }
}"#,
        );
        let contract_diagnostics = diagnostics(&contract);
        assert_eq!(contract_diagnostics.len(), 1);
        assert_eq!(contract_diagnostics[0].code.as_str(), "H0630");
        assert_eq!(contract_diagnostics[0].span.as_ref().unwrap().line, 8);

        let test_body = program(
            r#"app path_probe {
  why:
    reject test construction
  starts with:
    run_tool
  task run_tool(input: Path) -> Unit {
    does:
      return
  }
  test source_construction unit {
    does:
      expect run_tool("not a native Path") returns Unit
  }
}"#,
        );
        let test_diagnostics = diagnostics(&test_body);
        assert_eq!(test_diagnostics.len(), 1);
        assert_eq!(test_diagnostics[0].code.as_str(), "H0630");
        assert_eq!(test_diagnostics[0].span.as_ref().unwrap().line, 12);
    }

    #[test]
    fn ten_b3_path_real_path_uses_resolver_owned_call_and_place_identity() {
        let program = ten_b3_program_at_index(0);
        let authority = resolver_authority(&program);
        let analysis = analyze_at_resolver_boundary(&program, &authority);
        assert_eq!(path_observation(&analysis), Vec::<String>::new());

        let call = authority
            .calls
            .iter()
            .find(|call| call.target_definition_id == "builtin_files_read_text")
            .expect("real file-read resolver call");
        assert_eq!(call.source(), "files_read_text(input)");
        assert_eq!(call.canonical_argument_node_ids().len(), 1);
        let reference = authority
            .references
            .iter()
            .find(|reference| {
                reference.name == "input"
                    && reference.canonical_node_id.as_deref()
                        == call
                            .canonical_argument_node_ids()
                            .first()
                            .map(String::as_str)
            })
            .expect("real Path argument reference");
        assert_eq!(reference.resolution_status, "resolved_v0");
        assert!(reference.resolved_definition_semantic_identity.is_some());
    }

    #[test]
    fn ten_b3_path_canonical_corruption_and_substitution_fail_closed() {
        let left = ten_b3_program_at_index(0);
        let clean_authority = resolver_authority(&left);
        assert_eq!(
            path_observation(&analyze_at_resolver_boundary(&left, &clean_authority)),
            Vec::<String>::new()
        );

        let right = ten_b3_program_at_index(1);
        let right_authority = resolver_authority(&right);
        let left_call = clean_authority
            .calls
            .iter()
            .find(|call| call.target_definition_id == "builtin_files_read_text")
            .expect("left file-read call");
        let right_call = right_authority
            .calls
            .iter()
            .find(|call| call.target_definition_id == "builtin_files_read_text")
            .expect("right file-read call");
        assert_eq!(left_call.exact_call_span, right_call.exact_call_span);
        assert_eq!(left_call.source(), right_call.source());
        assert_ne!(
            left_call.canonical_call_node_id(),
            right_call.canonical_call_node_id()
        );

        let expected = ["H0630|task `run_tool` uses opaque Path parameter `input` outside the exact hardened file-read boundary|ten-b3-path.hum:16:7".to_string()];
        for _ in 0..2 {
            for corruption in [
                TenB3PathBoundaryCorruption::BuiltinCallOccurrence(
                    right_call.canonical_call_node_id().to_string(),
                ),
                TenB3PathBoundaryCorruption::ParameterDefinition,
                TenB3PathBoundaryCorruption::ParameterReference,
                TenB3PathBoundaryCorruption::GroupOccurrence,
                TenB3PathBoundaryCorruption::PermissionOccurrence,
                TenB3PathBoundaryCorruption::PlaceRootEdge,
            ] {
                assert_eq!(
                    path_observation(&with_ten_b3_path_boundary_corruption(corruption, || {
                        super::analyze(&left)
                    })),
                    expected
                );
            }
        }

        let source_call = program(
            r#"app source_call {
  why:
    reject source construction
  starts with:
    run_tool
  task run_tool(input: Path) -> Unit {
    does:
      return
  }
  task harmless(value: Text) -> Unit {
    does:
      return
  }
  task misuse -> Unit {
    does:
      run_tool("ordinary text")
      return
  }
}"#,
        );
        let source_call_expected = ["H0630|source call to `run_tool` attempts to construct opaque Path parameter `input`|path.hum:16:7".to_string()];
        assert_eq!(
            path_observation(&super::analyze(&source_call)),
            source_call_expected
        );
        assert_eq!(
            path_observation(&with_ten_b3_path_boundary_corruption(
                TenB3PathBoundaryCorruption::RemoveSourcePathCall,
                || super::analyze(&source_call),
            )),
            source_call_expected,
            "missing call-occurrence authority must fail closed through H0630"
        );
        assert_eq!(
            path_observation(&with_ten_b3_path_boundary_corruption(
                TenB3PathBoundaryCorruption::SourceCallTarget,
                || super::analyze(&source_call),
            )),
            Vec::<String>::new(),
            "resolver retargeting must change the Path consumer result instead of falling back by name"
        );
    }

    #[test]
    fn ten_b3_path_source_audit_rejects_expression_text_authority() {
        let source = include_str!("path_boundary.rs");
        let start = source
            .find("fn is_exact_file_read_consumption(")
            .expect("Path consumer start");
        let end = source[start..]
            .find("\nfn canonical_statement_roots(")
            .map(|offset| start + offset)
            .expect("Path consumer end");
        let consumer = &source[start..end];
        for prohibited in [
            "statement.text",
            "statement_expression(",
            "calls_in_expression(",
            "visible_view_source_root(",
            "resolved.or(structured)",
            "task.name ==",
        ] {
            assert!(
                !consumer.contains(prohibited),
                "Path consumer restored prohibited authority: {prohibited}"
            );
        }
        assert!(consumer.contains("call.canonical_call_node_id()"));
        assert!(consumer.contains("canonical_resolved_identifier_nodes("));

        let source_start = source
            .find("fn path_callee_for_canonical_call<")
            .expect("Path source-call consumer start");
        let source_end = source[source_start..]
            .find("\nfn invalid_declaration(")
            .map(|offset| source_start + offset)
            .expect("Path source-call consumer end");
        let source_consumer = &source[source_start..source_end];
        for prohibited in ["resolved.or(structured)", "task.name", "Identifier(name)"] {
            assert!(
                !source_consumer.contains(prohibited),
                "Path source-call consumer restored name fallback: {prohibited}"
            );
        }
        assert!(source_consumer.contains("reference.resolved_definition_semantic_identity"));
        assert!(source_consumer.contains("call.canonical_call_node_id()"));
        assert!(source_consumer.contains("call.target_definition_id"));
    }
}
