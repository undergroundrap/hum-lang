use crate::ast::{
    App, CallableTypeSyntax, CanonicalExpression, CanonicalExpressionKind, Field, Item, Param,
    ParamPermission, ParsedActualLexicalEvidence, ParsedBinaryOperator, ParsedBlockRelationship,
    ParsedBodyStatement, ParsedBodyStatementKind, ParsedCall, ParsedCallCloseStatus,
    ParsedCallSyntaxFacts, ParsedCallTrailingStatus, ParsedCanonicalNodeSyntax,
    ParsedDelimiterKind, ParsedDelimiterSyntax, ParsedEffectDeclaration,
    ParsedEffectDeclarationKind, ParsedExpectedLexicalEvidence, ParsedExpression,
    ParsedExpressionIntent, ParsedExpressionKind, ParsedExpressionOccurrenceFacts,
    ParsedIdentifier, ParsedLexicalStatus, ParsedLexicalTokenKind, ParsedLoopKind,
    ParsedLoopRelationshipFacts, ParsedLoopRelationshipKind, ParsedMalformedExpressionCause,
    ParsedMalformedExpressionFact, ParsedSourceRange, ParsedStatementSyntaxFacts,
    ParsedStatementSyntaxKind, ParsedTypedFailureWrapperKind, ParsedTypedFailureWrapperSyntax,
    ParserSyntaxNodeId, Section, SectionLine, SourceFile, Store, Task, Test, TypeDef, TypeSyntax,
    TypeSyntaxKind,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOccurrence, Span};
use crate::syntax;
use crate::typed_failure;

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
        self.emit_chained_comparison_diagnostics(&sections);
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
        let mut quoted = false;
        let mut escaped = false;
        for index in start..self.lines.len() {
            let text = &self.lines[index].text;
            for ch in text.chars() {
                if quoted {
                    if escaped {
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        quoted = false;
                    }
                    continue;
                }
                if ch == '"' {
                    quoted = true;
                    continue;
                }
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
            // Text literals are line-scoped in the current surface. A malformed
            // quote remains a malformed line; it cannot consume later item braces.
            quoted = false;
            escaped = false;
        }
        None
    }

    fn parse_sections(&mut self, start: usize, end: usize, section_indent: usize) -> Vec<Section> {
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

                let body_syntax = if name == "does" {
                    let semantic_node = self
                        .current_semantic_node
                        .as_deref()
                        .unwrap_or("resolver-item:unknown");
                    let retained = retain_body_syntax(&lines, semantic_node);
                    for line in &lines {
                        if let Some(identifier) =
                            invalid_non_ascii_return_identifier(line.text.trim())
                        {
                            self.invalid_identifier(
                                "value",
                                identifier,
                                IdentifierKind::Value,
                                line.span.line,
                            );
                        }
                    }
                    retained
                } else {
                    vec![None; lines.len()]
                };
                let mut expression_syntax = vec![None; lines.len()];
                if matches!(name.as_str(), "needs" | "ensures") {
                    let semantic_node = self
                        .current_semantic_node
                        .as_deref()
                        .unwrap_or("resolver-item:unknown");
                    let intent = if name == "needs" {
                        ParsedExpressionIntent::NeedsPredicate
                    } else {
                        ParsedExpressionIntent::EnsuresPredicate
                    };
                    for (line_index, section_line) in lines.iter().enumerate() {
                        let text = section_line.text.trim();
                        if text.is_empty() || text.starts_with('#') || text.starts_with("//") {
                            continue;
                        }
                        expression_syntax[line_index] = Some(parse_expression_syntax(
                            text,
                            section_line.span.clone(),
                            ParserSyntaxNodeId::new(format!(
                                "parser-contract:{semantic_node}:section-{name}:line-{line_index}"
                            )),
                            intent,
                        ));
                    }
                }
                sections.push(Section {
                    name,
                    lines,
                    body_syntax,
                    expression_syntax,
                    span: self.span(line.number),
                });
                index = cursor;
            } else {
                index += 1;
            }
        }

        sections
    }

    fn emit_chained_comparison_diagnostics(&mut self, sections: &[Section]) {
        for section in sections {
            for expression in section.expression_syntax.iter().flatten() {
                self.emit_expression_chains(expression);
            }
            for statement in section.body_syntax.iter().flatten() {
                for expression in statement_expressions(statement) {
                    self.emit_expression_chains(expression);
                }
            }
        }
    }

    fn emit_expression_chains(&mut self, expression: &ParsedExpression) {
        for chain in chained_comparison_sites(expression) {
            let diagnostic = Diagnostic::error(
                DiagnosticCode::CHAINED_COMPARISON_NOT_SUPPORTED,
                "comparison chaining is not supported",
                Some(chain.later.start.clone()),
            )
            .with_related_span(
                "first comparison already being chained",
                chain.first.start.clone(),
            )
            .with_help(
                "Repeat the middle operand and join independent comparisons, for example `1 < 2 and 2 < 3`.",
            );
            self.emit_for_semantic_node(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(179),
                "expression_node",
                chain.node_id.as_str().to_string(),
                diagnostic,
            );
        }
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
        self.emit_for_semantic_node(cause_key, node_role, semantic_node, diagnostic);
    }

    fn emit_for_semantic_node(
        &mut self,
        cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
        node_role: &'static str,
        semantic_node: String,
        diagnostic: Diagnostic,
    ) {
        let event = self.diagnostics.len();
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
    let statements = section
        .body_syntax
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    if validate_retained_body_syntax(&statements).is_ok() {
        statements
    } else {
        Vec::new()
    }
}

pub(crate) fn validate_retained_body_syntax(
    statements: &[ParsedBodyStatement],
) -> Result<(), &'static str> {
    let mut depth = 0usize;
    let mut identities = std::collections::BTreeSet::new();
    for (index, statement) in statements.iter().enumerate() {
        if !identities.insert(statement.source_node_id.as_str())
            || !statement
                .source_node_id
                .as_str()
                .ends_with(&format!("statement-{index}"))
            || statement.block_depth_before != depth
        {
            return Err("parser_body_identity_or_depth_corrupt_v0");
        }
        depth = match statement.block_relationship {
            ParsedBlockRelationship::Opens => depth.saturating_add(1),
            ParsedBlockRelationship::Closes => depth.saturating_sub(1),
            ParsedBlockRelationship::None => depth,
        };
        if statement.block_depth_after != depth {
            return Err("parser_body_relationship_corrupt_v0");
        }
        match &statement.kind {
            ParsedBodyStatementKind::Return(expression) => {
                validate_expression_occurrence(expression)?;
            }
            ParsedBodyStatementKind::Binding { value, .. } => {
                if let Some(expression) = value {
                    validate_expression_occurrence(expression)?;
                }
            }
            ParsedBodyStatementKind::Other { expressions } => {
                for expression in expressions {
                    validate_expression_occurrence(expression)?;
                }
            }
        }
        if let Some(target) = &statement.syntax.target {
            validate_expression_occurrence(target)?;
        }
        let exact_nodes = statement_kind_expressions(&statement.kind)
            .into_iter()
            .map(|expression| expression.canonical.node_id.clone())
            .collect::<Vec<_>>();
        if statement.syntax.expression_nodes != exact_nodes {
            return Err("parser_statement_expression_relationship_corrupt_v0");
        }
        match statement.syntax.kind {
            ParsedStatementSyntaxKind::Loop {
                kind: ParsedLoopKind::ForEach,
            } => {
                let Some(binding) = statement.syntax.binding.as_ref() else {
                    return Err("parser_loop_binding_relationship_corrupt_v0");
                };
                let Some(relationship) = statement.syntax.loop_relationship.as_ref() else {
                    return Err("parser_loop_binding_relationship_corrupt_v0");
                };
                let expressions = statement_kind_expressions(&statement.kind);
                if relationship.kind != ParsedLoopRelationshipKind::CollectionIn
                    || relationship.binding != *binding
                    || relationship.introducer.byte_len != 2
                    || relationship.bound.is_some()
                    || relationship.expression_nodes != exact_nodes
                    || expressions.len() != 1
                    || expressions[0].occurrence.intent != ParsedExpressionIntent::LoopCollection
                    || binding.span.file != relationship.introducer.start.file
                    || binding.span.line != relationship.introducer.start.line
                    || binding.span.column >= relationship.introducer.start.column
                    || relationship.introducer.start.column
                        >= expressions[0].canonical.range.start.column
                {
                    return Err("parser_loop_binding_relationship_corrupt_v0");
                }
            }
            ParsedStatementSyntaxKind::Loop {
                kind: ParsedLoopKind::ForIndex,
            } => {
                let Some(binding) = statement.syntax.binding.as_ref() else {
                    return Err("parser_loop_binding_relationship_corrupt_v0");
                };
                let Some(relationship) = statement.syntax.loop_relationship.as_ref() else {
                    return Err("parser_loop_binding_relationship_corrupt_v0");
                };
                let Some(bound) = relationship.bound.as_ref() else {
                    return Err("parser_loop_binding_relationship_corrupt_v0");
                };
                let expressions = statement_kind_expressions(&statement.kind);
                if !matches!(
                    relationship.kind,
                    ParsedLoopRelationshipKind::RangeUntil
                        | ParsedLoopRelationshipKind::RangeThrough
                ) || relationship.binding != *binding
                    || relationship.introducer.byte_len != 4
                    || bound.byte_len
                        != match relationship.kind {
                            ParsedLoopRelationshipKind::RangeUntil => 5,
                            ParsedLoopRelationshipKind::RangeThrough => 7,
                            ParsedLoopRelationshipKind::CollectionIn => unreachable!(),
                        }
                    || relationship.expression_nodes != exact_nodes
                    || expressions.len() != 2
                    || expressions[0].occurrence.intent != ParsedExpressionIntent::LoopRangeStart
                    || expressions[1].occurrence.intent != ParsedExpressionIntent::LoopRangeEnd
                    || binding.span.file != relationship.introducer.start.file
                    || binding.span.line != relationship.introducer.start.line
                    || binding.span.column >= relationship.introducer.start.column
                    || relationship.introducer.start.column
                        >= expressions[0].canonical.range.start.column
                    || expressions[0].canonical.range.start.column >= bound.start.column
                    || bound.start.column >= expressions[1].canonical.range.start.column
                {
                    return Err("parser_loop_binding_relationship_corrupt_v0");
                }
            }
            ParsedStatementSyntaxKind::Loop {
                kind: ParsedLoopKind::While | ParsedLoopKind::Unconditional,
            } => {
                if statement.syntax.loop_relationship.is_some() {
                    return Err("parser_loop_binding_relationship_corrupt_v0");
                }
            }
            _ => {
                if statement.syntax.loop_relationship.is_some() {
                    return Err("parser_loop_binding_relationship_corrupt_v0");
                }
            }
        }
    }
    Ok(())
}

fn retain_body_syntax(
    lines: &[SectionLine],
    semantic_node: &str,
) -> Vec<Option<ParsedBodyStatement>> {
    let mut depth = 0usize;
    let mut statement_index = 0usize;
    let mut retained = Vec::with_capacity(lines.len());
    for line in lines {
        let text = line.text.trim();
        if text.is_empty() || text.starts_with('#') || text.starts_with("//") {
            retained.push(None);
            continue;
        }
        let node_id = ParserSyntaxNodeId::new(format!(
            "parser-body:{semantic_node}:statement-{statement_index}"
        ));
        let relationship = parser_block_relationship(text);
        let before = depth;
        depth = match relationship {
            ParsedBlockRelationship::Opens => depth.saturating_add(1),
            ParsedBlockRelationship::Closes => depth.saturating_sub(1),
            ParsedBlockRelationship::None => depth,
        };
        retained.push(parse_body_statement_syntax(
            line,
            node_id,
            relationship,
            before,
            depth,
        ));
        statement_index += 1;
    }
    retained
}

fn parser_block_relationship(text: &str) -> ParsedBlockRelationship {
    if text == "}" {
        ParsedBlockRelationship::Closes
    } else if unquoted_last_non_whitespace(text) == Some('{') {
        ParsedBlockRelationship::Opens
    } else {
        ParsedBlockRelationship::None
    }
}

fn invalid_non_ascii_return_identifier(text: &str) -> Option<&str> {
    let candidate = keyword_rest(text, "return")?.trim();
    (!candidate.is_empty()
        && !candidate.is_ascii()
        && candidate
            .chars()
            .all(|ch| ch == '_' || ch.is_alphanumeric())
        && !is_valid_identifier(candidate, IdentifierKind::Value))
    .then_some(candidate)
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

fn parse_body_statement_syntax(
    line: &SectionLine,
    source_node_id: ParserSyntaxNodeId,
    block_relationship: ParsedBlockRelationship,
    block_depth_before: usize,
    block_depth_after: usize,
) -> Option<ParsedBodyStatement> {
    let text = line.text.trim();
    if text.is_empty() || text.starts_with('#') || text.starts_with("//") {
        return None;
    }
    if text == "return" {
        let expression = parse_expression_syntax(
            "",
            offset_span(&line.span, text.len()),
            source_node_id.child("expression-0"),
            ParsedExpressionIntent::Return,
        );
        return Some(parsed_body_statement(
            ParsedBodyStatementKind::Return(expression),
            line,
            source_node_id,
            block_relationship,
            block_depth_before,
            block_depth_after,
        ));
    }
    if let Some(rest) = keyword_rest(text, "return") {
        let offset = text.len() - rest.len();
        let expression = parse_expression_syntax(
            rest,
            offset_span(&line.span, offset),
            source_node_id.child("expression-0"),
            ParsedExpressionIntent::Return,
        );
        return Some(parsed_body_statement(
            ParsedBodyStatementKind::Return(expression),
            line,
            source_node_id,
            block_relationship,
            block_depth_before,
            block_depth_after,
        ));
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
                    source_node_id.child("expression-0"),
                    ParsedExpressionIntent::Binding,
                )
            });
            return Some(parsed_body_statement(
                ParsedBodyStatementKind::Binding {
                    mutable,
                    name,
                    value,
                },
                line,
                source_node_id,
                block_relationship,
                block_depth_before,
                block_depth_after,
            ));
        }
    }
    Some(parsed_body_statement(
        ParsedBodyStatementKind::Other {
            expressions: parse_other_statement_expressions(text, &line.span, &source_node_id),
        },
        line,
        source_node_id,
        block_relationship,
        block_depth_before,
        block_depth_after,
    ))
}

fn parsed_body_statement(
    kind: ParsedBodyStatementKind,
    line: &SectionLine,
    source_node_id: ParserSyntaxNodeId,
    block_relationship: ParsedBlockRelationship,
    block_depth_before: usize,
    block_depth_after: usize,
) -> ParsedBodyStatement {
    let (core_kind, core_status, core_expression_kind, core_reason) =
        parser_core_shape(line.text.trim(), &kind);
    let syntax = parser_statement_syntax(line.text.trim(), &line.span, &source_node_id, &kind);
    ParsedBodyStatement {
        kind,
        syntax,
        span: line.span.clone(),
        source_node_id,
        block_relationship,
        block_depth_before,
        block_depth_after,
        core_kind,
        core_status,
        core_expression_kind,
        core_reason,
    }
}

struct ParsedLoopHeaderParts<'a> {
    kind: ParsedLoopKind,
    binder: &'a str,
    binder_offset: usize,
    introducer_offset: usize,
    introducer_len: usize,
    bound: Option<(ParsedLoopRelationshipKind, usize, usize)>,
    expressions: Vec<(&'a str, usize, ParsedExpressionIntent)>,
}

fn parsed_loop_header_parts(text: &str) -> Option<ParsedLoopHeaderParts<'_>> {
    if let Some(rest) = keyword_rest(text, "for each") {
        let rest_offset = text.len() - rest.len();
        let body = rest.strip_suffix('{')?.trim_end();
        let split = find_top_level_phrase(body, " in ")?;
        let binder_raw = &body[..split];
        let binder = binder_raw.trim();
        let binder_leading = binder_raw.len() - binder_raw.trim_start().len();
        let collection_raw = &body[split + " in ".len()..];
        let collection = collection_raw.trim();
        let collection_leading = collection_raw.len() - collection_raw.trim_start().len();
        return Some(ParsedLoopHeaderParts {
            kind: ParsedLoopKind::ForEach,
            binder,
            binder_offset: rest_offset + binder_leading,
            introducer_offset: rest_offset + split + 1,
            introducer_len: 2,
            bound: Some((ParsedLoopRelationshipKind::CollectionIn, 0, 0)),
            expressions: if collection.is_empty() {
                Vec::new()
            } else {
                vec![(
                    collection,
                    rest_offset + split + " in ".len() + collection_leading,
                    ParsedExpressionIntent::LoopCollection,
                )]
            },
        });
    }

    let rest = keyword_rest(text, "for index")?;
    let rest_offset = text.len() - rest.len();
    let body = rest.strip_suffix('{')?.trim_end();
    let from_split = find_top_level_phrase(body, " from ")?;
    let binder_raw = &body[..from_split];
    let binder = binder_raw.trim();
    let binder_leading = binder_raw.len() - binder_raw.trim_start().len();
    let range_raw = &body[from_split + " from ".len()..];
    let range_offset = rest_offset + from_split + " from ".len();
    let (kind, bound_text, bound_split) = [
        (ParsedLoopRelationshipKind::RangeUntil, " until "),
        (ParsedLoopRelationshipKind::RangeThrough, " through "),
    ]
    .into_iter()
    .filter_map(|(kind, token)| {
        find_top_level_phrase(range_raw, token).map(|split| (kind, token, split))
    })
    .min_by_key(|(_, _, split)| *split)?;
    let start_raw = &range_raw[..bound_split];
    let end_raw = &range_raw[bound_split + bound_text.len()..];
    let start = start_raw.trim();
    let end = end_raw.trim();
    let start_leading = start_raw.len() - start_raw.trim_start().len();
    let end_leading = end_raw.len() - end_raw.trim_start().len();
    let mut expressions = Vec::new();
    if !start.is_empty() {
        expressions.push((
            start,
            range_offset + start_leading,
            ParsedExpressionIntent::LoopRangeStart,
        ));
    }
    if !end.is_empty() {
        expressions.push((
            end,
            range_offset + bound_split + bound_text.len() + end_leading,
            ParsedExpressionIntent::LoopRangeEnd,
        ));
    }
    Some(ParsedLoopHeaderParts {
        kind: ParsedLoopKind::ForIndex,
        binder,
        binder_offset: rest_offset + binder_leading,
        introducer_offset: rest_offset + from_split + 1,
        introducer_len: 4,
        bound: Some((
            kind,
            range_offset + bound_split + 1,
            bound_text.trim().len(),
        )),
        expressions,
    })
}

fn parser_statement_syntax(
    text: &str,
    span: &Span,
    source_node_id: &ParserSyntaxNodeId,
    body_kind: &ParsedBodyStatementKind,
) -> ParsedStatementSyntaxFacts {
    let keyword_text = text.split_ascii_whitespace().next().unwrap_or_default();
    let mut facts = ParsedStatementSyntaxFacts {
        kind: ParsedStatementSyntaxKind::Other,
        keyword: source_range(span, 0, keyword_text.len()),
        binding: None,
        target: None,
        destination: None,
        relationship_token: None,
        loop_relationship: None,
        expression_nodes: statement_kind_expressions(body_kind)
            .into_iter()
            .map(|expression| expression.canonical.node_id.clone())
            .collect(),
    };
    if text == "}" {
        facts.kind = ParsedStatementSyntaxKind::BlockClose;
        return facts;
    }
    if keyword_rest(text, "return").is_some() || text == "return" {
        facts.kind = ParsedStatementSyntaxKind::Return;
        return facts;
    }
    if let ParsedBodyStatementKind::Binding { mutable, name, .. } = body_kind {
        facts.kind = ParsedStatementSyntaxKind::Binding { mutable: *mutable };
        facts.binding = name.clone();
        if let Some(equals) = find_top_level_char(text, '=') {
            facts.relationship_token = Some(source_range(span, equals, 1));
        }
        return facts;
    }
    if let Some(rest) = keyword_rest(text, "set") {
        facts.kind = ParsedStatementSyntaxKind::Set;
        if let Some(equals) = find_top_level_char(rest, '=') {
            let rest_offset = text.len() - rest.len();
            let target_raw = &rest[..equals];
            let target = target_raw.trim();
            if !target.is_empty() {
                let target_leading = target_raw.len() - target_raw.trim_start().len();
                facts.target = Some(parse_expression_syntax(
                    target,
                    offset_span(span, rest_offset + target_leading),
                    source_node_id.child("set-target"),
                    ParsedExpressionIntent::Other,
                ));
            }
            facts.relationship_token = Some(source_range(span, rest_offset + equals, 1));
        }
        return facts;
    }
    if let Some(rest) = keyword_rest(text, "save") {
        facts.kind = ParsedStatementSyntaxKind::Save;
        if let Some(split) = find_top_level_phrase(rest, " in ") {
            let rest_offset = text.len() - rest.len();
            let destination_text = rest[split + " in ".len()..].trim();
            let destination_leading = rest[split + " in ".len()..].len()
                - rest[split + " in ".len()..].trim_start().len();
            facts.destination = is_value_identifier(destination_text).then(|| ParsedIdentifier {
                name: destination_text.to_string(),
                span: offset_span(
                    span,
                    rest_offset + split + " in ".len() + destination_leading,
                ),
            });
            facts.relationship_token = Some(source_range(span, rest_offset + split + 1, 2));
        }
        return facts;
    }
    if keyword_rest(text, "if").is_some() {
        facts.kind = ParsedStatementSyntaxKind::Condition;
        return facts;
    }
    if keyword_rest(text, "while").is_some() {
        facts.kind = ParsedStatementSyntaxKind::Loop {
            kind: ParsedLoopKind::While,
        };
        facts.keyword = source_range(span, 0, "while".len());
        return facts;
    }
    if let Some(loop_header) = parsed_loop_header_parts(text) {
        facts.kind = ParsedStatementSyntaxKind::Loop {
            kind: loop_header.kind,
        };
        facts.keyword = source_range(
            span,
            0,
            match loop_header.kind {
                ParsedLoopKind::ForEach => "for each".len(),
                ParsedLoopKind::ForIndex => "for index".len(),
                ParsedLoopKind::While | ParsedLoopKind::Unconditional => unreachable!(),
            },
        );
        facts.binding = (!loop_header.binder.is_empty()).then(|| ParsedIdentifier {
            name: loop_header.binder.to_string(),
            span: offset_span(span, loop_header.binder_offset),
        });
        let relationship_kind = loop_header
            .bound
            .map(|(kind, _, _)| kind)
            .unwrap_or(ParsedLoopRelationshipKind::CollectionIn);
        facts.loop_relationship =
            facts
                .binding
                .clone()
                .map(|binding| ParsedLoopRelationshipFacts {
                    kind: relationship_kind,
                    binding,
                    introducer: source_range(
                        span,
                        loop_header.introducer_offset,
                        loop_header.introducer_len,
                    ),
                    bound: loop_header.bound.and_then(|(_, offset, len)| {
                        (len != 0).then(|| source_range(span, offset, len))
                    }),
                    expression_nodes: facts.expression_nodes.clone(),
                });
        return facts;
    }
    if text == "loop {" {
        facts.kind = ParsedStatementSyntaxKind::Loop {
            kind: ParsedLoopKind::Unconditional,
        };
        return facts;
    }
    if keyword_rest(text, "fail").is_some() {
        facts.kind = ParsedStatementSyntaxKind::Failure;
    } else if keyword_rest(text, "expect").is_some() {
        facts.kind = ParsedStatementSyntaxKind::TestExpectation;
    }
    facts
}

fn statement_kind_expressions(kind: &ParsedBodyStatementKind) -> Vec<&ParsedExpression> {
    match kind {
        ParsedBodyStatementKind::Return(expression) => vec![expression],
        ParsedBodyStatementKind::Binding { value, .. } => value.iter().collect(),
        ParsedBodyStatementKind::Other { expressions } => expressions.iter().collect(),
    }
}

fn statement_expressions(statement: &ParsedBodyStatement) -> Vec<&ParsedExpression> {
    let mut expressions = statement_kind_expressions(&statement.kind);
    if let Some(target) = &statement.syntax.target {
        expressions.push(target);
    }
    expressions
}

fn parser_core_shape(
    text: &str,
    kind: &ParsedBodyStatementKind,
) -> (
    &'static str,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
) {
    if text == "}" {
        return ("block_close", "recognized_v0", None, None);
    }
    if matches!(
        text.strip_suffix(':').map(str::trim),
        Some("keeps" | "changes" | "needs" | "ensures" | "watch for" | "cost" | "does")
    ) {
        return (
            "nested_intent_header",
            "recognized_v0",
            None,
            Some("nested_intent_lowering_not_implemented"),
        );
    }
    if let Some(rest) = keyword_rest(text, "if")
        && unquoted_last_non_whitespace(rest) == Some('{')
    {
        return (
            "if_header",
            "recognized_v0",
            Some(parser_expression_kind_for_condition(
                rest.trim_end_matches('{').trim_end(),
            )),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "while")
        && unquoted_last_non_whitespace(rest) == Some('{')
    {
        return (
            "while_header",
            "recognized_v0",
            Some(parser_expression_kind_for_condition(
                rest.trim_end_matches('{').trim_end(),
            )),
            None,
        );
    }
    if text == "loop {" {
        return ("loop_header", "recognized_v0", None, None);
    }
    if let Some(rest) = keyword_rest(text, "for each")
        && unquoted_last_non_whitespace(rest) == Some('{')
    {
        return (
            "for_each_header",
            "recognized_v0",
            Some(parser_expression_kind(
                rest.trim_end_matches('{').trim_end(),
            )),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "for index")
        && unquoted_last_non_whitespace(rest) == Some('{')
    {
        return (
            "for_index_header",
            "recognized_v0",
            Some(parser_expression_kind(
                rest.trim_end_matches('{').trim_end(),
            )),
            None,
        );
    }
    if text == "return" {
        return (
            "return",
            "recognized_v0",
            Some(parser_expression_kind("")),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "return") {
        return (
            "return",
            "recognized_v0",
            Some(parser_expression_kind(rest)),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "fail") {
        return (
            "fail",
            "recognized_v0",
            Some(parser_expression_kind(rest)),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "change") {
        let accepted = matches!(
            kind,
            ParsedBodyStatementKind::Binding { value: Some(_), .. }
        );
        return (
            "mutable_binding",
            if accepted {
                "recognized_v0"
            } else {
                "unsupported_v0"
            },
            rest.split_once('=')
                .map(|(_, value)| parser_expression_kind(value.trim())),
            (!accepted).then_some("binding_missing_initializer"),
        );
    }
    if let Some(rest) = keyword_rest(text, "let") {
        let accepted = matches!(
            kind,
            ParsedBodyStatementKind::Binding { value: Some(_), .. }
        );
        return (
            "let_binding",
            if accepted {
                "recognized_v0"
            } else {
                "unsupported_v0"
            },
            rest.split_once('=')
                .map(|(_, value)| parser_expression_kind(value.trim())),
            (!accepted).then_some("binding_missing_initializer"),
        );
    }
    if let Some(rest) = keyword_rest(text, "set") {
        return (
            "set_place",
            "recognized_v0",
            rest.split_once('=')
                .map(|(_, value)| parser_expression_kind(value.trim())),
            None,
        );
    }
    if let Some(rest) = keyword_rest(text, "expect") {
        return (
            "test_expectation",
            "recognized_v0",
            Some(parser_expression_kind(rest)),
            Some("test_body_not_core_runtime"),
        );
    }
    if text.starts_with("save ") && text.contains(" in ") {
        return (
            "save_in_store",
            "unsupported_v0",
            None,
            Some("surface_save_requires_store_lowering"),
        );
    }
    if is_record_field_initializer_text(text) {
        return (
            "record_field_initializer",
            "recognized_v0",
            text.split_once(':')
                .map(|(_, value)| parser_expression_kind(value.trim())),
            Some("record_literal_lowering_not_implemented"),
        );
    }
    (
        "unknown_body_line",
        "unsupported_v0",
        None,
        Some("not_in_core_body_grammar_v0"),
    )
}

fn is_record_field_initializer_text(text: &str) -> bool {
    let Some((field, value)) = text.split_once(':') else {
        return false;
    };
    !text.ends_with(':')
        && !field.trim().is_empty()
        && field
            .trim()
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == ' ')
        && !value.trim().is_empty()
}

fn parser_expression_kind_for_condition(text: &str) -> &'static str {
    if parser_has_binary_operator(text) || text.contains(" is ") || text.contains(" does ") {
        "condition_text"
    } else {
        parser_expression_kind(text)
    }
}

fn parser_expression_kind(text: &str) -> &'static str {
    let text = text.trim();
    if typed_failure::is_try_candidate(text) {
        "try_call_like"
    } else if text.is_empty() {
        "unit"
    } else if text == "true" || text == "false" {
        "bool_literal"
    } else if text.chars().all(|ch| ch.is_ascii_digit()) {
        "int_literal"
    } else if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        "text_literal"
    } else if text.ends_with('{') {
        "record_literal_start"
    } else if text.contains('(') && text.contains(')') {
        "call_like"
    } else if parser_has_binary_operator(text) {
        "binary_expression"
    } else if text.contains('.') {
        "path_or_name"
    } else {
        "name_or_text"
    }
}

fn parser_has_binary_operator(text: &str) -> bool {
    [
        " == ", " != ", " <= ", " >= ", " < ", " > ", " + ", " - ", " * ", " / ", " and ", " or ",
    ]
    .iter()
    .any(|operator| text.contains(operator))
}

fn validate_canonical_expression(expression: &CanonicalExpression) -> Result<(), &'static str> {
    let mut identities = std::collections::BTreeSet::new();
    validate_canonical_node(expression, None, &mut identities)
}

pub(crate) fn validate_expression_occurrence(
    expression: &ParsedExpression,
) -> Result<(), &'static str> {
    validate_canonical_expression(&expression.canonical)?;
    if expression.occurrence.root_node_id != expression.canonical.node_id
        || expression.occurrence.nodes.is_empty()
        || expression.occurrence.nodes[0].node_id != expression.canonical.node_id
        || expression.occurrence.nodes[0].child_position != Vec::<usize>::new()
        || expression.occurrence.nodes[0].range != expression.canonical.range
        || expression.occurrence.nodes[0].lexical_status != expression.occurrence.lexical_status
    {
        return Err("parser_expression_occurrence_root_corrupt_v0");
    }
    validate_lexical_status(
        &expression.occurrence.lexical_status,
        &expression.canonical.range,
    )?;
    if let Some(intent_signal) = &expression.occurrence.intent_signal
        && (intent_signal.start.file != expression.canonical.range.start.file
            || intent_signal.start.line != expression.canonical.range.start.line
            || intent_signal.start.column < expression.canonical.range.start.column
            || intent_signal.start.column + intent_signal.byte_len
                > expression.canonical.range.start.column + expression.canonical.range.byte_len)
    {
        return Err("parser_expression_intent_signal_corrupt_v0");
    }
    let mut expected = Vec::new();
    collect_canonical_identity_projection(&expression.canonical, Vec::new(), &mut expected);
    let observed = expression
        .occurrence
        .nodes
        .iter()
        .map(|node| {
            (
                node.node_id.clone(),
                node.child_position.clone(),
                node.range.clone(),
            )
        })
        .collect::<Vec<_>>();
    if observed != expected {
        return Err("parser_expression_occurrence_projection_corrupt_v0");
    }
    for node in &expression.occurrence.nodes {
        validate_lexical_status(&node.lexical_status, &node.range)?;
    }
    let canonical_wrapper = match &expression.canonical.kind {
        CanonicalExpressionKind::Try {
            failure_root,
            failure_variant,
            ..
        } => Some((failure_root.as_ref(), failure_variant.as_ref())),
        _ => None,
    };
    let occurrence_wrapper = expression.occurrence.typed_failure_wrapper.as_ref();
    if canonical_wrapper.is_some() != occurrence_wrapper.is_some()
        || canonical_wrapper.is_some_and(|(root, variant)| {
            occurrence_wrapper.is_none_or(|wrapper| {
                root != wrapper.failure_root.as_ref() || variant != wrapper.failure_variant.as_ref()
            })
        })
    {
        return Err("parser_expression_occurrence_wrapper_corrupt_v0");
    }
    Ok(())
}

fn validate_lexical_status(
    status: &ParsedLexicalStatus,
    expression_range: &ParsedSourceRange,
) -> Result<(), &'static str> {
    let ParsedLexicalStatus::Malformed(fact) = status else {
        return Ok(());
    };
    if fact.offending.start.file != expression_range.start.file
        || fact.offending.start.line != expression_range.start.line
        || fact.offending.start.column < expression_range.start.column
        || fact.offending.start.column + fact.offending.byte_len
            > expression_range.start.column + expression_range.byte_len
    {
        return Err("parser_expression_lexical_range_corrupt_v0");
    }
    let exact = match fact.cause {
        ParsedMalformedExpressionCause::UnterminatedTextLiteral => matches!(
            (&fact.expected, &fact.actual),
            (
                ParsedExpectedLexicalEvidence::Token(ParsedLexicalTokenKind::TextQuote),
                ParsedActualLexicalEvidence::EndOfInput
            )
        ),
        ParsedMalformedExpressionCause::MissingDelimiter => matches!(
            (&fact.expected, &fact.actual),
            (
                ParsedExpectedLexicalEvidence::Token(
                    ParsedLexicalTokenKind::ParenthesisClose
                        | ParsedLexicalTokenKind::ListClose
                        | ParsedLexicalTokenKind::RecordClose
                ),
                ParsedActualLexicalEvidence::EndOfInput
            )
        ),
        ParsedMalformedExpressionCause::MismatchedDelimiter => matches!(
            (&fact.expected, &fact.actual),
            (
                ParsedExpectedLexicalEvidence::Token(
                    ParsedLexicalTokenKind::ParenthesisClose
                        | ParsedLexicalTokenKind::ListClose
                        | ParsedLexicalTokenKind::RecordClose
                ) | ParsedExpectedLexicalEvidence::Operand,
                ParsedActualLexicalEvidence::Token {
                    kind: ParsedLexicalTokenKind::ParenthesisClose
                        | ParsedLexicalTokenKind::ListClose
                        | ParsedLexicalTokenKind::RecordClose,
                    range,
                }
            ) if *range == fact.offending
        ),
        ParsedMalformedExpressionCause::DelimiterDepthExceeded => matches!(
            (&fact.expected, &fact.actual),
            (
                ParsedExpectedLexicalEvidence::MaximumDelimiterDepth(MAX_EXPRESSION_DELIMITER_DEPTH),
                ParsedActualLexicalEvidence::DelimiterDepth(actual)
            ) if *actual > MAX_EXPRESSION_DELIMITER_DEPTH
        ),
        ParsedMalformedExpressionCause::MissingOperand => {
            matches!(fact.expected, ParsedExpectedLexicalEvidence::Operand)
        }
        ParsedMalformedExpressionCause::InvalidComparisonOperator => matches!(
            (&fact.expected, &fact.actual),
            (
                ParsedExpectedLexicalEvidence::ComparisonOperator,
                ParsedActualLexicalEvidence::Token {
                    kind: ParsedLexicalTokenKind::ComparisonOperator,
                    range,
                }
            ) if *range == fact.offending
        ),
        ParsedMalformedExpressionCause::InvalidOperandStarter => {
            matches!(fact.expected, ParsedExpectedLexicalEvidence::Operand)
        }
        ParsedMalformedExpressionCause::MalformedFieldPlace => {
            matches!(fact.expected, ParsedExpectedLexicalEvidence::Identifier)
        }
        ParsedMalformedExpressionCause::ListElementSeparator => matches!(
            fact.expected,
            ParsedExpectedLexicalEvidence::ListSeparatorOrClose
        ),
        ParsedMalformedExpressionCause::ListTrailingComma
        | ParsedMalformedExpressionCause::ListNonTextElement => matches!(
            fact.expected,
            ParsedExpectedLexicalEvidence::TextListElement
        ),
        ParsedMalformedExpressionCause::IntegerLiteralOutOfRange => matches!(
            (&fact.expected, &fact.actual),
            (
                ParsedExpectedLexicalEvidence::Int64Value,
                ParsedActualLexicalEvidence::Token {
                    kind: ParsedLexicalTokenKind::IntegerLiteral,
                    range,
                }
            ) if *range == fact.offending
        ),
    };
    if exact {
        Ok(())
    } else {
        Err("parser_expression_lexical_evidence_corrupt_v0")
    }
}

fn collect_canonical_identity_projection(
    expression: &CanonicalExpression,
    child_position: Vec<usize>,
    output: &mut Vec<(ParserSyntaxNodeId, Vec<usize>, ParsedSourceRange)>,
) {
    output.push((
        expression.node_id.clone(),
        child_position.clone(),
        expression.range.clone(),
    ));
    let mut visit = |child: &CanonicalExpression, index: usize| {
        let mut position = child_position.clone();
        position.push(index);
        collect_canonical_identity_projection(child, position, output);
    };
    match &expression.kind {
        CanonicalExpressionKind::Field { base, .. } => visit(base, 0),
        CanonicalExpressionKind::ListLiteral(values) => {
            for (index, value) in values.iter().enumerate() {
                visit(value, index);
            }
        }
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            for (index, (_, value)) in fields.iter().enumerate() {
                visit(value, index);
            }
        }
        CanonicalExpressionKind::Call { callee, arguments } => {
            visit(callee, 0);
            for (index, argument) in arguments.iter().enumerate() {
                visit(argument, index + 1);
            }
        }
        CanonicalExpressionKind::Permission { value, .. }
        | CanonicalExpressionKind::Try { value, .. }
        | CanonicalExpressionKind::Group(value) => visit(value, 0),
        CanonicalExpressionKind::Binary { left, right, .. } => {
            visit(left, 0);
            visit(right, 1);
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

fn validate_canonical_node(
    expression: &CanonicalExpression,
    expected_id: Option<&ParserSyntaxNodeId>,
    identities: &mut std::collections::BTreeSet<String>,
) -> Result<(), &'static str> {
    if expected_id.is_some_and(|expected| expected != &expression.node_id)
        || expression.node_id.as_str().is_empty()
        || !identities.insert(expression.node_id.as_str().to_string())
        || (expression.range.byte_len == 0
            && !matches!(expression.kind, CanonicalExpressionKind::Unit))
    {
        return Err("canonical_expression_identity_or_range_corrupt_v0");
    }
    let validate_child =
        |child: &CanonicalExpression,
         role: &str,
         identities: &mut std::collections::BTreeSet<String>| {
            if child.range.start.file != expression.range.start.file
                || child.range.start.line != expression.range.start.line
                || child.range.start.column < expression.range.start.column
                || child.range.start.column + child.range.byte_len
                    > expression.range.start.column + expression.range.byte_len
            {
                return Err("canonical_expression_child_range_corrupt_v0");
            }
            validate_canonical_node(child, Some(&expression.node_id.child(role)), identities)
        };
    match &expression.kind {
        CanonicalExpressionKind::Field { base, .. } => {
            validate_child(base, "field-base", identities)?;
        }
        CanonicalExpressionKind::ListLiteral(values) => {
            for (index, value) in values.iter().enumerate() {
                validate_child(value, &format!("list-item-{index}"), identities)?;
            }
        }
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            for (index, (_, value)) in fields.iter().enumerate() {
                validate_child(value, &format!("record-field-{index}"), identities)?;
            }
        }
        CanonicalExpressionKind::Call { callee, arguments } => {
            validate_child(callee, "call-callee", identities)?;
            for (index, argument) in arguments.iter().enumerate() {
                validate_child(argument, &format!("call-argument-{index}"), identities)?;
            }
        }
        CanonicalExpressionKind::Permission { value, .. } => {
            validate_child(value, "permission-value", identities)?;
        }
        CanonicalExpressionKind::Try {
            value,
            failure_root,
            failure_variant,
        } => {
            validate_child(value, "try-value", identities)?;
            if failure_root.is_some() != failure_variant.is_some() {
                return Err("canonical_typed_failure_wrapper_corrupt_v0");
            }
        }
        CanonicalExpressionKind::Binary { left, right, .. } => {
            validate_child(left, "binary-left", identities)?;
            validate_child(right, "binary-right", identities)?;
            if left.range.start.column + left.range.byte_len > right.range.start.column {
                return Err("canonical_expression_child_order_corrupt_v0");
            }
        }
        CanonicalExpressionKind::Group(value) => {
            validate_child(value, "group-value", identities)?;
        }
        CanonicalExpressionKind::Unit
        | CanonicalExpressionKind::Identifier(_)
        | CanonicalExpressionKind::UIntLiteral(_)
        | CanonicalExpressionKind::IntLiteral(_)
        | CanonicalExpressionKind::BoolLiteral(_)
        | CanonicalExpressionKind::TextLiteral(_)
        | CanonicalExpressionKind::Unsupported => {}
    }
    Ok(())
}

fn parse_other_statement_expressions(
    text: &str,
    span: &Span,
    source_node_id: &ParserSyntaxNodeId,
) -> Vec<ParsedExpression> {
    if let Some(loop_header) = parsed_loop_header_parts(text) {
        return loop_header
            .expressions
            .into_iter()
            .enumerate()
            .map(|(index, (expression, offset, intent))| {
                parse_expression_syntax(
                    expression,
                    offset_span(span, offset),
                    source_node_id.child(&format!("expression-{index}")),
                    intent,
                )
            })
            .collect();
    }
    let (candidate, intent) = if let Some(rest) = keyword_rest(text, "set") {
        find_top_level_char(rest, '=')
            .map(|index| (&rest[index + 1..], text.len() - rest.len() + index + 1))
            .map_or((None, ParsedExpressionIntent::Other), |candidate| {
                (Some(candidate), ParsedExpressionIntent::SetValue)
            })
    } else if let Some(rest) = keyword_rest(text, "save") {
        let value = rest.split_once(" in ").map_or(rest, |(value, _)| value);
        (
            Some((value, text.len() - rest.len())),
            ParsedExpressionIntent::SaveValue,
        )
    } else if let Some(rest) = keyword_rest(text, "expect") {
        (
            Some((rest, text.len() - rest.len())),
            ParsedExpressionIntent::TestExpectation,
        )
    } else if let Some(rest) = keyword_rest(text, "fail") {
        (
            Some((rest, text.len() - rest.len())),
            ParsedExpressionIntent::Failure,
        )
    } else if let Some(rest) = keyword_rest(text, "if") {
        (
            Some((
                rest.trim_end_matches('{').trim_end(),
                text.len() - rest.len(),
            )),
            ParsedExpressionIntent::Condition,
        )
    } else if let Some(rest) = keyword_rest(text, "while") {
        (
            Some((
                rest.trim_end_matches('{').trim_end(),
                text.len() - rest.len(),
            )),
            ParsedExpressionIntent::Condition,
        )
    } else if text != "}" && !text.ends_with(':') {
        (Some((text, 0)), ParsedExpressionIntent::Other)
    } else {
        (None, ParsedExpressionIntent::Other)
    };
    candidate
        .filter(|(expression, _)| !expression.trim().is_empty())
        .map(|(expression, offset)| {
            vec![parse_expression_syntax(
                expression,
                offset_span(span, offset),
                source_node_id.child("expression-0"),
                intent,
            )]
        })
        .unwrap_or_default()
}

fn parse_expression_syntax(
    text: &str,
    span: Span,
    source_node_id: ParserSyntaxNodeId,
    intent: ParsedExpressionIntent,
) -> ParsedExpression {
    let leading = text.len() - text.trim_start().len();
    let text = text.trim();
    let span = offset_span(&span, leading);
    let (canonical, occurrence) =
        parse_canonical_expression_occurrence(text, &span, source_node_id.clone(), intent);
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
                    value: Box::new(parse_expression_syntax(
                        rest,
                        offset_span(&span, offset),
                        source_node_id.child("permission-value"),
                        intent,
                    )),
                },
                canonical: canonical.clone(),
                occurrence: occurrence.clone(),
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
            canonical: canonical.clone(),
            occurrence: occurrence.clone(),
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
            canonical: canonical.clone(),
            occurrence: occurrence.clone(),
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
            .enumerate()
            .map(|(index, range)| {
                parse_expression_syntax(
                    &text[range.clone()],
                    offset_span(&span, range.start),
                    source_node_id.child(&format!("compound-{index}")),
                    intent,
                )
            })
            .collect();
        return ParsedExpression {
            kind: ParsedExpressionKind::Compound { operands },
            canonical: canonical.clone(),
            occurrence: occurrence.clone(),
            span,
        };
    }

    if let Some(open) = text.find('(') {
        let callee_text = text[..open].trim();
        let callee_offset = text[..open].find(callee_text).unwrap_or_default();
        let callee = parse_expression_syntax(
            callee_text,
            offset_span(&span, callee_offset),
            source_node_id.child("call-callee"),
            intent,
        );
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
            .enumerate()
            .filter_map(|(index, range)| {
                let raw = &inside[range.clone()];
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return None;
                }
                let leading = raw.len() - raw.trim_start().len();
                Some(parse_expression_syntax(
                    trimmed,
                    offset_span(&span, open + 1 + range.start + leading),
                    source_node_id.child(&format!("call-argument-{index}")),
                    intent,
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
            canonical: canonical.clone(),
            occurrence: occurrence.clone(),
            span,
        };
    }

    if let Some(open) = text.find('[')
        && text[open + 1..].contains(')')
    {
        let callee_text = text[..open].trim();
        let callee = parse_expression_syntax(
            callee_text,
            span.clone(),
            source_node_id.child("call-callee"),
            intent,
        );
        return ParsedExpression {
            kind: ParsedExpressionKind::Call(ParsedCall {
                callee: Box::new(callee),
                arguments: Vec::new(),
                argument_separators_hws_valid: true,
                close_status: ParsedCallCloseStatus::Mismatched,
                trailing_status: ParsedCallTrailingStatus::Complete,
            }),
            canonical: canonical.clone(),
            occurrence: occurrence.clone(),
            span,
        };
    }

    let operands = compound_identifier_operands(text, &span, &source_node_id);
    if !operands.is_empty() {
        return ParsedExpression {
            kind: ParsedExpressionKind::Compound { operands },
            canonical: canonical.clone(),
            occurrence: occurrence.clone(),
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
        canonical,
        occurrence,
        span,
    }
}

fn parse_canonical_expression_occurrence(
    text: &str,
    span: &Span,
    node_id: ParserSyntaxNodeId,
    intent: ParsedExpressionIntent,
) -> (CanonicalExpression, ParsedExpressionOccurrenceFacts) {
    let canonical = parse_canonical_expression(text, span, node_id.clone());
    let (lexical_status, maximum_delimiter_depth) =
        expression_lexical_status(text, span, intent, &canonical);
    let mut nodes = Vec::new();
    collect_canonical_node_syntax(&canonical, text, span, intent, Vec::new(), 0, &mut nodes);
    let occurrence = ParsedExpressionOccurrenceFacts {
        root_node_id: node_id,
        intent,
        intent_signal: parser_expression_intent_signal(text, span),
        maximum_delimiter_depth,
        lexical_status,
        nodes,
        typed_failure_wrapper: parse_typed_failure_wrapper_syntax(text, span),
    };
    (canonical, occurrence)
}

#[derive(Debug, Clone)]
struct ChainedComparisonSites {
    node_id: ParserSyntaxNodeId,
    first: ParsedSourceRange,
    later: ParsedSourceRange,
}

fn chained_comparison_sites(expression: &ParsedExpression) -> Vec<ChainedComparisonSites> {
    let mut output = Vec::new();
    collect_chained_comparisons(
        &expression.canonical,
        &expression.occurrence.nodes,
        &mut output,
    );
    output
}

fn collect_chained_comparisons(
    expression: &CanonicalExpression,
    facts: &[ParsedCanonicalNodeSyntax],
    output: &mut Vec<ChainedComparisonSites>,
) {
    if is_comparison_node(expression) {
        let mut ranges = Vec::new();
        collect_connected_comparison_ranges(expression, facts, &mut ranges);
        if ranges.len() > 1 {
            ranges.sort_by_key(|range| {
                (
                    range.start.file.clone(),
                    range.start.line,
                    range.start.column,
                )
            });
            output.push(ChainedComparisonSites {
                node_id: expression.node_id.clone(),
                first: ranges[0].clone(),
                later: ranges[ranges.len() - 1].clone(),
            });
            if let CanonicalExpressionKind::Binary { left, right, .. } = &expression.kind {
                if !is_connected_comparison_operand(left) {
                    collect_chained_comparisons(left, facts, output);
                }
                if !is_connected_comparison_operand(right) {
                    collect_chained_comparisons(right, facts, output);
                }
            }
            return;
        }
    }
    visit_canonical_children(expression, |child| {
        collect_chained_comparisons(child, facts, output)
    });
}

fn collect_connected_comparison_ranges(
    expression: &CanonicalExpression,
    facts: &[ParsedCanonicalNodeSyntax],
    ranges: &mut Vec<ParsedSourceRange>,
) {
    let expression = unwrap_groups(expression);
    if !is_comparison_node(expression) {
        return;
    }
    if let Some(operator) = facts
        .iter()
        .find(|fact| fact.node_id == expression.node_id)
        .and_then(|fact| fact.operator.clone())
    {
        ranges.push(operator);
    }
    if let CanonicalExpressionKind::Binary { left, right, .. } = &expression.kind {
        collect_connected_comparison_ranges(left, facts, ranges);
        collect_connected_comparison_ranges(right, facts, ranges);
    }
}

fn unwrap_groups(mut expression: &CanonicalExpression) -> &CanonicalExpression {
    while let CanonicalExpressionKind::Group(value) = &expression.kind {
        expression = value;
    }
    expression
}

fn is_connected_comparison_operand(expression: &CanonicalExpression) -> bool {
    is_comparison_node(unwrap_groups(expression))
}

fn is_comparison_node(expression: &CanonicalExpression) -> bool {
    matches!(
        &expression.kind,
        CanonicalExpressionKind::Binary {
            operator: ParsedBinaryOperator::Equal
                | ParsedBinaryOperator::NotEqual
                | ParsedBinaryOperator::Less
                | ParsedBinaryOperator::LessEqual
                | ParsedBinaryOperator::Greater
                | ParsedBinaryOperator::GreaterEqual,
            left,
            right,
        }
        if is_present_comparison_operand(left) && is_present_comparison_operand(right)
    )
}

fn is_present_comparison_operand(expression: &CanonicalExpression) -> bool {
    !matches!(
        unwrap_groups(expression).kind,
        CanonicalExpressionKind::Unit | CanonicalExpressionKind::Unsupported
    )
}

fn visit_canonical_children(
    expression: &CanonicalExpression,
    mut visit: impl FnMut(&CanonicalExpression),
) {
    match &expression.kind {
        CanonicalExpressionKind::Field { base, .. } => visit(base),
        CanonicalExpressionKind::ListLiteral(values) => {
            for value in values {
                visit(value);
            }
        }
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            for (_, value) in fields {
                visit(value);
            }
        }
        CanonicalExpressionKind::Call { callee, arguments } => {
            visit(callee);
            for argument in arguments {
                visit(argument);
            }
        }
        CanonicalExpressionKind::Permission { value, .. }
        | CanonicalExpressionKind::Try { value, .. }
        | CanonicalExpressionKind::Group(value) => visit(value),
        CanonicalExpressionKind::Binary { left, right, .. } => {
            visit(left);
            visit(right);
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

fn collect_canonical_node_syntax(
    expression: &CanonicalExpression,
    root_text: &str,
    root_span: &Span,
    intent: ParsedExpressionIntent,
    child_position: Vec<usize>,
    delimiter_depth: usize,
    nodes: &mut Vec<ParsedCanonicalNodeSyntax>,
) {
    let node_text = source_text_for_range(root_text, root_span, &expression.range);
    let (lexical_status, _) =
        expression_lexical_status(node_text, &expression.range.start, intent, expression);
    let operator = matches!(expression.kind, CanonicalExpressionKind::Binary { .. })
        .then(|| {
            top_level_binary_operator(node_text).map(|(_, start, end)| {
                source_range(&expression.range.start, start, end.saturating_sub(start))
            })
        })
        .flatten();
    let delimiter = delimiter_syntax(expression, node_text);
    let call = matches!(expression.kind, CanonicalExpressionKind::Call { .. })
        .then(|| call_syntax_facts(node_text, &expression.range.start))
        .flatten();
    nodes.push(ParsedCanonicalNodeSyntax {
        node_id: expression.node_id.clone(),
        child_position: child_position.clone(),
        range: expression.range.clone(),
        operator,
        delimiter,
        call,
        delimiter_depth,
        lexical_status,
    });

    let mut visit = |child: &CanonicalExpression, index: usize, depth: usize| {
        let mut position = child_position.clone();
        position.push(index);
        collect_canonical_node_syntax(child, root_text, root_span, intent, position, depth, nodes);
    };
    match &expression.kind {
        CanonicalExpressionKind::Field { base, .. } => visit(base, 0, delimiter_depth),
        CanonicalExpressionKind::ListLiteral(values) => {
            for (index, value) in values.iter().enumerate() {
                visit(value, index, delimiter_depth + 1);
            }
        }
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            for (index, (_, value)) in fields.iter().enumerate() {
                visit(value, index, delimiter_depth + 1);
            }
        }
        CanonicalExpressionKind::Call { callee, arguments } => {
            visit(callee, 0, delimiter_depth);
            for (index, argument) in arguments.iter().enumerate() {
                visit(argument, index + 1, delimiter_depth + 1);
            }
        }
        CanonicalExpressionKind::Permission { value, .. } => visit(value, 0, delimiter_depth),
        CanonicalExpressionKind::Try { value, .. } => visit(value, 0, delimiter_depth),
        CanonicalExpressionKind::Binary { left, right, .. } => {
            visit(left, 0, delimiter_depth);
            visit(right, 1, delimiter_depth);
        }
        CanonicalExpressionKind::Group(value) => visit(value, 0, delimiter_depth + 1),
        CanonicalExpressionKind::Unit
        | CanonicalExpressionKind::Identifier(_)
        | CanonicalExpressionKind::UIntLiteral(_)
        | CanonicalExpressionKind::IntLiteral(_)
        | CanonicalExpressionKind::BoolLiteral(_)
        | CanonicalExpressionKind::TextLiteral(_)
        | CanonicalExpressionKind::Unsupported => {}
    }
}

fn source_text_for_range<'a>(
    root_text: &'a str,
    root_span: &Span,
    range: &ParsedSourceRange,
) -> &'a str {
    if range.start.file != root_span.file || range.start.line != root_span.line {
        return "";
    }
    let start = range.start.column.saturating_sub(root_span.column);
    let end = start.saturating_add(range.byte_len);
    root_text.get(start..end).unwrap_or("")
}

fn source_range(span: &Span, offset: usize, byte_len: usize) -> ParsedSourceRange {
    ParsedSourceRange {
        start: offset_span(span, offset),
        byte_len,
    }
}

fn delimiter_syntax(expression: &CanonicalExpression, text: &str) -> Option<ParsedDelimiterSyntax> {
    let (kind, open, close) = match &expression.kind {
        CanonicalExpressionKind::Group(_) => (
            ParsedDelimiterKind::Parenthesis,
            text.find('(')?,
            matching_delimiter_quoted(text, text.find('(')?, '(', ')'),
        ),
        CanonicalExpressionKind::ListLiteral(_) => (
            ParsedDelimiterKind::List,
            text.find('[')?,
            matching_delimiter_quoted(text, text.find('[')?, '[', ']'),
        ),
        CanonicalExpressionKind::RecordLiteral { .. } => (
            ParsedDelimiterKind::Record,
            text.find('{')?,
            matching_delimiter_quoted(text, text.find('{')?, '{', '}'),
        ),
        CanonicalExpressionKind::Call { .. } => {
            let open = find_top_level_open_paren(text)?;
            (
                ParsedDelimiterKind::Parenthesis,
                open,
                matching_delimiter_quoted(text, open, '(', ')'),
            )
        }
        _ => return None,
    };
    Some(ParsedDelimiterSyntax {
        kind,
        open: source_range(&expression.range.start, open, 1),
        close: close.map(|close| source_range(&expression.range.start, close, 1)),
    })
}

fn call_syntax_facts(text: &str, span: &Span) -> Option<ParsedCallSyntaxFacts> {
    let open = find_top_level_open_paren(text)?;
    let close = matching_delimiter_quoted(text, open, '(', ')');
    let inside_end = close.unwrap_or(text.len());
    let separators = top_level_separator_offsets(&text[open + 1..inside_end], ',')
        .into_iter()
        .map(|offset| source_range(span, open + 1 + offset, 1))
        .collect();
    let trailing = close.and_then(|close| {
        (close + 1 < text.len()).then(|| source_range(span, close + 1, text.len() - close - 1))
    });
    let gaps = whitespace_ranges(text)
        .into_iter()
        .map(|range| source_range(span, range.start, range.end - range.start))
        .collect();
    Some(ParsedCallSyntaxFacts {
        open: source_range(span, open, 1),
        close: close.map(|close| source_range(span, close, 1)),
        separators,
        trailing,
        gaps,
    })
}

fn top_level_separator_offsets(text: &str, delimiter: char) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if ch == delimiter && depth == 0 => offsets.push(index),
            _ => {}
        }
    }
    offsets
}

fn whitespace_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = None;
    for (index, ch) in text.char_indices() {
        if ch.is_ascii_whitespace() {
            start.get_or_insert(index);
        } else if let Some(start) = start.take() {
            ranges.push(start..index);
        }
    }
    if let Some(start) = start {
        ranges.push(start..text.len());
    }
    ranges
}

const MAX_EXPRESSION_DELIMITER_DEPTH: usize = 16;

fn expression_lexical_status(
    text: &str,
    span: &Span,
    intent: ParsedExpressionIntent,
    canonical: &CanonicalExpression,
) -> (ParsedLexicalStatus, usize) {
    let (delimiter_status, maximum) = delimiter_lexical_status(text, span);
    if delimiter_status != ParsedLexicalStatus::Complete {
        return (delimiter_status, maximum);
    }
    if matches!(
        intent,
        ParsedExpressionIntent::NeedsPredicate | ParsedExpressionIntent::EnsuresPredicate
    ) && parser_expression_intent_signal(text, span).is_some()
        && let Some(issue) = predicate_lexical_issue(text, span, canonical)
    {
        return (ParsedLexicalStatus::Malformed(issue), maximum);
    }
    (ParsedLexicalStatus::Complete, maximum)
}

fn delimiter_lexical_status(text: &str, span: &Span) -> (ParsedLexicalStatus, usize) {
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut quoted = false;
    let mut escaped = false;
    let mut quote_start = 0usize;
    let mut maximum = 0usize;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => {
                quoted = true;
                quote_start = index;
            }
            '(' | '[' | '{' => {
                stack.push((ch, index));
                maximum = maximum.max(stack.len());
                if maximum > MAX_EXPRESSION_DELIMITER_DEPTH {
                    return (
                        ParsedLexicalStatus::Malformed(ParsedMalformedExpressionFact {
                            cause: ParsedMalformedExpressionCause::DelimiterDepthExceeded,
                            offending: source_range(span, index, ch.len_utf8()),
                            expected: ParsedExpectedLexicalEvidence::MaximumDelimiterDepth(
                                MAX_EXPRESSION_DELIMITER_DEPTH,
                            ),
                            actual: ParsedActualLexicalEvidence::DelimiterDepth(maximum),
                        }),
                        maximum,
                    );
                }
            }
            ')' | ']' | '}' => {
                let expected_open = opening_delimiter_for_close(ch);
                if stack.last().map(|(open, _)| *open) == Some(expected_open) {
                    stack.pop();
                } else {
                    let expected =
                        stack
                            .last()
                            .map_or(ParsedExpectedLexicalEvidence::Operand, |(open, _)| {
                                ParsedExpectedLexicalEvidence::Token(closing_token_for_open(*open))
                            });
                    let offending = source_range(span, index, ch.len_utf8());
                    return (
                        ParsedLexicalStatus::Malformed(ParsedMalformedExpressionFact {
                            cause: ParsedMalformedExpressionCause::MismatchedDelimiter,
                            offending: offending.clone(),
                            expected,
                            actual: ParsedActualLexicalEvidence::Token {
                                kind: lexical_token_kind(ch),
                                range: offending,
                            },
                        }),
                        maximum,
                    );
                }
            }
            _ => {}
        }
    }
    if quoted {
        (
            ParsedLexicalStatus::Malformed(ParsedMalformedExpressionFact {
                cause: ParsedMalformedExpressionCause::UnterminatedTextLiteral,
                offending: source_range(span, quote_start, text.len() - quote_start),
                expected: ParsedExpectedLexicalEvidence::Token(ParsedLexicalTokenKind::TextQuote),
                actual: ParsedActualLexicalEvidence::EndOfInput,
            }),
            maximum,
        )
    } else if let Some((open, _)) = stack.last() {
        (
            ParsedLexicalStatus::Malformed(ParsedMalformedExpressionFact {
                cause: ParsedMalformedExpressionCause::MissingDelimiter,
                offending: source_range(span, text.len(), 0),
                expected: ParsedExpectedLexicalEvidence::Token(closing_token_for_open(*open)),
                actual: ParsedActualLexicalEvidence::EndOfInput,
            }),
            maximum,
        )
    } else {
        (ParsedLexicalStatus::Complete, maximum)
    }
}

fn opening_delimiter_for_close(close: char) -> char {
    match close {
        ')' => '(',
        ']' => '[',
        '}' => '{',
        _ => unreachable!(),
    }
}

fn closing_token_for_open(open: char) -> ParsedLexicalTokenKind {
    match open {
        '(' => ParsedLexicalTokenKind::ParenthesisClose,
        '[' => ParsedLexicalTokenKind::ListClose,
        '{' => ParsedLexicalTokenKind::RecordClose,
        _ => unreachable!(),
    }
}

fn lexical_token_kind(ch: char) -> ParsedLexicalTokenKind {
    match ch {
        '"' => ParsedLexicalTokenKind::TextQuote,
        '(' => ParsedLexicalTokenKind::ParenthesisOpen,
        ')' => ParsedLexicalTokenKind::ParenthesisClose,
        '[' => ParsedLexicalTokenKind::ListOpen,
        ']' => ParsedLexicalTokenKind::ListClose,
        '{' => ParsedLexicalTokenKind::RecordOpen,
        '}' => ParsedLexicalTokenKind::RecordClose,
        ',' => ParsedLexicalTokenKind::Comma,
        '.' => ParsedLexicalTokenKind::Dot,
        '<' | '>' | '=' | '!' => ParsedLexicalTokenKind::ComparisonOperator,
        value if value.is_ascii_digit() => ParsedLexicalTokenKind::IntegerLiteral,
        value if value.is_ascii_alphabetic() || value == '_' => ParsedLexicalTokenKind::Identifier,
        _ => ParsedLexicalTokenKind::Other,
    }
}

fn actual_lexical_evidence(text: &str, span: &Span, offset: usize) -> ParsedActualLexicalEvidence {
    text.get(offset..)
        .and_then(|rest| rest.chars().next())
        .map_or(ParsedActualLexicalEvidence::EndOfInput, |ch| {
            let range = source_range(span, offset, ch.len_utf8());
            ParsedActualLexicalEvidence::Token {
                kind: lexical_token_kind(ch),
                range,
            }
        })
}

fn parser_expression_intent_signal(text: &str, span: &Span) -> Option<ParsedSourceRange> {
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        if ch == '"' {
            quoted = true;
            continue;
        }
        if matches!(ch, '=' | '<' | '>') {
            return Some(source_range(span, index, ch.len_utf8()));
        }
        if ch == '!' {
            let next = text[index + ch.len_utf8()..]
                .char_indices()
                .find(|(_, next)| !matches!(next, ' ' | '\t'))
                .map(|(offset, next)| (index + ch.len_utf8() + offset, next));
            let left = text[..index].trim_end();
            if next.is_some_and(|(_, next)| next == '=')
                || (!left.is_empty()
                    && next.is_some_and(|(_, next)| {
                        matches!(next, '"' | '[' | '(' | '-' | '0'..='9' | 'a'..='z' | '_')
                    }))
            {
                return Some(source_range(span, index, ch.len_utf8()));
            }
        }
    }
    for name in ["old", "list_len", "list_count"] {
        let mut start = 0usize;
        while let Some(relative) = text[start..].find(name) {
            let index = start + relative;
            let before = index == 0
                || !text.as_bytes()[index - 1].is_ascii_alphanumeric()
                    && text.as_bytes()[index - 1] != b'_';
            let after_name = index + name.len();
            let after = after_name == text.len()
                || !text.as_bytes()[after_name].is_ascii_alphanumeric()
                    && text.as_bytes()[after_name] != b'_';
            if before && after {
                let open = text[after_name..]
                    .char_indices()
                    .find(|(_, ch)| !matches!(ch, ' ' | '\t'))
                    .map(|(offset, ch)| (after_name + offset, ch));
                if let Some((open, '(')) = open {
                    return Some(source_range(span, index, open + 1 - index));
                }
            }
            start = after_name;
        }
    }
    None
}

fn predicate_lexical_issue(
    text: &str,
    span: &Span,
    canonical: &CanonicalExpression,
) -> Option<ParsedMalformedExpressionFact> {
    let trimmed = text.trim();
    let leading = text.len() - text.trim_start().len();
    if trimmed.is_empty() {
        return Some(ParsedMalformedExpressionFact {
            cause: ParsedMalformedExpressionCause::MissingOperand,
            offending: source_range(span, text.len(), 0),
            expected: ParsedExpectedLexicalEvidence::Operand,
            actual: ParsedActualLexicalEvidence::EndOfInput,
        });
    }
    if let Some((offset, len)) = out_of_range_integer_token(trimmed) {
        let offending = source_range(span, leading + offset, len);
        return Some(ParsedMalformedExpressionFact {
            cause: ParsedMalformedExpressionCause::IntegerLiteralOutOfRange,
            offending: offending.clone(),
            expected: ParsedExpectedLexicalEvidence::Int64Value,
            actual: ParsedActualLexicalEvidence::Token {
                kind: ParsedLexicalTokenKind::IntegerLiteral,
                range: offending,
            },
        });
    }
    if let Some(issue) = first_predicate_list_lexical_issue(text, span) {
        return Some(issue);
    }
    if let Some(issue) = predicate_field_lexical_issue(trimmed, span, leading) {
        return Some(issue);
    }
    let first = trimmed.chars().next()?;
    if !matches!(first, '"' | '[' | '(' | '-' | '0'..='9' | 'a'..='z' | '_') {
        let offending = source_range(span, leading, first.len_utf8());
        return Some(ParsedMalformedExpressionFact {
            cause: ParsedMalformedExpressionCause::InvalidOperandStarter,
            offending: offending.clone(),
            expected: ParsedExpectedLexicalEvidence::Operand,
            actual: ParsedActualLexicalEvidence::Token {
                kind: lexical_token_kind(first),
                range: offending,
            },
        });
    }
    if let Some(issue) = missing_operand_issue(canonical, span, text) {
        return Some(issue);
    }
    if !canonical_contains_comparison(canonical)
        && let Some((offset, len)) = first_comparison_spelling(text)
    {
        let offending = source_range(span, offset, len);
        return Some(ParsedMalformedExpressionFact {
            cause: ParsedMalformedExpressionCause::InvalidComparisonOperator,
            offending: offending.clone(),
            expected: ParsedExpectedLexicalEvidence::ComparisonOperator,
            actual: ParsedActualLexicalEvidence::Token {
                kind: ParsedLexicalTokenKind::ComparisonOperator,
                range: offending,
            },
        });
    }
    None
}

fn out_of_range_integer_token(text: &str) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    let mut quoted = false;
    while index < bytes.len() {
        if bytes[index] == b'"' {
            quoted = !quoted;
            index += 1;
            continue;
        }
        if quoted {
            index += 1;
            continue;
        }
        let signed = bytes[index] == b'-'
            && bytes
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_digit());
        if bytes[index].is_ascii_digit() || signed {
            let start = index;
            if signed {
                index += 1;
            }
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
            let token = &text[start..index];
            if token.parse::<i64>().is_err() {
                return Some((start, index - start));
            }
            continue;
        }
        index += 1;
    }
    None
}

fn canonical_contains_comparison(expression: &CanonicalExpression) -> bool {
    is_comparison_node(expression)
        || match &expression.kind {
            CanonicalExpressionKind::Field { base, .. }
            | CanonicalExpressionKind::Permission { value: base, .. }
            | CanonicalExpressionKind::Try { value: base, .. }
            | CanonicalExpressionKind::Group(base) => canonical_contains_comparison(base),
            CanonicalExpressionKind::ListLiteral(values) => {
                values.iter().any(canonical_contains_comparison)
            }
            CanonicalExpressionKind::RecordLiteral { fields, .. } => fields
                .iter()
                .any(|(_, value)| canonical_contains_comparison(value)),
            CanonicalExpressionKind::Call { callee, arguments } => {
                canonical_contains_comparison(callee)
                    || arguments.iter().any(canonical_contains_comparison)
            }
            CanonicalExpressionKind::Binary { left, right, .. } => {
                canonical_contains_comparison(left) || canonical_contains_comparison(right)
            }
            CanonicalExpressionKind::Unit
            | CanonicalExpressionKind::Identifier(_)
            | CanonicalExpressionKind::UIntLiteral(_)
            | CanonicalExpressionKind::IntLiteral(_)
            | CanonicalExpressionKind::BoolLiteral(_)
            | CanonicalExpressionKind::TextLiteral(_)
            | CanonicalExpressionKind::Unsupported => false,
        }
}

fn first_comparison_spelling(text: &str) -> Option<(usize, usize)> {
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        if ch == '"' {
            quoted = true;
            continue;
        }
        if matches!(ch, '=' | '<' | '>' | '!') {
            let len = text[index..]
                .chars()
                .take_while(|value| matches!(value, '=' | '<' | '>' | '!'))
                .map(char::len_utf8)
                .sum::<usize>();
            return Some((index, len.max(ch.len_utf8())));
        }
    }
    None
}

fn missing_operand_issue(
    expression: &CanonicalExpression,
    root_span: &Span,
    root_text: &str,
) -> Option<ParsedMalformedExpressionFact> {
    if let CanonicalExpressionKind::Binary { left, right, .. } = &expression.kind {
        for child in [left.as_ref(), right.as_ref()] {
            if matches!(
                child.kind,
                CanonicalExpressionKind::Unit | CanonicalExpressionKind::Unsupported
            ) {
                let relative = child.range.start.column.saturating_sub(root_span.column);
                let offending = child.range.clone();
                return Some(ParsedMalformedExpressionFact {
                    cause: ParsedMalformedExpressionCause::MissingOperand,
                    expected: ParsedExpectedLexicalEvidence::Operand,
                    actual: actual_lexical_evidence(root_text, root_span, relative),
                    offending,
                });
            }
        }
        return missing_operand_issue(left, root_span, root_text)
            .or_else(|| missing_operand_issue(right, root_span, root_text));
    }
    None
}

fn predicate_field_lexical_issue(
    text: &str,
    span: &Span,
    leading: usize,
) -> Option<ParsedMalformedExpressionFact> {
    let dot_offsets = text
        .char_indices()
        .filter_map(|(index, ch)| (ch == '.').then_some(index))
        .collect::<Vec<_>>();
    let invalid = dot_offsets.iter().copied().find(|index| {
        let next = text[*index + 1..].chars().next();
        next.is_none_or(|ch| !(ch.is_ascii_lowercase() || ch == '_'))
    });
    let offending_offset = invalid.or_else(|| dot_offsets.get(1).copied())?;
    let offending = source_range(span, leading + offending_offset, 1);
    Some(ParsedMalformedExpressionFact {
        cause: ParsedMalformedExpressionCause::MalformedFieldPlace,
        offending: offending.clone(),
        expected: ParsedExpectedLexicalEvidence::Identifier,
        actual: actual_lexical_evidence(text, span, offending_offset),
    })
}

fn predicate_list_lexical_issue(
    text: &str,
    span: &Span,
    leading: usize,
) -> Option<ParsedMalformedExpressionFact> {
    if !(text.starts_with('[') && text.ends_with(']')) {
        return None;
    }
    let inside = &text[1..text.len() - 1];
    let ranges = split_top_level_ranges(inside, ',');
    if inside.trim_end().ends_with(',') {
        let offset = inside.rfind(',')? + 1;
        let offending = source_range(span, leading + offset, 1);
        return Some(ParsedMalformedExpressionFact {
            cause: ParsedMalformedExpressionCause::ListTrailingComma,
            offending: offending.clone(),
            expected: ParsedExpectedLexicalEvidence::TextListElement,
            actual: ParsedActualLexicalEvidence::EndOfInput,
        });
    }
    for range in ranges {
        let raw = &inside[range.clone()];
        let value = raw.trim();
        let value_leading = raw.len() - raw.trim_start().len();
        let offset = leading + 1 + range.start + value_leading;
        if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2) {
            let offending =
                source_range(span, offset, value.chars().next().map_or(0, char::len_utf8));
            return Some(ParsedMalformedExpressionFact {
                cause: ParsedMalformedExpressionCause::ListNonTextElement,
                offending: offending.clone(),
                expected: ParsedExpectedLexicalEvidence::TextListElement,
                actual: actual_lexical_evidence(text, span, 1 + range.start + value_leading),
            });
        }
        if let Some(close) = value[1..].find('"').map(|close| close + 1) {
            let trailing = value[close + 1..].trim_start();
            if !trailing.is_empty() {
                let trailing_offset = value.len() - trailing.len();
                let offending = source_range(span, offset + trailing_offset, 1);
                return Some(ParsedMalformedExpressionFact {
                    cause: ParsedMalformedExpressionCause::ListElementSeparator,
                    offending: offending.clone(),
                    expected: ParsedExpectedLexicalEvidence::ListSeparatorOrClose,
                    actual: actual_lexical_evidence(
                        value,
                        &offset_span(span, offset),
                        trailing_offset,
                    ),
                });
            }
        }
    }
    None
}

fn first_predicate_list_lexical_issue(
    text: &str,
    span: &Span,
) -> Option<ParsedMalformedExpressionFact> {
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        if ch == '"' {
            quoted = true;
            continue;
        }
        if ch == '['
            && let Some(close) = matching_delimiter_quoted(text, index, '[', ']')
        {
            let list = &text[index..=close];
            if let Some(issue) = predicate_list_lexical_issue(list, &offset_span(span, index), 0) {
                return Some(issue);
            }
        }
    }
    None
}

fn parse_typed_failure_wrapper_syntax(
    text: &str,
    span: &Span,
) -> Option<ParsedTypedFailureWrapperSyntax> {
    let rest = keyword_rest(text, "try")?;
    let rest_offset = text.len() - rest.len();
    let (_, wrapper) = split_typed_failure_wrapper(rest, span, rest_offset);
    Some(ParsedTypedFailureWrapperSyntax {
        kind: if wrapper.is_some() {
            ParsedTypedFailureWrapperKind::Wrap
        } else {
            ParsedTypedFailureWrapperKind::Try
        },
        try_keyword: source_range(span, 0, "try".len()),
        failure_root: wrapper
            .as_ref()
            .and_then(|wrapper| wrapper.failure_root.clone()),
        failure_variant: wrapper
            .as_ref()
            .and_then(|wrapper| wrapper.failure_variant.clone()),
    })
}

fn split_typed_failure_wrapper<'a>(
    text: &'a str,
    span: &Span,
    text_offset: usize,
) -> (&'a str, Option<ParsedTypedFailureWrapperSyntax>) {
    let Some(wrapper_start) = find_top_level_phrase(text, " or fail ") else {
        return (text, None);
    };
    let value = text[..wrapper_start].trim_end();
    let failure_start = wrapper_start + " or fail ".len();
    let failure = text[failure_start..].trim();
    let leading = text[failure_start..].len() - text[failure_start..].trim_start().len();
    let absolute = text_offset + failure_start + leading;
    let (root, variant) = failure.split_once('.').unwrap_or((failure, ""));
    let root = is_type_identifier(root).then(|| ParsedIdentifier {
        name: root.to_string(),
        span: offset_span(span, absolute),
    });
    let variant_offset = failure.find('.').map_or(failure.len(), |dot| dot + 1);
    let variant = is_value_identifier(variant).then(|| ParsedIdentifier {
        name: variant.to_string(),
        span: offset_span(span, absolute + variant_offset),
    });
    (
        value,
        Some(ParsedTypedFailureWrapperSyntax {
            kind: ParsedTypedFailureWrapperKind::Wrap,
            try_keyword: source_range(span, 0, "try".len()),
            failure_root: root,
            failure_variant: variant,
        }),
    )
}

fn find_top_level_phrase(text: &str, phrase: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 && text[index..].starts_with(phrase) => return Some(index),
            _ => {}
        }
    }
    None
}

fn parse_canonical_expression(
    text: &str,
    span: &Span,
    node_id: ParserSyntaxNodeId,
) -> CanonicalExpression {
    let leading = text.len() - text.trim_start().len();
    let text = text.trim();
    let span = offset_span(span, leading);
    let range = ParsedSourceRange {
        start: span.clone(),
        byte_len: text.len(),
    };

    if text.is_empty() {
        return CanonicalExpression {
            node_id,
            range,
            kind: CanonicalExpressionKind::Unit,
        };
    }

    if let Some(rest) = keyword_rest(text, "try") {
        let value_offset = text.len() - rest.len();
        let (value_text, wrapper) = split_typed_failure_wrapper(rest, &span, value_offset);
        return CanonicalExpression {
            node_id: node_id.clone(),
            range,
            kind: CanonicalExpressionKind::Try {
                value: Box::new(parse_canonical_expression(
                    value_text,
                    &offset_span(&span, value_offset),
                    node_id.child("try-value"),
                )),
                failure_root: wrapper
                    .as_ref()
                    .and_then(|wrapper| wrapper.failure_root.clone()),
                failure_variant: wrapper
                    .as_ref()
                    .and_then(|wrapper| wrapper.failure_variant.clone()),
            },
        };
    }

    for (keyword, permission) in [
        ("borrow", ParamPermission::Borrow),
        ("change", ParamPermission::Change),
        ("consume", ParamPermission::Consume),
    ] {
        if let Some(rest) = keyword_rest(text, keyword) {
            let offset = text.len() - rest.len();
            return CanonicalExpression {
                node_id: node_id.clone(),
                range,
                kind: CanonicalExpressionKind::Permission {
                    permission,
                    value: Box::new(parse_canonical_expression(
                        rest,
                        &offset_span(&span, offset),
                        node_id.child("permission-value"),
                    )),
                },
            };
        }
    }

    if let Ok(value) = text.parse::<i64>()
        && value < 0
    {
        return CanonicalExpression {
            node_id,
            range,
            kind: CanonicalExpressionKind::IntLiteral(value),
        };
    }

    if let Some((operator, start, end)) = top_level_binary_operator(text) {
        return CanonicalExpression {
            node_id: node_id.clone(),
            range,
            kind: CanonicalExpressionKind::Binary {
                operator,
                left: Box::new(parse_canonical_expression(
                    &text[..start],
                    &span,
                    node_id.child("binary-left"),
                )),
                right: Box::new(parse_canonical_expression(
                    &text[end..],
                    &offset_span(&span, end),
                    node_id.child("binary-right"),
                )),
            },
        };
    }

    if text.starts_with('(')
        && matching_delimiter_quoted(text, 0, '(', ')') == Some(text.len().saturating_sub(1))
    {
        return CanonicalExpression {
            node_id: node_id.clone(),
            range,
            kind: CanonicalExpressionKind::Group(Box::new(parse_canonical_expression(
                &text[1..text.len() - 1],
                &offset_span(&span, 1),
                node_id.child("group-value"),
            ))),
        };
    }

    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return CanonicalExpression {
            node_id,
            range,
            kind: CanonicalExpressionKind::TextLiteral(text[1..text.len() - 1].to_string()),
        };
    }
    if let Ok(value) = text.parse::<u64>() {
        return CanonicalExpression {
            node_id,
            range,
            kind: CanonicalExpressionKind::UIntLiteral(value),
        };
    }
    if matches!(text, "true" | "false") {
        return CanonicalExpression {
            node_id,
            range,
            kind: CanonicalExpressionKind::BoolLiteral(text == "true"),
        };
    }

    if text.starts_with('[')
        && matching_delimiter_quoted(text, 0, '[', ']') == Some(text.len().saturating_sub(1))
    {
        let inside = &text[1..text.len() - 1];
        let values = split_top_level_ranges_quoted(inside, ',')
            .into_iter()
            .enumerate()
            .filter_map(|(index, value_range)| {
                let raw = &inside[value_range.clone()];
                (!raw.trim().is_empty()).then(|| {
                    parse_canonical_expression(
                        raw,
                        &offset_span(&span, 1 + value_range.start),
                        node_id.child(&format!("list-item-{index}")),
                    )
                })
            })
            .collect();
        return CanonicalExpression {
            node_id,
            range,
            kind: CanonicalExpressionKind::ListLiteral(values),
        };
    }

    if text.ends_with('{') && is_type_identifier(text[..text.len() - 1].trim()) {
        return CanonicalExpression {
            node_id,
            range,
            kind: CanonicalExpressionKind::RecordLiteral {
                name: text[..text.len() - 1].trim().to_string(),
                fields: Vec::new(),
            },
        };
    }

    if let Some(open) = text.find('{')
        && matching_delimiter_quoted(text, open, '{', '}') == Some(text.len().saturating_sub(1))
        && (text[..open].trim().is_empty() || is_type_identifier(text[..open].trim()))
    {
        let name = text[..open].trim().to_string();
        let inside = &text[open + 1..text.len() - 1];
        let fields = split_top_level_ranges_quoted(inside, ',')
            .into_iter()
            .enumerate()
            .filter_map(|(index, field_range)| {
                let raw = &inside[field_range.clone()];
                let (field, value) = raw.split_once(':')?;
                let field = field.trim();
                if !is_value_identifier(field) || value.trim().is_empty() {
                    return None;
                }
                let value_offset = raw.find(value).unwrap_or_default();
                Some((
                    field.to_string(),
                    parse_canonical_expression(
                        value,
                        &offset_span(&span, open + 1 + field_range.start + value_offset),
                        node_id.child(&format!("record-field-{index}")),
                    ),
                ))
            })
            .collect();
        return CanonicalExpression {
            node_id,
            range,
            kind: CanonicalExpressionKind::RecordLiteral { name, fields },
        };
    }

    if let Some(open) = find_top_level_open_paren(text)
        && matching_delimiter_quoted(text, open, '(', ')') == Some(text.len().saturating_sub(1))
    {
        let inside = &text[open + 1..text.len() - 1];
        let arguments = split_top_level_ranges_quoted(inside, ',')
            .into_iter()
            .enumerate()
            .filter_map(|(index, argument_range)| {
                let raw = &inside[argument_range.clone()];
                (!raw.trim().is_empty()).then(|| {
                    parse_canonical_expression(
                        raw,
                        &offset_span(&span, open + 1 + argument_range.start),
                        node_id.child(&format!("call-argument-{index}")),
                    )
                })
            })
            .collect();
        return CanonicalExpression {
            node_id: node_id.clone(),
            range,
            kind: CanonicalExpressionKind::Call {
                callee: Box::new(parse_canonical_expression(
                    &text[..open],
                    &span,
                    node_id.child("call-callee"),
                )),
                arguments,
            },
        };
    }

    if let Some(dot) = find_top_level_dot(text) {
        let field = text[dot + 1..].trim();
        if is_value_identifier(field) {
            return CanonicalExpression {
                node_id: node_id.clone(),
                range,
                kind: CanonicalExpressionKind::Field {
                    base: Box::new(parse_canonical_expression(
                        &text[..dot],
                        &span,
                        node_id.child("field-base"),
                    )),
                    field: field.to_string(),
                },
            };
        }
    }
    CanonicalExpression {
        node_id,
        range,
        kind: if is_value_identifier(text) {
            CanonicalExpressionKind::Identifier(text.to_string())
        } else {
            CanonicalExpressionKind::Unsupported
        },
    }
}

fn top_level_binary_operator(text: &str) -> Option<(ParsedBinaryOperator, usize, usize)> {
    let operators: &[(&str, ParsedBinaryOperator, u8)] = &[
        ("or", ParsedBinaryOperator::Or, 1),
        ("and", ParsedBinaryOperator::And, 2),
        ("==", ParsedBinaryOperator::Equal, 3),
        ("!=", ParsedBinaryOperator::NotEqual, 3),
        ("<=", ParsedBinaryOperator::LessEqual, 3),
        (">=", ParsedBinaryOperator::GreaterEqual, 3),
        ("<", ParsedBinaryOperator::Less, 3),
        (">", ParsedBinaryOperator::Greater, 3),
        ("fails with", ParsedBinaryOperator::FailsWith, 3),
        ("returns", ParsedBinaryOperator::Returns, 3),
        ("does", ParsedBinaryOperator::Does, 3),
        ("is", ParsedBinaryOperator::Is, 3),
        ("+", ParsedBinaryOperator::Add, 4),
        ("-", ParsedBinaryOperator::Subtract, 4),
        ("*", ParsedBinaryOperator::Multiply, 5),
        ("/", ParsedBinaryOperator::Divide, 5),
    ];
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    let mut best: Option<(ParsedBinaryOperator, usize, usize, u8)> = None;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 => {
                for (spelling, operator, precedence) in operators {
                    if text[index..].starts_with(spelling)
                        && operator_boundary(text, index, spelling)
                        && !is_unary_sign(text, index, spelling)
                        && best.is_none_or(|(_, _, _, current)| *precedence <= current)
                    {
                        best = Some((*operator, index, index + spelling.len(), *precedence));
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    best.map(|(operator, start, end, _)| (operator, start, end))
}

fn is_unary_sign(text: &str, start: usize, spelling: &str) -> bool {
    if !matches!(spelling, "+" | "-") {
        return false;
    }
    let before = text[..start].chars().rev().find(|ch| !ch.is_whitespace());
    before.is_none_or(|ch| {
        matches!(
            ch,
            '(' | '[' | '{' | ',' | '=' | '!' | '<' | '>' | '+' | '-' | '*' | '/'
        )
    })
}

fn operator_boundary(text: &str, start: usize, spelling: &str) -> bool {
    if !matches!(
        spelling,
        "and" | "or" | "is" | "does" | "returns" | "fails with"
    ) {
        return true;
    }
    let before = text[..start].chars().next_back();
    let after = text[start + spelling.len()..].chars().next();
    before.is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'))
        && after.is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'))
}

fn find_top_level_open_paren(text: &str) -> Option<usize> {
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        if ch == '"' {
            quoted = true;
        } else if ch == '(' {
            return Some(index);
        }
    }
    None
}

fn find_top_level_dot(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    let mut found = None;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '.' if depth == 0 => found = Some(index),
            _ => {}
        }
    }
    found
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

fn compound_identifier_operands(
    text: &str,
    span: &Span,
    source_node_id: &ParserSyntaxNodeId,
) -> Vec<ParsedExpression> {
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
                let node_id = source_node_id.child(&format!("compound-{}", operands.len()));
                let (canonical, occurrence) = parse_canonical_expression_occurrence(
                    name,
                    &identifier_span,
                    node_id,
                    ParsedExpressionIntent::Other,
                );
                operands.push(ParsedExpression {
                    kind: ParsedExpressionKind::Identifier(ParsedIdentifier {
                        name: name.to_string(),
                        span: identifier_span.clone(),
                    }),
                    canonical,
                    occurrence,
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
    matching_delimiter_quoted(text, open, open_ch, close_ch)
}

fn matching_delimiter_quoted(
    text: &str,
    open: usize,
    open_ch: char,
    close_ch: char,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices().skip_while(|(index, _)| *index < open) {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
        } else if ch == '"' {
            quoted = true;
        } else if ch == open_ch {
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
    split_top_level_ranges_quoted(text, delimiter)
}

fn split_top_level_ranges_quoted(text: &str, delimiter: char) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
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
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if ch == needle && depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

fn unquoted_last_non_whitespace(text: &str) -> Option<char> {
    let mut quoted = false;
    let mut escaped = false;
    let mut last = None;
    for ch in text.chars() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        if ch == '"' {
            quoted = true;
        } else if !ch.is_whitespace() {
            last = Some(ch);
        }
    }
    last
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
    use super::{
        executable_call_nodes, parse_source, parse_source_at_index, validate_canonical_expression,
        validate_retained_body_syntax,
    };
    use crate::ast::{
        CanonicalExpressionKind, Item, ParsedBinaryOperator, ParsedBlockRelationship,
        ParsedBodyStatementKind, ParsedCallCloseStatus, ParsedExpressionIntent,
        ParsedExpressionKind, ParsedLexicalStatus, ParsedLoopKind, ParsedLoopRelationshipKind,
        ParsedMalformedExpressionCause, ParsedStatementSyntaxKind, ParsedTypedFailureWrapperKind,
        ParserSyntaxNodeId, TypeSyntaxKind,
    };
    use crate::diagnostic::{DiagnosticCode, Severity};

    #[derive(Debug)]
    struct RustFunction<'a> {
        name: &'a str,
        signature: &'a str,
        body: &'a str,
    }

    fn rust_code_mask(source: &str) -> String {
        let bytes = source.as_bytes();
        let mut masked = bytes.to_vec();
        let mut index = 0usize;
        let mut block_comment_depth = 0usize;
        while index < bytes.len() {
            if block_comment_depth > 0 {
                if bytes.get(index..index + 2) == Some(b"/*") {
                    masked[index] = b' ';
                    masked[index + 1] = b' ';
                    block_comment_depth += 1;
                    index += 2;
                } else if bytes.get(index..index + 2) == Some(b"*/") {
                    masked[index] = b' ';
                    masked[index + 1] = b' ';
                    block_comment_depth -= 1;
                    index += 2;
                } else {
                    if bytes[index] != b'\n' {
                        masked[index] = b' ';
                    }
                    index += 1;
                }
                continue;
            }
            if bytes.get(index..index + 2) == Some(b"//") {
                while index < bytes.len() && bytes[index] != b'\n' {
                    masked[index] = b' ';
                    index += 1;
                }
                continue;
            }
            if bytes.get(index..index + 2) == Some(b"/*") {
                masked[index] = b' ';
                masked[index + 1] = b' ';
                block_comment_depth = 1;
                index += 2;
                continue;
            }
            if bytes[index] == b'"' {
                masked[index] = b' ';
                index += 1;
                let mut escaped = false;
                while index < bytes.len() {
                    let byte = bytes[index];
                    if byte != b'\n' {
                        masked[index] = b' ';
                    }
                    index += 1;
                    if escaped {
                        escaped = false;
                    } else if byte == b'\\' {
                        escaped = true;
                    } else if byte == b'"' {
                        break;
                    }
                }
                continue;
            }
            if bytes[index] == b'\'' {
                let close = if bytes.get(index + 1) == Some(&b'\\') {
                    index + 3
                } else {
                    index + 2
                };
                if bytes.get(close) == Some(&b'\'') {
                    for value in masked.iter_mut().take(close + 1).skip(index) {
                        *value = b' ';
                    }
                    index = close + 1;
                    continue;
                }
            }
            index += 1;
        }
        String::from_utf8(masked).expect("Rust source mask stays UTF-8")
    }

    fn matching_code_brace(mask: &str, open: usize) -> Option<usize> {
        let mut depth = 0usize;
        for (offset, byte) in mask.as_bytes()[open..].iter().copied().enumerate() {
            match byte {
                b'{' => depth += 1,
                b'}' => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(open + offset);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn production_source_without_cfg_test_items(source: &str) -> Result<String, &'static str> {
        let mask = rust_code_mask(source);
        let marker = "#[cfg(test)]";
        let mut production = source.as_bytes().to_vec();
        let mut search = 0usize;
        let mut removed = 0usize;
        while let Some(relative) = mask[search..].find(marker) {
            let start = search + relative;
            let open = mask[start..]
                .find('{')
                .map(|offset| start + offset)
                .ok_or("cfg_test_item_open_missing_v0")?;
            let close = matching_code_brace(&mask, open).ok_or("cfg_test_item_close_missing_v0")?;
            for (index, value) in production
                .iter_mut()
                .enumerate()
                .take(close + 1)
                .skip(start)
            {
                if source.as_bytes()[index] != b'\n' {
                    *value = b' ';
                }
            }
            removed += 1;
            search = close + 1;
        }
        if removed == 0 {
            return Err("cfg_test_item_missing_v0");
        }
        String::from_utf8(production).map_err(|_| "production_source_utf8_corrupt_v0")
    }

    fn rust_functions(source: &str) -> Result<Vec<RustFunction<'_>>, &'static str> {
        let mask = rust_code_mask(source);
        let bytes = mask.as_bytes();
        let mut functions = Vec::new();
        let mut index = 0usize;
        while index + 2 < bytes.len() {
            let boundary_before = index == 0
                || !(bytes[index - 1].is_ascii_alphanumeric() || bytes[index - 1] == b'_');
            let boundary_after =
                !(bytes[index + 2].is_ascii_alphanumeric() || bytes[index + 2] == b'_');
            if boundary_before && boundary_after && &bytes[index..index + 2] == b"fn" {
                let mut name_start = index + 2;
                while bytes
                    .get(name_start)
                    .is_some_and(|byte| byte.is_ascii_whitespace())
                {
                    name_start += 1;
                }
                let mut name_end = name_start;
                while bytes
                    .get(name_end)
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                {
                    name_end += 1;
                }
                let open = mask[name_end..]
                    .find('{')
                    .map(|offset| name_end + offset)
                    .ok_or("production_function_open_missing_v0")?;
                let close = matching_code_brace(&mask, open)
                    .ok_or("production_function_close_missing_v0")?;
                functions.push(RustFunction {
                    name: &source[name_start..name_end],
                    signature: &source[index..open],
                    body: &source[open + 1..close],
                });
                index = close + 1;
            } else {
                index += 1;
            }
        }
        Ok(functions)
    }

    fn struct_field_names<'a>(
        source: &'a str,
        declaration: &str,
    ) -> Result<Vec<&'a str>, &'static str> {
        let mask = rust_code_mask(source);
        let start = mask
            .find(declaration)
            .ok_or("authority_struct_missing_v0")?;
        let open = mask[start..]
            .find('{')
            .map(|offset| start + offset)
            .ok_or("authority_struct_open_missing_v0")?;
        let close = matching_code_brace(&mask, open).ok_or("authority_struct_close_missing_v0")?;
        let mut fields = Vec::new();
        for line in source[open + 1..close].lines() {
            let line = line.trim();
            let line = line.strip_prefix("pub ").unwrap_or(line);
            if let Some((name, _)) = line.split_once(':') {
                fields.push(name.trim());
            }
        }
        Ok(fields)
    }

    fn audit_h0010_production_dataflow(
        parser_source: &str,
        ast_source: &str,
    ) -> Result<usize, &'static str> {
        let production = production_source_without_cfg_test_items(parser_source)?;
        let functions = rust_functions(&production)?;
        let roots = functions
            .iter()
            .filter(|function| {
                function
                    .body
                    .contains("DiagnosticCode::CHAINED_COMPARISON_NOT_SUPPORTED")
            })
            .collect::<Vec<_>>();
        if roots.len() != 1
            || production
                .matches("DiagnosticCode::CHAINED_COMPARISON_NOT_SUPPORTED")
                .count()
                != 1
            || production
                .matches("DiagnosticCauseKey::producer_owned(179)")
                .count()
                != 1
        {
            return Err("h0010_emission_authority_not_unique_v0");
        }
        let mut reachable = std::collections::BTreeSet::from([roots[0].name]);
        loop {
            let before = reachable.len();
            for function in &functions {
                if !reachable.contains(function.name) {
                    continue;
                }
                let mask = rust_code_mask(function.body);
                for candidate in &functions {
                    if mask.match_indices(candidate.name).any(|(index, _)| {
                        let before = index.checked_sub(1).and_then(|at| mask.as_bytes().get(at));
                        let after = mask.as_bytes().get(index + candidate.name.len());
                        !before.is_some_and(|byte| {
                            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':')
                        }) && after.is_some_and(|byte| *byte == b'(')
                    }) {
                        reachable.insert(candidate.name);
                    }
                }
            }
            if reachable.len() == before {
                break;
            }
        }
        for function in &functions {
            if !reachable.contains(function.name) {
                continue;
            }
            if function.signature.contains("&str")
                || function.signature.contains("String")
                || function.body.contains("ParsedExpressionKind")
                || function.body.contains("ParsedCall")
                || function.body.contains("render_parsed_expression")
                || function.body.contains("source_text_for_range")
                || function.body.contains("top_level_binary_operator")
                || function.body.contains("parser_owned_top_level_call_ranges")
            {
                return Err("h0010_raw_or_parallel_authority_reachable_v0");
            }
        }
        let occurrence_fields =
            struct_field_names(ast_source, "pub struct ParsedExpressionOccurrenceFacts")?;
        if occurrence_fields
            != [
                "root_node_id",
                "intent",
                "intent_signal",
                "maximum_delimiter_depth",
                "lexical_status",
                "nodes",
                "typed_failure_wrapper",
            ]
        {
            return Err("expression_occurrence_parallel_authority_field_v0");
        }
        let node_fields = struct_field_names(ast_source, "pub struct ParsedCanonicalNodeSyntax")?;
        if node_fields
            != [
                "node_id",
                "child_position",
                "range",
                "operator",
                "delimiter",
                "call",
                "delimiter_depth",
                "lexical_status",
            ]
        {
            return Err("canonical_node_parallel_authority_field_v0");
        }
        Ok(reachable.len())
    }

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

    #[test]
    fn string_braces_and_escaped_quotes_do_not_close_items() {
        let parsed = parse_source(
            "text-braces.hum",
            r#"task braces() -> Text {
  does:
    let first = "}{"
    return "escaped \" } { remains text"
}

task after() -> UInt {
  does:
    return 7
}
"#,
        );
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.file.items.len(), 2);
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        assert_eq!(task.body_syntax.len(), 2);
        assert!(task.body_syntax.iter().all(|statement| {
            statement.block_relationship == ParsedBlockRelationship::None
                && statement.block_depth_before == 0
                && statement.block_depth_after == 0
        }));
    }

    #[test]
    fn quote_escape_and_brace_direction_sabotage_changes_scope_facts() {
        let valid = parse_source(
            "valid-scope.hum",
            "task first() -> Text {\n  does:\n    return \"escaped \\\" } { text\"\n}\n\ntask second() -> UInt {\n  does:\n    return 2\n}\n",
        );
        assert!(valid.diagnostics.is_empty());
        assert_eq!(valid.file.items.len(), 2);

        let escape_removed = parse_source(
            "escape-removed.hum",
            "task first() -> Text {\n  does:\n    return \"escaped \" } { text\"\n}\n\ntask second() -> UInt {\n  does:\n    return 2\n}\n",
        );
        assert!(
            !escape_removed.diagnostics.is_empty() || escape_removed.file.items.len() != 2,
            "removing the escaped quote must not preserve the valid scope fact"
        );

        let quotes_removed = parse_source(
            "quotes-removed.hum",
            "task first() -> Text {\n  does:\n    return }{\n}\n\ntask second() -> UInt {\n  does:\n    return 2\n}\n",
        );
        assert!(
            !quotes_removed.diagnostics.is_empty() || quotes_removed.file.items.len() != 2,
            "reversing unquoted brace direction must not preserve the valid scope fact"
        );
    }

    #[test]
    fn retained_block_relationship_corruption_fails_closed() {
        let parsed = parse_source(
            "block-relationships.hum",
            "task block(value: UInt) -> UInt {\n  does:\n    if value > 0 {\n      return value\n    }\n    return 0\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        assert!(validate_retained_body_syntax(&task.body_syntax).is_ok());

        let mut wrong_direction = task.body_syntax.clone();
        wrong_direction[0].block_relationship = ParsedBlockRelationship::Closes;
        assert!(validate_retained_body_syntax(&wrong_direction).is_err());

        let mut wrong_depth = task.body_syntax.clone();
        wrong_depth[1].block_depth_before += 1;
        assert!(validate_retained_body_syntax(&wrong_depth).is_err());

        let mut wrong_id = task.body_syntax.clone();
        wrong_id[1].source_node_id = wrong_id[0].source_node_id.clone();
        assert!(validate_retained_body_syntax(&wrong_id).is_err());
    }

    #[test]
    fn genuine_unclosed_item_still_owns_h0004() {
        for literal in ["}{", "{{", "}}", "plain"] {
            let parsed = parse_source(
                "real-unclosed.hum",
                &format!("task unclosed() -> Text {{\n  does:\n    return \"{literal}\"\n"),
            );
            assert_eq!(
                parsed
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.code == DiagnosticCode::ITEM_BLOCK_MISSING_CLOSE_BRACE
                            && diagnostic.span.as_ref().is_some_and(|span| span.line == 1)
                    })
                    .count(),
                1,
                "literal {literal} cannot repair the real block"
            );
        }
    }

    #[test]
    fn canonical_expression_tree_is_left_associative_and_precedence_aware() {
        let parsed = parse_source(
            "expression-tree.hum",
            "task expressions() -> UInt {\n  does:\n    let a = 8 * 6 / 4\n    let b = 20 - 6 - 4\n    let c = 2 + 3 * 4\n    return (2 + 3) * 4\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let expressions = task
            .body_syntax
            .iter()
            .map(|statement| match &statement.kind {
                ParsedBodyStatementKind::Binding {
                    value: Some(value), ..
                }
                | ParsedBodyStatementKind::Return(value) => &value.canonical,
                other => panic!("expression statement: {other:?}"),
            })
            .collect::<Vec<_>>();
        assert!(matches!(
            &expressions[0].kind,
            CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Divide,
                left,
                ..
            } if matches!(left.kind, CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Multiply, ..
            })
        ));
        assert!(matches!(
            &expressions[1].kind,
            CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Subtract,
                left,
                ..
            } if matches!(left.kind, CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Subtract, ..
            })
        ));
        assert!(matches!(
            &expressions[2].kind,
            CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Add,
                right,
                ..
            } if matches!(right.kind, CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Multiply, ..
            })
        ));
        assert!(matches!(
            &expressions[3].kind,
            CanonicalExpressionKind::Binary {
                operator: ParsedBinaryOperator::Multiply,
                left,
                ..
            } if matches!(left.kind, CanonicalExpressionKind::Group(_))
        ));
        assert!(
            expressions
                .iter()
                .all(|expression| validate_canonical_expression(expression).is_ok())
        );
    }

    #[test]
    fn canonical_expression_corruption_fails_closed() {
        let parsed = parse_source(
            "expression-corruption.hum",
            "task expression() -> UInt {\n  does:\n    return 2 + 3 * 4\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let ParsedBodyStatementKind::Return(value) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        let mut wrong_id = value.canonical.clone();
        wrong_id.node_id = ParserSyntaxNodeId::new("substituted-node".to_string());
        assert!(validate_canonical_expression(&wrong_id).is_err());

        let mut wrong_span = value.canonical.clone();
        if let CanonicalExpressionKind::Binary { right, .. } = &mut wrong_span.kind {
            right.range.start.column =
                wrong_span.range.start.column + wrong_span.range.byte_len + 1;
        }
        assert!(validate_canonical_expression(&wrong_span).is_err());

        let mut reordered = value.canonical.clone();
        if let CanonicalExpressionKind::Binary { left, right, .. } = &mut reordered.kind {
            std::mem::swap(left, right);
        }
        assert!(validate_canonical_expression(&reordered).is_err());
    }

    #[test]
    fn non_ascii_unsupported_expression_is_utf8_safe() {
        let parsed = parse_source(
            "non-ascii-expression.hum",
            "task non_ascii() -> Text {\n  does:\n    return café\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let ParsedBodyStatementKind::Return(value) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        assert!(matches!(
            value.canonical.kind,
            CanonicalExpressionKind::Unsupported
        ));
        assert!(validate_canonical_expression(&value.canonical).is_ok());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(
            parsed.diagnostics[0].code,
            DiagnosticCode::INVALID_IDENTIFIER
        );
        assert_eq!(
            task.body_syntax[0].core_expression_kind,
            Some("name_or_text")
        );
    }

    #[test]
    fn signed_int_is_one_structural_literal_node() {
        let parsed = parse_source(
            "signed-int-expression.hum",
            "task signed() -> Int {\n  does:\n    return -1\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let ParsedBodyStatementKind::Return(value) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        assert!(matches!(
            value.canonical.kind,
            CanonicalExpressionKind::IntLiteral(-1)
        ));
        assert!(validate_canonical_expression(&value.canonical).is_ok());
        assert_eq!(
            task.body_syntax[0].core_expression_kind,
            Some("name_or_text")
        );
    }

    #[test]
    fn increment_10b1_expression_occurrence_facts_are_parser_owned_and_exact() {
        let parsed = parse_source(
            "expression-occurrence.hum",
            "task choose(value: UInt) -> UInt {\n  does:\n    return try apply(change value, [1, 2], Pair { left: 1, right: 2 }) or fail ChoiceError.invalid\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let statement = &task.body_syntax[0];
        let ParsedBodyStatementKind::Return(value) = &statement.kind else {
            panic!("return")
        };

        assert_eq!(value.occurrence.intent, ParsedExpressionIntent::Return);
        assert_eq!(value.occurrence.root_node_id, value.canonical.node_id);
        assert!(value.occurrence.maximum_delimiter_depth >= 2);
        assert_eq!(statement.syntax.kind, ParsedStatementSyntaxKind::Return);
        assert_eq!(
            statement.syntax.expression_nodes,
            vec![value.canonical.node_id.clone()]
        );
        let wrapper = value
            .occurrence
            .typed_failure_wrapper
            .as_ref()
            .expect("typed failure wrapper");
        assert_eq!(wrapper.kind, ParsedTypedFailureWrapperKind::Wrap);
        assert_eq!(
            wrapper
                .failure_root
                .as_ref()
                .map(|value| value.name.as_str()),
            Some("ChoiceError")
        );
        assert_eq!(
            wrapper
                .failure_variant
                .as_ref()
                .map(|value| value.name.as_str()),
            Some("invalid")
        );

        let call = value
            .occurrence
            .nodes
            .iter()
            .find_map(|node| node.call.as_ref())
            .expect("call syntax facts");
        assert!(call.close.is_some());
        assert_eq!(call.separators.len(), 2);
        assert!(!call.gaps.is_empty());
        assert!(
            value
                .occurrence
                .nodes
                .iter()
                .enumerate()
                .all(|(index, node)| index == 0 || !node.child_position.is_empty())
        );
        assert!(super::validate_expression_occurrence(value).is_ok());

        let mut missing_node = value.clone();
        missing_node.occurrence.nodes.pop();
        assert_eq!(
            super::validate_expression_occurrence(&missing_node),
            Err("parser_expression_occurrence_projection_corrupt_v0")
        );

        let mut substituted_position = value.clone();
        substituted_position.occurrence.nodes[1].child_position = vec![99];
        assert_eq!(
            super::validate_expression_occurrence(&substituted_position),
            Err("parser_expression_occurrence_projection_corrupt_v0")
        );
    }

    #[test]
    fn increment_10b1_malformed_lexical_facts_are_structured_exact_and_fail_closed() {
        let parse_contract = |text: &str| {
            let parsed = parse_source(
                "malformed-expression-facts.hum",
                &format!(
                    "task lexical(value: UInt) -> UInt {{\n  ensures:\n    {text}\n\n  does:\n    return value\n}}\n"
                ),
            );
            let Item::Task(task) = &parsed.file.items[0] else {
                panic!("task")
            };
            task.sections
                .iter()
                .find(|section| section.name == "ensures")
                .and_then(|section| section.expression_syntax[0].clone())
                .expect("parser-owned contract expression")
        };

        let cases = [
            (
                "value == \"unterminated",
                ParsedMalformedExpressionCause::UnterminatedTextLiteral,
            ),
            (
                "value == (1]",
                ParsedMalformedExpressionCause::MismatchedDelimiter,
            ),
            (
                "value == (1",
                ParsedMalformedExpressionCause::MissingDelimiter,
            ),
            ("value ==", ParsedMalformedExpressionCause::MissingOperand),
            (
                "value = 1",
                ParsedMalformedExpressionCause::InvalidComparisonOperator,
            ),
            (
                "@ == 1",
                ParsedMalformedExpressionCause::InvalidOperandStarter,
            ),
            (
                "value. == 1",
                ParsedMalformedExpressionCause::MalformedFieldPlace,
            ),
            (
                "list_count([\"a\" \"b\"], \"a\") > 0",
                ParsedMalformedExpressionCause::ListElementSeparator,
            ),
            (
                "list_count([\"a\",], \"a\") > 0",
                ParsedMalformedExpressionCause::ListTrailingComma,
            ),
            (
                "list_count([1], \"a\") > 0",
                ParsedMalformedExpressionCause::ListNonTextElement,
            ),
            (
                "999999999999999999999999 == 1",
                ParsedMalformedExpressionCause::IntegerLiteralOutOfRange,
            ),
            (
                "(((((((((((((((((value == 1)))))))))))))))))",
                ParsedMalformedExpressionCause::DelimiterDepthExceeded,
            ),
        ];
        for (text, expected_cause) in cases {
            let expression = parse_contract(text);
            let ParsedLexicalStatus::Malformed(fact) = &expression.occurrence.lexical_status else {
                panic!(
                    "missing structured lexical issue for {text}: {:#?}",
                    expression.occurrence
                )
            };
            assert_eq!(fact.cause, expected_cause, "{text}");
            assert!(expression.occurrence.intent_signal.is_some(), "{text}");
            assert!(
                super::validate_expression_occurrence(&expression).is_ok(),
                "{text}"
            );
            assert_eq!(
                expression.occurrence.nodes[0].lexical_status, expression.occurrence.lexical_status,
                "{text}"
            );
        }

        let expression = parse_contract("value == (1]");
        let mut missing = expression.clone();
        missing.occurrence.lexical_status = ParsedLexicalStatus::Complete;
        assert_eq!(
            super::validate_expression_occurrence(&missing),
            Err("parser_expression_occurrence_root_corrupt_v0")
        );

        let mut substituted = expression.clone();
        let ParsedLexicalStatus::Malformed(fact) = &mut substituted.occurrence.lexical_status
        else {
            panic!("malformed fact")
        };
        fact.cause = ParsedMalformedExpressionCause::IntegerLiteralOutOfRange;
        substituted.occurrence.nodes[0].lexical_status =
            substituted.occurrence.lexical_status.clone();
        assert_eq!(
            super::validate_expression_occurrence(&substituted),
            Err("parser_expression_lexical_evidence_corrupt_v0")
        );

        let mut wrong_range = expression.clone();
        let ParsedLexicalStatus::Malformed(fact) = &mut wrong_range.occurrence.lexical_status
        else {
            panic!("malformed fact")
        };
        fact.offending.start.column =
            expression.canonical.range.start.column + expression.canonical.range.byte_len + 1;
        wrong_range.occurrence.nodes[0].lexical_status =
            wrong_range.occurrence.lexical_status.clone();
        assert_eq!(
            super::validate_expression_occurrence(&wrong_range),
            Err("parser_expression_lexical_range_corrupt_v0")
        );

        let valid = parse_contract("value == 1");
        assert_eq!(
            valid.occurrence.lexical_status,
            ParsedLexicalStatus::Complete
        );
        assert!(matches!(
            valid.occurrence.intent_signal,
            Some(ref signal) if signal.byte_len == 1
        ));
        assert!(matches!(
            &valid.occurrence.nodes[0].operator,
            Some(operator) if operator.byte_len == 2
        ));
    }

    #[test]
    fn increment_10b1_h0010_source_audit_is_structural_complete_and_sabotage_sensitive() {
        let parser_source = include_str!("parser.rs");
        let ast_source = include_str!("ast.rs");
        assert_eq!(
            audit_h0010_production_dataflow(parser_source, ast_source),
            Ok(9)
        );

        let sabotage = parser_source
            .replace(
                "for chain in chained_comparison_sites(expression) {",
                "let _competing = renamed_raw_authority(expression.span.file.as_str());\n        for chain in chained_comparison_sites(expression) {",
            )
            .replacen(
                "#[cfg(test)]",
                "fn renamed_raw_authority(raw: &str) -> bool { raw.split('<').count() > 2 }\n\n#[cfg(test)]",
                1,
            );
        assert_eq!(
            audit_h0010_production_dataflow(&sabotage, ast_source),
            Err("h0010_raw_or_parallel_authority_reachable_v0")
        );
    }

    #[test]
    fn increment_10b1_h0010_depends_on_canonical_tree_not_retained_legacy_projection() {
        let parsed = parse_source(
            "canonical-chain-authority.hum",
            "task chained() -> Bool {\n  does:\n    return (1 < 2 < 3) and true\n}\n",
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let ParsedBodyStatementKind::Return(expression) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        assert_eq!(super::chained_comparison_sites(expression).len(), 1);

        let mut corrupted = expression.clone();
        let CanonicalExpressionKind::Binary { left, .. } = &mut corrupted.canonical.kind else {
            panic!("Boolean root")
        };
        let CanonicalExpressionKind::Group(chain) = &mut left.kind else {
            panic!("grouped chain")
        };
        chain.kind = CanonicalExpressionKind::BoolLiteral(true);
        assert!(super::chained_comparison_sites(&corrupted).is_empty());
        assert_eq!(
            corrupted.kind, expression.kind,
            "legacy projection held fixed"
        );
        assert_eq!(
            corrupted.occurrence, expression.occurrence,
            "occurrence projection held fixed"
        );
        assert!(super::validate_expression_occurrence(&corrupted).is_err());
    }

    #[test]
    fn increment_10b1_statement_relationship_facts_are_structured_and_exact() {
        let parsed = parse_source(
            "statement-facts.hum",
            r#"task statement_facts(value: UInt, items: List UInt) -> UInt {
  does:
    let item = value
    set item = item + 1
    save item in items
    if item > 0 {
      return item
    }
    while item < 3 {
      set item = item + 1
    }
    for each entry in items {
      return entry
    }
    for index position from 0 until 3 {
      return position
    }
    for index slot from 1 through 4 {
      return slot
    }
    return item
}
"#,
        );
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        assert!(validate_retained_body_syntax(&task.body_syntax).is_ok());

        let binding = &task.body_syntax[0].syntax;
        assert_eq!(
            binding.kind,
            ParsedStatementSyntaxKind::Binding { mutable: false }
        );
        assert_eq!(
            binding.binding.as_ref().map(|value| value.name.as_str()),
            Some("item")
        );
        assert!(binding.relationship_token.is_some());
        assert_eq!(binding.expression_nodes.len(), 1);

        let set = &task.body_syntax[1].syntax;
        assert_eq!(set.kind, ParsedStatementSyntaxKind::Set);
        assert!(matches!(
            set.target.as_ref().map(|target| &target.canonical.kind),
            Some(CanonicalExpressionKind::Identifier(name)) if name == "item"
        ));
        assert!(set.relationship_token.is_some());
        assert_eq!(set.expression_nodes.len(), 1);

        let save = &task.body_syntax[2].syntax;
        assert_eq!(save.kind, ParsedStatementSyntaxKind::Save);
        assert_eq!(
            save.destination.as_ref().map(|value| value.name.as_str()),
            Some("items")
        );
        assert!(save.relationship_token.is_some());
        assert_eq!(save.expression_nodes.len(), 1);

        let condition = &task.body_syntax[3];
        assert_eq!(condition.syntax.kind, ParsedStatementSyntaxKind::Condition);
        assert_eq!(
            super::statement_expressions(condition)[0].occurrence.intent,
            ParsedExpressionIntent::Condition
        );

        let while_loop = &task.body_syntax[6];
        assert_eq!(
            while_loop.syntax.kind,
            ParsedStatementSyntaxKind::Loop {
                kind: ParsedLoopKind::While
            }
        );
        assert_eq!(
            super::statement_expressions(while_loop)[0]
                .occurrence
                .intent,
            ParsedExpressionIntent::Condition
        );

        let for_each = &task.body_syntax[9];
        assert_eq!(
            for_each.syntax.kind,
            ParsedStatementSyntaxKind::Loop {
                kind: ParsedLoopKind::ForEach
            }
        );
        assert_eq!(
            super::statement_expressions(for_each)[0].occurrence.intent,
            ParsedExpressionIntent::LoopCollection
        );
        let for_each_relationship = for_each
            .syntax
            .loop_relationship
            .as_ref()
            .expect("for each relationship");
        assert_eq!(
            for_each_relationship.kind,
            ParsedLoopRelationshipKind::CollectionIn
        );
        assert_eq!(for_each_relationship.binding.name, "entry");
        assert_eq!(for_each_relationship.introducer.byte_len, 2);
        assert!(for_each_relationship.bound.is_none());
        assert_eq!(
            for_each_relationship.expression_nodes,
            for_each.syntax.expression_nodes
        );

        let for_index = &task.body_syntax[12];
        assert_eq!(
            for_index.syntax.kind,
            ParsedStatementSyntaxKind::Loop {
                kind: ParsedLoopKind::ForIndex
            }
        );
        let for_index_relationship = for_index
            .syntax
            .loop_relationship
            .as_ref()
            .expect("for index relationship");
        assert_eq!(
            for_index_relationship.kind,
            ParsedLoopRelationshipKind::RangeUntil
        );
        assert_eq!(for_index_relationship.binding.name, "position");
        assert_eq!(for_index_relationship.introducer.byte_len, 4);
        assert_eq!(
            for_index_relationship
                .bound
                .as_ref()
                .map(|range| range.byte_len),
            Some(5)
        );
        assert_eq!(
            super::statement_expressions(for_index)
                .iter()
                .map(|expression| expression.occurrence.intent)
                .collect::<Vec<_>>(),
            [
                ParsedExpressionIntent::LoopRangeStart,
                ParsedExpressionIntent::LoopRangeEnd
            ]
        );

        let for_index_through = &task.body_syntax[15];
        let through_relationship = for_index_through
            .syntax
            .loop_relationship
            .as_ref()
            .expect("for index through relationship");
        assert_eq!(
            through_relationship.kind,
            ParsedLoopRelationshipKind::RangeThrough
        );
        assert_eq!(through_relationship.binding.name, "slot");
        assert_eq!(
            through_relationship
                .bound
                .as_ref()
                .map(|range| range.byte_len),
            Some(7)
        );

        let mut wrong_relationship = task.body_syntax.clone();
        wrong_relationship[1].syntax.expression_nodes =
            wrong_relationship[0].syntax.expression_nodes.clone();
        assert_eq!(
            validate_retained_body_syntax(&wrong_relationship),
            Err("parser_statement_expression_relationship_corrupt_v0")
        );

        let mut missing_loop_binding = task.body_syntax.clone();
        missing_loop_binding[9].syntax.binding = None;
        assert_eq!(
            validate_retained_body_syntax(&missing_loop_binding),
            Err("parser_loop_binding_relationship_corrupt_v0")
        );

        let mut substituted_loop_binding = task.body_syntax.clone();
        substituted_loop_binding[9]
            .syntax
            .loop_relationship
            .as_mut()
            .expect("for each relationship")
            .binding = task.body_syntax[12]
            .syntax
            .binding
            .clone()
            .expect("for index binding");
        assert_eq!(
            validate_retained_body_syntax(&substituted_loop_binding),
            Err("parser_loop_binding_relationship_corrupt_v0")
        );

        let mut substituted_range_relation = task.body_syntax.clone();
        substituted_range_relation[12]
            .syntax
            .loop_relationship
            .as_mut()
            .expect("for index relationship")
            .kind = ParsedLoopRelationshipKind::RangeThrough;
        assert_eq!(
            validate_retained_body_syntax(&substituted_range_relation),
            Err("parser_loop_binding_relationship_corrupt_v0")
        );

        let mut missing_range_bound = task.body_syntax.clone();
        missing_range_bound[12]
            .syntax
            .loop_relationship
            .as_mut()
            .expect("for index relationship")
            .bound = None;
        assert_eq!(
            validate_retained_body_syntax(&missing_range_bound),
            Err("parser_loop_binding_relationship_corrupt_v0")
        );

        let mut reordered_range = task.body_syntax.clone();
        let ParsedBodyStatementKind::Other { expressions } = &mut reordered_range[12].kind else {
            panic!("for index expressions")
        };
        expressions.swap(0, 1);
        assert_eq!(
            validate_retained_body_syntax(&reordered_range),
            Err("parser_statement_expression_relationship_corrupt_v0")
        );
    }

    #[test]
    fn increment_10b1_h0010_recurses_over_canonical_expression_children() {
        let parsed = parse_source(
            "recursive-chain.hum",
            r#"task inspect(value: UInt) -> Bool {
  does:
    let call_chain = choose(1 < 2 < 3)
    let list_chain = [1 < 2 < 3]
    let record_chain = Pair { value: 1 < 2 < 3 }
    let permission_chain = borrow (1 < 2 < 3)
    let try_chain = try choose(1 < 2 < 3)
    return (1 < 2 < 3) and true
}
"#,
        );
        let diagnostics = parsed
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code == DiagnosticCode::CHAINED_COMPARISON_NOT_SUPPORTED
            })
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 6, "{:#?}", parsed.diagnostics);
        assert!(diagnostics.iter().all(|diagnostic| {
            diagnostic.message == "comparison chaining is not supported"
                && diagnostic.related_spans.len() == 1
                && diagnostic.related_spans[0].label == "first comparison already being chained"
                && diagnostic
                    .span
                    .as_ref()
                    .is_some_and(|primary| primary.column > diagnostic.related_spans[0].span.column)
        }));

        let occurrences = parsed
            .diagnostic_occurrences
            .normalized_occurrences()
            .into_iter()
            .filter(|occurrence| {
                occurrence.diagnostic().code == DiagnosticCode::CHAINED_COMPARISON_NOT_SUPPORTED
            })
            .collect::<Vec<_>>();
        assert_eq!(occurrences.len(), 6);
        assert!(occurrences.iter().all(|occurrence| {
            occurrence.cause_key().ordinal() == 179
                && occurrence.owning_stage() == "parser"
                && occurrence.semantic_owner() == "source_shape"
        }));
        assert_eq!(
            occurrences
                .iter()
                .map(|occurrence| occurrence.id())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            6
        );

        let accepted = parse_source(
            "independent-comparisons.hum",
            "task accepted() -> Bool {\n  does:\n    let text = \"1 < 2 < 3\"\n    return 1 < 2 and 2 < 3\n}\n",
        );
        assert!(accepted.diagnostics.iter().all(|diagnostic| {
            diagnostic.code != DiagnosticCode::CHAINED_COMPARISON_NOT_SUPPORTED
        }));
    }
}
