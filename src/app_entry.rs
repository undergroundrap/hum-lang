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

#[derive(Debug)]
pub(crate) struct CanonicalNativeLayout<'a> {
    pub(crate) file: &'a SourceFile,
    pub(crate) app: &'a App,
    pub(crate) entry: &'a Task,
    pub(crate) normalized_path: String,
}

#[derive(Debug)]
pub(crate) struct CanonicalNativeLayoutAnalysis<'a> {
    pub(crate) layout: Option<CanonicalNativeLayout<'a>>,
    pub(crate) diagnostic: Option<Diagnostic>,
    pub(crate) diagnostic_occurrence: Option<DiagnosticOccurrence>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) reason: Option<&'static str>,
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

#[cfg_attr(not(test), allow(dead_code))]
pub fn diagnostics(program: &Program) -> Vec<Diagnostic> {
    analyze(program).diagnostic.into_iter().collect()
}

pub(crate) fn diagnostics_for_file_with_semantic_index(
    file: &SourceFile,
    semantic_file_index: usize,
) -> (Vec<Diagnostic>, crate::diagnostic::DiagnosticOccurrenceSet) {
    let mut files = (0..semantic_file_index)
        .map(|index| {
            SourceFile::empty_non_authoritative(
                format!("<semantic-file-{index}>"),
                None,
                Vec::new(),
            )
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

pub(crate) fn analyze_canonical_native_layout<'a>(
    program: &'a Program,
    logical_path: &str,
    accepted_entry: Option<&AppEntry<'a>>,
) -> CanonicalNativeLayoutAnalysis<'a> {
    let [file] = program.files.as_slice() else {
        return native_layout_rejected(
            "path_module_app_identity_v0",
            None,
            Vec::new(),
            "native execution requires exactly one canonical program source file".to_string(),
        );
    };
    let occurrences = &file.module_occurrences;
    if occurrences.len() != 1 {
        let mut related = occurrences
            .iter()
            .map(|occurrence| {
                (
                    format!("module `{}`", occurrence.name),
                    occurrence.span.clone(),
                )
            })
            .collect::<Vec<_>>();
        let primary = related.first().map(|(_, span)| span.clone());
        if related.len() == 1 {
            related.clear();
        }
        return native_layout_rejected(
            "module_count_v0",
            primary,
            related,
            format!(
                "canonical native program requires exactly one module declaration; found {}",
                occurrences.len()
            ),
        );
    }
    let occurrence = occurrences.first();
    if occurrence.is_some_and(|occurrence| {
        file.items
            .iter()
            .any(|item| item.span().line < occurrence.span.line)
    }) {
        let occurrence = occurrence.expect("module occurrence checked above");
        return native_layout_rejected(
            "module_first_v0",
            Some(occurrence.span.clone()),
            Vec::new(),
            "canonical native program module must be the first semantic item".to_string(),
        );
    }

    let apps = file
        .items
        .iter()
        .filter_map(|item| match item {
            Item::App(app) => Some(app),
            Item::Type(_) | Item::Store(_) | Item::Task(_) | Item::Test(_) => None,
        })
        .collect::<Vec<_>>();
    let [app] = apps.as_slice() else {
        return native_layout_rejected(
            "missing_app_v0",
            occurrence.map(|occurrence| occurrence.span.clone()),
            Vec::new(),
            "canonical native program requires one final app".to_string(),
        );
    };

    let Some(expected) = canonical_program_identity(logical_path) else {
        return native_identity_rejected(file, occurrence, app, logical_path, None);
    };
    let retained = canonical_program_identity(&file.path);
    let expected_module = format!("programs.{}", expected.stem);
    if retained
        .as_ref()
        .map(|identity| identity.normalized.as_str())
        != Some(expected.normalized.as_str())
        || occurrence.is_some_and(|occurrence| {
            occurrence.name != expected_module
                || file.module.as_deref() != Some(occurrence.name.as_str())
        })
        || app.name != expected.stem
        || accepted_entry.is_none_or(|entry| !std::ptr::eq(*app, entry.app))
    {
        return native_identity_rejected(
            file,
            occurrence,
            app,
            logical_path,
            Some((&expected.normalized, &expected_module, &expected.stem)),
        );
    }

    let app_index = file
        .items
        .iter()
        .position(|item| matches!(item, Item::App(candidate) if std::ptr::eq(candidate, *app)))
        .expect("accepted app belongs to source file");
    for item in &file.items[..app_index] {
        match item {
            Item::Type(_) => {}
            Item::Store(_) | Item::Task(_) | Item::Test(_) => {
                return native_layout_rejected(
                    "illegal_pre_app_item_v0",
                    Some(item.span().clone()),
                    Vec::new(),
                    format!(
                        "canonical native program forbids top-level `{}` before its app",
                        item.kind()
                    ),
                );
            }
            Item::App(_) => unreachable!("sole app index is first app"),
        }
    }
    if file.items[app_index + 1..]
        .iter()
        .any(|item| matches!(item, Item::Type(_)))
    {
        let item = file.items[app_index + 1..]
            .iter()
            .find(|item| matches!(item, Item::Type(_)))
            .expect("late type");
        return native_layout_rejected(
            "type_after_app_v0",
            Some(item.span().clone()),
            Vec::new(),
            "canonical native program types must precede its app".to_string(),
        );
    }
    if let Some(item) = file.items[app_index + 1..]
        .iter()
        .find(|item| !matches!(item, Item::Type(_)))
    {
        return native_layout_rejected(
            "app_finality_v0",
            Some(item.span().clone()),
            Vec::new(),
            "canonical native program app must be the final top-level semantic item".to_string(),
        );
    }
    let Some(Item::Task(first_task)) = app.items.first() else {
        return native_layout_rejected(
            "first_entry_task_v0",
            Some(app.span.clone()),
            Vec::new(),
            "canonical native program entry must be its first direct-child task".to_string(),
        );
    };
    let Some(accepted_entry) = accepted_entry else {
        return native_layout_rejected(
            "first_entry_task_v0",
            Some(first_task.span.clone()),
            Vec::new(),
            "canonical native program entry was not authenticated by app analysis".to_string(),
        );
    };
    if !std::ptr::eq(first_task, accepted_entry.task)
        || app.items.iter().any(|item| match item {
            Item::Task(_) => false,
            Item::App(_) | Item::Type(_) | Item::Store(_) | Item::Test(_) => true,
        })
    {
        return native_layout_rejected(
            "first_entry_task_v0",
            Some(first_task.span.clone()),
            Vec::new(),
            "canonical native program entry must be first and all later app children must be helper tasks"
                .to_string(),
        );
    }

    CanonicalNativeLayoutAnalysis {
        layout: Some(CanonicalNativeLayout {
            file,
            app,
            entry: accepted_entry.task,
            normalized_path: expected.normalized,
        }),
        diagnostic: None,
        diagnostic_occurrence: None,
        reason: None,
    }
}

struct CanonicalProgramIdentity {
    normalized: String,
    stem: String,
}

fn canonical_program_identity(path: &str) -> Option<CanonicalProgramIdentity> {
    if path.is_empty()
        || path.starts_with(['/', '\\'])
        || path.as_bytes().get(1) == Some(&b':')
        || path.contains("//")
        || path.contains("\\\\")
        || path.contains("/\\")
        || path.contains("\\/")
    {
        return None;
    }
    let normalized = path.replace('\\', "/");
    let mut components = normalized.split('/');
    let (Some(directory), Some(file), None) =
        (components.next(), components.next(), components.next())
    else {
        return None;
    };
    if directory != "programs" || !file.ends_with(".hum") {
        return None;
    }
    let stem = file.strip_suffix(".hum")?.to_string();
    if !is_value_identifier(&stem) || stem.starts_with('_') {
        return None;
    }
    Some(CanonicalProgramIdentity { normalized, stem })
}

fn native_identity_rejected<'a>(
    file: &SourceFile,
    occurrence: Option<&crate::ast::ModuleOccurrence>,
    app: &App,
    logical_path: &str,
    expected: Option<(&str, &str, &str)>,
) -> CanonicalNativeLayoutAnalysis<'a> {
    let expected = expected
        .map(|(path, module, app)| {
            format!("expected path `{path}`, module `{module}`, app `{app}`; ")
        })
        .unwrap_or_default();
    native_layout_rejected(
        "path_module_app_identity_v0",
        Some(occurrence.map_or_else(|| app.span.clone(), |occurrence| occurrence.span.clone())),
        vec![(format!("observed app `{}`", app.name), app.span.clone())],
        format!(
            "{expected}observed logical path `{logical_path}`, retained path `{}`, module `{}`, app `{}`",
            file.path,
            occurrence.map_or("<missing>", |occurrence| occurrence.name.as_str()),
            app.name
        ),
    )
}

fn native_layout_rejected<'a>(
    reason: &'static str,
    span: Option<crate::diagnostic::Span>,
    related: Vec<(String, crate::diagnostic::Span)>,
    message: String,
) -> CanonicalNativeLayoutAnalysis<'a> {
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::CANONICAL_NATIVE_PROGRAM_LAYOUT,
        message,
        span,
    )
    .with_help(
        "Use one `programs/<name>.hum` source with `module programs.<name>` first, optional types, one final matching app, and its start task first.",
    );
    for (label, span) in related {
        diagnostic = diagnostic.with_related_span(label, span);
    }
    let (diagnostic, diagnostic_occurrence) = DiagnosticOccurrence::producer_diagnostic(
        crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(180),
        diagnostic,
        format!("canonical-native-layout:{reason}"),
        vec![format!("layout_reason={reason}")],
    )
    .expect("canonical native layout cause must be producer-owned");
    CanonicalNativeLayoutAnalysis {
        layout: None,
        diagnostic: Some(diagnostic),
        diagnostic_occurrence: Some(diagnostic_occurrence),
        reason: Some(reason),
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

    use super::{analyze, analyze_canonical_native_layout};

    fn program(source: &str) -> crate::ast::Program {
        let parsed = parser::parse_source("app.hum", source);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        crate::ast::Program {
            files: vec![parsed.file],
        }
    }

    fn native_program(path: &str, source: &str) -> crate::ast::Program {
        let parsed = parser::parse_source(path, source);
        assert!(
            parsed.diagnostics.is_empty(),
            "{path}: {:#?}",
            parsed.diagnostics
        );
        crate::ast::Program {
            files: vec![parsed.file],
        }
    }

    fn layout_reason_for(
        retained_path: &str,
        logical_path: &str,
        source: &str,
    ) -> Option<&'static str> {
        let program = native_program(retained_path, source);
        let entry_analysis = analyze(&program);
        let layout =
            analyze_canonical_native_layout(&program, logical_path, entry_analysis.entry.as_ref());
        if let Some(diagnostic) = layout.diagnostic.as_ref() {
            assert_eq!(diagnostic.code.as_str(), "H0634");
            assert_eq!(layout.diagnostic_occurrence.as_ref().map(|_| 1), Some(1));
        }
        layout.reason
    }

    fn layout_reason(path: &str, source: &str) -> Option<&'static str> {
        layout_reason_for(path, path, source)
    }

    #[test]
    fn canonical_native_program_layout_is_ordered_and_load_bearing() {
        const INTEGER_SIGN: &str = include_str!("../programs/integer_sign.hum");
        let parsed = parser::parse_source("programs/integer_sign.hum", INTEGER_SIGN);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        assert_eq!(parsed.file.module.as_deref(), Some("programs.integer_sign"));
        assert_eq!(parsed.file.module_occurrences.len(), 1);
        assert_eq!(
            parsed.file.module_occurrences[0].name,
            "programs.integer_sign"
        );
        assert_eq!(parsed.file.module_occurrences[0].span.line, 1);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let entry_analysis = analyze(&program);
        assert!(entry_analysis.diagnostic.is_none());
        let entry = entry_analysis.entry.as_ref().expect("integer_sign entry");
        let accepted =
            analyze_canonical_native_layout(&program, "programs/integer_sign.hum", Some(entry));
        let layout = accepted.layout.expect("canonical integer_sign layout");
        assert_eq!(layout.normalized_path, "programs/integer_sign.hum");
        assert_eq!(layout.file.path, "programs/integer_sign.hum");
        assert_eq!(layout.app.name, "integer_sign");
        assert_eq!(layout.entry.name, "run_tool");

        let valid = include_str!("../fixtures/programs/integer_sign/layout_valid_pass.hum");
        assert_eq!(layout_reason("programs/layout_valid_pass.hum", valid), None);
        let missing_module = layout_reason(
            "programs/missing_module_fail.hum",
            include_str!("../fixtures/programs/integer_sign/missing_module_fail.hum"),
        );
        let duplicate_module = native_program(
            "programs/duplicate_module_fail.hum",
            include_str!("../fixtures/programs/integer_sign/duplicate_module_fail.hum"),
        );
        let duplicate_entry = analyze(&duplicate_module).entry.expect("entry");
        let duplicate = analyze_canonical_native_layout(
            &duplicate_module,
            "programs/duplicate_module_fail.hum",
            Some(&duplicate_entry),
        );
        assert_eq!(
            (
                missing_module,
                duplicate.reason,
                duplicate
                    .diagnostic
                    .as_ref()
                    .map(|diagnostic| diagnostic.related_spans.len()),
            ),
            (Some("module_count_v0"), Some("module_count_v0"), Some(2),),
            "M07 missing and duplicate module cases must both retain H0634 without panic"
        );
        assert_eq!(
            layout_reason(
                "programs/late_module_fail.hum",
                include_str!("../fixtures/programs/integer_sign/late_module_fail.hum"),
            ),
            Some("module_first_v0"),
            "M08 late module must retain its H0634 reason"
        );
        assert_eq!(
            layout_reason(
                "programs/missing_app_fail.hum",
                include_str!("../fixtures/programs/integer_sign/missing_app_fail.hum"),
            ),
            Some("missing_app_v0")
        );
        let illegal_pre_app_outcomes = [
            layout_reason(
                "programs/illegal_pre_app_store_fail.hum",
                include_str!("../fixtures/programs/integer_sign/illegal_pre_app_store_fail.hum"),
            ),
            layout_reason(
                "programs/illegal_pre_app_task_fail.hum",
                include_str!("../fixtures/programs/integer_sign/illegal_pre_app_task_fail.hum"),
            ),
            layout_reason(
                "programs/illegal_pre_app_test_fail.hum",
                include_str!("../fixtures/programs/integer_sign/illegal_pre_app_test_fail.hum"),
            ),
        ];
        assert_eq!(
            illegal_pre_app_outcomes,
            [Some("illegal_pre_app_item_v0"); 3],
            "M13 Store, Task, and Test must each retain illegal-item H0634"
        );
        assert_eq!(
            layout_reason(
                "programs/type_after_app_fail.hum",
                include_str!("../fixtures/programs/integer_sign/type_after_app_fail.hum"),
            ),
            Some("type_after_app_v0"),
            "M09 type-after-app must retain its H0634 reason"
        );
        assert_eq!(
            layout_reason(
                "programs/semantic_after_app_fail.hum",
                include_str!("../fixtures/programs/integer_sign/semantic_after_app_fail.hum"),
            ),
            Some("app_finality_v0"),
            "M10 semantic-item-after-app must retain its H0634 reason"
        );
        let entry_order_outcomes = [
            layout_reason(
                "programs/start_not_first_fail.hum",
                include_str!("../fixtures/programs/integer_sign/start_not_first_fail.hum"),
            ),
            layout_reason(
                "programs/helper_before_start_fail.hum",
                include_str!("../fixtures/programs/integer_sign/helper_before_start_fail.hum"),
            ),
        ];
        assert_eq!(
            entry_order_outcomes,
            [Some("first_entry_task_v0"); 2],
            "M11 start_not_first and helper_before_start must both retain entry-order H0634"
        );

        let app_name_mismatch = INTEGER_SIGN.replacen("app integer_sign {", "app foreign_app {", 1);
        let identity_outcomes = [
            layout_reason_for(
                "programs/integer_sign.hum",
                "programs/renamed_program.hum",
                INTEGER_SIGN,
            ),
            layout_reason(
                "programs/integer_sign.hum",
                include_str!("../fixtures/programs/integer_sign/module_path_identity_fail.hum"),
            ),
            layout_reason("programs/integer_sign.hum", &app_name_mismatch),
        ];
        assert_eq!(
            identity_outcomes,
            [Some("path_module_app_identity_v0"); 3],
            "M12 filename stem, module suffix, and app name must each retain identity H0634"
        );
        let duplicate_app = native_program(
            "programs/duplicate_app_fail.hum",
            include_str!("../fixtures/programs/integer_sign/duplicate_app_fail.hum"),
        );
        let duplicate_app_diagnostic = analyze(&duplicate_app).diagnostic.expect("H0615");
        assert_eq!(duplicate_app_diagnostic.code.as_str(), "H0615");

        assert_eq!(
            layout_reason("programs\\integer_sign.hum", INTEGER_SIGN),
            None,
            "normalized backslash spelling must remain accepted"
        );
        for (label, invalid) in [
            ("directory case", "Programs/integer_sign.hum"),
            ("stem case", "programs/Integer_sign.hum"),
            ("extension case", "programs/integer_sign.HUM"),
            ("foreign directory", "other/integer_sign.hum"),
            ("cache lookalike", "cargo-home/integer_sign.hum"),
            ("missing extension", "programs/integer_sign"),
            ("slash-root absolute", "/programs/integer_sign.hum"),
            (
                "drive-root absolute",
                concat!("C:", r"\programs\integer_sign.hum"),
            ),
            (
                "UNC absolute",
                concat!(r"\", r"\server\programs\integer_sign.hum"),
            ),
            ("dot component", "programs/./integer_sign.hum"),
            ("parent component", "programs/../integer_sign.hum"),
            ("duplicate slash", "programs//integer_sign.hum"),
            ("duplicate backslash", r"programs\\integer_sign.hum"),
            ("slash-backslash", r"programs/\integer_sign.hum"),
            ("backslash-slash", r"programs\/integer_sign.hum"),
        ] {
            assert_eq!(
                layout_reason(invalid, INTEGER_SIGN),
                Some("path_module_app_identity_v0"),
                "rejected path row {label}: {invalid}"
            );
        }

        let unsupported =
            include_str!("../fixtures/programs/integer_sign/unsupported_shape_fail.hum");
        assert_eq!(
            layout_reason("programs/integer_sign.hum", unsupported),
            None
        );
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
