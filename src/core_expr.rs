use crate::ast::{CanonicalExpression, CanonicalExpressionKind, ParsedBinaryOperator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreExpressionPreview {
    pub text: String,
    pub kind: &'static str,
    pub status: &'static str,
    pub atoms: Vec<ExpressionAtom>,
    pub operators: Vec<&'static str>,
    pub ast: CoreExpressionAstPreview,
    pub reason: Option<&'static str>,
    pub(crate) canonical: CanonicalExpression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionAtom {
    pub text: String,
    pub kind: &'static str,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreExpressionAstPreview {
    pub status: &'static str,
    pub type_status: &'static str,
    pub type_text: Option<String>,
    pub type_source: Option<&'static str>,
    pub effect_status: &'static str,
    pub node_count: usize,
    pub root: CoreExpressionNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreExpressionNode {
    pub id: String,
    pub form: &'static str,
    pub text: String,
    pub operator: Option<&'static str>,
    pub type_status: &'static str,
    pub type_text: Option<String>,
    pub type_source: Option<&'static str>,
    pub effect_status: &'static str,
    pub children: Vec<CoreExpressionNode>,
    pub reason: Option<&'static str>,
}

pub const CORE_EXPRESSION_AST_STATUS: &str = "ast_preview_v0";
pub const CORE_EXPRESSION_CONTEXTUAL_AST_STATUS: &str = "contextual_ast_preview_v0";
pub const CORE_EXPRESSION_SURFACE_AST_STATUS: &str = "surface_ast_preview_v0";
pub const CORE_EXPRESSION_TYPE_STATUS: &str = "not_type_checked_v0";
pub const CORE_EXPRESSION_CHECKED_TRIVIAL_RETURN_TYPE_STATUS: &str =
    "checked_trivial_return_type_v0";
pub const CORE_EXPRESSION_CHECKED_TRIVIAL_RETURN_MISMATCH_STATUS: &str =
    "checked_trivial_return_type_mismatch_v0";
pub const CORE_EXPRESSION_EFFECT_STATUS: &str = "not_effect_checked_v0";
pub const CORE_PREDICATE_EXPRESSION_STATUS: &str = "typed_predicate_ast_v2";
pub const CORE_PREDICATE_AST_STATUS: &str = "predicate_ast_v2";
pub const CORE_PREDICATE_TYPE_STATUS: &str = "predicate_v2_typed_v0";
pub const CORE_PREDICATE_EFFECT_STATUS: &str = "contract_only_pure_v0";

pub fn analyze_canonical_expression(
    canonical: &CanonicalExpression,
    text: &str,
    legacy_kind: Option<&'static str>,
) -> CoreExpressionPreview {
    let kind = public_kind(canonical, legacy_kind);
    let status = match kind {
        "record_literal_start" => "contextual_preview_v0",
        "unsupported_try_expression" | "surface_text" => "surface_phrase_preview_v0",
        "binary_expression" | "condition_or_surface_binary" => "compound_preview_v0",
        _ => "atom_preview_v0",
    };
    let mut atoms = Vec::new();
    collect_canonical_atoms(canonical, false, &mut atoms);
    let mut operators = Vec::new();
    collect_canonical_operators(canonical, &mut operators);
    let reason = match kind {
        "record_literal_start" => Some("record_literal_context_required"),
        "unsupported_try_expression" => Some("unsupported_try_expression_shape_v0"),
        _ => None,
    };
    let ast = build_ast_preview(kind, status, text, &atoms, &operators, reason);
    CoreExpressionPreview {
        text: text.to_string(),
        kind,
        status,
        atoms,
        operators,
        ast,
        reason,
        canonical: canonical.clone(),
    }
}

pub(crate) fn validate_private_canonical_projection(
    observed: &CanonicalExpression,
    expected: &CanonicalExpression,
) -> Result<(), &'static str> {
    if observed != expected {
        return Err("core_private_expression_tree_corrupt_v0");
    }
    crate::parser::validate_canonical_expression_for_consumer(observed)
}

fn canonical_kind(expression: &CanonicalExpression) -> &'static str {
    match &expression.kind {
        CanonicalExpressionKind::Unit => "unit",
        CanonicalExpressionKind::UIntLiteral(_) | CanonicalExpressionKind::IntLiteral(_) => {
            "int_literal"
        }
        CanonicalExpressionKind::BoolLiteral(_) => "bool_literal",
        CanonicalExpressionKind::TextLiteral(_) => "text_literal",
        CanonicalExpressionKind::ListLiteral(_) => "list_literal",
        CanonicalExpressionKind::RecordLiteral { fields, .. } if fields.is_empty() => {
            "record_literal_start"
        }
        CanonicalExpressionKind::RecordLiteral { .. } => "record_literal",
        CanonicalExpressionKind::Call { .. } | CanonicalExpressionKind::Try { .. } => "call_like",
        CanonicalExpressionKind::Field { .. } | CanonicalExpressionKind::Element { .. } => {
            "path_or_field_read"
        }
        CanonicalExpressionKind::Identifier(_) => "name",
        CanonicalExpressionKind::Permission { value, .. }
        | CanonicalExpressionKind::Group(value) => canonical_kind(value),
        CanonicalExpressionKind::Binary { operator, .. } => {
            if matches!(
                operator,
                ParsedBinaryOperator::Is
                    | ParsedBinaryOperator::Does
                    | ParsedBinaryOperator::Returns
                    | ParsedBinaryOperator::FailsWith
                    | ParsedBinaryOperator::And
                    | ParsedBinaryOperator::Or
            ) {
                "condition_or_surface_binary"
            } else {
                "binary_expression"
            }
        }
        CanonicalExpressionKind::Unsupported => "surface_text",
    }
}

fn public_kind(
    expression: &CanonicalExpression,
    legacy_kind: Option<&'static str>,
) -> &'static str {
    match legacy_kind {
        Some("condition_text") => "condition_or_surface_binary",
        Some("path_or_name") => "path_or_field_read",
        Some("name_or_text") => "surface_text",
        Some("try_call_like") => "try_call_like",
        Some(kind) => kind,
        None => match expression.kind {
            CanonicalExpressionKind::Try { .. } => "try_call_like",
            _ => canonical_kind(expression),
        },
    }
}

fn collect_canonical_atoms(
    expression: &CanonicalExpression,
    compound_context: bool,
    atoms: &mut Vec<ExpressionAtom>,
) {
    match &expression.kind {
        CanonicalExpressionKind::Binary { left, right, .. } => {
            collect_canonical_atoms(left, true, atoms);
            collect_canonical_atoms(right, true, atoms);
        }
        CanonicalExpressionKind::Group(value)
        | CanonicalExpressionKind::Permission { value, .. } => {
            collect_canonical_atoms(value, compound_context, atoms);
        }
        CanonicalExpressionKind::Try { call, .. } => collect_canonical_atoms(call, false, atoms),
        CanonicalExpressionKind::Call { callee, arguments } => {
            if compound_context {
                atoms.push(classify_canonical_atom(expression));
                return;
            }
            atoms.push(ExpressionAtom {
                text: canonical_text(callee),
                kind: "callee_name",
                status: "atom_preview_v0",
            });
            for argument in arguments {
                collect_canonical_atoms(argument, false, atoms);
            }
        }
        _ => atoms.push(classify_canonical_atom(expression)),
    }
}

fn collect_canonical_operators(
    expression: &CanonicalExpression,
    operators: &mut Vec<&'static str>,
) {
    match &expression.kind {
        CanonicalExpressionKind::Binary {
            operator,
            left,
            right,
            ..
        } => {
            collect_canonical_operators(left, operators);
            operators.push(operator_name(*operator));
            collect_canonical_operators(right, operators);
        }
        CanonicalExpressionKind::Group(value)
        | CanonicalExpressionKind::Permission { value, .. } => {
            collect_canonical_operators(value, operators);
        }
        CanonicalExpressionKind::Try { call, .. } => {
            collect_canonical_operators(call, operators);
        }
        CanonicalExpressionKind::Call { callee, arguments } => {
            collect_canonical_operators(callee, operators);
            for argument in arguments {
                collect_canonical_operators(argument, operators);
            }
        }
        CanonicalExpressionKind::ListLiteral(values) => {
            for value in values {
                collect_canonical_operators(value, operators);
            }
        }
        CanonicalExpressionKind::RecordLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_canonical_operators(value, operators);
            }
        }
        CanonicalExpressionKind::Field { base, .. }
        | CanonicalExpressionKind::Element { base, .. } => {
            collect_canonical_operators(base, operators);
        }
        CanonicalExpressionKind::Unit
        | CanonicalExpressionKind::Identifier(_)
        | CanonicalExpressionKind::UIntLiteral(_)
        | CanonicalExpressionKind::IntLiteral(_)
        | CanonicalExpressionKind::BoolLiteral(_)
        | CanonicalExpressionKind::TextLiteral(_)
        | CanonicalExpressionKind::Unsupported => {}
    }
}

fn classify_canonical_atom(expression: &CanonicalExpression) -> ExpressionAtom {
    let kind = match &expression.kind {
        CanonicalExpressionKind::Unit => "unit",
        CanonicalExpressionKind::BoolLiteral(_) => "bool_literal",
        CanonicalExpressionKind::UIntLiteral(_) | CanonicalExpressionKind::IntLiteral(_) => {
            "int_literal"
        }
        CanonicalExpressionKind::TextLiteral(_) => "text_literal",
        CanonicalExpressionKind::Call { .. } | CanonicalExpressionKind::Try { .. } => "call_like",
        CanonicalExpressionKind::Field { .. } | CanonicalExpressionKind::Element { .. } => {
            "path_or_field_read"
        }
        CanonicalExpressionKind::Identifier(_) => "name",
        CanonicalExpressionKind::ListLiteral(_)
        | CanonicalExpressionKind::RecordLiteral { .. }
        | CanonicalExpressionKind::Permission { .. }
        | CanonicalExpressionKind::Binary { .. }
        | CanonicalExpressionKind::Group(_) => "surface_text",
        CanonicalExpressionKind::Unsupported => "surface_text",
    };
    ExpressionAtom {
        text: canonical_text(expression),
        kind,
        status: if kind == "surface_text" {
            "surface_phrase_preview_v0"
        } else {
            "atom_preview_v0"
        },
    }
}

pub(crate) fn canonical_text(expression: &CanonicalExpression) -> String {
    match &expression.kind {
        CanonicalExpressionKind::Unit => String::new(),
        CanonicalExpressionKind::Identifier(name) => name.clone(),
        CanonicalExpressionKind::Field { base, field } => {
            format!("{}.{}", canonical_text(base), field)
        }
        CanonicalExpressionKind::Element { base, index } => {
            format!("{}[{index}]", canonical_text(base))
        }
        CanonicalExpressionKind::UIntLiteral(value) => value.to_string(),
        CanonicalExpressionKind::IntLiteral(value) => value.to_string(),
        CanonicalExpressionKind::BoolLiteral(value) => value.to_string(),
        CanonicalExpressionKind::TextLiteral(value) => format!("\"{value}\""),
        CanonicalExpressionKind::ListLiteral(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CanonicalExpressionKind::RecordLiteral { name, fields } => {
            if fields.is_empty() {
                format!("{name} {{")
            } else {
                format!(
                    "{name} {{{}}}",
                    fields
                        .iter()
                        .map(|(field, value)| format!("{field}: {}", canonical_text(value)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        CanonicalExpressionKind::Call { callee, arguments } => format!(
            "{}({})",
            canonical_text(callee),
            arguments
                .iter()
                .map(canonical_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CanonicalExpressionKind::Permission { permission, value } => {
            let permission = match permission {
                crate::ast::ParamPermission::Borrow => "borrow",
                crate::ast::ParamPermission::Change => "change",
                crate::ast::ParamPermission::Consume => "consume",
            };
            format!("{permission} {}", canonical_text(value))
        }
        CanonicalExpressionKind::Try { call, wrapper } => wrapper.as_ref().map_or_else(
            || format!("try {}", canonical_text(call)),
            |wrapper| format!("try {} or fail {wrapper}", canonical_text(call)),
        ),
        CanonicalExpressionKind::Binary {
            operator,
            left,
            right,
            ..
        } => format!(
            "{} {} {}",
            canonical_text(left),
            operator_spelling(*operator),
            canonical_text(right)
        ),
        CanonicalExpressionKind::Group(value) => format!("({})", canonical_text(value)),
        CanonicalExpressionKind::Unsupported => "<unsupported>".to_string(),
    }
}

#[cfg(test)]
pub(crate) fn canonical_semantic_shape(expression: &CanonicalExpression) -> String {
    match &expression.kind {
        CanonicalExpressionKind::Unit => "unit".to_string(),
        CanonicalExpressionKind::Identifier(name) => format!("id({name})"),
        CanonicalExpressionKind::Field { base, field } => {
            format!("field({},{field})", canonical_semantic_shape(base))
        }
        CanonicalExpressionKind::Element { base, index } => {
            format!("element({},{index})", canonical_semantic_shape(base))
        }
        CanonicalExpressionKind::UIntLiteral(value) => format!("uint({value})"),
        CanonicalExpressionKind::IntLiteral(value) => format!("int({value})"),
        CanonicalExpressionKind::BoolLiteral(value) => format!("bool({value})"),
        CanonicalExpressionKind::TextLiteral(value) => format!("text({value:?})"),
        CanonicalExpressionKind::ListLiteral(values) => format!(
            "list({})",
            values
                .iter()
                .map(canonical_semantic_shape)
                .collect::<Vec<_>>()
                .join(",")
        ),
        CanonicalExpressionKind::RecordLiteral { name, fields } => format!(
            "record({name};{})",
            fields
                .iter()
                .map(|(field, value)| { format!("{field}={}", canonical_semantic_shape(value)) })
                .collect::<Vec<_>>()
                .join(",")
        ),
        CanonicalExpressionKind::Call { callee, arguments } => format!(
            "call({};{})",
            canonical_semantic_shape(callee),
            arguments
                .iter()
                .map(canonical_semantic_shape)
                .collect::<Vec<_>>()
                .join(",")
        ),
        CanonicalExpressionKind::Permission { permission, value } => {
            format!(
                "permission({};{})",
                permission.as_str(),
                canonical_semantic_shape(value)
            )
        }
        CanonicalExpressionKind::Try { call, wrapper } => format!(
            "try({};{})",
            canonical_semantic_shape(call),
            wrapper.as_deref().unwrap_or("none")
        ),
        CanonicalExpressionKind::Binary {
            operator,
            left,
            right,
            ..
        } => format!(
            "binary({};{};{})",
            operator_name(*operator),
            canonical_semantic_shape(left),
            canonical_semantic_shape(right)
        ),
        CanonicalExpressionKind::Group(value) => {
            format!("group({})", canonical_semantic_shape(value))
        }
        CanonicalExpressionKind::Unsupported => "unsupported".to_string(),
    }
}

pub(crate) const fn operator_name(operator: ParsedBinaryOperator) -> &'static str {
    match operator {
        ParsedBinaryOperator::Multiply => "mul",
        ParsedBinaryOperator::Divide => "div",
        ParsedBinaryOperator::Add => "add",
        ParsedBinaryOperator::Subtract => "sub",
        ParsedBinaryOperator::Equal => "eq",
        ParsedBinaryOperator::NotEqual => "ne",
        ParsedBinaryOperator::Less => "lt",
        ParsedBinaryOperator::LessEqual => "le",
        ParsedBinaryOperator::Greater => "gt",
        ParsedBinaryOperator::GreaterEqual => "ge",
        ParsedBinaryOperator::Is => "is",
        ParsedBinaryOperator::Does => "does",
        ParsedBinaryOperator::Returns => "returns",
        ParsedBinaryOperator::FailsWith => "fails_with",
        ParsedBinaryOperator::And => "and",
        ParsedBinaryOperator::Or => "or",
    }
}

pub(crate) const fn operator_spelling(operator: ParsedBinaryOperator) -> &'static str {
    match operator {
        ParsedBinaryOperator::Multiply => "*",
        ParsedBinaryOperator::Divide => "/",
        ParsedBinaryOperator::Add => "+",
        ParsedBinaryOperator::Subtract => "-",
        ParsedBinaryOperator::Equal => "==",
        ParsedBinaryOperator::NotEqual => "!=",
        ParsedBinaryOperator::Less => "<",
        ParsedBinaryOperator::LessEqual => "<=",
        ParsedBinaryOperator::Greater => ">",
        ParsedBinaryOperator::GreaterEqual => ">=",
        ParsedBinaryOperator::Is => "is",
        ParsedBinaryOperator::Does => "does",
        ParsedBinaryOperator::Returns => "returns",
        ParsedBinaryOperator::FailsWith => "fails with",
        ParsedBinaryOperator::And => "and",
        ParsedBinaryOperator::Or => "or",
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
    } else if matches!(kind, "call_like" | "try_call_like") {
        branch_node(
            "root",
            if kind == "try_call_like" {
                "try_call_candidate"
            } else {
                "call_candidate"
            },
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
        type_text: None,
        type_source: None,
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
        type_text: None,
        type_source: None,
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

pub fn annotate_expression_type(
    preview: &mut CoreExpressionPreview,
    type_status: &'static str,
    type_text: Option<&str>,
    type_source: Option<&'static str>,
) {
    preview.ast.type_status = type_status;
    preview.ast.type_text = type_text.map(str::to_string);
    preview.ast.type_source = type_source;
    preview.ast.root.type_status = type_status;
    preview.ast.root.type_text = type_text.map(str::to_string);
    preview.ast.root.type_source = type_source;
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
    use crate::ast::{Item, ParsedBodyStatementKind};
    use crate::parser::parse_source;

    use super::analyze_canonical_expression;

    fn analyze(text: &str) -> super::CoreExpressionPreview {
        let source = format!("task demo() -> UInt {{\n  does:\n    return {text}\n}}\n");
        let parsed = parse_source("core-expression.hum", &source);
        let Item::Task(task) = &parsed.file.items[0] else {
            panic!("task")
        };
        let ParsedBodyStatementKind::Return(expression) = &task.body_syntax[0].kind else {
            panic!("return")
        };
        analyze_canonical_expression(&expression.canonical, text, None)
    }

    #[test]
    fn recognizes_literal_name_path_and_record_start_atoms() {
        assert_eq!(analyze("42").kind, "int_literal");
        assert_eq!(analyze("false").kind, "bool_literal");
        assert_eq!(analyze("\"hello\"").kind, "text_literal");
        assert_eq!(analyze("title").kind, "name");
        assert_eq!(analyze("clock.now_text").kind, "path_or_field_read");
    }

    #[test]
    fn recognizes_compound_expression_atoms_operators_and_ast() {
        let expression = analyze("attempts + 1");
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
        assert_eq!(expression.ast.root.type_text, None);
        assert_eq!(expression.ast.root.type_source, None);
        assert_eq!(expression.ast.root.effect_status, "not_effect_checked_v0");

        let condition = analyze("title is empty");
        assert_eq!(condition.kind, "condition_or_surface_binary");
        assert_eq!(condition.operators, vec!["is"]);
        assert_eq!(condition.ast.root.form, "binary_operation_candidate");
    }

    #[test]
    fn recognizes_call_like_expression_atoms_and_ast() {
        let expression = analyze("remember(\"demo\")");
        assert_eq!(expression.kind, "call_like");
        assert_eq!(expression.status, "atom_preview_v0");
        assert_eq!(expression.atoms.len(), 2);
        assert_eq!(expression.atoms[0].kind, "callee_name");
        assert_eq!(expression.atoms[1].kind, "text_literal");
        assert_eq!(expression.ast.root.form, "call_candidate");
        assert_eq!(expression.ast.node_count, 3);

        let surface = analyze("remember(\"demo\") returns WorkItem");
        assert_eq!(surface.kind, "condition_or_surface_binary");
        assert_eq!(surface.operators, vec!["returns"]);
        assert_eq!(surface.ast.root.form, "binary_operation_candidate");
    }

    #[test]
    fn recognizes_try_call_without_inventing_a_try_callee() {
        let expression = analyze("try load(7) or fail OuterError.context");
        assert_eq!(expression.kind, "try_call_like");
        assert_eq!(expression.atoms[0].text, "load");
        assert_eq!(expression.ast.root.form, "try_call_candidate");
    }
}
