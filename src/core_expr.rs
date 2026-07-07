#[derive(Debug, Clone)]
pub struct CoreExpressionPreview {
    pub text: String,
    pub kind: &'static str,
    pub status: &'static str,
    pub atoms: Vec<ExpressionAtom>,
    pub operators: Vec<&'static str>,
    pub reason: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ExpressionAtom {
    pub text: String,
    pub kind: &'static str,
    pub status: &'static str,
}

const OPERATOR_PATTERNS: &[(&str, &str)] = &[
    (" fails with ", "fails_with"),
    (" returns ", "returns"),
    (" == ", "eq"),
    (" != ", "ne"),
    (" <= ", "le"),
    (" >= ", "ge"),
    (" < ", "lt"),
    (" > ", "gt"),
    (" + ", "add"),
    (" - ", "sub"),
    (" * ", "mul"),
    (" / ", "div"),
    (" and ", "and"),
    (" or ", "or"),
    (" is ", "is"),
    (" does ", "does"),
];

pub fn analyze_expression(text: &str) -> CoreExpressionPreview {
    let text = text.trim();
    if text.is_empty() {
        return expression("unit", "atom_preview_v0", text, vec![], vec![], None);
    }

    if text.ends_with('{') {
        let constructor = text.trim_end_matches('{').trim();
        let atoms = if constructor.is_empty() {
            Vec::new()
        } else {
            vec![classify_atom(constructor)]
        };
        return expression(
            "record_literal_start",
            "contextual_preview_v0",
            text,
            atoms,
            vec![],
            Some("record_literal_context_required"),
        );
    }

    let operators = operators_in_expression(text);
    if !operators.is_empty() {
        let atoms = atoms_from_compound(text);
        let kind = if operators.iter().any(|operator| {
            matches!(
                *operator,
                "is" | "does" | "returns" | "fails_with" | "and" | "or"
            )
        }) {
            "condition_or_surface_binary"
        } else {
            "binary_expression"
        };
        return expression(kind, "compound_preview_v0", text, atoms, operators, None);
    }

    if has_call_shape(text) {
        let status = if text.ends_with(')') {
            "atom_preview_v0"
        } else {
            "surface_phrase_preview_v0"
        };
        let reason = if text.ends_with(')') {
            None
        } else {
            Some("trailing_surface_phrase_after_call")
        };
        return expression("call_like", status, text, call_atoms(text), vec![], reason);
    }

    let atom = classify_atom(text);
    expression(atom.kind, atom.status, text, vec![atom], vec![], None)
}

fn expression(
    kind: &'static str,
    status: &'static str,
    text: &str,
    atoms: Vec<ExpressionAtom>,
    operators: Vec<&'static str>,
    reason: Option<&'static str>,
) -> CoreExpressionPreview {
    CoreExpressionPreview {
        text: text.to_string(),
        kind,
        status,
        atoms,
        operators,
        reason,
    }
}

fn operators_in_expression(text: &str) -> Vec<&'static str> {
    OPERATOR_PATTERNS
        .iter()
        .filter_map(|(pattern, operator)| text.contains(pattern).then_some(*operator))
        .collect()
}

fn atoms_from_compound(text: &str) -> Vec<ExpressionAtom> {
    let mut parts = vec![text.trim()];
    for (pattern, _operator) in OPERATOR_PATTERNS {
        let mut next = Vec::new();
        for part in parts {
            next.extend(
                part.split(pattern)
                    .map(str::trim)
                    .filter(|part| !part.is_empty()),
            );
        }
        parts = next;
    }

    parts.into_iter().map(classify_atom).collect()
}

fn call_atoms(text: &str) -> Vec<ExpressionAtom> {
    let Some((callee, rest)) = text.split_once('(') else {
        return vec![classify_atom(text)];
    };
    let mut atoms = vec![ExpressionAtom {
        text: callee.trim().to_string(),
        kind: "callee_name",
        status: "atom_preview_v0",
    }];

    let args = rest
        .split_once(')')
        .map_or(rest, |(inside, _after)| inside)
        .split(',')
        .map(str::trim)
        .filter(|arg| !arg.is_empty());
    atoms.extend(args.map(classify_atom));
    atoms
}

fn classify_atom(text: &str) -> ExpressionAtom {
    let text = text.trim();
    let (kind, status) = if text.is_empty() {
        ("unit", "atom_preview_v0")
    } else if text == "true" || text == "false" {
        ("bool_literal", "atom_preview_v0")
    } else if text.chars().all(|ch| ch.is_ascii_digit()) {
        ("int_literal", "atom_preview_v0")
    } else if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        ("text_literal", "atom_preview_v0")
    } else if has_call_shape(text) {
        ("call_like", "atom_preview_v0")
    } else if text.contains('.') {
        ("path_or_field_read", "atom_preview_v0")
    } else if is_identifier_like(text) {
        ("name", "atom_preview_v0")
    } else {
        ("surface_text", "surface_phrase_preview_v0")
    };

    ExpressionAtom {
        text: text.to_string(),
        kind,
        status,
    }
}

fn has_call_shape(text: &str) -> bool {
    text.contains('(') && text.contains(')')
}

fn is_identifier_like(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::analyze_expression;

    #[test]
    fn recognizes_literal_name_path_and_record_start_atoms() {
        assert_eq!(analyze_expression("42").kind, "int_literal");
        assert_eq!(analyze_expression("false").kind, "bool_literal");
        assert_eq!(analyze_expression("\"hello\"").kind, "text_literal");
        assert_eq!(analyze_expression("title").kind, "name");
        assert_eq!(
            analyze_expression("clock.now_text").kind,
            "path_or_field_read"
        );

        let record = analyze_expression("WorkItem {");
        assert_eq!(record.kind, "record_literal_start");
        assert_eq!(record.status, "contextual_preview_v0");
        assert_eq!(record.reason, Some("record_literal_context_required"));
    }

    #[test]
    fn recognizes_compound_expression_atoms_and_operators() {
        let expression = analyze_expression("attempts + 1");
        assert_eq!(expression.kind, "binary_expression");
        assert_eq!(expression.status, "compound_preview_v0");
        assert_eq!(expression.operators, vec!["add"]);
        assert_eq!(expression.atoms.len(), 2);
        assert_eq!(expression.atoms[0].kind, "name");
        assert_eq!(expression.atoms[1].kind, "int_literal");

        let condition = analyze_expression("title is empty");
        assert_eq!(condition.kind, "condition_or_surface_binary");
        assert_eq!(condition.operators, vec!["is"]);
    }

    #[test]
    fn recognizes_call_like_expression_atoms() {
        let expression = analyze_expression("remember(\"demo\")");
        assert_eq!(expression.kind, "call_like");
        assert_eq!(expression.status, "atom_preview_v0");
        assert_eq!(expression.atoms.len(), 2);
        assert_eq!(expression.atoms[0].kind, "callee_name");
        assert_eq!(expression.atoms[1].kind, "text_literal");

        let surface = analyze_expression("remember(\"demo\") returns WorkItem");
        assert_eq!(surface.kind, "condition_or_surface_binary");
        assert_eq!(surface.operators, vec!["returns"]);
    }
}
