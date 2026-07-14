use std::collections::BTreeMap;

use crate::ast::{Item, Param, Program, Section};
use crate::callable::{self, CallableAnalysis};
use crate::core_body::{self, BodyStatement};
use crate::core_contract;
use crate::core_verify;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticOccurrence, DiagnosticOccurrenceSet, Severity, Span,
};
use crate::element_place;
use crate::field_place::{self, FieldTypeMap};
use crate::predicate::{self, PredicateFact};
use crate::return_dependency;
use crate::type_check;
use crate::typed_failure::{self, FailureFact, ProgramFailureAnalysis};
use crate::version;
use crate::writable_field_alias;

pub const FULL_TYPE_CHECK_SCHEMA: &str = "hum.full_type_check.v0";
pub const FULL_TYPE_CHECK_MODE: &str = "recognized_core_body_type_gate_v0";
pub const FULL_TYPE_CHECK_STATUS: &str = "recognized_core_body_type_gate_available_v0";

const NON_CLAIMS: &[&str] = &[
    "no executable semantics",
    "no Hum IR emission",
    "no backend lowering",
    "no proof artifact",
    "no memory-safety proof",
    "no effect checking",
    "no ownership or borrow checking",
    "no generic, trait, layout, or ABI checking",
    "no optimization claim",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullTypeCheckSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub source_errors: usize,
    pub resolver_errors: usize,
    pub type_errors: usize,
    pub core_verify_errors: usize,
    pub items: usize,
    pub body_items: usize,
    pub statements: usize,
    pub checked_statements: usize,
    pub accepted_statements: usize,
    pub rejected_statements: usize,
    pub unchecked_statements: usize,
    pub unsupported_statements: usize,
    pub blocking_issues: usize,
    pub execution_ready: usize,
    pub ir_ready: usize,
}

struct FullTypeCheckReport {
    type_check_summary: type_check::TypeCheckSummary,
    core_verify_summary: core_verify::CoreVerifyReadinessSummary,
    items: Vec<FullTypeItem>,
    files: usize,
    item_count: usize,
    source_errors: usize,
    predicates: Vec<PredicateFact>,
    diagnostic_occurrences: DiagnosticOccurrenceSet,
}

struct FullTypeItem {
    id: String,
    kind: &'static str,
    name: String,
    span: Span,
    status: &'static str,
    statements: Vec<TypedStatement>,
}

struct TypedStatement {
    id: String,
    span: Span,
    statement_kind: &'static str,
    expression_text: Option<String>,
    expected_type: Option<String>,
    actual_type: Option<String>,
    type_source: Option<&'static str>,
    status: &'static str,
    reason: Option<&'static str>,
    failure_form: Option<&'static str>,
    callee: Option<String>,
    callee_result_root: Option<String>,
    caller_result_root: Option<String>,
    wrapper_root: Option<String>,
    call_span: Option<Span>,
    callee_span: Option<Span>,
    caller_span: Option<Span>,
    diagnostic_code: Option<&'static str>,
    help: Option<String>,
    prior_blocker: Option<crate::diagnostic::PriorBlockerRef>,
    diagnostic_occurrence: Option<DiagnosticOccurrence>,
}

#[derive(Debug, Clone)]
struct TypeFact {
    type_text: String,
    source: &'static str,
}

struct StdoutWriteTypeIssue {
    call_source: String,
    call_span: Span,
    actual_type: Option<TypeFact>,
    reason: &'static str,
}

struct ReplayTickTypeIssue {
    call_source: String,
    call_span: Span,
    reason: &'static str,
}

struct FileReadTypeIssue {
    call_source: String,
    call_span: Span,
    actual_type: Option<TypeFact>,
    reason: &'static str,
}

pub fn full_type_check_has_errors(program: &Program, diagnostics: &[Diagnostic]) -> bool {
    full_type_check_summary(program, diagnostics).blocking_issues > 0
}

pub fn full_type_check_has_only_predicate_errors(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> bool {
    let report = build_report(program, diagnostics);
    report.rejected_predicates() > 0
        && report.source_errors == 0
        && report.type_check_summary.resolver_errors == 0
        && report.type_check_summary.type_errors == 0
        && report.core_verify_summary.failed_checks == 0
        && report.rejected_statements() == 0
        && report.unchecked_statements() == 0
        && report.unsupported_statements() == 0
}

pub fn full_type_check_summary(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> FullTypeCheckSummary {
    let report = build_report(program, diagnostics);
    FullTypeCheckSummary {
        schema: FULL_TYPE_CHECK_SCHEMA,
        status: report.status(),
        mode: FULL_TYPE_CHECK_MODE,
        source_errors: report.source_errors,
        resolver_errors: report.type_check_summary.resolver_errors,
        type_errors: report.type_check_summary.type_errors,
        core_verify_errors: report.core_verify_summary.failed_checks,
        items: report.item_count(),
        body_items: report.items.len(),
        statements: report.statement_count(),
        checked_statements: report.checked_statements(),
        accepted_statements: report.accepted_statements(),
        rejected_statements: report.rejected_statements(),
        unchecked_statements: report.unchecked_statements(),
        unsupported_statements: report.unsupported_statements(),
        blocking_issues: report.blocking_issues(),
        execution_ready: 0,
        ir_ready: 0,
    }
}

pub(crate) fn validate_typed_failure_prior_blockers(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> Result<(), crate::diagnostic::DiagnosticInvariantError> {
    let analysis = typed_failure::analyze_program(program);
    let mut collector = crate::diagnostic::DiagnosticOccurrenceCollector::default();
    for occurrence in analysis.occurrences() {
        collector.insert(occurrence)?;
    }
    let report = build_report(program, diagnostics);
    for prior in report
        .items
        .iter()
        .flat_map(|item| item.statements.iter())
        .filter_map(|statement| statement.prior_blocker.as_ref())
    {
        collector.validate_prior(prior)?;
    }
    Ok(())
}

pub fn full_type_check_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum full type check ({FULL_TYPE_CHECK_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {FULL_TYPE_CHECK_MODE}\n"));
    out.push_str(&format!("status: {}\n", report.status()));
    out.push_str(&format!(
        "dependencies: core_contract={} type_check={} core_verify={}\n",
        core_contract::CORE_CONTRACT_SCHEMA,
        type_check::TYPE_CHECK_SCHEMA,
        core_verify::CORE_VERIFY_SCHEMA
    ));
    out.push_str(&format!(
        "summary: files={} items={} body_items={} statements={} checked_statements={} accepted_statements={} rejected_statements={} unchecked_statements={} unsupported_statements={} blocking_issues={} source_errors={} resolver_errors={} type_errors={} core_verify_errors={} execution_ready=0 ir_ready=0\n",
        report.files(),
        report.item_count(),
        report.items.len(),
        report.statement_count(),
        report.checked_statements(),
        report.accepted_statements(),
        report.rejected_statements(),
        report.unchecked_statements(),
        report.unsupported_statements(),
        report.blocking_issues(),
        report.source_errors,
        report.type_check_summary.resolver_errors,
        report.type_check_summary.type_errors,
        report.core_verify_summary.failed_checks
    ));

    if report.items.is_empty() {
        out.push_str("typed_items: none\n");
    } else {
        out.push_str("typed_items:\n");
        for item in &report.items {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} `{}` statements={}\n",
                item.span.file,
                item.span.line,
                item.span.column,
                item.status,
                item.kind,
                item.name,
                item.statements.len()
            ));
            for statement in &item.statements {
                out.push_str(&format!(
                    "    {}:{}:{} [{}] {}",
                    statement.span.file,
                    statement.span.line,
                    statement.span.column,
                    statement.status,
                    statement.statement_kind
                ));
                if let Some(expression) = &statement.expression_text {
                    out.push_str(&format!(" `{expression}`"));
                }
                out.push_str(&format!(
                    " expected={} actual={}",
                    statement.expected_type.as_deref().unwrap_or("none"),
                    statement.actual_type.as_deref().unwrap_or("unknown")
                ));
                if let Some(reason) = statement.reason {
                    out.push_str(&format!(" reason={reason}"));
                }
                if let Some(code) = statement.diagnostic_code {
                    out.push_str(&format!(" diagnostic={code}"));
                }
                if let Some(form) = statement.failure_form {
                    out.push_str(&format!(" failure_form={form}"));
                }
                if let Some(root) = &statement.callee_result_root {
                    out.push_str(&format!(" callee_root={root}"));
                }
                if let Some(root) = &statement.caller_result_root {
                    out.push_str(&format!(" caller_root={root}"));
                }
                if let Some(help) = &statement.help {
                    out.push_str(&format!(" help={help}"));
                }
                out.push('\n');
            }
        }
    }

    if report.predicates.is_empty() {
        out.push_str("predicate_facts: none\n");
    } else {
        out.push_str("predicate_facts:\n");
        for fact in &report.predicates {
            out.push_str(&format!(
                "  {}:{}:{} [{}] task=`{}` section={} reason={} text=`{}`",
                fact.line_span.file.replace('\\', "/"),
                fact.line_span.line,
                fact.line_span.column,
                fact.status.as_str(),
                fact.task,
                fact.section,
                fact.reason,
                fact.text
            ));
            for place in &fact.places {
                out.push_str(&format!(
                    " place=`{}` place_span={}:{}:{} scope={} definition={} root_definition={} resolution={} eligibility={} type={}",
                    place.text,
                    place.span.file.replace('\\', "/"),
                    place.span.line,
                    place.span.column,
                    place.scope_id,
                    place.definition_id.as_deref().unwrap_or("none"),
                    place.root_definition_id.as_deref().unwrap_or("none"),
                    place.resolution,
                    place.eligibility,
                    place.type_text.as_deref().unwrap_or("unknown")
                ));
            }
            if let Some(expected) = &fact.expected {
                out.push_str(&format!(" expected={expected}"));
            }
            if let Some(actual) = &fact.actual {
                out.push_str(&format!(" actual={actual}"));
            }
            if let Some(operator) = fact.comparison {
                out.push_str(&format!(" operator={operator}"));
            }
            if let (Some(left), Some(right)) = (&fact.left_type, &fact.right_type) {
                out.push_str(&format!(" left_type={left} right_type={right}"));
            }
            if fact.blocks() {
                out.push_str(&format!(" diagnostic=H0704 help={}", fact.repair()));
            }
            if let Some(span) = &fact.intent_span {
                out.push_str(&format!(
                    " intent_span={}:{}:{}",
                    span.file, span.line, span.column
                ));
            }
            if let Some(span) = &fact.offending_span {
                out.push_str(&format!(
                    " offending_span={}:{}:{}",
                    span.file, span.line, span.column
                ));
            }
            out.push('\n');
        }
    }

    out.push_str("non_claims:\n");
    for non_claim in NON_CLAIMS {
        out.push_str(&format!("  - {non_claim}\n"));
    }

    out
}

pub fn full_type_check_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", FULL_TYPE_CHECK_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", FULL_TYPE_CHECK_MODE, true);
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
    push_string_field(
        &mut out,
        2,
        "core_verify_schema",
        core_verify::CORE_VERIFY_SCHEMA,
        true,
    );
    push_dependency_summaries(&mut out, &report, 2, true);
    push_summary(&mut out, &report, 2, true);
    push_items(&mut out, &report.items, 2, true);
    push_predicates(&mut out, &report.predicates, 2, true);
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> FullTypeCheckReport {
    let type_check_summary = type_check::type_check_summary(program, diagnostics);
    let core_verify_summary = core_verify::core_verify_readiness_summary(program, diagnostics);
    let callables = callable::analyze_program(program);
    let source_errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let blocked = source_errors > 0
        || type_check_summary.resolver_errors > 0
        || type_check_summary.type_errors > 0
        || core_verify_summary.failed_checks > 0;
    let task_returns = task_return_types(program);
    let failure_analysis = typed_failure::analyze_program(program);
    let field_types = field_place::collect_field_types(program);
    let predicates = predicate::analyze_program(program);
    let mut items = Vec::new();
    let context = FullTypeCollectionContext {
        program,
        blocked,
        failure_analysis: &failure_analysis,
        field_types: &field_types,
        callables: &callables,
    };
    for file in &program.files {
        collect_items(&context, &file.items, &task_returns, &mut items);
    }

    let mut diagnostic_occurrences = core_verify::diagnostic_occurrence_set(program, diagnostics);
    extend_full_type_occurrences(&failure_analysis, &items, &mut diagnostic_occurrences);

    FullTypeCheckReport {
        type_check_summary,
        core_verify_summary,
        items,
        files: program.files.len(),
        item_count: count_items(program),
        source_errors,
        predicates: predicates.facts().to_vec(),
        diagnostic_occurrences,
    }
}

fn extend_full_type_occurrences(
    failure_analysis: &typed_failure::ProgramFailureAnalysis,
    items: &[FullTypeItem],
    diagnostic_occurrences: &mut DiagnosticOccurrenceSet,
) {
    for occurrence in failure_analysis
        .occurrences()
        .into_iter()
        .filter(|occurrence| occurrence.owning_stage() == "full_type_check")
    {
        diagnostic_occurrences
            .insert_owned(occurrence)
            .expect("typed-failure occurrences must remain unique at full type");
    }
    for occurrence in items
        .iter()
        .flat_map(|item| &item.statements)
        .filter_map(|statement| statement.diagnostic_occurrence.clone())
    {
        diagnostic_occurrences
            .insert_owned(occurrence)
            .expect("built-in call occurrences must be unique");
    }
}

pub(crate) fn diagnostic_occurrence_set(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> DiagnosticOccurrenceSet {
    build_report(program, diagnostics).diagnostic_occurrences
}

pub(crate) fn diagnostic_occurrence_set_from_source(
    program: &Program,
    diagnostics: &[Diagnostic],
    source_occurrences: &DiagnosticOccurrenceSet,
) -> Result<DiagnosticOccurrenceSet, crate::diagnostic::DiagnosticInvariantError> {
    let report = build_report(program, diagnostics);
    let mut occurrences = core_verify::validate_diagnostic_projection_from_source(
        program,
        diagnostics,
        source_occurrences,
    )?;
    let failure_analysis = typed_failure::analyze_program(program);
    extend_full_type_occurrences(&failure_analysis, &report.items, &mut occurrences);
    Ok(occurrences)
}

struct FullTypeCollectionContext<'a> {
    program: &'a Program,
    blocked: bool,
    failure_analysis: &'a ProgramFailureAnalysis,
    field_types: &'a FieldTypeMap,
    callables: &'a CallableAnalysis,
}

fn collect_items(
    context: &FullTypeCollectionContext<'_>,
    items: &[Item],
    task_returns: &BTreeMap<String, TypeFact>,
    out: &mut Vec<FullTypeItem>,
) {
    for item in items {
        if let Some(typed_item) = type_item(
            context.program,
            item,
            context.blocked,
            task_returns,
            context.failure_analysis,
            context.field_types,
            context.callables,
        ) {
            out.push(typed_item);
        }
        if let Item::App(app) = item {
            let app_task_returns = task_return_types_from_items(&app.items);
            collect_items(context, &app.items, &app_task_returns, out);
        }
    }
}

fn type_item(
    program: &Program,
    item: &Item,
    blocked: bool,
    task_returns: &BTreeMap<String, TypeFact>,
    failure_analysis: &ProgramFailureAnalysis,
    field_types: &FieldTypeMap,
    callables: &CallableAnalysis,
) -> Option<FullTypeItem> {
    let item_identity = crate::resolve::semantic_item_identity_for(program, item);
    let does = item_sections(item)
        .iter()
        .find(|section| section.name == "does")?;
    let body = core_body::analyze_does_section(does);
    let mut environment = initial_environment(item_params(item));
    let mut statements = Vec::new();
    for (index, statement) in body.statements.iter().enumerate() {
        let typed = type_statement(
            &item_identity,
            item,
            index,
            statement,
            &mut environment,
            task_returns,
            field_types,
            blocked,
            match item {
                Item::Task(task) => failure_analysis.fact(task, index),
                _ => None,
            },
            callables,
        );
        statements.push(typed);
    }
    let status = item_status(&statements, blocked);
    Some(FullTypeItem {
        id: prefixed_id(
            "hum_full_type_item",
            &format!("{}_{}_{}", item.kind(), item.name(), item.span().line),
        ),
        kind: item.kind(),
        name: item.name().to_string(),
        span: portable_span(item.span()),
        status,
        statements,
    })
}

#[allow(clippy::too_many_arguments)]
fn type_statement(
    item_identity: &str,
    item: &Item,
    index: usize,
    statement: &BodyStatement,
    environment: &mut BTreeMap<String, TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
    blocked: bool,
    failure_fact: Option<&FailureFact>,
    callables: &CallableAnalysis,
) -> TypedStatement {
    if blocked {
        return typed_statement(
            statement,
            index,
            None,
            None,
            None,
            "not_checked_blocked_by_prior_errors_v0",
            Some("source_resolver_type_or_core_verify_errors"),
        );
    }

    if statement.status == "unsupported_v0" {
        return typed_statement(
            statement,
            index,
            None,
            None,
            None,
            "blocked_unsupported_statement_v0",
            statement
                .reason
                .or(Some("statement_not_in_core_body_grammar_v0")),
        );
    }

    if let Some(issue) = stdout_write_type_issue(statement, environment, task_returns, field_types)
    {
        let mut typed = typed_statement(
            statement,
            index,
            Some(issue.call_source),
            Some("Text".to_string()),
            issue.actual_type,
            "rejected_invalid_stdout_write_call_v0",
            Some(issue.reason),
        );
        typed.failure_form = Some("bounded_output_builtin");
        typed.call_span = Some(issue.call_span);
        typed.caller_span = Some(item.span().clone());
        typed.diagnostic_code = Some(DiagnosticCode::INVALID_STDOUT_WRITE_CALL.as_str());
        typed.help = Some(
            "Pass exactly one checked `Text` argument to `stdout_write`, then handle its `OutputError` explicitly."
                .to_string(),
        );
        attach_builtin_occurrence(
            &mut typed,
            item_identity,
            index,
            DiagnosticCode::INVALID_STDOUT_WRITE_CALL,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(100),
            "stdout_write_call_shape",
        );
        return typed;
    }

    if let Some(issue) = replay_tick_type_issue(statement) {
        let mut typed = typed_statement(
            statement,
            index,
            Some(issue.call_source),
            Some("no arguments".to_string()),
            None,
            "rejected_invalid_clock_replay_call_v0",
            Some(issue.reason),
        );
        typed.failure_form = Some("runner_replay_builtin");
        typed.call_span = Some(issue.call_span);
        typed.caller_span = Some(item.span().clone());
        typed.diagnostic_code = Some(DiagnosticCode::INVALID_CLOCK_REPLAY_CALL.as_str());
        typed.help = Some(
            "Call `clock_replay_tick()` with no arguments, then handle its `ReplayClockError` explicitly."
                .to_string(),
        );
        attach_builtin_occurrence(
            &mut typed,
            item_identity,
            index,
            DiagnosticCode::INVALID_CLOCK_REPLAY_CALL,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(101),
            "clock_replay_call_shape",
        );
        return typed;
    }

    if let Some(issue) = file_read_type_issue(statement, environment, task_returns, field_types) {
        let mut typed = typed_statement(
            statement,
            index,
            Some(issue.call_source),
            Some("Path".to_string()),
            issue.actual_type,
            "rejected_invalid_files_read_text_call_v0",
            Some(issue.reason),
        );
        typed.failure_form = Some("hardened_exact_file_read_builtin");
        typed.call_span = Some(issue.call_span);
        typed.caller_span = Some(item.span().clone());
        typed.diagnostic_code = Some(DiagnosticCode::INVALID_FILE_READ_CALL.as_str());
        typed.help = Some(
            "Pass exactly the runner-owned opaque `Path` to `files_read_text`, then handle its `FileReadError` explicitly."
                .to_string(),
        );
        attach_builtin_occurrence(
            &mut typed,
            item_identity,
            index,
            DiagnosticCode::INVALID_FILE_READ_CALL,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(102),
            "files_read_call_shape",
        );
        return typed;
    }

    if let Some(fact) = failure_fact {
        let effect_owned_missing_declaration = fact.diagnostic_code
            == Some(crate::diagnostic::DiagnosticCode::MISSING_FAILURE_DECLARATION);
        let actual = fact
            .success_type
            .as_ref()
            .map(|type_text| type_fact(type_text, "typed_failure_success_v0"));
        let expected_type = if statement.kind == "fail" {
            fact.caller_result_root.clone()
        } else {
            binding_annotation(statement)
        };
        let mut typed = typed_statement(
            statement,
            index,
            expression_text_for_statement(statement).map(str::to_string),
            expected_type,
            actual.clone(),
            if effect_owned_missing_declaration {
                "accepted_typed_failure_deferred_to_effect_v0"
            } else {
                fact.status
            },
            if effect_owned_missing_declaration {
                Some("fails_when_declaration_deferred_to_effect_v0")
            } else {
                fact.reason
            },
        );
        apply_failure_fact(&mut typed, fact);
        if effect_owned_missing_declaration {
            typed.diagnostic_code = None;
            typed.help = None;
        }
        if matches!(statement.kind, "let_binding" | "mutable_binding")
            && (fact.diagnostic_code.is_none() || effect_owned_missing_declaration)
            && let Some((name, type_fact)) = binding_type_fact(statement, actual.as_ref())
        {
            environment.insert(name_key(&name), type_fact);
        }
        return typed;
    }

    let expression_text = expression_text_for_statement(statement).map(str::to_string);
    let expected_type = expected_type_for_statement(item, statement, environment, field_types);
    let callable_actual = match item {
        Item::Task(task) => callables
            .indirect_application(task, &statement.span)
            .map(|fact| {
                type_fact(
                    &fact.result_type,
                    "checked_indirect_callable_application_v0",
                )
            }),
        _ => None,
    };
    let actual = callable_actual.or_else(|| {
        expression_text.as_deref().and_then(|expression| {
            infer_expression_type(expression, environment, task_returns, field_types)
        })
    });
    let (status, reason) = statement_status(statement, expected_type.as_deref(), actual.as_ref());

    if matches!(statement.kind, "let_binding" | "mutable_binding")
        && let Some((name, fact)) = binding_type_fact(statement, actual.as_ref())
    {
        environment.insert(name_key(&name), fact);
    }

    typed_statement(
        statement,
        index,
        expression_text,
        expected_type,
        actual,
        status,
        reason,
    )
}

fn stdout_write_type_issue(
    statement: &BodyStatement,
    environment: &BTreeMap<String, TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<StdoutWriteTypeIssue> {
    let expression = expression_text_for_statement(statement)?;
    let expression_offset = statement.text.find(expression).unwrap_or(0);
    let call = typed_failure::calls_in_expression(expression)
        .into_iter()
        .find(|call| call.callee == "stdout_write")?;
    let call_span = Span {
        file: statement.span.file.clone(),
        line: statement.span.line,
        column: statement.span.column
            + statement.text[..expression_offset + call.source_offset]
                .chars()
                .count(),
    };
    let args = call
        .source
        .strip_prefix("stdout_write(")?
        .strip_suffix(')')?;
    let arguments = split_call_arguments(args);
    if arguments.len() != 1 {
        return Some(StdoutWriteTypeIssue {
            call_source: call.source,
            call_span,
            actual_type: None,
            reason: "stdout_write_requires_exactly_one_argument_v0",
        });
    }
    let actual_type = infer_expression_type(arguments[0], environment, task_returns, field_types);
    if actual_type
        .as_ref()
        .is_some_and(|actual| actual.type_text == "Text")
    {
        return None;
    }
    Some(StdoutWriteTypeIssue {
        call_source: call.source,
        call_span,
        actual_type,
        reason: "stdout_write_argument_must_be_text_v0",
    })
}

fn replay_tick_type_issue(statement: &BodyStatement) -> Option<ReplayTickTypeIssue> {
    let expression = expression_text_for_statement(statement)?;
    let expression_offset = statement.text.find(expression).unwrap_or(0);
    let call = typed_failure::calls_in_expression(expression)
        .into_iter()
        .find(|call| call.callee == "clock_replay_tick")?;
    let call_span = Span {
        file: statement.span.file.clone(),
        line: statement.span.line,
        column: statement.span.column
            + statement.text[..expression_offset + call.source_offset]
                .chars()
                .count(),
    };
    let args = call
        .source
        .strip_prefix("clock_replay_tick(")?
        .strip_suffix(')')?;
    (!args.trim().is_empty()).then_some(ReplayTickTypeIssue {
        call_source: call.source,
        call_span,
        reason: "clock_replay_tick_requires_zero_arguments_v0",
    })
}

fn file_read_type_issue(
    statement: &BodyStatement,
    environment: &BTreeMap<String, TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<FileReadTypeIssue> {
    let expression = expression_text_for_statement(statement)?;
    let expression_offset = statement.text.find(expression).unwrap_or(0);
    let call = typed_failure::calls_in_expression(expression)
        .into_iter()
        .find(|call| call.callee == "files_read_text")?;
    let call_span = Span {
        file: statement.span.file.clone(),
        line: statement.span.line,
        column: statement.span.column
            + statement.text[..expression_offset + call.source_offset]
                .chars()
                .count(),
    };
    let args = call
        .source
        .strip_prefix("files_read_text(")?
        .strip_suffix(')')?;
    let arguments = split_call_arguments(args);
    if arguments.len() != 1 {
        return Some(FileReadTypeIssue {
            call_source: call.source,
            call_span,
            actual_type: None,
            reason: "files_read_text_requires_exactly_one_argument_v0",
        });
    }
    let actual_type = infer_expression_type(arguments[0], environment, task_returns, field_types);
    if actual_type
        .as_ref()
        .is_some_and(|actual| actual.type_text == "Path")
    {
        return None;
    }
    Some(FileReadTypeIssue {
        call_source: call.source,
        call_span,
        actual_type,
        reason: "files_read_text_argument_must_be_opaque_path_v0",
    })
}

fn split_call_arguments(text: &str) -> Vec<&str> {
    let mut arguments = Vec::new();
    let mut start = 0usize;
    let mut depth = 0isize;
    let mut in_string = false;
    for (index, ch) in text.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' | '[' | '{' if !in_string => depth += 1,
            ')' | ']' | '}' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                let argument = text[start..index].trim();
                if !argument.is_empty() {
                    arguments.push(argument);
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    let argument = text[start..].trim();
    if !argument.is_empty() {
        arguments.push(argument);
    }
    arguments
}

fn expected_type_for_statement(
    item: &Item,
    statement: &BodyStatement,
    environment: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<String> {
    match statement.kind {
        "return" => item_result(item).map(expected_return_value_type),
        "fail" => item_result(item).and_then(expected_error_value_type),
        "if_header" | "while_header" => Some("Bool".to_string()),
        "let_binding" | "mutable_binding" => binding_annotation(statement),
        "set_place" => set_place_name(statement)
            .and_then(|name| place_type_fact(name, environment, field_types))
            .map(|fact| fact.type_text),
        _ => None,
    }
}

fn statement_status(
    statement: &BodyStatement,
    expected_type: Option<&str>,
    actual: Option<&TypeFact>,
) -> (&'static str, Option<&'static str>) {
    match statement.kind {
        "block_close" | "loop_header" => ("accepted_no_expression_type_obligation_v0", None),
        "nested_intent_header" => (
            "blocked_unsupported_statement_v0",
            Some("nested_intent_lowering_not_implemented"),
        ),
        "test_expectation" => (
            "unchecked_statement_type_v0",
            Some("test_expectation_typing_not_implemented"),
        ),
        "for_each_header" | "for_index_header" => (
            "unchecked_statement_type_v0",
            Some("iterator_type_checking_not_implemented"),
        ),
        "record_field_initializer" => (
            "unchecked_statement_type_v0",
            Some("record_field_context_not_tracked_v0"),
        ),
        "return" | "fail" | "if_header" | "while_header" | "set_place" => {
            typed_expression_status(expected_type, actual)
        }
        "let_binding" | "mutable_binding" => {
            let writable_alias_candidate =
                writable_field_alias::candidate_name(statement).is_some();
            let unsupported_or_untyped_alias_candidate = writable_alias_candidate
                && (writable_field_alias::exact_binding(statement).is_none() || actual.is_none());
            if unsupported_or_untyped_alias_candidate {
                (
                    "accepted_writable_field_alias_candidate_deferred_to_ownership_v0",
                    Some("writable_field_alias_shape_deferred_to_ownership_v0"),
                )
            } else if expected_type.is_some() {
                typed_expression_status(expected_type, actual)
            } else if actual.is_some() {
                ("accepted_inferred_binding_type_v0", None)
            } else {
                (
                    "unchecked_statement_type_v0",
                    Some("binding_initializer_type_unknown_v0"),
                )
            }
        }
        _ => (
            "blocked_unsupported_statement_v0",
            statement
                .reason
                .or(Some("statement_type_rule_not_implemented")),
        ),
    }
}

fn typed_expression_status(
    expected_type: Option<&str>,
    actual: Option<&TypeFact>,
) -> (&'static str, Option<&'static str>) {
    let Some(expected_type) = expected_type else {
        return (
            "unchecked_statement_type_v0",
            Some("expected_type_context_missing_v0"),
        );
    };
    let Some(actual) = actual else {
        return (
            "unchecked_statement_type_v0",
            Some("expression_type_unknown_v0"),
        );
    };
    if types_compatible(expected_type, &actual.type_text) {
        ("accepted_statement_type_v0", None)
    } else {
        (
            "rejected_statement_type_mismatch_v0",
            Some("statement_expression_type_mismatch"),
        )
    }
}

fn typed_statement(
    statement: &BodyStatement,
    index: usize,
    expression_text: Option<String>,
    expected_type: Option<String>,
    actual: Option<TypeFact>,
    status: &'static str,
    reason: Option<&'static str>,
) -> TypedStatement {
    TypedStatement {
        id: prefixed_id(
            "hum_full_type_stmt",
            &format!("{}_{}_{}", statement.kind, statement.span.line, index),
        ),
        span: portable_span(&statement.span),
        statement_kind: statement.kind,
        expression_text,
        expected_type,
        actual_type: actual.as_ref().map(|fact| fact.type_text.clone()),
        type_source: actual.map(|fact| fact.source),
        status,
        reason,
        failure_form: None,
        callee: None,
        callee_result_root: None,
        caller_result_root: None,
        wrapper_root: None,
        call_span: None,
        callee_span: None,
        caller_span: None,
        diagnostic_code: None,
        help: None,
        prior_blocker: None,
        diagnostic_occurrence: None,
    }
}

fn attach_builtin_occurrence(
    statement: &mut TypedStatement,
    item_identity: &str,
    statement_index: usize,
    code: DiagnosticCode,
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    semantic_kind: &'static str,
) {
    let cause = crate::diagnostic_catalog::diagnostic_cause_for_key(cause_key)
        .expect("built-in semantic producer must name one registered cause");
    let semantic_origin =
        format!("full-type:{item_identity}:statement-{statement_index}:builtin-call-0");
    let identity = DiagnosticOccurrence::semantic_relationship_identity(
        cause.origin_kind,
        cause.route_kind,
        semantic_origin,
        vec![
            format!("item_identity={item_identity}"),
            format!("statement_index={statement_index}"),
            format!("semantic_call_kind={semantic_kind}"),
        ],
    );
    let diagnostic = Diagnostic::error(
        code,
        statement
            .reason
            .expect("built-in semantic producer has a detail reason"),
        statement
            .call_span
            .clone()
            .or_else(|| Some(statement.span.clone())),
    )
    .with_help(
        statement
            .help
            .clone()
            .expect("built-in semantic producer has repair text"),
    );
    statement.diagnostic_occurrence = Some(
        DiagnosticOccurrence::registered(cause, identity, diagnostic)
            .expect("built-in semantic occurrence must validate at production"),
    );
}

fn apply_failure_fact(statement: &mut TypedStatement, fact: &FailureFact) {
    statement.failure_form = Some(fact.form);
    statement.callee = fact.callee.clone();
    statement.callee_result_root = fact.callee_result_root.clone();
    statement.caller_result_root = fact.caller_result_root.clone();
    statement.wrapper_root = fact.wrapper_root.clone();
    statement.call_span = Some(fact.call_span.clone());
    statement.callee_span = fact.callee_span.clone();
    statement.caller_span = Some(fact.caller_span.clone());
    statement.diagnostic_code = fact.diagnostic_code.map(|code| code.as_str());
    statement.help = fact.help.clone();
    statement.prior_blocker = fact
        .occurrence
        .as_ref()
        .map(|occurrence| occurrence.prior_blocker());
}

fn task_return_types(program: &Program) -> BTreeMap<String, TypeFact> {
    let mut returns = session_z_builtin_return_types();
    for file in &program.files {
        collect_task_return_types(&file.items, &mut returns);
    }
    returns
}

fn task_return_types_from_items(items: &[Item]) -> BTreeMap<String, TypeFact> {
    let mut returns = session_z_builtin_return_types();
    collect_task_return_types(items, &mut returns);
    returns
}

fn session_z_builtin_return_types() -> BTreeMap<String, TypeFact> {
    BTreeMap::from([
        (
            name_key("stdout_write"),
            type_fact("Unit", "stdout_write_builtin_v0"),
        ),
        (
            name_key("clock_replay_tick"),
            type_fact("UInt", "runner_replay_builtin_v0"),
        ),
        (
            name_key("files_read_text"),
            type_fact("Text", "hardened_exact_file_read_builtin_v0"),
        ),
    ])
}

fn collect_task_return_types(items: &[Item], returns: &mut BTreeMap<String, TypeFact>) {
    for item in items {
        match item {
            Item::Task(task) => {
                if let Some(result) = task.result.as_deref() {
                    returns.insert(
                        name_key(&task.name),
                        type_fact(expected_return_value_type(result), "task_call_result_v0"),
                    );
                }
            }
            Item::App(app) => collect_task_return_types(&app.items, returns),
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn initial_environment(params: &[Param]) -> BTreeMap<String, TypeFact> {
    let mut environment = BTreeMap::new();
    for param in params {
        let type_text = param.ty.trim();
        if !type_text.is_empty() {
            environment.insert(
                name_key(&param.name),
                TypeFact {
                    type_text: type_text.to_string(),
                    source: "parameter_annotation_v0",
                },
            );
        }
    }
    environment
}

fn binding_type_fact(
    statement: &BodyStatement,
    actual: Option<&TypeFact>,
) -> Option<(String, TypeFact)> {
    let annotation = binding_annotation(statement);
    let name = binding_name(statement)?;
    if let Some(type_text) = annotation {
        return Some((
            name,
            TypeFact {
                type_text,
                source: "binding_annotation_v0",
            },
        ));
    }
    let actual = actual?;
    Some((name, actual.clone()))
}

fn binding_annotation(statement: &BodyStatement) -> Option<String> {
    let left = binding_left(statement)?;
    let (_name, type_text) = left.split_once(':')?;
    let type_text = type_text.trim();
    if type_text.is_empty() {
        None
    } else {
        Some(type_text.to_string())
    }
}

fn binding_name(statement: &BodyStatement) -> Option<String> {
    let left = binding_left(statement)?;
    let name = left.split_once(':').map_or(left, |(name, _type_text)| name);
    let name = name.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn binding_left(statement: &BodyStatement) -> Option<&str> {
    if !matches!(statement.kind, "let_binding" | "mutable_binding") {
        return None;
    }
    let keyword = if statement.kind == "let_binding" {
        "let"
    } else {
        "change"
    };
    let rest = strip_keyword(&statement.text, keyword)?;
    rest.split_once('=').map(|(left, _value)| left.trim())
}

fn set_place_name(statement: &BodyStatement) -> Option<&str> {
    let rest = strip_keyword(&statement.text, "set")?;
    let (place, _value) = rest.split_once('=')?;
    let place = place.trim();
    if place.is_empty() { None } else { Some(place) }
}

fn place_type_fact(
    name: &str,
    environment: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<TypeFact> {
    if let Some((root, _index)) = element_place::split_element_place(name) {
        let root_fact = environment.get(&name_key(root))?;
        let type_text = element_place::list_element_type(&root_fact.type_text)?;
        return Some(type_fact(type_text, "list_element_place_v0"));
    }
    if let Some((root, field)) = field_place::split_field_place(name) {
        let root_fact = environment.get(&name_key(root))?;
        let type_text = field_place::field_type(field_types, &root_fact.type_text, field)?;
        return Some(type_fact(type_text, "record_field_place_v0"));
    }
    environment.get(&name_key(name)).cloned()
}

fn infer_expression_type(
    expression_text: &str,
    environment: &BTreeMap<String, TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<TypeFact> {
    let text = expression_text.trim();
    if text.is_empty() {
        return Some(type_fact("Unit", "unit_expression_v0"));
    }
    if let Some(argument) = strip_permission_expression(text) {
        return infer_expression_type(argument, environment, task_returns, field_types);
    }
    if text == "true" || text == "false" {
        return Some(type_fact("Bool", "bool_literal_v0"));
    }
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return Some(type_fact("Text", "text_literal_v0"));
    }
    if return_dependency::is_closed_view_derivation_expression(text) {
        return Some(type_fact("Text", "closed_view_derivation_slice_until_v0"));
    }
    if is_list_literal(text) {
        return Some(type_fact("list_literal", "list_literal_v0"));
    }
    if text.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(type_fact("integer_literal", "integer_literal_v0"));
    }
    if let Some(fact) = place_type_fact(text, environment, field_types) {
        return Some(fact);
    }
    if is_condition_expression(text) {
        return Some(type_fact("Bool", "condition_expression_v0"));
    }
    if let Some(type_name) = record_literal_type_name(text) {
        return Some(type_fact(type_name, "record_literal_constructor_v0"));
    }
    if let Some(root) = path_root_type_name(text) {
        return Some(type_fact(root, "path_root_type_v0"));
    }
    if let Some(fact) = infer_additive_expression_type(text, environment, task_returns, field_types)
    {
        return Some(fact);
    }
    if let Some(fact) =
        infer_multiplicative_expression_type(text, environment, task_returns, field_types)
    {
        return Some(fact);
    }
    if let Some((callee, _args)) = split_call(text) {
        if callee == "list_append" {
            return Some(type_fact("Unit", "list_append_builtin_v0"));
        }
        return task_returns.get(&name_key(callee)).cloned();
    }
    place_type_fact(text, environment, field_types)
}

fn infer_additive_expression_type(
    text: &str,
    environment: &BTreeMap<String, TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<TypeFact> {
    let (left, right) = text.split_once(" + ")?;
    let left = infer_expression_type(left, environment, task_returns, field_types)?;
    let right = infer_expression_type(right, environment, task_returns, field_types)?;
    if right.type_text == "integer_literal" || left.type_text == right.type_text {
        Some(TypeFact {
            type_text: left.type_text,
            source: "additive_expression_v0",
        })
    } else {
        None
    }
}

fn infer_multiplicative_expression_type(
    text: &str,
    environment: &BTreeMap<String, TypeFact>,
    task_returns: &BTreeMap<String, TypeFact>,
    field_types: &FieldTypeMap,
) -> Option<TypeFact> {
    let (left, right) = text.split_once(" * ")?;
    let left = infer_expression_type(left, environment, task_returns, field_types)?;
    let right = infer_expression_type(right, environment, task_returns, field_types)?;
    if right.type_text == "integer_literal" || left.type_text == right.type_text {
        Some(TypeFact {
            type_text: left.type_text,
            source: "multiplicative_expression_v0",
        })
    } else {
        None
    }
}

fn is_list_literal(text: &str) -> bool {
    text.starts_with('[') && text.ends_with(']')
}

fn strip_permission_expression(text: &str) -> Option<&str> {
    ["borrow", "change", "consume"]
        .iter()
        .find_map(|keyword| strip_keyword(text.trim(), keyword))
}

fn split_call(text: &str) -> Option<(&str, &str)> {
    let text = text.trim();
    let inside = text.strip_suffix(')')?;
    let (callee, args) = inside.split_once('(')?;
    let callee = callee.trim();
    if callee.is_empty() {
        None
    } else {
        Some((callee, args))
    }
}

fn is_condition_expression(text: &str) -> bool {
    [
        " == ", " != ", " <= ", " >= ", " < ", " > ", " is ", " does ", " and ", " or ",
    ]
    .iter()
    .any(|operator| text.contains(operator))
}

fn record_literal_type_name(text: &str) -> Option<String> {
    let constructor = text.trim().strip_suffix('{')?.trim();
    if is_type_like_name(constructor) {
        Some(constructor.to_string())
    } else {
        None
    }
}

fn path_root_type_name(text: &str) -> Option<String> {
    let (root, _field) = text.split_once('.')?;
    let root = root.trim();
    if is_type_like_name(root) {
        Some(root.to_string())
    } else {
        None
    }
}

fn is_type_like_name(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
        && text
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == ' ')
}

fn expected_return_value_type(expected_type: &str) -> String {
    let expected_type = return_dependency::result_type_without_return_dependency(expected_type);
    let tokens = type_tokens(&expected_type);
    if matches!(
        tokens.first().map(String::as_str),
        Some("Result" | "Option" | "Maybe" | "Slice" | "Span")
    ) && tokens.len() >= 2
    {
        tokens[1].clone()
    } else {
        expected_type
    }
}

fn expected_error_value_type(expected_type: &str) -> Option<String> {
    let expected_type = return_dependency::result_type_without_return_dependency(expected_type);
    let tokens = type_tokens(&expected_type);
    if matches!(tokens.first().map(String::as_str), Some("Result")) && tokens.len() >= 3 {
        Some(tokens[2].clone())
    } else {
        None
    }
}

fn types_compatible(expected_type: &str, actual_type: &str) -> bool {
    let expected_type = return_dependency::result_type_without_return_dependency(expected_type);
    let actual_key = name_key(actual_type);
    if actual_key.is_empty() {
        return false;
    }
    if actual_key == name_key(&expected_type) {
        return true;
    }
    if actual_key == "integer_literal" {
        return matches!(name_key(&expected_type).as_str(), "int" | "uint" | "float");
    }
    actual_key == "list_literal" && name_key(&expected_type).starts_with("list")
}

fn type_tokens(type_text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in type_text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn type_fact(type_text: impl Into<String>, source: &'static str) -> TypeFact {
    TypeFact {
        type_text: type_text.into(),
        source,
    }
}

fn expression_text_for_statement(statement: &BodyStatement) -> Option<&str> {
    match statement.kind {
        "return" => strip_keyword(&statement.text, "return"),
        "fail" => strip_keyword(&statement.text, "fail"),
        "let_binding" | "mutable_binding" | "set_place" => statement
            .text
            .split_once('=')
            .map(|(_left, value)| value.trim()),
        "if_header" => header_body(&statement.text, "if"),
        "while_header" => header_body(&statement.text, "while"),
        "for_each_header" => header_body(&statement.text, "for each"),
        "for_index_header" => header_body(&statement.text, "for index"),
        "record_field_initializer" => statement
            .text
            .split_once(':')
            .map(|(_field, value)| value.trim()),
        "test_expectation" => strip_keyword(&statement.text, "expect"),
        _ => None,
    }
}

fn header_body<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = strip_keyword(text, keyword)?;
    rest.strip_suffix('{').map(str::trim)
}

fn strip_keyword<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    if text == keyword {
        return Some("");
    }
    text.strip_prefix(keyword)
        .and_then(|rest| rest.strip_prefix(char::is_whitespace))
        .map(str::trim)
}

fn item_sections(item: &Item) -> &[Section] {
    match item {
        Item::App(app) => &app.sections,
        Item::Type(type_def) => &type_def.sections,
        Item::Store(store) => &store.sections,
        Item::Task(task) => &task.sections,
        Item::Test(test) => &test.sections,
    }
}

fn item_params(item: &Item) -> &[Param] {
    match item {
        Item::Task(task) => &task.params,
        Item::Test(test) => &test.params,
        _ => &[],
    }
}

fn item_result(item: &Item) -> Option<&str> {
    match item {
        Item::Task(task) => task.result.as_deref(),
        _ => None,
    }
}

fn item_status(statements: &[TypedStatement], blocked: bool) -> &'static str {
    if blocked {
        "blocked_by_prior_errors"
    } else if statements
        .iter()
        .any(|statement| statement.status.starts_with("rejected_"))
    {
        "full_type_errors_v0"
    } else if statements.iter().any(is_blocking_statement) {
        "blocked_by_unchecked_body_types_v0"
    } else {
        "recognized_core_body_types_checked_v0"
    }
}

fn is_blocking_statement(statement: &TypedStatement) -> bool {
    matches!(
        statement.status,
        "rejected_statement_type_mismatch_v0"
            | "rejected_typed_failure_relationship_v0"
            | "rejected_invalid_stdout_write_call_v0"
            | "rejected_invalid_clock_replay_call_v0"
            | "rejected_invalid_files_read_text_call_v0"
            | "unchecked_statement_type_v0"
            | "blocked_unsupported_statement_v0"
            | "not_checked_blocked_by_prior_errors_v0"
    )
}

impl FullTypeCheckReport {
    fn status(&self) -> &'static str {
        if self.source_errors > 0 {
            "blocked_by_source_errors"
        } else if self.type_check_summary.resolver_errors > 0 {
            "blocked_by_resolver_errors"
        } else if self.type_check_summary.type_errors > 0 {
            "blocked_by_type_errors"
        } else if self.core_verify_summary.failed_checks > 0 {
            "blocked_by_core_verify_errors"
        } else if self.rejected_statements() > 0 || self.rejected_predicates() > 0 {
            "full_type_errors_v0"
        } else if self.unchecked_statements() > 0 || self.unsupported_statements() > 0 {
            "blocked_by_unchecked_body_types_v0"
        } else {
            "recognized_core_body_types_checked_v0"
        }
    }

    fn files(&self) -> usize {
        self.files
    }

    fn item_count(&self) -> usize {
        self.item_count
    }

    fn statement_count(&self) -> usize {
        self.items.iter().map(|item| item.statements.len()).sum()
    }

    fn checked_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                matches!(
                    statement.status,
                    "accepted_statement_type_v0"
                        | "accepted_inferred_binding_type_v0"
                        | "accepted_writable_field_alias_candidate_deferred_to_ownership_v0"
                        | "accepted_no_expression_type_obligation_v0"
                        | "accepted_same_root_failure_propagation_v0"
                        | "accepted_causal_failure_wrap_v0"
                        | "accepted_nominal_direct_failure_v0"
                        | "accepted_typed_failure_deferred_to_effect_v0"
                        | "rejected_typed_failure_relationship_v0"
                        | "rejected_statement_type_mismatch_v0"
                        | "rejected_invalid_stdout_write_call_v0"
                        | "rejected_invalid_clock_replay_call_v0"
                        | "rejected_invalid_files_read_text_call_v0"
                )
            })
            .count()
    }

    fn accepted_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                matches!(
                    statement.status,
                    "accepted_statement_type_v0"
                        | "accepted_inferred_binding_type_v0"
                        | "accepted_writable_field_alias_candidate_deferred_to_ownership_v0"
                        | "accepted_no_expression_type_obligation_v0"
                        | "accepted_same_root_failure_propagation_v0"
                        | "accepted_causal_failure_wrap_v0"
                        | "accepted_nominal_direct_failure_v0"
                        | "accepted_typed_failure_deferred_to_effect_v0"
                )
            })
            .count()
    }

    fn rejected_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                matches!(
                    statement.status,
                    "rejected_statement_type_mismatch_v0"
                        | "rejected_typed_failure_relationship_v0"
                        | "rejected_invalid_stdout_write_call_v0"
                        | "rejected_invalid_clock_replay_call_v0"
                        | "rejected_invalid_files_read_text_call_v0"
                )
            })
            .count()
    }

    fn unchecked_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| statement.status == "unchecked_statement_type_v0")
            .count()
    }

    fn unsupported_statements(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| item.statements.iter())
            .filter(|statement| {
                matches!(
                    statement.status,
                    "blocked_unsupported_statement_v0" | "not_checked_blocked_by_prior_errors_v0"
                )
            })
            .count()
    }

    fn blocking_issues(&self) -> usize {
        debug_assert!(
            self.items
                .iter()
                .flat_map(|item| item.statements.iter())
                .filter_map(|statement| statement.prior_blocker.as_ref())
                .all(|prior| !prior.occurrence_id.as_str().is_empty())
        );
        self.source_errors
            + self.type_check_summary.resolver_errors
            + self.type_check_summary.type_errors
            + self.core_verify_summary.failed_checks
            + self.rejected_statements()
            + self.unchecked_statements()
            + self.unsupported_statements()
            + self.rejected_predicates()
    }

    fn rejected_predicates(&self) -> usize {
        self.predicates.iter().filter(|fact| fact.blocks()).count()
    }
}

fn push_predicates(out: &mut String, facts: &[PredicateFact], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "predicate_facts");
    out.push_str(": [\n");
    for (index, fact) in facts.iter().enumerate() {
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "task", &fact.task, true);
        push_span_field(out, indent + 4, "task_span", &fact.task_span, true);
        push_string_field(out, indent + 4, "section", &fact.section, true);
        push_string_field(out, indent + 4, "text", &fact.text, true);
        push_string_field(
            out,
            indent + 4,
            "predicate_recognition_status",
            fact.status.as_str(),
            true,
        );
        push_string_field(out, indent + 4, "reason", fact.reason, true);
        push_string_field(
            out,
            indent + 4,
            "diagnostic_code",
            if fact.blocks() { "H0704" } else { "none" },
            true,
        );
        push_indent(out, indent + 4);
        push_json_string(out, "places");
        out.push_str(": [");
        for (place_index, place) in fact.places.iter().enumerate() {
            if place_index > 0 {
                out.push_str(", ");
            }
            out.push('{');
            push_json_string(out, "text");
            out.push_str(": ");
            push_json_string(out, &place.text);
            out.push_str(", ");
            push_json_string(out, "span");
            out.push_str(": {");
            push_json_string(out, "file");
            out.push_str(": ");
            push_json_string(out, &place.span.file.replace('\\', "/"));
            out.push_str(&format!(
                ", \"line\": {}, \"column\": {}",
                place.span.line, place.span.column
            ));
            out.push_str("}, ");
            for (name, value) in [
                ("scope_id", Some(place.scope_id.as_str())),
                ("root_definition_id", place.root_definition_id.as_deref()),
                ("definition_id", place.definition_id.as_deref()),
                ("resolution", Some(place.resolution)),
                ("eligibility", Some(place.eligibility)),
                ("type", place.type_text.as_deref()),
            ] {
                push_json_string(out, name);
                out.push_str(": ");
                push_json_string(out, value.unwrap_or("none"));
                if name != "type" {
                    out.push_str(", ");
                }
            }
            out.push('}');
        }
        out.push_str("],\n");
        push_string_field(
            out,
            indent + 4,
            "expected",
            fact.expected.as_deref().unwrap_or("none"),
            true,
        );
        push_string_field(
            out,
            indent + 4,
            "actual",
            fact.actual.as_deref().unwrap_or("none"),
            true,
        );
        push_string_field(
            out,
            indent + 4,
            "comparison_operator",
            fact.comparison.unwrap_or("none"),
            true,
        );
        push_string_field(
            out,
            indent + 4,
            "left_type",
            fact.left_type.as_deref().unwrap_or("unknown"),
            true,
        );
        push_string_field(
            out,
            indent + 4,
            "right_type",
            fact.right_type.as_deref().unwrap_or("unknown"),
            true,
        );
        push_string_field(out, indent + 4, "repair", &fact.repair(), true);
        push_optional_span_field(
            out,
            indent + 4,
            "intent_span",
            fact.intent_span.as_ref(),
            true,
        );
        push_optional_span_field(
            out,
            indent + 4,
            "offending_span",
            fact.offending_span.as_ref(),
            true,
        );
        push_usize_field(
            out,
            indent + 4,
            "delimiter_depth",
            fact.delimiter_depth,
            true,
        );
        push_span_field(out, indent + 4, "line_span", &fact.line_span, false);
        push_indent(out, indent + 2);
        out.push('}');
        push_comma_newline(out, index + 1 < facts.len());
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn count_items(program: &Program) -> usize {
    program
        .files
        .iter()
        .map(|file| count_items_in(&file.items))
        .sum()
}

fn count_items_in(items: &[Item]) -> usize {
    items
        .iter()
        .map(|item| {
            1 + match item {
                Item::App(app) => count_items_in(&app.items),
                _ => 0,
            }
        })
        .sum()
}

fn portable_span(span: &Span) -> Span {
    Span {
        file: span.file.replace('\\', "/"),
        line: span.line,
        column: span.column,
    }
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

fn name_key(name: &str) -> String {
    snake_identifier(name)
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

fn push_dependency_summaries(
    out: &mut String,
    report: &FullTypeCheckReport,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "dependencies");
    out.push_str(": {\n");
    push_indent(out, indent + 2);
    push_json_string(out, "type_check");
    out.push_str(": {\n");
    push_string_field(
        out,
        indent + 4,
        "schema",
        report.type_check_summary.schema,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "status",
        report.type_check_summary.status,
        true,
    );
    push_usize_field(
        out,
        indent + 4,
        "type_errors",
        report.type_check_summary.type_errors,
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("},\n");
    push_indent(out, indent + 2);
    push_json_string(out, "core_verify");
    out.push_str(": {\n");
    push_string_field(
        out,
        indent + 4,
        "schema",
        report.core_verify_summary.schema,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "status",
        report.core_verify_summary.status,
        true,
    );
    push_usize_field(
        out,
        indent + 4,
        "failed_checks",
        report.core_verify_summary.failed_checks,
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("}\n");
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_summary(out: &mut String, report: &FullTypeCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "summary");
    out.push_str(": {\n");
    push_usize_field(out, indent + 2, "files", report.files(), true);
    push_usize_field(out, indent + 2, "items", report.item_count(), true);
    push_usize_field(out, indent + 2, "body_items", report.items.len(), true);
    push_usize_field(
        out,
        indent + 2,
        "statements",
        report.statement_count(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "checked_statements",
        report.checked_statements(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "accepted_statements",
        report.accepted_statements(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_statements",
        report.rejected_statements(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_statements",
        report.unchecked_statements(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unsupported_statements",
        report.unsupported_statements(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocking_issues",
        report.blocking_issues(),
        true,
    );
    push_usize_field(out, indent + 2, "source_errors", report.source_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        report.type_check_summary.resolver_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_errors",
        report.type_check_summary.type_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "core_verify_errors",
        report.core_verify_summary.failed_checks,
        true,
    );
    push_usize_field(out, indent + 2, "execution_ready", 0, true);
    push_usize_field(out, indent + 2, "ir_ready", 0, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_items(out: &mut String, items: &[FullTypeItem], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "typed_items");
    out.push_str(": [");
    if !items.is_empty() {
        out.push('\n');
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_item(out, item, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_item(out: &mut String, item: &FullTypeItem, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &item.id, true);
    push_string_field(out, indent + 2, "kind", item.kind, true);
    push_string_field(out, indent + 2, "name", &item.name, true);
    push_span_field(out, indent + 2, "source_span", &item.span, true);
    push_string_field(out, indent + 2, "status", item.status, true);
    push_statements(out, &item.statements, indent + 2, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_statements(out: &mut String, statements: &[TypedStatement], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "statements");
    out.push_str(": [");
    if !statements.is_empty() {
        out.push('\n');
        for (index, statement) in statements.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_statement(out, statement, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_statement(out: &mut String, statement: &TypedStatement, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &statement.id, true);
    push_span_field(out, indent + 2, "source_span", &statement.span, true);
    push_string_field(
        out,
        indent + 2,
        "statement_kind",
        statement.statement_kind,
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "expression_text",
        statement.expression_text.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "expected_type",
        statement.expected_type.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "actual_type",
        statement.actual_type.as_deref(),
        true,
    );
    push_optional_string_field(out, indent + 2, "type_source", statement.type_source, true);
    push_string_field(out, indent + 2, "status", statement.status, true);
    push_optional_string_field(out, indent + 2, "reason", statement.reason, true);
    push_optional_string_field(
        out,
        indent + 2,
        "failure_form",
        statement.failure_form,
        true,
    );
    push_optional_string_field(out, indent + 2, "callee", statement.callee.as_deref(), true);
    push_optional_string_field(
        out,
        indent + 2,
        "callee_result_root",
        statement.callee_result_root.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "caller_result_root",
        statement.caller_result_root.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "wrapper_root",
        statement.wrapper_root.as_deref(),
        true,
    );
    push_optional_span_field(
        out,
        indent + 2,
        "call_span",
        statement.call_span.as_ref(),
        true,
    );
    push_optional_span_field(
        out,
        indent + 2,
        "callee_span",
        statement.callee_span.as_ref(),
        true,
    );
    push_optional_span_field(
        out,
        indent + 2,
        "caller_span",
        statement.caller_span.as_ref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "diagnostic_code",
        statement.diagnostic_code,
        true,
    );
    push_optional_string_field(out, indent + 2, "help", statement.help.as_deref(), false);
    push_indent(out, indent);
    out.push('}');
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

fn push_optional_span_field(
    out: &mut String,
    indent: usize,
    key: &str,
    span: Option<&Span>,
    comma: bool,
) {
    match span {
        Some(span) => push_span_field(out, indent, key, span, comma),
        None => {
            push_indent(out, indent);
            push_json_string(out, key);
            out.push_str(": null");
            push_comma_newline(out, comma);
        }
    }
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

fn push_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    out.push_str(&value.to_string());
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

    use super::{
        build_report, full_type_check_has_errors, full_type_check_json, full_type_check_text,
    };
    use crate::diagnostic::DiagnosticCode;

    #[test]
    fn json_accepts_recognized_task_body_types_without_execution_claims() {
        let program = typed_demo_program();
        let json = full_type_check_json(&program, &[]);

        assert!(!full_type_check_has_errors(&program, &[]));
        assert!(json.contains("\"schema\": \"hum.full_type_check.v0\""));
        assert!(json.contains("\"status\": \"recognized_core_body_types_checked_v0\""));
        assert!(json.contains("\"accepted_statement_type_v0\""));
        assert!(json.contains("\"accepted_inferred_binding_type_v0\""));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"ir_ready\": 0"));
        assert!(json.contains("\"no executable semantics\""));
    }

    #[test]
    fn json_blocks_unknown_and_unsupported_body_types() {
        let program = reference_like_program();
        let json = full_type_check_json(&program, &[]);

        assert!(full_type_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"blocked_by_unchecked_body_types_v0\""));
        assert!(json.contains("\"blocked_unsupported_statement_v0\""));
        assert!(json.contains("\"record_field_context_not_tracked_v0\""));
        assert!(json.contains("\"surface_save_requires_store_lowering\""));
    }

    #[test]
    fn unknown_writable_alias_candidates_defer_without_weakening_other_unknown_bindings() {
        let alias_program = Program {
            files: vec![
                parse_source(
                    "alias_defer.hum",
                    r#"type Point {
  x: UInt
  y: UInt
}

task alias_defer(change point: Point) -> Point {
  does:
    let first = change point.x
    let second = change first.y
    let nested = change point.x.deep
    return point
}
"#,
                )
                .file,
            ],
        };
        let alias_json = full_type_check_json(&alias_program, &[]);
        assert!(!full_type_check_has_errors(&alias_program, &[]));
        assert!(alias_json.contains("\"status\": \"recognized_core_body_types_checked_v0\""));
        assert!(alias_json.contains(
            "\"status\": \"accepted_writable_field_alias_candidate_deferred_to_ownership_v0\""
        ));
        assert!(
            alias_json
                .contains("\"reason\": \"writable_field_alias_shape_deferred_to_ownership_v0\"")
        );
        assert!(alias_json.contains("\"unchecked_statements\": 0"));

        let ordinary_program = Program {
            files: vec![
                parse_source(
                    "ordinary_unknown.hum",
                    r#"type Point {
  x: UInt
}

task ordinary_unknown(change point: Point) -> Point {
  does:
    let ordinary = point.x.deep
    return point
}
"#,
                )
                .file,
            ],
        };
        let ordinary_json = full_type_check_json(&ordinary_program, &[]);
        assert!(full_type_check_has_errors(&ordinary_program, &[]));
        assert!(ordinary_json.contains("\"status\": \"blocked_by_unchecked_body_types_v0\""));
        assert!(ordinary_json.contains("\"reason\": \"binding_initializer_type_unknown_v0\""));
    }

    #[test]
    fn text_reports_full_type_gate_without_safety_claims() {
        let program = typed_demo_program();
        let text = full_type_check_text(&program, &[]);

        assert!(text.contains("Hum full type check (hum.full_type_check.v0)"));
        assert!(text.contains("status: recognized_core_body_types_checked_v0"));
        assert!(text.contains("no memory-safety proof"));
    }

    #[test]
    fn ao_full_type_consumes_exact_typed_failure_occurrences_and_defers_h0907() {
        let h0901 = typed_failure_program(false);
        let report = build_report(&h0901, &[]);
        let statement = report
            .items
            .iter()
            .flat_map(|item| item.statements.iter())
            .find(|statement| statement.diagnostic_code == Some("H0901"))
            .expect("H0901 statement");
        let occurrence = crate::typed_failure::analyze_program(&h0901)
            .occurrences()
            .into_iter()
            .find(|occurrence| occurrence.code == DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY)
            .expect("H0901 occurrence");
        statement
            .prior_blocker
            .as_ref()
            .expect("full type prior")
            .validate_against(&occurrence)
            .expect("exact H0901 prior");

        let h0907 = typed_failure_program(true);
        let report = build_report(&h0907, &[]);
        let statement = report
            .items
            .iter()
            .flat_map(|item| item.statements.iter())
            .find(|statement| statement.status == "accepted_typed_failure_deferred_to_effect_v0")
            .expect("deferred H0907 statement");
        assert_eq!(statement.diagnostic_code, None);
        let occurrence = crate::typed_failure::analyze_program(&h0907)
            .occurrences()
            .into_iter()
            .find(|occurrence| occurrence.code == DiagnosticCode::MISSING_FAILURE_DECLARATION)
            .expect("H0907 occurrence");
        statement
            .prior_blocker
            .as_ref()
            .expect("deferred H0907 prior")
            .validate_against(&occurrence)
            .expect("exact H0907 prior");
    }

    fn typed_failure_program(explicit_try_without_declaration: bool) -> Program {
        let call = if explicit_try_without_declaration {
            "try source()"
        } else {
            "source()"
        };
        let caller_failure = if explicit_try_without_declaration {
            ""
        } else {
            "  fails when:\n    the caller source fails\n"
        };
        let source = format!(
            r#"module tests.ao.full_type

type SourceError {{
  code: Text
}}

task source() -> Result UInt, SourceError {{
  fails when:
    the source fails
  does:
    fail SourceError.origin
}}

task caller() -> Result UInt, SourceError {{
{caller_failure}  does:
    let value = {call}
    return value
}}
"#
        );
        Program {
            files: vec![parse_source("session_ao_full_type.hum", &source).file],
        }
    }

    fn typed_demo_program() -> crate::ast::Program {
        Program {
            files: vec![
                parse_source(
                    "typed_demo.hum",
                    r#"type WorkItem {
  title: Text
}

type WorkError {
  code: Text
}

task remember(item: WorkItem) -> Result WorkItem, WorkError {
  why:
    keep a typed body small

  needs:
    item exists

  ensures:
    item is returned

  fails when:
    item is missing

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    if item is missing {
      fail WorkError.empty_title
    }

    let checked = item
    return checked
}
"#,
                )
                .file,
            ],
        }
    }

    fn reference_like_program() -> crate::ast::Program {
        Program {
            files: vec![
                parse_source(
                    "reference_like.hum",
                    r#"type WorkItem {
  title: Text
  done: Bool
}

type WorkError {
  code: Text
}

store work_items: list WorkItem {
  why:
    keep work_items
}

task remember(title: Text) -> Result WorkItem, WorkError {
  why:
    keep a typed body small

  needs:
    title exists

  changes:
    work_items

  ensures:
    item is returned

  fails when:
    title is empty

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    let item = WorkItem {
      title: title
      done: false
    }

    save item in work_items
    return item
}
"#,
                )
                .file,
            ],
        }
    }
}
