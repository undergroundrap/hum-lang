use std::collections::BTreeSet;

use crate::ast::{Item, Program, Section, SectionLine, Task};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::graph::{
    TestCoverage, collect_test_coverages, coverage_key, coverage_match_kind, evidence_obligations,
    is_meaningful_line_text, linked_test_count, test_obligations,
};
use crate::node_id;
use crate::target_facts;

pub const SEMANTIC_GRAPH_SCHEMA: &str = "hum.semantic_graph.v0";

pub fn program_to_json(program: &Program, diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len().saturating_sub(errors);
    let test_coverages = collect_test_coverages(program);

    out.push_str("{\n");
    out.push_str(&format!(
        "  \"schema\": {},\n",
        quote(SEMANTIC_GRAPH_SCHEMA)
    ));
    out.push_str("  \"summary\": {");
    out.push_str(&format!(
        "\"files\": {}, \"items\": {}, \"tasks\": {}, \"tests\": {}, \"errors\": {}, \"warnings\": {}",
        program.files.len(),
        count_items(program),
        count_kind(program, "task"),
        count_kind(program, "test"),
        errors,
        warnings
    ));
    out.push_str("},\n");

    write_portability_reservation(&mut out, program);

    out.push_str("  \"files\": [\n");
    for (file_index, file) in program.files.iter().enumerate() {
        comma_line(&mut out, file_index, 4);
        out.push_str("    {\n");
        out.push_str(&format!(
            "      \"id\": {},\n",
            quote(&node_id::file(&file.path))
        ));
        out.push_str(&format!("      \"path\": {},\n", quote(&file.path)));
        out.push_str(&format!(
            "      \"module\": {},\n",
            file.module
                .as_ref()
                .map_or("null".to_string(), |module| quote(module))
        ));
        out.push_str("      \"folding_ranges\": [\n");
        write_folding_ranges(&mut out, &file.items, 8);
        out.push_str("\n      ],\n");
        out.push_str("      \"symbols\": [\n");
        write_symbols(&mut out, &file.items, 8);
        out.push_str("\n      ],\n");
        out.push_str("      \"items\": [\n");
        write_items(&mut out, &file.items, 0, 8, &test_coverages);
        out.push_str("\n      ]\n");
        out.push_str("    }");
    }
    out.push_str("\n  ],\n");

    out.push_str("  \"diagnostics\": [\n");
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        comma_line(&mut out, index, 4);
        write_diagnostic(&mut out, diagnostic, 4);
    }
    out.push_str("\n  ]\n");
    out.push_str("}\n");
    out
}

struct SourceTargetDeclaration<'a> {
    owner: &'a Item,
    line: &'a SectionLine,
    kind: &'static str,
    value: String,
}

fn write_portability_reservation(out: &mut String, program: &Program) {
    let source_target_declarations = collect_source_target_declarations(program);
    let target_fact_records = portability_values(&source_target_declarations, "target_fact_record");
    let required_capability_families =
        portability_values(&source_target_declarations, "required_capability_family");
    let denied_capability_families =
        portability_values(&source_target_declarations, "denied_capability_family");

    out.push_str("  \"portability\": {\n");
    out.push_str("    \"status\": \"reserved_v0\",\n");
    out.push_str("    \"mode\": \"source_analysis_only_no_target_selection\",\n");
    out.push_str("    \"target_facts_schema\": ");
    out.push_str(&quote(target_facts::TARGET_FACTS_SCHEMA));
    out.push_str(",\n");
    out.push_str("    \"target_fact_record_schema\": ");
    out.push_str(&quote(target_facts::TARGET_FACT_RECORD_SCHEMA));
    out.push_str(",\n");
    out.push_str("    \"boundary_model\": \"docs/PORTABILITY_BOUNDARY_MODEL.md\",\n");
    out.push_str("    \"default_policy\": \"unknown_fails_closed\",\n");
    write_string_array_field(out, 4, "target_fact_records", &target_fact_records, true);
    write_string_array_field(
        out,
        4,
        "required_capability_families",
        &required_capability_families,
        true,
    );
    write_string_array_field(
        out,
        4,
        "denied_capability_families",
        &denied_capability_families,
        true,
    );
    out.push_str("    \"unavailable_capability_families\": [],\n");
    write_source_target_declarations(out, &source_target_declarations);
    out.push_str(",\n");
    out.push_str("    \"adapter_identities\": [],\n");
    out.push_str("    \"artifact_evidence\": [],\n");
    out.push_str("    \"non_claims\": [\n");
    out.push_str("      \"no target selected\",\n");
    out.push_str("      \"no host capability probing\",\n");
    out.push_str("      \"no profile enforcement\",\n");
    out.push_str("      \"no artifact generated\"\n");
    out.push_str("    ]\n");
    out.push_str("  },\n");
}

fn collect_source_target_declarations(program: &Program) -> Vec<SourceTargetDeclaration<'_>> {
    let mut declarations = Vec::new();
    for file in &program.files {
        collect_source_target_declarations_from_items(&file.items, &mut declarations);
    }
    declarations
}

fn collect_source_target_declarations_from_items<'a>(
    items: &'a [Item],
    declarations: &mut Vec<SourceTargetDeclaration<'a>>,
) {
    for item in items {
        for section in item_sections(item) {
            if section.name != "targets" {
                continue;
            }

            for line in section
                .lines
                .iter()
                .filter(|line| is_meaningful_line_text(&line.text))
            {
                if let Some((kind, value)) =
                    target_facts::parse_source_target_declaration_line(&line.text)
                {
                    declarations.push(SourceTargetDeclaration {
                        owner: item,
                        line,
                        kind: kind.as_str(),
                        value,
                    });
                }
            }
        }

        if let Item::App(app) = item {
            collect_source_target_declarations_from_items(&app.items, declarations);
        }
    }
}

fn portability_values(declarations: &[SourceTargetDeclaration<'_>], kind: &str) -> Vec<String> {
    declarations
        .iter()
        .filter(|declaration| declaration.kind == kind)
        .map(|declaration| declaration.value.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn write_source_target_declarations(
    out: &mut String,
    declarations: &[SourceTargetDeclaration<'_>],
) {
    out.push_str("    \"source_target_declarations\": [");
    for (index, declaration) in declarations.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }

        out.push('{');
        out.push_str(&format!(
            "\"id\": {}, ",
            quote(&node_id::span(
                "target-declaration",
                &declaration.line.span,
                &format!("{} {}", declaration.kind, declaration.value)
            ))
        ));
        out.push_str(&format!("\"kind\": {}, ", quote(declaration.kind)));
        out.push_str(&format!("\"value\": {}, ", quote(&declaration.value)));
        out.push_str("\"status\": \"declared_not_enforced_v0\", ");
        out.push_str("\"source_section\": \"targets\", ");
        out.push_str(&format!("\"text\": {}, ", quote(&declaration.line.text)));
        out.push_str(&format!(
            "\"owner\": {{\"id\": {}, \"kind\": {}, \"name\": {}}}, ",
            quote(&item_node_id(declaration.owner)),
            quote(declaration.owner.kind()),
            quote(declaration.owner.name())
        ));
        out.push_str("\"span\": ");
        write_span(out, &declaration.line.span);
        out.push('}');
    }
    out.push(']');
}

fn write_string_array_field(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[String],
    comma: bool,
) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}{}: [", quote(key)));
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&quote(value));
    }
    out.push(']');
    if comma {
        out.push(',');
    }
    out.push('\n');
}
struct SectionFold<'a> {
    owner: &'a Item,
    section: &'a Section,
}

fn write_folding_ranges(out: &mut String, items: &[Item], indent: usize) {
    let folds = collect_section_folds(items);
    for (index, fold) in folds.iter().enumerate() {
        comma_line(out, index, indent);
        write_folding_range(out, fold, indent);
    }
}

fn collect_section_folds(items: &[Item]) -> Vec<SectionFold<'_>> {
    let mut folds = Vec::new();
    collect_section_folds_from_items(items, &mut folds);
    folds
}

fn collect_section_folds_from_items<'a>(items: &'a [Item], folds: &mut Vec<SectionFold<'a>>) {
    for item in items {
        let sections = item_sections(item);
        for section in sections {
            folds.push(SectionFold {
                owner: item,
                section,
            });
        }
        if let Item::App(app) = item {
            collect_section_folds_from_items(&app.items, folds);
        }
    }
}

fn write_folding_range(out: &mut String, fold: &SectionFold<'_>, indent: usize) {
    let pad = " ".repeat(indent);
    let section = fold.section;
    out.push_str(&format!("{pad}{{"));
    out.push_str(&format!(
        "\"id\": {}, ",
        quote(&node_id::span("section", &section.span, &section.name))
    ));
    out.push_str("\"kind\": \"section\", ");
    out.push_str(&format!("\"name\": {}, ", quote(&section.name)));
    out.push_str(&format!(
        "\"owner\": {{\"id\": {}, \"kind\": {}, \"name\": {}}}, ",
        quote(&item_node_id(fold.owner)),
        quote(fold.owner.kind()),
        quote(fold.owner.name())
    ));
    out.push_str("\"span\": ");
    write_span(out, &section.span);
    out.push_str(&format!(
        ", \"start_line\": {}, \"end_line\": {}",
        section.span.line,
        section_end_line(section)
    ));
    out.push('}');
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

fn section_end_line(section: &Section) -> usize {
    section
        .lines
        .last()
        .map_or(section.span.line, |line| line.span.line)
}
fn write_symbols(out: &mut String, items: &[Item], indent: usize) {
    for (index, item) in items.iter().enumerate() {
        comma_line(out, index, indent);
        write_symbol(out, item, indent);
    }
}

fn write_symbol(out: &mut String, item: &Item, indent: usize) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}{{"));
    out.push_str(&format!("\"id\": {}, ", quote(&item_node_id(item))));
    out.push_str(&format!("\"kind\": {}, ", quote(item.kind())));
    out.push_str(&format!("\"name\": {}, ", quote(item.name())));
    out.push_str("\"span\": ");
    write_span(out, item.span());
    out.push_str(", \"children\": [");

    match item {
        Item::App(app) => {
            if !app.items.is_empty() {
                out.push('\n');
                write_symbols(out, &app.items, indent + 2);
                out.push_str(&format!("\n{pad}"));
            }
        }
        Item::Type(type_def) => {
            for (index, field) in type_def.fields.iter().enumerate() {
                if index == 0 {
                    out.push('\n');
                }
                comma_line(out, index, indent + 2);
                let field_id =
                    node_id::span("field", &field.span, &format!("{index} {}", field.name));
                out.push_str(&format!(
                    "{{\"id\": {}, \"kind\": \"field\", \"name\": {}, \"span\": ",
                    quote(&field_id),
                    quote(&field.name)
                ));
                write_span(out, &field.span);
                out.push_str(", \"children\": []}");
            }
            if !type_def.fields.is_empty() {
                out.push_str(&format!("\n{pad}"));
            }
        }
        _ => {}
    }

    out.push_str("]}");
}

fn item_node_id(item: &Item) -> String {
    node_id::span(
        "item",
        item.span(),
        &format!("{} {}", item.kind(), item.name()),
    )
}
fn write_items(
    out: &mut String,
    items: &[Item],
    start_index: usize,
    indent: usize,
    test_coverages: &[TestCoverage<'_>],
) {
    for (offset, item) in items.iter().enumerate() {
        comma_line(out, start_index + offset, indent);
        write_item(out, item, indent, test_coverages);
    }
}

fn write_item(out: &mut String, item: &Item, indent: usize, test_coverages: &[TestCoverage<'_>]) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}{{\n"));
    out.push_str(&format!("{pad}  \"id\": {},\n", quote(&item_node_id(item))));
    out.push_str(&format!("{pad}  \"kind\": {},\n", quote(item.kind())));
    out.push_str(&format!("{pad}  \"name\": {},\n", quote(item.name())));
    out.push_str(&format!("{pad}  \"span\": "));
    write_span(out, item.span());

    match item {
        Item::App(app) => {
            out.push_str(",\n");
            write_sections_field(out, &app.sections, indent + 2);
            out.push_str(",\n");
            out.push_str(&format!("{pad}  \"items\": [\n"));
            write_items(out, &app.items, 0, indent + 4, test_coverages);
            out.push_str(&format!("\n{pad}  ]\n"));
        }
        Item::Type(type_def) => {
            out.push_str(",\n");
            out.push_str(&format!("{pad}  \"fields\": ["));
            for (index, field) in type_def.fields.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                let field_id =
                    node_id::span("field", &field.span, &format!("{index} {}", field.name));
                out.push_str(&format!(
                    "{{\"id\": {}, \"name\": {}, \"type\": {}, \"span\": ",
                    quote(&field_id),
                    quote(&field.name),
                    quote(&field.ty)
                ));
                write_span(out, &field.span);
                out.push('}');
            }
            out.push_str("],\n");
            write_sections_field(out, &type_def.sections, indent + 2);
            out.push('\n');
        }
        Item::Store(store) => {
            out.push_str(",\n");
            out.push_str(&format!("{pad}  \"type\": {},\n", quote(&store.ty)));
            write_sections_field(out, &store.sections, indent + 2);
            out.push('\n');
        }
        Item::Task(task) => {
            out.push_str(",\n");
            out.push_str(&format!("{pad}  \"params\": ["));
            write_params(out, &task.params);
            out.push_str("],\n");
            out.push_str(&format!(
                "{pad}  \"result\": {},\n",
                task.result
                    .as_ref()
                    .map_or("null".to_string(), |result| quote(result))
            ));
            write_sections_field(out, &task.sections, indent + 2);
            out.push_str(",\n");
            write_test_obligations_field(out, task, indent + 2, test_coverages);
            out.push_str(",\n");
            write_evidence_obligations_field(out, task, indent + 2, test_coverages);
            out.push('\n');
        }
        Item::Test(test) => {
            out.push_str(",\n");
            out.push_str(&format!("{pad}  \"params\": ["));
            write_params(out, &test.params);
            out.push_str("],\n");
            out.push_str(&format!("{pad}  \"modifiers\": ["));
            for (index, modifier) in test.modifiers.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str(&quote(modifier));
            }
            out.push_str("],\n");
            write_sections_field(out, &test.sections, indent + 2);
            out.push('\n');
        }
    }
    out.push_str(&format!("{pad}}}"));
}

fn write_params(out: &mut String, params: &[crate::ast::Param]) {
    for (index, param) in params.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        let param_id = node_id::span("param", &param.span, &format!("{index} {}", param.name));
        out.push_str(&format!(
            "{{\"id\": {}, \"name\": {}, \"type\": {}, \"span\": ",
            quote(&param_id),
            quote(&param.name),
            quote(&param.ty)
        ));
        write_span(out, &param.span);
        out.push('}');
    }
}

fn write_sections_field(out: &mut String, sections: &[Section], indent: usize) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}\"sections\": ["));
    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        let section_id = node_id::span("section", &section.span, &section.name);
        out.push_str(&format!(
            "{{\"id\": {}, \"name\": {}, \"span\": ",
            quote(&section_id),
            quote(&section.name)
        ));
        write_span(out, &section.span);
        out.push_str(&format!(
            ", \"lines\": {}, \"line_items\": [",
            section_line_count(section)
        ));
        write_section_line_items(out, section);
        out.push_str("]}");
    }
    out.push(']');
}

fn section_line_count(section: &Section) -> usize {
    section
        .lines
        .iter()
        .filter(|line| !line.text.is_empty())
        .count()
}

fn write_section_line_items(out: &mut String, section: &Section) {
    for (index, line) in section
        .lines
        .iter()
        .filter(|line| !line.text.is_empty())
        .enumerate()
    {
        if index > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str(&format!("\"id\": {}, ", quote(&node_id::line(&line.span))));
        out.push_str(&format!("\"text\": {}, ", quote(&line.text)));
        out.push_str("\"span\": ");
        write_span(out, &line.span);
        out.push_str(&format!(
            ", \"meaningful\": {}",
            if is_meaningful_line_text(&line.text) {
                "true"
            } else {
                "false"
            }
        ));
        out.push('}');
    }
}

fn write_test_obligations_field(
    out: &mut String,
    task: &Task,
    indent: usize,
    test_coverages: &[TestCoverage<'_>],
) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}\"test_obligations\": ["));
    let obligations = test_obligations(task);
    for (index, obligation) in obligations.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        let linked_test_count = linked_test_count(&obligation.covers, test_coverages);
        out.push('{');
        out.push_str(&format!("\"id\": {}, ", quote(&obligation.id)));
        out.push_str(&format!("\"kind\": {}, ", quote(obligation.kind)));
        out.push_str(&format!("\"blame\": {}, ", quote(obligation.blame)));
        out.push_str(&format!(
            "\"source_section\": {}, ",
            quote(obligation.source_section)
        ));
        out.push_str(&format!("\"text\": {}, ", quote(&obligation.line.text)));
        out.push_str("\"span\": ");
        write_span(out, &obligation.line.span);
        out.push_str(&format!(", \"covers\": {}", quote(&obligation.covers)));
        out.push_str(&format!(
            ", \"coverage_key\": {}",
            quote(&coverage_key(&obligation.covers))
        ));
        out.push_str(&format!(
            ", \"suggested_test\": {}",
            quote(&obligation.suggested_test)
        ));
        out.push_str(&format!(
            ", \"link_status\": {}",
            quote(if linked_test_count == 0 {
                "unlinked"
            } else {
                "linked"
            })
        ));
        out.push_str(", \"linked_tests\": [");
        write_linked_tests(out, &obligation.covers, test_coverages);
        out.push(']');
        out.push('}');
    }
    out.push(']');
}

fn write_evidence_obligations_field(
    out: &mut String,
    task: &Task,
    indent: usize,
    test_coverages: &[TestCoverage<'_>],
) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}\"evidence_obligations\": ["));
    let obligations = evidence_obligations(task);
    for (index, obligation) in obligations.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        let linked_evidence_count = linked_test_count(&obligation.covers, test_coverages);
        out.push('{');
        out.push_str(&format!("\"id\": {}, ", quote(&obligation.id)));
        out.push_str(&format!("\"kind\": {}, ", quote(obligation.kind)));
        out.push_str(&format!("\"blame\": {}, ", quote(obligation.blame)));
        out.push_str(&format!(
            "\"source_section\": {}, ",
            quote(obligation.source_section)
        ));
        out.push_str(&format!("\"text\": {}, ", quote(&obligation.line.text)));
        out.push_str("\"span\": ");
        write_span(out, &obligation.line.span);
        out.push_str(&format!(", \"covers\": {}", quote(&obligation.covers)));
        out.push_str(&format!(
            ", \"coverage_key\": {}",
            quote(&coverage_key(&obligation.covers))
        ));
        out.push_str(&format!(
            ", \"suggested_evidence\": {}",
            quote(&obligation.suggested_evidence)
        ));
        out.push_str(&format!(
            ", \"verification_status\": {}",
            quote(if linked_evidence_count == 0 {
                "unverified"
            } else {
                "linked"
            })
        ));
        out.push_str(", \"linked_evidence\": [");
        write_linked_evidence(out, &obligation.covers, test_coverages);
        out.push(']');
        out.push('}');
    }
    out.push(']');
}

fn write_linked_evidence(out: &mut String, covers: &str, test_coverages: &[TestCoverage<'_>]) {
    for (written, (coverage, match_kind)) in test_coverages
        .iter()
        .filter_map(|coverage| coverage_match_kind(covers, coverage).map(|kind| (coverage, kind)))
        .enumerate()
    {
        if written > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str("\"kind\": \"test\", ");
        write_linked_test_fields(out, coverage, match_kind);
        out.push('}');
    }
}

fn write_linked_tests(out: &mut String, covers: &str, test_coverages: &[TestCoverage<'_>]) {
    for (written, (coverage, match_kind)) in test_coverages
        .iter()
        .filter_map(|coverage| coverage_match_kind(covers, coverage).map(|kind| (coverage, kind)))
        .enumerate()
    {
        if written > 0 {
            out.push_str(", ");
        }
        out.push('{');
        write_linked_test_fields(out, coverage, match_kind);
        out.push('}');
    }
}

fn write_linked_test_fields(out: &mut String, coverage: &TestCoverage<'_>, match_kind: &str) {
    out.push_str(&format!("\"name\": {}, ", quote(coverage.test_name)));
    out.push_str("\"modifiers\": [");
    for (index, modifier) in coverage.modifiers.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&quote(modifier));
    }
    out.push_str("], ");
    out.push_str(&format!("\"covers\": {}, ", quote(&coverage.covers)));
    out.push_str(&format!(
        "\"coverage_key\": {}, ",
        quote(&coverage.coverage_key)
    ));
    out.push_str(&format!("\"match\": {}, ", quote(match_kind)));
    out.push_str("\"span\": ");
    write_span(out, &coverage.line.span);
}

fn write_diagnostic(out: &mut String, diagnostic: &Diagnostic, indent: usize) {
    let pad = " ".repeat(indent);
    out.push_str(&format!("{pad}{{"));
    out.push_str(&format!(
        "\"code\": {}, \"title\": {}, \"severity\": {}, \"message\": {}",
        quote(diagnostic.code.as_str()),
        quote(diagnostic.code.title()),
        quote(diagnostic.severity.as_str()),
        quote(&diagnostic.message)
    ));
    if let Some(span) = &diagnostic.span {
        out.push_str(", \"span\": ");
        write_span(out, span);
    }
    if let Some(help) = &diagnostic.help {
        out.push_str(&format!(", \"help\": {}", quote(help)));
    }
    out.push('}');
}

fn write_span(out: &mut String, span: &Span) {
    out.push_str(&format!(
        "{{\"file\": {}, \"line\": {}, \"column\": {}}}",
        quote(&span.file),
        span.line,
        span.column
    ));
}

fn comma_line(out: &mut String, index: usize, indent: usize) {
    if index > 0 {
        out.push_str(",\n");
    }
    out.push_str(&" ".repeat(indent));
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

fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}
#[cfg(test)]
mod tests {
    use super::program_to_json;
    use crate::ast::Program;
    use crate::diagnostic::{Diagnostic, DiagnosticCode, Span};
    use crate::parser::parse_source;

    #[test]
    fn diagnostic_json_includes_code_and_title() {
        let diagnostic = Diagnostic::warning(
            DiagnosticCode::SECTION_OUT_OF_ORDER,
            "section is out of order",
            Some(Span::new("bad.hum", 5, 1)),
        );
        let json = program_to_json(&Program::default(), &[diagnostic]);

        assert!(json.contains("\"code\": \"H0108\""));
        assert!(json.contains("\"title\": \"section out of order\""));
    }

    #[test]
    fn graph_json_includes_portability_reservation() {
        let json = program_to_json(&Program::default(), &[]);

        assert!(json.contains("\"portability\""));
        assert!(json.contains("\"status\": \"reserved_v0\""));
        assert!(json.contains("\"mode\": \"source_analysis_only_no_target_selection\""));
        assert!(json.contains("\"target_facts_schema\": \"hum.target_facts.v0\""));
        assert!(json.contains("\"target_fact_record_schema\": \"hum.target_fact_record.v0\""));
        assert!(json.contains("\"default_policy\": \"unknown_fails_closed\""));
        assert!(json.contains("\"target_fact_records\": []"));
        assert!(json.contains("\"required_capability_families\": []"));
        assert!(json.contains("\"source_target_declarations\": []"));
        assert!(json.contains("\"no target selected\""));
        assert!(json.contains("\"no artifact generated\""));
    }

    #[test]
    fn graph_json_includes_source_target_declarations() {
        let source = r#"task read manifest() {
  why:
    read a release manifest without assuming host powers

  targets:
    triple: wasm32-wasi-preview1
    requires: os.filesystem
    denies: os.network

  needs:
    manifest path is inside the declared root

  cost:
    time: O(1)
    check: warn

  does:
    return manifest
}
"#;
        let parsed = parse_source("portable.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"target_fact_records\": [\"wasm32-wasi-preview1\"]"));
        assert!(json.contains("\"required_capability_families\": [\"os.filesystem\"]"));
        assert!(json.contains("\"denied_capability_families\": [\"os.network\"]"));
        assert!(json.contains("\"source_target_declarations\": ["));
        assert!(json.contains("\"kind\": \"target_fact_record\""));
        assert!(json.contains("\"kind\": \"required_capability_family\""));
        assert!(json.contains("\"kind\": \"denied_capability_family\""));
        assert!(json.contains("\"status\": \"declared_not_enforced_v0\""));
        assert!(json.contains("\"source_section\": \"targets\""));
        assert!(json.contains("\"owner\": {\"id\": \"item:portable.hum:1:1:task-read-manifest\""));
        assert!(json.contains("\"no target selected\""));
    }
    #[test]
    fn task_json_includes_test_and_evidence_obligations() {
        let source = r#"task add task(title: Text) -> Result Task, TaskError {
  why:
    save a task

  needs:
    title is not empty

  ensures:
    new task is saved

  protects:
    user data remains private

  trusts:
    local profile storage is durable

  watch for:
    title may be only spaces

  tests:
    empty title is rejected

  does:
    return task
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"test_obligations\""));
        assert!(json.contains("\"kind\": \"precondition\""));
        assert!(json.contains("\"blame\": \"caller\""));
        assert!(json.contains("\"kind\": \"postcondition\""));
        assert!(json.contains("\"blame\": \"callee\""));
        assert!(json.contains("\"kind\": \"edge_case\""));
        assert!(json.contains("\"kind\": \"declared_test\""));
        assert!(json.contains("\"blame\": \"evidence\""));
        assert!(json.contains("\"evidence_obligations\""));
        assert!(json.contains("\"kind\": \"security_property\""));
        assert!(json.contains("\"blame\": \"security_boundary\""));
        assert!(json.contains("\"kind\": \"trust_boundary\""));
        assert!(json.contains("\"blame\": \"trust_boundary\""));
        assert!(json.contains("\"verification_status\": \"unverified\""));
        assert!(json.contains("\"covers\": \"add task protects user data remains private\""));
        assert!(json.contains("\"coverage_key\": \"add task protects user data remains private\""));
        assert!(json.contains("\"linked_evidence\": []"));
        assert!(
            json.contains("\"suggested_evidence\": \"add task proves user data remains private\"")
        );
        assert!(json.contains("\"suggested_test\": \"add task requires title is not empty\""));
        assert!(
            json.contains("\"id\": \"obligation:demo.hum:6:5:add-task-needs-title-is-not-empty\"")
        );
        assert!(json.contains(
            "\"id\": \"evidence:demo.hum:12:5:add-task-protects-user-data-remains-private\""
        ));
    }

    #[test]
    fn evidence_obligations_link_to_covering_tests() {
        let source = r#"task add task(title: Text) -> Task {
  why:
    save a task

  protects:
    user data remains private

  trusts:
    local profile storage is durable

  does:
    return task
}

test add task privacy evidence unit {
  why:
    prove visible security and trust evidence links

  covers:
    add task protects user data remains private
    Add Task ASSUMES: local profile storage is durable.

  does:
    expect security review recorded
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"verification_status\": \"linked\""));
        assert!(json.contains(
            "\"linked_evidence\": [{\"kind\": \"test\", \"name\": \"add task privacy evidence\""
        ));
        assert!(json.contains("\"covers\": \"add task protects user data remains private\""));
        assert!(json.contains("\"match\": \"exact\""));
        assert!(
            json.contains("\"covers\": \"Add Task ASSUMES: local profile storage is durable.\"")
        );
        assert!(json.contains("\"match\": \"canonical\""));
    }

    #[test]
    fn task_obligations_link_to_covering_tests() {
        let source = r#"task add task(title: Text) -> Task {
  why:
    save a task

  needs:
    title is not empty

  ensures:
    new task is saved

  does:
    return task
}

test add task saves nonempty title property {
  why:
    prove saving behavior

  covers:
    add task ensures new task is saved

  does:
    expect task saved
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"link_status\": \"linked\""));
        assert!(json.contains("\"link_status\": \"unlinked\""));
        assert!(json.contains("\"linked_tests\": [{\"name\": \"add task saves nonempty title\""));
        assert!(json.contains("\"modifiers\": [\"property\"]"));
        assert!(json.contains("\"covers\": \"add task ensures new task is saved\""));
    }

    #[test]
    fn task_obligations_report_canonical_test_links() {
        let source = r#"task add task(title: Text) -> Task {
  why:
    save a task

  needs:
    title is not empty

  does:
    return task
}

test add task rejects empty title unit {
  why:
    prove input validation

  covers:
    Add Task REQUIRES: the title is non-empty.

  does:
    expect empty title rejected
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"coverage_key\": \"add task needs title not empty\""));
        assert!(json.contains("\"match\": \"canonical\""));
        assert!(json.contains("\"covers\": \"Add Task REQUIRES: the title is non-empty.\""));
    }

    #[test]
    fn graph_json_includes_section_folding_ranges() {
        let source = r#"task add task(title: Text) -> Task {
  why:
    save a task
    preserve intent folding

  does:
    return task
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"folding_ranges\""));
        assert!(json.contains("\"id\": \"section:demo.hum:2:3:why\""));
        assert!(json.contains("\"owner\": {\"id\": \"item:demo.hum:1:1:task-add-task\""));
        assert!(json.contains("\"start_line\": 2, \"end_line\": 5"));
        assert!(json.contains("\"id\": \"section:demo.hum:6:3:does\""));
    }

    #[test]
    fn graph_json_includes_document_symbols() {
        let source = r#"app Demo {
  why:
    group tasks

  task nested task(title: Text) {
    why:
      prove nested symbol output

    does:
      return title
  }
}

type WorkItem {
  id: Text
  title: Text
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"symbols\""));
        assert!(json.contains("\"id\": \"item:demo.hum:1:1:app-demo\", \"kind\": \"app\""));
        assert!(
            json.contains("\"id\": \"item:demo.hum:5:3:task-nested-task\", \"kind\": \"task\"")
        );
        assert!(json.contains("\"id\": \"item:demo.hum:14:1:type-workitem\", \"kind\": \"type\""));
        assert!(json.contains("\"id\": \"field:demo.hum:15:3:0-id\", \"kind\": \"field\""));
    }

    #[test]
    fn graph_json_includes_source_derived_node_ids() {
        let source = r#"task add task(title: Text) -> Task {
  why:
    save a task

  does:
    return task
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"id\": \"file:demo.hum\""));
        assert!(json.contains("\"id\": \"item:demo.hum:1:1:task-add-task\""));
        assert!(json.contains("\"id\": \"param:demo.hum:1:1:0-title\""));
        assert!(json.contains("\"id\": \"section:demo.hum:2:3:why\""));
        assert!(json.contains("\"id\": \"line:demo.hum:3:5\""));
    }

    #[test]
    fn section_json_includes_line_items_with_spans() {
        let source = r#"task add task(title: Text) -> Task {
  why:
    save a task
    // explain later

  does:
    return task
}
"#;
        let parsed = parse_source("demo.hum", source);
        let program = Program {
            files: vec![parsed.file],
        };
        let json = program_to_json(&program, &[]);

        assert!(json.contains("\"name\": \"why\", \"span\": {\"file\": \"demo.hum\", \"line\": 2"));
        assert!(
            json.contains(
                "\"text\": \"save a task\", \"span\": {\"file\": \"demo.hum\", \"line\": 3"
            )
        );
        assert!(json.contains("\"meaningful\": true"));
        assert!(json.contains("\"text\": \"// explain later\""));
        assert!(json.contains("\"meaningful\": false"));
    }
}
