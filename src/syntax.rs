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
pub const TEST_OBLIGATION_SECTIONS: &[(&str, &str)] = &[
    ("needs", "precondition"),
    ("ensures", "postcondition"),
    ("watch for", "edge_case"),
    ("tests", "declared_test"),
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
        false,
    );
    push_indent(&mut out, 2);
    out.push_str("}\n");
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
    values: &[(&str, &str)],
    trailing_comma: bool,
) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [");
    for (index, (name, kind)) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str("{\"name\": ");
        push_json_string(out, name);
        out.push_str(", \"kind\": ");
        push_json_string(out, kind);
        out.push('}');
    }
    out.push(']');
    push_comma_newline(out, trailing_comma);
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
        assert!(json.contains("{\"name\": \"watch for\", \"kind\": \"edge_case\"}"));
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
