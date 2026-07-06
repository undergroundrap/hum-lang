pub const SYNTAX_SCHEMA: &str = "hum.syntax_surface.v0";
pub const SOURCE_EXTENSION: &str = ".hum";
pub const MODULE_KEYWORD: &str = "module";
pub const ITEM_KINDS: &[&str] = &["app", "type", "store", "task", "test"];
pub const COMMENT_PREFIXES: &[&str] = &["#", "//"];
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
    use super::{is_item_start, is_test_modifier, syntax_json};

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
}
