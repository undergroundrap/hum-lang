use std::collections::BTreeMap;

use crate::ast::{Item, Program};

pub(crate) type FieldTypeMap = BTreeMap<(String, String), String>;

pub(crate) fn collect_field_types(program: &Program) -> FieldTypeMap {
    let mut fields = BTreeMap::new();
    for file in &program.files {
        collect_item_field_types(&file.items, &mut fields);
    }
    fields
}

pub(crate) fn field_type<'a>(
    fields: &'a FieldTypeMap,
    record_type: &str,
    field_name: &str,
) -> Option<&'a str> {
    fields
        .get(&(name_key(record_type), name_key(field_name)))
        .map(String::as_str)
}

pub(crate) fn split_field_place(text: &str) -> Option<(&str, &str)> {
    let text = text.trim();
    let (root, field) = text.split_once('.')?;
    if field.contains('.') {
        return None;
    }
    let root = root.trim();
    let field = field.trim();
    if is_value_ident(root) && is_value_ident(field) {
        Some((root, field))
    } else {
        None
    }
}

pub(crate) fn name_key(text: &str) -> String {
    text.trim().to_ascii_lowercase()
}

fn collect_item_field_types(items: &[Item], fields: &mut FieldTypeMap) {
    for item in items {
        match item {
            Item::App(app) => collect_item_field_types(&app.items, fields),
            Item::Type(type_def) => {
                for field in &type_def.fields {
                    fields.insert(
                        (name_key(&type_def.name), name_key(&field.name)),
                        field.ty.trim().to_string(),
                    );
                }
            }
            Item::Store(_) | Item::Task(_) | Item::Test(_) => {}
        }
    }
}

fn is_value_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first == '_')
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}
