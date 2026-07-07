use std::collections::BTreeSet;

use crate::ast::{App, Field, Item, Param, Program, Store, Task, Test, TypeDef};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::resolve;
use crate::version;

pub const TYPE_ENV_SCHEMA: &str = "hum.type_env.v0";
pub const TYPE_ENV_MODE: &str = "declaration_inventory_no_type_check";

const RESERVED_TYPE_ROOTS: &[&str] = &[
    "Unit", "Bool", "Int", "UInt", "Float", "Text", "Bytes", "Result", "Option", "Maybe", "list",
    "List", "Vec", "Slice", "Span", "Map", "Set",
];

const NON_CLAIMS: &[&str] = &[
    "no full type checking",
    "no expression type inference",
    "no generic validation",
    "no trait or interface checking",
    "no layout or ABI proof",
    "no borrow checking",
    "no executable semantics",
];

#[derive(Debug, Clone)]
pub struct TypeEnvReport {
    pub files: usize,
    pub items: usize,
    pub source_errors: usize,
    pub source_warnings: usize,
    pub resolver_summary: resolve::ResolveReadinessSummary,
    pub resolver_definitions: Vec<resolve::ResolveDefinitionSummary>,
    pub type_names: Vec<TypeNameFact>,
    pub declarations: Vec<TypeDeclaration>,
}

#[derive(Debug, Clone)]
pub struct TypeNameFact {
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub resolver_definition_id: Option<String>,
    pub source_span: Span,
    pub status: &'static str,
}

#[derive(Debug, Clone)]
pub struct TypeDeclaration {
    pub id: String,
    pub declaration_kind: &'static str,
    pub owner_kind: &'static str,
    pub owner_name: String,
    pub name: String,
    pub resolver_definition_id: Option<String>,
    pub source_span: Span,
    pub type_text: String,
    pub type_references: Vec<TypeReferenceFact>,
    pub status: &'static str,
}

#[derive(Debug, Clone)]
pub struct TypeReferenceFact {
    pub text: String,
    pub normalized_name: String,
    pub role: &'static str,
    pub status: &'static str,
}

struct DeclarationInput<'a> {
    declaration_kind: &'static str,
    owner_kind: &'static str,
    owner_name: &'a str,
    name: &'a str,
    resolver_definition_id: Option<String>,
    span: &'a Span,
    type_text: &'a str,
}

pub fn type_env_has_errors(program: &Program, diagnostics: &[Diagnostic]) -> bool {
    let summary = resolve::resolve_readiness_summary(program, diagnostics);
    summary.source_errors > 0 || summary.resolver_errors > 0
}

pub fn type_env_report(program: &Program, diagnostics: &[Diagnostic]) -> TypeEnvReport {
    build_report(program, diagnostics)
}

pub fn type_env_text(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str(&format!("Hum type environment ({TYPE_ENV_SCHEMA})\n"));
    out.push_str(&format!(
        "tool: hum {} {}\n",
        version::HUM_VERSION,
        version::HUM_STATUS
    ));
    out.push_str(&format!("milestone: {}\n", version::HUM_MILESTONE));
    out.push_str(&format!("mode: {TYPE_ENV_MODE}\n"));
    out.push_str(&format!("status: {}\n", report.status()));
    out.push_str(&format!(
        "resolver: schema={} status={} mode={} definitions={} resolver_errors={}\n",
        report.resolver_summary.schema,
        report.resolver_summary.status,
        report.resolver_summary.mode,
        report.resolver_summary.definitions,
        report.resolver_summary.resolver_errors
    ));
    out.push_str(&format!(
        "summary: files={} items={} type_names={} declarations={} type_references={} unknown_type_references={} source_errors={} resolver_errors={}\n",
        report.files,
        report.items,
        report.type_names.len(),
        report.declarations.len(),
        report.type_reference_count(),
        report.unknown_type_references(),
        report.source_errors,
        report.resolver_summary.resolver_errors
    ));
    out.push_str("reserved_type_roots:");
    for root in RESERVED_TYPE_ROOTS {
        out.push(' ');
        out.push_str(root);
    }
    out.push('\n');

    if report.type_names.is_empty() {
        out.push_str("type_names: none\n");
    } else {
        out.push_str("type_names:\n");
        for type_name in &report.type_names {
            out.push_str(&format!(
                "  {}:{}:{} [{}] type `{}` resolver_definition_id={}\n",
                type_name.source_span.file,
                type_name.source_span.line,
                type_name.source_span.column,
                type_name.status,
                type_name.name,
                type_name
                    .resolver_definition_id
                    .as_deref()
                    .unwrap_or("none")
            ));
        }
    }

    if report.declarations.is_empty() {
        out.push_str("declarations: none\n");
    } else {
        out.push_str("declarations:\n");
        for declaration in &report.declarations {
            out.push_str(&format!(
                "  {}:{}:{} [{}] {} {} `{}`: {}\n",
                declaration.source_span.file,
                declaration.source_span.line,
                declaration.source_span.column,
                declaration.status,
                declaration.owner_kind,
                declaration.declaration_kind,
                declaration.name,
                declaration.type_text
            ));
            if !declaration.type_references.is_empty() {
                out.push_str("    type_references:");
                for reference in &declaration.type_references {
                    out.push_str(&format!(" {}[{}]", reference.text, reference.status));
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

pub fn type_env_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let report = build_report(program, diagnostics);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", TYPE_ENV_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", report.status(), true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "mode", TYPE_ENV_MODE, true);
    push_resolver_summary(&mut out, &report.resolver_summary, 2, true);
    push_summary(&mut out, &report, 2, true);
    push_string_array(
        &mut out,
        2,
        "reserved_type_roots",
        RESERVED_TYPE_ROOTS,
        true,
    );
    push_type_names(&mut out, 2, &report.type_names, true);
    push_declarations(&mut out, 2, &report.declarations, true);
    push_string_array(&mut out, 2, "non_claims_v0", NON_CLAIMS, false);
    out.push_str("}\n");
    out
}

fn build_report(program: &Program, diagnostics: &[Diagnostic]) -> TypeEnvReport {
    let resolver_summary = resolve::resolve_readiness_summary(program, diagnostics);
    let resolver_definitions = resolve::resolve_definition_summaries(program, diagnostics);
    let declared_types = declared_type_names(program);
    let source_errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let source_warnings = diagnostics.len().saturating_sub(source_errors);

    let mut type_names = Vec::new();
    let mut declarations = Vec::new();
    for file in &program.files {
        collect_items(
            &file.items,
            &resolver_definitions,
            &declared_types,
            &mut type_names,
            &mut declarations,
        );
    }

    TypeEnvReport {
        files: program.files.len(),
        items: count_items(program),
        source_errors,
        source_warnings,
        resolver_summary,
        resolver_definitions,
        type_names,
        declarations,
    }
}

fn collect_items(
    items: &[Item],
    resolver_definitions: &[resolve::ResolveDefinitionSummary],
    declared_types: &BTreeSet<String>,
    type_names: &mut Vec<TypeNameFact>,
    declarations: &mut Vec<TypeDeclaration>,
) {
    for item in items {
        match item {
            Item::App(app) => collect_app(
                app,
                resolver_definitions,
                declared_types,
                type_names,
                declarations,
            ),
            Item::Type(type_def) => collect_type(
                type_def,
                resolver_definitions,
                declared_types,
                type_names,
                declarations,
            ),
            Item::Store(store) => {
                collect_store(store, resolver_definitions, declared_types, declarations)
            }
            Item::Task(task) => {
                collect_task(task, resolver_definitions, declared_types, declarations)
            }
            Item::Test(test) => {
                collect_test(test, resolver_definitions, declared_types, declarations)
            }
        }
    }
}

fn collect_app(
    app: &App,
    resolver_definitions: &[resolve::ResolveDefinitionSummary],
    declared_types: &BTreeSet<String>,
    type_names: &mut Vec<TypeNameFact>,
    declarations: &mut Vec<TypeDeclaration>,
) {
    collect_items(
        &app.items,
        resolver_definitions,
        declared_types,
        type_names,
        declarations,
    );
}

fn collect_type(
    type_def: &TypeDef,
    resolver_definitions: &[resolve::ResolveDefinitionSummary],
    declared_types: &BTreeSet<String>,
    type_names: &mut Vec<TypeNameFact>,
    declarations: &mut Vec<TypeDeclaration>,
) {
    let resolver_definition_id =
        find_definition_id(resolver_definitions, "type", &type_def.name, &type_def.span);
    type_names.push(TypeNameFact {
        id: prefixed_id(
            "hum_type_name",
            &format!("{}_{}", type_def.name, type_def.span.line),
        ),
        name: type_def.name.clone(),
        normalized_name: name_key(&type_def.name),
        resolver_definition_id,
        source_span: portable_span(&type_def.span),
        status: "declared_type_name_v0",
    });

    for field in &type_def.fields {
        collect_field(
            type_def,
            field,
            resolver_definitions,
            declared_types,
            declarations,
        );
    }
}

fn collect_field(
    type_def: &TypeDef,
    field: &Field,
    resolver_definitions: &[resolve::ResolveDefinitionSummary],
    declared_types: &BTreeSet<String>,
    declarations: &mut Vec<TypeDeclaration>,
) {
    let resolver_definition_id =
        find_definition_id(resolver_definitions, "field", &field.name, &field.span);
    declarations.push(type_declaration(
        DeclarationInput {
            declaration_kind: "field",
            owner_kind: "type",
            owner_name: &type_def.name,
            name: &field.name,
            resolver_definition_id,
            span: &field.span,
            type_text: &field.ty,
        },
        declared_types,
    ));
}

fn collect_store(
    store: &Store,
    resolver_definitions: &[resolve::ResolveDefinitionSummary],
    declared_types: &BTreeSet<String>,
    declarations: &mut Vec<TypeDeclaration>,
) {
    let resolver_definition_id =
        find_definition_id(resolver_definitions, "store", &store.name, &store.span);
    declarations.push(type_declaration(
        DeclarationInput {
            declaration_kind: "store",
            owner_kind: "store",
            owner_name: &store.name,
            name: &store.name,
            resolver_definition_id,
            span: &store.span,
            type_text: &store.ty,
        },
        declared_types,
    ));
}

fn collect_task(
    task: &Task,
    resolver_definitions: &[resolve::ResolveDefinitionSummary],
    declared_types: &BTreeSet<String>,
    declarations: &mut Vec<TypeDeclaration>,
) {
    for param in &task.params {
        collect_param(
            "parameter",
            "task",
            &task.name,
            param,
            resolver_definitions,
            declared_types,
            declarations,
        );
    }
    if let Some(result) = &task.result {
        declarations.push(type_declaration(
            DeclarationInput {
                declaration_kind: "result",
                owner_kind: "task",
                owner_name: &task.name,
                name: "return",
                resolver_definition_id: None,
                span: &task.span,
                type_text: result,
            },
            declared_types,
        ));
    }
}

fn collect_test(
    test: &Test,
    resolver_definitions: &[resolve::ResolveDefinitionSummary],
    declared_types: &BTreeSet<String>,
    declarations: &mut Vec<TypeDeclaration>,
) {
    for param in &test.params {
        collect_param(
            "parameter",
            "test",
            &test.name,
            param,
            resolver_definitions,
            declared_types,
            declarations,
        );
    }
}

fn collect_param(
    declaration_kind: &'static str,
    owner_kind: &'static str,
    owner_name: &str,
    param: &Param,
    resolver_definitions: &[resolve::ResolveDefinitionSummary],
    declared_types: &BTreeSet<String>,
    declarations: &mut Vec<TypeDeclaration>,
) {
    let resolver_definition_id =
        find_definition_id(resolver_definitions, "parameter", &param.name, &param.span);
    declarations.push(type_declaration(
        DeclarationInput {
            declaration_kind,
            owner_kind,
            owner_name,
            name: &param.name,
            resolver_definition_id,
            span: &param.span,
            type_text: &param.ty,
        },
        declared_types,
    ));
}

fn type_declaration(
    input: DeclarationInput<'_>,
    declared_types: &BTreeSet<String>,
) -> TypeDeclaration {
    let type_references = type_references(input.type_text, declared_types);
    let status = declaration_status(&type_references, input.type_text);
    TypeDeclaration {
        id: prefixed_id(
            "hum_type_decl",
            &format!(
                "{}_{}_{}_{}_{}",
                input.declaration_kind,
                input.owner_kind,
                input.owner_name,
                input.name,
                input.span.line
            ),
        ),
        declaration_kind: input.declaration_kind,
        owner_kind: input.owner_kind,
        owner_name: input.owner_name.to_string(),
        name: input.name.to_string(),
        resolver_definition_id: input.resolver_definition_id,
        source_span: portable_span(input.span),
        type_text: input.type_text.trim().to_string(),
        type_references,
        status,
    }
}

fn type_references(type_text: &str, declared_types: &BTreeSet<String>) -> Vec<TypeReferenceFact> {
    type_tokens(type_text)
        .into_iter()
        .enumerate()
        .map(|(index, token)| {
            let normalized_name = name_key(&token);
            let status = if declared_types.contains(&normalized_name) {
                "declared_type_v0"
            } else if is_reserved_type_name(&normalized_name) {
                "reserved_type_v0"
            } else {
                "unknown_type_name_v0"
            };
            TypeReferenceFact {
                text: token,
                normalized_name,
                role: if index == 0 {
                    "type_root"
                } else {
                    "type_argument"
                },
                status,
            }
        })
        .collect()
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

fn declaration_status(type_references: &[TypeReferenceFact], type_text: &str) -> &'static str {
    if type_text.trim().is_empty() {
        "missing_type_annotation_v0"
    } else if type_references
        .iter()
        .any(|reference| reference.status == "unknown_type_name_v0")
    {
        "contains_unknown_type_names_v0"
    } else if type_references
        .iter()
        .any(|reference| reference.status == "declared_type_v0")
    {
        "references_declared_type_v0"
    } else {
        "reserved_type_annotation_v0"
    }
}

fn declared_type_names(program: &Program) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for file in &program.files {
        collect_declared_type_names(&file.items, &mut names);
    }
    names
}

fn collect_declared_type_names(items: &[Item], names: &mut BTreeSet<String>) {
    for item in items {
        match item {
            Item::App(app) => collect_declared_type_names(&app.items, names),
            Item::Type(type_def) => {
                names.insert(name_key(&type_def.name));
            }
            _ => {}
        }
    }
}

fn find_definition_id(
    definitions: &[resolve::ResolveDefinitionSummary],
    definition_kind: &str,
    name: &str,
    span: &Span,
) -> Option<String> {
    let normalized_name = name_key(name);
    let portable = portable_span(span);
    definitions
        .iter()
        .find(|definition| {
            definition.definition_kind == definition_kind
                && definition.normalized_name == normalized_name
                && definition.source_span.file == portable.file
                && definition.source_span.line == portable.line
                && definition.source_span.column == portable.column
        })
        .map(|definition| definition.id.clone())
}

fn is_reserved_type_name(normalized_name: &str) -> bool {
    RESERVED_TYPE_ROOTS
        .iter()
        .any(|name| name_key(name) == normalized_name)
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

impl TypeEnvReport {
    pub fn status(&self) -> &'static str {
        if self.source_errors > 0 {
            "blocked_by_source_errors"
        } else if self.resolver_summary.resolver_errors > 0 {
            "blocked_by_resolver_errors"
        } else if self.unknown_type_references() > 0 {
            "type_environment_with_unknowns_v0"
        } else {
            "type_environment_v0"
        }
    }

    pub fn type_reference_count(&self) -> usize {
        self.declarations
            .iter()
            .map(|declaration| declaration.type_references.len())
            .sum()
    }

    pub fn unknown_type_references(&self) -> usize {
        self.declarations
            .iter()
            .flat_map(|declaration| declaration.type_references.iter())
            .filter(|reference| reference.status == "unknown_type_name_v0")
            .count()
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

fn portable_span(span: &Span) -> Span {
    Span {
        file: span.file.replace('\\', "/"),
        line: span.line,
        column: span.column,
    }
}

fn push_resolver_summary(
    out: &mut String,
    summary: &resolve::ResolveReadinessSummary,
    indent: usize,
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"resolver\": {\n");
    push_string_field(out, indent + 2, "schema", summary.schema, true);
    push_string_field(out, indent + 2, "status", summary.status, true);
    push_string_field(out, indent + 2, "mode", summary.mode, true);
    push_usize_field(out, indent + 2, "definitions", summary.definitions, true);
    push_usize_field(
        out,
        indent + 2,
        "resolved_references",
        summary.resolved_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unresolved_references",
        summary.unresolved_references,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "duplicate_definitions",
        summary.duplicate_definitions,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "mutable_place_errors",
        summary.mutable_place_errors,
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "resolver_errors",
        summary.resolver_errors,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_summary(out: &mut String, report: &TypeEnvReport, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {\n");
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
    push_usize_field(
        out,
        indent + 2,
        "resolver_definitions",
        report.resolver_definitions.len(),
        true,
    );
    push_usize_field(out, indent + 2, "type_names", report.type_names.len(), true);
    push_usize_field(
        out,
        indent + 2,
        "declarations",
        report.declarations.len(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_references",
        report.type_reference_count(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "unknown_type_references",
        report.unknown_type_references(),
        true,
    );
    push_usize_field(
        out,
        indent + 2,
        "type_env_errors",
        report.resolver_summary.resolver_errors,
        false,
    );
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_type_names(out: &mut String, indent: usize, type_names: &[TypeNameFact], comma: bool) {
    push_indent(out, indent);
    out.push_str("\"type_names\": [");
    if !type_names.is_empty() {
        out.push('\n');
        for (index, type_name) in type_names.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_type_name(out, indent + 2, type_name);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_type_name(out: &mut String, indent: usize, type_name: &TypeNameFact) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &type_name.id, true);
    push_string_field(out, indent + 2, "name", &type_name.name, true);
    push_string_field(
        out,
        indent + 2,
        "normalized_name",
        &type_name.normalized_name,
        true,
    );
    push_optional_string_field(
        out,
        indent + 2,
        "resolver_definition_id",
        type_name.resolver_definition_id.as_deref(),
        true,
    );
    push_span_field(out, indent + 2, "source_span", &type_name.source_span, true);
    push_string_field(out, indent + 2, "status", type_name.status, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_declarations(
    out: &mut String,
    indent: usize,
    declarations: &[TypeDeclaration],
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"declarations\": [");
    if !declarations.is_empty() {
        out.push('\n');
        for (index, declaration) in declarations.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_declaration(out, indent + 2, declaration);
        }
        out.push('\n');
        push_indent(out, indent);
    }
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_declaration(out: &mut String, indent: usize, declaration: &TypeDeclaration) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "id", &declaration.id, true);
    push_string_field(
        out,
        indent + 2,
        "declaration_kind",
        declaration.declaration_kind,
        true,
    );
    push_string_field(out, indent + 2, "owner_kind", declaration.owner_kind, true);
    push_string_field(out, indent + 2, "owner_name", &declaration.owner_name, true);
    push_string_field(out, indent + 2, "name", &declaration.name, true);
    push_optional_string_field(
        out,
        indent + 2,
        "resolver_definition_id",
        declaration.resolver_definition_id.as_deref(),
        true,
    );
    push_span_field(
        out,
        indent + 2,
        "source_span",
        &declaration.source_span,
        true,
    );
    push_string_field(out, indent + 2, "type_text", &declaration.type_text, true);
    push_type_references(out, indent + 2, &declaration.type_references, true);
    push_string_field(out, indent + 2, "status", declaration.status, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_type_references(
    out: &mut String,
    indent: usize,
    references: &[TypeReferenceFact],
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"type_references\": [");
    if !references.is_empty() {
        out.push('\n');
        for (index, reference) in references.iter().enumerate() {
            if index > 0 {
                out.push_str(",\n");
            }
            push_indent(out, indent + 2);
            out.push_str("{\n");
            push_string_field(out, indent + 4, "text", &reference.text, true);
            push_string_field(
                out,
                indent + 4,
                "normalized_name",
                &reference.normalized_name,
                true,
            );
            push_string_field(out, indent + 4, "role", reference.role, true);
            push_string_field(out, indent + 4, "status", reference.status, false);
            push_indent(out, indent + 2);
            out.push('}');
        }
        out.push('\n');
        push_indent(out, indent);
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

    use super::{type_env_has_errors, type_env_json, type_env_text};

    #[test]
    fn json_reports_declared_type_environment_without_type_check_claims() {
        let program = demo_program();
        let json = type_env_json(&program, &[]);

        assert!(json.contains("\"schema\": \"hum.type_env.v0\""));
        assert!(json.contains("\"mode\": \"declaration_inventory_no_type_check\""));
        assert!(json.contains("\"resolver\""));
        assert!(json.contains("\"schema\": \"hum.resolve.v0\""));
        assert!(json.contains("\"status\": \"type_environment_with_unknowns_v0\""));
        assert!(json.contains("\"resolver_errors\": 0"));
        assert!(json.contains("\"type_names\""));
        assert!(json.contains("\"name\": \"WorkItem\""));
        assert!(json.contains("\"declaration_kind\": \"field\""));
        assert!(json.contains("\"declaration_kind\": \"store\""));
        assert!(json.contains("\"declaration_kind\": \"parameter\""));
        assert!(json.contains("\"declaration_kind\": \"result\""));
        assert!(json.contains("\"text\": \"WorkError\""));
        assert!(json.contains("\"status\": \"unknown_type_name_v0\""));
        assert!(json.contains("\"no full type checking\""));
        assert!(json.contains("\"no executable semantics\""));
    }

    #[test]
    fn text_report_summarizes_unknowns_and_non_claims() {
        let program = demo_program();
        let text = type_env_text(&program, &[]);

        assert!(text.contains("Hum type environment (hum.type_env.v0)"));
        assert!(text.contains("mode: declaration_inventory_no_type_check"));
        assert!(text.contains("unknown_type_references=1"));
        assert!(text.contains("type `WorkItem`"));
        assert!(text.contains("WorkError[unknown_type_name_v0]"));
        assert!(text.contains("no full type checking"));
    }

    #[test]
    fn resolver_errors_block_type_environment_authority() {
        let source = r#"task bad names() -> UInt {
  does:
    return missing
}
"#;
        let parsed = parse_source("bad.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = type_env_json(&program, &[]);

        assert!(type_env_has_errors(&program, &[]));
        assert!(json.contains("\"status\": \"blocked_by_resolver_errors\""));
        assert!(json.contains("\"resolver_errors\": 1"));
    }

    fn demo_program() -> Program {
        let source = r#"type WorkItem {
  title: Text
  done: Bool
}

store work items: list WorkItem {
  why:
    remember work
}

task remember work item(title: Text) -> Result WorkItem, WorkError {
  changes:
    work items

  does:
    let item = WorkItem {
      title: title
      done: false
    }
    save item in work items
    return item
}
"#;
        let parsed = parse_source("types.hum", source);
        Program {
            files: vec![parsed.file],
        }
    }
}
