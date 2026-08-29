use hum_sha256::digest_hex;

const ACTIVE_MARKER: &str = "<!-- hum-active-workorder:v1 -->\n";
const OWNER: &str = "Owner: BDFL (Ocean).";
const GATE: &str = "## Current authorization gate";
const GATE_END: &str = "<!-- workorder-current-authorization-gate:end -->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusFacts {
    pub base_sha256: String,
    pub immutable_sha256: String,
    pub projected_sha256: String,
    pub bytes: Vec<u8>,
}

fn unique(text: &str, needle: &str) -> Result<usize, String> {
    let mut matches = text.match_indices(needle);
    let first = matches
        .next()
        .ok_or_else(|| format!("missing Work Order boundary `{needle}`"))?;
    if matches.next().is_some() {
        return Err(format!("duplicate Work Order boundary `{needle}`"));
    }
    Ok(first.0)
}

fn spans(text: &str) -> Result<(usize, usize, usize, usize), String> {
    if text
        .lines()
        .any(|line| matches!(line, "<<<<<<<" | "=======" | ">>>>>>>"))
    {
        return Err("Work Order contains a conflict marker".into());
    }
    if text.matches(ACTIVE_MARKER).count() != 1 {
        return Err("active Work Order marker is not unique".into());
    }
    let status = unique(text, "Status:")? + "Status:".len();
    let owner = unique(text, OWNER)?;
    let gate = unique(text, GATE)? + GATE.len();
    let end = unique(text, GATE_END)?;
    if !(status < owner && owner < gate && gate < end) || &text[end + GATE_END.len()..] != "\n" {
        return Err("Work Order mutable-region ordering or final framing is invalid".into());
    }
    Ok((status, owner, gate, end))
}

pub fn immutable_projection(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "Work Order is not UTF-8")?;
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) || bytes.contains(&b'\r') || !text.ends_with('\n') {
        return Err("Work Order framing is not canonical LF UTF-8".into());
    }
    let (status, owner, gate, end) = spans(text)?;
    let before = text[..status].replace(ACTIVE_MARKER, "");
    Ok(format!(
        "{before}<hum-status-header-body>{}<hum-current-gate-body>{}",
        &text[owner..gate],
        &text[end..]
    )
    .into_bytes())
}

pub fn project_status(
    base: &[u8],
    base_sha256: &str,
    status_body: &str,
    gate_body: &str,
) -> Result<StatusFacts, String> {
    if digest_hex(base) != base_sha256 {
        return Err("Work Order base identity mismatch".into());
    }
    if [status_body, gate_body]
        .iter()
        .any(|body| body.contains('\r') || body.contains('\0'))
    {
        return Err("requested mutable body has invalid framing".into());
    }
    let text = std::str::from_utf8(base).map_err(|_| "Work Order is not UTF-8")?;
    let (status, owner, gate, end) = spans(text)?;
    let bytes = format!(
        "{}{}{}{}{}",
        &text[..status],
        status_body,
        &text[owner..gate],
        gate_body,
        &text[end..]
    )
    .into_bytes();
    let before = immutable_projection(base)?;
    let after = immutable_projection(&bytes)?;
    if before != after {
        return Err("status projection changed an immutable Work Order byte".into());
    }
    Ok(StatusFacts {
        base_sha256: base_sha256.into(),
        immutable_sha256: digest_hex(&before),
        projected_sha256: digest_hex(&bytes),
        bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<u8> {
        b"# Work Order 25\n<!-- hum-active-workorder:v1 -->\nStatus: OLD\nOwner: BDFL (Ocean).\n\n## Frozen\nunchanged\n\n## Current authorization gate\n\nold gate\n<!-- workorder-current-authorization-gate:end -->\n".to_vec()
    }

    fn replace_exact_once(input: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
        assert!(!needle.is_empty(), "replacement needle must not be empty");
        let matches = input
            .windows(needle.len())
            .enumerate()
            .filter_map(|(index, value)| (value == needle).then_some(index))
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "replacement needle must occur exactly once"
        );
        let index = matches[0];
        let mut output = Vec::with_capacity(input.len() - needle.len() + replacement.len());
        output.extend_from_slice(&input[..index]);
        output.extend_from_slice(replacement);
        output.extend_from_slice(&input[index + needle.len()..]);
        output
    }

    #[test]
    fn authoritative_active_marker_is_exact_and_fail_closed() {
        let base = sample();
        assert!(immutable_projection(&base).is_ok());
        for (label, value) in [
            (
                "invented marker",
                replace_exact_once(
                    &base,
                    ACTIVE_MARKER.as_bytes(),
                    b"<!-- workorder-active: v1 -->\n",
                ),
            ),
            (
                "missing marker",
                replace_exact_once(&base, ACTIVE_MARKER.as_bytes(), b""),
            ),
            (
                "duplicate marker",
                [ACTIVE_MARKER.as_bytes(), base.as_slice()].concat(),
            ),
            (
                "case substitution",
                replace_exact_once(
                    &base,
                    ACTIVE_MARKER.as_bytes(),
                    b"<!-- Hum-active-workorder:v1 -->\n",
                ),
            ),
            (
                "spacing substitution",
                replace_exact_once(
                    &base,
                    ACTIVE_MARKER.as_bytes(),
                    b"<!-- hum-active-workorder: v1 -->\n",
                ),
            ),
            (
                "version substitution",
                replace_exact_once(
                    &base,
                    ACTIVE_MARKER.as_bytes(),
                    b"<!-- hum-active-workorder:v2 -->\n",
                ),
            ),
        ] {
            assert!(
                immutable_projection(&value).is_err(),
                "{label} earned active-marker credit"
            );
        }
    }

    #[test]
    fn real_active_workorder_projects_only_authenticated_mutable_regions() {
        let base = include_bytes!("../../../workorders/active/WORKORDER_25.md");
        let text = std::str::from_utf8(base).unwrap();
        assert_eq!(text.matches(ACTIVE_MARKER).count(), 1);
        assert!(!text.contains("<!-- workorder-active: v1 -->"));
        let before = immutable_projection(base).unwrap();
        let projected = project_status(
            base,
            &digest_hex(base),
            " AUTHENTICATED PROJECTION TEST\n",
            "\nreview only\n",
        )
        .unwrap();
        assert_eq!(immutable_projection(&projected.bytes).unwrap(), before);
        assert_ne!(projected.bytes, base);
    }

    #[test]
    fn status_facts_touch_only_authenticated_mutable_regions() {
        let base = sample();
        let facts = project_status(&base, &digest_hex(&base), " NEW\n", "\nnew gate\n").unwrap();
        assert_eq!(
            immutable_projection(&base).unwrap(),
            immutable_projection(&facts.bytes).unwrap()
        );
        assert!(
            String::from_utf8(facts.bytes.clone())
                .unwrap()
                .contains("Status: NEW\n")
        );
        for corrupt in ["wrong base", "immutable", "duplicate", "framing"] {
            let result = match corrupt {
                "wrong base" => project_status(&base, &"0".repeat(64), " NEW\n", "\nnew\n"),
                "immutable" => {
                    let mut value = base.clone();
                    value[0] = b'!';
                    project_status(&value, &digest_hex(&base), " NEW\n", "\nnew\n")
                }
                "duplicate" => {
                    let mut value = base.clone();
                    value.extend_from_slice(b"Status:\n");
                    project_status(&value, &digest_hex(&value), " NEW\n", "\nnew\n")
                }
                _ => project_status(&base, &digest_hex(&base), " NEW\r\n", "\nnew\n"),
            };
            assert!(
                result.is_err(),
                "{corrupt} Work Order projection authenticated"
            );
        }
    }
}
