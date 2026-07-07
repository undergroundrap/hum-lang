use crate::ast::{Item, Program, Section};
use crate::core_body::{self, BodyGrammarReport, BodyStatement};
use crate::core_contract;
use crate::core_expr::{
    self, CoreExpressionAstPreview, CoreExpressionNode, CoreExpressionPreview, ExpressionAtom,
};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::version;

pub const CORE_PREVIEW_SCHEMA: &str = "hum.core_preview.v0";
pub const CORE_PREVIEW_STATUS: &str = "preview_v0";

struct CorePreviewReport {
    files: usize,
    items: usize,
    tasks: usize,
    tests: usize,
    errors: usize,
    warnings: usize,
    candidates: Vec<CoreCandidate>,
}

struct CoreCandidate {
    id: String,
    kind: &'static str,
    name: String,
    span: Span,
    status: &'static str,
    body_status: &'static str,
    grammar_status: &'static str,
    meaningful_lines: usize,
    lowerable_preview_statements: usize,
    contextual_preview_statements: usize,
    blocked_statements: usize,
    expression_previews: usize,
    expression_atoms: usize,
    expression_ast_nodes: usize,
    compound_expression_previews: usize,
    block_status: &'static str,
    block_count: usize,
    max_block_depth: usize,
    unmatched_block_closes: usize,
    unclosed_blocks: usize,
    source_sections: Vec<String>,
    block_preview: CoreBlockPreview,
    statements: Vec<CoreStatementPreview>,
}

struct CoreStatementPreview {
    span: Span,
    text: String,
    source_kind: &'static str,
    source_status: &'static str,
    core_operation: &'static str,
    status: &'static str,
    expression_kind: Option<&'static str>,
    expression_preview: Option<CoreExpressionPreview>,
    reason: Option<&'static str>,
}

struct CoreBlockPreview {
    status: &'static str,
    block_count: usize,
    max_depth: usize,
    unmatched_closes: usize,
    unclosed_blocks: usize,
    root: CoreBlockNode,
}

struct CoreBlockNode {
    id: String,
    block_kind: &'static str,
    status: &'static str,
    header_statement_index: Option<usize>,
    closing_statement_index: Option<usize>,
    children: Vec<CoreBlockChild>,
    reason: Option<&'static str>,
}

enum CoreBlockChild {
    Statement(CoreBlockStatementRef),
    Block(CoreBlockNode),
}

struct CoreBlockStatementRef {
    statement_index: usize,
    core_operation: &'static str,
    status: &'static str,
    reason: Option<&'static str>,
}

pub fn core_preview_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum Core preview ({CORE_PREVIEW_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!(
        "core_contract_schema: {}\n",
        core_contract::CORE_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "summary: files={} items={} tasks={} tests={} core_candidates={} execution_ready=0 errors={} warnings={} lowerable_preview_statements={} contextual_preview_statements={} blocked_statements={} expression_previews={} expression_atoms={} expression_ast_nodes={} compound_expression_previews={} block_count={} max_block_depth={} unmatched_block_closes={} unclosed_blocks={}\n",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.candidates.len(),
        report.errors,
        report.warnings,
        report.lowerable_preview_statements(),
        report.contextual_preview_statements(),
        report.blocked_statements(),
        report.expression_previews(),
        report.expression_atoms(),
        report.expression_ast_nodes(),
        report.compound_expression_previews(),
        report.block_count(),
        report.max_block_depth(),
        report.unmatched_block_closes(),
        report.unclosed_blocks()
    ));

    if report.candidates.is_empty() {
        out.push_str("core_candidates: none\n");
        return out;
    }

    out.push_str("core_candidates:\n");
    for candidate in &report.candidates {
        out.push_str(&format!(
            "  {}:{}:{} [{}] {} `{}`\n",
            candidate.span.file,
            candidate.span.line,
            candidate.span.column,
            candidate.status,
            candidate.kind,
            candidate.name
        ));
        out.push_str(&format!(
            "    body: {} grammar={} block_status={} meaningful_lines={} lowerable_preview_statements={} contextual_preview_statements={} blocked_statements={} expression_previews={} expression_atoms={} expression_ast_nodes={} compound_expression_previews={} block_count={} max_block_depth={} unmatched_block_closes={} unclosed_blocks={}\n",
            candidate.body_status,
            candidate.grammar_status,
            candidate.block_status,
            candidate.meaningful_lines,
            candidate.lowerable_preview_statements,
            candidate.contextual_preview_statements,
            candidate.blocked_statements,
            candidate.expression_previews,
            candidate.expression_atoms,
            candidate.expression_ast_nodes,
            candidate.compound_expression_previews,
            candidate.block_count,
            candidate.max_block_depth,
            candidate.unmatched_block_closes,
            candidate.unclosed_blocks
        ));
        for statement in &candidate.statements {
            out.push_str(&format!(
                "    {}:{}:{} [{}] {} -> {}\n",
                statement.span.file,
                statement.span.line,
                statement.span.column,
                statement.status,
                statement.source_kind,
                statement.core_operation
            ));
        }
    }

    out
}

pub fn core_preview_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", CORE_PREVIEW_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(
        &mut out,
        2,
        "core_contract_schema",
        core_contract::CORE_CONTRACT_SCHEMA,
        true,
    );
    push_summary(&mut out, &report, 2, true);
    push_candidates(&mut out, &report.candidates, 2, true);
    push_string_array(
        &mut out,
        2,
        "non_goals_v0",
        &[
            "no executable semantics",
            "no type checking",
            "no effect checking",
            "no interpreter",
            "no backend IR",
            "no generated artifact",
            "no safety proof",
        ],
        false,
    );
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> CorePreviewReport {
    let mut candidates = Vec::new();
    for file in &program.files {
        collect_candidates_from_items(&file.items, diagnostics, &mut candidates);
    }

    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len().saturating_sub(errors);

    CorePreviewReport {
        files: program.files.len(),
        items: count_items(program),
        tasks: count_kind(program, "task"),
        tests: count_kind(program, "test"),
        errors,
        warnings,
        candidates,
    }
}

fn collect_candidates_from_items(
    items: &[Item],
    diagnostics: &[Diagnostic],
    candidates: &mut Vec<CoreCandidate>,
) {
    for item in items {
        if let Some(candidate) = core_candidate(item, diagnostics) {
            candidates.push(candidate);
        }
        if let Item::App(app) = item {
            collect_candidates_from_items(&app.items, diagnostics, candidates);
        }
    }
}

fn core_candidate(item: &Item, diagnostics: &[Diagnostic]) -> Option<CoreCandidate> {
    let section = item_sections(item)
        .iter()
        .find(|section| section.name == "does")?;
    let body = core_body::analyze_does_section(section);
    let statements = core_statement_previews(&body.statements);
    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);
    let lowerable_preview_statements = statements
        .iter()
        .filter(|statement| statement.status == "lowerable_preview_v0")
        .count();
    let contextual_preview_statements = statements
        .iter()
        .filter(|statement| statement.status == "contextual_preview_v0")
        .count();
    let blocked_statements = statements
        .iter()
        .filter(|statement| statement.status == "blocked_v0")
        .count();
    let expression_previews = statements
        .iter()
        .filter(|statement| statement.expression_preview.is_some())
        .count();
    let expression_atoms = statements
        .iter()
        .filter_map(|statement| statement.expression_preview.as_ref())
        .map(|expression| expression.atoms.len())
        .sum();
    let expression_ast_nodes = statements
        .iter()
        .filter_map(|statement| statement.expression_preview.as_ref())
        .map(|expression| expression.ast.node_count)
        .sum();
    let compound_expression_previews = statements
        .iter()
        .filter_map(|statement| statement.expression_preview.as_ref())
        .filter(|expression| expression.status == "compound_preview_v0")
        .count();
    let id = preview_id(item);
    let block_preview = core_block_preview(&id, &statements);
    let block_status = block_preview.status;
    let block_count = block_preview.block_count;
    let max_block_depth = block_preview.max_depth;
    let unmatched_block_closes = block_preview.unmatched_closes;
    let unclosed_blocks = block_preview.unclosed_blocks;

    Some(CoreCandidate {
        id,
        kind: item.kind(),
        name: item.name().to_string(),
        span: portable_span(item.span()),
        status: candidate_status(
            has_errors,
            &body,
            lowerable_preview_statements,
            contextual_preview_statements,
            blocked_statements,
        ),
        body_status: body.status,
        grammar_status: body.grammar_status,
        meaningful_lines: body.meaningful_lines,
        lowerable_preview_statements,
        contextual_preview_statements,
        blocked_statements,
        expression_previews,
        expression_atoms,
        expression_ast_nodes,
        compound_expression_previews,
        block_status,
        block_count,
        max_block_depth,
        unmatched_block_closes,
        unclosed_blocks,
        source_sections: item_sections(item)
            .iter()
            .map(|section| section.name.clone())
            .collect(),
        block_preview,
        statements,
    })
}

fn core_statement_previews(statements: &[BodyStatement]) -> Vec<CoreStatementPreview> {
    let mut previews = Vec::new();
    let mut in_record_literal = false;

    for statement in statements {
        let mut preview = core_statement_preview(statement);
        if in_record_literal && statement.kind == "block_close" {
            preview.core_operation = "record_construction_close";
            preview.status = "contextual_preview_v0";
            preview.reason = Some("record_literal_context_required");
            in_record_literal = false;
        }
        if statement.expression_kind == Some("record_literal_start") {
            in_record_literal = true;
        }
        previews.push(preview);
    }

    previews
}
fn core_statement_preview(statement: &BodyStatement) -> CoreStatementPreview {
    let (core_operation, status, fallback_reason) = match statement.kind {
        "return" => ("return", "lowerable_preview_v0", None),
        "fail" => ("fail", "lowerable_preview_v0", None),
        "let_binding" => ("let_binding", "lowerable_preview_v0", None),
        "mutable_binding" => ("mutable_binding", "lowerable_preview_v0", None),
        "set_place" => ("set_place", "lowerable_preview_v0", None),
        "if_header" => ("if_statement", "lowerable_preview_v0", None),
        "while_header" => ("while_loop", "lowerable_preview_v0", None),
        "for_each_header" => ("for_each", "lowerable_preview_v0", None),
        "for_index_header" => ("for_index", "lowerable_preview_v0", None),
        "loop_header" => ("loop", "lowerable_preview_v0", None),
        "block_close" => ("block_close", "lowerable_preview_v0", None),
        "record_field_initializer" => (
            "record_construction_field",
            "contextual_preview_v0",
            Some("record_literal_context_required"),
        ),
        "nested_intent_header" => (
            "contract_context",
            "contextual_preview_v0",
            Some("nested_intent_lowering_not_implemented"),
        ),
        "test_expectation" => (
            "test_expectation",
            "contextual_preview_v0",
            Some("test_body_not_core_runtime"),
        ),
        "save_in_store" => (
            "store_write_deferred",
            "blocked_v0",
            Some("surface_save_requires_store_lowering"),
        ),
        _ => ("unknown", "blocked_v0", Some("not_in_core_preview_v0")),
    };

    let status = if statement.status == "unsupported_v0" {
        "blocked_v0"
    } else {
        status
    };
    let expression_preview =
        expression_text_for_statement(statement).map(core_expr::analyze_expression);

    CoreStatementPreview {
        span: portable_span(&statement.span),
        text: statement.text.clone(),
        source_kind: statement.kind,
        source_status: statement.status,
        core_operation,
        status,
        expression_kind: statement.expression_kind,
        expression_preview,
        reason: statement.reason.or(fallback_reason),
    }
}

fn expression_text_for_statement(statement: &BodyStatement) -> Option<&str> {
    match statement.kind {
        "return" => strip_keyword(&statement.text, "return"),
        "fail" => strip_keyword(&statement.text, "fail"),
        "if_header" => header_body(&statement.text, "if"),
        "while_header" => header_body(&statement.text, "while"),
        "for_each_header" => for_each_collection(&statement.text),
        "for_index_header" => header_body(&statement.text, "for index"),
        "let_binding" | "mutable_binding" | "set_place" => statement
            .text
            .split_once('=')
            .map(|(_left, expression)| expression.trim()),
        "record_field_initializer" => statement
            .text
            .split_once(':')
            .map(|(_field, expression)| expression.trim()),
        "test_expectation" => strip_keyword(&statement.text, "expect"),
        _ => None,
    }
}

fn header_body<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = strip_keyword(text, keyword)?;
    rest.strip_suffix('{').map(str::trim)
}

fn for_each_collection(text: &str) -> Option<&str> {
    let body = header_body(text, "for each")?;
    body.split_once(" in ")
        .map(|(_binding, collection)| collection.trim())
}

fn strip_keyword<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    if text == keyword {
        return Some("");
    }
    text.strip_prefix(keyword)
        .and_then(|rest| rest.strip_prefix(char::is_whitespace))
        .map(str::trim)
}

fn core_block_preview(candidate_id: &str, statements: &[CoreStatementPreview]) -> CoreBlockPreview {
    let mut cursor = 0usize;
    let mut serial = 0usize;
    let mut root = parse_block_node(
        candidate_id,
        "root",
        None,
        None,
        statements,
        &mut cursor,
        &mut serial,
    );
    let block_count = count_block_nodes(&root);
    let max_depth = max_block_depth(&root, 0);
    let unmatched_closes = count_unmatched_closes(&root);
    let unclosed_blocks = count_unclosed_blocks(&root);
    let status = if unmatched_closes > 0 || unclosed_blocks > 0 {
        "block_preview_with_mismatch_v0"
    } else {
        "block_preview_v0"
    };
    root.status = status;

    CoreBlockPreview {
        status,
        block_count,
        max_depth,
        unmatched_closes,
        unclosed_blocks,
        root,
    }
}

fn parse_block_node(
    candidate_id: &str,
    block_kind: &'static str,
    header_statement_index: Option<usize>,
    reason: Option<&'static str>,
    statements: &[CoreStatementPreview],
    cursor: &mut usize,
    serial: &mut usize,
) -> CoreBlockNode {
    let id = next_block_id(candidate_id, block_kind, header_statement_index, serial);
    let mut node = CoreBlockNode {
        id,
        block_kind,
        status: block_status(block_kind, true),
        header_statement_index,
        closing_statement_index: None,
        children: Vec::new(),
        reason,
    };

    while *cursor < statements.len() {
        let statement_index = *cursor;
        let statement = &statements[statement_index];
        if statement.source_kind == "block_close" {
            if block_kind == "root" {
                node.children
                    .push(CoreBlockChild::Statement(CoreBlockStatementRef {
                        statement_index,
                        core_operation: statement.core_operation,
                        status: "unmatched_block_close_v0",
                        reason: Some("unmatched_block_close"),
                    }));
                *cursor += 1;
                continue;
            }

            node.closing_statement_index = Some(statement_index);
            *cursor += 1;
            return node;
        }

        if let Some((child_kind, child_reason)) = opened_block_kind(statement) {
            *cursor += 1;
            let child = parse_block_node(
                candidate_id,
                child_kind,
                Some(statement_index),
                child_reason,
                statements,
                cursor,
                serial,
            );
            node.children.push(CoreBlockChild::Block(child));
        } else {
            node.children
                .push(CoreBlockChild::Statement(CoreBlockStatementRef {
                    statement_index,
                    core_operation: statement.core_operation,
                    status: statement.status,
                    reason: statement.reason,
                }));
            *cursor += 1;
        }
    }

    if block_kind != "root" {
        node.status = "unclosed_block_preview_v0";
        node.reason = Some("block_close_missing");
    }
    node
}

fn opened_block_kind(
    statement: &CoreStatementPreview,
) -> Option<(&'static str, Option<&'static str>)> {
    match statement.core_operation {
        "if_statement" => Some(("if_statement", None)),
        "while_loop" => Some(("while_loop", None)),
        "for_each" => Some(("for_each", None)),
        "for_index" => Some(("for_index", None)),
        "loop" => Some(("loop", None)),
        "let_binding" if statement.expression_kind == Some("record_literal_start") => Some((
            "record_construction",
            Some("record_literal_context_required"),
        )),
        _ => None,
    }
}

fn next_block_id(
    candidate_id: &str,
    block_kind: &str,
    header_statement_index: Option<usize>,
    serial: &mut usize,
) -> String {
    if block_kind == "root" {
        return format!("{candidate_id}_block_root");
    }
    let current = *serial;
    *serial += 1;
    match header_statement_index {
        Some(index) => format!("{candidate_id}_block_{current}_{block_kind}_{index}"),
        None => format!("{candidate_id}_block_{current}_{block_kind}"),
    }
}

fn block_status(block_kind: &str, closed: bool) -> &'static str {
    if !closed {
        "unclosed_block_preview_v0"
    } else if block_kind == "record_construction" {
        "contextual_block_preview_v0"
    } else {
        "block_preview_v0"
    }
}

fn count_block_nodes(node: &CoreBlockNode) -> usize {
    1 + node
        .children
        .iter()
        .filter_map(|child| match child {
            CoreBlockChild::Block(block) => Some(count_block_nodes(block)),
            CoreBlockChild::Statement(_) => None,
        })
        .sum::<usize>()
}

fn max_block_depth(node: &CoreBlockNode, depth: usize) -> usize {
    node.children
        .iter()
        .filter_map(|child| match child {
            CoreBlockChild::Block(block) => Some(max_block_depth(block, depth + 1)),
            CoreBlockChild::Statement(_) => None,
        })
        .max()
        .unwrap_or(depth)
}

fn count_unmatched_closes(node: &CoreBlockNode) -> usize {
    let local = node
        .children
        .iter()
        .filter(|child| {
            matches!(
                child,
                CoreBlockChild::Statement(statement)
                    if statement.status == "unmatched_block_close_v0"
            )
        })
        .count();
    local
        + node
            .children
            .iter()
            .filter_map(|child| match child {
                CoreBlockChild::Block(block) => Some(count_unmatched_closes(block)),
                CoreBlockChild::Statement(_) => None,
            })
            .sum::<usize>()
}

fn count_unclosed_blocks(node: &CoreBlockNode) -> usize {
    usize::from(node.status == "unclosed_block_preview_v0")
        + node
            .children
            .iter()
            .filter_map(|child| match child {
                CoreBlockChild::Block(block) => Some(count_unclosed_blocks(block)),
                CoreBlockChild::Statement(_) => None,
            })
            .sum::<usize>()
}

fn candidate_status(
    has_errors: bool,
    body: &BodyGrammarReport,
    lowerable_preview_statements: usize,
    contextual_preview_statements: usize,
    blocked_statements: usize,
) -> &'static str {
    if has_errors {
        "blocked_by_source_errors"
    } else if body.meaningful_lines == 0 {
        "empty_body"
    } else if blocked_statements > 0 {
        "preview_with_blockers"
    } else if lowerable_preview_statements > 0 && contextual_preview_statements == 0 {
        "lowerable_preview_v0"
    } else {
        "contextual_preview_v0"
    }
}

impl CorePreviewReport {
    fn lowerable_preview_statements(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.lowerable_preview_statements)
            .sum()
    }

    fn contextual_preview_statements(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.contextual_preview_statements)
            .sum()
    }

    fn blocked_statements(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.blocked_statements)
            .sum()
    }

    fn expression_previews(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.expression_previews)
            .sum()
    }

    fn expression_atoms(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.expression_atoms)
            .sum()
    }

    fn expression_ast_nodes(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.expression_ast_nodes)
            .sum()
    }

    fn compound_expression_previews(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.compound_expression_previews)
            .sum()
    }

    fn block_count(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.block_count)
            .sum()
    }

    fn max_block_depth(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.max_block_depth)
            .max()
            .unwrap_or(0)
    }

    fn unmatched_block_closes(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.unmatched_block_closes)
            .sum()
    }

    fn unclosed_blocks(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.unclosed_blocks)
            .sum()
    }
}

fn item_sections(item: &Item) -> &[Section] {
    match item {
        Item::App(item) => &item.sections,
        Item::Type(item) => &item.sections,
        Item::Store(item) => &item.sections,
        Item::Task(item) => &item.sections,
        Item::Test(item) => &item.sections,
    }
}

fn count_items(program: &Program) -> usize {
    program
        .files
        .iter()
        .map(|file| count_items_slice(&file.items))
        .sum()
}

fn count_items_slice(items: &[Item]) -> usize {
    items
        .iter()
        .map(|item| match item {
            Item::App(app) => 1 + count_items_slice(&app.items),
            _ => 1,
        })
        .sum()
}

fn count_kind(program: &Program, kind: &str) -> usize {
    program
        .files
        .iter()
        .map(|file| count_kind_slice(&file.items, kind))
        .sum()
}

fn count_kind_slice(items: &[Item], kind: &str) -> usize {
    items
        .iter()
        .map(|item| {
            let nested = match item {
                Item::App(app) => count_kind_slice(&app.items, kind),
                _ => 0,
            };
            usize::from(item.kind() == kind) + nested
        })
        .sum()
}

fn preview_id(item: &Item) -> String {
    prefixed_id(
        "hum_core_preview",
        &format!(
            "{}_{}_{}_{}",
            item.kind(),
            item.name(),
            item.span().line,
            item.span().column
        ),
    )
}

fn prefixed_id(prefix: &str, text: &str) -> String {
    let mut body = snake_identifier(text);
    if body.len() < 4 {
        body.push_str("_item");
    }
    if body.len() > 96 {
        body.truncate(96);
        body = body.trim_matches('_').to_string();
    }
    format!("{prefix}_{body}")
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
    out.trim_matches('_').to_string()
}

fn portable_span(span: &Span) -> Span {
    Span {
        file: span.file.replace('\\', "/"),
        line: span.line,
        column: span.column,
    }
}

fn push_summary(out: &mut String, report: &CorePreviewReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"core_candidates\": {}, \"execution_ready\": 0, \"errors\": {}, \"warnings\": {}, \"lowerable_preview_statements\": {}, \"contextual_preview_statements\": {}, \"blocked_statements\": {}, \"expression_previews\": {}, \"expression_atoms\": {}, \"expression_ast_nodes\": {}, \"compound_expression_previews\": {}, \"block_count\": {}, \"max_block_depth\": {}, \"unmatched_block_closes\": {}, \"unclosed_blocks\": {}",
        report.files,
        report.items,
        report.tasks,
        report.tests,
        report.candidates.len(),
        report.errors,
        report.warnings,
        report.lowerable_preview_statements(),
        report.contextual_preview_statements(),
        report.blocked_statements(),
        report.expression_previews(),
        report.expression_atoms(),
        report.expression_ast_nodes(),
        report.compound_expression_previews(),
        report.block_count(),
        report.max_block_depth(),
        report.unmatched_block_closes(),
        report.unclosed_blocks()
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_candidates(out: &mut String, candidates: &[CoreCandidate], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"core_candidates\": [\n");
    for (index, candidate) in candidates.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_candidate(out, candidate, indent + 2);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_candidate(out: &mut String, candidate: &CoreCandidate, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &candidate.id, true);
    push_string_field(out, indent + 2, "kind", candidate.kind, true);
    push_string_field(out, indent + 2, "name", &candidate.name, true);
    push_span_field(out, indent + 2, "source_span", &candidate.span, true);
    push_string_field(out, indent + 2, "status", candidate.status, true);
    push_string_field(
        out,
        indent + 2,
        "core_contract_schema",
        core_contract::CORE_CONTRACT_SCHEMA,
        true,
    );
    push_string_field(out, indent + 2, "body_status", candidate.body_status, true);
    push_string_field(
        out,
        indent + 2,
        "grammar_status",
        candidate.grammar_status,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "block_status",
        candidate.block_status,
        true,
    );
    push_indent(out, indent + 2);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"meaningful_lines\": {}, \"lowerable_preview_statements\": {}, \"contextual_preview_statements\": {}, \"blocked_statements\": {}, \"expression_previews\": {}, \"expression_atoms\": {}, \"expression_ast_nodes\": {}, \"compound_expression_previews\": {}, \"block_count\": {}, \"max_block_depth\": {}, \"unmatched_block_closes\": {}, \"unclosed_blocks\": {}",
        candidate.meaningful_lines,
        candidate.lowerable_preview_statements,
        candidate.contextual_preview_statements,
        candidate.blocked_statements,
        candidate.expression_previews,
        candidate.expression_atoms,
        candidate.expression_ast_nodes,
        candidate.compound_expression_previews,
        candidate.block_count,
        candidate.max_block_depth,
        candidate.unmatched_block_closes,
        candidate.unclosed_blocks
    ));
    out.push_str("},\n");
    push_owned_string_array(
        out,
        indent + 2,
        "source_sections",
        &candidate.source_sections,
        true,
    );
    push_block_preview(out, indent + 2, &candidate.block_preview, true);
    push_statements(out, indent + 2, &candidate.statements, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_statements(
    out: &mut String,
    indent: usize,
    statements: &[CoreStatementPreview],
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"statements\": [\n");
    for (index, statement) in statements.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_statement(out, indent + 2, statement);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_statement(out: &mut String, indent: usize, statement: &CoreStatementPreview) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_span_field(out, indent + 2, "source_span", &statement.span, true);
    push_string_field(out, indent + 2, "text", &statement.text, true);
    push_string_field(out, indent + 2, "source_kind", statement.source_kind, true);
    push_string_field(
        out,
        indent + 2,
        "source_status",
        statement.source_status,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "core_operation",
        statement.core_operation,
        true,
    );
    push_string_field(out, indent + 2, "status", statement.status, true);
    push_optional_string_field(
        out,
        indent + 2,
        "expression_kind",
        statement.expression_kind,
        true,
    );
    push_expression_preview_field(out, indent + 2, statement.expression_preview.as_ref(), true);
    push_optional_string_field(out, indent + 2, "reason", statement.reason, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_expression_preview_field(
    out: &mut String,
    indent: usize,
    expression: Option<&CoreExpressionPreview>,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "expression_preview");
    out.push_str(": ");
    match expression {
        Some(expression) => push_expression_preview(out, indent, expression),
        None => out.push_str("null"),
    }
    push_comma_newline(out, comma);
}

fn push_expression_preview(out: &mut String, indent: usize, expression: &CoreExpressionPreview) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "text", &expression.text, true);
    push_string_field(out, indent + 2, "kind", expression.kind, true);
    push_string_field(out, indent + 2, "status", expression.status, true);
    push_expression_atoms(out, indent + 2, &expression.atoms, true);
    push_string_slice_array(out, indent + 2, "operators", &expression.operators, true);
    push_expression_ast(out, indent + 2, &expression.ast, true);
    push_optional_string_field(out, indent + 2, "reason", expression.reason, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_expression_atoms(out: &mut String, indent: usize, atoms: &[ExpressionAtom], comma: bool) {
    push_indent(out, indent);
    out.push_str("\"atoms\": [");
    for (index, atom) in atoms.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str("\"text\": ");
        push_json_string(out, &atom.text);
        out.push_str(", \"kind\": ");
        push_json_string(out, atom.kind);
        out.push_str(", \"status\": ");
        push_json_string(out, atom.status);
        out.push('}');
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_string_slice_array(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[&str],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(out, value);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_block_preview(out: &mut String, indent: usize, preview: &CoreBlockPreview, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "block_preview");
    out.push_str(": {\n");
    push_string_field(out, indent + 2, "status", preview.status, true);
    push_usize_field(out, indent + 2, "block_count", preview.block_count, true);
    push_usize_field(out, indent + 2, "max_depth", preview.max_depth, true);
    push_usize_field(
        out,
        indent + 2,
        "unmatched_closes",
        preview.unmatched_closes,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unclosed_blocks",
        preview.unclosed_blocks,
        true,
    );
    push_indent(out, indent + 2);
    push_json_string(out, "root");
    out.push_str(": ");
    push_block_node(out, indent + 2, &preview.root);
    out.push('\n');
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_block_node(out: &mut String, indent: usize, node: &CoreBlockNode) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "node_kind", "block", true);
    push_string_field(out, indent + 2, "id", &node.id, true);
    push_string_field(out, indent + 2, "block_kind", node.block_kind, true);
    push_string_field(out, indent + 2, "status", node.status, true);
    push_optional_usize_field(
        out,
        indent + 2,
        "header_statement_index",
        node.header_statement_index,
        true,
    );
    push_optional_usize_field(
        out,
        indent + 2,
        "closing_statement_index",
        node.closing_statement_index,
        true,
    );
    push_optional_string_field(out, indent + 2, "reason", node.reason, true);
    push_block_children(out, indent + 2, &node.children, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_block_children(out: &mut String, indent: usize, children: &[CoreBlockChild], comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "children");
    out.push_str(": [");
    if !children.is_empty() {
        out.push('\n');
        for (index, child) in children.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            push_block_child(out, indent + 2, child);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_block_child(out: &mut String, indent: usize, child: &CoreBlockChild) {
    match child {
        CoreBlockChild::Statement(statement) => push_block_statement_ref(out, indent, statement),
        CoreBlockChild::Block(block) => push_block_node(out, indent, block),
    }
}

fn push_block_statement_ref(out: &mut String, indent: usize, statement: &CoreBlockStatementRef) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "node_kind", "statement_ref", true);
    push_usize_field(
        out,
        indent + 2,
        "statement_index",
        statement.statement_index,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "core_operation",
        statement.core_operation,
        true,
    );
    push_string_field(out, indent + 2, "status", statement.status, true);
    push_optional_string_field(out, indent + 2, "reason", statement.reason, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_expression_ast(
    out: &mut String,
    indent: usize,
    ast: &CoreExpressionAstPreview,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "ast");
    out.push_str(": {\n");
    push_string_field(out, indent + 2, "status", ast.status, true);
    push_string_field(out, indent + 2, "type_status", ast.type_status, true);
    push_string_field(out, indent + 2, "effect_status", ast.effect_status, true);
    push_usize_field(out, indent + 2, "node_count", ast.node_count, true);
    push_indent(out, indent + 2);
    push_json_string(out, "root");
    out.push_str(": ");
    push_expression_node(out, indent + 2, &ast.root);
    out.push('\n');
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_expression_node(out: &mut String, indent: usize, node: &CoreExpressionNode) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &node.id, true);
    push_string_field(out, indent + 2, "form", node.form, true);
    push_string_field(out, indent + 2, "text", &node.text, true);
    push_optional_string_field(out, indent + 2, "operator", node.operator, true);
    push_string_field(out, indent + 2, "type_status", node.type_status, true);
    push_string_field(out, indent + 2, "effect_status", node.effect_status, true);
    push_optional_string_field(out, indent + 2, "reason", node.reason, true);
    push_expression_node_children(out, indent + 2, &node.children, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_expression_node_children(
    out: &mut String,
    indent: usize,
    children: &[CoreExpressionNode],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "children");
    out.push_str(": [");
    if !children.is_empty() {
        out.push('\n');
        for (index, child) in children.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            push_expression_node(out, indent + 2, child);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_optional_usize_field(
    out: &mut String,
    indent: usize,
    key: &str,
    value: Option<usize>,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    match value {
        Some(value) => out.push_str(&value.to_string()),
        None => out.push_str("null"),
    }
    push_comma_newline(out, comma);
}
fn push_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    out.push_str(&value.to_string());
    push_comma_newline(out, comma);
}
fn push_span_field(out: &mut String, indent: usize, key: &str, span: &Span, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": {");
    out.push_str("\"file\": ");
    push_json_string(out, &span.file);
    out.push_str(&format!(
        ", \"line\": {}, \"column\": {}",
        span.line, span.column
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_owned_string_array(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[String],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(out, value);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_string_array(out: &mut String, indent: usize, key: &str, values: &[&str], comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(out, value);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_optional_string_field(
    out: &mut String,
    indent: usize,
    key: &str,
    value: Option<&str>,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    match value {
        Some(value) => push_json_string(out, value),
        None => out.push_str("null"),
    }
    push_comma_newline(out, comma);
}

fn push_string_field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_json_string(out, value);
    push_comma_newline(out, comma);
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn push_comma_newline(out: &mut String, comma: bool) {
    if comma {
        out.push(',');
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use crate::ast::Program;
    use crate::parser::parse_source;

    use super::{core_preview_json, core_preview_text};

    #[test]
    fn text_preview_lists_core_candidates_without_execution_claims() {
        let program = demo_program();
        let text = core_preview_text(&program, &[]);

        assert!(text.contains("Hum Core preview (hum.core_preview.v0)"));
        assert!(text.contains("core_contract_schema: hum.core_contract.v0"));
        assert!(text.contains("core_candidates=2 execution_ready=0"));
        assert!(text.contains("expression_previews="));
        assert!(text.contains("expression_atoms="));
        assert!(text.contains("expression_ast_nodes="));
        assert!(text.contains("block_status="));
        assert!(text.contains("return -> return"));
        assert!(text.contains("save_in_store -> store_write_deferred"));
    }

    #[test]
    fn json_preview_reports_lowerable_contextual_and_blocked_statements() {
        let program = demo_program();
        let json = core_preview_json(&program, &[]);

        assert!(json.contains("\"schema\": \"hum.core_preview.v0\""));
        assert!(json.contains("\"core_contract_schema\": \"hum.core_contract.v0\""));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"status\": \"preview_with_blockers\""));
        assert!(json.contains("\"source_kind\": \"return\""));
        assert!(json.contains("\"core_operation\": \"return\""));
        assert!(json.contains("\"expression_previews\""));
        assert!(json.contains("\"block_status\": \"block_preview_v0\""));
        assert!(json.contains("\"block_preview\""));
        assert!(json.contains("\"block_kind\": \"record_construction\""));
        assert!(json.contains("\"node_kind\": \"statement_ref\""));
        assert!(json.contains("\"expression_preview\""));
        assert!(json.contains("\"atoms\""));
        assert!(json.contains("\"operators\""));
        assert!(json.contains("\"ast\""));
        assert!(json.contains("\"form\": \"binary_operation_candidate\""));
        assert!(json.contains("\"type_status\": \"not_type_checked_v0\""));
        assert!(json.contains("\"effect_status\": \"not_effect_checked_v0\""));
        assert!(json.contains("\"status\": \"compound_preview_v0\""));
        assert!(json.contains("\"status\": \"lowerable_preview_v0\""));
        assert!(json.contains("\"source_kind\": \"record_field_initializer\""));
        assert!(json.contains("\"core_operation\": \"record_construction_close\""));
        assert!(json.contains("\"status\": \"contextual_preview_v0\""));
        assert!(json.contains("\"core_operation\": \"store_write_deferred\""));
        assert!(json.contains("\"reason\": \"surface_save_requires_store_lowering\""));
        assert!(json.contains("\"no executable semantics\""));
        assert!(json.contains("\"no interpreter\""));
    }

    #[test]
    fn json_preview_reports_nested_block_tree_without_execution_claims() {
        let source = r#"task find session(user: User, sessions: Sessions) -> Session {
  why:
    find a session

  does:
    for each session in sessions {
      if session.user == user {
        return session
      }
    }

    fail SessionError.not_found
}
"#;
        let parsed = parse_source("nested.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = core_preview_json(&program, &[]);

        assert!(json.contains("\"block_preview\""));
        assert!(json.contains("\"block_count\": 3"));
        assert!(json.contains("\"max_block_depth\": 2"));
        assert!(json.contains("\"max_depth\": 2"));
        assert!(json.contains("\"block_kind\": \"for_each\""));
        assert!(json.contains("\"block_kind\": \"if_statement\""));
        assert!(json.contains("\"header_statement_index\": 0"));
        assert!(json.contains("\"closing_statement_index\": 4"));
        assert!(json.contains("\"unmatched_block_closes\": 0"));
        assert!(json.contains("\"unclosed_blocks\": 0"));
        assert!(json.contains("\"no executable semantics\""));
    }
    fn demo_program() -> Program {
        let source = r#"type Task {
  title: Text
}

task add task(title: Text) -> Task {
  why:
    save a task

  does:
    let task = Task {
      title: title
    }
    save task in tasks
    return task
}

test add task unit {
  covers:
    add task returns task

  does:
    expect add task("demo") returns Task
}
"#;
        let parsed = parse_source("demo.hum", source);
        Program {
            files: vec![parsed.file],
        }
    }
}
