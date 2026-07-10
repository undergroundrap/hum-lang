use crate::diagnostic::Span;

pub fn source_path_identity(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn file(path: &str) -> String {
    format!("file:{}", component(path))
}

pub fn span(kind: &str, span: &Span, label: &str) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        kind,
        component(&span.file),
        span.line,
        span.column,
        component(label)
    )
}

pub fn line(span: &Span) -> String {
    format!(
        "line:{}:{}:{}",
        component(&span.file),
        span.line,
        span.column
    )
}

fn component(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;

    for ch in value.chars() {
        let mapped = match ch {
            'a'..='z' | '0'..='9' | '/' | '.' | '_' => Some(ch),
            'A'..='Z' => Some(ch.to_ascii_lowercase()),
            '\\' => Some('/'),
            '-' => Some('-'),
            ch if ch.is_ascii_whitespace() => Some('-'),
            ch if ch.is_ascii() => Some('-'),
            _ => None,
        };

        if let Some(ch) = mapped {
            if ch == '-' {
                if !last_dash && !out.is_empty() {
                    out.push('-');
                    last_dash = true;
                }
            } else {
                out.push(ch);
                last_dash = false;
            }
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        "unnamed".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{file, line, source_path_identity, span};
    use crate::diagnostic::Span;

    #[test]
    fn ids_normalize_paths_and_labels() {
        let source_span = Span::new("Examples\\Task List.hum", 12, 1);

        assert_eq!(
            file("Examples\\Task List.hum"),
            "file:examples/task-list.hum"
        );
        assert_eq!(
            span("item", &source_span, "Task Add Item!"),
            "item:examples/task-list.hum:12:1:task-add-item"
        );
        assert_eq!(line(&source_span), "line:examples/task-list.hum:12:1");
        assert_eq!(
            source_path_identity("Examples\\Task List.hum"),
            source_path_identity("Examples/Task List.hum")
        );
    }
}
