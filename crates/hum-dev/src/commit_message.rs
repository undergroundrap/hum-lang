use std::{fmt, fs, path::Path};

const TYPES: &[&str] = &[
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];
const EXEMPTIONS: &[&str] = &["Merge ", "Revert ", "fixup! ", "squash! "];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectError {
    pub predicate: &'static str,
    pub detail: String,
}

impl SubjectError {
    fn new(predicate: &'static str, detail: impl Into<String>) -> Self {
        Self {
            predicate,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for SubjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.predicate, self.detail)
    }
}

pub fn validate_subject(subject: &str) -> Result<(), SubjectError> {
    if subject.is_empty() {
        return Err(SubjectError::new("subject_nonempty", "subject is empty"));
    }
    if subject
        .bytes()
        .any(|byte| matches!(byte, 0 | b'\r' | b'\n'))
    {
        return Err(SubjectError::new(
            "subject_one_line",
            "subject contains CR, LF, or NUL",
        ));
    }
    if EXEMPTIONS.iter().any(|prefix| {
        subject
            .strip_prefix(prefix)
            .is_some_and(|rest| !rest.is_empty())
    }) {
        return Ok(());
    }
    let open = subject
        .find('(')
        .ok_or_else(|| SubjectError::new("subject_scoped", "scope is required"))?;
    let kind = &subject[..open];
    if !TYPES.contains(&kind) {
        return Err(SubjectError::new(
            "subject_type",
            format!("unknown type `{kind}`"),
        ));
    }
    let close = subject[open + 1..]
        .find(')')
        .map(|index| open + 1 + index)
        .ok_or_else(|| SubjectError::new("subject_scope_close", "scope is not closed"))?;
    let scope = &subject[open + 1..close];
    if scope.is_empty()
        || !scope.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
    {
        return Err(SubjectError::new(
            "subject_scope",
            "scope must match [a-z0-9._-]+",
        ));
    }
    let tail = &subject[close + 1..];
    let summary = tail
        .strip_prefix(": ")
        .or_else(|| tail.strip_prefix("!: "))
        .ok_or_else(|| {
            SubjectError::new("subject_separator", "expected optional ! followed by `: `")
        })?;
    if summary.is_empty() {
        return Err(SubjectError::new("subject_summary", "summary is empty"));
    }
    Ok(())
}

pub fn subject_from_message_bytes(bytes: &[u8]) -> Result<&str, SubjectError> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(SubjectError::new("message_bom", "UTF-8 BOM is forbidden"));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| SubjectError::new("message_utf8", "message is not UTF-8"))?;
    let subject = text.split_once('\n').map_or(text, |(first, _)| first);
    validate_subject(subject)?;
    Ok(subject)
}

pub fn validate_message_file(path: &Path) -> Result<String, SubjectError> {
    let bytes =
        fs::read(path).map_err(|error| SubjectError::new("message_read", error.to_string()))?;
    subject_from_message_bytes(&bytes).map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{subject_from_message_bytes, validate_subject};

    fn decode(value: &str) -> Vec<u8> {
        value
            .replace("<LF>", "\n")
            .replace("<CR>", "\r")
            .replace("<NUL>", "\0")
            .replace("<SP>", " ")
            .into_bytes()
    }

    #[test]
    fn canonical_rule_is_portable_and_exact() {
        let corpus = include_str!("../../../fixtures/evidence/commit_message_cases.v1.txt");
        for (line_number, line) in corpus.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut fields = line.splitn(3, '|');
            let input = fields.next().unwrap();
            let expected = fields.next().unwrap();
            let value = fields.next().unwrap();
            let decoded = decode(value);
            let result = if input == "subject" {
                std::str::from_utf8(&decoded)
                    .map_err(|_| super::SubjectError::new("fixture_utf8", "fixture is not UTF-8"))
                    .and_then(validate_subject)
            } else {
                subject_from_message_bytes(&decoded).map(|_| ())
            };
            assert_eq!(
                result.is_ok(),
                expected == "accept",
                "invalid commit message passes the permanent corpus: line {}: {line}",
                line_number + 1
            );
        }
        let unknown = validate_subject("unknown(scope): summary").unwrap_err();
        assert_eq!(
            unknown.predicate, "subject_type",
            "invalid commit message passes the permanent corpus"
        );
        assert!(subject_from_message_bytes(b"docs(scope): subject\n\nbody\n").is_ok());
    }
}
