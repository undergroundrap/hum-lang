use crate::ast::{Item, Param, Program, Section, SectionLine};
use crate::core_body::{self, BodyGrammarReport, BodyStatement};
use crate::core_contract;
use crate::core_expr::{
    self, CoreExpressionAstPreview, CoreExpressionNode, CoreExpressionPreview, ExpressionAtom,
};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::graph::is_meaningful_line_text;
use crate::type_check::{self, CheckedReturnSummary};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorePreviewReadinessSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub files: usize,
    pub items: usize,
    pub tasks: usize,
    pub tests: usize,
    pub core_candidates: usize,
    pub errors: usize,
    pub warnings: usize,
    pub lowerable_preview_statements: usize,
    pub contextual_preview_statements: usize,
    pub blocked_statements: usize,
    pub expression_previews: usize,
    pub expression_ast_nodes: usize,
    pub typed_expression_previews: usize,
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
    typed_expression_previews: usize,
    block_status: &'static str,
    block_count: usize,
    max_block_depth: usize,
    unmatched_block_closes: usize,
    unclosed_blocks: usize,
    name_status: &'static str,
    name_preview: CoreNamePreview,
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
struct CoreNamePreview {
    status: &'static str,
    scope_model: &'static str,
    scope_id: String,
    checked_resolver_status: &'static str,
    resolver_diagnostic_status: &'static str,
    resolver_diagnostic_count: usize,
    scope_count: usize,
    definition_count: usize,
    reference_count: usize,
    resolved_reference_count: usize,
    unresolved_reference_count: usize,
    external_reference_count: usize,
    shadowed_definition_count: usize,
    scopes: Vec<CoreNameScope>,
    definitions: Vec<CoreNameDefinition>,
    references: Vec<CoreNameReference>,
}

struct CoreNameScope {
    id: String,
    parent_scope_id: Option<String>,
    scope_kind: &'static str,
    block_id: Option<String>,
    header_statement_index: Option<usize>,
    closing_statement_index: Option<usize>,
}

struct CoreNameDefinition {
    id: String,
    name: String,
    normalized_name: String,
    definition_kind: &'static str,
    scope_id: String,
    statement_index: Option<usize>,
    span: Span,
    status: &'static str,
    shadowed_definition_id: Option<String>,
    reason: Option<&'static str>,
}

struct CoreNameReference {
    id: String,
    name: String,
    normalized_name: String,
    reference_kind: &'static str,
    scope_id: String,
    statement_index: Option<usize>,
    span: Span,
    resolution_status: &'static str,
    resolved_definition_id: Option<String>,
    reason: Option<&'static str>,
}

#[derive(Clone)]
struct VisibleDefinition {
    normalized_name: String,
    id: String,
}

struct PendingNameReference {
    name: String,
    reference_kind: &'static str,
    external_if_unresolved: bool,
}

#[derive(Clone, Copy)]
enum DefinitionConflictMode {
    Shadow,
    DuplicateDeclaration,
}

struct NameDefinitionInput<'a> {
    name: &'a str,
    definition_kind: &'static str,
    statement_index: Option<usize>,
    span: &'a Span,
    conflict_mode: DefinitionConflictMode,
}

struct NameReferenceInput<'a> {
    name: &'a str,
    reference_kind: &'static str,
    statement_index: Option<usize>,
    span: &'a Span,
    external_if_unresolved: bool,
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
        "summary: files={} items={} tasks={} tests={} core_candidates={} execution_ready=0 errors={} warnings={} lowerable_preview_statements={} contextual_preview_statements={} blocked_statements={} expression_previews={} expression_atoms={} expression_ast_nodes={} compound_expression_previews={} typed_expression_previews={} block_count={} max_block_depth={} unmatched_block_closes={} unclosed_blocks={} name_definitions={} name_references={} resolved_name_references={} unresolved_name_references={} external_name_references={} shadowed_name_definitions={}\n",
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
        report.typed_expression_previews(),
        report.block_count(),
        report.max_block_depth(),
        report.unmatched_block_closes(),
        report.unclosed_blocks(),
        report.name_definitions(),
        report.name_references(),
        report.resolved_name_references(),
        report.unresolved_name_references(),
        report.external_name_references(),
        report.shadowed_name_definitions()
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
            "    body: {} grammar={} block_status={} name_status={} meaningful_lines={} lowerable_preview_statements={} contextual_preview_statements={} blocked_statements={} expression_previews={} expression_atoms={} expression_ast_nodes={} compound_expression_previews={} typed_expression_previews={} block_count={} max_block_depth={} unmatched_block_closes={} unclosed_blocks={} name_definitions={} name_references={} resolved_name_references={} unresolved_name_references={} external_name_references={} shadowed_name_definitions={}\n",
            candidate.body_status,
            candidate.grammar_status,
            candidate.block_status,
            candidate.name_status,
            candidate.meaningful_lines,
            candidate.lowerable_preview_statements,
            candidate.contextual_preview_statements,
            candidate.blocked_statements,
            candidate.expression_previews,
            candidate.expression_atoms,
            candidate.expression_ast_nodes,
            candidate.compound_expression_previews,
            candidate.typed_expression_previews,
            candidate.block_count,
            candidate.max_block_depth,
            candidate.unmatched_block_closes,
            candidate.unclosed_blocks,
            candidate.name_preview.definition_count,
            candidate.name_preview.reference_count,
            candidate.name_preview.resolved_reference_count,
            candidate.name_preview.unresolved_reference_count,
            candidate.name_preview.external_reference_count,
            candidate.name_preview.shadowed_definition_count
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

pub fn core_preview_readiness_summary(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> CorePreviewReadinessSummary {
    let report = build_report(program, diagnostics);
    CorePreviewReadinessSummary {
        schema: CORE_PREVIEW_SCHEMA,
        status: CORE_PREVIEW_STATUS,
        files: report.files,
        items: report.items,
        tasks: report.tasks,
        tests: report.tests,
        core_candidates: report.candidates.len(),
        errors: report.errors,
        warnings: report.warnings,
        lowerable_preview_statements: report.lowerable_preview_statements(),
        contextual_preview_statements: report.contextual_preview_statements(),
        blocked_statements: report.blocked_statements(),
        expression_previews: report.expression_previews(),
        expression_ast_nodes: report.expression_ast_nodes(),
        typed_expression_previews: report.typed_expression_previews(),
    }
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
    push_string_field(
        &mut out,
        2,
        "type_check_schema",
        type_check::TYPE_CHECK_SCHEMA,
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
            "no independent type checking",
            "no broad expression type checking",
            "no effect checking",
            "no interpreter",
            "no backend IR",
            "no generated artifact",
            "no safety proof",
            "no module or global name resolution",
            "no checked name resolution",
        ],
        false,
    );
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> CorePreviewReport {
    let mut candidates = Vec::new();
    let checked_returns = type_check::checked_return_summaries(program, diagnostics);
    for file in &program.files {
        collect_candidates_from_items(&file.items, diagnostics, &checked_returns, &mut candidates);
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
    checked_returns: &[CheckedReturnSummary],
    candidates: &mut Vec<CoreCandidate>,
) {
    for item in items {
        if let Some(candidate) = core_candidate(item, diagnostics, checked_returns) {
            candidates.push(candidate);
        }
        if let Item::App(app) = item {
            collect_candidates_from_items(&app.items, diagnostics, checked_returns, candidates);
        }
    }
}

fn core_candidate(
    item: &Item,
    diagnostics: &[Diagnostic],
    checked_returns: &[CheckedReturnSummary],
) -> Option<CoreCandidate> {
    let section = item_sections(item)
        .iter()
        .find(|section| section.name == "does")?;
    let body = core_body::analyze_does_section(section);
    let statements = core_statement_previews(item, &body.statements, checked_returns);
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
    let typed_expression_previews = statements
        .iter()
        .filter_map(|statement| statement.expression_preview.as_ref())
        .filter(|expression| expression.ast.type_status != core_expr::CORE_EXPRESSION_TYPE_STATUS)
        .count();
    let id = preview_id(item);
    let block_preview = core_block_preview(&id, &statements);
    let block_status = block_preview.status;
    let block_count = block_preview.block_count;
    let max_block_depth = block_preview.max_depth;
    let unmatched_block_closes = block_preview.unmatched_closes;
    let unclosed_blocks = block_preview.unclosed_blocks;
    let name_preview = core_name_preview(item, &id, &statements, &block_preview);
    let name_status = name_preview.status;

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
        typed_expression_previews,
        block_status,
        block_count,
        max_block_depth,
        unmatched_block_closes,
        unclosed_blocks,
        name_status,
        name_preview,
        source_sections: item_sections(item)
            .iter()
            .map(|section| section.name.clone())
            .collect(),
        block_preview,
        statements,
    })
}

fn core_statement_previews(
    item: &Item,
    statements: &[BodyStatement],
    checked_returns: &[CheckedReturnSummary],
) -> Vec<CoreStatementPreview> {
    let mut previews = Vec::new();
    let mut in_record_literal = false;

    for statement in statements {
        let mut preview = core_statement_preview(statement);
        if let Some(checked_return) = checked_return_for_statement(item, statement, checked_returns)
        {
            annotate_return_expression_type(&mut preview, checked_return);
        }
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

fn checked_return_for_statement<'a>(
    item: &Item,
    statement: &BodyStatement,
    checked_returns: &'a [CheckedReturnSummary],
) -> Option<&'a CheckedReturnSummary> {
    if item.kind() != "task" || statement.kind != "return" {
        return None;
    }
    let expression_text = strip_keyword(&statement.text, "return")?.trim();
    let span = portable_span(&statement.span);
    checked_returns.iter().find(|checked_return| {
        checked_return.owner_kind == "task"
            && checked_return.owner_name == item.name()
            && checked_return.source_span.file == span.file
            && checked_return.source_span.line == span.line
            && checked_return.source_span.column == span.column
            && checked_return.expression_text == expression_text
    })
}

fn annotate_return_expression_type(
    preview: &mut CoreStatementPreview,
    checked_return: &CheckedReturnSummary,
) {
    let Some(expression) = preview.expression_preview.as_mut() else {
        return;
    };
    let Some(actual_type) = checked_return.actual_type.as_deref() else {
        return;
    };
    let Some(type_status) = checked_return_type_status(checked_return.status) else {
        return;
    };
    core_expr::annotate_expression_type(
        expression,
        type_status,
        Some(actual_type),
        checked_return.type_source,
    );
}

fn checked_return_type_status(status: &str) -> Option<&'static str> {
    match status {
        "accepted_return_expression_v0" => {
            Some(core_expr::CORE_EXPRESSION_CHECKED_TRIVIAL_RETURN_TYPE_STATUS)
        }
        "rejected_return_type_mismatch_v0" => {
            Some(core_expr::CORE_EXPRESSION_CHECKED_TRIVIAL_RETURN_MISMATCH_STATUS)
        }
        _ => None,
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
struct NamePreviewContext<'a> {
    candidate_id: &'a str,
    statements: &'a [CoreStatementPreview],
    definitions: Vec<CoreNameDefinition>,
    references: Vec<CoreNameReference>,
    scopes: Vec<CoreNameScope>,
    visible: Vec<VisibleDefinition>,
    definition_serial: usize,
    reference_serial: usize,
    scope_serial: usize,
}

impl<'a> NamePreviewContext<'a> {
    fn new(candidate_id: &'a str, statements: &'a [CoreStatementPreview]) -> Self {
        Self {
            candidate_id,
            statements,
            definitions: Vec::new(),
            references: Vec::new(),
            scopes: Vec::new(),
            visible: Vec::new(),
            definition_serial: 0,
            reference_serial: 0,
            scope_serial: 0,
        }
    }

    fn build(mut self, item: &Item, block_preview: &CoreBlockPreview) -> CoreNamePreview {
        let root_scope_id = format!("{}_scope_root", self.candidate_id);
        self.add_scope(CoreNameScope {
            id: root_scope_id.clone(),
            parent_scope_id: None,
            scope_kind: "root",
            block_id: Some(block_preview.root.id.clone()),
            header_statement_index: None,
            closing_statement_index: None,
        });

        for param in item_params(item) {
            self.add_definition(
                &root_scope_id,
                NameDefinitionInput {
                    name: &param.name,
                    definition_kind: "parameter",
                    statement_index: None,
                    span: &param.span,
                    conflict_mode: DefinitionConflictMode::Shadow,
                },
            );
        }

        for (definition_kind, line) in declared_name_lines(item) {
            if let Some(name) = declared_name_from_line(&line.text) {
                self.add_definition(
                    &root_scope_id,
                    NameDefinitionInput {
                        name: &name,
                        definition_kind,
                        statement_index: None,
                        span: &line.span,
                        conflict_mode: DefinitionConflictMode::DuplicateDeclaration,
                    },
                );
            }
        }

        self.walk_block(&root_scope_id, &block_preview.root);
        self.finish(root_scope_id)
    }

    fn finish(self, root_scope_id: String) -> CoreNamePreview {
        let definition_count = self.definitions.len();
        let reference_count = self.references.len();
        let resolved_reference_count = self
            .references
            .iter()
            .filter(|reference| reference.resolution_status == "resolved_preview_v0")
            .count();
        let unresolved_reference_count = self
            .references
            .iter()
            .filter(|reference| reference.resolution_status == "unresolved_preview_v0")
            .count();
        let external_reference_count = self
            .references
            .iter()
            .filter(|reference| reference.resolution_status == "external_reference_preview_v0")
            .count();
        let shadowed_definition_count = self
            .definitions
            .iter()
            .filter(|definition| definition.status == "shadowed_definition_preview_v0")
            .count();
        let status = if unresolved_reference_count > 0 {
            "name_preview_with_unresolved_v0"
        } else if shadowed_definition_count > 0 {
            "name_preview_with_shadowing_v0"
        } else {
            "name_preview_v0"
        };

        CoreNamePreview {
            status,
            scope_model: "lexical_block_scope_preview_v0",
            scope_id: root_scope_id,
            checked_resolver_status: "not_run_v0",
            resolver_diagnostic_status: "preview_facts_only_v0",
            resolver_diagnostic_count: 0,
            scope_count: self.scopes.len(),
            definition_count,
            reference_count,
            resolved_reference_count,
            unresolved_reference_count,
            external_reference_count,
            shadowed_definition_count,
            scopes: self.scopes,
            definitions: self.definitions,
            references: self.references,
        }
    }

    fn walk_block(&mut self, scope_id: &str, block: &CoreBlockNode) {
        if block.block_kind == "root" {
            self.walk_block_children(scope_id, block);
            return;
        }

        if let Some(header_statement_index) = block.header_statement_index {
            self.process_statement_references(scope_id, header_statement_index);
        }

        if lexical_block_scope_kind(block.block_kind) {
            let child_scope_id = self.next_scope_id(block);
            let visible_len = self.visible.len();
            self.add_scope(CoreNameScope {
                id: child_scope_id.clone(),
                parent_scope_id: Some(scope_id.to_string()),
                scope_kind: block.block_kind,
                block_id: Some(block.id.clone()),
                header_statement_index: block.header_statement_index,
                closing_statement_index: block.closing_statement_index,
            });

            if let Some(header_statement_index) = block.header_statement_index {
                self.process_statement_definition(&child_scope_id, header_statement_index);
            }

            self.walk_block_children(&child_scope_id, block);
            self.visible.truncate(visible_len);
        } else {
            if let Some(header_statement_index) = block.header_statement_index {
                self.process_statement_definition(scope_id, header_statement_index);
            }
            self.walk_block_children(scope_id, block);
        }
    }

    fn walk_block_children(&mut self, scope_id: &str, block: &CoreBlockNode) {
        for child in &block.children {
            match child {
                CoreBlockChild::Statement(statement) => {
                    self.process_statement(scope_id, statement.statement_index);
                }
                CoreBlockChild::Block(block) => self.walk_block(scope_id, block),
            }
        }
    }

    fn process_statement(&mut self, scope_id: &str, statement_index: usize) {
        self.process_statement_references(scope_id, statement_index);
        self.process_statement_definition(scope_id, statement_index);
    }

    fn process_statement_references(&mut self, scope_id: &str, statement_index: usize) {
        let (references, span) = {
            let statement = &self.statements[statement_index];
            (statement_name_references(statement), statement.span.clone())
        };

        for reference in references {
            self.add_reference(
                scope_id,
                NameReferenceInput {
                    name: &reference.name,
                    reference_kind: reference.reference_kind,
                    statement_index: Some(statement_index),
                    span: &span,
                    external_if_unresolved: reference.external_if_unresolved,
                },
            );
        }
    }

    fn process_statement_definition(&mut self, scope_id: &str, statement_index: usize) {
        let (definition, span) = {
            let statement = &self.statements[statement_index];
            (statement_definition(statement), statement.span.clone())
        };

        if let Some((name, definition_kind)) = definition {
            self.add_definition(
                scope_id,
                NameDefinitionInput {
                    name: &name,
                    definition_kind,
                    statement_index: Some(statement_index),
                    span: &span,
                    conflict_mode: DefinitionConflictMode::Shadow,
                },
            );
        }
    }

    fn add_scope(&mut self, scope: CoreNameScope) {
        self.scopes.push(scope);
    }

    fn next_scope_id(&mut self, block: &CoreBlockNode) -> String {
        let current = self.scope_serial;
        self.scope_serial += 1;
        match block.header_statement_index {
            Some(index) => format!(
                "{}_scope_{}_{}_{}",
                self.candidate_id, current, block.block_kind, index
            ),
            None => format!(
                "{}_scope_{}_{}",
                self.candidate_id, current, block.block_kind
            ),
        }
    }

    fn add_definition(&mut self, scope_id: &str, input: NameDefinitionInput<'_>) {
        let name = input.name;
        let definition_kind = input.definition_kind;
        let statement_index = input.statement_index;
        let span = input.span;
        let conflict_mode = input.conflict_mode;
        let normalized_name = name_key(name);
        if normalized_name.is_empty() {
            return;
        }

        let shadowed = visible_definition(&self.visible, &normalized_name);
        let id = format!(
            "{}_def_{}_{}",
            self.candidate_id, self.definition_serial, normalized_name
        );
        self.definition_serial += 1;
        let (status, shadowed_definition_id, reason, push_visible) = match (shadowed, conflict_mode)
        {
            (Some(existing), DefinitionConflictMode::Shadow) => (
                "shadowed_definition_preview_v0",
                Some(existing.id.clone()),
                Some("definition_shadows_visible_name"),
                true,
            ),
            (Some(existing), DefinitionConflictMode::DuplicateDeclaration) => (
                "duplicate_declaration_preview_v0",
                Some(existing.id.clone()),
                Some("declaration_already_available_in_candidate_scope"),
                false,
            ),
            (None, _) => ("defined_preview_v0", None, None, true),
        };

        self.definitions.push(CoreNameDefinition {
            id: id.clone(),
            name: name.trim().to_string(),
            normalized_name: normalized_name.clone(),
            definition_kind,
            scope_id: scope_id.to_string(),
            statement_index,
            span: portable_span(span),
            status,
            shadowed_definition_id,
            reason,
        });

        if push_visible {
            self.visible.push(VisibleDefinition {
                normalized_name,
                id,
            });
        }
    }

    fn add_reference(&mut self, scope_id: &str, input: NameReferenceInput<'_>) {
        let name = input.name;
        let reference_kind = input.reference_kind;
        let statement_index = input.statement_index;
        let span = input.span;
        let external_if_unresolved = input.external_if_unresolved;
        let normalized_name = name_key(name);
        if normalized_name.is_empty() {
            return;
        }

        let resolved = visible_definition(&self.visible, &normalized_name);
        let external_reference = external_if_unresolved || is_external_root(name);
        let (resolution_status, resolved_definition_id, reason) = if let Some(definition) = resolved
        {
            ("resolved_preview_v0", Some(definition.id.clone()), None)
        } else if external_reference {
            (
                "external_reference_preview_v0",
                None,
                Some("global_or_type_name_resolution_not_implemented"),
            )
        } else {
            (
                "unresolved_preview_v0",
                None,
                Some("name_not_in_candidate_scope"),
            )
        };

        self.references.push(CoreNameReference {
            id: format!(
                "{}_ref_{}_{}",
                self.candidate_id, self.reference_serial, normalized_name
            ),
            name: name.trim().to_string(),
            normalized_name,
            reference_kind,
            scope_id: scope_id.to_string(),
            statement_index,
            span: portable_span(span),
            resolution_status,
            resolved_definition_id,
            reason,
        });
        self.reference_serial += 1;
    }
}

fn core_name_preview(
    item: &Item,
    candidate_id: &str,
    statements: &[CoreStatementPreview],
    block_preview: &CoreBlockPreview,
) -> CoreNamePreview {
    NamePreviewContext::new(candidate_id, statements).build(item, block_preview)
}

fn lexical_block_scope_kind(block_kind: &str) -> bool {
    matches!(
        block_kind,
        "if_statement" | "while_loop" | "for_each" | "for_index" | "loop"
    )
}

fn visible_definition<'a>(
    visible: &'a [VisibleDefinition],
    normalized_name: &str,
) -> Option<&'a VisibleDefinition> {
    visible
        .iter()
        .rev()
        .find(|definition| definition.normalized_name == normalized_name)
}
fn item_params(item: &Item) -> Vec<&Param> {
    match item {
        Item::Task(task) => task.params.iter().collect(),
        Item::Test(test) => test.params.iter().collect(),
        _ => Vec::new(),
    }
}

fn declared_name_lines(item: &Item) -> Vec<(&'static str, &SectionLine)> {
    let mut lines = Vec::new();
    for section in item_sections(item) {
        let definition_kind = match section.name.as_str() {
            "uses" => "declared_use",
            "changes" => "declared_change",
            _ => continue,
        };
        for line in &section.lines {
            if is_meaningful_line_text(&line.text) {
                lines.push((definition_kind, line));
            }
        }
    }
    lines
}

fn declared_name_from_line(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Some((root, _field)) = text.split_once('.') {
        return Some(root.trim().to_string());
    }
    Some(text.to_string())
}

fn statement_definition(statement: &CoreStatementPreview) -> Option<(String, &'static str)> {
    match statement.source_kind {
        "let_binding" => binding_name(&statement.text, "let").map(|name| (name, "let_binding")),
        "mutable_binding" => {
            binding_name(&statement.text, "change").map(|name| (name, "mutable_binding"))
        }
        "for_each_header" => {
            for_each_binding(&statement.text).map(|name| (name, "for_each_binding"))
        }
        "for_index_header" => {
            for_index_binding(&statement.text).map(|name| (name, "for_index_binding"))
        }
        _ => None,
    }
}

fn statement_name_references(statement: &CoreStatementPreview) -> Vec<PendingNameReference> {
    let mut references = Vec::new();
    if statement.source_kind == "set_place"
        && let Some(target) = set_target(&statement.text)
    {
        references.push(PendingNameReference {
            name: target,
            reference_kind: "mutation_target",
            external_if_unresolved: false,
        });
    }

    if let Some(expression) = &statement.expression_preview {
        references.extend(expression_name_references(expression));
    }
    references
}

fn expression_name_references(expression: &CoreExpressionPreview) -> Vec<PendingNameReference> {
    let mut references = Vec::new();
    for (index, atom) in expression.atoms.iter().enumerate() {
        if skips_predicate_atom(expression, index) {
            continue;
        }
        match atom.kind {
            "name" => references.push(PendingNameReference {
                name: atom.text.clone(),
                reference_kind: "name_ref",
                external_if_unresolved: false,
            }),
            "path_or_field_read" => {
                if let Some(root) = path_root(&atom.text) {
                    references.push(PendingNameReference {
                        external_if_unresolved: is_external_root(&root),
                        name: root,
                        reference_kind: "path_root_ref",
                    });
                }
            }
            "callee_name" => references.push(PendingNameReference {
                name: atom.text.clone(),
                reference_kind: "callee_ref",
                external_if_unresolved: true,
            }),
            "call_like" => {
                if let Some(callee) = call_callee(&atom.text) {
                    references.push(PendingNameReference {
                        name: callee,
                        reference_kind: "callee_ref",
                        external_if_unresolved: true,
                    });
                }
            }
            _ => {}
        }
    }
    references
}

fn skips_predicate_atom(expression: &CoreExpressionPreview, index: usize) -> bool {
    expression.operators.len() == 1 && expression.operators[0] == "is" && index > 0
}

fn binding_name(text: &str, keyword: &str) -> Option<String> {
    let rest = strip_keyword(text, keyword)?;
    let (left, _right) = rest.split_once('=')?;
    let name = left.split_once(':').map_or(left, |(name, _ty)| name).trim();
    (!name.is_empty()).then(|| name.to_string())
}

fn for_each_binding(text: &str) -> Option<String> {
    let body = header_body(text, "for each")?;
    body.split_once(" in ")
        .map(|(binding, _collection)| binding.trim().to_string())
        .filter(|binding| !binding.is_empty())
}

fn for_index_binding(text: &str) -> Option<String> {
    let body = header_body(text, "for index")?;
    body.split_whitespace().next().map(str::to_string)
}

fn set_target(text: &str) -> Option<String> {
    let rest = strip_keyword(text, "set")?;
    rest.split_once('=')
        .map(|(target, _value)| target.trim().to_string())
        .filter(|target| !target.is_empty())
}

fn path_root(text: &str) -> Option<String> {
    text.split_once('.')
        .map(|(root, _field)| root.trim().to_string())
        .filter(|root| !root.is_empty())
}

fn call_callee(text: &str) -> Option<String> {
    text.split_once('(')
        .map(|(callee, _args)| callee.trim().to_string())
        .filter(|callee| !callee.is_empty())
}

fn is_external_root(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
}

fn name_key(name: &str) -> String {
    snake_identifier(name)
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

    fn typed_expression_previews(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.typed_expression_previews)
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
    fn name_definitions(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.name_preview.definition_count)
            .sum()
    }

    fn name_references(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.name_preview.reference_count)
            .sum()
    }

    fn resolved_name_references(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.name_preview.resolved_reference_count)
            .sum()
    }

    fn unresolved_name_references(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.name_preview.unresolved_reference_count)
            .sum()
    }

    fn external_name_references(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.name_preview.external_reference_count)
            .sum()
    }

    fn shadowed_name_definitions(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.name_preview.shadowed_definition_count)
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
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"core_candidates\": {}, \"execution_ready\": 0, \"errors\": {}, \"warnings\": {}, \"lowerable_preview_statements\": {}, \"contextual_preview_statements\": {}, \"blocked_statements\": {}, \"expression_previews\": {}, \"expression_atoms\": {}, \"expression_ast_nodes\": {}, \"compound_expression_previews\": {}, \"typed_expression_previews\": {}, \"block_count\": {}, \"max_block_depth\": {}, \"unmatched_block_closes\": {}, \"unclosed_blocks\": {}, \"name_definitions\": {}, \"name_references\": {}, \"resolved_name_references\": {}, \"unresolved_name_references\": {}, \"external_name_references\": {}, \"shadowed_name_definitions\": {}",
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
        report.typed_expression_previews(),
        report.block_count(),
        report.max_block_depth(),
        report.unmatched_block_closes(),
        report.unclosed_blocks(),
        report.name_definitions(),
        report.name_references(),
        report.resolved_name_references(),
        report.unresolved_name_references(),
        report.external_name_references(),
        report.shadowed_name_definitions()
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
    push_string_field(out, indent + 2, "name_status", candidate.name_status, true);

    push_indent(out, indent + 2);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"meaningful_lines\": {}, \"lowerable_preview_statements\": {}, \"contextual_preview_statements\": {}, \"blocked_statements\": {}, \"expression_previews\": {}, \"expression_atoms\": {}, \"expression_ast_nodes\": {}, \"compound_expression_previews\": {}, \"typed_expression_previews\": {}, \"block_count\": {}, \"max_block_depth\": {}, \"unmatched_block_closes\": {}, \"unclosed_blocks\": {}, \"name_definitions\": {}, \"name_references\": {}, \"resolved_name_references\": {}, \"unresolved_name_references\": {}, \"external_name_references\": {}, \"shadowed_name_definitions\": {}",
        candidate.meaningful_lines,
        candidate.lowerable_preview_statements,
        candidate.contextual_preview_statements,
        candidate.blocked_statements,
        candidate.expression_previews,
        candidate.expression_atoms,
        candidate.expression_ast_nodes,
        candidate.compound_expression_previews,
        candidate.typed_expression_previews,
        candidate.block_count,
        candidate.max_block_depth,
        candidate.unmatched_block_closes,
        candidate.unclosed_blocks,
        candidate.name_preview.definition_count,
        candidate.name_preview.reference_count,
        candidate.name_preview.resolved_reference_count,
        candidate.name_preview.unresolved_reference_count,
        candidate.name_preview.external_reference_count,
        candidate.name_preview.shadowed_definition_count
    ));
    out.push_str("},\n");
    push_owned_string_array(
        out,
        indent + 2,
        "source_sections",
        &candidate.source_sections,
        true,
    );
    push_name_preview(out, indent + 2, &candidate.name_preview, true);

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
fn push_name_preview(out: &mut String, indent: usize, preview: &CoreNamePreview, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "name_preview");
    out.push_str(": {\n");
    push_string_field(out, indent + 2, "status", preview.status, true);
    push_string_field(out, indent + 2, "scope_model", preview.scope_model, true);
    push_string_field(out, indent + 2, "scope_id", &preview.scope_id, true);
    push_string_field(
        out,
        indent + 2,
        "checked_resolver_status",
        preview.checked_resolver_status,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "resolver_diagnostic_status",
        preview.resolver_diagnostic_status,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_diagnostic_count",
        preview.resolver_diagnostic_count,
        true,
    );
    push_usize_field(out, indent + 2, "scope_count", preview.scope_count, true);
    push_usize_field(
        out,
        indent + 2,
        "definition_count",
        preview.definition_count,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "reference_count",
        preview.reference_count,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolved_reference_count",
        preview.resolved_reference_count,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unresolved_reference_count",
        preview.unresolved_reference_count,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "external_reference_count",
        preview.external_reference_count,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "shadowed_definition_count",
        preview.shadowed_definition_count,
        true,
    );
    push_name_scopes(out, indent + 2, &preview.scopes, true);
    push_name_definitions(out, indent + 2, &preview.definitions, true);
    push_name_references(out, indent + 2, &preview.references, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_name_scopes(out: &mut String, indent: usize, scopes: &[CoreNameScope], comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "scopes");
    out.push_str(": [");
    if !scopes.is_empty() {
        out.push('\n');
        for (index, scope) in scopes.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            push_name_scope(out, indent + 2, scope);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_name_scope(out: &mut String, indent: usize, scope: &CoreNameScope) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &scope.id, true);
    push_optional_string_field(
        out,
        indent + 2,
        "parent_scope_id",
        scope.parent_scope_id.as_deref(),
        true,
    );
    push_string_field(out, indent + 2, "scope_kind", scope.scope_kind, true);
    push_optional_string_field(out, indent + 2, "block_id", scope.block_id.as_deref(), true);
    push_optional_usize_field(
        out,
        indent + 2,
        "header_statement_index",
        scope.header_statement_index,
        true,
    );
    push_optional_usize_field(
        out,
        indent + 2,
        "closing_statement_index",
        scope.closing_statement_index,
        false,
    );
    push_indent(out, indent);
    out.push('}');
}
fn push_name_definitions(
    out: &mut String,
    indent: usize,
    definitions: &[CoreNameDefinition],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "definitions");
    out.push_str(": [");
    if !definitions.is_empty() {
        out.push('\n');
        for (index, definition) in definitions.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            push_name_definition(out, indent + 2, definition);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_name_definition(out: &mut String, indent: usize, definition: &CoreNameDefinition) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &definition.id, true);
    push_string_field(out, indent + 2, "name", &definition.name, true);
    push_string_field(
        out,
        indent + 2,
        "normalized_name",
        &definition.normalized_name,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "definition_kind",
        definition.definition_kind,
        true,
    );
    push_string_field(out, indent + 2, "scope_id", &definition.scope_id, true);
    push_optional_usize_field(
        out,
        indent + 2,
        "statement_index",
        definition.statement_index,
        true,
    );
    push_span_field(out, indent + 2, "source_span", &definition.span, true);
    push_string_field(out, indent + 2, "status", definition.status, true);
    push_optional_string_field(
        out,
        indent + 2,
        "shadowed_definition_id",
        definition.shadowed_definition_id.as_deref(),
        true,
    );
    push_optional_string_field(out, indent + 2, "reason", definition.reason, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_name_references(
    out: &mut String,
    indent: usize,
    references: &[CoreNameReference],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "references");
    out.push_str(": [");
    if !references.is_empty() {
        out.push('\n');
        for (index, reference) in references.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            push_name_reference(out, indent + 2, reference);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_name_reference(out: &mut String, indent: usize, reference: &CoreNameReference) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &reference.id, true);
    push_string_field(out, indent + 2, "name", &reference.name, true);
    push_string_field(
        out,
        indent + 2,
        "normalized_name",
        &reference.normalized_name,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "reference_kind",
        reference.reference_kind,
        true,
    );
    push_string_field(out, indent + 2, "scope_id", &reference.scope_id, true);
    push_optional_usize_field(
        out,
        indent + 2,
        "statement_index",
        reference.statement_index,
        true,
    );
    push_span_field(out, indent + 2, "source_span", &reference.span, true);
    push_string_field(
        out,
        indent + 2,
        "resolution_status",
        reference.resolution_status,
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "resolved_definition_id",
        reference.resolved_definition_id.as_deref(),
        true,
    );
    push_optional_string_field(out, indent + 2, "reason", reference.reason, false);
    push_indent(out, indent);
    out.push('}');
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
    push_optional_string_field(out, indent + 2, "type_text", ast.type_text.as_deref(), true);
    push_optional_string_field(out, indent + 2, "type_source", ast.type_source, true);
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
    push_optional_string_field(
        out,
        indent + 2,
        "type_text",
        node.type_text.as_deref(),
        true,
    );
    push_optional_string_field(out, indent + 2, "type_source", node.type_source, true);
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
        assert!(text.contains("core_candidates=1 execution_ready=0"));
        assert!(text.contains("typed_expression_previews="));
        assert!(text.contains("expression_previews="));
        assert!(text.contains("expression_atoms="));
        assert!(text.contains("expression_ast_nodes="));
        assert!(text.contains("block_status="));
        assert!(text.contains("name_status="));
        assert!(text.contains("name_definitions="));
        assert!(text.contains("resolved_name_references="));
        assert!(text.contains("return -> return"));
        assert!(text.contains("save_in_store -> store_write_deferred"));
    }

    #[test]
    fn json_preview_reports_lowerable_contextual_and_blocked_statements() {
        let program = demo_program();
        let json = core_preview_json(&program, &[]);

        assert!(json.contains("\"schema\": \"hum.core_preview.v0\""));
        assert!(json.contains("\"core_contract_schema\": \"hum.core_contract.v0\""));
        assert!(json.contains("\"type_check_schema\": \"hum.type_check.v0\""));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"status\": \"preview_with_blockers\""));
        assert!(json.contains("\"source_kind\": \"return\""));
        assert!(json.contains("\"core_operation\": \"return\""));
        assert!(json.contains("\"expression_previews\""));
        assert!(json.contains("\"typed_expression_previews\""));
        assert!(json.contains("\"block_status\": \"block_preview_v0\""));
        assert!(json.contains("\"name_status\": \"name_preview_v0\""));
        assert!(json.contains("\"name_preview\""));
        assert!(json.contains("\"scope_model\": \"lexical_block_scope_preview_v0\""));
        assert!(json.contains("\"checked_resolver_status\": \"not_run_v0\""));
        assert!(json.contains("\"resolver_diagnostic_status\": \"preview_facts_only_v0\""));
        assert!(json.contains("\"resolver_diagnostic_count\": 0"));
        assert!(json.contains("\"scope_count\""));
        assert!(json.contains("\"scopes\""));
        assert!(json.contains("\"scope_kind\": \"root\""));
        assert!(json.contains("\"parent_scope_id\""));
        assert!(json.contains("\"definition_kind\": \"parameter\""));
        assert!(json.contains("\"definition_kind\": \"let_binding\""));
        assert!(json.contains("\"reference_kind\": \"name_ref\""));
        assert!(json.contains("\"resolution_status\": \"resolved_preview_v0\""));
        assert!(json.contains("\"resolution_status\": \"external_reference_preview_v0\""));
        assert!(json.contains("\"unresolved_name_references\": 0"));
        assert!(json.contains("\"block_preview\""));
        assert!(json.contains("\"block_kind\": \"record_construction\""));
        assert!(json.contains("\"node_kind\": \"statement_ref\""));
        assert!(json.contains("\"expression_preview\""));
        assert!(json.contains("\"atoms\""));
        assert!(json.contains("\"operators\""));
        assert!(json.contains("\"ast\""));
        assert!(json.contains("\"form\": \"binary_operation_candidate\""));
        assert!(json.contains("\"type_status\": \"not_type_checked_v0\""));
        assert!(json.contains("\"type_status\": \"checked_trivial_return_type_v0\""));
        assert!(json.contains("\"type_text\": \"WorkItem\""));
        assert!(json.contains("\"type_source\": \"record_literal_constructor_v0\""));
        assert!(json.contains("\"effect_status\": \"not_effect_checked_v0\""));
        assert!(json.contains("\"status\": \"compound_preview_v0\""));
        assert!(json.contains("\"status\": \"lowerable_preview_v0\""));
        assert!(json.contains("\"source_kind\": \"record_field_initializer\""));
        assert!(json.contains("\"core_operation\": \"record_construction_close\""));
        assert!(json.contains("\"status\": \"contextual_preview_v0\""));
        assert!(json.contains("\"core_operation\": \"store_write_deferred\""));
        assert!(json.contains("\"reason\": \"surface_save_requires_store_lowering\""));
        assert!(json.contains("\"no executable semantics\""));
        assert!(json.contains("\"no independent type checking\""));
        assert!(json.contains("\"no broad expression type checking\""));
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
        assert!(json.contains("\"scope_model\": \"lexical_block_scope_preview_v0\""));
        assert!(json.contains("\"scope_kind\": \"for_each\""));
        assert!(json.contains("\"scope_kind\": \"if_statement\""));
        assert!(json.contains("\"definition_kind\": \"for_each_binding\""));
        assert!(json.contains("\"no executable semantics\""));
    }

    #[test]
    fn json_preview_scopes_loop_binders_to_their_block() {
        let source = r#"task find session(user: User, sessions: Sessions) -> Session {
  does:
    for each session in sessions {
      if session.user == user {
        let user = session.user
        return session
      }
    }

    return session
}
"#;
        let parsed = parse_source("scopes.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = core_preview_json(&program, &[]);

        assert!(json.contains("\"name_status\": \"name_preview_with_unresolved_v0\""));
        assert!(json.contains("\"scope_model\": \"lexical_block_scope_preview_v0\""));
        assert!(json.contains("\"scope_kind\": \"for_each\""));
        assert!(json.contains("\"scope_kind\": \"if_statement\""));
        assert!(json.contains("\"definition_kind\": \"for_each_binding\""));
        assert!(json.contains("\"resolution_status\": \"resolved_preview_v0\""));
        assert!(json.contains("\"resolution_status\": \"unresolved_preview_v0\""));
        assert!(json.contains("\"reason\": \"name_not_in_candidate_scope\""));
        assert!(json.contains("\"status\": \"shadowed_definition_preview_v0\""));
        assert!(json.contains("\"reason\": \"definition_shadows_visible_name\""));
    }

    #[test]
    fn json_preview_reports_shadowed_and_unresolved_names_honestly() {
        let source = r#"task check title(title: Text) -> Text {
  does:
    let title = "shadow"
    return missing
}
"#;
        let parsed = parse_source("names.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = core_preview_json(&program, &[]);

        assert!(json.contains("\"name_status\": \"name_preview_with_unresolved_v0\""));
        assert!(json.contains("\"status\": \"shadowed_definition_preview_v0\""));
        assert!(json.contains("\"reason\": \"definition_shadows_visible_name\""));
        assert!(json.contains("\"resolution_status\": \"unresolved_preview_v0\""));
        assert!(json.contains("\"reason\": \"name_not_in_candidate_scope\""));
        assert!(json.contains("\"shadowed_name_definitions\": 1"));
        assert!(json.contains("\"unresolved_name_references\": 1"));
        assert!(json.contains("\"no module or global name resolution\""));
        assert!(json.contains("\"no checked name resolution\""));
    }

    fn demo_program() -> Program {
        let source = r#"type WorkItem {
  title: Text
  done: Bool
}

type WorkError {
  code: Text
}

store work items: list WorkItem {
  why:
    keep test work items
}

task remember work item(title: Text) -> Result WorkItem, WorkError {
  why:
    save a work item

  changes:
    work items

  does:
    if title is empty {
      fail WorkError.empty_title
    }

    let item = WorkItem {
      title: title
      done: false
    }
    save item in work items
    return item
}


"#;
        let parsed = parse_source("demo.hum", source);
        Program {
            files: vec![parsed.file],
        }
    }
}
