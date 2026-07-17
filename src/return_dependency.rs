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

pub(crate) fn is_closed_view_derivation_expression(
    expression: &crate::ast::CanonicalExpression,
) -> bool {
    closed_view_derivation_arguments(expression).is_some()
}

pub(crate) fn visible_view_source_root(
    expression: &crate::ast::CanonicalExpression,
) -> Option<String> {
    use crate::ast::CanonicalExpressionKind as Kind;
    match &expression.kind {
        Kind::Identifier(root) => Some(root.clone()),
        Kind::Field { base, .. } | Kind::Element { base, .. } => {
            base.direct_identifier().map(str::to_string)
        }
        Kind::Permission { value, .. } | Kind::Group(value) => visible_view_source_root(value),
        Kind::Call { .. } => closed_view_derivation_source(expression),
        _ => None,
    }
}

pub(crate) fn closed_view_derivation_source(
    expression: &crate::ast::CanonicalExpression,
) -> Option<String> {
    let args = closed_view_derivation_arguments(expression)?;
    visible_view_source_root(&args[0])
}

fn closed_view_derivation_arguments(
    expression: &crate::ast::CanonicalExpression,
) -> Option<&[crate::ast::CanonicalExpression]> {
    use crate::ast::CanonicalExpressionKind as Kind;
    let expression = match &expression.kind {
        Kind::Group(inner) => inner,
        _ => expression,
    };
    let Kind::Call { callee, arguments } = &expression.kind else {
        return None;
    };
    (callee.direct_identifier() == Some(SLICE_UNTIL_OPERATION) && arguments.len() == 2)
        .then_some(arguments)
}

#[cfg(test)]
mod tests {
    use super::{
        closed_view_derivation_source, is_closed_view_derivation_expression,
        visible_view_source_root,
    };
    use crate::ast::{CanonicalExpression, Item, ParsedBodyStatementKind};

    fn expression(source: &str) -> CanonicalExpression {
        let parsed = crate::parser::parse_source(
            "return-dependency-test.hum",
            &format!("task inspect(text: Text) -> Text {{\n  does:\n    return {source}\n}}\n"),
        );
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("expected task");
        };
        let ParsedBodyStatementKind::Return(expression) = &task.body_syntax[0].kind else {
            panic!("expected return expression");
        };
        expression.canonical.clone()
    }

    #[test]
    fn closed_view_derivation_tracks_first_argument_root() {
        assert_eq!(
            closed_view_derivation_source(&expression("slice_until(text, \" \")")),
            Some("text".to_string())
        );
        assert_eq!(
            closed_view_derivation_source(&expression(
                "slice_until(slice_until(text, \" \"), \"u\")"
            )),
            Some("text".to_string())
        );
        assert_eq!(
            closed_view_derivation_source(&expression("identity_text(slice_until(text, \" \"))")),
            None
        );
    }

    #[test]
    fn visible_view_source_accepts_bare_or_closed_only() {
        assert_eq!(
            visible_view_source_root(&expression("text")),
            Some("text".to_string())
        );
        assert_eq!(
            visible_view_source_root(&expression("slice_until(text, \" \" )")),
            Some("text".to_string())
        );
        assert_eq!(
            visible_view_source_root(&expression("identity_text(text)")),
            None
        );
        assert!(is_closed_view_derivation_expression(&expression(
            "slice_until(text, \" \" )"
        )));
    }
}
