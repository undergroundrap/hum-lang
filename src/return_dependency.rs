#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReturnDependency {
    pub result_type: String,
    pub source: String,
}

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
