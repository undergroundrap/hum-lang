use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{Item, Param, ParamPermission, Program, Section};
use crate::core_body::{self, BodyStatement};
use crate::core_expr::{self, CoreExpressionPreview};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity, Span};
use crate::graph::is_meaningful_line_text;
use crate::predicate;
use crate::version;
use crate::writable_field_alias;

pub const RESOLVE_REPORT_SCHEMA: &str = "hum.resolve.v0";
pub const RESOLVE_MODE: &str = "source_analysis_only_no_type_or_borrow_check";

struct ResolveReport {
    files: usize,
    items: usize,
    source_errors: usize,
    source_warnings: usize,
    scopes: Vec<ResolveScope>,
    definitions: Vec<ResolveDefinition>,
    references: Vec<ResolveReference>,
    diagnostics: Vec<ResolverDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolveReadinessSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub mode: &'static str,
    pub files: usize,
    pub items: usize,
    pub source_errors: usize,
    pub source_warnings: usize,
    pub scopes: usize,
    pub definitions: usize,
    pub references: usize,
    pub resolved_references: usize,
    pub unresolved_references: usize,
    pub external_references: usize,
    pub duplicate_definitions: usize,
    pub mutable_place_errors: usize,
    pub resolver_errors: usize,
    pub resolver_warnings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveDefinitionSummary {
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub definition_kind: &'static str,
    pub scope_id: String,
    pub mutable: bool,
    pub state_kind: &'static str,
    pub source_span: Span,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveScopeSummary {
    pub id: String,
    pub parent_scope_id: Option<String>,
    pub scope_kind: &'static str,
    pub owner_kind: &'static str,
    pub owner_name: String,
    pub source_span: Option<Span>,
}

#[derive(Debug, Clone)]
struct ResolveScope {
    id: String,
    parent_scope_id: Option<String>,
    scope_kind: &'static str,
    owner_kind: &'static str,
    owner_name: String,
    source_span: Option<Span>,
}

#[derive(Debug, Clone)]
struct ResolveDefinition {
    id: String,
    name: String,
    normalized_name: String,
    definition_kind: &'static str,
    scope_id: String,
    mutable: bool,
    state_kind: &'static str,
    source_span: Span,
    status: &'static str,
    duplicate_of: Option<String>,
}

#[derive(Debug, Clone)]
struct ResolveReference {
    id: String,
    name: String,
    normalized_name: String,
    reference_kind: &'static str,
    scope_id: String,
    mutable_required: bool,
    source_span: Span,
    resolution_status: &'static str,
    resolved_definition_id: Option<String>,
    reason: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct ResolverDiagnostic {
    code: DiagnosticCode,
    severity: Severity,
    title: &'static str,
    message: String,
    source_span: Span,
    help: &'static str,
    reference_id: Option<String>,
    definition_id: Option<String>,
}

#[derive(Clone)]
struct DefinitionRef {
    index: usize,
}

struct PendingReferenceInput<'a> {
    name: &'a str,
    reference_kind: &'static str,
    mutable_required: bool,
    external_if_unresolved: bool,
    span: &'a Span,
}

struct DefinitionInput<'a> {
    name: &'a str,
    definition_kind: &'static str,
    mutable: bool,
    state_kind: &'static str,
    span: &'a Span,
    defer_duplicate_diagnostic: bool,
}

struct ResolverContext {
    scopes: Vec<ResolveScope>,
    definitions: Vec<ResolveDefinition>,
    references: Vec<ResolveReference>,
    diagnostics: Vec<ResolverDiagnostic>,
    scope_parents: BTreeMap<String, Option<String>>,
    definitions_by_scope_name: BTreeMap<(String, String), DefinitionRef>,
    scope_serial: usize,
    definition_serial: usize,
    reference_serial: usize,
}

pub fn resolve_has_errors(program: &Program, source_diagnostics: &[Diagnostic]) -> bool {
    let report = build_report(program, source_diagnostics);
    report.source_errors > 0 || report.error_count() > 0
}

pub fn resolve_readiness_summary(
    program: &Program,
    source_diagnostics: &[Diagnostic],
) -> ResolveReadinessSummary {
    let report = build_report(program, source_diagnostics);
    ResolveReadinessSummary {
        schema: RESOLVE_REPORT_SCHEMA,
        status: report.status(),
        mode: RESOLVE_MODE,
        files: report.files,
        items: report.items,
        source_errors: report.source_errors,
        source_warnings: report.source_warnings,
        scopes: report.scopes.len(),
        definitions: report.definitions.len(),
        references: report.references.len(),
        resolved_references: report.resolved_references(),
        unresolved_references: report.unresolved_references(),
        external_references: report.external_references(),
        duplicate_definitions: report.duplicate_definitions(),
        mutable_place_errors: report.mutable_place_errors(),
        resolver_errors: report.error_count(),
        resolver_warnings: report.warning_count(),
    }
}

pub fn resolve_definition_summaries(
    program: &Program,
    source_diagnostics: &[Diagnostic],
) -> Vec<ResolveDefinitionSummary> {
    let report = build_report(program, source_diagnostics);
    report
        .definitions
        .iter()
        .map(|definition| ResolveDefinitionSummary {
            id: definition.id.clone(),
            name: definition.name.clone(),
            normalized_name: definition.normalized_name.clone(),
            definition_kind: definition.definition_kind,
            scope_id: definition.scope_id.clone(),
            mutable: definition.mutable,
            state_kind: definition.state_kind,
            source_span: definition.source_span.clone(),
            status: definition.status,
        })
        .collect()
}

pub fn resolve_scope_summaries(
    program: &Program,
    source_diagnostics: &[Diagnostic],
) -> Vec<ResolveScopeSummary> {
    build_report(program, source_diagnostics)
        .scopes
        .iter()
        .map(|scope| ResolveScopeSummary {
            id: scope.id.clone(),
            parent_scope_id: scope.parent_scope_id.clone(),
            scope_kind: scope.scope_kind,
            owner_kind: scope.owner_kind,
            owner_name: scope.owner_name.clone(),
            source_span: scope.source_span.clone(),
        })
        .collect()
}

pub fn resolve_text(program: &Program, source_diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, source_diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum resolver report ({RESOLVE_REPORT_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {RESOLVE_MODE}\n"));
    out.push_str(&format!(
        "summary: files={} items={} scopes={} definitions={} references={} resolved={} unresolved={} external={} duplicate_definitions={} mutable_place_errors={} resolver_errors={} source_errors={}\n",
        report.files,
        report.items,
        report.scopes.len(),
        report.definitions.len(),
        report.references.len(),
        report.resolved_references(),
        report.unresolved_references(),
        report.external_references(),
        report.duplicate_definitions(),
        report.mutable_place_errors(),
        report.error_count(),
        report.source_errors
    ));

    if report.diagnostics.is_empty() {
        out.push_str("diagnostics: none\n");
    } else {
        out.push_str("diagnostics:\n");
        for diagnostic in &report.diagnostics {
            out.push_str(&format!(
                "  {}:{}:{}: {}[{}]: {}\n",
                diagnostic.source_span.file,
                diagnostic.source_span.line,
                diagnostic.source_span.column,
                diagnostic.severity.as_str(),
                diagnostic.code.as_str(),
                diagnostic.message
            ));
            out.push_str(&format!("    help: {}\n", diagnostic.help));
        }
    }

    out.push_str(&predicate::analyze_program(program).place_facts_text());
    out.push_str("non_claims:\n");
    for non_claim in NON_CLAIMS {
        out.push_str(&format!("  - {non_claim}\n"));
    }

    out
}

pub fn resolve_json(program: &Program, source_diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, source_diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", RESOLVE_REPORT_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", RESOLVE_MODE, true);
    push_summary(&mut out, 2, &report, true);
    push_scopes(&mut out, 2, &report.scopes, true);
    push_definitions(&mut out, 2, &report.definitions, true);
    push_references(&mut out, 2, &report.references, true);
    push_diagnostics(&mut out, 2, &report.diagnostics, true);
    push_indent(&mut out, 2);
    push_json_string(&mut out, "predicate_place_facts");
    out.push_str(": ");
    out.push_str(&predicate::analyze_program(program).place_facts_json());
    out.push_str(",\n");
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, source_diagnostics: &[Diagnostic]) -> ResolveReport {
    let mut context = ResolverContext::new();
    for (file_index, file) in program.files.iter().enumerate() {
        let file_scope = context.add_scope(
            None,
            "file",
            "file",
            &file.path,
            file.items.first().map(|item| item.span()),
            format!("file_{file_index}_{}", id_fragment(&file.path)),
        );
        context.collect_item_definitions(&file.items, &file_scope);
        context.resolve_items(&file.items, &file_scope);
    }

    let source_errors = source_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let source_warnings = source_diagnostics.len().saturating_sub(source_errors);

    ResolveReport {
        files: program.files.len(),
        items: count_items(program),
        source_errors,
        source_warnings,
        scopes: context.scopes,
        definitions: context.definitions,
        references: context.references,
        diagnostics: context.diagnostics,
    }
}

impl ResolverContext {
    fn new() -> Self {
        Self {
            scopes: Vec::new(),
            definitions: Vec::new(),
            references: Vec::new(),
            diagnostics: Vec::new(),
            scope_parents: BTreeMap::new(),
            definitions_by_scope_name: BTreeMap::new(),
            scope_serial: 0,
            definition_serial: 0,
            reference_serial: 0,
        }
    }

    fn add_scope(
        &mut self,
        parent_scope_id: Option<&str>,
        scope_kind: &'static str,
        owner_kind: &'static str,
        owner_name: &str,
        span: Option<&Span>,
        preferred_id: String,
    ) -> String {
        let current = self.scope_serial;
        self.scope_serial += 1;
        let base = if preferred_id.is_empty() {
            format!("scope_{scope_kind}_{}", id_fragment(owner_name))
        } else {
            preferred_id
        };
        let source_identity = span.map_or_else(
            || "generated".to_string(),
            |span| {
                format!(
                    "{}_{}_{}",
                    id_fragment(&portable_path(&span.file)),
                    span.line,
                    span.column
                )
            },
        );
        let id = format!("{base}_{source_identity}_{current}");
        let parent = parent_scope_id.map(str::to_string);
        self.scope_parents.insert(id.clone(), parent.clone());
        self.scopes.push(ResolveScope {
            id: id.clone(),
            parent_scope_id: parent,
            scope_kind,
            owner_kind,
            owner_name: portable_path(owner_name.trim()),
            source_span: span.map(portable_span),
        });
        id
    }

    fn collect_item_definitions(&mut self, items: &[Item], scope_id: &str) {
        for item in items {
            let (definition_kind, mutable, state_kind) = match item {
                Item::App(_) => ("app", false, "item"),
                Item::Type(_) => ("type", false, "type"),
                Item::Store(_) => ("store", true, "store"),
                Item::Task(_) => ("task", false, "callable"),
                Item::Test(_) => ("test", false, "callable"),
            };
            self.add_definition(
                scope_id,
                DefinitionInput {
                    name: item.name(),
                    definition_kind,
                    mutable,
                    state_kind,
                    span: item.span(),
                    defer_duplicate_diagnostic: false,
                },
            );
        }
    }

    fn resolve_items(&mut self, items: &[Item], scope_id: &str) {
        for item in items {
            match item {
                Item::App(app) => {
                    let app_scope = self.add_scope(
                        Some(scope_id),
                        "app",
                        "app",
                        &app.name,
                        Some(&app.span),
                        format!("app_{}_scope", id_fragment(&app.name)),
                    );
                    self.collect_item_definitions(&app.items, &app_scope);
                    self.resolve_items(&app.items, &app_scope);
                }
                Item::Type(type_def) => {
                    let type_scope = self.add_scope(
                        Some(scope_id),
                        "type",
                        "type",
                        &type_def.name,
                        Some(&type_def.span),
                        format!("type_{}_scope", id_fragment(&type_def.name)),
                    );
                    for field in &type_def.fields {
                        self.add_definition(
                            &type_scope,
                            DefinitionInput {
                                name: &field.name,
                                definition_kind: "field",
                                mutable: false,
                                state_kind: "field",
                                span: &field.span,
                                defer_duplicate_diagnostic: false,
                            },
                        );
                    }
                }
                Item::Store(_) => {}
                Item::Task(task) => {
                    self.resolve_callable(
                        scope_id,
                        "task",
                        &task.name,
                        &task.params,
                        &task.sections,
                        &task.span,
                    );
                }
                Item::Test(test) => {
                    self.resolve_callable(
                        scope_id,
                        "test",
                        &test.name,
                        &test.params,
                        &test.sections,
                        &test.span,
                    );
                }
            }
        }
    }

    fn resolve_callable(
        &mut self,
        parent_scope_id: &str,
        owner_kind: &'static str,
        owner_name: &str,
        params: &[Param],
        sections: &[Section],
        span: &Span,
    ) {
        let callable_scope = self.add_scope(
            Some(parent_scope_id),
            "callable",
            owner_kind,
            owner_name,
            Some(span),
            format!("{}_{}_scope", owner_kind, id_fragment(owner_name)),
        );
        for param in params {
            self.add_definition(
                &callable_scope,
                DefinitionInput {
                    name: &param.name,
                    definition_kind: "parameter",
                    mutable: parameter_is_mutable(param),
                    state_kind: parameter_state_kind(param),
                    span: &param.span,
                    defer_duplicate_diagnostic: false,
                },
            );
        }

        self.resolve_declared_sections(&callable_scope, sections);

        if let Some(section) = find_section(sections, "does") {
            self.resolve_does_section(&callable_scope, section);
        }
    }

    fn resolve_declared_sections(&mut self, scope_id: &str, sections: &[Section]) {
        for section in sections {
            let (reference_kind, definition_kind, mutable, state_kind) = match section.name.as_str()
            {
                "uses" => (
                    "declared_use",
                    "declared_use_permission",
                    false,
                    "external_state",
                ),
                "changes" => (
                    "declared_change",
                    "declared_change_permission",
                    true,
                    "external_state",
                ),
                _ => continue,
            };
            for line in &section.lines {
                if !is_meaningful_line_text(&line.text) {
                    continue;
                }
                let Some(name) = declared_name_from_line(&line.text) else {
                    continue;
                };
                self.add_reference(
                    scope_id,
                    PendingReferenceInput {
                        name: &name,
                        reference_kind,
                        mutable_required: false,
                        external_if_unresolved: true,
                        span: &line.span,
                    },
                );
                self.add_definition(
                    scope_id,
                    DefinitionInput {
                        name: &name,
                        definition_kind,
                        mutable,
                        state_kind,
                        span: &line.span,
                        defer_duplicate_diagnostic: false,
                    },
                );
            }
        }
    }

    fn resolve_does_section(&mut self, root_scope_id: &str, section: &Section) {
        let body = core_body::analyze_does_section(section);
        let existing_names = self
            .definitions_by_scope_name
            .keys()
            .filter(|(scope_id, _name)| scope_id == root_scope_id)
            .map(|(_scope_id, name)| name.clone())
            .collect::<BTreeSet<_>>();
        let alias_analysis =
            writable_field_alias::analyze_with_existing_names(&body.statements, &existing_names);
        let mut active_scopes = vec![root_scope_id.to_string()];
        let mut record_literal_depth = 0usize;
        for (statement_index, statement) in body.statements.iter().enumerate() {
            if statement.kind == "block_close" {
                if record_literal_depth > 0 {
                    record_literal_depth -= 1;
                } else if active_scopes.len() > 1 {
                    active_scopes.pop();
                }
                continue;
            }

            let current_scope = active_scopes
                .last()
                .cloned()
                .unwrap_or_else(|| root_scope_id.to_string());
            self.resolve_statement_references(&current_scope, statement);

            match statement.kind {
                "let_binding" => {
                    if let Some(name) = binding_name(&statement.text, "let") {
                        let writable_alias =
                            writable_field_alias::candidate_name(statement).is_some();
                        let alias_rebinding = alias_analysis.issues.iter().any(|issue| {
                            issue.index == statement_index
                                && matches!(
                                    issue.reason,
                                    "writable_alias_rebinding_v0"
                                        | "writable_alias_binding_rebinding_v0"
                                        | "writable_alias_owner_rebinding_v0"
                                        | "writable_alias_rebinds_its_owner_v0"
                                )
                        });
                        self.add_definition(
                            &current_scope,
                            DefinitionInput {
                                name: &name,
                                definition_kind: if writable_alias {
                                    "writable_field_alias"
                                } else {
                                    "let_binding"
                                },
                                mutable: writable_alias,
                                state_kind: if writable_alias {
                                    "writable_field_alias"
                                } else {
                                    "immutable_value"
                                },
                                span: &statement.span,
                                defer_duplicate_diagnostic: alias_rebinding,
                            },
                        );
                    }
                }
                "mutable_binding" => {
                    if let Some(name) = binding_name(&statement.text, "change") {
                        self.add_definition(
                            &current_scope,
                            DefinitionInput {
                                name: &name,
                                definition_kind: "mutable_binding",
                                mutable: true,
                                state_kind: "mutable_local",
                                span: &statement.span,
                                defer_duplicate_diagnostic: false,
                            },
                        );
                    }
                }
                "for_each_header" => {
                    let block_scope = self.open_block_scope(
                        root_scope_id,
                        &current_scope,
                        "for_each",
                        statement_index,
                        &statement.span,
                    );
                    if let Some(name) = for_each_binding(&statement.text) {
                        self.add_definition(
                            &block_scope,
                            DefinitionInput {
                                name: &name,
                                definition_kind: "for_each_binding",
                                mutable: false,
                                state_kind: "immutable_value",
                                span: &statement.span,
                                defer_duplicate_diagnostic: false,
                            },
                        );
                    }
                    active_scopes.push(block_scope);
                }
                "for_index_header" => {
                    let block_scope = self.open_block_scope(
                        root_scope_id,
                        &current_scope,
                        "for_index",
                        statement_index,
                        &statement.span,
                    );
                    if let Some(name) = for_index_binding(&statement.text) {
                        self.add_definition(
                            &block_scope,
                            DefinitionInput {
                                name: &name,
                                definition_kind: "for_index_binding",
                                mutable: false,
                                state_kind: "immutable_value",
                                span: &statement.span,
                                defer_duplicate_diagnostic: false,
                            },
                        );
                    }
                    active_scopes.push(block_scope);
                }
                "if_header" | "while_header" | "loop_header" => {
                    let block_kind = match statement.kind {
                        "if_header" => "if",
                        "while_header" => "while",
                        _ => "loop",
                    };
                    let block_scope = self.open_block_scope(
                        root_scope_id,
                        &current_scope,
                        block_kind,
                        statement_index,
                        &statement.span,
                    );
                    active_scopes.push(block_scope);
                }
                _ => {}
            }

            if statement.expression_kind == Some("record_literal_start") {
                record_literal_depth += 1;
            }
        }
    }

    fn open_block_scope(
        &mut self,
        root_scope_id: &str,
        parent_scope_id: &str,
        block_kind: &'static str,
        statement_index: usize,
        span: &Span,
    ) -> String {
        self.add_scope(
            Some(parent_scope_id),
            block_kind,
            "block",
            block_kind,
            Some(span),
            format!(
                "{}_block_{}_{}_scope",
                root_scope_id, statement_index, block_kind
            ),
        )
    }

    fn resolve_statement_references(&mut self, scope_id: &str, statement: &BodyStatement) {
        if let Some(expression) = expression_text_for_statement(statement) {
            let expression = core_expr::analyze_expression(expression);
            for reference in expression_name_references(&expression) {
                self.add_reference(
                    scope_id,
                    PendingReferenceInput {
                        name: &reference.name,
                        reference_kind: reference.reference_kind,
                        mutable_required: false,
                        external_if_unresolved: reference.external_if_unresolved,
                        span: &statement.span,
                    },
                );
            }
        }

        if let Some(target) = set_target(&statement.text) {
            self.add_reference(
                scope_id,
                PendingReferenceInput {
                    name: &target,
                    reference_kind: "mutation_target",
                    mutable_required: true,
                    external_if_unresolved: false,
                    span: &statement.span,
                },
            );
        }

        if let Some((value, target)) = save_parts(&statement.text) {
            self.add_reference(
                scope_id,
                PendingReferenceInput {
                    name: &value,
                    reference_kind: "store_write_value",
                    mutable_required: false,
                    external_if_unresolved: false,
                    span: &statement.span,
                },
            );
            self.add_reference(
                scope_id,
                PendingReferenceInput {
                    name: &target,
                    reference_kind: "store_write_target",
                    mutable_required: true,
                    external_if_unresolved: false,
                    span: &statement.span,
                },
            );
        }
    }

    fn add_definition(&mut self, scope_id: &str, input: DefinitionInput<'_>) -> Option<String> {
        let normalized_name = name_key(input.name);
        if normalized_name.is_empty() {
            return None;
        }
        let key = (scope_id.to_string(), normalized_name.clone());
        let duplicate = self.definitions_by_scope_name.get(&key).cloned();
        let duplicate_id = duplicate
            .as_ref()
            .map(|existing| self.definitions[existing.index].id.clone());
        let id = format!(
            "def_{}_{}_{}",
            self.definition_serial, input.definition_kind, normalized_name
        );
        self.definition_serial += 1;
        let status = if duplicate.is_some() && input.defer_duplicate_diagnostic {
            "duplicate_definition_deferred_to_ownership_v0"
        } else if duplicate.is_some() {
            "duplicate_definition_v0"
        } else {
            "defined_v0"
        };
        let definition_index = self.definitions.len();
        self.definitions.push(ResolveDefinition {
            id: id.clone(),
            name: input.name.trim().to_string(),
            normalized_name: normalized_name.clone(),
            definition_kind: input.definition_kind,
            scope_id: scope_id.to_string(),
            mutable: input.mutable,
            state_kind: input.state_kind,
            source_span: portable_span(input.span),
            status,
            duplicate_of: duplicate_id.clone(),
        });
        if duplicate.is_none() {
            self.definitions_by_scope_name.insert(
                key,
                DefinitionRef {
                    index: definition_index,
                },
            );
        } else if !input.defer_duplicate_diagnostic {
            self.add_duplicate_definition_diagnostic(
                input.name,
                input.span,
                &id,
                duplicate_id.as_deref(),
            );
        }
        self.add_read_before_declare_diagnostics(&normalized_name, input.span, &id);
        Some(id)
    }

    fn add_reference(
        &mut self,
        scope_id: &str,
        input: PendingReferenceInput<'_>,
    ) -> Option<String> {
        let normalized_name = name_key(input.name);
        if normalized_name.is_empty() {
            return None;
        }
        let resolved = self
            .resolve_definition(scope_id, &normalized_name, input.reference_kind)
            .map(|definition| {
                (
                    definition.id.clone(),
                    definition.mutable,
                    definition.definition_kind,
                )
            });
        let builtin_name = input.name.trim();
        let builtin_callee = input.reference_kind == "callee_ref"
            && matches!(
                builtin_name,
                "stdout_write" | "clock_replay_tick" | "files_read_text"
            );
        let app_local_callee = input.reference_kind == "callee_ref"
            && self.scope_is_within_app_boundary(scope_id)
            && !builtin_callee;
        let external =
            (input.external_if_unresolved || is_external_root(input.name)) && !app_local_callee;
        let id = format!(
            "ref_{}_{}_{}",
            self.reference_serial, input.reference_kind, normalized_name
        );
        self.reference_serial += 1;

        let (resolution_status, resolved_definition_id, reason) = if builtin_callee {
            (
                "builtin_reference_v0",
                None,
                Some(match builtin_name {
                    "stdout_write" => "session_z_stdout_builtin_v0",
                    "clock_replay_tick" => "session_aa_runner_replay_builtin_v0",
                    "files_read_text" => "session_ad_exact_file_read_builtin_v0",
                    _ => unreachable!("pinned builtin"),
                }),
            )
        } else if let Some((definition_id, mutable, definition_kind)) = resolved {
            if input.mutable_required && !mutable && definition_kind != "parameter" {
                (
                    "resolved_immutable_place_v0",
                    Some(definition_id),
                    Some("target_is_not_mutable"),
                )
            } else {
                ("resolved_v0", Some(definition_id), None)
            }
        } else if external {
            (
                "external_reference_v0",
                None,
                Some("outside_current_source_or_capability_v0"),
            )
        } else {
            ("unresolved_v0", None, Some("name_not_in_visible_scope"))
        };

        self.references.push(ResolveReference {
            id: id.clone(),
            name: input.name.trim().to_string(),
            normalized_name,
            reference_kind: input.reference_kind,
            scope_id: scope_id.to_string(),
            mutable_required: input.mutable_required,
            source_span: portable_span(input.span),
            resolution_status,
            resolved_definition_id: resolved_definition_id.clone(),
            reason,
        });

        match resolution_status {
            "unresolved_v0" => {
                self.add_unresolved_name_diagnostic(input.name, input.span, &id, app_local_callee);
            }
            "resolved_immutable_place_v0" => {
                self.add_immutable_place_diagnostic(
                    input.name,
                    input.span,
                    &id,
                    resolved_definition_id.as_deref(),
                );
            }
            _ => {}
        }

        Some(id)
    }

    fn resolve_definition(
        &self,
        scope_id: &str,
        normalized_name: &str,
        reference_kind: &str,
    ) -> Option<&ResolveDefinition> {
        let mut cursor = Some(scope_id.to_string());
        while let Some(current_scope) = cursor {
            let key = (current_scope.clone(), normalized_name.to_string());
            if let Some(definition) = self
                .definitions_by_scope_name
                .get(&key)
                .map(|definition| &self.definitions[definition.index])
                .filter(|definition| {
                    reference_kind != "callee_ref"
                        || matches!(definition.definition_kind, "task" | "test" | "type")
                })
            {
                return Some(definition);
            }
            if reference_kind == "callee_ref" && self.is_app_scope(&current_scope) {
                return None;
            }
            cursor = self
                .scope_parents
                .get(&current_scope)
                .and_then(|parent| parent.clone());
        }
        None
    }

    fn is_app_scope(&self, scope_id: &str) -> bool {
        self.scopes
            .iter()
            .any(|scope| scope.id == scope_id && scope.scope_kind == "app")
    }

    fn scope_is_within_app_boundary(&self, scope_id: &str) -> bool {
        let mut cursor = Some(scope_id.to_string());
        while let Some(current_scope) = cursor {
            if self.is_app_scope(&current_scope) {
                return true;
            }
            cursor = self
                .scope_parents
                .get(&current_scope)
                .and_then(|parent| parent.clone());
        }
        false
    }

    fn add_unresolved_name_diagnostic(
        &mut self,
        name: &str,
        span: &Span,
        reference_id: &str,
        app_local_callee: bool,
    ) {
        self.diagnostics.push(ResolverDiagnostic {
            code: DiagnosticCode::UNRESOLVED_NAME,
            severity: Severity::Error,
            title: DiagnosticCode::UNRESOLVED_NAME.title(),
            message: format!("name `{}` is not visible in this scope", name.trim()),
            source_span: portable_span(span),
            help: if app_local_callee {
                "Declare or nest the helper task inside the current app; app-local calls cannot resolve file-level helpers."
            } else {
                "Declare the name before use, add a matching item, or list an external dependency under `uses:` when it is intentional."
            },
            reference_id: Some(reference_id.to_string()),
            definition_id: None,
        });
    }

    fn add_duplicate_definition_diagnostic(
        &mut self,
        name: &str,
        span: &Span,
        definition_id: &str,
        duplicate_of: Option<&str>,
    ) {
        self.diagnostics.push(ResolverDiagnostic {
            code: DiagnosticCode::DUPLICATE_NAME_IN_SCOPE,
            severity: Severity::Error,
            title: DiagnosticCode::DUPLICATE_NAME_IN_SCOPE.title(),
            message: format!("name `{}` is already defined in this scope", name.trim()),
            source_span: portable_span(span),
            help: "Rename one binding or move it into a narrower block so each scope has one definition for the name.",
            reference_id: duplicate_of.map(str::to_string),
            definition_id: Some(definition_id.to_string()),
        });
    }

    fn add_immutable_place_diagnostic(
        &mut self,
        name: &str,
        span: &Span,
        reference_id: &str,
        definition_id: Option<&str>,
    ) {
        self.diagnostics.push(ResolverDiagnostic {
            code: DiagnosticCode::SET_TARGET_IMMUTABLE,
            severity: Severity::Error,
            title: DiagnosticCode::SET_TARGET_IMMUTABLE.title(),
            message: format!("cannot mutate immutable place `{}`", name.trim()),
            source_span: portable_span(span),
            help: "Declare the local with `change`, target a declared `changes:` permission, or keep the value immutable.",
            reference_id: Some(reference_id.to_string()),
            definition_id: definition_id.map(str::to_string),
        });
    }

    fn add_read_before_declare_diagnostics(
        &mut self,
        normalized_name: &str,
        definition_span: &Span,
        definition_id: &str,
    ) {
        let mut seen = BTreeSet::new();
        for reference in &self.references {
            if reference.normalized_name != normalized_name
                || reference.resolution_status != "unresolved_v0"
                || reference.source_span.file != portable_path(&definition_span.file)
                || reference.source_span.line >= definition_span.line
                || !seen.insert(reference.id.clone())
            {
                continue;
            }
            self.diagnostics.push(ResolverDiagnostic {
                code: DiagnosticCode::READ_BEFORE_DECLARE,
                severity: Severity::Error,
                title: DiagnosticCode::READ_BEFORE_DECLARE.title(),
                message: format!(
                    "name `{}` is read before its declaration",
                    reference.name.trim()
                ),
                source_span: reference.source_span.clone(),
                help: "Move the declaration above the read or pass the value in through an explicit parameter.",
                reference_id: Some(reference.id.clone()),
                definition_id: Some(definition_id.to_string()),
            });
        }
    }
}

impl ResolveReport {
    fn status(&self) -> &'static str {
        if self.source_errors > 0 {
            "blocked_by_source_errors"
        } else if self.error_count() > 0 {
            "checked_resolver_with_errors_v0"
        } else {
            "checked_resolver_v0"
        }
    }

    fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .count()
    }

    fn warning_count(&self) -> usize {
        self.diagnostics.len().saturating_sub(self.error_count())
    }

    fn resolved_references(&self) -> usize {
        self.references
            .iter()
            .filter(|reference| reference.resolution_status == "resolved_v0")
            .count()
    }

    fn unresolved_references(&self) -> usize {
        self.references
            .iter()
            .filter(|reference| reference.resolution_status == "unresolved_v0")
            .count()
    }

    fn external_references(&self) -> usize {
        self.references
            .iter()
            .filter(|reference| reference.resolution_status == "external_reference_v0")
            .count()
    }

    fn duplicate_definitions(&self) -> usize {
        self.definitions
            .iter()
            .filter(|definition| definition.status == "duplicate_definition_v0")
            .count()
    }

    fn mutable_place_errors(&self) -> usize {
        self.references
            .iter()
            .filter(|reference| reference.resolution_status == "resolved_immutable_place_v0")
            .count()
    }
}

struct PendingNameReference {
    name: String,
    reference_kind: &'static str,
    external_if_unresolved: bool,
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
            "surface_text" => {
                if let Some(name) = permission_argument_name(&atom.text) {
                    references.push(PendingNameReference {
                        name,
                        reference_kind: "name_ref",
                        external_if_unresolved: false,
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

fn for_each_collection(text: &str) -> Option<&str> {
    let body = header_body(text, "for each")?;
    body.split_once(" in ")
        .map(|(_binding, collection)| collection.trim())
}

fn for_index_binding(text: &str) -> Option<String> {
    let body = header_body(text, "for index")?;
    body.split_whitespace().next().map(str::to_string)
}

fn set_target(text: &str) -> Option<String> {
    let rest = strip_keyword(text, "set")?;
    rest.split_once('=')
        .map(|(target, _value)| place_root(target.trim()))
        .filter(|target| !target.is_empty())
}

fn save_parts(text: &str) -> Option<(String, String)> {
    let rest = strip_keyword(text, "save")?;
    let (value, target) = rest.rsplit_once(" in ")?;
    Some((value.trim().to_string(), target.trim().to_string()))
}

fn declared_name_from_line(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Some((key, value)) = text.split_once(':')
        && matches!(
            key.trim(),
            "triple" | "record" | "target" | "requires" | "denies"
        )
    {
        return Some(value.trim().to_string());
    }
    if let Some((root, _field)) = text.split_once('.') {
        return Some(root.trim().to_string());
    }
    Some(text.to_string())
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

fn parameter_is_mutable(param: &Param) -> bool {
    matches!(
        param.permission,
        ParamPermission::Change | ParamPermission::Consume
    )
}

fn parameter_state_kind(param: &Param) -> &'static str {
    match param.permission {
        ParamPermission::Borrow => "borrow_parameter",
        ParamPermission::Change => "change_parameter",
        ParamPermission::Consume => "consume_parameter",
    }
}

fn permission_argument_name(text: &str) -> Option<String> {
    let rest = ["borrow", "change", "consume"]
        .iter()
        .find_map(|keyword| strip_keyword(text.trim(), keyword))?;
    let name = place_root(rest);
    if name.is_empty() { None } else { Some(name) }
}

fn place_root(text: &str) -> String {
    text.split(|ch: char| ch == '.' || ch == '[' || ch.is_whitespace() || ch == ',')
        .find(|part| !part.is_empty())
        .unwrap_or(text)
        .trim()
        .to_string()
}

fn path_root(text: &str) -> Option<String> {
    let text = strip_permission_expression(text);
    text.split(['.', '['])
        .next()
        .map(str::trim)
        .filter(|root| !root.is_empty() && *root != text)
        .map(str::to_string)
}

fn strip_permission_expression(text: &str) -> &str {
    ["borrow", "change", "consume"]
        .iter()
        .find_map(|keyword| strip_keyword(text.trim(), keyword))
        .unwrap_or(text)
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

fn find_section<'a>(sections: &'a [Section], name: &str) -> Option<&'a Section> {
    sections.iter().find(|section| section.name == name)
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

fn name_key(name: &str) -> String {
    snake_identifier(name)
}

fn id_fragment(text: &str) -> String {
    let mut fragment = snake_identifier(text);
    if fragment.is_empty() {
        fragment.push_str("root");
    }
    if fragment.len() > 64 {
        fragment.truncate(64);
        fragment = fragment.trim_matches('_').to_string();
    }
    fragment
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
        file: portable_path(&span.file),
        line: span.line,
        column: span.column,
    }
}

fn portable_path(path: &str) -> String {
    path.replace('\\', "/")
}

const NON_CLAIMS: &[&str] = &[
    "no type checking",
    "no borrow checking",
    "no lifetime inference",
    "no effect checking",
    "no module import resolution",
    "no executable semantics",
    "no optimizer authority",
];

fn push_summary(out: &mut String, indent: usize, report: &ResolveReport, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, "summary");
    out.push_str(": {\n");
    push_usize_field(out, indent + 2, "files", report.files, true);
    push_usize_field(out, indent + 2, "items", report.items, true);
    push_usize_field(out, indent + 2, "source_errors", report.source_errors, true);
    push_usize_field(
        out,
        indent + 2,
        "source_warnings",
        report.source_warnings,
        true,
    );
    push_usize_field(out, indent + 2, "scopes", report.scopes.len(), true);
    push_usize_field(
        out,
        indent + 2,
        "definitions",
        report.definitions.len(),
        true,
    );
    push_usize_field(out, indent + 2, "references", report.references.len(), true);
    push_usize_field(
        out,
        indent + 2,
        "resolved_references",
        report.resolved_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unresolved_references",
        report.unresolved_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "external_references",
        report.external_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "duplicate_definitions",
        report.duplicate_definitions(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "mutable_place_errors",
        report.mutable_place_errors(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        report.error_count(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_warnings",
        report.warning_count(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_scopes(out: &mut String, indent: usize, scopes: &[ResolveScope], comma: bool) {
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
            push_scope(out, indent + 2, scope);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_scope(out: &mut String, indent: usize, scope: &ResolveScope) {
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
    push_string_field(out, indent + 2, "owner_kind", scope.owner_kind, true);
    push_string_field(out, indent + 2, "owner_name", &scope.owner_name, true);
    push_optional_span_field(
        out,
        indent + 2,
        "source_span",
        scope.source_span.as_ref(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
}

fn push_definitions(
    out: &mut String,
    indent: usize,
    definitions: &[ResolveDefinition],
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
            push_definition(out, indent + 2, definition);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_definition(out: &mut String, indent: usize, definition: &ResolveDefinition) {
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
    push_bool_field(out, indent + 2, "mutable", definition.mutable, true);
    push_string_field(out, indent + 2, "state_kind", definition.state_kind, true);
    push_span_field(
        out,
        indent + 2,
        "source_span",
        &definition.source_span,
        true,
    );
    push_string_field(out, indent + 2, "status", definition.status, true);
    push_optional_string_field(
        out,
        indent + 2,
        "duplicate_of",
        definition.duplicate_of.as_deref(),
        false,
    );
    push_indent(out, indent);
    out.push('}');
}

fn push_references(out: &mut String, indent: usize, references: &[ResolveReference], comma: bool) {
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
            push_reference(out, indent + 2, reference);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_reference(out: &mut String, indent: usize, reference: &ResolveReference) {
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
    push_bool_field(
        out,
        indent + 2,
        "mutable_required",
        reference.mutable_required,
        true,
    );
    push_span_field(out, indent + 2, "source_span", &reference.source_span, true);
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

fn push_diagnostics(
    out: &mut String,
    indent: usize,
    diagnostics: &[ResolverDiagnostic],
    comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, "diagnostics");
    out.push_str(": [");
    if !diagnostics.is_empty() {
        out.push('\n');
        for (index, diagnostic) in diagnostics.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            push_resolver_diagnostic(out, indent + 2, diagnostic);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_resolver_diagnostic(out: &mut String, indent: usize, diagnostic: &ResolverDiagnostic) {
    out.push_str("{\n");
    push_string_field(out, indent + 2, "code", diagnostic.code.as_str(), true);
    push_string_field(out, indent + 2, "title", diagnostic.title, true);
    push_string_field(
        out,
        indent + 2,
        "severity",
        diagnostic.severity.as_str(),
        true,
    );
    push_string_field(out, indent + 2, "message", &diagnostic.message, true);
    push_span_field(
        out,
        indent + 2,
        "source_span",
        &diagnostic.source_span,
        true,
    );
    push_string_field(out, indent + 2, "help", diagnostic.help, true);
    push_optional_string_field(
        out,
        indent + 2,
        "reference_id",
        diagnostic.reference_id.as_deref(),
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "definition_id",
        diagnostic.definition_id.as_deref(),
        false,
    );
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
        push_span_object(out, span);
    } else {
        out.push_str("null");
    }
    push_comma_newline(out, comma);
}

fn push_span_field(out: &mut String, indent: usize, key: &str, span: &Span, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_span_object(out, span);
    push_comma_newline(out, comma);
}

fn push_span_object(out: &mut String, span: &Span) {
    out.push('{');
    out.push_str("\"file\": ");
    push_json_string(out, &span.file);
    out.push_str(&format!(
        ", \"line\": {}, \"column\": {}",
        span.line, span.column
    ));
    out.push('}');
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

fn push_bool_field(out: &mut String, indent: usize, key: &str, value: bool, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    out.push_str(if value { "true" } else { "false" });
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

    use super::{resolve_json, resolve_text};

    #[test]
    fn json_resolves_items_permissions_and_local_places() {
        let source = r#"type WorkItem {
  title: Text
}

store work_items: list WorkItem {
  why:
    remember work
}

task remember_work_item(title: Text) -> WorkItem {
  uses:
    clock

  changes:
    work_items

  does:
    let item = WorkItem {
      title: title
    }
    save item in work_items
    return item
}
"#;
        let parsed = parse_source("resolve.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = resolve_json(&program, &[]);

        assert!(json.contains("\"schema\": \"hum.resolve.v0\""));
        assert!(json.contains("\"mode\": \"source_analysis_only_no_type_or_borrow_check\""));
        assert!(json.contains("\"definition_kind\": \"store\""));
        assert!(json.contains("\"definition_kind\": \"declared_change_permission\""));
        assert!(json.contains("\"reference_kind\": \"declared_change\""));
        assert!(json.contains("\"reference_kind\": \"store_write_target\""));
        assert!(json.contains("\"reference_kind\": \"store_write_value\""));
        assert!(json.contains("\"resolution_status\": \"resolved_v0\""));
        assert!(json.contains("\"normalized_name\": \"work_items\""));
        assert!(json.contains("\"normalized_name\": \"item\""));
        assert!(json.contains("\"no type checking\""));
        assert!(json.contains("\"no borrow checking\""));
    }

    #[test]
    fn json_reports_unresolved_duplicates_read_before_declare_and_immutable_set() {
        let source = r#"task bad_names() -> UInt {
  does:
    return later
    let later: UInt = 0
    let frozen: UInt = 1
    let frozen: UInt = 2
    set frozen = 3
    return missing
}
"#;
        let parsed = parse_source("bad.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = resolve_json(&program, &[]);

        assert!(json.contains("\"status\": \"checked_resolver_with_errors_v0\""));
        assert!(json.contains("\"code\": \"H0601\""));
        assert!(json.contains("\"code\": \"H0602\""));
        assert!(json.contains("\"code\": \"H0603\""));
        assert!(json.contains("\"code\": \"H0604\""));
        assert!(json.contains("\"resolution_status\": \"resolved_immutable_place_v0\""));
        assert!(json.contains("\"status\": \"duplicate_definition_v0\""));
    }

    #[test]
    fn text_report_summarizes_without_execution_claims() {
        let source = r#"task ok(value: UInt) -> UInt {
  does:
    return value
}
"#;
        let parsed = parse_source("ok.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let text = resolve_text(&program, &[]);

        assert!(text.contains("Hum resolver report (hum.resolve.v0)"));
        assert!(text.contains("summary: files=1 items=1"));
        assert!(text.contains("diagnostics: none"));
        assert!(text.contains("no executable semantics"));
    }

    #[test]
    fn writable_alias_rebinding_defers_duplicate_blame_to_ownership() {
        let parsed = parse_source(
            "fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum",
            include_str!("../fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum"),
        );
        let program = Program {
            files: vec![parsed.file],
        };
        let json = resolve_json(&program, &[]);

        assert!(json.contains("\"resolver_errors\": 0"));
        assert!(json.contains("\"duplicate_definition_deferred_to_ownership_v0\""));
        assert!(!json.contains("\"code\": \"H0602\""));
    }
}
