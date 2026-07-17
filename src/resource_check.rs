use crate::ast::{Item, Program, Section, Task};
use crate::callable::{self, CallableAnalysis};
use crate::core_body;
use crate::diagnostic::{Diagnostic, DiagnosticOccurrenceSet, PriorBlockerRef, Severity, Span};
use crate::graph::is_meaningful_line_text;
use crate::node_id;
use crate::ownership_check;
use crate::resource_report;
use crate::version;

pub const RESOURCE_CHECK_SCHEMA: &str = "hum.resource_check.v0";
pub const RESOURCE_CHECK_MODE: &str = "recognized_core_resource_gate_v0";
pub const RESOURCE_CHECK_STATUS: &str = "recognized_core_resource_gate_available_v0";

const NON_CLAIMS: &[&str] = &[
    "no executable semantics",
    "no Hum IR emission",
    "no backend lowering",
    "no proof artifact",
    "no allocation-freedom proof",
    "no memory-safety proof",
    "no complete resource analysis",
    "no complete cost analysis",
    "no profile enforcement",
    "no optimization claim",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceCheckSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub source_errors: usize,
    pub ownership_errors: usize,
    pub resource_report_errors: usize,
    pub tasks: usize,
    pub resource_items: usize,
    pub resource_claims: usize,
    pub allocation_claims: usize,
    pub allocation_free_claims: usize,
    pub checks: usize,
    pub accepted_checks: usize,
    pub rejected_checks: usize,
    pub unchecked_checks: usize,
    pub blocking_issues: usize,
    pub proof_ready: usize,
    pub execution_ready: usize,
    pub ir_ready: usize,
}

struct ResourceCheckReport {
    ownership_check_summary: ownership_check::OwnershipCheckSummary,
    resource_report_summary: resource_report::ResourceReportSummary,
    files: usize,
    tasks: usize,
    source_errors: usize,
    items: Vec<ResourceItem>,
    prior_blockers: Vec<PriorBlockerRef>,
    diagnostic_occurrences: DiagnosticOccurrenceSet,
}

struct ResourceItem {
    id: String,
    name: String,
    graph_node_id: String,
    span: Span,
    status: &'static str,
    declarations: ResourceDeclarations,
    checks: Vec<ResourceCheck>,
}

#[derive(Default)]
struct ResourceDeclarations {
    allocations: Vec<DeclaredResource>,
    constant_space: Option<DeclaredResource>,
}

struct DeclaredResource {
    section: &'static str,
    text: String,
    normalized: String,
    span: Span,
}

struct ResourceCheck {
    id: String,
    span: Span,
    check: &'static str,
    declaration: Option<String>,
    status: &'static str,
    reason: Option<&'static str>,
}

pub fn resource_check_has_errors(program: &Program, diagnostics: &[Diagnostic]) -> bool {
    resource_check_summary(program, diagnostics).blocking_issues > 0
}

pub fn resource_check_summary(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> ResourceCheckSummary {
    let report = build_report(program, diagnostics);
    ResourceCheckSummary {
        schema: RESOURCE_CHECK_SCHEMA,
        status: report.status(),
        mode: RESOURCE_CHECK_MODE,
        source_errors: report.source_errors,
        ownership_errors: report.ownership_errors(),
        resource_report_errors: report.resource_report_summary.errors,
        tasks: report.tasks,
        resource_items: report.items.len(),
        resource_claims: report.resource_report_summary.resource_claims,
        allocation_claims: report.resource_report_summary.allocation_claims,
        allocation_free_claims: report.resource_report_summary.allocation_free_claims,
        checks: report.checks(),
        accepted_checks: report.accepted_checks(),
        rejected_checks: report.rejected_checks(),
        unchecked_checks: report.unchecked_checks(),
        blocking_issues: report.blocking_issues(),
        proof_ready: 0,
        execution_ready: 0,
        ir_ready: 0,
    }
}

pub fn resource_check_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum resource check ({RESOURCE_CHECK_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {RESOURCE_CHECK_MODE}\n"));
    out.push_str(&format!("status: {}\n", report.status()));
    out.push_str(&format!(
        "dependencies: ownership_check={} resource_report={}\n",
        ownership_check::OWNERSHIP_CHECK_SCHEMA,
        resource_report::RESOURCE_REPORT_SCHEMA
    ));
    out.push_str(&format!(
        "summary: files={} tasks={} resource_items={} resource_claims={} allocation_claims={} allocation_free_claims={} checks={} accepted_checks={} rejected_checks={} unchecked_checks={} blocking_issues={} source_errors={} ownership_errors={} resource_report_errors={} proof_ready=0 execution_ready=0 ir_ready=0\n",
        report.files,
        report.tasks,
        report.items.len(),
        report.resource_report_summary.resource_claims,
        report.resource_report_summary.allocation_claims,
        report.resource_report_summary.allocation_free_claims,
        report.checks(),
        report.accepted_checks(),
        report.rejected_checks(),
        report.unchecked_checks(),
        report.blocking_issues(),
        report.source_errors,
        report.ownership_errors(),
        report.resource_report_summary.errors,
    ));

    if report.items.is_empty() {
        out.push_str("resource_items: none\n");
    } else {
        out.push_str("resource_items:\n");
        for item in &report.items {
            out.push_str(&format!(
                "  {}:{}:{} [{}] task `{}` checks={}\n",
                item.span.file,
                item.span.line,
                item.span.column,
                item.status,
                item.name,
                item.checks.len()
            ));
            for check in &item.checks {
                out.push_str(&format!(
                    "    {}:{}:{} [{}] {}",
                    check.span.file, check.span.line, check.span.column, check.status, check.check
                ));
                if let Some(declaration) = &check.declaration {
                    out.push_str(&format!(" declaration={declaration}"));
                }
                if let Some(reason) = check.reason {
                    out.push_str(&format!(" reason={reason}"));
                }
                out.push('\n');
            }
        }
    }

    out.push_str("non_claims:\n");
    for non_claim in NON_CLAIMS {
        out.push_str(&format!("  - {non_claim}\n"));
    }

    out
}

pub fn resource_check_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", RESOURCE_CHECK_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", RESOURCE_CHECK_MODE, true);
    push_string_field(
        &mut out,
        2,
        "ownership_check_schema",
        ownership_check::OWNERSHIP_CHECK_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "resource_report_schema",
        resource_report::RESOURCE_REPORT_SCHEMA,
        true,
    );
    push_dependencies(&mut out, &report, 2, true);
    push_summary(&mut out, &report, 2, true);
    push_items(&mut out, &report.items, 2, true);
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> ResourceCheckReport {
    let ownership_check_summary = ownership_check::ownership_check_summary(program, diagnostics);
    let resource_report_summary = resource_report::resource_report_summary(program, diagnostics);
    let source_errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let blocked = source_errors > 0 || ownership_check_summary.blocking_issues > 0;
    let mut items = Vec::new();
    let mut tasks = 0;
    let callables = callable::analyze_program(program);
    for file in &program.files {
        collect_items(&file.items, blocked, &callables, &mut tasks, &mut items);
    }
    let diagnostic_occurrences = ownership_check::diagnostic_occurrence_set(program, diagnostics);
    let projection = diagnostic_projection_from_ownership(&diagnostic_occurrences)
        .expect("resource check must carry one sealed ownership projection");
    projection
        .validate_against("resource_check", &diagnostic_occurrences)
        .expect("resource check must validate its ownership authority");
    let prior_blockers = diagnostic_occurrences.prior_blockers();
    ResourceCheckReport {
        ownership_check_summary,
        resource_report_summary,
        files: program.files.len(),
        tasks,
        source_errors,
        items,
        prior_blockers,
        diagnostic_occurrences,
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
    let occurrences = ownership_check::diagnostic_occurrence_set_from_source(
        program,
        diagnostics,
        source_occurrences,
    )?;
    let projection = diagnostic_projection_from_ownership(&occurrences)?;
    projection.validate_against("resource_check", &occurrences)?;
    Ok(occurrences)
}

pub(crate) fn diagnostic_projection_from_ownership(
    occurrences: &DiagnosticOccurrenceSet,
) -> Result<crate::diagnostic::DiagnosticProjection, crate::diagnostic::DiagnosticInvariantError> {
    crate::diagnostic::DiagnosticProjection::from_upstream("resource_check", occurrences)
}

pub(crate) fn validate_prior_blocker_projection(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> Result<(), crate::diagnostic::DiagnosticInvariantError> {
    crate::effect_check::validate_static_prior_blocker_projection(program, diagnostics)?;
    let report = build_report(program, diagnostics);
    report
        .diagnostic_occurrences
        .validate_prior_blockers(&report.prior_blockers)
}

fn collect_items(
    items: &[Item],
    blocked: bool,
    callables: &CallableAnalysis,
    tasks: &mut usize,
    out: &mut Vec<ResourceItem>,
) {
    for item in items {
        match item {
            Item::App(app) => collect_items(&app.items, blocked, callables, tasks, out),
            Item::Task(task) => {
                *tasks += 1;
                if let Some(item) = check_task(task, blocked, callables) {
                    out.push(item);
                }
            }
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn check_task(task: &Task, blocked: bool, callables: &CallableAnalysis) -> Option<ResourceItem> {
    let does = task.section("does")?;
    let body = core_body::analyze_does_section(does);
    let declarations = collect_resource_declarations(&task.sections);
    let checks = task_resource_checks(
        task,
        &declarations,
        &body.statements,
        blocked,
        callables.task_participates(task),
        callables.bridge_status(),
        callables.is_nonretained_closed_empty_task_definition(task),
    );
    let status = item_status(&checks, blocked);
    Some(ResourceItem {
        id: prefixed_id(
            "hum_resource_item",
            &format!("{}_{}", task.name, task.span.line),
        ),
        name: task.name.clone(),
        graph_node_id: node_id::span("item", &task.span, &format!("task {}", task.name)),
        span: portable_span(&task.span),
        status,
        declarations,
        checks,
    })
}

fn task_resource_checks(
    task: &Task,
    declarations: &ResourceDeclarations,
    statements: &[core_body::BodyStatement],
    blocked: bool,
    callable_slice_participant: bool,
    callable_bridge_status: &'static str,
    nonretained_closed_empty_task_definition: bool,
) -> Vec<ResourceCheck> {
    if blocked {
        return vec![resource_check(
            task,
            "resource_gate_prior_blocker",
            None,
            "not_checked_blocked_by_prior_errors_v0",
            Some("source_or_ownership_check_errors"),
        )];
    }

    if declarations.allocations.is_empty() {
        if callable_slice_participant {
            return vec![resource_check(
                task,
                "callable_resource_relationship",
                None,
                callable_bridge_status,
                Some("nonretained_definition_handle_has_no_callable_environment_v0"),
            )];
        }
        if nonretained_closed_empty_task_definition
            && declarations.constant_space.is_some()
            && !statements.iter().any(has_visible_allocation_risk)
            && !statements
                .iter()
                .any(|statement| statement.expression_kind == Some("call_like"))
        {
            return vec![resource_check(
                task,
                "callable_definition_constant_space",
                declarations
                    .constant_space
                    .as_ref()
                    .map(|declaration| declaration.normalized.clone()),
                "accepted_nonretained_callable_definition_constant_space_v0",
                Some("explicit_constant_space_and_no_visible_allocation_or_call_v0"),
            )];
        }
        return vec![resource_check(
            task,
            "allocation_intent_declared",
            None,
            "rejected_missing_allocation_declaration_v0",
            Some("task_body_requires_explicit_allocates_intent_v0"),
        )];
    }

    if declarations.has_allocation_free_claim() {
        if statements.iter().any(has_visible_allocation_risk) {
            return vec![resource_check(
                task,
                "allocation_free_visibility",
                declarations.first_allocation_text(),
                "rejected_allocation_free_claim_has_visible_allocation_risk_v0",
                Some("allocation_free_claim_contradicts_visible_construction_v0"),
            )];
        }
        if statements
            .iter()
            .any(|statement| statement.expression_kind == Some("call_like"))
        {
            return vec![resource_check(
                task,
                "allocation_free_visibility",
                declarations.first_allocation_text(),
                "unchecked_call_allocation_effect_v0",
                Some("callee_allocation_effect_not_checked_v0"),
            )];
        }
        return vec![resource_check(
            task,
            "allocation_free_visibility",
            declarations.first_allocation_text(),
            "accepted_conservative_allocation_free_claim_v0",
            Some("declared_not_proven"),
        )];
    }

    vec![resource_check(
        task,
        "allocation_intent_declared",
        declarations.first_allocation_text(),
        "accepted_allocation_behavior_declared_v0",
        Some("declared_not_proven"),
    )]
}

fn has_visible_allocation_risk(statement: &core_body::BodyStatement) -> bool {
    statement.expression_kind == Some("record_literal_start")
        || statement.kind == "record_field_initializer"
        || statement
            .primary_expression()
            .is_some_and(canonical_has_visible_allocation)
}

fn canonical_has_visible_allocation(expression: &crate::ast::CanonicalExpression) -> bool {
    use crate::ast::CanonicalExpressionKind;

    match &expression.kind {
        CanonicalExpressionKind::ListLiteral(_) | CanonicalExpressionKind::RecordLiteral { .. } => {
            true
        }
        CanonicalExpressionKind::Field { base, .. }
        | CanonicalExpressionKind::Element { base, .. }
        | CanonicalExpressionKind::Group(base)
        | CanonicalExpressionKind::Permission { value: base, .. } => {
            canonical_has_visible_allocation(base)
        }
        CanonicalExpressionKind::Try { call, .. } => canonical_has_visible_allocation(call),
        CanonicalExpressionKind::Call { callee, arguments } => {
            canonical_has_visible_allocation(callee)
                || arguments.iter().any(canonical_has_visible_allocation)
        }
        CanonicalExpressionKind::Binary { left, right, .. } => {
            canonical_has_visible_allocation(left) || canonical_has_visible_allocation(right)
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

fn collect_resource_declarations(sections: &[Section]) -> ResourceDeclarations {
    let mut declarations = ResourceDeclarations::default();
    for section in sections {
        for line in meaningful_lines(section) {
            if section.name == "allocates" {
                declarations.allocations.push(declared_resource(
                    "allocates",
                    &line.text,
                    &line.span,
                ));
            } else if section.name == "cost" && normalized_starts_with(&line.text, "allocates:") {
                declarations.allocations.push(declared_resource(
                    "cost",
                    line.text
                        .split_once(':')
                        .map(|(_key, value)| value.trim())
                        .unwrap_or(&line.text),
                    &line.span,
                ));
            } else if section.name == "cost" && normalize_resource_text(&line.text) == "space: o(1)"
            {
                declarations.constant_space = Some(declared_resource(
                    "cost",
                    line.text
                        .split_once(':')
                        .map(|(_key, value)| value.trim())
                        .unwrap_or(&line.text),
                    &line.span,
                ));
            }
        }
    }
    declarations
}

fn declared_resource(section: &'static str, text: &str, span: &Span) -> DeclaredResource {
    DeclaredResource {
        section,
        text: text.trim().to_string(),
        normalized: normalize_resource_text(text),
        span: portable_span(span),
    }
}

fn meaningful_lines(section: &Section) -> impl Iterator<Item = &crate::ast::SectionLine> {
    section
        .lines
        .iter()
        .filter(|line| is_meaningful_line_text(&line.text))
}

fn normalized_starts_with(text: &str, prefix: &str) -> bool {
    text.trim()
        .to_ascii_lowercase()
        .starts_with(&prefix.to_ascii_lowercase())
}

fn normalize_resource_text(text: &str) -> String {
    text.trim()
        .trim_end_matches('.')
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_allocation_free_text(text: &str) -> bool {
    matches!(
        normalize_resource_text(text).as_str(),
        "nothing" | "none" | "no heap allocation" | "no allocation" | "zero allocations"
    )
}

fn resource_check(
    task: &Task,
    check: &'static str,
    declaration: Option<String>,
    status: &'static str,
    reason: Option<&'static str>,
) -> ResourceCheck {
    ResourceCheck {
        id: prefixed_id(
            "hum_resource_check",
            &format!("{}_{}_{}", task.name, check, task.span.line),
        ),
        span: portable_span(&task.span),
        check,
        declaration,
        status,
        reason,
    }
}

fn item_status(checks: &[ResourceCheck], blocked: bool) -> &'static str {
    if blocked {
        "blocked_by_prior_errors"
    } else if checks
        .iter()
        .any(|check| check.status.starts_with("rejected_"))
    {
        "resource_errors_v0"
    } else if checks
        .iter()
        .any(|check| check.status.starts_with("unchecked_"))
    {
        "blocked_by_unchecked_resource_facts_v0"
    } else {
        "recognized_core_resource_facts_checked_v0"
    }
}

impl ResourceDeclarations {
    fn has_allocation_free_claim(&self) -> bool {
        self.allocations
            .iter()
            .any(|resource| is_allocation_free_text(&resource.text))
    }

    fn first_allocation_text(&self) -> Option<String> {
        self.allocations
            .first()
            .map(|resource| resource.normalized.clone())
    }
}

impl ResourceCheckReport {
    fn status(&self) -> &'static str {
        if self.source_errors > 0 {
            "blocked_by_source_errors"
        } else if self.ownership_errors() > 0 {
            "blocked_by_ownership_check_errors"
        } else if self.rejected_checks() > 0 {
            "resource_errors_v0"
        } else if self.unchecked_checks() > 0 {
            "blocked_by_unchecked_resource_facts_v0"
        } else {
            "recognized_core_resources_checked_v0"
        }
    }

    fn ownership_errors(&self) -> usize {
        self.ownership_check_summary.blocking_issues
    }

    fn checks(&self) -> usize {
        self.items.iter().map(|item| item.checks.len()).sum()
    }

    fn accepted_checks(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| &item.checks)
            .filter(|check| check.status.starts_with("accepted_"))
            .count()
    }

    fn rejected_checks(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| &item.checks)
            .filter(|check| check.status.starts_with("rejected_"))
            .count()
    }

    fn unchecked_checks(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| &item.checks)
            .filter(|check| check.status.starts_with("unchecked_"))
            .count()
    }

    fn blocking_issues(&self) -> usize {
        self.source_errors
            + self.ownership_errors()
            + self.rejected_checks()
            + self.unchecked_checks()
    }
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

fn push_dependencies(out: &mut String, report: &ResourceCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "dependencies");
    out.push_str(": {\n");
    push_indent(out, indent + 2);
    push_json_string(out, "ownership_check");
    out.push_str(": {\n");
    push_string_field(
        out,
        indent + 4,
        "schema",
        report.ownership_check_summary.schema,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "status",
        report.ownership_check_summary.status,
        true,
    );
    push_usize_field(
        out,
        indent + 4,
        "blocking_issues",
        report.ownership_check_summary.blocking_issues,
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("},\n");
    push_indent(out, indent + 2);
    push_json_string(out, "resource_report");
    out.push_str(": {\n");
    push_string_field(
        out,
        indent + 4,
        "schema",
        report.resource_report_summary.schema,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "status",
        report.resource_report_summary.status,
        true,
    );
    push_usize_field(
        out,
        indent + 4,
        "resource_claims",
        report.resource_report_summary.resource_claims,
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("}\n");
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_summary(out: &mut String, report: &ResourceCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "summary");
    out.push_str(": {\n");
    push_usize_field(out, indent + 2, "files", report.files, true);
    push_usize_field(out, indent + 2, "tasks", report.tasks, true);
    push_usize_field(out, indent + 2, "resource_items", report.items.len(), true);
    push_usize_field(
        out,
        indent + 2,
        "resource_claims",
        report.resource_report_summary.resource_claims,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "allocation_claims",
        report.resource_report_summary.allocation_claims,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "allocation_free_claims",
        report.resource_report_summary.allocation_free_claims,
        true,
    );
    push_usize_field(out, indent + 2, "checks", report.checks(), true);
    push_usize_field(
        out,
        indent + 2,
        "accepted_checks",
        report.accepted_checks(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "rejected_checks",
        report.rejected_checks(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unchecked_checks",
        report.unchecked_checks(),
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
        "ownership_errors",
        report.ownership_errors(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resource_report_errors",
        report.resource_report_summary.errors,
        true,
    );
    push_usize_field(out, indent + 2, "proof_ready", 0, true);
    push_usize_field(out, indent + 2, "execution_ready", 0, true);
    push_usize_field(out, indent + 2, "ir_ready", 0, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_items(out: &mut String, items: &[ResourceItem], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "resource_items");
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

fn push_item(out: &mut String, item: &ResourceItem, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &item.id, true);
    push_string_field(out, indent + 2, "kind", "task", true);
    push_string_field(out, indent + 2, "name", &item.name, true);
    push_string_field(out, indent + 2, "graph_node_id", &item.graph_node_id, true);
    push_span_field(out, indent + 2, "source_span", &item.span, true);
    push_string_field(out, indent + 2, "status", item.status, true);
    push_declarations(out, &item.declarations, indent + 2, true);
    push_checks(out, &item.checks, indent + 2, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_declarations(
    out: &mut String,
    declarations: &ResourceDeclarations,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "declarations");
    out.push_str(": {\n");
    push_indent(out, indent + 2);
    push_json_string(out, "allocations");
    out.push_str(": [");
    if !declarations.allocations.is_empty() {
        out.push('\n');
        for (index, allocation) in declarations.allocations.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_declared_resource(out, allocation, indent + 4);
        }
        out.push('\n');
        push_indent(out, indent + 2);
    }
    out.push_str("]\n");
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_declared_resource(out: &mut String, resource: &DeclaredResource, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "section", resource.section, true);
    push_string_field(out, indent + 2, "text", &resource.text, true);
    push_string_field(out, indent + 2, "normalized", &resource.normalized, true);
    push_span_field(out, indent + 2, "source_span", &resource.span, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_checks(out: &mut String, checks: &[ResourceCheck], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "checks");
    out.push_str(": [");
    if !checks.is_empty() {
        out.push('\n');
        for (index, check) in checks.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_check(out, check, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_check(out: &mut String, check: &ResourceCheck, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &check.id, true);
    push_span_field(out, indent + 2, "source_span", &check.span, true);
    push_string_field(out, indent + 2, "check", check.check, true);
    push_optional_string_field(
        out,
        indent + 2,
        "declaration",
        check.declaration.as_deref(),
        true,
    );
    push_string_field(out, indent + 2, "status", check.status, true);
    push_optional_string_field(out, indent + 2, "reason", check.reason, false);
    push_indent(out, indent);
    out.push('}');
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
        resource_check_has_errors, resource_check_json, resource_check_summary, resource_check_text,
    };

    #[test]
    fn json_accepts_declared_allocation_free_local_body_without_proof_claims() {
        let program = resource_demo_program();
        let json = resource_check_json(&program, &[]);

        assert!(!resource_check_has_errors(&program, &[]));
        assert!(json.contains("\"schema\": \"hum.resource_check.v0\""));
        assert!(json.contains("\"status\": \"recognized_core_resources_checked_v0\""));
        assert!(json.contains("\"accepted_conservative_allocation_free_claim_v0\""));
        assert!(json.contains("\"proof_ready\": 0"));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"ir_ready\": 0"));
        assert!(json.contains("\"no allocation-freedom proof\""));
    }

    #[test]
    fn json_rejects_missing_allocation_intent_for_body() {
        let program = missing_allocation_program();
        let json = resource_check_json(&program, &[]);

        assert!(resource_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"resource_errors_v0\""));
        assert!(json.contains("\"rejected_missing_allocation_declaration_v0\""));
    }

    #[test]
    fn summary_blocks_on_prior_ownership_errors() {
        let program = ownership_blocked_program();
        let summary = resource_check_summary(&program, &[]);

        assert_eq!(summary.status, "blocked_by_ownership_check_errors");
        assert!(summary.ownership_errors > 0);
        assert!(summary.blocking_issues >= summary.ownership_errors);
    }

    #[test]
    fn text_reports_resource_gate_without_safety_claims() {
        let program = resource_demo_program();
        let text = resource_check_text(&program, &[]);

        assert!(text.contains("Hum resource check (hum.resource_check.v0)"));
        assert!(text.contains("status: recognized_core_resources_checked_v0"));
        assert!(text.contains("no memory-safety proof"));
        assert!(text.contains("no complete resource analysis"));
    }

    fn resource_demo_program() -> Program {
        parse_program(
            "resource_demo.hum",
            r#"type WorkError {
  code: Text
}

task retry(flag: Bool) -> Result UInt, WorkError {
  why:
    keep the resource gate small

  needs:
    flag is provided

  ensures:
    attempts is returned

  fails when:
    flag is false

  cost:
    time: O(1)
    space: O(1)
    check: compile

  allocates:
    nothing

  does:
    change attempts: UInt = 0
    if flag == false {
      fail WorkError.no_flag
    }

    set attempts = attempts + 1
    return attempts
}
"#,
        )
    }

    fn missing_allocation_program() -> Program {
        parse_program(
            "missing_allocation.hum",
            r#"task add(count: UInt) -> UInt {
  does:
    let next: UInt = count + 1
    return next
}
"#,
        )
    }

    fn ownership_blocked_program() -> Program {
        parse_program(
            "ownership_blocked.hum",
            r#"task duplicate() -> UInt {
  allocates:
    nothing

  does:
    let count: UInt = 0
    let count: UInt = 1
    return count
}
"#,
        )
    }

    fn parse_program(path: &str, source: &str) -> Program {
        Program {
            files: vec![parse_source(path, source).file],
        }
    }
}
