use crate::ast::{
    App, Field, Item, Param, ParamPermission, Section, SectionLine, SourceFile, Store, Task, Test,
    TypeDef,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Span};
use crate::syntax;

#[derive(Debug, Clone)]
pub struct ParseOutput {
    pub file: SourceFile,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
struct SourceLine {
    number: usize,
    text: String,
}

#[derive(Debug, Clone, Copy)]
enum IdentifierKind {
    Value,
    Type,
}

struct Parser {
    path: String,
    lines: Vec<SourceLine>,
    diagnostics: Vec<Diagnostic>,
}

pub fn parse_source(path: impl Into<String>, source: &str) -> ParseOutput {
    let path = path.into();
    let lines = source
        .lines()
        .enumerate()
        .map(|(index, text)| SourceLine {
            number: index + 1,
            text: text.to_string(),
        })
        .collect();

    let mut parser = Parser {
        path: path.clone(),
        lines,
        diagnostics: Vec::new(),
    };
    let (module, items) = parser.parse_file_items();

    ParseOutput {
        file: SourceFile {
            path,
            module,
            items,
        },
        diagnostics: parser.diagnostics,
    }
}

impl Parser {
    fn parse_file_items(&mut self) -> (Option<String>, Vec<Item>) {
        let mut module = None;
        let mut items = Vec::new();
        let mut index = 0;

        while index < self.lines.len() {
            let line = self.lines[index].clone();
            let trimmed = line.text.trim();

            if is_ignorable(trimmed) {
                index += 1;
                continue;
            }

            if count_indent(&line.text) == 0 {
                if let Some(rest) = trimmed.strip_prefix("module ") {
                    let module_name = rest.trim().to_string();
                    self.validate_module_path(&module_name, line.number);
                    module = Some(module_name);
                    index += 1;
                    continue;
                }

                if syntax::is_item_start(trimmed) {
                    match self.parse_item_at(index) {
                        Some((item, next_index)) => {
                            items.push(item);
                            index = next_index;
                        }
                        None => index += 1,
                    }
                    continue;
                }
            }

            self.diagnostics.push(
                Diagnostic::warning(
                    DiagnosticCode::UNEXPECTED_TOP_LEVEL_LINE,
                    "unexpected top-level line",
                    Some(self.span(line.number)),
                )
                .with_help(
                    "Hum milestone 0 accepts module, app, type, store, task, and test at the top level.",
                ),
            );
            index += 1;
        }

        (module, items)
    }

    fn parse_items_in_range(&mut self, start: usize, end: usize, item_indent: usize) -> Vec<Item> {
        let mut items = Vec::new();
        let mut index = start;
        while index < end {
            let line_number = self.lines[index].number;
            let line_text = self.lines[index].text.clone();
            let trimmed = line_text.trim();
            if is_ignorable(trimmed) {
                index += 1;
                continue;
            }

            if count_indent(&line_text) == item_indent && syntax::is_item_start(trimmed) {
                match self.parse_item_at(index) {
                    Some((item, next_index)) if next_index <= end + 1 => {
                        items.push(item);
                        index = next_index;
                    }
                    Some((_item, next_index)) => {
                        self.diagnostics.push(Diagnostic::error(
                            DiagnosticCode::NESTED_ITEM_EXTENDS_PAST_BLOCK,
                            "nested item extends past containing block",
                            Some(self.span(line_number)),
                        ));
                        index = next_index;
                    }
                    None => index += 1,
                }
            } else {
                index += 1;
            }
        }
        items
    }

    fn parse_item_at(&mut self, index: usize) -> Option<(Item, usize)> {
        let line = self.lines.get(index)?.clone();
        let trimmed = line.text.trim();
        let header = match trimmed.strip_suffix('{') {
            Some(header) => header.trim(),
            None => {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::ITEM_HEADER_MISSING_OPEN_BRACE,
                        "item header must end with `{`",
                        Some(self.span(line.number)),
                    )
                    .with_help(
                        "Write items as `task name(...) { ... }`, `type Name { ... }`, and so on.",
                    ),
                );
                return None;
            }
        };

        let close_index = match self.find_matching_close(index) {
            Some(close_index) => close_index,
            None => {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ITEM_BLOCK_MISSING_CLOSE_BRACE,
                    "item block is missing a closing `}`",
                    Some(self.span(line.number)),
                ));
                return None;
            }
        };

        let item_indent = count_indent(&line.text);
        let body_start = index + 1;
        let body_end = close_index;
        let sections = self.parse_sections(body_start, body_end, item_indent + 2);
        let span = self.span(line.number);

        let item = if header.starts_with("app ") {
            let name = header.trim_start_matches("app ").trim().to_string();
            self.validate_identifier("app name", &name, IdentifierKind::Value, line.number);
            let nested_items = self.parse_items_in_range(body_start, body_end, item_indent + 2);
            Item::App(App {
                name,
                sections,
                items: nested_items,
                span,
            })
        } else if header.starts_with("type ") {
            let name = header.trim_start_matches("type ").trim().to_string();
            self.validate_identifier("type name", &name, IdentifierKind::Type, line.number);
            let fields = self.parse_fields(body_start, body_end, item_indent + 2);
            Item::Type(TypeDef {
                name,
                fields,
                sections,
                span,
            })
        } else if header.starts_with("store ") {
            let (name, ty) = parse_store_header(header);
            self.validate_identifier("store name", &name, IdentifierKind::Value, line.number);
            Item::Store(Store {
                name,
                ty,
                sections,
                span,
            })
        } else if header.starts_with("task ") {
            let (name, params, result) = self.parse_task_header(header, line.number);
            Item::Task(Task {
                name,
                params,
                result,
                sections,
                span,
            })
        } else if header.starts_with("test ") {
            let (name, params, modifiers) = self.parse_test_header(header, line.number);
            Item::Test(Test {
                name,
                params,
                modifiers,
                sections,
                span,
            })
        } else {
            self.diagnostics.push(Diagnostic::warning(
                DiagnosticCode::UNKNOWN_ITEM_KIND,
                "unknown item kind",
                Some(self.span(line.number)),
            ));
            return None;
        };

        Some((item, close_index + 1))
    }

    fn find_matching_close(&self, start: usize) -> Option<usize> {
        let mut depth = 0usize;
        for index in start..self.lines.len() {
            let text = &self.lines[index].text;
            for ch in text.chars() {
                match ch {
                    '{' => depth = depth.saturating_add(1),
                    '}' => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            return Some(index);
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn parse_sections(&self, start: usize, end: usize, section_indent: usize) -> Vec<Section> {
        let mut sections = Vec::new();
        let mut index = start;

        while index < end {
            let line = self.lines[index].clone();
            let trimmed = line.text.trim();
            if count_indent(&line.text) == section_indent && is_section_header(trimmed) {
                let name = trimmed.trim_end_matches(':').trim().to_string();
                let mut lines = Vec::new();
                let mut cursor = index + 1;

                while cursor < end {
                    let candidate = &self.lines[cursor];
                    let candidate_trimmed = candidate.text.trim();
                    let candidate_indent = count_indent(&candidate.text);
                    if candidate_indent == section_indent
                        && (is_section_header(candidate_trimmed)
                            || syntax::is_item_start(candidate_trimmed))
                    {
                        break;
                    }
                    lines.push(SectionLine {
                        text: candidate.text.trim().to_string(),
                        span: self.span(candidate.number),
                    });
                    cursor += 1;
                }

                sections.push(Section {
                    name,
                    lines,
                    span: self.span(line.number),
                });
                index = cursor;
            } else {
                index += 1;
            }
        }

        sections
    }

    fn parse_fields(&mut self, start: usize, end: usize, field_indent: usize) -> Vec<Field> {
        let mut fields = Vec::new();
        for index in start..end {
            let line = self.lines[index].clone();
            let trimmed = line.text.trim();
            if is_ignorable(trimmed) || count_indent(&line.text) != field_indent {
                continue;
            }
            if is_section_header(trimmed) || syntax::is_item_start(trimmed) {
                continue;
            }
            if let Some((name, ty)) = trimmed.split_once(':') {
                let name = name.trim().to_string();
                self.validate_identifier("field name", &name, IdentifierKind::Value, line.number);
                fields.push(Field {
                    name,
                    ty: ty.trim().to_string(),
                    span: self.span(line.number),
                });
            }
        }
        fields
    }

    fn parse_task_header(
        &mut self,
        header: &str,
        line_number: usize,
    ) -> (String, Vec<Param>, Option<String>) {
        let rest = header.trim_start_matches("task ").trim();
        let (signature, result) = match rest.split_once("->") {
            Some((left, right)) => (left.trim(), Some(right.trim().to_string())),
            None => (rest, None),
        };

        let (name, params, trailing) = self.parse_callable_signature(signature, line_number);
        self.validate_identifier("task name", &name, IdentifierKind::Value, line_number);
        if !trailing.trim().is_empty() {
            self.diagnostics.push(Diagnostic::warning(
                DiagnosticCode::UNEXPECTED_SIGNATURE_TEXT,
                "unexpected text after task parameter list",
                Some(self.span(line_number)),
            ));
        }
        (name, params, result)
    }

    fn parse_test_header(
        &mut self,
        header: &str,
        line_number: usize,
    ) -> (String, Vec<Param>, Vec<String>) {
        let rest = header.trim_start_matches("test ").trim();
        if rest.contains('(') {
            let (name, params, trailing) = self.parse_callable_signature(rest, line_number);
            let modifiers = trailing
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>();
            (name, params, modifiers)
        } else {
            let mut words = rest.split_whitespace().collect::<Vec<_>>();
            let mut modifiers = Vec::new();
            while let Some(last) = words.last().copied() {
                if syntax::is_test_modifier(last) {
                    modifiers.insert(0, last.to_string());
                    words.pop();
                } else {
                    break;
                }
            }
            (words.join(" "), Vec::new(), modifiers)
        }
    }

    fn parse_callable_signature(
        &mut self,
        signature: &str,
        line_number: usize,
    ) -> (String, Vec<Param>, String) {
        let Some(open) = signature.find('(') else {
            return (signature.trim().to_string(), Vec::new(), String::new());
        };
        let Some(close) = signature[open + 1..]
            .find(')')
            .map(|offset| open + 1 + offset)
        else {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN,
                "callable signature is missing `)`",
                Some(self.span(line_number)),
            ));
            return (
                signature[..open].trim().to_string(),
                Vec::new(),
                String::new(),
            );
        };

        let name = signature[..open].trim().to_string();
        let params_text = &signature[open + 1..close];
        let trailing = signature[close + 1..].trim().to_string();
        let params = self.parse_params(params_text, line_number);
        (name, params, trailing)
    }

    fn parse_params(&mut self, params_text: &str, line_number: usize) -> Vec<Param> {
        let mut params = Vec::new();
        if params_text.trim().is_empty() {
            return params;
        }

        for raw_param in params_text.split(',') {
            let param = raw_param.trim();
            if let Some((name, ty)) = param.split_once(':') {
                let (permission, name) = parse_param_permission(name.trim());
                let name = name.to_string();
                self.validate_identifier(
                    "parameter name",
                    &name,
                    IdentifierKind::Value,
                    line_number,
                );
                params.push(Param {
                    name,
                    ty: ty.trim().to_string(),
                    permission,
                    span: self.span(line_number),
                });
            } else {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::PARAMETER_MISSING_TYPE,
                    format!("parameter `{param}` is missing a type"),
                    Some(self.span(line_number)),
                ));
            }
        }
        params
    }

    fn validate_module_path(&mut self, module_name: &str, line_number: usize) {
        if module_name.is_empty() {
            self.invalid_identifier(
                "module path",
                module_name,
                IdentifierKind::Value,
                line_number,
            );
            return;
        }

        for segment in module_name.split('.') {
            if !is_valid_identifier(segment, IdentifierKind::Value) {
                self.invalid_identifier(
                    "module segment",
                    segment,
                    IdentifierKind::Value,
                    line_number,
                );
            }
        }
    }

    fn validate_identifier(
        &mut self,
        label: &str,
        name: &str,
        kind: IdentifierKind,
        line_number: usize,
    ) {
        if !is_valid_identifier(name, kind) {
            self.invalid_identifier(label, name, kind, line_number);
        }
    }

    fn invalid_identifier(
        &mut self,
        label: &str,
        name: &str,
        kind: IdentifierKind,
        line_number: usize,
    ) {
        let suggestion = identifier_suggestion(name, kind);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::INVALID_IDENTIFIER,
                format!("{label} `{name}` is not a valid Hum identifier"),
                Some(self.span(line_number)),
            )
            .with_help(format!(
                "Use `{suggestion}`. Value names use snake_case, type names use PascalCase, and sentences belong in `why:`."
            )),
        );
    }

    fn span(&self, line_number: usize) -> Span {
        let column = self
            .lines
            .iter()
            .find(|line| line.number == line_number)
            .map_or(1, |line| first_visible_column(&line.text));
        Span::new(self.path.clone(), line_number, column)
    }
}

fn parse_param_permission(raw_name: &str) -> (ParamPermission, &str) {
    let raw_name = raw_name.trim();
    let Some(first) = raw_name.split_whitespace().next() else {
        return (ParamPermission::Borrow, raw_name);
    };
    let permission = match first {
        "borrow" => ParamPermission::Borrow,
        "change" => ParamPermission::Change,
        "consume" => ParamPermission::Consume,
        _ => return (ParamPermission::Borrow, raw_name),
    };
    let name = raw_name[first.len()..].trim();
    (permission, name)
}

fn is_valid_identifier(name: &str, kind: IdentifierKind) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    match kind {
        IdentifierKind::Value => {
            (first.is_ascii_lowercase() || first == '_')
                && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        }
        IdentifierKind::Type => {
            first.is_ascii_uppercase() && chars.all(|ch| ch.is_ascii_alphanumeric())
        }
    }
}

fn identifier_suggestion(name: &str, kind: IdentifierKind) -> String {
    match kind {
        IdentifierKind::Value => snake_identifier(name),
        IdentifierKind::Type => pascal_identifier(name),
    }
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
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        "name".to_string()
    } else if out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        format!("_{out}")
    } else {
        out
    }
}

fn pascal_identifier(text: &str) -> String {
    let words = text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    if words.is_empty() {
        return "Name".to_string();
    }

    let mut out = String::new();
    for word in words {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            for ch in chars {
                out.push(ch);
            }
        }
    }
    out
}

fn first_visible_column(text: &str) -> usize {
    text.chars()
        .position(|ch| !ch.is_whitespace())
        .map_or(1, |index| index + 1)
}

fn parse_store_header(header: &str) -> (String, String) {
    let rest = header.trim_start_matches("store ").trim();
    match rest.split_once(':') {
        Some((name, ty)) => (name.trim().to_string(), ty.trim().to_string()),
        None => (rest.to_string(), String::new()),
    }
}

fn count_indent(text: &str) -> usize {
    text.chars().take_while(|ch| *ch == ' ').count()
}

fn is_ignorable(trimmed: &str) -> bool {
    trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//")
}

fn is_section_header(trimmed: &str) -> bool {
    if !trimmed.ends_with(':') || trimmed.len() <= 1 {
        return false;
    }
    let name = trimmed.trim_end_matches(':').trim();
    !name.is_empty() && name.chars().all(|ch| ch.is_ascii_alphabetic() || ch == ' ')
}

#[cfg(test)]
mod tests {
    use super::parse_source;
    use crate::ast::Item;
    use crate::diagnostic::{DiagnosticCode, Severity};

    #[test]
    fn parses_task_with_sections() {
        let source = r#"module demo

task add_task(title: Text) -> Result Task, TaskError {
  why:
    remember a task

  changes:
    tasks

  does:
    save item in tasks
}
"#;
        let parsed = parse_source("demo.hum", source);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(parsed.file.module.as_deref(), Some("demo"));
        match &parsed.file.items[0] {
            Item::Task(task) => {
                assert_eq!(task.name, "add_task");
                assert_eq!(task.sections.len(), 3);
            }
            other => panic!("expected task, got {other:?}"),
        }
    }

    #[test]
    fn peels_test_modifier_from_name() {
        let source = r#"test blank title regression {
  why:
    keep a bug fixed
  does:
    expect fixed
}
"#;
        let parsed = parse_source("demo.hum", source);
        match &parsed.file.items[0] {
            Item::Test(test) => {
                assert_eq!(test.name, "blank title");
                assert_eq!(test.modifiers, vec!["regression"]);
            }
            other => panic!("expected test, got {other:?}"),
        }
    }

    #[test]
    fn parses_parameter_permission_modes() {
        let source = r#"task permissions(item: WorkItem, borrow view: WorkItem, change draft: WorkItem, consume owned: WorkItem) {
  does:
    return owned
}
"#;
        let parsed = parse_source("permissions.hum", source);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        match &parsed.file.items[0] {
            Item::Task(task) => {
                assert_eq!(
                    task.params[0].permission,
                    crate::ast::ParamPermission::Borrow
                );
                assert_eq!(
                    task.params[1].permission,
                    crate::ast::ParamPermission::Borrow
                );
                assert_eq!(
                    task.params[2].permission,
                    crate::ast::ParamPermission::Change
                );
                assert_eq!(
                    task.params[3].permission,
                    crate::ast::ParamPermission::Consume
                );
                assert_eq!(task.params[3].name, "owned");
            }
            other => panic!("expected task, got {other:?}"),
        }
    }

    #[test]
    fn reports_stable_code_for_untyped_parameter() {
        let source = r#"task save_task(title) {
  why:
    save it
  does:
    return title
}
"#;
        let parsed = parse_source("bad.hum", source);
        assert!(
            parsed
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == DiagnosticCode::PARAMETER_MISSING_TYPE)
        );
    }

    #[test]
    fn rejects_spaced_task_name_with_snake_case_help() {
        let source = r#"task save task(title: Text) {
  does:
    return title
}
"#;
        let parsed = parse_source("spaced-name.hum", source);
        assert!(parsed.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::INVALID_IDENTIFIER
                && diagnostic.severity == Severity::Error
                && diagnostic
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains("save_task"))
        }));
    }

    #[test]
    fn reports_stable_codes_for_malformed_sources() {
        let cases = [
            (
                "missing-open-brace.hum",
                "task save_task()\n  why:\n    save it\n",
                DiagnosticCode::ITEM_HEADER_MISSING_OPEN_BRACE,
                Severity::Error,
            ),
            (
                "missing-close-brace.hum",
                "task save_task() {\n  why:\n    save it\n",
                DiagnosticCode::ITEM_BLOCK_MISSING_CLOSE_BRACE,
                Severity::Error,
            ),
            (
                "missing-close-paren.hum",
                "task save_task(title: Text {\n  why:\n    save it\n}\n",
                DiagnosticCode::CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN,
                Severity::Error,
            ),
            (
                "unexpected-top-level.hum",
                "does:\n  orphan body line\n",
                DiagnosticCode::UNEXPECTED_TOP_LEVEL_LINE,
                Severity::Warning,
            ),
        ];

        for (path, source, expected_code, expected_severity) in cases {
            let parsed = parse_source(path, source);
            assert!(
                parsed.diagnostics.iter().any(|diagnostic| {
                    diagnostic.code == expected_code
                        && diagnostic.severity == expected_severity
                        && diagnostic
                            .span
                            .as_ref()
                            .is_some_and(|span| span.file == path)
                }),
                "expected {expected_code:?} in {path}, got {:?}",
                parsed.diagnostics
            );
        }
    }

    #[test]
    fn reports_missing_brace_for_nested_item_headers() {
        let source = r#"app demo {
  why:
    show nested parsing

  task nested_task()
    why:
      missing brace
}
"#;
        let parsed = parse_source("nested-bad.hum", source);
        assert!(parsed.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::ITEM_HEADER_MISSING_OPEN_BRACE
                && diagnostic.span.as_ref().is_some_and(|span| span.line == 5)
        }));
    }

    #[test]
    fn parses_nested_app_items_from_contract() {
        let source = r#"app demo {
  why:
    group the demo

  task add_task(title: Text) {
    why:
      add the task

    does:
      return title
  }
}
"#;
        let parsed = parse_source("nested.hum", source);
        assert!(parsed.diagnostics.is_empty());
        match &parsed.file.items[0] {
            Item::App(app) => {
                assert_eq!(app.items.len(), 1);
                assert!(matches!(&app.items[0], Item::Task(task) if task.name == "add_task"));
            }
            other => panic!("expected app, got {other:?}"),
        }
    }

    #[test]
    fn preserves_comment_lines_inside_sections() {
        let source = r#"task explain() {
  why:
    # keep this visible to graph consumers
    explain the thing

  does:
    return
}
"#;
        let parsed = parse_source("comments.hum", source);
        let task = match &parsed.file.items[0] {
            Item::Task(task) => task,
            other => panic!("expected task, got {other:?}"),
        };
        let why = task.section("why").expect("why section");
        assert_eq!(why.lines[0].text, "# keep this visible to graph consumers");
        assert_eq!(why.lines[1].text, "explain the thing");
    }
}
