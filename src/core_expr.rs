#[derive(Debug, Clone)]
pub struct CoreExpressionPreview {
    pub text: String,
    pub kind: &'static str,
    pub status: &'static str,
    pub atoms: Vec<ExpressionAtom>,
    pub operators: Vec<&'static str>,
    pub ast: CoreExpressionAstPreview,
    pub reason: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ExpressionAtom {
    pub text: String,
    pub kind: &'static str,
    pub status: &'static str,
}

#[derive(Debug, Clone)]
pub struct CoreExpressionAstPreview {
    pub status: &'static str,
    pub type_status: &'static str,
    pub effect_status: &'static str,
    pub node_count: usize,
    pub root: CoreExpressionNode,
}

#[derive(Debug, Clone)]
pub struct CoreExpressionNode {
    pub id: String,
    pub form: &'static str,
    pub text: String,
    pub operator: Option<&'static str>,
    pub type_status: &'static str,
    pub effect_status: &'static str,
    pub children: Vec<CoreExpressionNode>,
    pub reason: Option<&'static str>,
}

pub const CORE_EXPRESSION_AST_STATUS: &str = "ast_preview_v0";
pub const CORE_EXPRESSION_CONTEXTUAL_AST_STATUS: &str = "contextual_ast_preview_v0";
pub const CORE_EXPRESSION_SURFACE_AST_STATUS: &str = "surface_ast_preview_v0";
pub const CORE_EXPRESSION_TYPE_STATUS: &str = "not_type_checked_v0";
pub const CORE_EXPRESSION_EFFECT_STATUS: &str = "not_effect_checked_v0";

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
    let ast = build_ast_preview(kind, status, text, &atoms, &operators, reason);
    CoreExpressionPreview {
        text: text.to_string(),
        kind,
        status,
        atoms,
        operators,
        ast,
        reason,
    }
}

fn build_ast_preview(
    kind: &'static str,
    status: &'static str,
    text: &str,
    atoms: &[ExpressionAtom],
    operators: &[&'static str],
    reason: Option<&'static str>,
) -> CoreExpressionAstPreview {
    let root = if !operators.is_empty() {
        compound_node(text, atoms, operators)
    } else if kind == "record_literal_start" {
        branch_node(
            "root",
            "record_construction_candidate",
            text,
            None,
            atoms
                .iter()
                .enumerate()
                .map(|(index, atom)| atom_node(index, atom))
                .collect(),
            reason,
        )
    } else if kind == "call_like" {
        branch_node(
            "root",
            "call_candidate",
            text,
            None,
            atoms
                .iter()
                .enumerate()
                .map(|(index, atom)| atom_node(index, atom))
                .collect(),
            reason,
        )
    } else if let Some(atom) = atoms.first() {
        atom_node(0, atom)
    } else {
        leaf_node("root", "unit_literal", text, reason)
    };

    CoreExpressionAstPreview {
        status: ast_status(status),
        type_status: CORE_EXPRESSION_TYPE_STATUS,
        effect_status: CORE_EXPRESSION_EFFECT_STATUS,
        node_count: count_nodes(&root),
        root,
    }
}

fn compound_node(
    text: &str,
    atoms: &[ExpressionAtom],
    operators: &[&'static str],
) -> CoreExpressionNode {
    let form = if operators.len() == 1 && atoms.len() == 2 {
        "binary_operation_candidate"
    } else {
        "compound_expression_candidate"
    };
    let reason = if operators.len() > 1 {
        Some("operator_precedence_not_resolved")
    } else {
        None
    };
    branch_node(
        "root",
        form,
        text,
        operators.first().copied(),
        atoms
            .iter()
            .enumerate()
            .map(|(index, atom)| atom_node(index, atom))
            .collect(),
        reason,
    )
}

fn atom_node(index: usize, atom: &ExpressionAtom) -> CoreExpressionNode {
    let form = match atom.kind {
        "unit" => "unit_literal",
        "bool_literal" => "bool_literal",
        "int_literal" => "int_literal",
        "text_literal" => "text_literal",
        "callee_name" => "callee_name",
        "name" => "name_ref",
        "path_or_field_read" => "path_or_field_read",
        "call_like" => "call_candidate_atom",
        "surface_text" => "surface_phrase",
        _ => "unknown_atom",
    };
    let reason = if atom.status == "surface_phrase_preview_v0" {
        Some("surface_phrase_not_lowered")
    } else {
        None
    };
    leaf_node(&format!("atom_{index}"), form, &atom.text, reason)
}

fn branch_node(
    prefix: &str,
    form: &'static str,
    text: &str,
    operator: Option<&'static str>,
    children: Vec<CoreExpressionNode>,
    reason: Option<&'static str>,
) -> CoreExpressionNode {
    CoreExpressionNode {
        id: ast_node_id(prefix, text),
        form,
        text: text.to_string(),
        operator,
        type_status: CORE_EXPRESSION_TYPE_STATUS,
        effect_status: CORE_EXPRESSION_EFFECT_STATUS,
        children,
        reason,
    }
}

fn leaf_node(
    prefix: &str,
    form: &'static str,
    text: &str,
    reason: Option<&'static str>,
) -> CoreExpressionNode {
    branch_node(prefix, form, text, None, Vec::new(), reason)
}

fn ast_status(status: &str) -> &'static str {
    match status {
        "contextual_preview_v0" => CORE_EXPRESSION_CONTEXTUAL_AST_STATUS,
        "surface_phrase_preview_v0" => CORE_EXPRESSION_SURFACE_AST_STATUS,
        _ => CORE_EXPRESSION_AST_STATUS,
    }
}

fn count_nodes(node: &CoreExpressionNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

fn ast_node_id(prefix: &str, text: &str) -> String {
    let mut body = snake_identifier(text);
    if body.is_empty() {
        body.push_str("unit");
    }
    if body.len() > 64 {
        body.truncate(64);
        body = body.trim_matches('_').to_string();
    }
    format!("expr_{prefix}_{body}")
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
        assert_eq!(record.ast.status, "contextual_ast_preview_v0");
        assert_eq!(record.ast.root.form, "record_construction_candidate");
        assert_eq!(record.reason, Some("record_literal_context_required"));
    }

    #[test]
    fn recognizes_compound_expression_atoms_operators_and_ast() {
        let expression = analyze_expression("attempts + 1");
        assert_eq!(expression.kind, "binary_expression");
        assert_eq!(expression.status, "compound_preview_v0");
        assert_eq!(expression.operators, vec!["add"]);
        assert_eq!(expression.atoms.len(), 2);
        assert_eq!(expression.atoms[0].kind, "name");
        assert_eq!(expression.atoms[1].kind, "int_literal");
        assert_eq!(expression.ast.status, "ast_preview_v0");
        assert_eq!(expression.ast.node_count, 3);
        assert_eq!(expression.ast.root.form, "binary_operation_candidate");
        assert_eq!(expression.ast.root.operator, Some("add"));
        assert_eq!(expression.ast.root.type_status, "not_type_checked_v0");
        assert_eq!(expression.ast.root.effect_status, "not_effect_checked_v0");

        let condition = analyze_expression("title is empty");
        assert_eq!(condition.kind, "condition_or_surface_binary");
        assert_eq!(condition.operators, vec!["is"]);
        assert_eq!(condition.ast.root.form, "binary_operation_candidate");
    }

    #[test]
    fn recognizes_call_like_expression_atoms_and_ast() {
        let expression = analyze_expression("remember(\"demo\")");
        assert_eq!(expression.kind, "call_like");
        assert_eq!(expression.status, "atom_preview_v0");
        assert_eq!(expression.atoms.len(), 2);
        assert_eq!(expression.atoms[0].kind, "callee_name");
        assert_eq!(expression.atoms[1].kind, "text_literal");
        assert_eq!(expression.ast.root.form, "call_candidate");
        assert_eq!(expression.ast.node_count, 3);

        let surface = analyze_expression("remember(\"demo\") returns WorkItem");
        assert_eq!(surface.kind, "condition_or_surface_binary");
        assert_eq!(surface.operators, vec!["returns"]);
        assert_eq!(surface.ast.root.form, "binary_operation_candidate");
    }
}
