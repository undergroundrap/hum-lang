use crate::command::EvidenceProfile;
use hum_sha256::digest_hex;

pub const SCHEMA: &str = "hum.evidence_summary.v1";
pub const POLICY: &str = "wo25.unit_a.v1";
const SELECTORS: &[&str] = &[
    "commit_message::tests::canonical_rule_is_portable_and_exact",
    "identity::tests::candidate_identity_binds_commit_tree_index_and_paths",
    "summary::tests::evidence_summary_v1_is_canonical_and_hash_bound",
    "cleanup::tests::owned_resources_close_on_every_controlled_terminal_path",
    "command::tests::evidence_profiles_are_typed_and_fail_closed",
    "cli::legacy_equivalence_preserves_exit_stages_and_stream_hashes",
];
const MUTATIONS: &[&str] = &["I01", "I02", "I03", "I04"];
const STAGES: &[&str] = &["selectors", "mutations", "legacy_equivalence", "cleanup"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageDisposition {
    Passed,
}
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)] pub struct StageRecord { name: String, disposition: StageDisposition, skip_reason: Option<String>, skip_predicate: Option<String>, binding: String, evidence_sha256: String }
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)] pub struct MutationRecord { id: String, result: String, restored_sha256: String }
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)] pub struct EvidenceSummary { generator: String, commit: String, tree: String, candidate_manifest: String, platform: String, target: String, toolchain: String, profile: EvidenceProfile, invocation_binding: String, selectors: Vec<String>, selector_stream_sha256: String, mutations: Vec<MutationRecord>, mutation_stream_sha256: String, expected_stages: Vec<String>, stages: Vec<StageRecord>, terminal: String, exit: i32, stdout_sha256: String, stderr_sha256: String, event_sha256: String, cleanup: String }

#[rustfmt::skip]
pub(crate) fn quoted(out: &mut String, value: &str) {
    out.push('"'); for character in value.chars() { match character { '"' => out.push_str("\\\""), '\\' => out.push_str("\\\\"), '\n' => out.push_str("\\n"), '\r' => out.push_str("\\r"), '\t' => out.push_str("\\t"), c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)), c => out.push(c) }} out.push('"');
}
#[rustfmt::skip]
pub(crate) fn string_array(out: &mut String, values: &[String]) { out.push('['); for (index, value) in values.iter().enumerate() { if index != 0 { out.push(','); } quoted(out, value); } out.push(']'); }
fn hash_is_exact(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn stream(values: impl IntoIterator<Item = String>) -> String {
    values.into_iter().map(|value| value + "\n").collect()
}
fn selector_hash(values: &[String]) -> String {
    digest_hex(stream(values.iter().cloned()).as_bytes())
}
fn mutation_hash(values: &[MutationRecord]) -> String {
    digest_hex(
        stream(
            values
                .iter()
                .map(|m| format!("{}|{}|{}", m.id, m.result, m.restored_sha256)),
        )
        .as_bytes(),
    )
}
fn invocation_hash(values: &[&str]) -> String {
    digest_hex(stream(values.iter().map(|value| (*value).to_string())).as_bytes())
}
fn stage_hash(name: &str, binding: &str) -> String {
    digest_hex(format!("{name}|passed|{binding}\n").as_bytes())
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
#[rustfmt::skip]
impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self { Self { bytes, at: 0 } }
    fn expect(&mut self, value: &[u8]) -> Result<(), String> { if self.bytes.get(self.at..self.at + value.len()) != Some(value) { return Err(format!("summary syntax mismatch at byte {}", self.at)); } self.at += value.len(); Ok(()) }
    fn string(&mut self) -> Result<String, String> {
        self.expect(b"\"")?; let mut out = Vec::new();
        loop { let byte = *self.bytes.get(self.at).ok_or("unterminated summary string")?; self.at += 1; match byte {
            b'"' => return String::from_utf8(out).map_err(|_| "summary string is not UTF-8".into()),
            b'\\' => { let escaped = *self.bytes.get(self.at).ok_or("unterminated summary escape")?; self.at += 1; match escaped { b'"' | b'\\' | b'/' => out.push(escaped), b'n' => out.push(b'\n'), b'r' => out.push(b'\r'), b't' => out.push(b'\t'), b'u' => { let digits = self.bytes.get(self.at..self.at + 4).ok_or("short Unicode escape")?; self.at += 4; let text = std::str::from_utf8(digits).map_err(|_| "Unicode escape is not ASCII")?; let value = u32::from_str_radix(text, 16).map_err(|_| "invalid Unicode escape")?; let character = char::from_u32(value).ok_or("invalid Unicode scalar")?; let mut encoded = [0; 4]; out.extend_from_slice(character.encode_utf8(&mut encoded).as_bytes()); }, _ => return Err("unsupported summary escape".into()) } },
            0..=31 => return Err("raw control byte in summary string".into()), value => out.push(value),
        }}
    }
    fn optional_string(&mut self) -> Result<Option<String>, String> { if self.bytes.get(self.at..self.at + 4) == Some(b"null") { self.at += 4; Ok(None) } else { self.string().map(Some) } }
    fn number(&mut self) -> Result<i32, String> { let start = self.at; if self.bytes.get(self.at) == Some(&b'-') { self.at += 1; } while self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) { self.at += 1; } std::str::from_utf8(&self.bytes[start..self.at]).map_err(|_| "number is not ASCII")?.parse().map_err(|_| "invalid summary number".into()) }
    fn strings(&mut self) -> Result<Vec<String>, String> { self.expect(b"[")?; let mut out = Vec::new(); if self.bytes.get(self.at) == Some(&b']') { self.at += 1; return Ok(out); } loop { out.push(self.string()?); match self.bytes.get(self.at) { Some(b',') => self.at += 1, Some(b']') => { self.at += 1; return Ok(out); }, _ => return Err("invalid string array delimiter".into()) } } }
    fn mutations(&mut self) -> Result<Vec<MutationRecord>, String> { self.expect(b"[")?; let mut out = Vec::new(); if self.bytes.get(self.at) == Some(&b']') { self.at += 1; return Ok(out); } loop { self.expect(b"{\"id\":")?; let id = self.string()?; self.expect(b",\"result\":")?; let result = self.string()?; self.expect(b",\"restored_sha256\":")?; let restored_sha256 = self.string()?; self.expect(b"}")?; out.push(MutationRecord { id, result, restored_sha256 }); match self.bytes.get(self.at) { Some(b',') => self.at += 1, Some(b']') => { self.at += 1; return Ok(out); }, _ => return Err("invalid mutation array delimiter".into()) } } }
    fn stages(&mut self) -> Result<Vec<StageRecord>, String> { self.expect(b"[")?; let mut out = Vec::new(); if self.bytes.get(self.at) == Some(&b']') { self.at += 1; return Ok(out); } loop { self.expect(b"{\"name\":")?; let name = self.string()?; self.expect(b",\"disposition\":")?; if self.string()? != "passed" { return Err("unknown stage disposition".into()); } self.expect(b",\"skip_reason\":")?; let skip_reason = self.optional_string()?; self.expect(b",\"skip_predicate\":")?; let skip_predicate = self.optional_string()?; self.expect(b",\"binding\":")?; let binding = self.string()?; self.expect(b",\"evidence_sha256\":")?; let evidence_sha256 = self.string()?; self.expect(b"}")?; out.push(StageRecord { name, disposition: StageDisposition::Passed, skip_reason, skip_predicate, binding, evidence_sha256 }); match self.bytes.get(self.at) { Some(b',') => self.at += 1, Some(b']') => { self.at += 1; return Ok(out); }, _ => return Err("invalid stage array delimiter".into()) } } }
}

#[rustfmt::skip]
impl EvidenceSummary {
    pub fn validate_stage_closure(&self) -> Result<(), String> {
        if self.profile != EvidenceProfile::Focused { return Err("Unit A can authenticate successful focused summaries only".into()); }
        let binding = invocation_hash(&[POLICY, &self.generator, &self.commit, &self.tree, &self.candidate_manifest, &self.platform, &self.target, &self.toolchain, self.profile.name()]); if self.invocation_binding != binding { return Err("invocation binding mismatch".into()); }
        if self.selectors.iter().map(String::as_str).collect::<Vec<_>>() != SELECTORS { return Err("profile selector membership mismatch".into()); }
        if self.selector_stream_sha256 != selector_hash(&self.selectors) { return Err("selector stream hash mismatch".into()); }
        if self.mutations.iter().map(|m| m.id.as_str()).collect::<Vec<_>>() != MUTATIONS { return Err("profile mutation membership mismatch".into()); }
        if self.mutations.iter().any(|m| m.result != "rejected" || !hash_is_exact(&m.restored_sha256)) { return Err("mutation result or restoration identity mismatch".into()); }
        if self.mutation_stream_sha256 != mutation_hash(&self.mutations) { return Err("mutation stream hash mismatch".into()); }
        if self.expected_stages.iter().map(String::as_str).collect::<Vec<_>>() != STAGES { return Err("profile expected-stage set mismatch".into()); }
        if self.stages.len() != STAGES.len() { return Err("stage set cardinality mismatch".into()); }
        for (index, expected) in STAGES.iter().enumerate() { let stage = &self.stages[index]; if stage.name != *expected { return Err(format!("stage {index} is not `{expected}`")); }
            if stage.binding != self.invocation_binding || stage.evidence_sha256 != stage_hash(expected, &self.invocation_binding) { return Err(format!("stage `{expected}` has mixed or unauthenticated evidence")); }
            if stage.disposition != StageDisposition::Passed || stage.skip_reason.is_some() || stage.skip_predicate.is_some() { return Err(format!("stage `{expected}` is not one authenticated pass")); } }
        if self.invocation_binding.is_empty() || self.terminal != "success" || self.exit != 0 || self.cleanup != "closed" { return Err("overall success contradicts owned evidence".into()); }
        if [&self.stdout_sha256, &self.stderr_sha256, &self.event_sha256].iter().any(|value| !hash_is_exact(value)) { return Err("stream identity is malformed".into()); } Ok(())
    }
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate_stage_closure()?; let mut out = String::from("{\"schema\":\""); out.push_str(SCHEMA); out.push_str("\",\"policy\":\""); out.push_str(POLICY); out.push_str("\",\"generator\":"); quoted(&mut out, &self.generator);
        for (name, value) in [("commit", &self.commit), ("tree", &self.tree), ("candidate_manifest", &self.candidate_manifest), ("platform", &self.platform), ("target", &self.target), ("toolchain", &self.toolchain)] { out.push_str(",\""); out.push_str(name); out.push_str("\":"); quoted(&mut out, value); }
        out.push_str(",\"profile\":"); quoted(&mut out, self.profile.name()); out.push_str(",\"invocation_binding\":"); quoted(&mut out, &self.invocation_binding); out.push_str(",\"selectors\":"); string_array(&mut out, &self.selectors); out.push_str(",\"selector_stream_sha256\":"); quoted(&mut out, &self.selector_stream_sha256);
        out.push_str(",\"mutations\":["); for (index, mutation) in self.mutations.iter().enumerate() { if index != 0 { out.push(','); } out.push_str("{\"id\":"); quoted(&mut out, &mutation.id); out.push_str(",\"result\":"); quoted(&mut out, &mutation.result); out.push_str(",\"restored_sha256\":"); quoted(&mut out, &mutation.restored_sha256); out.push('}'); }
        out.push_str("],\"mutation_stream_sha256\":"); quoted(&mut out, &self.mutation_stream_sha256); out.push_str(",\"expected_stages\":"); string_array(&mut out, &self.expected_stages); out.push_str(",\"stages\":[");
        for (index, stage) in self.stages.iter().enumerate() { if index != 0 { out.push(','); } out.push_str("{\"name\":"); quoted(&mut out, &stage.name); out.push_str(",\"disposition\":\"passed\",\"skip_reason\":null,\"skip_predicate\":null,\"binding\":"); quoted(&mut out, &stage.binding); out.push_str(",\"evidence_sha256\":"); quoted(&mut out, &stage.evidence_sha256); out.push('}'); }
        out.push_str("],\"terminal\":"); quoted(&mut out, &self.terminal); out.push_str(&format!(",\"exit\":{}", self.exit)); for (name, value) in [("stdout_sha256", &self.stdout_sha256), ("stderr_sha256", &self.stderr_sha256), ("event_sha256", &self.event_sha256), ("cleanup", &self.cleanup)] { out.push_str(",\""); out.push_str(name); out.push_str("\":"); quoted(&mut out, value); } out.push_str("}\n"); Ok(out.into_bytes())
    }
    pub fn parse_canonical(bytes: &[u8]) -> Result<Self, String> {
        if bytes.starts_with(&[0xef, 0xbb, 0xbf]) || bytes.last() != Some(&b'\n') || bytes.contains(&b'\r') { return Err("summary framing is not canonical UTF-8/LF".into()); } std::str::from_utf8(bytes).map_err(|_| "summary is not UTF-8")?; let mut reader = Reader::new(bytes); macro_rules! field { ($name:literal) => {{ reader.expect(concat!(",\"", $name, "\":").as_bytes())?; reader.string()? }}; }
        reader.expect(b"{\"schema\":")?; if reader.string()? != SCHEMA { return Err("unknown summary schema".into()); } reader.expect(b",\"policy\":")?; if reader.string()? != POLICY { return Err("unknown summary policy".into()); }
        let generator = field!("generator"); let commit = field!("commit"); let tree = field!("tree"); let candidate_manifest = field!("candidate_manifest"); let platform = field!("platform"); let target = field!("target"); let toolchain = field!("toolchain"); let profile = EvidenceProfile::parse(&field!("profile"))?; let invocation_binding = field!("invocation_binding");
        reader.expect(b",\"selectors\":")?; let selectors = reader.strings()?; let selector_stream_sha256 = field!("selector_stream_sha256"); reader.expect(b",\"mutations\":")?; let mutations = reader.mutations()?; let mutation_stream_sha256 = field!("mutation_stream_sha256"); reader.expect(b",\"expected_stages\":")?; let expected_stages = reader.strings()?; reader.expect(b",\"stages\":")?; let stages = reader.stages()?; let terminal = field!("terminal"); reader.expect(b",\"exit\":")?; let exit = reader.number()?; let stdout_sha256 = field!("stdout_sha256"); let stderr_sha256 = field!("stderr_sha256"); let event_sha256 = field!("event_sha256"); let cleanup = field!("cleanup"); reader.expect(b"}\n")?; if reader.at != bytes.len() { return Err("trailing summary bytes".into()); }
        let summary = Self { generator, commit, tree, candidate_manifest, platform, target, toolchain, profile, invocation_binding, selectors, selector_stream_sha256, mutations, mutation_stream_sha256, expected_stages, stages, terminal, exit, stdout_sha256, stderr_sha256, event_sha256, cleanup }; summary.validate_stage_closure()?; if summary.canonical_bytes()? != bytes { return Err("summary is valid JSON but not canonical bytes".into()); } Ok(summary)
    }
    pub fn authenticate_canonical_input(bytes: &[u8], expected: &Self) -> Result<Self, String> {
        let parsed = Self::parse_canonical(bytes)?; if &parsed != expected { return Err("summary value differs from authenticated expected evidence".into()); } Ok(parsed)
    }
}

pub fn summarize_without_authenticated_records() -> Result<Vec<u8>, String> {
    let _intake: fn(&[u8], &EvidenceSummary) -> Result<EvidenceSummary, String> =
        EvidenceSummary::authenticate_canonical_input;
    Err("authenticated underlying selector, mutation, and stage evidence is required; Unit A cannot synthesize success".into())
}

#[rustfmt::skip]
#[cfg(test)] mod tests {
    use super::*;
    fn sample() -> EvidenceSummary {
        let selectors = SELECTORS.iter().map(|v| (*v).into()).collect::<Vec<_>>(); let mutations = MUTATIONS.iter().map(|id| MutationRecord { id: (*id).into(), result: "rejected".into(), restored_sha256: "d".repeat(64) }).collect::<Vec<_>>(); let generator = "hum-dev 0.0.1"; let commit = "a".repeat(40); let tree = "b".repeat(40); let manifest = "c".repeat(64); let platform = "windows"; let target = "x86_64-pc-windows-msvc"; let toolchain = "cargo 1.96.0"; let binding = invocation_hash(&[POLICY, generator, &commit, &tree, &manifest, platform, target, toolchain, "focused"]);
        EvidenceSummary { generator: generator.into(), commit, tree, candidate_manifest: manifest, platform: platform.into(), target: target.into(), toolchain: toolchain.into(), profile: EvidenceProfile::Focused, invocation_binding: binding.clone(), selector_stream_sha256: selector_hash(&selectors), mutation_stream_sha256: mutation_hash(&mutations), selectors, mutations, expected_stages: STAGES.iter().map(|v| (*v).into()).collect(), stages: STAGES.iter().map(|name| StageRecord { name: (*name).into(), disposition: StageDisposition::Passed, skip_reason: None, skip_predicate: None, binding: binding.clone(), evidence_sha256: stage_hash(name, &binding) }).collect(), terminal: "success".into(), exit: 0, stdout_sha256: digest_hex(b"out"), stderr_sha256: digest_hex(b""), event_sha256: digest_hex(b"events"), cleanup: "closed".into() }
    }
    #[test] fn evidence_summary_v1_is_canonical_and_hash_bound() {
        let valid = sample(); let bytes = valid.canonical_bytes().unwrap(); assert!(bytes.starts_with(format!("{{\"schema\":\"{SCHEMA}\",").as_bytes()), "summary bytes/hash remain falsely accepted"); assert_eq!(EvidenceSummary::authenticate_canonical_input(&bytes, &valid).unwrap(), valid); assert_eq!(bytes.last(), Some(&b'\n')); assert!(!bytes.contains(&b'\r'));
        for corrupt in ["selectors", "mutations", "stages", "binding", "terminal"] { let mut value = sample(); match corrupt { "selectors" => value.selectors.clear(), "mutations" => value.mutations.clear(), "stages" => value.stages.clear(), "binding" => value.stages[0].binding = "foreign".into(), _ => value.terminal = "failed".into() } assert!(value.canonical_bytes().is_err(), "{corrupt} fabricated success"); }
        let mut duplicate = sample(); duplicate.stages.push(duplicate.stages[0].clone()); assert!(duplicate.canonical_bytes().is_err()); let mut changed = bytes; changed[2] ^= 1; assert!(EvidenceSummary::authenticate_canonical_input(&changed, &sample()).is_err());
    }
}
