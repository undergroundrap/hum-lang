#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReturnDependency {
    pub result_type: String,
    pub source: String,
}

pub(crate) const SLICE_UNTIL_OPERATION: &str = "slice_until";

pub(crate) fn parse_return_dependency(result: &str) -> Option<ReturnDependency> {
    let tokens = result.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 3 || tokens[tokens.len() - 2] != "from" {
        return None;
    }
    Some(ReturnDependency {
        result_type: tokens[..tokens.len() - 2].join(" "),
        source: tokens[tokens.len() - 1].to_string(),
    })
}

pub(crate) fn result_type_without_return_dependency(result: &str) -> String {
    parse_return_dependency(result)
        .map(|dependency| dependency.result_type)
        .unwrap_or_else(|| result.trim().to_string())
}

pub(crate) fn is_bare_source_name(source: &str) -> bool {
    let source = source.trim();
    let mut chars = source.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first == '_')
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

pub(crate) fn is_closed_view_deriving_operation(callee: &str) -> bool {
    callee.trim() == SLICE_UNTIL_OPERATION
}

pub(crate) fn is_closed_view_derivation_expression(expression: &str) -> bool {
    closed_view_derivation_arguments(expression).is_some()
}

pub(crate) fn visible_view_source_root(expression: &str) -> Option<String> {
    let expression = trim_outer_parens(expression.trim());
    bare_place_root(expression).or_else(|| closed_view_derivation_source(expression))
}

pub(crate) fn closed_view_derivation_source(expression: &str) -> Option<String> {
    let args = closed_view_derivation_arguments(expression)?;
    visible_view_source_root(args[0])
}

fn closed_view_derivation_arguments(expression: &str) -> Option<Vec<&str>> {
    let expression = trim_outer_parens(expression.trim());
    let (callee, args) = split_call(expression)?;
    if !is_closed_view_deriving_operation(callee) {
        return None;
    }
    let args = split_arguments(args);
    (args.len() == 2).then_some(args)
}

fn bare_place_root(expression: &str) -> Option<String> {
    let expression = expression.trim();
    if expression.is_empty()
        || expression.contains('(')
        || expression.contains(' ')
        || expression.starts_with('"')
        || expression.starts_with('[')
        || expression.starts_with('{')
    {
        return None;
    }
    let root = first_resource(expression);
    root.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase() || ch == '_')
        .then_some(root)
}

fn first_resource(text: &str) -> String {
    text.split(|ch: char| ch == '.' || ch == '[' || ch.is_whitespace() || ch == ',')
        .find(|part| !part.is_empty())
        .unwrap_or(text)
        .trim()
        .to_string()
}

fn split_call(text: &str) -> Option<(&str, &str)> {
    if !text.ends_with(')') {
        return None;
    }
    let open = find_top_level_char(text, '(')?;
    Some((&text[..open], &text[open + 1..text.len() - 1]))
}

fn split_arguments(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut depth = 0isize;
    let mut in_string = false;
    for (index, ch) in text.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' | '[' | '{' if !in_string => depth += 1,
            ')' | ']' | '}' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                let part = text[start..index].trim();
                if !part.is_empty() {
                    parts.push(part);
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    let part = text[start..].trim();
    if !part.is_empty() {
        parts.push(part);
    }
    parts
}

fn find_top_level_char(text: &str, needle: char) -> Option<usize> {
    let mut depth = 0isize;
    let mut in_string = false;
    for (index, ch) in text.char_indices() {
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if ch == needle && depth == 0 {
            return Some(index);
        }
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {}
        }
    }
    None
}

fn trim_outer_parens(mut text: &str) -> &str {
    loop {
        let trimmed = text.trim();
        if trimmed.starts_with('(') && trimmed.ends_with(')') && outer_parens_wrap(trimmed) {
            text = &trimmed[1..trimmed.len() - 1];
        } else {
            return trimmed;
        }
    }
}

fn outer_parens_wrap(text: &str) -> bool {
    let mut depth = 0isize;
    let mut in_string = false;
    for (index, ch) in text.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' if !in_string => depth += 1,
            ')' if !in_string => {
                depth -= 1;
                if depth == 0 && index != text.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

#[cfg(test)]
mod tests {
    use super::{
        closed_view_derivation_source, is_closed_view_derivation_expression,
        visible_view_source_root,
    };

    #[test]
    fn closed_view_derivation_tracks_first_argument_root() {
        assert_eq!(
            closed_view_derivation_source("slice_until(text, \" \")"),
            Some("text".to_string())
        );
        assert_eq!(
            closed_view_derivation_source("slice_until(slice_until(text, \" \"), \"u\")"),
            Some("text".to_string())
        );
        assert_eq!(
            closed_view_derivation_source("identity_text(slice_until(text, \" \"))"),
            None
        );
    }

    #[test]
    fn visible_view_source_accepts_bare_or_closed_only() {
        assert_eq!(visible_view_source_root("text"), Some("text".to_string()));
        assert_eq!(
            visible_view_source_root("slice_until(text, \" \" )"),
            Some("text".to_string())
        );
        assert_eq!(visible_view_source_root("identity_text(text)"), None);
        assert!(is_closed_view_derivation_expression(
            "slice_until(text, \" \" )"
        ));
    }
}
