use crate::ast::{App, Item, Program, Section, SectionLine, SourceFile, Task};
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOccurrence};
use crate::graph::is_meaningful_line_text;
use crate::typed_failure;

#[derive(Debug)]
pub struct AppEntry<'a> {
    pub app: &'a App,
    pub task: &'a Task,
}

#[derive(Debug)]
pub struct Analysis<'a> {
    pub entry: Option<AppEntry<'a>>,
    pub diagnostic: Option<Diagnostic>,
    pub(crate) diagnostic_occurrence: Option<DiagnosticOccurrence>,
}

pub fn analyze(program: &Program) -> Analysis<'_> {
    let apps = top_level_apps(program);
    match apps.as_slice() {
        [] => Analysis {
            entry: None,
            diagnostic: None,
            diagnostic_occurrence: None,
        },
        [app] => analyze_app(program, app),
        [first, second, rest @ ..] => {
            let first_id = crate::resolve::semantic_app_identity(program, first);
            let second_id = crate::resolve::semantic_app_identity(program, second);
            let mut diagnostic = Diagnostic::error(
                DiagnosticCode::MULTIPLE_EXECUTABLE_APPS,
                format!(
                    "run input contains multiple top-level apps; `{}` and `{}` both claim the executable root",
                    first.name, second.name
                ),
                Some(second.span.clone()),
            )
            .with_related_span(format!("first app `{}`", first.name), first.span.clone())
            .with_help(
                "Run exactly one top-level app input, or use `--entry <task>` for a direct legacy task probe.",
            );
            for app in rest {
                diagnostic = diagnostic
                    .with_related_span(format!("additional app `{}`", app.name), app.span.clone());
            }
            rejected(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(89),
                format!("app-set:first={first_id}:second={second_id}"),
                vec![
                    format!("first_app_identity={first_id}"),
                    format!("second_app_identity={second_id}"),
                    format!("top_level_app_count={}", apps.len()),
                ],
                diagnostic,
            )
        }
    }
}

pub fn diagnostics(program: &Program) -> Vec<Diagnostic> {
    analyze(program).diagnostic.into_iter().collect()
}

pub(crate) fn diagnostics_for_file_with_semantic_index(
    file: &SourceFile,
    semantic_file_index: usize,
) -> (Vec<Diagnostic>, crate::diagnostic::DiagnosticOccurrenceSet) {
    let mut files = (0..semantic_file_index)
        .map(|index| SourceFile {
            path: format!("<semantic-file-{index}>"),
            module: None,
            items: Vec::new(),
        })
        .collect::<Vec<_>>();
    files.push(file.clone());
    let program = Program { files };
    let analysis = analyze(&program);
    let diagnostics = analysis.diagnostic.clone().into_iter().collect();
    let mut occurrences = crate::diagnostic::DiagnosticOccurrenceSet::default();
    if let Some(occurrence) = analysis.diagnostic_occurrence {
        occurrences
            .insert_owned(occurrence)
            .expect("file app-entry diagnostic occurrence must be unique");
    }
    (diagnostics, occurrences)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn diagnostic_occurrence_set(
    program: &Program,
) -> crate::diagnostic::DiagnosticOccurrenceSet {
    let analysis = analyze(program);
    let mut occurrences = crate::diagnostic::DiagnosticOccurrenceSet::default();
    if let Some(occurrence) = analysis.diagnostic_occurrence {
        occurrences
            .insert_owned(occurrence)
            .expect("app-entry diagnostic occurrences must be unique");
    }
    occurrences
}

pub fn is_app_entry_diagnostic(diagnostic: &Diagnostic) -> bool {
    is_app_entry_code(diagnostic.code)
}

pub(crate) fn is_app_entry_code(code: DiagnosticCode) -> bool {
    matches!(
        code,
        DiagnosticCode::APP_START_MISSING
            | DiagnosticCode::APP_START_EMPTY
            | DiagnosticCode::APP_START_DUPLICATE
            | DiagnosticCode::APP_START_INVALID_NAME
            | DiagnosticCode::APP_START_NOT_CHILD
            | DiagnosticCode::MULTIPLE_EXECUTABLE_APPS
            | DiagnosticCode::APP_START_INVALID_RESULT
    )
}

fn analyze_app<'a>(program: &'a Program, app: &'a App) -> Analysis<'a> {
    let app_identity = crate::resolve::semantic_app_identity(program, app);
    let starts = app
        .sections
        .iter()
        .filter(|section| section.name == "starts with")
        .collect::<Vec<_>>();
    let section = match starts.as_slice() {
        [] => {
            return rejected(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(84),
                format!("app-start:{app_identity}"),
                vec![format!("app_identity={app_identity}")],
                Diagnostic::error(
                    DiagnosticCode::APP_START_MISSING,
                    format!("executable app `{}` has no `starts with:` section", app.name),
                    Some(app.span.clone()),
                )
                .with_help(
                    "Add one `starts with:` section containing the bare name of one task directly nested in this app.",
                ),
            );
        }
        [section] => *section,
        [first, second, rest @ ..] => {
            let mut diagnostic = Diagnostic::error(
                DiagnosticCode::APP_START_DUPLICATE,
                format!("app `{}` declares `starts with:` more than once", app.name),
                Some(second.span.clone()),
            )
            .with_related_span(format!("app `{}`", app.name), app.span.clone())
            .with_related_span("first `starts with:` section", first.span.clone())
            .with_help(
                "Keep exactly one `starts with:` section with exactly one meaningful bare task name.",
            );
            for section in rest {
                diagnostic = diagnostic
                    .with_related_span("additional `starts with:` section", section.span.clone());
            }
            return rejected(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(86),
                format!("app-start-sections:{app_identity}"),
                vec![
                    format!("app_identity={app_identity}"),
                    format!("start_section_count={}", starts.len()),
                    format!(
                        "first_start_section={}",
                        section_identity(&app_identity, app, first)
                    ),
                    format!(
                        "second_start_section={}",
                        section_identity(&app_identity, app, second)
                    ),
                ],
                diagnostic,
            );
        }
    };

    let lines = meaningful_lines(section);
    let start_section_identity = section_identity(&app_identity, app, section);
    let start_line = match lines.as_slice() {
        [] => {
            return rejected(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(85),
                format!("app-start-reference:{start_section_identity}"),
                vec![
                    format!("app_identity={app_identity}"),
                    format!("start_section_identity={start_section_identity}"),
                    format!("start_line_count={}", lines.len()),
                ],
                Diagnostic::error(
                    DiagnosticCode::APP_START_EMPTY,
                    format!("app `{}` has an empty `starts with:` section", app.name),
                    Some(section.span.clone()),
                )
                .with_related_span(format!("app `{}`", app.name), app.span.clone())
                .with_help(
                    "Put exactly one bare snake_case direct-child task name under `starts with:`.",
                ),
            );
        }
        [line] => *line,
        [first, second, rest @ ..] => {
            let mut diagnostic = Diagnostic::error(
                DiagnosticCode::APP_START_DUPLICATE,
                format!("app `{}` declares more than one start task", app.name),
                Some(second.span.clone()),
            )
            .with_related_span(format!("app `{}`", app.name), app.span.clone())
            .with_related_span("first start declaration", first.span.clone())
            .with_help(
                "Keep one meaningful bare task name under the single `starts with:` section.",
            );
            for line in rest {
                diagnostic =
                    diagnostic.with_related_span("additional start declaration", line.span.clone());
            }
            return rejected(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(86),
                format!("app-start-lines:{start_section_identity}"),
                vec![
                    format!("app_identity={app_identity}"),
                    format!("start_section_identity={start_section_identity}"),
                    format!(
                        "first_start_reference={}",
                        line_identity(&start_section_identity, section, first)
                    ),
                    format!(
                        "second_start_reference={}",
                        line_identity(&start_section_identity, section, second)
                    ),
                    format!("start_line_count={}", lines.len()),
                ],
                diagnostic,
            );
        }
    };

    let name = start_line.text.trim();
    let start_reference_identity = line_identity(&start_section_identity, section, start_line);
    if !is_value_identifier(name) {
        return rejected(
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(87),
            format!("app-start-reference:{start_reference_identity}"),
            vec![
                format!("app_identity={app_identity}"),
                format!("start_reference_identity={start_reference_identity}"),
            ],
            Diagnostic::error(
                DiagnosticCode::APP_START_INVALID_NAME,
                format!(
                    "app `{}` start `{name}` is not one bare snake_case task name",
                    app.name
                ),
                Some(start_line.span.clone()),
            )
            .with_related_span(format!("app `{}`", app.name), app.span.clone())
            .with_help(
                "Use only a direct-child task name such as `run_tool`; do not write a call, path, assignment, or state initializer.",
            ),
        );
    }

    let task = app.items.iter().find_map(|item| match item {
        Item::Task(task) if task.name == name => Some(task),
        _ => None,
    });
    let Some(task) = task else {
        let non_child = find_non_child_task(program, app, name);
        let message = if non_child.is_some() {
            format!(
                "app `{}` start `{name}` names a task that is not a direct child",
                app.name
            )
        } else {
            format!(
                "app `{}` start `{name}` does not name a directly nested task",
                app.name
            )
        };
        let mut diagnostic = Diagnostic::error(
            DiagnosticCode::APP_START_NOT_CHILD,
            message,
            Some(start_line.span.clone()),
        )
        .with_related_span(format!("app `{}`", app.name), app.span.clone())
        .with_help(format!(
            "Nest task `{name}` directly inside app `{}` or change `starts with:` to an existing direct child; app mode never falls back to a same-named external task.",
            app.name
        ));
        let has_non_child = non_child.is_some();
        if let Some(non_child) = non_child {
            diagnostic = diagnostic.with_related_span(
                format!("non-child task `{name}` is not selectable"),
                non_child.span.clone(),
            );
        }
        return rejected(
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(88),
            format!("app-start-reference:{start_reference_identity}"),
            vec![
                format!("app_identity={app_identity}"),
                format!("start_reference_identity={start_reference_identity}"),
                format!("non_child_candidate={has_non_child}"),
                non_child
                    .map(|task| {
                        format!(
                            "non_child_task_identity={}",
                            crate::resolve::semantic_task_identity(program, task)
                        )
                    })
                    .unwrap_or_else(|| "non_child_task_identity=none".to_string()),
            ],
            diagnostic,
        );
    };

    if !valid_start_result(task) {
        let declared = task.result.as_deref().unwrap_or("implicit Unit");
        let task_identity = crate::resolve::semantic_task_identity(program, task);
        return rejected(
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(90),
            format!("app-start-task:{task_identity}"),
            vec![
                format!("app_identity={app_identity}"),
                format!("start_reference_identity={start_reference_identity}"),
                format!("start_task_identity={task_identity}"),
            ],
            Diagnostic::error(
                DiagnosticCode::APP_START_INVALID_RESULT,
                format!(
                    "app `{}` start task `{name}` returns `{declared}`; an app start must return `Unit` or `Result Unit, E`",
                    app.name
                ),
                Some(task.span.clone()),
            )
            .with_related_span(format!("app `{}`", app.name), app.span.clone())
            .with_related_span("start declaration", start_line.span.clone())
            .with_help(
                "Change the start task result to `Unit` (including an omitted result) or `Result Unit, ErrorType`.",
            ),
        );
    }

    Analysis {
        entry: Some(AppEntry { app, task }),
        diagnostic: None,
        diagnostic_occurrence: None,
    }
}

fn section_identity(app_identity: &str, app: &App, target: &Section) -> String {
    let index = app
        .sections
        .iter()
        .position(|section| std::ptr::eq(section, target))
        .expect("start section belongs to app");
    format!("{app_identity}:section-{index}")
}

fn line_identity(section_identity: &str, section: &Section, target: &SectionLine) -> String {
    let index = section
        .lines
        .iter()
        .position(|line| std::ptr::eq(line, target))
        .expect("start line belongs to section");
    format!("{section_identity}:line-{index}")
}

fn rejected(
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    semantic_origin: String,
    relationship_route: Vec<String>,
    diagnostic: Diagnostic,
) -> Analysis<'static> {
    let (diagnostic, diagnostic_occurrence) = DiagnosticOccurrence::producer_diagnostic(
        cause_key,
        diagnostic,
        semantic_origin,
        relationship_route,
    )
    .expect("app-entry diagnostic cause must be producer-owned");
    Analysis {
        entry: None,
        diagnostic: Some(diagnostic),
        diagnostic_occurrence: Some(diagnostic_occurrence),
    }
}

fn top_level_apps(program: &Program) -> Vec<&App> {
    program
        .files
        .iter()
        .flat_map(|file| &file.items)
        .filter_map(|item| match item {
            Item::App(app) => Some(app),
            _ => None,
        })
        .collect()
}

fn meaningful_lines(section: &Section) -> Vec<&SectionLine> {
    section
        .lines
        .iter()
        .filter(|line| is_meaningful_line_text(&line.text))
        .collect()
}

fn valid_start_result(task: &Task) -> bool {
    match task.result.as_deref().map(str::trim) {
        None | Some("") | Some("Unit") => true,
        Some(result) => {
            typed_failure::result_success_type(result).as_deref() == Some("Unit")
                && typed_failure::result_error_root(result).is_some()
        }
    }
}

fn find_non_child_task<'a>(program: &'a Program, target: &'a App, name: &str) -> Option<&'a Task> {
    for item in &target.items {
        if let Item::App(app) = item
            && let Some(task) = app
                .items
                .iter()
                .find_map(|item| find_task_in_item(item, name))
        {
            return Some(task);
        }
    }
    for file in &program.files {
        for item in &file.items {
            if matches!(item, Item::App(app) if std::ptr::eq(app, target)) {
                continue;
            }
            if let Some(task) = find_task_in_item(item, name) {
                return Some(task);
            }
        }
    }
    None
}

fn find_task_in_item<'a>(item: &'a Item, name: &str) -> Option<&'a Task> {
    match item {
        Item::Task(task) if task.name == name => Some(task),
        Item::App(app) => app
            .items
            .iter()
            .find_map(|item| find_task_in_item(item, name)),
        _ => None,
    }
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
    use crate::parser;

    use super::analyze;

    fn program(source: &str) -> crate::ast::Program {
        let parsed = parser::parse_source("app.hum", source);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        crate::ast::Program {
            files: vec![parsed.file],
        }
    }

    #[test]
    fn selects_direct_child_unit_task() {
        let program = program(
            r#"app tool {
  why:
    prove structural selection
  starts with:
    run_tool
  task run_tool -> Unit {
    does:
      return
  }
}"#,
        );
        let analysis = analyze(&program);
        assert!(analysis.diagnostic.is_none());
        let entry = analysis.entry.expect("app entry");
        assert_eq!(entry.app.name, "tool");
        assert_eq!(entry.task.name, "run_tool");
    }

    #[test]
    fn external_same_name_is_related_but_not_selected() {
        let program = program(
            r#"task run_tool -> Unit {
  does:
    return
}
app tool {
  why:
    prove lexical app selection
  starts with:
    run_tool
}"#,
        );
        let diagnostic = analyze(&program).diagnostic.expect("diagnostic");
        assert_eq!(diagnostic.code.as_str(), "H0614");
        assert!(
            diagnostic
                .related_spans
                .iter()
                .any(|related| related.label == "non-child task `run_tool` is not selectable")
        );
    }

    #[test]
    fn app_occurrence_identity_is_structural_and_display_name_independent() {
        fn two_apps(first: &str, second: &str) -> crate::ast::Program {
            program(&format!(
                r#"app {first} {{
  starts with:
    run_tool
  task run_tool -> Unit {{
    does:
      return
  }}
}}
app {second} {{
  starts with:
    run_tool
  task run_tool -> Unit {{
    does:
      return
  }}
}}"#
            ))
        }

        let same_named = analyze(&two_apps("tool", "tool"))
            .diagnostic_occurrence
            .expect("multiple-app occurrence");
        let renamed = analyze(&two_apps("first_tool", "second_tool"))
            .diagnostic_occurrence
            .expect("renamed multiple-app occurrence");
        assert_eq!(same_named.semantic_origin(), renamed.semantic_origin());
        assert_eq!(
            same_named.relationship_route(),
            renamed.relationship_route()
        );
        assert_ne!(
            same_named.relationship_route()[0],
            same_named.relationship_route()[1],
            "same display names must retain distinct lexical app identities"
        );
    }
}
