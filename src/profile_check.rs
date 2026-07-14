use crate::ast::{Item, Program, Section};
use crate::diagnostic::{
    Diagnostic, DiagnosticInvariantError, DiagnosticOccurrenceSet, DiagnosticProjection,
    PriorBlockerRef, Severity, Span,
};
use crate::graph::is_meaningful_line_text;
use crate::node_id;
use crate::resource_check;
use crate::runtime_profiles;
use crate::version;

pub const PROFILE_CHECK_SCHEMA: &str = "hum.profile_check.v0";
pub const PROFILE_CHECK_MODE: &str = "recognized_profile_policy_gate_v0";
pub const PROFILE_CHECK_STATUS: &str = "recognized_core_profile_gate_available_v0";

const NON_CLAIMS: &[&str] = &[
    "no profile enforcement",
    "no stdlib narrowing",
    "no executable runtime behavior",
    "no certification claim",
    "no target selection",
    "no host probing",
    "no performance or footprint measurement",
    "no concurrency-safety proof",
    "no memory-safety proof",
    "no Hum IR emission",
    "no backend lowering",
    "no proof artifact",
];

pub(crate) struct ProfileDiagnosticTransport {
    authoritative: DiagnosticOccurrenceSet,
    ir_projection: DiagnosticProjection,
    graph_projection: DiagnosticProjection,
}

impl ProfileDiagnosticTransport {
    pub(crate) fn authoritative(&self) -> &DiagnosticOccurrenceSet {
        &self.authoritative
    }

    pub(crate) fn ir_projection(&self) -> &DiagnosticProjection {
        &self.ir_projection
    }

    pub(crate) fn graph_projection(&self) -> &DiagnosticProjection {
        &self.graph_projection
    }

    #[cfg(test)]
    pub(crate) fn graph_projection_mut_for_test(&mut self) -> &mut DiagnosticProjection {
        &mut self.graph_projection
    }
}

fn outgoing_diagnostic_transport(
    authoritative: DiagnosticOccurrenceSet,
) -> Result<ProfileDiagnosticTransport, DiagnosticInvariantError> {
    let ir_projection = DiagnosticProjection::from_upstream("ir_readiness", &authoritative)?;
    let graph_projection = DiagnosticProjection::from_upstream("graph", &authoritative)?;
    Ok(ProfileDiagnosticTransport {
        authoritative,
        ir_projection,
        graph_projection,
    })
}

pub(crate) fn diagnostic_transport(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> Result<ProfileDiagnosticTransport, DiagnosticInvariantError> {
    outgoing_diagnostic_transport(diagnostic_occurrence_set(program, diagnostics))
}

pub(crate) fn diagnostic_transport_from_source(
    program: &Program,
    diagnostics: &[Diagnostic],
    source_occurrences: &DiagnosticOccurrenceSet,
) -> Result<ProfileDiagnosticTransport, DiagnosticInvariantError> {
    outgoing_diagnostic_transport(diagnostic_occurrence_set_from_source(
        program,
        diagnostics,
        source_occurrences,
    )?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileCheckSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub source_errors: usize,
    pub resource_check_errors: usize,
    pub tasks: usize,
    pub profile_items: usize,
    pub declared_profiles: usize,
    pub default_profiles: usize,
    pub known_profiles: usize,
    pub unknown_profiles: usize,
    pub strict_profiles: usize,
    pub checks: usize,
    pub accepted_checks: usize,
    pub rejected_checks: usize,
    pub unchecked_checks: usize,
    pub blocking_issues: usize,
    pub proof_ready: usize,
    pub execution_ready: usize,
    pub ir_ready: usize,
}

struct ProfileCheckReport {
    resource_check_summary: resource_check::ResourceCheckSummary,
    files: usize,
    tasks: usize,
    source_errors: usize,
    items: Vec<ProfileItem>,
    prior_blockers: Vec<PriorBlockerRef>,
    diagnostic_occurrences: DiagnosticOccurrenceSet,
}

struct ProfileItem {
    id: String,
    kind: &'static str,
    name: String,
    graph_node_id: String,
    span: Span,
    status: &'static str,
    declarations: Vec<ProfileDeclaration>,
    checks: Vec<ProfileCheck>,
}

struct ProfileDeclaration {
    source_section: String,
    text: String,
    normalized: String,
    span: Span,
    known_profile: Option<&'static runtime_profiles::RuntimeProfile>,
}

struct ProfileCheck {
    id: String,
    span: Span,
    check: &'static str,
    declaration: Option<String>,
    profile_id: Option<String>,
    status: &'static str,
    reason: Option<&'static str>,
}

pub fn profile_check_has_errors(program: &Program, diagnostics: &[Diagnostic]) -> bool {
    profile_check_summary(program, diagnostics).blocking_issues > 0
}

pub fn profile_check_summary(program: &Program, diagnostics: &[Diagnostic]) -> ProfileCheckSummary {
    let report = build_report(program, diagnostics);
    ProfileCheckSummary {
        schema: PROFILE_CHECK_SCHEMA,
        status: report.status(),
        mode: PROFILE_CHECK_MODE,
        source_errors: report.source_errors,
        resource_check_errors: report.resource_check_errors(),
        tasks: report.tasks,
        profile_items: report.items.len(),
        declared_profiles: report.declared_profiles(),
        default_profiles: report.default_profiles(),
        known_profiles: report.known_profiles(),
        unknown_profiles: report.unknown_profiles(),
        strict_profiles: report.strict_profiles(),
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

pub fn profile_check_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum profile check ({PROFILE_CHECK_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {PROFILE_CHECK_MODE}\n"));
    out.push_str(&format!("status: {}\n", report.status()));
    out.push_str(&format!(
        "dependencies: resource_check={} runtime_profiles={} runtime_profile={}\n",
        resource_check::RESOURCE_CHECK_SCHEMA,
        runtime_profiles::RUNTIME_PROFILES_SCHEMA,
        runtime_profiles::RUNTIME_PROFILE_SCHEMA
    ));
    out.push_str(&format!(
        "summary: files={} tasks={} profile_items={} declared_profiles={} default_profiles={} known_profiles={} unknown_profiles={} strict_profiles={} checks={} accepted_checks={} rejected_checks={} unchecked_checks={} blocking_issues={} source_errors={} resource_check_errors={} proof_ready=0 execution_ready=0 ir_ready=0\n",
        report.files,
        report.tasks,
        report.items.len(),
        report.declared_profiles(),
        report.default_profiles(),
        report.known_profiles(),
        report.unknown_profiles(),
        report.strict_profiles(),
        report.checks(),
        report.accepted_checks(),
        report.rejected_checks(),
        report.unchecked_checks(),
        report.blocking_issues(),
        report.source_errors,
        report.resource_check_errors(),
    ));

    if report.items.is_empty() {
        out.push_str("profile_items: none\n");
    } else {
        out.push_str("profile_items:\n");
        for item in &report.items {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} `{}` checks={}\n",
                item.span.file,
                item.span.line,
                item.span.column,
                item.status,
                item.kind,
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
                if let Some(profile_id) = &check.profile_id {
                    out.push_str(&format!(" profile={profile_id}"));
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

pub fn profile_check_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", PROFILE_CHECK_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", PROFILE_CHECK_MODE, true);
    push_string_field(
        &mut out,
        2,
        "resource_check_schema",
        resource_check::RESOURCE_CHECK_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "runtime_profiles_schema",
        runtime_profiles::RUNTIME_PROFILES_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "runtime_profile_schema",
        runtime_profiles::RUNTIME_PROFILE_SCHEMA,
        true,
    );
    push_string_field(
        &mut out,
        2,
        "runtime_profile_mode",
        runtime_profiles::RUNTIME_PROFILE_MODE,
        true,
    );
    push_dependencies(&mut out, &report, 2, true);
    push_summary(&mut out, &report, 2, true);
    push_items(&mut out, &report.items, 2, true);
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> ProfileCheckReport {
    let resource_check_summary = resource_check::resource_check_summary(program, diagnostics);
    let source_errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let blocked = source_errors > 0 || resource_check_summary.blocking_issues > 0;
    let mut items = Vec::new();
    let mut tasks = 0;
    for file in &program.files {
        collect_items(&file.items, blocked, &mut tasks, &mut items);
    }
    let diagnostic_occurrences = resource_check::diagnostic_occurrence_set(program, diagnostics);
    let projection = diagnostic_projection_from_resource(&diagnostic_occurrences)
        .expect("profile check must carry one sealed resource projection");
    projection
        .validate_against("profile_check", &diagnostic_occurrences)
        .expect("profile check must validate its resource authority");
    let prior_blockers = diagnostic_occurrences.prior_blockers();
    ProfileCheckReport {
        resource_check_summary,
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
    let occurrences = resource_check::diagnostic_occurrence_set_from_source(
        program,
        diagnostics,
        source_occurrences,
    )?;
    let projection = diagnostic_projection_from_resource(&occurrences)?;
    projection.validate_against("profile_check", &occurrences)?;
    Ok(occurrences)
}

pub(crate) fn diagnostic_projection_from_resource(
    occurrences: &DiagnosticOccurrenceSet,
) -> Result<crate::diagnostic::DiagnosticProjection, crate::diagnostic::DiagnosticInvariantError> {
    crate::diagnostic::DiagnosticProjection::from_upstream("profile_check", occurrences)
}

pub(crate) fn validate_prior_blocker_projection(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> Result<(), crate::diagnostic::DiagnosticInvariantError> {
    let report = build_report(program, diagnostics);
    resource_check::validate_prior_blocker_projection(program, diagnostics)?;
    report
        .diagnostic_occurrences
        .validate_prior_blockers(&report.prior_blockers)
}

fn collect_items(items: &[Item], blocked: bool, tasks: &mut usize, out: &mut Vec<ProfileItem>) {
    for item in items {
        match item {
            Item::App(app) => {
                if let Some(item) =
                    profile_item("app", &app.name, &app.sections, &app.span, blocked, false)
                {
                    out.push(item);
                }
                collect_items(&app.items, blocked, tasks, out);
            }
            Item::Task(task) => {
                *tasks += 1;
                if let Some(item) = profile_item(
                    "task",
                    &task.name,
                    &task.sections,
                    &task.span,
                    blocked,
                    task.section("does").is_some(),
                ) {
                    out.push(item);
                }
            }
            Item::Type(type_def) => {
                if let Some(item) = profile_item(
                    "type",
                    &type_def.name,
                    &type_def.sections,
                    &type_def.span,
                    blocked,
                    false,
                ) {
                    out.push(item);
                }
            }
            Item::Store(store) => {
                if let Some(item) = profile_item(
                    "store",
                    &store.name,
                    &store.sections,
                    &store.span,
                    blocked,
                    false,
                ) {
                    out.push(item);
                }
            }
            Item::Test(test) => {
                if let Some(item) = profile_item(
                    "test",
                    &test.name,
                    &test.sections,
                    &test.span,
                    blocked,
                    false,
                ) {
                    out.push(item);
                }
            }
        }
    }
}

fn profile_item(
    kind: &'static str,
    name: &str,
    sections: &[Section],
    span: &Span,
    blocked: bool,
    default_normal: bool,
) -> Option<ProfileItem> {
    let mut declarations = collect_profile_declarations(sections);
    if declarations.is_empty() && default_normal {
        declarations.push(default_profile_declaration(span));
    }
    if declarations.is_empty() {
        return None;
    }

    let checks = item_profile_checks(kind, name, span, &declarations, blocked);
    let status = item_status(&checks, blocked);
    Some(ProfileItem {
        id: prefixed_id("hum_profile_item", &format!("{kind}_{name}_{}", span.line)),
        kind,
        name: name.to_string(),
        graph_node_id: node_id::span("item", span, &format!("{kind} {name}")),
        span: portable_span(span),
        status,
        declarations,
        checks,
    })
}

fn item_profile_checks(
    kind: &'static str,
    name: &str,
    span: &Span,
    declarations: &[ProfileDeclaration],
    blocked: bool,
) -> Vec<ProfileCheck> {
    let owner_key = format!("{kind}_{name}");
    if blocked {
        return vec![profile_check(
            &owner_key,
            span,
            "profile_gate_prior_blocker",
            None,
            None,
            "not_checked_blocked_by_prior_errors_v0",
            Some("source_or_resource_check_errors"),
        )];
    }

    declarations
        .iter()
        .map(|declaration| match declaration.known_profile {
            None => profile_check(
                &owner_key,
                &declaration.span,
                "profile_known_to_catalog",
                Some(declaration.normalized.clone()),
                None,
                "rejected_unknown_profile_v0",
                Some("unknown_profile_fails_closed_v0"),
            ),
            Some(profile) if profile.id == "normal" => profile_check(
                &owner_key,
                &declaration.span,
                "normal_profile_policy",
                Some(declaration.normalized.clone()),
                Some(profile.id.to_string()),
                "accepted_normal_profile_policy_v0",
                Some("recognized_not_enforced"),
            ),
            Some(profile) => profile_check(
                &owner_key,
                &declaration.span,
                "strict_profile_policy",
                Some(declaration.normalized.clone()),
                Some(profile.id.to_string()),
                "unchecked_strict_profile_enforcement_v0",
                Some("strict_profile_requires_future_enforcement_and_evidence_v0"),
            ),
        })
        .collect()
}

fn collect_profile_declarations(sections: &[Section]) -> Vec<ProfileDeclaration> {
    let mut declarations = Vec::new();
    for section in sections {
        if !is_profile_section(&section.name) {
            continue;
        }
        for line in section
            .lines
            .iter()
            .filter(|line| is_meaningful_line_text(&line.text))
        {
            declarations.push(profile_declaration(&section.name, &line.text, &line.span));
        }
    }
    declarations
}

fn is_profile_section(name: &str) -> bool {
    matches!(
        normalize_section_name(name).as_str(),
        "profile" | "profiles" | "runtime_profile" | "runtime_profiles"
    )
}

fn normalize_section_name(name: &str) -> String {
    normalize_identifier(name)
}

fn profile_declaration(section: &str, text: &str, span: &Span) -> ProfileDeclaration {
    let normalized = normalize_profile_text(text);
    ProfileDeclaration {
        source_section: section.to_string(),
        text: text.trim().to_string(),
        known_profile: runtime_profiles::runtime_profile_by_id(&normalized),
        normalized,
        span: portable_span(span),
    }
}

fn default_profile_declaration(span: &Span) -> ProfileDeclaration {
    let normalized = "normal".to_string();
    ProfileDeclaration {
        source_section: "default".to_string(),
        text: "normal".to_string(),
        known_profile: runtime_profiles::runtime_profile_by_id(&normalized),
        normalized,
        span: portable_span(span),
    }
}

fn normalize_profile_text(text: &str) -> String {
    let mut value = text.trim().trim_end_matches('.').to_ascii_lowercase();
    for prefix in ["profile:", "profiles:", "profile ", "profiles "] {
        if value.starts_with(prefix) {
            value = value[prefix.len()..].trim().to_string();
        }
    }
    normalize_identifier(&value)
}

fn normalize_identifier(text: &str) -> String {
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

fn profile_check(
    owner_key: &str,
    span: &Span,
    check: &'static str,
    declaration: Option<String>,
    profile_id: Option<String>,
    status: &'static str,
    reason: Option<&'static str>,
) -> ProfileCheck {
    ProfileCheck {
        id: prefixed_id(
            "hum_profile_check",
            &format!("{owner_key}_{check}_{}", span.line),
        ),
        span: portable_span(span),
        check,
        declaration,
        profile_id,
        status,
        reason,
    }
}

fn item_status(checks: &[ProfileCheck], blocked: bool) -> &'static str {
    if blocked {
        "blocked_by_prior_errors"
    } else if checks
        .iter()
        .any(|check| check.status.starts_with("rejected_"))
    {
        "profile_errors_v0"
    } else if checks
        .iter()
        .any(|check| check.status.starts_with("unchecked_"))
    {
        "blocked_by_unchecked_profile_policy_v0"
    } else {
        "recognized_profile_policy_checked_v0"
    }
}

impl ProfileCheckReport {
    fn status(&self) -> &'static str {
        if self.source_errors > 0 {
            "blocked_by_source_errors"
        } else if self.resource_check_errors() > 0 {
            "blocked_by_resource_check_errors"
        } else if self.rejected_checks() > 0 {
            "profile_errors_v0"
        } else if self.unchecked_checks() > 0 {
            "blocked_by_unchecked_profile_policy_v0"
        } else {
            "recognized_profile_policy_checked_v0"
        }
    }

    fn resource_check_errors(&self) -> usize {
        self.resource_check_summary.blocking_issues
    }

    fn declared_profiles(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| &item.declarations)
            .filter(|declaration| declaration.source_section != "default")
            .count()
    }

    fn default_profiles(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| &item.declarations)
            .filter(|declaration| declaration.source_section == "default")
            .count()
    }

    fn known_profiles(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| &item.declarations)
            .filter(|declaration| declaration.known_profile.is_some())
            .count()
    }

    fn unknown_profiles(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| &item.declarations)
            .filter(|declaration| declaration.known_profile.is_none())
            .count()
    }

    fn strict_profiles(&self) -> usize {
        self.items
            .iter()
            .flat_map(|item| &item.declarations)
            .filter(|declaration| {
                matches!(declaration.known_profile, Some(profile) if profile.id != "normal")
            })
            .count()
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
            + self.resource_check_errors()
            + self.rejected_checks()
            + self.unchecked_checks()
    }
}

fn push_dependencies(out: &mut String, report: &ProfileCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "dependencies");
    out.push_str(": {\n");
    push_indent(out, indent + 2);
    push_json_string(out, "resource_check");
    out.push_str(": {\n");
    push_string_field(
        out,
        indent + 4,
        "schema",
        report.resource_check_summary.schema,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "status",
        report.resource_check_summary.status,
        true,
    );
    push_usize_field(
        out,
        indent + 4,
        "blocking_issues",
        report.resource_check_summary.blocking_issues,
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("},\n");
    push_indent(out, indent + 2);
    push_json_string(out, "runtime_profiles");
    out.push_str(": {\n");
    push_string_field(
        out,
        indent + 4,
        "schema",
        runtime_profiles::RUNTIME_PROFILES_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "profile_schema",
        runtime_profiles::RUNTIME_PROFILE_SCHEMA,
        true,
    );
    push_string_field(
        out,
        indent + 4,
        "mode",
        runtime_profiles::RUNTIME_PROFILE_MODE,
        true,
    );
    push_usize_field(
        out,
        indent + 4,
        "profiles",
        runtime_profiles::runtime_profiles().len(),
        false,
    );
    push_indent(out, indent + 2);
    out.push_str("}\n");
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_summary(out: &mut String, report: &ProfileCheckReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "summary");
    out.push_str(": {\n");
    push_usize_field(out, indent + 2, "files", report.files, true);
    push_usize_field(out, indent + 2, "tasks", report.tasks, true);
    push_usize_field(out, indent + 2, "profile_items", report.items.len(), true);
    push_usize_field(
        out,
        indent + 2,
        "declared_profiles",
        report.declared_profiles(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "default_profiles",
        report.default_profiles(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "known_profiles",
        report.known_profiles(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unknown_profiles",
        report.unknown_profiles(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "strict_profiles",
        report.strict_profiles(),
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
        "resource_check_errors",
        report.resource_check_errors(),
        true,
    );
    push_usize_field(out, indent + 2, "proof_ready", 0, true);
    push_usize_field(out, indent + 2, "execution_ready", 0, true);
    push_usize_field(out, indent + 2, "ir_ready", 0, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_items(out: &mut String, items: &[ProfileItem], indent: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "profile_items");
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

fn push_item(out: &mut String, item: &ProfileItem, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &item.id, true);
    push_string_field(out, indent + 2, "kind", item.kind, true);
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
    declarations: &[ProfileDeclaration],
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "declarations");
    out.push_str(": [");
    if !declarations.is_empty() {
        out.push('\n');
        for (index, declaration) in declarations.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_declaration(out, declaration, indent + 2);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_declaration(out: &mut String, declaration: &ProfileDeclaration, indent: usize) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(
        out,
        indent + 2,
        "source_section",
        &declaration.source_section,
        true,
    );
    push_string_field(out, indent + 2, "text", &declaration.text, true);
    push_string_field(out, indent + 2, "normalized", &declaration.normalized, true);
    push_span_field(out, indent + 2, "source_span", &declaration.span, true);
    push_profile_policy(out, declaration.known_profile, indent + 2, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_profile_policy(
    out: &mut String,
    profile: Option<&runtime_profiles::RuntimeProfile>,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "runtime_profile");
    out.push_str(": ");
    match profile {
        Some(profile) => {
            out.push_str("{\n");
            push_string_field(
                out,
                indent + 2,
                "schema",
                runtime_profiles::RUNTIME_PROFILE_SCHEMA,
                true,
            );
            push_string_field(out, indent + 2, "id", profile.id, true);
            push_string_field(
                out,
                indent + 2,
                "source_spelling",
                profile.source_spelling,
                true,
            );
            push_string_field(out, indent + 2, "status", profile.status, true);
            push_string_field(out, indent + 2, "purpose", profile.purpose, true);
            push_string_array(
                out,
                indent + 2,
                "forbids_by_default",
                profile.forbids_by_default,
                true,
            );
            push_string_array(
                out,
                indent + 2,
                "requires_evidence",
                profile.requires_evidence,
                true,
            );
            push_string_array(
                out,
                indent + 2,
                "allowed_capability_families",
                profile.allowed_capability_families,
                true,
            );
            push_string_array(
                out,
                indent + 2,
                "denied_capability_families",
                profile.denied_capability_families,
                false,
            );
            push_indent(out, indent);
            out.push('}');
        }
        None => out.push_str("null"),
    }
    push_comma_newline(out, comma);
}

fn push_checks(out: &mut String, checks: &[ProfileCheck], indent: usize, comma: bool) {
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

fn push_check(out: &mut String, check: &ProfileCheck, indent: usize) {
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
    push_optional_string_field(
        out,
        indent + 2,
        "profile_id",
        check.profile_id.as_deref(),
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

fn portable_span(span: &Span) -> Span {
    Span {
        file: span.file.replace('\\', "/"),
        line: span.line,
        column: span.column,
    }
}

fn prefixed_id(prefix: &str, text: &str) -> String {
    let mut body = normalize_identifier(text);
    if body.len() < 4 {
        body.push_str("_item");
    }
    if body.len() > 96 {
        body.truncate(96);
        body = body.trim_matches('_').to_string();
    }
    format!("{prefix}_{body}")
}

#[cfg(test)]
mod tests {
    use crate::ast::Program;
    use crate::parser::parse_source;

    use super::{
        profile_check_has_errors, profile_check_json, profile_check_summary, profile_check_text,
    };

    #[test]
    fn json_accepts_explicit_normal_profile_without_enforcement_claims() {
        let program = profile_demo_program("normal");
        let json = profile_check_json(&program, &[]);

        assert!(!profile_check_has_errors(&program, &[]));
        assert!(json.contains("\"schema\": \"hum.profile_check.v0\""));
        assert!(json.contains("\"status\": \"recognized_profile_policy_checked_v0\""));
        assert!(json.contains("\"runtime_profiles_schema\": \"hum.runtime_profiles.v0\""));
        assert!(json.contains("\"resource_check_schema\": \"hum.resource_check.v0\""));
        assert!(json.contains("\"accepted_normal_profile_policy_v0\""));
        assert!(json.contains("\"profile_id\": \"normal\""));
        assert!(json.contains("\"proof_ready\": 0"));
        assert!(json.contains("\"execution_ready\": 0"));
        assert!(json.contains("\"ir_ready\": 0"));
        assert!(json.contains("\"no profile enforcement\""));
        assert!(json.contains("\"no certification claim\""));
    }

    #[test]
    fn summary_defaults_body_tasks_to_normal_profile() {
        let program = profile_demo_program("");
        let summary = profile_check_summary(&program, &[]);

        assert_eq!(summary.status, "recognized_profile_policy_checked_v0");
        assert_eq!(summary.default_profiles, 1);
        assert_eq!(summary.known_profiles, 1);
        assert_eq!(summary.blocking_issues, 0);
    }

    #[test]
    fn json_rejects_unknown_profile_names_fail_closed() {
        let program = profile_demo_program("moon base");
        let json = profile_check_json(&program, &[]);

        assert!(profile_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"profile_errors_v0\""));
        assert!(json.contains("\"rejected_unknown_profile_v0\""));
        assert!(json.contains("unknown_profile_fails_closed_v0"));
    }

    #[test]
    fn json_blocks_known_strict_profiles_until_enforcement_exists() {
        let program = profile_demo_program("hard realtime");
        let json = profile_check_json(&program, &[]);

        assert!(profile_check_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"blocked_by_unchecked_profile_policy_v0\""));
        assert!(json.contains("\"profile_id\": \"hard_realtime\""));
        assert!(json.contains("unchecked_strict_profile_enforcement_v0"));
        assert!(json.contains("strict_profile_requires_future_enforcement_and_evidence_v0"));
    }

    #[test]
    fn summary_blocks_on_prior_resource_errors() {
        let program = missing_allocation_program();
        let summary = profile_check_summary(&program, &[]);

        assert_eq!(summary.status, "blocked_by_resource_check_errors");
        assert!(summary.resource_check_errors > 0);
        assert!(summary.blocking_issues >= summary.resource_check_errors);
    }

    #[test]
    fn text_reports_profile_gate_without_safety_claims() {
        let program = profile_demo_program("normal");
        let text = profile_check_text(&program, &[]);

        assert!(text.contains("Hum profile check (hum.profile_check.v0)"));
        assert!(text.contains("status: recognized_profile_policy_checked_v0"));
        assert!(text.contains("no profile enforcement"));
        assert!(text.contains("no memory-safety proof"));
    }

    fn profile_demo_program(profile_line: &str) -> Program {
        let profiles = if profile_line.is_empty() {
            String::new()
        } else {
            format!("\n  profiles:\n    {profile_line}\n")
        };
        parse_program(
            "profile_demo.hum",
            &format!(
                r#"type WorkError {{
  code: Text
}}

task retry(flag: Bool) -> Result UInt, WorkError {{
  why:
    keep the profile gate small
{profiles}
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
    if flag == false {{
      fail WorkError.no_flag
    }}

    set attempts = attempts + 1
    return attempts
}}
"#
            ),
        )
    }

    fn missing_allocation_program() -> Program {
        parse_program(
            "missing_allocation.hum",
            r#"task add(count: UInt) -> UInt {
  profiles:
    normal

  does:
    let next: UInt = count + 1
    return next
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
