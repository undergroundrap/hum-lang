use std::collections::BTreeMap;

use crate::ast::{App, Item, Program, Section, Task};
use crate::core_body;
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOccurrence, Span};
use crate::graph::is_meaningful_line_text;
use crate::typed_failure;

pub(crate) fn diagnostics(program: &Program) -> Vec<Diagnostic> {
    analyze(program).diagnostics
}

#[derive(Default)]
struct PathAnalysis {
    diagnostics: Vec<Diagnostic>,
    diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diagnostic_occurrence_set(
    program: &Program,
) -> crate::diagnostic::DiagnosticOccurrenceSet {
    analyze(program).diagnostic_occurrences
}

fn analyze(program: &Program) -> PathAnalysis {
    let mut diagnostics = PathAnalysis::default();
    for file in &program.files {
        check_scope(program, &file.items, None, &mut diagnostics);
    }
    diagnostics
}

fn check_scope(
    program: &Program,
    items: &[Item],
    selected: Option<&Task>,
    diagnostics: &mut PathAnalysis,
) {
    for item in items {
        let item_identity = crate::resolve::semantic_item_identity_for(program, item);
        match item {
            Item::App(app) => check_scope(program, &app.items, local_start_task(app), diagnostics),
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
            Item::Task(task) => {
                check_task_signature(program, task, &item_identity, selected, diagnostics)
            }
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
                Some((task.name.as_str(), task))
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
                SourceOwner {
                    kind: "task",
                    name: &task.name,
                    span: &task.span,
                    identity: crate::resolve::semantic_item_identity_for(program, item),
                },
                task.section("does"),
                &callable_paths,
                diagnostics,
            ),
            Item::Test(test) => check_source_path_construction(
                program,
                SourceOwner {
                    kind: "test",
                    name: &test.name,
                    span: &test.span,
                    identity: crate::resolve::semantic_item_identity_for(program, item),
                },
                test.section("does"),
                &callable_paths,
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
    let contract_use = ["needs", "ensures"].into_iter().find_map(|name| {
        task.section(name).and_then(|section| {
            section.lines.iter().find_map(|line| {
                let fact = crate::predicate::fact_for_line(program, task, name, line)?;
                (is_meaningful_line_text(&line.text)
                    && fact.ast.is_some()
                    && fact.reason == "opaque_path_inspection_owned_by_h0630"
                    && contains_identifier(&line.text, &parameter.name))
                .then(|| (line, fact.semantic_line_identity().to_string()))
            })
        })
    });
    let body_use = task.section("does").and_then(|does| {
        let body = core_body::analyze_does_section(does);
        body.statements
            .into_iter()
            .enumerate()
            .find(|(_, statement)| {
                contains_identifier(&statement.text, &parameter.name)
                    && !is_exact_file_read_consumption(statement, &parameter.name)
            })
    });
    let task_identity = crate::resolve::semantic_task_identity(program, task);
    let source_use = contract_use
        .map(|(line, line_identity)| (line.span.clone(), line_identity))
        .or_else(|| {
            body_use.map(|(index, statement)| {
                (
                    statement.span,
                    format!("{task_identity}:does-statement-{index}"),
                )
            })
        });
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

fn is_exact_file_read_consumption(statement: &core_body::BodyStatement, parameter: &str) -> bool {
    let Some(expression) = expression_text(&statement.text) else {
        return false;
    };
    let Some(call) = typed_failure::calls_in_expression(expression)
        .into_iter()
        .find(|call| call.callee == "files_read_text")
    else {
        return false;
    };
    let Some(arguments) = call
        .source
        .strip_prefix("files_read_text(")
        .and_then(|source| source.strip_suffix(')'))
    else {
        return false;
    };
    arguments.trim() == parameter && identifier_occurrences(&statement.text, parameter) == 1
}

fn identifier_occurrences(text: &str, expected: &str) -> usize {
    let mut count = 0usize;
    let mut token = String::new();
    let mut in_string = false;
    for character in text.chars().chain(std::iter::once(' ')) {
        if character == '"' {
            if !in_string && token == expected {
                count += 1;
            }
            token.clear();
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if character.is_ascii_alphanumeric() || character == '_' {
            token.push(character);
        } else {
            if token == expected {
                count += 1;
            }
            token.clear();
        }
    }
    count
}

struct SourceOwner<'a> {
    kind: &'static str,
    name: &'a str,
    span: &'a Span,
    identity: String,
}

fn check_source_path_construction(
    program: &Program,
    owner: SourceOwner<'_>,
    does: Option<&Section>,
    path_callees: &BTreeMap<&str, &Task>,
    diagnostics: &mut PathAnalysis,
) {
    let Some(does) = does else {
        return;
    };
    let body = core_body::analyze_does_section(does);
    for (statement_index, statement) in body.statements.into_iter().enumerate() {
        let Some(expression) = expression_text(&statement.text) else {
            continue;
        };
        let Some(call) = typed_failure::calls_in_expression(expression)
            .into_iter()
            .find(|call| path_callees.contains_key(call.callee.as_str()))
        else {
            continue;
        };
        let callee = path_callees[call.callee.as_str()];
        let parameter = callee
            .params
            .iter()
            .find(|parameter| contains_path_type(&parameter.ty))
            .expect("Path callee parameter");
        let expression_offset = statement.text.find(expression).unwrap_or(0);
        let call_span = Span {
            file: statement.span.file.clone(),
            line: statement.span.line,
            column: statement.span.column
                + statement.text[..expression_offset + call.source_offset]
                    .chars()
                    .count(),
        };
        let callee_identity = crate::resolve::semantic_task_identity(program, callee);
        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(107), "path_source_call",
            format!("{}:does-statement-{statement_index}:callee-{callee_identity}", owner.identity),
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

fn contains_identifier(text: &str, expected: &str) -> bool {
    let mut token = String::new();
    let mut in_string = false;
    for character in text.chars().chain(std::iter::once(' ')) {
        if character == '"' {
            if !in_string && token == expected {
                return true;
            }
            token.clear();
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if character.is_ascii_alphanumeric() || character == '_' {
            token.push(character);
        } else {
            if token == expected {
                return true;
            }
            token.clear();
        }
    }
    false
}

fn expression_text(statement: &str) -> Option<&str> {
    let text = statement.trim();
    for keyword in ["return", "let", "change", "set"] {
        if let Some(rest) = text.strip_prefix(keyword)
            && rest.starts_with(char::is_whitespace)
        {
            let rest = rest.trim();
            return if matches!(keyword, "let" | "change" | "set") {
                rest.split_once('=')
                    .map(|(_, expression)| expression.trim())
            } else {
                Some(rest)
            };
        }
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use crate::parser;

    use super::diagnostics;

    fn program(source: &str) -> crate::ast::Program {
        let parsed = parser::parse_source("path.hum", source);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        crate::ast::Program {
            files: vec![parsed.file],
        }
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
}
