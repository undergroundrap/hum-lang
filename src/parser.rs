use crate::ast::{
    App, Field, Item, Param, Section, SectionLine, SourceFile, Store, Task, Test, TypeDef,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Span};

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
            let line = &self.lines[index];
            let trimmed = line.text.trim();

            if is_ignorable(trimmed) {
                index += 1;
                continue;
            }

            if count_indent(&line.text) == 0 {
                if let Some(rest) = trimmed.strip_prefix("module ") {
                    module = Some(rest.trim().to_string());
                    index += 1;
                    continue;
                }

                if is_item_header(trimmed) {
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

            if count_indent(&line_text) == item_indent && is_item_header(trimmed) {
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
            let nested_items = self.parse_items_in_range(body_start, body_end, item_indent + 2);
            Item::App(App {
                name,
                sections,
                items: nested_items,
                span,
            })
        } else if header.starts_with("type ") {
            let name = header
                .trim_start_matches("type ")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            let fields = self.parse_fields(body_start, body_end, item_indent + 2);
            Item::Type(TypeDef {
                name,
                fields,
                sections,
                span,
            })
        } else if header.starts_with("store ") {
            let (name, ty) = parse_store_header(header);
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
            let line = &self.lines[index];
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
                            || is_item_header(candidate_trimmed))
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

    fn parse_fields(&self, start: usize, end: usize, field_indent: usize) -> Vec<Field> {
        let mut fields = Vec::new();
        for index in start..end {
            let line = &self.lines[index];
            let trimmed = line.text.trim();
            if is_ignorable(trimmed) || count_indent(&line.text) != field_indent {
                continue;
            }
            if is_section_header(trimmed) || is_item_header(trimmed) {
                continue;
            }
            if let Some((name, ty)) = trimmed.split_once(':') {
                fields.push(Field {
                    name: name.trim().to_string(),
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
                if is_test_modifier(last) {
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
                params.push(Param {
                    name: name.trim().to_string(),
                    ty: ty.trim().to_string(),
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

    fn span(&self, line: usize) -> Span {
        Span::new(self.path.clone(), line, 1)
    }
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

fn is_item_header(trimmed: &str) -> bool {
    trimmed.ends_with('{')
        && (trimmed.starts_with("app ")
            || trimmed.starts_with("type ")
            || trimmed.starts_with("store ")
            || trimmed.starts_with("task ")
            || trimmed.starts_with("test "))
}

fn is_section_header(trimmed: &str) -> bool {
    if !trimmed.ends_with(':') || trimmed.len() <= 1 {
        return false;
    }
    let name = trimmed.trim_end_matches(':').trim();
    !name.is_empty() && name.chars().all(|ch| ch.is_ascii_alphabetic() || ch == ' ')
}

fn is_test_modifier(word: &str) -> bool {
    matches!(
        word,
        "property" | "fuzz" | "regression" | "integration" | "model" | "unit"
    )
}

#[cfg(test)]
mod tests {
    use super::parse_source;
    use crate::ast::Item;
    use crate::diagnostic::DiagnosticCode;

    #[test]
    fn parses_task_with_sections() {
        let source = r#"module demo

task add task(title: Text) -> Result Task, TaskError {
  why:
    remember a task

  changes:
    tasks

  does:
    save task in tasks
}
"#;
        let parsed = parse_source("demo.hum", source);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(parsed.file.module.as_deref(), Some("demo"));
        match &parsed.file.items[0] {
            Item::Task(task) => {
                assert_eq!(task.name, "add task");
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
    fn reports_stable_code_for_untyped_parameter() {
        let source = r#"task save task(title) {
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
}
