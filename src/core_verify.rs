use crate::ast::Program;
use crate::callable;
use crate::core_contract;
use crate::core_expr;
use crate::core_lower::{self, CoreLowerItem, CoreLowerOperation, CoreLowerReport};
use crate::core_preview;
use crate::diagnostic::{Diagnostic, Span};
use crate::ir_contract;
use crate::predicate;
use crate::resolve;
use crate::type_check;
use crate::version;

pub const CORE_VERIFY_SCHEMA: &str = "hum.core_verify.v0";
pub const CORE_VERIFY_STATUS: &str = "verified_non_executing_core_artifact_v0";
pub const CORE_VERIFY_FAILED_STATUS: &str = "core_artifact_verification_failed_v0";
pub const CORE_VERIFY_MODE: &str = "non_executing_artifact_invariant_check_v0";

const NON_GOALS: &[&str] = &[
    "no executable semantics",
    "no Hum IR emission",
    "no backend lowering",
    "no proof artifact",
    "no memory-safety proof",
    "no optimization claim",
    "no full type checking",
    "no effect checking",
    "no ownership checking",
    "no profile enforcement",
];

pub struct CoreVerifyReadinessSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub files: usize,
    pub items: usize,
    pub tasks: usize,
    pub tests: usize,
    pub core_items: usize,
    pub verified_items: usize,
    pub lower_blocked_items: usize,
    pub operations: usize,
    pub verified_operations: usize,
    pub lower_blocked_operations: usize,
    pub checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub execution_ready: usize,
    pub ir_ready: usize,
    pub errors: usize,
    pub warnings: usize,
    pub resolver_errors: usize,
    pub type_errors: usize,
    pub preview_blocked_statements: usize,
}

struct CoreVerifyReport {
    lower: CoreLowerReport,
    checks: Vec<CoreVerifyCheck>,
}

struct CoreVerifyCheck {
    id: String,
    scope: &'static str,
    scope_id: String,
    span: Option<Span>,
    status: &'static str,
    rule: &'static str,
    detail: String,
}

pub fn core_verify_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum Core verify ({CORE_VERIFY_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!(
        "verification_status: {}\nmode: {CORE_VERIFY_MODE}\ncore_contract_schema: {}\ncore_lower_schema: {}\ncore_preview_schema: {}\nir_contract_schema: {}\n",
        report.verification_status(),
        core_contract::CORE_CONTRACT_SCHEMA,
        core_lower::CORE_LOWER_SCHEMA,
        core_preview::CORE_PREVIEW_SCHEMA,
        ir_contract::IR_CONTRACT_SCHEMA
    ));
    out.push_str(&format!(
        "summary: files={} items={} tasks={} tests={} core_items={} verified_items={} lower_blocked_items={} operations={} verified_operations={} lower_blocked_operations={} checks={} passed_checks={} failed_checks={} execution_ready=0 ir_ready=0 errors={} warnings={} resolver_errors={} type_errors={} preview_blocked_statements={}\n",
        report.lower.files,
        report.lower.items,
        report.lower.tasks,
        report.lower.tests,
        report.lower.core_items.len(),
        report.verified_items(),
        report.lower.blocked_items(),
        report.operations(),
        report.verified_operations(),
        report.lower.blocked_operations(),
        report.checks.len(),
        report.passed_checks(),
        report.failed_checks(),
        report.lower.errors,
        report.lower.warnings,
        report.lower.resolver_errors,
        report.lower.type_errors,
        report.lower.preview_blocked_statements
    ));
    out.push_str(&format!(
        "core_lower: schema={} status={} lowered_items={} blocked_items={} lowered_operations={} blocked_operations={} execution_ready=0 ir_ready=0\n",
        core_lower::CORE_LOWER_SCHEMA,
        core_lower::CORE_LOWER_STATUS,
        report.lower.lowered_items(),
        report.lower.blocked_items(),
        report.lower.lowered_operations(),
        report.lower.blocked_operations()
    ));

    if report.lower.core_items.is_empty() {
        out.push_str("core_items: none\n");
    } else {
        out.push_str("core_items:\n");
        for item in &report.lower.core_items {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} `{}` lower_status={} operations={} blockers={}\n",
                item.span.file,
                item.span.line,
                item.span.column,
                report.item_status(item),
                item.kind,
                item.name,
                item.status,
                item.operations.len(),
                item.blockers.len()
            ));
        }
    }

    if report.failed_checks() == 0 {
        out.push_str("verification_failures: none\n");
    } else {
        out.push_str("verification_failures:\n");
        for check in report
            .checks
            .iter()
            .filter(|check| check.status == "failed_v0")
        {
            if let Some(span) = &check.span {
                out.push_str(&format!(
                    "  {}:{}:{} [{}] {}: {}\n",
                    span.file, span.line, span.column, check.rule, check.scope_id, check.detail
                ));
            } else {
                out.push_str(&format!(
                    "  [{}] {}: {}\n",
                    check.rule, check.scope_id, check.detail
                ));
            }
        }
    }

    out.push_str(&predicate::analyze_program(program).place_facts_text());
    out
}

pub fn core_verify_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", CORE_VERIFY_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(
        &mut out,
        2,
        "verification_status",
        report.verification_status(),
        true,
    );
    push_string_field(&mut out, 2, "mode", CORE_VERIFY_MODE, true);
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
        "core_lower_schema",
        core_lower::CORE_LOWER_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "core_preview_schema",
        core_preview::CORE_PREVIEW_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "resolve_schema",
        resolve::RESOLVE_REPORT_SCHEMA,
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
        "ir_contract_schema",
        ir_contract::IR_CONTRACT_SCHEMA,
        true,
    );
    push_summary(&mut out, &report, 2, true);
    push_core_lower_summary(&mut out, &report, 2, true);
    push_items(&mut out, &report, 2, true);
    push_checks(&mut out, &report.checks, 2, true);
    push_indent(&mut out, 2);
    push_json_string(&mut out, "predicate_place_facts");
    out.push_str(": ");
    out.push_str(&predicate::analyze_program(program).place_facts_json());
    out.push_str(",\n");
    push_string_array(&mut out, 2, "non_goals_v0", NON_GOALS, false);
    out.push_str("}\n");
    out
}

pub fn core_verify_has_errors(program: &Program, diagnostics: &[Diagnostic]) -> bool {
    build_report(program, diagnostics).failed_checks() > 0
}

pub fn core_verify_readiness_summary(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> CoreVerifyReadinessSummary {
    let report = build_report(program, diagnostics);
    CoreVerifyReadinessSummary {
        schema: CORE_VERIFY_SCHEMA,
        status: report.verification_status(),
        mode: CORE_VERIFY_MODE,
        files: report.lower.files,
        items: report.lower.items,
        tasks: report.lower.tasks,
        tests: report.lower.tests,
        core_items: report.lower.core_items.len(),
        verified_items: report.verified_items(),
        lower_blocked_items: report.lower.blocked_items(),
        operations: report.operations(),
        verified_operations: report.verified_operations(),
        lower_blocked_operations: report.lower.blocked_operations(),
        checks: report.checks.len(),
        passed_checks: report.passed_checks(),
        failed_checks: report.failed_checks(),
        execution_ready: 0,
        ir_ready: 0,
        errors: report.lower.errors,
        warnings: report.lower.warnings,
        resolver_errors: report.lower.resolver_errors,
        type_errors: report.lower.type_errors,
        preview_blocked_statements: report.lower.preview_blocked_statements,
    }
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> CoreVerifyReport {
    let lower = core_lower::build_core_lower_report(program, diagnostics);
    let mut checks = verify_lower_report(&lower);
    let callable_failures = callable::analyze_program(program).verify();
    if callable_failures.is_empty() {
        push_check(
            &mut checks,
            "callable_semantic_spine",
            "session-al-callable-facts",
            None,
            true,
            "callable_closed_fact_consistency",
            "callable definition, type, row, value, and application facts are internally consistent",
        );
    } else {
        for failure in callable_failures {
            push_check(
                &mut checks,
                "callable_semantic_spine",
                "session-al-callable-facts",
                None,
                false,
                failure,
                format!("callable fact verification failed: {failure}"),
            );
        }
    }
    CoreVerifyReport { lower, checks }
}

fn verify_lower_report(lower: &CoreLowerReport) -> Vec<CoreVerifyCheck> {
    let mut checks = Vec::new();
    push_check(
        &mut checks,
        "summary",
        "core-lower-summary",
        None,
        lower.execution_ready == 0,
        "claim_honesty",
        "execution_ready remains 0",
    );
    push_check(
        &mut checks,
        "summary",
        "core-lower-summary",
        None,
        lower.ir_ready == 0,
        "claim_honesty",
        "ir_ready remains 0",
    );
    push_check(
        &mut checks,
        "summary",
        "core-lower-summary",
        None,
        core_lower::CORE_LOWER_STATUS == "unverified_core_artifact_v0",
        "claim_honesty",
        "core-lower status is explicitly unverified",
    );

    for item in &lower.core_items {
        verify_item(item, &mut checks);
    }

    checks
}

fn verify_item(item: &CoreLowerItem, checks: &mut Vec<CoreVerifyCheck>) {
    push_span_check(checks, "core_item", &item.id, &item.span);
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        !item.id.trim().is_empty(),
        "row_identity",
        "core item id is present",
    );
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        item.verification_status == "unverified_v0",
        "claim_honesty",
        "core-lower item remains unverified before core-verify",
    );
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        item.grammar_status == crate::core_body::CORE_BODY_GRAMMAR_STATUS,
        "body_grammar_consistency",
        "item keeps partial body grammar provenance",
    );
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        valid_item_status(item.status),
        "item_status_known",
        format!("item status `{}` is known to core-verify", item.status),
    );
    push_check(
        checks,
        "core_item",
        &item.id,
        Some(&item.span),
        item_status_consistent(item),
        "item_status_consistent",
        format!(
            "item lower status `{}` agrees with operations and blockers",
            item.status
        ),
    );

    for (expected_index, operation) in item.operations.iter().enumerate() {
        verify_operation(item, operation, expected_index, checks);
    }
    for blocker in &item.blockers {
        push_span_check(checks, "blocker", &item.id, &blocker.span);
        push_check(
            checks,
            "blocker",
            &item.id,
            Some(&blocker.span),
            valid_blocker_status(blocker.status),
            "blocker_status_known",
            format!(
                "blocker `{}` has known status `{}`",
                blocker.reason, blocker.status
            ),
        );
    }
}

fn verify_operation(
    item: &CoreLowerItem,
    operation: &CoreLowerOperation,
    expected_index: usize,
    checks: &mut Vec<CoreVerifyCheck>,
) {
    push_span_check(checks, "operation", &operation.id, &operation.span);
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        !operation.id.trim().is_empty(),
        "row_identity",
        "operation id is present",
    );
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        operation.index == expected_index,
        "operation_index_consistent",
        format!("operation index is {}", operation.index),
    );
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        operation_status_consistent(operation),
        "operation_family_status_consistent",
        format!(
            "{} uses status {}",
            operation.core_operation, operation.status
        ),
    );
    push_check(
        checks,
        "operation",
        &operation.id,
        Some(&operation.span),
        source_status_consistent(operation),
        "source_status_consistent",
        format!(
            "source kind {} with source status {} maps to {}",
            operation.source_kind, operation.source_status, operation.status
        ),
    );

    if operation.status == "blocked_operation_v0" {
        let detail = operation.reason.unwrap_or("missing_blocker_reason");
        push_check(
            checks,
            "operation",
            &operation.id,
            Some(&operation.span),
            operation.reason.is_some(),
            "blocked_operation_has_reason",
            format!("blocked operation reason: {detail}"),
        );
        push_check(
            checks,
            "operation",
            &operation.id,
            Some(&operation.span),
            has_matching_blocker(item, operation),
            "blocked_operation_has_matching_blocker",
            format!("blocked operation has matching blocker: {detail}"),
        );
    }

    match &operation.expression {
        Some(expression) => {
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                operation.source_status != "unsupported_v0",
                "expression_source_status_consistent",
                "unsupported source rows do not carry expression previews",
            );
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                valid_expression_status(expression.status),
                "expression_status_known",
                format!("expression status `{}` is known", expression.status),
            );
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                valid_expression_ast_status(expression.ast_status),
                "expression_ast_status_known",
                format!("expression AST status `{}` is known", expression.ast_status),
            );
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                expression.node_count > 0,
                "expression_ast_present",
                "expression AST root is present",
            );
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                valid_type_status(expression.type_status),
                "type_claim_honesty",
                format!(
                    "type status `{}` is provenance-limited",
                    expression.type_status
                ),
            );
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                matches!(
                    expression.effect_status,
                    core_expr::CORE_EXPRESSION_EFFECT_STATUS
                        | core_expr::CORE_PREDICATE_EFFECT_STATUS
                ),
                "effect_claim_honesty",
                "expression effects remain not checked",
            );
        }
        None => {
            push_check(
                checks,
                "operation_expression",
                &operation.id,
                Some(&operation.span),
                operation.source_status == "unsupported_v0"
                    || blocked_operation_family(operation.core_operation)
                    || !operation_kind_requires_expression(operation.source_kind),
                "expression_absence_consistent",
                "operation expression absence is consistent with source kind",
            );
        }
    }
}

fn push_span_check(
    checks: &mut Vec<CoreVerifyCheck>,
    scope: &'static str,
    scope_id: &str,
    span: &Span,
) {
    push_check(
        checks,
        scope,
        scope_id,
        Some(span),
        span_is_sane(span),
        "source_span_sane",
        "source span has file, line, and column",
    );
}

fn push_check(
    checks: &mut Vec<CoreVerifyCheck>,
    scope: &'static str,
    scope_id: &str,
    span: Option<&Span>,
    passed: bool,
    rule: &'static str,
    detail: impl Into<String>,
) {
    checks.push(CoreVerifyCheck {
        id: format!("core-verify-check-{}", checks.len() + 1),
        scope,
        scope_id: scope_id.to_string(),
        span: span.cloned(),
        status: if passed { "passed_v0" } else { "failed_v0" },
        rule,
        detail: detail.into(),
    });
}

fn span_is_sane(span: &Span) -> bool {
    !span.file.trim().is_empty() && span.line > 0 && span.column > 0
}

fn valid_item_status(status: &str) -> bool {
    matches!(
        status,
        "lowered_unverified_core_v0"
            | "blocked_by_source_errors"
            | "blocked_by_resolver_errors"
            | "blocked_by_type_errors"
            | "blocked_before_core_execution"
            | "empty_body"
    )
}

fn item_status_consistent(item: &CoreLowerItem) -> bool {
    let blocked_operations = item
        .operations
        .iter()
        .filter(|operation| operation.status == "blocked_operation_v0")
        .count();
    match item.status {
        "lowered_unverified_core_v0" => item.blockers.is_empty() && blocked_operations == 0,
        "blocked_by_source_errors" => item
            .blockers
            .iter()
            .any(|blocker| blocker.status == "blocked_by_source_errors"),
        "blocked_by_resolver_errors" => item
            .blockers
            .iter()
            .any(|blocker| blocker.status == "blocked_by_resolver_errors"),
        "blocked_by_type_errors" => item
            .blockers
            .iter()
            .any(|blocker| blocker.status == "blocked_by_type_errors"),
        "blocked_before_core_execution" => !item.blockers.is_empty(),
        "empty_body" => item
            .blockers
            .iter()
            .any(|blocker| blocker.status == "empty_body"),
        _ => false,
    }
}

fn operation_status_consistent(operation: &CoreLowerOperation) -> bool {
    match operation.status {
        "lowered_unverified_operation_v0" => lowered_operation_family(operation.core_operation),
        "blocked_operation_v0" => blocked_operation_family(operation.core_operation),
        _ => false,
    }
}

fn source_status_consistent(operation: &CoreLowerOperation) -> bool {
    match operation.source_status {
        "recognized_v0" => true,
        "recognized_typed_executable_predicate_v2" => {
            operation.core_operation == "checked_contract_predicate_v2"
                && operation.status == "lowered_unverified_operation_v0"
        }
        "malformed_executable_predicate_v2" | "rejected_executable_predicate_semantics_v2" => {
            operation.core_operation == "blocked_contract_predicate_v2"
                && operation.status == "blocked_operation_v0"
        }
        "unsupported_v0" => {
            operation.status == "blocked_operation_v0" && operation.expression.is_none()
        }
        _ => false,
    }
}

fn lowered_operation_family(core_operation: &str) -> bool {
    matches!(
        core_operation,
        "return"
            | "fail"
            | "let_binding"
            | "mutable_binding"
            | "set_place"
            | "if_statement"
            | "while_loop"
            | "for_each"
            | "for_index"
            | "loop"
            | "block_close"
            | "checked_contract_predicate_v2"
    )
}

fn blocked_operation_family(core_operation: &str) -> bool {
    matches!(
        core_operation,
        "blocked_surface_statement"
            | "blocked_unsupported_try_expression"
            | "record_construction_field"
            | "contract_context"
            | "test_expectation"
            | "blocked_contract_predicate_v2"
    )
}

fn valid_blocker_status(status: &str) -> bool {
    matches!(
        status,
        "blocked_by_source_errors"
            | "blocked_by_resolver_errors"
            | "blocked_by_type_errors"
            | "blocked_operation_v0"
            | "empty_body"
    )
}

fn has_matching_blocker(item: &CoreLowerItem, operation: &CoreLowerOperation) -> bool {
    item.blockers.iter().any(|blocker| {
        blocker.status == "blocked_operation_v0"
            && blocker.span == operation.span
            && operation.reason == Some(blocker.reason)
    })
}

fn valid_expression_status(status: &str) -> bool {
    matches!(
        status,
        "atom_preview_v0"
            | "compound_preview_v0"
            | "contextual_preview_v0"
            | "surface_phrase_preview_v0"
            | core_expr::CORE_PREDICATE_EXPRESSION_STATUS
    )
}

fn valid_expression_ast_status(status: &str) -> bool {
    matches!(
        status,
        core_expr::CORE_EXPRESSION_AST_STATUS
            | core_expr::CORE_EXPRESSION_CONTEXTUAL_AST_STATUS
            | core_expr::CORE_EXPRESSION_SURFACE_AST_STATUS
            | core_expr::CORE_PREDICATE_AST_STATUS
    )
}

fn valid_type_status(status: &str) -> bool {
    matches!(
        status,
        core_expr::CORE_EXPRESSION_TYPE_STATUS
            | core_expr::CORE_EXPRESSION_CHECKED_TRIVIAL_RETURN_TYPE_STATUS
            | core_expr::CORE_EXPRESSION_CHECKED_TRIVIAL_RETURN_MISMATCH_STATUS
            | core_expr::CORE_PREDICATE_TYPE_STATUS
    )
}

fn operation_kind_requires_expression(source_kind: &str) -> bool {
    matches!(
        source_kind,
        "return"
            | "fail"
            | "let_binding"
            | "mutable_binding"
            | "set_place"
            | "if_header"
            | "while_header"
            | "for_each_header"
            | "for_index_header"
            | "record_field_initializer"
            | "test_expectation"
    )
}

impl CoreVerifyReport {
    fn verification_status(&self) -> &'static str {
        if self.failed_checks() == 0 {
            CORE_VERIFY_STATUS
        } else {
            CORE_VERIFY_FAILED_STATUS
        }
    }

    fn passed_checks(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| check.status == "passed_v0")
            .count()
    }

    fn failed_checks(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| check.status == "failed_v0")
            .count()
    }

    fn operations(&self) -> usize {
        self.lower
            .core_items
            .iter()
            .map(|item| item.operations.len())
            .sum()
    }

    fn verified_items(&self) -> usize {
        if self.failed_checks() == 0 {
            self.lower.core_items.len()
        } else {
            0
        }
    }

    fn verified_operations(&self) -> usize {
        if self.failed_checks() == 0 {
            self.operations()
        } else {
            0
        }
    }

    fn item_status(&self, item: &CoreLowerItem) -> &'static str {
        if self.failed_checks() == 0 || !self.item_has_failed_check(item) {
            "verified_core_artifact_item_v0"
        } else {
            "core_artifact_item_verification_failed_v0"
        }
    }

    fn item_has_failed_check(&self, item: &CoreLowerItem) -> bool {
        self.checks.iter().any(|check| {
            check.status == "failed_v0"
                && (check.scope_id == item.id
                    || item
                        .operations
                        .iter()
                        .any(|operation| operation.id == check.scope_id))
        })
    }
}

fn push_summary(out: &mut String, report: &CoreVerifyReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"core_items\": {}, \"verified_items\": {}, \"lower_blocked_items\": {}, \"operations\": {}, \"verified_operations\": {}, \"lower_blocked_operations\": {}, \"checks\": {}, \"passed_checks\": {}, \"failed_checks\": {}, \"execution_ready\": 0, \"ir_ready\": 0, \"errors\": {}, \"warnings\": {}, \"resolver_errors\": {}, \"type_errors\": {}, \"preview_blocked_statements\": {}",
        report.lower.files,
        report.lower.items,
        report.lower.tasks,
        report.lower.tests,
        report.lower.core_items.len(),
        report.verified_items(),
        report.lower.blocked_items(),
        report.operations(),
        report.verified_operations(),
        report.lower.blocked_operations(),
        report.checks.len(),
        report.passed_checks(),
        report.failed_checks(),
        report.lower.errors,
        report.lower.warnings,
        report.lower.resolver_errors,
        report.lower.type_errors,
        report.lower.preview_blocked_statements
    ));
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_core_lower_summary(
    out: &mut String,
    report: &CoreVerifyReport,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"core_lower\": {\n");
    push_string_field(
        out,
        indent + 2,
        "schema",
        core_lower::CORE_LOWER_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 2,
        "status",
        core_lower::CORE_LOWER_STATUS,
        true,
    );
    push_usize_field(out, indent + 2, "files", report.lower.files, true);
    push_usize_field(out, indent + 2, "items", report.lower.items, true);
    push_usize_field(out, indent + 2, "tasks", report.lower.tasks, true);
    push_usize_field(out, indent + 2, "tests", report.lower.tests, true);
    push_usize_field(
        out,
        indent + 2,
        "core_items",
        report.lower.core_items.len(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "lowered_items",
        report.lower.lowered_items(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocked_items",
        report.lower.blocked_items(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "lowered_operations",
        report.lower.lowered_operations(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "blocked_operations",
        report.lower.blocked_operations(),
        true,
    );
    push_usize_field(out, indent + 2, "execution_ready", 0, true);
    push_usize_field(out, indent + 2, "ir_ready", 0, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_items(out: &mut String, report: &CoreVerifyReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"core_items\": [\n");
    for (index, item) in report.lower.core_items.iter().enumerate() {
        push_item(
            out,
            report,
            item,
            indent + 2,
            index + 1 < report.lower.core_items.len(),
        );
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_item(
    out: &mut String,
    report: &CoreVerifyReport,
    item: &CoreLowerItem,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &item.id, true);
    push_string_field(out, indent + 2, "kind", item.kind, true);
    push_string_field(out, indent + 2, "name", &item.name, true);
    push_span_field(out, indent + 2, "source_span", &item.span, true);
    push_string_field(out, indent + 2, "lower_status", item.status, true);
    push_string_field(
        out,
        indent + 2,
        "verification_status",
        report.item_status(item),
        true,
    );
    push_usize_field(out, indent + 2, "operations", item.operations.len(), true);
    push_usize_field(out, indent + 2, "blockers", item.blockers.len(), false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_checks(out: &mut String, checks: &[CoreVerifyCheck], indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"checks\": [\n");
    for (index, check) in checks.iter().enumerate() {
        push_check_json(out, check, indent + 2, index + 1 < checks.len());
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_check_json(out: &mut String, check: &CoreVerifyCheck, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &check.id, true);
    push_string_field(out, indent + 2, "scope", check.scope, true);
    push_string_field(out, indent + 2, "scope_id", &check.scope_id, true);
    push_optional_span_field(out, indent + 2, "source_span", check.span.as_ref(), true);
    push_string_field(out, indent + 2, "status", check.status, true);
    push_string_field(out, indent + 2, "rule", check.rule, true);
    push_string_field(out, indent + 2, "detail", &check.detail, false);
    push_indent(out, indent);
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
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    if let Some(span) = span {
        push_span(out, span);
    } else {
        out.push_str("null");
    }
    push_comma_newline(out, comma);
}

fn push_span_field(out: &mut String, indent: usize, key: &str, span: &Span, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_span(out, span);
    push_comma_newline(out, comma);
}

fn push_span(out: &mut String, span: &Span) {
    out.push('{');
    out.push_str("\"file\": ");
    push_json_string(out, &span.file);
    out.push_str(&format!(
        ", \"line\": {}, \"column\": {}",
        span.line, span.column
    ));
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

fn push_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    out.push_str(&value.to_string());
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

    use super::{core_verify_json, core_verify_text};

    #[test]
    fn json_verifies_tiny_core_artifact_without_execution_claims() {
        let source = r#"task add(a: Int, b: Int) -> Int {
  does:
    return a + b
}
"#;
        let parsed = parse_source("add.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = core_verify_json(&program, &parsed.diagnostics);

        assert!(json.contains("\"schema\": \"hum.core_verify.v0\""));
        assert!(json.contains("\"core_lower_schema\": \"hum.core_lower.v0\""));
        assert!(
            json.contains("\"verification_status\": \"verified_non_executing_core_artifact_v0\"")
        );
        assert!(json.contains("\"mode\": \"non_executing_artifact_invariant_check_v0\""));
        assert!(json.contains("\"rule\": \"source_span_sane\""));
        assert!(json.contains("\"rule\": \"operation_family_status_consistent\""));
        assert!(json.contains("\"rule\": \"claim_honesty\""));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"ir_ready\": 0"));
        assert!(json.contains("\"failed_checks\": 0"));
        assert!(json.contains("\"no Hum IR emission\""));
        assert!(json.contains("\"no memory-safety proof\""));
        assert!(json.contains("\"no optimization claim\""));
    }

    #[test]
    fn text_and_json_verify_blocked_lowering_rows_as_honest_blockers() {
        let source = r#"type WorkItem {
  id: Text
}

store work: list WorkItem {
  why:
    keep work
}

task remember(item: WorkItem) -> WorkItem {
  changes:
    work

  does:
    save item in work
    return item
}
"#;
        let parsed = parse_source("blocked.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let text = core_verify_text(&program, &parsed.diagnostics);
        let json = core_verify_json(&program, &parsed.diagnostics);

        assert!(text.contains("Hum Core verify (hum.core_verify.v0)"));
        assert!(text.contains("verification_failures: none"));
        assert!(
            json.contains("\"verification_status\": \"verified_non_executing_core_artifact_v0\"")
        );
        assert!(json.contains("\"lower_blocked_items\": 1"));
        assert!(json.contains("\"blocked_operations\": 1"));
        assert!(json.contains("surface_save_requires_store_lowering"));
        assert!(json.contains("blocked_operation_has_matching_blocker"));
        assert!(json.contains("\"failed_checks\": 0"));
    }
}
