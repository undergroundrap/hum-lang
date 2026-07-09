pub(crate) fn split_element_place(text: &str) -> Option<(&str, usize)> {
    let text = text.trim();
    let (root, rest) = text.split_once('[')?;
    let index_text = rest.strip_suffix(']')?;
    if rest.contains('[')
        || index_text.is_empty()
        || !index_text.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    let root = root.trim();
    if is_value_ident(root) {
        Some((root, index_text.parse().ok()?))
    } else {
        None
    }
}

pub(crate) fn list_element_type(type_text: &str) -> Option<&str> {
    let text = type_text.trim();
    if let Some(rest) = text.strip_prefix("List ") {
        let element = rest.trim();
        return (!element.is_empty()).then_some(element);
    }
    let inside = text.strip_prefix("List<")?.strip_suffix('>')?.trim();
    (!inside.is_empty()).then_some(inside)
}

fn is_value_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first == '_')
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::{list_element_type, split_element_place};

    #[test]
    fn parses_direct_element_places_only() {
        assert_eq!(split_element_place("items[0]"), Some(("items", 0)));
        assert_eq!(split_element_place("items[12]"), Some(("items", 12)));
        assert_eq!(split_element_place("Items[0]"), None);
        assert_eq!(split_element_place("items[index]"), None);
        assert_eq!(split_element_place("items[0].field"), None);
    }

    #[test]
    fn extracts_list_element_types() {
        assert_eq!(list_element_type("List Text"), Some("Text"));
        assert_eq!(list_element_type("List<Point>"), Some("Point"));
        assert_eq!(list_element_type("Text"), None);
    }
}
