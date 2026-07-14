use crate::ast::{
    App, CallableTypeSyntax, Field, Item, Param, ParamPermission, ParsedBodyStatement,
    ParsedBodyStatementKind, ParsedCall, ParsedCallCloseStatus, ParsedCallTrailingStatus,
    ParsedEffectDeclaration, ParsedEffectDeclarationKind, ParsedExpression, ParsedExpressionKind,
    ParsedIdentifier, Section, SectionLine, SourceFile, Store, Task, Test, TypeDef, TypeSyntax,
    TypeSyntaxKind,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOccurrence, Span};
use crate::syntax;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ParsedCallPosition {
    statement_index: usize,
    path: Vec<usize>,
}

impl ParsedCallPosition {
    pub(crate) fn statement_index(&self) -> usize {
        self.statement_index
    }

    pub(crate) fn stable_component(&self) -> String {
        format!(
            "statement-{}:path-{}",
            self.statement_index,
            self.path
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedCallIdentifierUse {
    pub(crate) name: String,
    pub(crate) ordinal: usize,
    pub(crate) consumed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedExecutableCallNode {
    pub(crate) position: ParsedCallPosition,
    pub(crate) callee: String,
    pub(crate) source: String,
    pub(crate) span: Span,
    pub(crate) identifier_uses: Vec<ParsedCallIdentifierUse>,
}

#[derive(Debug, Clone)]
pub struct ParseOutput {
    pub file: SourceFile,
    pub diagnostics: Vec<Diagnostic>,
    pub(crate) diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet,
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
    semantic_file_index: usize,
    current_semantic_node: Option<String>,
    lines: Vec<SourceLine>,
    diagnostics: Vec<Diagnostic>,
    diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet,
}

#[cfg(test)]
pub fn parse_source(path: impl Into<String>, source: &str) -> ParseOutput {
    parse_source_at_index(path, source, 0)
}

pub(crate) fn parse_source_at_index(
    path: impl Into<String>,
    source: &str,
    semantic_file_index: usize,
) -> ParseOutput {
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
        semantic_file_index,
        current_semantic_node: None,
        lines,
        diagnostics: Vec::new(),
        diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet::default(),
    };
    let (module, items) = parser.parse_file_items();
    parser
        .diagnostic_occurrences
        .validate()
        .expect("parser diagnostics must use registered parser causes");

    ParseOutput {
        file: SourceFile {
            path,
            module,
            items,
        },
        diagnostics: parser.diagnostics,
        diagnostic_occurrences: parser.diagnostic_occurrences,
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
                    let item_path = vec![items.len()];
                    match self.parse_item_at_semantic_node(index, &item_path) {
                        Some((item, next_index)) => {
                            items.push(item);
                            index = next_index;
                        }
                        None => index += 1,
                    }
                    continue;
                }
            }

            self.emit(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(48),
                "top_level_line",
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

    fn parse_items_in_range(
        &mut self,
        start: usize,
        end: usize,
        item_indent: usize,
        parent_path: &[usize],
    ) -> Vec<Item> {
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
                let mut item_path = parent_path.to_vec();
                item_path.push(items.len());
                match self.parse_item_at_semantic_node(index, &item_path) {
                    Some((item, next_index)) if next_index <= end + 1 => {
                        items.push(item);
                        index = next_index;
                    }
                    Some((_item, next_index)) => {
                        self.emit(
                            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(49),
                            "nested_item",
                            Diagnostic::error(
                                DiagnosticCode::NESTED_ITEM_EXTENDS_PAST_BLOCK,
                                "nested item extends past containing block",
                                Some(self.span(line_number)),
                            ),
                        );
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

    fn parse_item_at_semantic_node(
        &mut self,
        index: usize,
        item_path: &[usize],
    ) -> Option<(Item, usize)> {
        let semantic_node = format!(
            "resolver-item:file-{}:path-{}",
            self.semantic_file_index,
            item_path
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(".")
        );
        let prior = self.current_semantic_node.replace(semantic_node);
        let parsed = self.parse_item_at(index, item_path);
        self.current_semantic_node = prior;
        parsed
    }

    fn parse_item_at(&mut self, index: usize, item_path: &[usize]) -> Option<(Item, usize)> {
        let line = self.lines.get(index)?.clone();
        let trimmed = line.text.trim();
        let header = match trimmed.strip_suffix('{') {
            Some(header) => header.trim(),
            None => {
                self.emit(
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(50),
                    "item_header",
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
                self.emit(
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(51),
                    "item_block",
                    Diagnostic::error(
                        DiagnosticCode::ITEM_BLOCK_MISSING_CLOSE_BRACE,
                        "item block is missing a closing `}`",
                        Some(self.span(line.number)),
                    ),
                );
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
            let nested_items =
                self.parse_items_in_range(body_start, body_end, item_indent + 2, item_path);
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
            let (name, params, result, result_syntax) = self.parse_task_header(header, line.number);
            let effect_syntax = parse_task_effect_syntax(&sections);
            let body_syntax = parse_task_body_syntax(&sections);
            Item::Task(Task {
                name,
                params,
                result,
                result_syntax,
                sections,
                effect_syntax,
                body_syntax,
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
            self.emit(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(52),
                "item_kind",
                Diagnostic::warning(
                    DiagnosticCode::UNKNOWN_ITEM_KIND,
                    "unknown item kind",
                    Some(self.span(line.number)),
                ),
            );
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
    ) -> (String, Vec<Param>, Option<String>, Option<TypeSyntax>) {
        let rest = header.trim_start_matches("task ").trim();
        let (signature, result, result_offset) = match find_top_level_arrow(rest) {
            Some(index) => (
                rest[..index].trim(),
                Some(rest[index + 2..].trim().to_string()),
                Some(index + 2 + rest[index + 2..].len() - rest[index + 2..].trim_start().len()),
            ),
            None => (rest, None, None),
        };

        let signature_column = self.span(line_number).column + "task ".len();
        let (name, params, trailing) =
            self.parse_callable_signature(signature, line_number, signature_column);
        self.validate_identifier("task name", &name, IdentifierKind::Value, line_number);
        if !trailing.trim().is_empty() {
            self.emit(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(53),
                "task_signature",
                Diagnostic::warning(
                    DiagnosticCode::UNEXPECTED_SIGNATURE_TEXT,
                    "unexpected text after task parameter list",
                    Some(self.span(line_number)),
                ),
            );
        }
        let result_syntax = result.as_ref().map(|result| {
            let column =
                self.span(line_number).column + "task ".len() + result_offset.unwrap_or_default();
            parse_type_syntax(result, Span::new(self.path.clone(), line_number, column))
        });
        (name, params, result, result_syntax)
    }

    fn parse_test_header(
        &mut self,
        header: &str,
        line_number: usize,
    ) -> (String, Vec<Param>, Vec<String>) {
        let rest = header.trim_start_matches("test ").trim();
        if rest.contains('(') {
            let signature_column = self.span(line_number).column + "test ".len();
            let (name, params, trailing) =
                self.parse_callable_signature(rest, line_number, signature_column);
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
        signature_column: usize,
    ) -> (String, Vec<Param>, String) {
        let Some(open) = signature.find('(') else {
            return (signature.trim().to_string(), Vec::new(), String::new());
        };
        let Some(close) = matching_delimiter(signature, open, '(', ')') else {
            self.emit(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(54),
                "callable_signature",
                Diagnostic::error(
                    DiagnosticCode::CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN,
                    "callable signature is missing `)`",
                    Some(self.span(line_number)),
                ),
            );
            return (
                signature[..open].trim().to_string(),
                Vec::new(),
                String::new(),
            );
        };

        let name = signature[..open].trim().to_string();
        let params_text = &signature[open + 1..close];
        let trailing = signature[close + 1..].trim().to_string();
        let params = self.parse_params(
            params_text,
            line_number,
            signature_column + signature[..open + 1].chars().count(),
        );
        (name, params, trailing)
    }

    fn parse_params(
        &mut self,
        params_text: &str,
        line_number: usize,
        params_column: usize,
    ) -> Vec<Param> {
        let mut params = Vec::new();
        if params_text.trim().is_empty() {
            return params;
        }

        let mut byte_offset = 0;
        for (param_index, raw_param) in split_top_level_ranges(params_text, ',')
            .into_iter()
            .enumerate()
        {
            let raw_param = &params_text[raw_param.clone()];
            let param = raw_param.trim();
            let leading_bytes = raw_param.len() - raw_param.trim_start().len();
            let column = params_column
                + params_text[..byte_offset].chars().count()
                + raw_param[..leading_bytes].chars().count();
            let param_span = Span::new(self.path.clone(), line_number, column);
            if let Some((name, ty)) = param.split_once(':') {
                let (permission, permission_explicit, name) = parse_param_permission(name.trim());
                let name = name.to_string();
                let colon = param.find(':').unwrap_or_default();
                let raw_type = &param[colon + 1..];
                let type_hws_valid = raw_type
                    .as_bytes()
                    .first()
                    .is_some_and(|byte| matches!(byte, b' ' | b'\t'));
                let type_leading = raw_type.len() - raw_type.trim_start().len();
                let type_column = column + param[..colon + 1].chars().count() + type_leading;
                let ty = ty.trim().to_string();
                self.validate_identifier(
                    "parameter name",
                    &name,
                    IdentifierKind::Value,
                    line_number,
                );
                params.push(Param {
                    name,
                    type_syntax: parse_type_syntax(
                        &ty,
                        Span::new(self.path.clone(), line_number, type_column),
                    ),
                    ty,
                    permission,
                    permission_explicit,
                    type_hws_valid,
                    separator_hws_valid: param_index == 0 || leading_bytes > 0,
                    span: param_span,
                });
            } else {
                self.emit(
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(55),
                    "parameter",
                    Diagnostic::error(
                        DiagnosticCode::PARAMETER_MISSING_TYPE,
                        format!("parameter `{param}` is missing a type"),
                        Some(param_span),
                    ),
                );
            }
            byte_offset += raw_param.len() + 1;
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
        self.emit(
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(56),
            "identifier",
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

    fn emit(
        &mut self,
        cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
        node_role: &'static str,
        diagnostic: Diagnostic,
    ) {
        let event = self.diagnostics.len();
        let semantic_node = self.current_semantic_node.clone().unwrap_or_else(|| {
            format!(
                "parser-source:file-{}:event-{event}",
                self.semantic_file_index
            )
        });
        let semantic_origin =
            format!("parser-node:{semantic_node}:event-{event}:role-{node_role}",);
        let route = vec![
            format!("parser_file_index={}", self.semantic_file_index),
            format!("parser_event={event}"),
            format!("parser_node_role={node_role}"),
            format!("parser_semantic_node={semantic_node}"),
        ];
        let (diagnostic, occurrence) = DiagnosticOccurrence::producer_diagnostic(
            cause_key,
            diagnostic,
            semantic_origin,
            route,
        )
        .expect("parser diagnostic cause and producer identity must be registered");
        self.diagnostic_occurrences
            .insert_owned(occurrence)
            .expect("parser diagnostic occurrences must be unique");
        self.diagnostics.push(diagnostic);
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

fn parse_task_body_syntax(sections: &[Section]) -> Vec<ParsedBodyStatement> {
    let Some(section) = sections.iter().find(|section| section.name == "does") else {
        return Vec::new();
    };
    section
        .lines
        .iter()
        .filter_map(parse_body_statement_syntax)
        .collect()
}

pub(crate) fn executable_call_nodes(
    statements: &[ParsedBodyStatement],
) -> Vec<ParsedExecutableCallNode> {
    let mut calls = Vec::new();
    for (statement_index, statement) in statements.iter().enumerate() {
        match &statement.kind {
            ParsedBodyStatementKind::Return(expression) => {
                collect_executable_calls(expression, statement_index, vec![0], &mut calls)
            }
            ParsedBodyStatementKind::Binding { value, .. } => {
                if let Some(expression) = value {
                    collect_executable_calls(expression, statement_index, vec![0], &mut calls);
                }
            }
            ParsedBodyStatementKind::Other { expressions } => {
                for (expression_index, expression) in expressions.iter().enumerate() {
                    collect_executable_calls(
                        expression,
                        statement_index,
                        vec![expression_index],
                        &mut calls,
                    );
                }
            }
        }
    }
    calls
}

fn collect_executable_calls(
    expression: &ParsedExpression,
    statement_index: usize,
    path: Vec<usize>,
    calls: &mut Vec<ParsedExecutableCallNode>,
) {
    match &expression.kind {
        ParsedExpressionKind::Call(call) => {
            if let ParsedExpressionKind::Identifier(identifier) = &call.callee.kind
                && is_executable_callee(&identifier.name)
            {
                let mut identifier_uses = Vec::new();
                for argument in &call.arguments {
                    collect_call_identifier_uses(argument, false, &mut identifier_uses);
                }
                for (ordinal, identifier_use) in identifier_uses.iter_mut().enumerate() {
                    identifier_use.ordinal = ordinal;
                }
                calls.push(ParsedExecutableCallNode {
                    position: ParsedCallPosition {
                        statement_index,
                        path: path.clone(),
                    },
                    callee: identifier.name.clone(),
                    source: render_parsed_expression(expression),
                    span: expression.span.clone(),
                    identifier_uses,
                });
            }
            for (argument_index, argument) in call.arguments.iter().enumerate() {
                let mut argument_path = path.clone();
                argument_path.push(argument_index);
                collect_executable_calls(argument, statement_index, argument_path, calls);
            }
        }
        ParsedExpressionKind::Permission { value, .. } => {
            collect_executable_calls(value, statement_index, path, calls);
        }
        ParsedExpressionKind::Compound { operands } => {
            for (operand_index, operand) in operands.iter().enumerate() {
                let mut operand_path = path.clone();
                operand_path.push(operand_index);
                collect_executable_calls(operand, statement_index, operand_path, calls);
            }
        }
        ParsedExpressionKind::Identifier(_)
        | ParsedExpressionKind::UIntLiteral(_)
        | ParsedExpressionKind::Unsupported { .. }
        | ParsedExpressionKind::Other => {}
    }
}

fn collect_call_identifier_uses(
    expression: &ParsedExpression,
    consumed: bool,
    identifier_uses: &mut Vec<ParsedCallIdentifierUse>,
) {
    match &expression.kind {
        ParsedExpressionKind::Identifier(identifier) => {
            identifier_uses.push(ParsedCallIdentifierUse {
                name: identifier.name.clone(),
                ordinal: 0,
                consumed,
            });
        }
        ParsedExpressionKind::Permission { permission, value } => collect_call_identifier_uses(
            value,
            consumed || *permission == ParamPermission::Consume,
            identifier_uses,
        ),
        ParsedExpressionKind::Compound { operands } => {
            for operand in operands {
                collect_call_identifier_uses(operand, consumed, identifier_uses);
            }
        }
        ParsedExpressionKind::Call(_)
        | ParsedExpressionKind::UIntLiteral(_)
        | ParsedExpressionKind::Unsupported { .. }
        | ParsedExpressionKind::Other => {}
    }
}

fn is_executable_callee(name: &str) -> bool {
    is_value_identifier(name)
        && !matches!(
            name,
            "borrow" | "change" | "consume" | "expect" | "fail" | "if" | "return" | "try" | "while"
        )
}

fn render_parsed_expression(expression: &ParsedExpression) -> String {
    match &expression.kind {
        ParsedExpressionKind::Identifier(identifier) => identifier.name.clone(),
        ParsedExpressionKind::UIntLiteral(value) => value.to_string(),
        ParsedExpressionKind::Call(call) => {
            let mut rendered = render_parsed_expression(&call.callee);
            rendered.push('(');
            rendered.push_str(
                &call
                    .arguments
                    .iter()
                    .map(render_parsed_expression)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            if call.close_status == ParsedCallCloseStatus::Closed {
                rendered.push(')');
            }
            rendered
        }
        ParsedExpressionKind::Permission { permission, value } => format!(
            "{} {}",
            match permission {
                ParamPermission::Borrow => "borrow",
                ParamPermission::Change => "change",
                ParamPermission::Consume => "consume",
            },
            render_parsed_expression(value)
        ),
        ParsedExpressionKind::Compound { operands } => operands
            .iter()
            .map(render_parsed_expression)
            .collect::<Vec<_>>()
            .join(" "),
        ParsedExpressionKind::Unsupported { .. } | ParsedExpressionKind::Other => String::new(),
    }
}

fn parse_task_effect_syntax(sections: &[Section]) -> Vec<ParsedEffectDeclaration> {
    sections
        .iter()
        .filter_map(|section| {
            let kind = match section.name.as_str() {
                "uses" => ParsedEffectDeclarationKind::Use,
                "changes" => ParsedEffectDeclarationKind::Change,
                "fails when" => ParsedEffectDeclarationKind::Failure,
                _ => return None,
            };
            Some(section.lines.iter().filter_map(move |line| {
                let text = line.text.trim();
                (!text.is_empty() && !text.starts_with('#') && !text.starts_with("//")).then(|| {
                    ParsedEffectDeclaration {
                        kind,
                        span: line.span.clone(),
                    }
                })
            }))
        })
        .flatten()
        .collect()
}

fn parse_body_statement_syntax(line: &SectionLine) -> Option<ParsedBodyStatement> {
    let text = line.text.trim();
    if text.is_empty() || text.starts_with('#') || text.starts_with("//") {
        return None;
    }
    if let Some(rest) = keyword_rest(text, "return") {
        let offset = text.len() - rest.len();
        return Some(ParsedBodyStatement {
            kind: ParsedBodyStatementKind::Return(parse_expression_syntax(
                rest,
                offset_span(&line.span, offset),
            )),
            span: line.span.clone(),
        });
    }
    for (keyword, mutable) in [("let", false), ("change", true)] {
        if let Some(rest) = keyword_rest(text, keyword) {
            let rest_offset = text.len() - rest.len();
            let (left, value) = find_top_level_char(rest, '=').map_or((rest, None), |index| {
                (&rest[..index], Some(&rest[index + 1..]))
            });
            let name_text = left
                .split_once(':')
                .map_or(left, |(name, _annotation)| name)
                .trim();
            let name_offset = rest.find(name_text).unwrap_or_default();
            let name = is_value_identifier(name_text).then(|| ParsedIdentifier {
                name: name_text.to_string(),
                span: offset_span(&line.span, rest_offset + name_offset),
            });
            let value = value.map(|value| {
                let leading = value.len() - value.trim_start().len();
                let value = value.trim();
                let equals = find_top_level_char(rest, '=').unwrap_or(rest.len());
                parse_expression_syntax(
                    value,
                    offset_span(&line.span, rest_offset + equals + 1 + leading),
                )
            });
            return Some(ParsedBodyStatement {
                kind: ParsedBodyStatementKind::Binding {
                    mutable,
                    name,
                    value,
                },
                span: line.span.clone(),
            });
        }
    }
    Some(ParsedBodyStatement {
        kind: ParsedBodyStatementKind::Other {
            expressions: parse_other_statement_expressions(text, &line.span),
        },
        span: line.span.clone(),
    })
}

fn parse_other_statement_expressions(text: &str, span: &Span) -> Vec<ParsedExpression> {
    let candidate = if let Some(rest) = keyword_rest(text, "set") {
        find_top_level_char(rest, '=')
            .map(|index| (&rest[index + 1..], text.len() - rest.len() + index + 1))
    } else if let Some(rest) = keyword_rest(text, "save") {
        let value = rest.split_once(" in ").map_or(rest, |(value, _)| value);
        Some((value, text.len() - rest.len()))
    } else if let Some(rest) = keyword_rest(text, "expect") {
        Some((rest, text.len() - rest.len()))
    } else if let Some(rest) = keyword_rest(text, "fail") {
        Some((rest, text.len() - rest.len()))
    } else if let Some(rest) = keyword_rest(text, "if") {
        Some((
            rest.trim_end_matches('{').trim_end(),
            text.len() - rest.len(),
        ))
    } else if let Some(rest) = keyword_rest(text, "while") {
        Some((
            rest.trim_end_matches('{').trim_end(),
            text.len() - rest.len(),
        ))
    } else if let Some(rest) = keyword_rest(text, "for each") {
        rest.split_once(" in ").map(|(_, collection)| {
            (
                collection.trim_end_matches('{').trim_end(),
                text.len() - collection.len(),
            )
        })
    } else if text != "}" && !text.ends_with(':') {
        Some((text, 0))
    } else {
        None
    };
    candidate
        .filter(|(expression, _)| !expression.trim().is_empty())
        .map(|(expression, offset)| {
            vec![parse_expression_syntax(
                expression,
                offset_span(span, offset),
            )]
        })
        .unwrap_or_default()
}

fn parse_expression_syntax(text: &str, span: Span) -> ParsedExpression {
    let leading = text.len() - text.trim_start().len();
    let text = text.trim();
    let span = offset_span(&span, leading);
    for (keyword, permission) in [
        ("borrow", ParamPermission::Borrow),
        ("change", ParamPermission::Change),
        ("consume", ParamPermission::Consume),
    ] {
        if let Some(rest) = keyword_rest(text, keyword) {
            let offset = text.len() - rest.len();
            return ParsedExpression {
                kind: ParsedExpressionKind::Permission {
                    permission,
                    value: Box::new(parse_expression_syntax(rest, offset_span(&span, offset))),
                },
                span,
            };
        }
    }
    if is_value_identifier(text) {
        return ParsedExpression {
            kind: ParsedExpressionKind::Identifier(ParsedIdentifier {
                name: text.to_string(),
                span: span.clone(),
            }),
            span,
        };
    }
    if !text.is_empty() && text.chars().all(|ch| ch.is_ascii_digit()) {
        return ParsedExpression {
            kind: text.parse::<u64>().map_or(
                ParsedExpressionKind::Unsupported {
                    reason: "uint_literal_out_of_range_v0",
                },
                ParsedExpressionKind::UIntLiteral,
            ),
            span,
        };
    }

    let parser_owned_calls = parser_owned_top_level_call_ranges(text);
    if parser_owned_calls.len() > 1
        || parser_owned_calls
            .first()
            .is_some_and(|range| range.start > 0)
    {
        let operands = parser_owned_calls
            .into_iter()
            .map(|range| {
                parse_expression_syntax(&text[range.clone()], offset_span(&span, range.start))
            })
            .collect();
        return ParsedExpression {
            kind: ParsedExpressionKind::Compound { operands },
            span,
        };
    }

    if let Some(open) = text.find('(') {
        let callee_text = text[..open].trim();
        let callee_offset = text[..open].find(callee_text).unwrap_or_default();
        let callee = parse_expression_syntax(callee_text, offset_span(&span, callee_offset));
        let (inside, close, trailing) = match matching_delimiter(text, open, '(', ')') {
            Some(close) => (
                &text[open + 1..close],
                ParsedCallCloseStatus::Closed,
                &text[close + 1..],
            ),
            None => (&text[open + 1..], ParsedCallCloseStatus::Missing, ""),
        };
        let argument_ranges = split_top_level_ranges(inside, ',');
        let argument_separators_hws_valid = argument_ranges.iter().skip(1).all(|range| {
            inside[range.clone()]
                .as_bytes()
                .first()
                .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
        });
        let arguments = argument_ranges
            .into_iter()
            .filter_map(|range| {
                let raw = &inside[range.clone()];
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return None;
                }
                let leading = raw.len() - raw.trim_start().len();
                Some(parse_expression_syntax(
                    trimmed,
                    offset_span(&span, open + 1 + range.start + leading),
                ))
            })
            .collect();
        let trailing = classify_call_trailing(trailing);
        return ParsedExpression {
            kind: ParsedExpressionKind::Call(ParsedCall {
                callee: Box::new(callee),
                arguments,
                argument_separators_hws_valid,
                close_status: close,
                trailing_status: trailing,
            }),
            span,
        };
    }

    if let Some(open) = text.find('[')
        && text[open + 1..].contains(')')
    {
        let callee_text = text[..open].trim();
        let callee = parse_expression_syntax(callee_text, span.clone());
        return ParsedExpression {
            kind: ParsedExpressionKind::Call(ParsedCall {
                callee: Box::new(callee),
                arguments: Vec::new(),
                argument_separators_hws_valid: true,
                close_status: ParsedCallCloseStatus::Mismatched,
                trailing_status: ParsedCallTrailingStatus::Complete,
            }),
            span,
        };
    }

    let operands = compound_identifier_operands(text, &span);
    if !operands.is_empty() {
        return ParsedExpression {
            kind: ParsedExpressionKind::Compound { operands },
            span,
        };
    }

    ParsedExpression {
        kind: if text.contains("task") || text.contains(')') || text.contains('(') {
            ParsedExpressionKind::Unsupported {
                reason: "unsupported_callable_expression_shape_v0",
            }
        } else {
            ParsedExpressionKind::Other
        },
        span,
    }
}

fn parser_owned_top_level_call_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
    let bytes = text.as_bytes();
    let mut calls = Vec::new();
    let mut index = 0;
    let mut quoted = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            quoted = true;
            index += 1;
            continue;
        }
        if !(byte.is_ascii_lowercase() || byte == b'_')
            || index.checked_sub(1).is_some_and(|previous| {
                bytes[previous].is_ascii_alphanumeric() || matches!(bytes[previous], b'_' | b'.')
            })
        {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
        {
            index += 1;
        }
        let callee = &text[start..index];
        let mut open = index;
        while open < bytes.len() && bytes[open].is_ascii_whitespace() {
            open += 1;
        }
        if !is_executable_callee(callee) || open >= bytes.len() || bytes[open] != b'(' {
            continue;
        }
        let end = matching_delimiter(text, open, '(', ')')
            .map_or(text.len(), |close| close + ')'.len_utf8());
        calls.push(start..end);
    }

    calls
        .iter()
        .filter(|call| {
            !calls
                .iter()
                .any(|candidate| candidate.start < call.start && call.end <= candidate.end)
        })
        .cloned()
        .collect()
}

fn compound_identifier_operands(text: &str, span: &Span) -> Vec<ParsedExpression> {
    let bytes = text.as_bytes();
    let mut operands = Vec::new();
    let mut index = 0;
    let mut quoted = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            quoted = true;
            index += 1;
            continue;
        }
        if byte.is_ascii_lowercase() || byte == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_lowercase()
                    || bytes[index].is_ascii_digit()
                    || bytes[index] == b'_')
            {
                index += 1;
            }
            let name = &text[start..index];
            if is_value_identifier(name) {
                let identifier_span = offset_span(span, start);
                operands.push(ParsedExpression {
                    kind: ParsedExpressionKind::Identifier(ParsedIdentifier {
                        name: name.to_string(),
                        span: identifier_span.clone(),
                    }),
                    span: identifier_span,
                });
            }
            continue;
        }
        index += 1;
    }
    operands
}

fn classify_call_trailing(trailing: &str) -> ParsedCallTrailingStatus {
    let trailing = trailing.trim();
    if trailing.is_empty() {
        ParsedCallTrailingStatus::Complete
    } else if trailing.chars().all(|ch| ch == ')') {
        ParsedCallTrailingStatus::ExtraClose
    } else if trailing.starts_with('(') {
        ParsedCallTrailingStatus::Chained
    } else {
        ParsedCallTrailingStatus::Prose
    }
}

fn parse_type_syntax(text: &str, span: Span) -> TypeSyntax {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix("Result ")
        && let Some(comma) = find_top_level_char(rest, ',')
    {
        let value_text = rest[..comma].trim();
        let root = rest[comma + 1..].trim();
        return TypeSyntax {
            kind: TypeSyntaxKind::Result {
                value: Box::new(parse_type_syntax(value_text, span.clone())),
                failure_root: root.to_string(),
            },
            span,
        };
    }
    if text.starts_with("task") {
        return TypeSyntax {
            kind: parse_callable_type_syntax(text, &span),
            span,
        };
    }
    TypeSyntax {
        kind: if is_type_identifier(text) {
            TypeSyntaxKind::Named {
                name: text.to_string(),
            }
        } else {
            TypeSyntaxKind::Other
        },
        span,
    }
}

fn parse_callable_type_syntax(text: &str, span: &Span) -> TypeSyntaxKind {
    let Some(rest) = text.strip_prefix("task(") else {
        return TypeSyntaxKind::CallableCandidate {
            reason: "callable_type_requires_task_open_paren_v0",
        };
    };
    let open = "task".len();
    let Some(close) = matching_delimiter(text, open, '(', ')') else {
        return TypeSyntaxKind::CallableCandidate {
            reason: "callable_type_missing_close_paren_v0",
        };
    };
    let after = &text[close + 1..];
    let leading = after.len() - after.trim_start_matches([' ', '\t']).len();
    if leading == 0 {
        return TypeSyntaxKind::CallableCandidate {
            reason: "callable_type_requires_space_before_arrow_v0",
        };
    }
    let after = &after[leading..];
    let Some(result_text) = after.strip_prefix("->") else {
        return TypeSyntaxKind::CallableCandidate {
            reason: "callable_type_missing_arrow_v0",
        };
    };
    let result_leading = result_text.len() - result_text.trim_start_matches([' ', '\t']).len();
    if result_leading == 0 || result_text[result_leading..].is_empty() {
        return TypeSyntaxKind::CallableCandidate {
            reason: "callable_type_requires_result_v0",
        };
    }
    let inside = &rest[..close - open - 1];
    let inputs = split_top_level_ranges(inside, ',')
        .into_iter()
        .filter_map(|range| {
            let raw = &inside[range.clone()];
            let value = raw.trim();
            (!value.is_empty())
                .then(|| parse_type_syntax(value, offset_span(span, open + 1 + range.start)))
        })
        .collect();
    let result = parse_type_syntax(
        result_text[result_leading..].trim(),
        offset_span(span, close + 1 + leading + 2 + result_leading),
    );
    TypeSyntaxKind::Callable(CallableTypeSyntax {
        inputs,
        result: Box::new(result),
    })
}

fn keyword_rest<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    text.strip_prefix(keyword)
        .and_then(|rest| rest.strip_prefix([' ', '\t']))
        .map(|rest| rest.trim_start_matches([' ', '\t']))
}

fn find_top_level_arrow(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index + 1 < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b'-' if bytes[index + 1] == b'>' && depth == 0 => return Some(index),
            _ => {}
        }
        index += 1;
    }
    None
}

fn matching_delimiter(text: &str, open: usize, open_ch: char, close_ch: char) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in text.char_indices().skip_while(|(index, _)| *index < open) {
        if ch == open_ch {
            depth += 1;
        } else if ch == close_ch {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn split_top_level_ranges(text: &str, delimiter: char) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if ch == delimiter && depth == 0 => {
                ranges.push(start..index);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    ranges.push(start..text.len());
    ranges
}

fn find_top_level_char(text: &str, needle: char) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if ch == needle && depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

fn offset_span(span: &Span, byte_offset: usize) -> Span {
    Span::new(span.file.clone(), span.line, span.column + byte_offset)
}

fn is_value_identifier(text: &str) -> bool {
    is_valid_identifier(text, IdentifierKind::Value)
}

fn is_type_identifier(text: &str) -> bool {
    is_valid_identifier(text, IdentifierKind::Type)
}

fn parse_param_permission(raw_name: &str) -> (ParamPermission, bool, &str) {
    let raw_name = raw_name.trim();
    let Some(first) = raw_name.split_whitespace().next() else {
        return (ParamPermission::Borrow, false, raw_name);
    };
    let permission = match first {
        "borrow" => ParamPermission::Borrow,
        "change" => ParamPermission::Change,
        "consume" => ParamPermission::Consume,
        _ => return (ParamPermission::Borrow, false, raw_name),
    };
    let name = raw_name[first.len()..].trim();
    (permission, true, name)
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
    use super::{executable_call_nodes, parse_source, parse_source_at_index};
    use crate::ast::{
        Item, ParsedBodyStatementKind, ParsedCallCloseStatus, ParsedExpressionKind, TypeSyntaxKind,
    };
    use crate::diagnostic::{DiagnosticCode, Severity};

    #[test]
    fn parser_body_syntax_owns_repeated_sibling_and_nested_calls() {
        let parsed = parse_source(
            "parser-owned-calls.hum",
            r#"task caller(value: UInt) -> UInt {
  does:
    return leaf(value) + leaf(value) + leaf(leaf(consume value))
}
"#,
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let calls = executable_call_nodes(&task.body_syntax);
        assert_eq!(calls.len(), 4);
        assert_eq!(
            calls
                .iter()
                .map(|call| call.position.stable_component())
                .collect::<Vec<_>>(),
            [
                "statement-0:path-0.0",
                "statement-0:path-0.1",
                "statement-0:path-0.2",
                "statement-0:path-0.2.0",
            ]
        );
        assert_eq!(
            calls
                .iter()
                .map(|call| call.source.as_str())
                .collect::<Vec<_>>(),
            [
                "leaf(value)",
                "leaf(value)",
                "leaf(leaf(consume value))",
                "leaf(consume value)",
            ]
        );
        assert!(
            calls[3]
                .identifier_uses
                .iter()
                .any(|identifier| identifier.name == "value" && identifier.consumed)
        );
    }

    #[test]
    fn parser_occurrence_identity_uses_only_producer_owned_file_and_event_facts() {
        let first = parse_source_at_index("display-one.hum", "unexpected prose\n", 3);
        let renamed = parse_source_at_index("renamed-display.hum", "unexpected prose\n", 3);
        let other_file = parse_source_at_index("display-one.hum", "unexpected prose\n", 4);
        let first = first
            .diagnostic_occurrences
            .occurrences()
            .next()
            .expect("parser occurrence");
        let renamed = renamed
            .diagnostic_occurrences
            .occurrences()
            .next()
            .expect("renamed parser occurrence");
        let other_file = other_file
            .diagnostic_occurrences
            .occurrences()
            .next()
            .expect("other-file parser occurrence");

        assert_eq!(first.semantic_origin(), renamed.semantic_origin());
        assert_eq!(first.relationship_route(), renamed.relationship_route());
        assert_ne!(first.semantic_origin(), other_file.semantic_origin());
        assert!(!first.semantic_origin().contains(".hum"));
        assert!(
            first
                .relationship_route()
                .iter()
                .all(|entry| !entry.contains(".hum"))
        );
    }

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

    #[test]
    fn callable_type_and_indirect_call_are_parser_owned_nodes() {
        let parsed = parse_source(
            "callable_nodes.hum",
            "task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {\n  does:\n    return transform(value)\n}\n",
        );
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let TypeSyntaxKind::Callable(callable) = &task.params[0].type_syntax.kind else {
            panic!("callable type")
        };
        assert_eq!(callable.inputs.len(), 1);
        let ParsedBodyStatementKind::Return(expression) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        let ParsedExpressionKind::Call(call) = &expression.kind else {
            panic!("call")
        };
        assert_eq!(call.close_status, ParsedCallCloseStatus::Closed);
        assert_eq!(call.arguments.len(), 1);
    }

    #[test]
    fn missing_indirect_close_remains_a_structured_candidate() {
        let parsed = parse_source(
            "callable_missing_close.hum",
            "task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {\n  does:\n    return transform(value\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let ParsedBodyStatementKind::Return(expression) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        let ParsedExpressionKind::Call(call) = &expression.kind else {
            panic!("call candidate")
        };
        assert_eq!(call.close_status, ParsedCallCloseStatus::Missing);
    }
}
