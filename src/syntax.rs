pub const SYNTAX_SCHEMA: &str = "hum.syntax_surface.v0";
pub const TEXTMATE_SCOPE_NAME: &str = "source.hum";
pub const SOURCE_EXTENSION: &str = ".hum";
pub const MODULE_KEYWORD: &str = "module";
pub const ITEM_KINDS: &[&str] = &["app", "type", "store", "task", "test"];
pub const DOUBLE_SLASH_COMMENT_PREFIX: &str = concat!("/", "/");
pub const COMMENT_PREFIXES: &[&str] = &["#", DOUBLE_SLASH_COMMENT_PREFIX];
pub const TEST_MODIFIERS: &[&str] = &[
    "unit",
    "property",
    "fuzz",
    "regression",
    "integration",
    "model",
];
pub const TASK_SECTION_ORDER: &[&str] = &[
    "why",
    "uses",
    "changes",
    "needs",
    "ensures",
    "protects",
    "trusts",
    "fails when",
    "watch for",
    "cost",
    "allocates",
    "avoids",
    "tradeoffs",
    "optimizes",
    "tests",
    "does",
];
pub const TEST_SECTION_ORDER: &[&str] = &[
    "why",
    "uses",
    "needs",
    "regression",
    "covers",
    "avoids",
    "cost",
    "does",
];
pub const TEST_OBLIGATION_SECTIONS: &[TaskObligationSection] = &[
    TaskObligationSection {
        name: "needs",
        kind: "precondition",
        blame: "caller",
    },
    TaskObligationSection {
        name: "ensures",
        kind: "postcondition",
        blame: "callee",
    },
    TaskObligationSection {
        name: "watch for",
        kind: "edge_case",
        blame: "evidence",
    },
    TaskObligationSection {
        name: "tests",
        kind: "declared_test",
        blame: "evidence",
    },
];

pub struct TaskObligationSection {
    pub name: &'static str,
    pub kind: &'static str,
    pub blame: &'static str,
}

pub struct SectionHelp {
    pub name: &'static str,
    pub applies_to: &'static [&'static str],
    pub hover: &'static str,
}

pub struct SemanticTokenRule {
    pub source: &'static str,
    pub token_type: &'static str,
    pub modifiers: &'static [&'static str],
    pub hum_role: &'static str,
}

pub const SEMANTIC_TOKEN_TYPES: &[&str] = &[
    "namespace",
    "type",
    "function",
    "variable",
    "parameter",
    "property",
    "keyword",
    "comment",
];

pub const SEMANTIC_TOKEN_MODIFIERS: &[&str] = &[
    "declaration",
    "definition",
    "documentation",
    "readonly",
    "modification",
];

pub const SEMANTIC_TOKEN_RULES: &[SemanticTokenRule] = &[
    SemanticTokenRule {
        source: "module_keyword",
        token_type: "keyword",
        modifiers: &["declaration"],
        hum_role: "module",
    },
    SemanticTokenRule {
        source: "module_path",
        token_type: "namespace",
        modifiers: &["declaration"],
        hum_role: "module",
    },
    SemanticTokenRule {
        source: "item_kind",
        token_type: "keyword",
        modifiers: &["declaration"],
        hum_role: "item",
    },
    SemanticTokenRule {
        source: "app_name",
        token_type: "namespace",
        modifiers: &["definition"],
        hum_role: "item",
    },
    SemanticTokenRule {
        source: "type_name",
        token_type: "type",
        modifiers: &["definition"],
        hum_role: "item",
    },
    SemanticTokenRule {
        source: "store_name",
        token_type: "variable",
        modifiers: &["definition", "modification"],
        hum_role: "state",
    },
    SemanticTokenRule {
        source: "task_name",
        token_type: "function",
        modifiers: &["definition"],
        hum_role: "behavior",
    },
    SemanticTokenRule {
        source: "test_name",
        token_type: "function",
        modifiers: &["definition"],
        hum_role: "evidence",
    },
    SemanticTokenRule {
        source: "parameter_name",
        token_type: "parameter",
        modifiers: &["declaration"],
        hum_role: "signature",
    },
    SemanticTokenRule {
        source: "type_field_name",
        token_type: "property",
        modifiers: &["declaration"],
        hum_role: "shape",
    },
    SemanticTokenRule {
        source: "section_header",
        token_type: "keyword",
        modifiers: &["documentation"],
        hum_role: "intent",
    },
    SemanticTokenRule {
        source: "section:uses",
        token_type: "keyword",
        modifiers: &["documentation", "readonly"],
        hum_role: "effect_read",
    },
    SemanticTokenRule {
        source: "section:changes",
        token_type: "keyword",
        modifiers: &["documentation", "modification"],
        hum_role: "effect_write",
    },
    SemanticTokenRule {
        source: "section:protects",
        token_type: "keyword",
        modifiers: &["documentation"],
        hum_role: "security",
    },
    SemanticTokenRule {
        source: "section:trusts",
        token_type: "keyword",
        modifiers: &["documentation"],
        hum_role: "security",
    },
    SemanticTokenRule {
        source: "section:cost",
        token_type: "keyword",
        modifiers: &["documentation"],
        hum_role: "performance",
    },
    SemanticTokenRule {
        source: "section:allocates",
        token_type: "keyword",
        modifiers: &["documentation"],
        hum_role: "performance",
    },
    SemanticTokenRule {
        source: "section:optimizes",
        token_type: "keyword",
        modifiers: &["documentation"],
        hum_role: "performance",
    },
    SemanticTokenRule {
        source: "test_modifier",
        token_type: "keyword",
        modifiers: &["declaration"],
        hum_role: "evidence",
    },
    SemanticTokenRule {
        source: "comment",
        token_type: "comment",
        modifiers: &["documentation"],
        hum_role: "trivia",
    },
];

pub const SECTION_CATALOG: &[SectionHelp] = &[
    SectionHelp {
        name: "why",
        applies_to: &["app", "type", "store", "task", "test"],
        hover: "Explains why this item exists and what value it should provide.",
    },
    SectionHelp {
        name: "uses",
        applies_to: &["task", "test"],
        hover: "Lists read dependencies, capabilities, or outside facts this item relies on.",
    },
    SectionHelp {
        name: "changes",
        applies_to: &["task"],
        hover: "Lists resources or state this task is allowed to mutate.",
    },
    SectionHelp {
        name: "needs",
        applies_to: &["task", "test"],
        hover: "States preconditions that callers, fixtures, or context must satisfy.",
    },
    SectionHelp {
        name: "ensures",
        applies_to: &["task"],
        hover: "States promises the task must satisfy after successful completion.",
    },
    SectionHelp {
        name: "protects",
        applies_to: &["task"],
        hover: "Names the safety or security property this task must preserve.",
    },
    SectionHelp {
        name: "trusts",
        applies_to: &["task"],
        hover: "Names boundaries, inputs, systems, or assumptions this task relies on.",
    },
    SectionHelp {
        name: "fails when",
        applies_to: &["task"],
        hover: "States expected failure modes that should be visible to callers and tests.",
    },
    SectionHelp {
        name: "watch for",
        applies_to: &["task"],
        hover: "Records edge cases and risky inputs that should become test obligations.",
    },
    SectionHelp {
        name: "cost",
        applies_to: &["task", "test"],
        hover: "States time, space, allocation, and check expectations.",
    },
    SectionHelp {
        name: "allocates",
        applies_to: &["task"],
        hover: "Records allocation expectations or limits that reviewers and tools should see.",
    },
    SectionHelp {
        name: "avoids",
        applies_to: &["task", "test"],
        hover: "Names implementation shapes, regressions, or risks this item should avoid.",
    },
    SectionHelp {
        name: "tradeoffs",
        applies_to: &["task"],
        hover: "Explains accepted compromises so optimization choices are reviewable.",
    },
    SectionHelp {
        name: "optimizes",
        applies_to: &["task"],
        hover: "Names the priority to optimize when correct designs have competing costs.",
    },
    SectionHelp {
        name: "tests",
        applies_to: &["task"],
        hover: "Declares test obligations that should be linked to first-class tests.",
    },
    SectionHelp {
        name: "does",
        applies_to: &["task", "test"],
        hover: "Contains body text captured by Milestone 0; executable semantics are future work.",
    },
    SectionHelp {
        name: "regression",
        applies_to: &["test"],
        hover: "Describes the old failure mode this regression test prevents from returning.",
    },
    SectionHelp {
        name: "covers",
        applies_to: &["test"],
        hover: "Links a test to the task promise, obligation, or behavior it proves.",
    },
];

struct TextMateRule {
    name: &'static str,
    pattern: String,
}

pub fn is_item_start(trimmed: &str) -> bool {
    ITEM_KINDS.iter().any(|kind| {
        trimmed
            .strip_prefix(*kind)
            .is_some_and(|rest| rest.starts_with(' '))
    })
}

pub fn is_test_modifier(word: &str) -> bool {
    TEST_MODIFIERS.contains(&word)
}

pub fn syntax_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_string_property(&mut out, 2, "schema", SYNTAX_SCHEMA, true);
    push_string_property(&mut out, 2, "source_extension", SOURCE_EXTENSION, true);
    push_string_property(&mut out, 2, "module_keyword", MODULE_KEYWORD, true);
    push_string_array(&mut out, 2, "item_kinds", ITEM_KINDS, true);
    push_string_array(&mut out, 2, "comment_prefixes", COMMENT_PREFIXES, true);
    push_string_array(&mut out, 2, "test_modifiers", TEST_MODIFIERS, true);
    push_indent(&mut out, 2);
    out.push_str("\"section_headers\": {\n");
    push_string_array(&mut out, 4, "task_order", TASK_SECTION_ORDER, true);
    push_string_array(&mut out, 4, "test_order", TEST_SECTION_ORDER, true);
    push_obligation_sections(
        &mut out,
        4,
        "task_obligations",
        TEST_OBLIGATION_SECTIONS,
        true,
    );
    push_section_catalog(&mut out, 4, "section_catalog", SECTION_CATALOG, false);
    push_indent(&mut out, 2);
    out.push_str("},\n");
    push_semantic_tokens(&mut out, 2);
    out.push_str("}\n");
    out
}

pub fn textmate_json() -> String {
    let section_names = unique_section_names();
    let module_pattern = format!(
        "^[[:space:]]*({})(?=[^[:alnum:]_]|$)",
        regex_word_literal(MODULE_KEYWORD)
    );
    let item_pattern = format!(
        "^[[:space:]]*({})(?=[^[:alnum:]_]|$)",
        regex_alternation(ITEM_KINDS)
    );
    let section_pattern = format!(
        "^[[:space:]]*({})[[:space:]]*:",
        regex_alternation(&section_names)
    );
    let modifier_pattern = format!(
        "(^|[^[:alnum:]_])({})($|[^[:alnum:]_])",
        regex_alternation(TEST_MODIFIERS)
    );

    let comment_rules = [
        TextMateRule {
            name: "comment.line.number-sign.hum",
            pattern: "#.*$".to_string(),
        },
        TextMateRule {
            name: "comment.line.double-slash.hum",
            pattern: "/{2}.*$".to_string(),
        },
    ];
    let module_rules = [TextMateRule {
        name: "keyword.control.module.hum",
        pattern: module_pattern,
    }];
    let item_rules = [TextMateRule {
        name: "keyword.declaration.item.hum",
        pattern: item_pattern,
    }];
    let section_rules = [TextMateRule {
        name: "keyword.other.section.hum",
        pattern: section_pattern,
    }];
    let modifier_rules = [TextMateRule {
        name: "storage.modifier.test.hum",
        pattern: modifier_pattern,
    }];

    let mut out = String::new();
    out.push_str("{\n");
    push_string_property(&mut out, 2, "name", "Hum", true);
    push_string_property(&mut out, 2, "scopeName", TEXTMATE_SCOPE_NAME, true);
    push_string_array(&mut out, 2, "fileTypes", &["hum"], true);
    push_indent(&mut out, 2);
    out.push_str("\"patterns\": [\n");
    push_include(&mut out, 4, "#comments", true);
    push_include(&mut out, 4, "#module", true);
    push_include(&mut out, 4, "#items", true);
    push_include(&mut out, 4, "#sections", true);
    push_include(&mut out, 4, "#test-modifiers", false);
    push_indent(&mut out, 2);
    out.push_str("],\n");
    push_indent(&mut out, 2);
    out.push_str("\"repository\": {\n");
    push_repository_matches(&mut out, 4, "comments", &comment_rules, true);
    push_repository_matches(&mut out, 4, "module", &module_rules, true);
    push_repository_matches(&mut out, 4, "items", &item_rules, true);
    push_repository_matches(&mut out, 4, "sections", &section_rules, true);
    push_repository_matches(&mut out, 4, "test-modifiers", &modifier_rules, false);
    push_indent(&mut out, 2);
    out.push_str("}\n");
    out.push_str("}\n");
    out
}

fn unique_section_names() -> Vec<&'static str> {
    let mut names = Vec::new();
    for name in TASK_SECTION_ORDER.iter().chain(TEST_SECTION_ORDER.iter()) {
        if !names.contains(name) {
            names.push(*name);
        }
    }
    names
}

fn regex_alternation(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| regex_word_literal(value))
        .collect::<Vec<_>>()
        .join("|")
}

fn regex_word_literal(value: &str) -> String {
    let mut out = String::new();
    let mut previous_was_space = false;
    for ch in value.chars() {
        if ch == ' ' {
            if !previous_was_space {
                out.push_str("[[:space:]]+");
            }
            previous_was_space = true;
        } else {
            previous_was_space = false;
            push_regex_escaped_char(&mut out, ch);
        }
    }
    out
}

fn push_regex_escaped_char(out: &mut String, ch: char) {
    if matches!(
        ch,
        '\\' | '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
    ) {
        out.push('\\');
    }
    out.push(ch);
}

fn push_string_property(
    out: &mut String,
    indent: usize,
    key: &str,
    value: &str,
    trailing_comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_json_string(out, value);
    push_comma_newline(out, trailing_comma);
}

fn push_string_array(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[&str],
    trailing_comma: bool,
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
    push_comma_newline(out, trailing_comma);
}

fn push_include(out: &mut String, indent: usize, include: &str, trailing_comma: bool) {
    push_indent(out, indent);
    out.push_str("{\"include\": ");
    push_json_string(out, include);
    out.push('}');
    push_comma_newline(out, trailing_comma);
}

fn push_repository_matches(
    out: &mut String,
    indent: usize,
    key: &str,
    rules: &[TextMateRule],
    trailing_comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": {\n");
    push_indent(out, indent + 2);
    out.push_str("\"patterns\": [\n");
    for (index, rule) in rules.iter().enumerate() {
        push_indent(out, indent + 4);
        out.push_str("{\"name\": ");
        push_json_string(out, rule.name);
        out.push_str(", \"match\": ");
        push_json_string(out, &rule.pattern);
        out.push('}');
        push_comma_newline(out, index + 1 < rules.len());
    }
    push_indent(out, indent + 2);
    out.push_str("]\n");
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, trailing_comma);
}

fn push_obligation_sections(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[TaskObligationSection],
    trailing_comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    for (index, section) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str("{\"name\": ");
        push_json_string(out, section.name);
        out.push_str(", \"kind\": ");
        push_json_string(out, section.kind);
        out.push_str(", \"blame\": ");
        push_json_string(out, section.blame);
        out.push('}');
    }
    out.push(']');
    push_comma_newline(out, trailing_comma);
}

fn push_section_catalog(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[SectionHelp],
    trailing_comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [\n");
    for (index, section) in values.iter().enumerate() {
        push_indent(out, indent + 2);
        out.push_str("{\"name\": ");
        push_json_string(out, section.name);
        out.push_str(", \"applies_to\": [");
        push_json_string_values(out, section.applies_to);
        out.push_str("], \"hover\": ");
        push_json_string(out, section.hover);
        out.push('}');
        push_comma_newline(out, index + 1 < values.len());
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, trailing_comma);
}

fn push_semantic_tokens(out: &mut String, indent: usize) {
    push_indent(out, indent);
    out.push_str("\"semantic_tokens\": {\n");
    push_string_array(out, indent + 2, "token_types", SEMANTIC_TOKEN_TYPES, true);
    push_string_array(
        out,
        indent + 2,
        "token_modifiers",
        SEMANTIC_TOKEN_MODIFIERS,
        true,
    );
    push_semantic_token_rules(out, indent + 2, "rules", SEMANTIC_TOKEN_RULES, false);
    push_indent(out, indent);
    out.push_str("}\n");
}

fn push_semantic_token_rules(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[SemanticTokenRule],
    trailing_comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [\n");
    for (index, rule) in values.iter().enumerate() {
        push_indent(out, indent + 2);
        out.push_str("{\"source\": ");
        push_json_string(out, rule.source);
        out.push_str(", \"token_type\": ");
        push_json_string(out, rule.token_type);
        out.push_str(", \"modifiers\": [");
        push_json_string_values(out, rule.modifiers);
        out.push_str("], \"hum_role\": ");
        push_json_string(out, rule.hum_role);
        out.push('}');
        push_comma_newline(out, index + 1 < values.len());
    }
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, trailing_comma);
}

fn push_json_string_values(out: &mut String, values: &[&str]) {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(out, value);
    }
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
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
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

fn push_comma_newline(out: &mut String, trailing_comma: bool) {
    if trailing_comma {
        out.push(',');
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::{is_item_start, is_test_modifier, syntax_json, textmate_json};

    #[test]
    fn item_start_requires_kind_and_space() {
        assert!(is_item_start("task save task() {"));
        assert!(is_item_start("type Task {"));
        assert!(!is_item_start("task"));
        assert!(!is_item_start("taskish name {"));
    }

    #[test]
    fn exposes_current_editor_surface_as_json() {
        let json = syntax_json();
        assert!(json.contains("\"schema\": \"hum.syntax_surface.v0\""));
        assert!(
            json.contains("\"item_kinds\": [\"app\", \"type\", \"store\", \"task\", \"test\"]")
        );
        assert!(json.contains("\"task_order\": [\"why\", \"uses\", \"changes\""));
        assert!(json.contains(
            "{\"name\": \"watch for\", \"kind\": \"edge_case\", \"blame\": \"evidence\"}"
        ));
        assert!(json.contains("\"section_catalog\": ["));
        assert!(json.contains(
            "{\"name\": \"cost\", \"applies_to\": [\"task\", \"test\"], \"hover\": \"States time, space, allocation, and check expectations.\"}"
        ));
        assert!(json.contains("\"semantic_tokens\": {"));
        assert!(json.contains(
            "\"token_types\": [\"namespace\", \"type\", \"function\", \"variable\", \"parameter\", \"property\", \"keyword\", \"comment\"]"
        ));
        assert!(json.contains(
            "{\"source\": \"section:protects\", \"token_type\": \"keyword\", \"modifiers\": [\"documentation\"], \"hum_role\": \"security\"}"
        ));
        assert!(is_test_modifier("regression"));
        assert!(!is_test_modifier("benchmark"));
    }

    #[test]
    fn emits_textmate_grammar_from_current_surface() {
        let json = textmate_json();
        assert!(json.contains("\"scopeName\": \"source.hum\""));
        assert!(json.contains("app|type|store|task|test"));
        assert!(json.contains("fails[[:space:]]+when"));
        assert!(json.contains("watch[[:space:]]+for"));
        assert!(json.contains("unit|property|fuzz|regression|integration|model"));
    }
}
