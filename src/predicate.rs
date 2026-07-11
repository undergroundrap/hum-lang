use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::ast::{App, Item, Program, SectionLine, Task};
use crate::core_body;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Span};
use crate::field_place::{self, FieldTypeMap};
use crate::graph::is_meaningful_line_text;
use crate::node_id;
use crate::resolve::{self, ResolveDefinitionSummary, ResolveScopeSummary};
use crate::typed_failure;

pub(crate) const MAX_DELIMITER_DEPTH: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecognitionStatus {
    NonExecutableProse,
    MalformedExecutable,
    RejectedSemantics,
    RecognizedTyped,
}

impl RecognitionStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NonExecutableProse => "non_executable_prose_v0",
            Self::MalformedExecutable => "malformed_executable_predicate_v2",
            Self::RejectedSemantics => "rejected_executable_predicate_semantics_v2",
            Self::RecognizedTyped => "recognized_typed_executable_predicate_v2",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Comparison {
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

impl Comparison {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::Less => "<",
            Self::LessEq => "<=",
            Self::Greater => ">",
            Self::GreaterEq => ">=",
        }
    }

    fn is_equality(self) -> bool {
        matches!(self, Self::Eq | Self::NotEq)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Place {
    pub(crate) root: String,
    pub(crate) field: Option<String>,
    pub(crate) range: SourceRange,
}

impl Place {
    pub(crate) fn text(&self) -> String {
        self.field.as_ref().map_or_else(
            || self.root.clone(),
            |field| format!("{}.{}", self.root, field),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Expr {
    Bool(bool, SourceRange),
    Integer(i64, SourceRange),
    Text(String, SourceRange),
    ListText(Vec<String>, SourceRange),
    Place(Place),
    Old(Place, SourceRange),
    ListLen(Place, SourceRange),
    ListCount(Box<Expr>, Box<Expr>, SourceRange),
    Binary(Box<Expr>, Arithmetic, Box<Expr>, SourceRange),
    Group(Box<Expr>, SourceRange),
}

impl Expr {
    pub(crate) fn range(&self) -> SourceRange {
        match self {
            Self::Bool(_, range)
            | Self::Integer(_, range)
            | Self::Text(_, range)
            | Self::ListText(_, range)
            | Self::Old(_, range)
            | Self::ListLen(_, range)
            | Self::ListCount(_, _, range)
            | Self::Binary(_, _, _, range)
            | Self::Group(_, range) => *range,
            Self::Place(place) => place.range,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PredicateAst {
    pub(crate) left: Expr,
    pub(crate) comparison: Comparison,
    pub(crate) operator_range: SourceRange,
    pub(crate) right: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PredicateFact {
    pub(crate) task: String,
    pub(crate) task_span: Span,
    pub(crate) section: String,
    pub(crate) text: String,
    pub(crate) line_span: Span,
    pub(crate) status: RecognitionStatus,
    pub(crate) intent_span: Option<Span>,
    pub(crate) offending_span: Option<Span>,
    pub(crate) reason: &'static str,
    pub(crate) expected: Option<String>,
    pub(crate) actual: Option<String>,
    pub(crate) places: Vec<PredicatePlaceFact>,
    pub(crate) comparison: Option<&'static str>,
    pub(crate) left_type: Option<String>,
    pub(crate) right_type: Option<String>,
    pub(crate) delimiter_depth: usize,
    pub(crate) ast: Option<PredicateAst>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PredicatePlaceFact {
    pub(crate) text: String,
    pub(crate) span: Span,
    pub(crate) source_range: SourceRange,
    pub(crate) scope_id: String,
    pub(crate) root_definition_id: Option<String>,
    pub(crate) definition_id: Option<String>,
    pub(crate) resolution: &'static str,
    pub(crate) eligibility: &'static str,
    pub(crate) type_text: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PredicateAnalysis {
    facts: Vec<PredicateFact>,
}

impl PredicateAnalysis {
    pub(crate) fn build(program: &Program) -> Self {
        let fields = field_place::collect_field_types(program);
        let context = LexicalContext::build(program);
        let mut facts = Vec::new();
        for file in &program.files {
            collect_facts(&file.items, &fields, &context, &mut facts);
        }
        Self { facts }
    }

    pub(crate) fn place_facts_text(&self) -> String {
        let places = self
            .facts
            .iter()
            .flat_map(|fact| fact.places.iter().map(move |place| (fact, place)));
        let mut out = String::from("predicate_place_facts:\n");
        let mut count = 0usize;
        for (fact, place) in places {
            count += 1;
            out.push_str(&format!(
                "  {}:{}:{} task=`{}` section={} text=`{}` scope={} root_definition={} definition={} resolution={} eligibility={} type={}\n",
                place.span.file.replace('\\', "/"),
                place.span.line,
                place.span.column,
                fact.task,
                fact.section,
                place.text,
                place.scope_id,
                place.root_definition_id.as_deref().unwrap_or("none"),
                place.definition_id.as_deref().unwrap_or("none"),
                place.resolution,
                place.eligibility,
                place.type_text.as_deref().unwrap_or("unknown")
            ));
        }
        if count == 0 {
            out.push_str("  none\n");
        }
        out
    }

    pub(crate) fn place_facts_json(&self) -> String {
        fn escaped(value: &str) -> String {
            value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t")
        }
        let mut rows = Vec::new();
        for fact in &self.facts {
            for place in &fact.places {
                rows.push(format!(
                    "{{\"task\": \"{}\", \"section\": \"{}\", \"text\": \"{}\", \"span\": {{\"file\": \"{}\", \"line\": {}, \"column\": {}}}, \"scope_id\": \"{}\", \"root_definition_id\": \"{}\", \"definition_id\": \"{}\", \"resolution\": \"{}\", \"eligibility\": \"{}\", \"type\": \"{}\"}}",
                    escaped(&fact.task),
                    escaped(&fact.section),
                    escaped(&place.text),
                    escaped(&place.span.file.replace('\\', "/")),
                    place.span.line,
                    place.span.column,
                    escaped(&place.scope_id),
                    escaped(place.root_definition_id.as_deref().unwrap_or("none")),
                    escaped(place.definition_id.as_deref().unwrap_or("none")),
                    place.resolution,
                    place.eligibility,
                    escaped(place.type_text.as_deref().unwrap_or("unknown"))
                ));
            }
        }
        format!("[{}]", rows.join(", "))
    }

    pub(crate) fn facts(&self) -> &[PredicateFact] {
        &self.facts
    }

    pub(crate) fn facts_for_task<'a>(
        &'a self,
        task: &Task,
    ) -> impl Iterator<Item = &'a PredicateFact> {
        let span = task.span.clone();
        self.facts.iter().filter(move |fact| fact.task_span == span)
    }

    pub(crate) fn fact_for_line(
        &self,
        task: &Task,
        section: &str,
        line: &SectionLine,
    ) -> Option<&PredicateFact> {
        self.facts.iter().find(|fact| {
            fact.task_span == task.span && fact.section == section && fact.line_span == line.span
        })
    }

    pub(crate) fn reachable_diagnostics(
        &self,
        program: &Program,
        entry: &Task,
        active_app: Option<&App>,
    ) -> Vec<Diagnostic> {
        let mut pending = vec![entry];
        let mut visited = BTreeSet::new();
        let mut diagnostics = Vec::new();
        while let Some(task) = pending.pop() {
            let identity = (
                task.span.file.replace('\\', "/"),
                task.span.line,
                task.span.column,
            );
            if !visited.insert(identity) {
                continue;
            }
            diagnostics.extend(
                self.facts_for_task(task)
                    .filter_map(PredicateFact::diagnostic),
            );
            let Some(does) = task.section("does") else {
                continue;
            };
            let body = core_body::analyze_does_section(does);
            for statement in &body.statements {
                let Some(expression) = typed_failure::statement_expression(statement) else {
                    continue;
                };
                for call in typed_failure::calls_in_expression(expression) {
                    if let Some(callee) = find_task(program, active_app, &call.callee) {
                        pending.push(callee);
                    }
                }
            }
        }
        diagnostics
    }
}

fn find_task<'a>(
    program: &'a Program,
    active_app: Option<&'a App>,
    name: &str,
) -> Option<&'a Task> {
    fn in_items<'a>(items: &'a [Item], name: &str) -> Option<&'a Task> {
        items.iter().find_map(|item| match item {
            Item::Task(task) if task.name == name => Some(task),
            Item::App(app) => in_items(&app.items, name),
            Item::Task(_) | Item::Type(_) | Item::Store(_) | Item::Test(_) => None,
        })
    }
    active_app.map_or_else(
        || {
            program
                .files
                .iter()
                .find_map(|file| in_items(&file.items, name))
        },
        |app| in_items(&app.items, name),
    )
}

impl PredicateFact {
    pub(crate) fn repair(&self) -> String {
        repair_for(self.reason)
    }

    pub(crate) fn diagnostic(&self) -> Option<Diagnostic> {
        if !matches!(
            self.status,
            RecognitionStatus::MalformedExecutable | RecognitionStatus::RejectedSemantics
        ) || self.reason == "opaque_path_inspection_owned_by_h0630"
        {
            return None;
        }
        let expected = self
            .expected
            .as_deref()
            .unwrap_or("one complete typed Predicate v2 comparison");
        let actual = self.actual.as_deref().unwrap_or("invalid predicate shape");
        let mut diagnostic = Diagnostic::error(
            DiagnosticCode::INVALID_EXECUTABLE_PREDICATE,
            format!(
                "task `{}` has invalid executable {} predicate `{}`: {} (expected {expected}; actual {actual}; status={})",
                self.task,
                self.section,
                self.text,
                self.reason,
                self.status.as_str()
            ),
            self.offending_span.clone().or_else(|| Some(self.line_span.clone())),
        )
        .with_related_span(format!("{} contract", self.section), self.line_span.clone())
        .with_help(self.repair());
        if let Some(span) = &self.intent_span {
            diagnostic = diagnostic.with_related_span("executable predicate intent", span.clone());
        }
        Some(diagnostic)
    }

    pub(crate) fn blocks(&self) -> bool {
        self.diagnostic().is_some()
    }
}

thread_local! {
    static ANALYSIS_CACHE: RefCell<Option<(String, Arc<PredicateAnalysis>)>> = const { RefCell::new(None) };
}

pub(crate) fn analyze_program(program: &Program) -> Arc<PredicateAnalysis> {
    let key = format!("{program:?}");
    ANALYSIS_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some((cached_key, analysis)) = cache.as_ref()
            && cached_key == &key
        {
            return Arc::clone(analysis);
        }
        let analysis = Arc::new(PredicateAnalysis::build(program));
        *cache = Some((key, Arc::clone(&analysis)));
        analysis
    })
}

pub(crate) fn fact_for_line(
    program: &Program,
    task: &Task,
    section: &str,
    line: &SectionLine,
) -> Option<PredicateFact> {
    if !matches!(section, "needs" | "ensures") || !is_meaningful_line_text(&line.text) {
        return None;
    }
    analyze_program(program)
        .fact_for_line(task, section, line)
        .cloned()
}

fn collect_facts(
    items: &[Item],
    fields: &FieldTypeMap,
    context: &LexicalContext,
    out: &mut Vec<PredicateFact>,
) {
    for item in items {
        match item {
            Item::Task(task) => out.extend(analyze_task_with_context(task, fields, context)),
            Item::App(app) => collect_facts(&app.items, fields, context, out),
            Item::Type(_) | Item::Store(_) | Item::Test(_) => {}
        }
    }
}

fn analyze_task_with_context(
    task: &Task,
    fields: &FieldTypeMap,
    context: &LexicalContext,
) -> Vec<PredicateFact> {
    let mut facts = Vec::new();
    for section_name in ["needs", "ensures"] {
        if let Some(section) = task.section(section_name) {
            for line in &section.lines {
                if is_meaningful_line_text(&line.text) {
                    facts.push(analyze_line(task, section_name, line, fields, context));
                }
            }
        }
    }
    facts
}

fn analyze_line(
    task: &Task,
    section: &str,
    line: &SectionLine,
    fields: &FieldTypeMap,
    context: &LexicalContext,
) -> PredicateFact {
    let text = line.text.trim().to_string();
    let intent = find_intent_signal(&text);
    let Some(intent_range) = intent else {
        return base_fact(
            task,
            section,
            line,
            text,
            RecognitionStatus::NonExecutableProse,
            None,
            "no_executable_predicate_intent_v0",
        );
    };
    let mut parser = Parser::new(&text);
    let parsed = parser.parse_predicate();
    let delimiter_depth = parser.max_depth;
    let ast = match parsed {
        Ok(ast) => ast,
        Err(error) => {
            let mut fact = base_fact(
                task,
                section,
                line,
                text,
                RecognitionStatus::MalformedExecutable,
                Some(intent_range),
                error.reason,
            );
            fact.offending_span = Some(range_span(line, error.range));
            fact.expected = Some(error.expected.to_string());
            fact.actual = Some(error.actual);
            fact.delimiter_depth = delimiter_depth;
            return fact;
        }
    };

    let mut fact = base_fact(
        task,
        section,
        line,
        text,
        RecognitionStatus::RecognizedTyped,
        Some(intent_range),
        "typed_predicate_v2",
    );
    fact.delimiter_depth = delimiter_depth;
    fact.comparison = Some(ast.comparison.as_str());
    fact.places = collect_place_facts(&ast, task, section, line, fields, context);
    match type_predicate(&ast, section, &fact.places) {
        Ok((left, right)) => {
            fact.left_type = Some(left.display());
            fact.right_type = Some(right.display());
        }
        Err(issue) => {
            fact.status = RecognitionStatus::RejectedSemantics;
            fact.reason = issue.reason;
            fact.offending_span = Some(range_span(line, issue.range));
            fact.expected = Some(issue.expected);
            fact.actual = Some(issue.actual);
            fact.ast = Some(ast);
            return fact;
        }
    }
    fact.ast = Some(ast);
    fact
}

fn base_fact(
    task: &Task,
    section: &str,
    line: &SectionLine,
    text: String,
    status: RecognitionStatus,
    intent: Option<SourceRange>,
    reason: &'static str,
) -> PredicateFact {
    PredicateFact {
        task: task.name.clone(),
        task_span: task.span.clone(),
        section: section.to_string(),
        text,
        line_span: line.span.clone(),
        status,
        intent_span: intent.map(|range| range_span(line, range)),
        offending_span: None,
        reason,
        expected: None,
        actual: None,
        places: Vec::new(),
        comparison: None,
        left_type: None,
        right_type: None,
        delimiter_depth: 0,
        ast: None,
    }
}

struct LexicalContext {
    scopes: Vec<ResolveScopeSummary>,
    definitions: Vec<ResolveDefinitionSummary>,
    scope_parents: BTreeMap<String, Option<String>>,
}

impl LexicalContext {
    fn build(program: &Program) -> Self {
        let scopes = resolve::resolve_scope_summaries(program, &[]);
        let definitions = resolve::resolve_definition_summaries(program, &[]);
        let scope_parents = scopes
            .iter()
            .map(|scope| (scope.id.clone(), scope.parent_scope_id.clone()))
            .collect();
        Self {
            scopes,
            definitions,
            scope_parents,
        }
    }

    fn task_scope(&self, task: &Task) -> Option<&ResolveScopeSummary> {
        self.scopes.iter().find(|scope| {
            scope.scope_kind == "callable"
                && scope.owner_kind == "task"
                && scope.owner_name == task.name
                && scope
                    .source_span
                    .as_ref()
                    .is_some_and(|span| same_span(span, &task.span))
        })
    }

    fn lexical_definition(&self, scope_id: &str, name: &str) -> Option<&ResolveDefinitionSummary> {
        let mut cursor = Some(scope_id.to_string());
        while let Some(scope) = cursor {
            if let Some(definition) = self.definitions.iter().find(|definition| {
                definition.scope_id == scope
                    && definition.normalized_name == name.to_ascii_lowercase()
            }) {
                return Some(definition);
            }
            cursor = self.scope_parents.get(&scope).cloned().flatten();
        }
        None
    }
}

fn same_span(left: &Span, right: &Span) -> bool {
    node_id::source_path_identity(&left.file) == node_id::source_path_identity(&right.file)
        && left.line == right.line
        && left.column == right.column
}

fn collect_place_facts(
    ast: &PredicateAst,
    task: &Task,
    section: &str,
    line: &SectionLine,
    fields: &FieldTypeMap,
    context: &LexicalContext,
) -> Vec<PredicatePlaceFact> {
    fn collect(
        expr: &Expr,
        task: &Task,
        section: &str,
        line: &SectionLine,
        fields: &FieldTypeMap,
        context: &LexicalContext,
        out: &mut Vec<PredicatePlaceFact>,
    ) {
        match expr {
            Expr::Place(place) | Expr::ListLen(place, _) => {
                out.push(resolve_place(
                    place, task, section, line, fields, context, false,
                ));
            }
            Expr::Old(place, _) => {
                out.push(resolve_place(
                    place, task, section, line, fields, context, true,
                ));
            }
            Expr::ListCount(left, right, _) | Expr::Binary(left, _, right, _) => {
                collect(left, task, section, line, fields, context, out);
                collect(right, task, section, line, fields, context, out);
            }
            Expr::Group(inner, _) => collect(inner, task, section, line, fields, context, out),
            Expr::Bool(_, _) | Expr::Integer(_, _) | Expr::Text(_, _) | Expr::ListText(_, _) => {}
        }
    }
    let mut facts = Vec::new();
    collect(&ast.left, task, section, line, fields, context, &mut facts);
    collect(&ast.right, task, section, line, fields, context, &mut facts);
    facts
}

fn resolve_place(
    place: &Place,
    task: &Task,
    section: &str,
    line: &SectionLine,
    fields: &FieldTypeMap,
    context: &LexicalContext,
    old_context: bool,
) -> PredicatePlaceFact {
    let scope_id = context.task_scope(task).map_or_else(
        || "missing_callable_scope_v0".to_string(),
        |scope| scope.id.clone(),
    );
    let span = range_span(line, place.range);
    let mut resolution = "unresolved_v0";
    let mut eligibility = "ineligible_v0";
    let mut root_definition_id = None;
    let mut definition_id = None;
    let mut type_text = None;

    if place.root == "result" && section == "ensures" && !old_context {
        resolution = "resolved_v0";
        eligibility = "eligible_v0";
        let id = node_id::span("predicate-result", &task.span, &task.name);
        root_definition_id = Some(id.clone());
        definition_id = Some(id);
        type_text = Some(
            task.result
                .as_deref()
                .map(result_value_type)
                .unwrap_or("Unit")
                .to_string(),
        );
    } else if let Some(definition) = context.lexical_definition(&scope_id, &place.root) {
        resolution = "resolved_v0";
        root_definition_id = Some(definition.id.clone());
        definition_id = Some(definition.id.clone());
        if definition.definition_kind == "parameter" {
            eligibility = if old_context && section != "ensures" {
                "ineligible_v0"
            } else {
                "eligible_v0"
            };
            type_text = task
                .params
                .iter()
                .find(|param| param.name == place.root)
                .map(|param| param.ty.clone());
        }
    }

    if let Some(field) = &place.field
        && resolution == "resolved_v0"
        && eligibility == "eligible_v0"
    {
        let root_type = type_text.clone();
        let field_definition = root_type.as_deref().and_then(|root_type| {
            let type_scope = context
                .scopes
                .iter()
                .find(|scope| scope.scope_kind == "type" && scope.owner_name == root_type)?;
            context.definitions.iter().find(|definition| {
                definition.scope_id == type_scope.id
                    && definition.definition_kind == "field"
                    && definition.normalized_name == field.to_ascii_lowercase()
            })
        });
        if let Some(field_definition) = field_definition {
            definition_id = Some(field_definition.id.clone());
            type_text = root_type
                .as_deref()
                .and_then(|root_type| field_place::field_type(fields, root_type, field))
                .map(str::to_string);
        } else {
            resolution = "unresolved_v0";
            definition_id = None;
            type_text = None;
        }
    }

    PredicatePlaceFact {
        text: place.text(),
        span,
        source_range: place.range,
        scope_id,
        root_definition_id,
        definition_id,
        resolution,
        eligibility,
        type_text,
    }
}

fn range_span(line: &SectionLine, range: SourceRange) -> Span {
    let prefix = &line.text[..range.start.min(line.text.len())];
    Span::new(
        line.span.file.clone(),
        line.span.line,
        line.span.column + prefix.chars().count(),
    )
}

fn find_intent_signal(text: &str) -> Option<SourceRange> {
    let shielded = quoted_ranges(text);
    let is_shielded = |index: usize| {
        shielded
            .iter()
            .any(|range| range.start <= index && index < range.end)
    };
    let bytes = text.as_bytes();
    for (index, byte) in bytes.iter().copied().enumerate() {
        if is_shielded(index) {
            continue;
        }
        if matches!(byte, b'=' | b'<' | b'>') {
            return Some(SourceRange {
                start: index,
                end: index + 1,
            });
        }
        if byte == b'!' {
            let mut next = index + 1;
            while next < bytes.len() && matches!(bytes[next], b' ' | b'\t') {
                next += 1;
            }
            if bytes.get(next) == Some(&b'=')
                || (operand_ends_before(text, index) && operand_starts_at(text, next))
            {
                return Some(SourceRange {
                    start: index,
                    end: index + 1,
                });
            }
        }
    }
    for name in ["old", "list_len", "list_count"] {
        let mut start = 0;
        while let Some(offset) = text[start..].find(name) {
            let index = start + offset;
            let boundary_before = index == 0 || !is_ident_byte(bytes[index - 1]);
            let after_name = index + name.len();
            let boundary_after = after_name == bytes.len() || !is_ident_byte(bytes[after_name]);
            if boundary_before && boundary_after && !is_shielded(index) {
                let mut open = after_name;
                while open < bytes.len() && matches!(bytes[open], b' ' | b'\t') {
                    open += 1;
                }
                if bytes.get(open) == Some(&b'(') {
                    return Some(SourceRange {
                        start: index,
                        end: open + 1,
                    });
                }
            }
            start = after_name;
        }
    }
    None
}

fn quoted_ranges(text: &str) -> Vec<SourceRange> {
    let mut ranges = Vec::new();
    let mut open = None;
    for (index, ch) in text.char_indices() {
        if ch == '"' {
            if let Some(start) = open.take() {
                ranges.push(SourceRange {
                    start,
                    end: index + 1,
                });
            } else {
                open = Some(index);
            }
        }
    }
    ranges
}

fn operand_ends_before(text: &str, index: usize) -> bool {
    let left = text[..index].trim_end();
    left.chars().last().is_some_and(|ch| {
        ch == ')' || ch == ']' || ch == '"' || ch.is_ascii_alphanumeric() || ch == '_'
    })
}

fn operand_starts_at(text: &str, index: usize) -> bool {
    let rest = &text[index..];
    rest.starts_with('"')
        || rest.starts_with('[')
        || rest.starts_with('(')
        || rest.starts_with("true")
        || rest.starts_with("false")
        || rest.chars().next().is_some_and(|ch| {
            ch.is_ascii_digit() || ch == '-' || ch.is_ascii_lowercase() || ch == '_'
        })
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

struct ParseError {
    reason: &'static str,
    range: SourceRange,
    expected: &'static str,
    actual: String,
}

struct Parser<'a> {
    text: &'a str,
    pos: usize,
    depth: usize,
    max_depth: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            pos: 0,
            depth: 0,
            max_depth: 0,
        }
    }

    fn parse_predicate(&mut self) -> Result<PredicateAst, ParseError> {
        self.hws();
        let left = self.parse_additive()?;
        self.hws();
        let (comparison, operator_range) = self.parse_comparison()?;
        self.hws();
        let right = self.parse_additive()?;
        self.hws();
        if self.pos != self.text.len() {
            return Err(self.error("trailing_tokens_after_predicate_v2", "end of contract line"));
        }
        Ok(PredicateAst {
            left,
            comparison,
            operator_range,
            right,
        })
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            self.hws();
            let Some(op) = self.peek_char() else {
                break;
            };
            let arithmetic = match op {
                '+' => Arithmetic::Add,
                '-' => Arithmetic::Subtract,
                _ => break,
            };
            self.bump_char();
            self.hws();
            let right = self.parse_multiplicative()?;
            let range = SourceRange {
                start: left.range().start,
                end: right.range().end,
            };
            left = Expr::Binary(Box::new(left), arithmetic, Box::new(right), range);
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;
        loop {
            self.hws();
            let Some(op) = self.peek_char() else {
                break;
            };
            let arithmetic = match op {
                '*' => Arithmetic::Multiply,
                '/' => Arithmetic::Divide,
                _ => break,
            };
            self.bump_char();
            self.hws();
            let right = self.parse_primary()?;
            let range = SourceRange {
                start: left.range().start,
                end: right.range().end,
            };
            left = Expr::Binary(Box::new(left), arithmetic, Box::new(right), range);
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.hws();
        let start = self.pos;
        let Some(ch) = self.peek_char() else {
            return Err(self.error("missing_operand_v2", "operand"));
        };
        if ch == '(' {
            self.open_delimiter()?;
            self.hws();
            let inner = self.parse_additive()?;
            self.hws();
            self.expect_close(')')?;
            return Ok(Expr::Group(
                Box::new(inner),
                SourceRange {
                    start,
                    end: self.pos,
                },
            ));
        }
        if ch == '[' {
            return self.parse_list();
        }
        if ch == '"' {
            let (value, range) = self.parse_text()?;
            return Ok(Expr::Text(value, range));
        }
        if ch.is_ascii_digit()
            || (ch == '-'
                && self
                    .peek_after_char()
                    .is_some_and(|next| next.is_ascii_digit()))
        {
            return self.parse_integer();
        }
        if is_ident_start(ch) {
            let name = self.parse_identifier();
            let range = SourceRange {
                start,
                end: self.pos,
            };
            if name == "true" {
                return Ok(Expr::Bool(true, range));
            }
            if name == "false" {
                return Ok(Expr::Bool(false, range));
            }
            if self.peek_char() == Some('(') {
                return match name.as_str() {
                    "old" => self.parse_place_call(start, true),
                    "list_len" => self.parse_place_call(start, false),
                    "list_count" => self.parse_list_count(start),
                    _ => Err(ParseError {
                        reason: "arbitrary_helper_call_not_allowed_v2",
                        range,
                        expected: "a Predicate v2 operand or exact old/list_len/list_count call",
                        actual: format!("call `{name}(...)`"),
                    }),
                };
            }
            if self
                .peek_char()
                .is_some_and(|next| next == ' ' || next == '\t')
            {
                let saved = self.pos;
                self.hws();
                if self.peek_char() == Some('(')
                    && matches!(name.as_str(), "old" | "list_len" | "list_count")
                {
                    return Err(ParseError {
                        reason: "known_call_requires_no_gap_v2",
                        range: SourceRange {
                            start,
                            end: self.pos + 1,
                        },
                        expected: "call name immediately followed by `(`",
                        actual: format!("`{}` followed by horizontal whitespace", name),
                    });
                }
                self.pos = saved;
            }
            let mut place = Place {
                root: name,
                field: None,
                range,
            };
            if self.peek_char() == Some('.') {
                self.bump_char();
                if !self.peek_char().is_some_and(is_ident_start) {
                    return Err(
                        self.error("malformed_syntactic_place_v2", "one identifier after `.`")
                    );
                }
                let field = self.parse_identifier();
                place.field = Some(field);
                place.range.end = self.pos;
                if self.peek_char() == Some('.') {
                    return Err(
                        self.error("malformed_syntactic_place_v2", "at most one direct field")
                    );
                }
            }
            return Ok(Expr::Place(place));
        }
        Err(self.error(
            "invalid_operand_starter_v2",
            "boolean, integer, place, Text, List Text, known call, or parenthesized operand",
        ))
    }

    fn parse_place_call(&mut self, start: usize, old: bool) -> Result<Expr, ParseError> {
        self.open_delimiter()?;
        self.hws();
        let place = self.parse_place_only()?;
        self.hws();
        self.expect_close(')')?;
        let range = SourceRange {
            start,
            end: self.pos,
        };
        Ok(if old {
            Expr::Old(place, range)
        } else {
            Expr::ListLen(place, range)
        })
    }

    fn parse_list_count(&mut self, start: usize) -> Result<Expr, ParseError> {
        self.open_delimiter()?;
        self.hws();
        let list = if self.peek_char() == Some('[') {
            self.parse_list()?
        } else {
            Expr::Place(self.parse_place_only()?)
        };
        self.hws();
        if self.peek_char() != Some(',') {
            return Err(self.error("list_count_wrong_arity_v2", "`,` and a Text source"));
        }
        self.bump_char();
        self.hws();
        let text = if self.peek_char() == Some('"') {
            let (value, range) = self.parse_text()?;
            Expr::Text(value, range)
        } else {
            Expr::Place(self.parse_place_only()?)
        };
        self.hws();
        self.expect_close(')')?;
        Ok(Expr::ListCount(
            Box::new(list),
            Box::new(text),
            SourceRange {
                start,
                end: self.pos,
            },
        ))
    }

    fn parse_place_only(&mut self) -> Result<Place, ParseError> {
        let start = self.pos;
        if !self.peek_char().is_some_and(is_ident_start) {
            return Err(self.error(
                "malformed_syntactic_place_v2",
                "identifier or direct-field place",
            ));
        }
        let root = self.parse_identifier();
        let mut place = Place {
            root,
            field: None,
            range: SourceRange {
                start,
                end: self.pos,
            },
        };
        if self.peek_char() == Some('.') {
            self.bump_char();
            if !self.peek_char().is_some_and(is_ident_start) {
                return Err(self.error("malformed_syntactic_place_v2", "field identifier"));
            }
            place.field = Some(self.parse_identifier());
            place.range.end = self.pos;
        }
        if self.peek_char() == Some('.') {
            return Err(self.error("malformed_syntactic_place_v2", "at most one direct field"));
        }
        Ok(place)
    }

    fn parse_list(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        self.open_delimiter()?;
        self.hws();
        let mut values = Vec::new();
        if self.peek_char() == Some(']') {
            self.expect_close(']')?;
            return Ok(Expr::ListText(
                values,
                SourceRange {
                    start,
                    end: self.pos,
                },
            ));
        }
        loop {
            if self.peek_char() != Some('"') {
                return Err(self.error(
                    "list_text_literal_requires_text_elements_v2",
                    "Text literal list element",
                ));
            }
            values.push(self.parse_text()?.0);
            self.hws();
            match self.peek_char() {
                Some(']') => {
                    self.expect_close(']')?;
                    break;
                }
                Some(',') => {
                    self.bump_char();
                    self.hws();
                    if self.peek_char() == Some(']') {
                        return Err(self.error(
                            "list_text_literal_trailing_comma_v2",
                            "Text literal after `,`",
                        ));
                    }
                }
                _ => return Err(self.error("list_text_literal_separator_v2", "`,` or `]`")),
            }
        }
        Ok(Expr::ListText(
            values,
            SourceRange {
                start,
                end: self.pos,
            },
        ))
    }

    fn parse_text(&mut self) -> Result<(String, SourceRange), ParseError> {
        let start = self.pos;
        self.bump_char();
        let content_start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                let value = self.text[content_start..self.pos].to_string();
                self.bump_char();
                return Ok((
                    value,
                    SourceRange {
                        start,
                        end: self.pos,
                    },
                ));
            }
            self.bump_char();
        }
        Err(ParseError {
            reason: "unterminated_text_literal_v2",
            range: SourceRange {
                start,
                end: self.text.len(),
            },
            expected: "closing `\"`",
            actual: "end of line".to_string(),
        })
    }

    fn parse_integer(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        if self.peek_char() == Some('-') {
            self.bump_char();
        }
        while self.peek_char().is_some_and(|ch| ch.is_ascii_digit()) {
            self.bump_char();
        }
        let raw = &self.text[start..self.pos];
        let value = raw.parse::<i64>().map_err(|_| ParseError {
            reason: "integer_literal_out_of_range_v2",
            range: SourceRange {
                start,
                end: self.pos,
            },
            expected: "existing Int literal",
            actual: raw.to_string(),
        })?;
        Ok(Expr::Integer(
            value,
            SourceRange {
                start,
                end: self.pos,
            },
        ))
    }

    fn parse_comparison(&mut self) -> Result<(Comparison, SourceRange), ParseError> {
        let start = self.pos;
        for (token, comparison) in [
            ("==", Comparison::Eq),
            ("!=", Comparison::NotEq),
            ("<=", Comparison::LessEq),
            (">=", Comparison::GreaterEq),
            ("<", Comparison::Less),
            (">", Comparison::Greater),
        ] {
            if self.text[self.pos..].starts_with(token) {
                self.pos += token.len();
                return Ok((
                    comparison,
                    SourceRange {
                        start,
                        end: self.pos,
                    },
                ));
            }
        }
        Err(self.error(
            "invalid_comparison_operator_v2",
            "one atomic comparison operator",
        ))
    }

    fn open_delimiter(&mut self) -> Result<(), ParseError> {
        self.bump_char();
        self.depth += 1;
        self.max_depth = self.max_depth.max(self.depth);
        if self.depth > MAX_DELIMITER_DEPTH {
            return Err(self.error(
                "delimiter_depth_exceeded_v2",
                "at most 16 open delimiter frames",
            ));
        }
        Ok(())
    }

    fn expect_close(&mut self, expected: char) -> Result<(), ParseError> {
        if self.peek_char() != Some(expected) {
            return Err(self.error(
                "mismatched_or_missing_delimiter_v2",
                "matching closing delimiter",
            ));
        }
        self.bump_char();
        self.depth = self.depth.saturating_sub(1);
        Ok(())
    }

    fn parse_identifier(&mut self) -> String {
        let start = self.pos;
        self.bump_char();
        while self.peek_char().is_some_and(is_ident_continue) {
            self.bump_char();
        }
        self.text[start..self.pos].to_string()
    }

    fn hws(&mut self) {
        while self.peek_char().is_some_and(|ch| ch == ' ' || ch == '\t') {
            self.bump_char();
        }
    }
    fn peek_char(&self) -> Option<char> {
        self.text[self.pos..].chars().next()
    }
    fn peek_after_char(&self) -> Option<char> {
        let mut chars = self.text[self.pos..].chars();
        chars.next()?;
        chars.next()
    }
    fn bump_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.pos += ch.len_utf8();
        }
    }
    fn error(&self, reason: &'static str, expected: &'static str) -> ParseError {
        let end = self
            .peek_char()
            .map_or(self.pos, |ch| self.pos + ch.len_utf8());
        ParseError {
            reason,
            range: SourceRange {
                start: self.pos,
                end,
            },
            expected,
            actual: self
                .peek_char()
                .map_or_else(|| "end of line".to_string(), |ch| format!("`{ch}`")),
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch == '_'
}
fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OperandType {
    Bool,
    Int,
    UInt,
    IntegerLiteral,
    Text,
    ListText,
    Path,
    Other(String),
}

impl OperandType {
    fn display(&self) -> String {
        match self {
            Self::Bool => "Bool".into(),
            Self::Int => "Int".into(),
            Self::UInt => "UInt".into(),
            Self::IntegerLiteral => "integer literal".into(),
            Self::Text => "Text".into(),
            Self::ListText => "List Text".into(),
            Self::Path => "Path".into(),
            Self::Other(value) => value.clone(),
        }
    }
    fn numeric(&self) -> bool {
        matches!(self, Self::Int | Self::UInt | Self::IntegerLiteral)
    }
}

struct TypeIssue {
    reason: &'static str,
    range: SourceRange,
    expected: String,
    actual: String,
}

fn type_predicate(
    ast: &PredicateAst,
    section: &str,
    places: &[PredicatePlaceFact],
) -> Result<(OperandType, OperandType), Box<TypeIssue>> {
    let left = type_expr(&ast.left, section, places)?;
    let right = type_expr(&ast.right, section, places)?;
    if matches!(left, OperandType::Path) || matches!(right, OperandType::Path) {
        return Err(type_issue(
            "opaque_path_inspection_owned_by_h0630",
            if matches!(left, OperandType::Path) {
                ast.left.range()
            } else {
                ast.right.range()
            },
            "opaque Path is not predicate-readable",
            format!(
                "{} {} {}",
                left.display(),
                ast.comparison.as_str(),
                right.display()
            ),
        ));
    }
    let compatible = if left.numeric() && right.numeric() {
        true
    } else {
        left == right
    };
    if !compatible {
        return Err(type_issue(
            "cross_type_comparison_v2",
            ast.right.range(),
            left.display(),
            right.display(),
        ));
    }
    if left == OperandType::ListText
        && !is_list_text_literal(&ast.left)
        && !is_list_text_literal(&ast.right)
    {
        return Err(type_issue(
            "list_text_comparison_requires_literal_v2",
            ast.operator_range,
            "a List Text place compared with a List Text literal",
            "two non-literal List Text operands",
        ));
    }
    if !ast.comparison.is_equality()
        && matches!(
            left,
            OperandType::Text | OperandType::ListText | OperandType::Bool
        )
    {
        return Err(type_issue(
            "operator_not_supported_for_operand_type_v2",
            ast.operator_range,
            format!("== or != for {}", left.display()),
            ast.comparison.as_str().to_string(),
        ));
    }
    Ok((left, right))
}

fn is_list_text_literal(expr: &Expr) -> bool {
    match expr {
        Expr::ListText(_, _) => true,
        Expr::Group(inner, _) => is_list_text_literal(inner),
        _ => false,
    }
}

fn type_expr(
    expr: &Expr,
    section: &str,
    places: &[PredicatePlaceFact],
) -> Result<OperandType, Box<TypeIssue>> {
    match expr {
        Expr::Bool(_, _) => Ok(OperandType::Bool),
        Expr::Integer(_, _) => Ok(OperandType::IntegerLiteral),
        Expr::Text(_, _) => Ok(OperandType::Text),
        Expr::ListText(_, _) => Ok(OperandType::ListText),
        Expr::Place(place) => type_place(place, places),
        Expr::Old(place, range) => {
            if section != "ensures" || place.root == "result" {
                return Err(type_issue(
                    "old_place_not_entry_readable_v2",
                    *range,
                    "parameter or parameter field in ensures",
                    "section-ineligible old place",
                ));
            }
            type_place(place, places).map_err(|mut issue| {
                issue.range = *range;
                issue
            })
        }
        Expr::ListLen(place, range) => {
            let ty = type_place(place, places)?;
            let is_list = matches!(ty, OperandType::ListText)
                || matches!(&ty, OperandType::Other(value) if value.starts_with("List "));
            if !is_list {
                return Err(type_issue(
                    "list_len_requires_list_v2",
                    *range,
                    "List value",
                    ty.display(),
                ));
            }
            Ok(OperandType::UInt)
        }
        Expr::ListCount(list, text, range) => {
            let list_ty = type_expr(list, section, places)?;
            if list_ty != OperandType::ListText {
                return Err(type_issue(
                    "list_count_requires_list_text_v2",
                    list.range(),
                    "List Text",
                    list_ty.display(),
                ));
            }
            let text_ty = type_expr(text, section, places)?;
            if text_ty != OperandType::Text {
                return Err(type_issue(
                    "list_count_requires_text_match_v2",
                    text.range(),
                    "Text",
                    text_ty.display(),
                ));
            }
            let _ = range;
            Ok(OperandType::UInt)
        }
        Expr::Binary(left, _, right, range) => {
            let left_ty = type_expr(left, section, places)?;
            let right_ty = type_expr(right, section, places)?;
            if !left_ty.numeric() || !right_ty.numeric() {
                return Err(type_issue(
                    "arithmetic_requires_numeric_operands_v2",
                    *range,
                    "Int/UInt operands",
                    format!("{} and {}", left_ty.display(), right_ty.display()),
                ));
            }
            if matches!(left_ty, OperandType::UInt) || matches!(right_ty, OperandType::UInt) {
                Ok(OperandType::UInt)
            } else {
                Ok(OperandType::Int)
            }
        }
        Expr::Group(inner, _) => type_expr(inner, section, places),
    }
}

fn type_place(place: &Place, places: &[PredicatePlaceFact]) -> Result<OperandType, Box<TypeIssue>> {
    let fact = places
        .iter()
        .find(|fact| fact.source_range == place.range)
        .ok_or_else(|| {
            type_issue(
                "predicate_place_unresolved_v2",
                place.range,
                "resolved task parameter or ensures result place",
                "unresolved place",
            )
        })?;
    if fact.resolution != "resolved_v0" {
        return Err(type_issue(
            if place.field.is_some() {
                "predicate_field_unresolved_v2"
            } else {
                "predicate_place_unresolved_v2"
            },
            place.range,
            if place.field.is_some() {
                "declared direct field"
            } else {
                "resolved task parameter or ensures result place"
            },
            "unresolved place",
        ));
    }
    if fact.eligibility != "eligible_v0" {
        return Err(type_issue(
            "predicate_place_ineligible_v2",
            place.range,
            "task parameter or ensures result place",
            "resolved but ineligible place",
        ));
    }
    Ok(parse_type(fact.type_text.as_deref().unwrap_or("unknown")))
}

fn result_value_type(result: &str) -> &str {
    let result = result.trim();
    if let Some(rest) = result.strip_prefix("Result ") {
        rest.split_once(',').map_or(rest, |(ok, _)| ok).trim()
    } else {
        result
    }
}

fn parse_type(text: &str) -> OperandType {
    match text.trim() {
        "Bool" => OperandType::Bool,
        "Int" => OperandType::Int,
        "UInt" => OperandType::UInt,
        "Text" => OperandType::Text,
        "List Text" => OperandType::ListText,
        "Path" => OperandType::Path,
        other => OperandType::Other(other.to_string()),
    }
}

fn type_issue(
    reason: &'static str,
    range: SourceRange,
    expected: impl Into<String>,
    actual: impl Into<String>,
) -> Box<TypeIssue> {
    let actual = actual.into();
    Box::new(TypeIssue {
        reason,
        range,
        expected: expected.into(),
        actual,
    })
}

fn repair_for(reason: &str) -> String {
    match reason {
        "known_call_requires_no_gap_v2" => "Write `old(place)`, `list_len(place)`, or `list_count(list_text, text)` with no gap before `(`.".to_string(),
        "predicate_place_unresolved_v2" | "predicate_field_unresolved_v2" => "Use a declared task parameter (or `result` in `ensures:`) and at most one declared direct field.".to_string(),
        "predicate_place_ineligible_v2" => "Use a task parameter, or `result` in `ensures:`; task names and other source definitions are not contract places.".to_string(),
        "cross_type_comparison_v2" | "operator_not_supported_for_operand_type_v2" => "Compare operands of the same supported type; Text and List Text support only `==` and `!=`.".to_string(),
        "list_count_requires_list_text_v2" | "list_count_requires_text_match_v2" => "Write `list_count(<List Text place or literal>, <Text place or literal>)`.".to_string(),
        "list_text_comparison_requires_literal_v2" => "Compare a `List Text` place with one exact flat `List Text` literal.".to_string(),
        _ => "Use one complete Predicate v2 comparison, for example `result == \"parse\"`, `result == [\"parse\", \"check\", \"run\"]`, or `result == list_count(items, \"hum\")`.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn fact(contract: &str, result: &str) -> PredicateFact {
        let source = format!(
            "task demo(items: List Text, text_value: Text) -> {result} {{\n  ensures:\n    {contract}\n  does:\n    return 2\n}}\n"
        );
        let parsed = parser::parse_source("predicate.hum", &source);
        let program = Program {
            files: vec![parsed.file],
        };
        let Item::Task(task) = &program.files[0].items[0] else {
            panic!()
        };
        analyze_program(&program)
            .facts_for_task(task)
            .next()
            .expect("predicate fact")
            .clone()
    }

    #[test]
    fn one_program_reuses_one_immutable_analysis() {
        let parsed = parser::parse_source(
            "predicate.hum",
            "task demo() -> UInt {\n  ensures:\n    result == 2\n  does:\n    return 2\n}\n",
        );
        let program = Program {
            files: vec![parsed.file],
        };
        let first = analyze_program(&program);
        let second = analyze_program(&program);
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.facts(), second.facts());
    }

    #[test]
    fn boundary_statuses_are_deterministic() {
        assert_eq!(
            fact("result equals two", "UInt").status,
            RecognitionStatus::NonExecutableProse
        );
        for text in [
            "result = 2",
            "result === 2",
            "result ! = 2",
            "result ! 2",
            "result!2",
            "result <> 2",
            "list_count (items, \"hum\") == result",
            "helper(items) == result",
        ] {
            assert_eq!(
                fact(text, "UInt").status,
                RecognitionStatus::MalformedExecutable,
                "{text}"
            );
        }
        assert_eq!(
            fact("result == 2", "UInt").status,
            RecognitionStatus::RecognizedTyped
        );
        for text in [
            "old (items) == result",
            "list_count(items) == result",
            "result == [\"parse\"",
            "result == [\"parse\")]",
            "(result == 2",
            "result == \"parse",
            "result == \"parse\"\"",
            "@result == 2",
            ", result == 2",
            "== result",
            "result == 2 matching words",
            "list_counted(items, \"hum\") == result",
            "(result) ! \"x\"",
        ] {
            assert_eq!(
                fact(text, "UInt").status,
                RecognitionStatus::MalformedExecutable,
                "{text}"
            );
        }
        for text in [
            "result equals two",
            "\"result == list_count(items, hum)\"",
            "must hold!",
        ] {
            assert_eq!(
                fact(text, "UInt").status,
                RecognitionStatus::NonExecutableProse,
                "{text}"
            );
        }
    }

    #[test]
    fn predicate_v2_types_text_lists_and_count() {
        for (text, result) in [
            ("result == \"parse\"", "Text"),
            ("result == [\"parse\", \"check\"]", "List Text"),
            ("result == list_count([\"hum\", \"hum\"], \"hum\")", "UInt"),
        ] {
            assert_eq!(
                fact(text, result).status,
                RecognitionStatus::RecognizedTyped,
                "{text}"
            );
        }
        let rejected = fact("text_value == 2", "UInt");
        assert_eq!(rejected.status, RecognitionStatus::RejectedSemantics);
        assert_eq!(rejected.reason, "cross_type_comparison_v2");
    }

    #[test]
    fn delimiter_depth_is_bounded() {
        let valid = format!("{}result{} == 2", "(".repeat(16), ")".repeat(16));
        assert_eq!(
            fact(&valid, "UInt").status,
            RecognitionStatus::RecognizedTyped
        );
        let invalid = format!("{}result{} == 2", "(".repeat(17), ")".repeat(17));
        let invalid = fact(&invalid, "UInt");
        assert_eq!(invalid.status, RecognitionStatus::MalformedExecutable);
        assert_eq!(invalid.reason, "delimiter_depth_exceeded_v2");
    }
}
