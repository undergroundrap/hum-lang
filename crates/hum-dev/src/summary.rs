use crate::{command::EvidenceProfile, identity::CandidateIdentity};
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
#[derive(Debug, Clone, PartialEq, Eq)] pub(crate) struct StageRecord { name: String, disposition: StageDisposition, skip_reason: Option<String>, skip_predicate: Option<String>, binding: String, evidence_sha256: String }
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
pub(crate) fn oid_is_exact(value: &str) -> bool {
    value.len() == 40
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
    fn unsigned(&mut self) -> Result<u64, String> { let start = self.at; while self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) { self.at += 1; } let text=std::str::from_utf8(&self.bytes[start..self.at]).map_err(|_|"number is not ASCII")?; if text.is_empty() || (text.len()>1 && text.starts_with('0')) { return Err("summary integer is not canonical unsigned decimal".into()); } text.parse().map_err(|_|"invalid summary integer".into()) }
    fn positive(&mut self) -> Result<u64, String> { let start = self.at; while self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) { self.at += 1; } let text = std::str::from_utf8(&self.bytes[start..self.at]).map_err(|_| "number is not ASCII")?; if text.is_empty() || text == "0" || (text.len() > 1 && text.starts_with('0')) { return Err("summary integer is not canonical positive decimal".into()); } text.parse().map_err(|_| "invalid summary integer".into()) }
    fn strings(&mut self) -> Result<Vec<String>, String> { self.expect(b"[")?; let mut out = Vec::new(); if self.bytes.get(self.at) == Some(&b']') { self.at += 1; return Ok(out); } loop { out.push(self.string()?); match self.bytes.get(self.at) { Some(b',') => self.at += 1, Some(b']') => { self.at += 1; return Ok(out); }, _ => return Err("invalid string array delimiter".into()) } } }
    fn mutations(&mut self) -> Result<Vec<MutationRecord>, String> { self.expect(b"[")?; let mut out = Vec::new(); if self.bytes.get(self.at) == Some(&b']') { self.at += 1; return Ok(out); } loop { self.expect(b"{\"id\":")?; let id = self.string()?; self.expect(b",\"result\":")?; let result = self.string()?; self.expect(b",\"restored_sha256\":")?; let restored_sha256 = self.string()?; self.expect(b"}")?; out.push(MutationRecord { id, result, restored_sha256 }); match self.bytes.get(self.at) { Some(b',') => self.at += 1, Some(b']') => { self.at += 1; return Ok(out); }, _ => return Err("invalid mutation array delimiter".into()) } } }
    fn stages(&mut self) -> Result<Vec<StageRecord>, String> { self.expect(b"[")?; let mut out = Vec::new(); if self.bytes.get(self.at) == Some(&b']') { self.at += 1; return Ok(out); } loop { self.expect(b"{\"name\":")?; let name = self.string()?; self.expect(b",\"disposition\":")?; if self.string()? != "passed" { return Err("unknown stage disposition".into()); } self.expect(b",\"skip_reason\":")?; let skip_reason = self.optional_string()?; self.expect(b",\"skip_predicate\":")?; let skip_predicate = self.optional_string()?; self.expect(b",\"binding\":")?; let binding = self.string()?; self.expect(b",\"evidence_sha256\":")?; let evidence_sha256 = self.string()?; self.expect(b"}")?; out.push(StageRecord { name, disposition: StageDisposition::Passed, skip_reason, skip_predicate, binding, evidence_sha256 }); match self.bytes.get(self.at) { Some(b',') => self.at += 1, Some(b']') => { self.at += 1; return Ok(out); }, _ => return Err("invalid stage array delimiter".into()) } } }
}

pub const JOB_POLICY: &str = "wo25.unit_b.v1";
pub const JOB_SELECTORS: usize = 128;
pub const JOB_MUTATIONS: usize = 7;
const JOB_STAGES: &[&str] = &[
    "classifier",
    "workspace",
    "selectors",
    "mutations",
    "backend",
    "readiness",
    "hygiene",
    "claims",
    "release",
];

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JobSummary { pub(crate) generator: String, pub(crate) commit: String, pub(crate) parent: String, pub(crate) tree: String, pub(crate) candidate_manifest: String, pub(crate) raw_additions: u64, pub(crate) raw_deletions: u64, pub(crate) whitespace_additions: u64, pub(crate) whitespace_deletions: u64, pub(crate) cargo_lock_sha256: String, pub(crate) dependency_closure_sha256: String, pub(crate) configuration_sha256: String, pub(crate) platform: String, pub(crate) target: String, pub(crate) toolchain: String, pub(crate) compiler_sha256: String, pub(crate) producer_executable_sha256: String, pub(crate) profile: String, pub(crate) workflow: String, pub(crate) event: String, pub(crate) run_id: u64, pub(crate) run_attempt: u64, pub(crate) job_id: u64, pub(crate) checkout_sha: String, pub(crate) classifier_mode: String, pub(crate) classifier_reason: String, pub(crate) anchor: String, pub(crate) transitions: String, pub(crate) selector_ledger_sha256: String, pub(crate) selector_count: u64, pub(crate) mutation_ledger_sha256: String, pub(crate) mutation_count: u64, pub(crate) suite_count: u64, pub(crate) readiness: String, pub(crate) hygiene_file_count: u64, pub(crate) claims: String, pub(crate) nonclaims: String, pub(crate) release_version: String, pub(crate) expected_stages: Vec<String>, pub(crate) stages: Vec<StageRecord>, pub(crate) terminal: String, pub(crate) exit: i32, pub(crate) started_ticks: u64, pub(crate) completed_ticks: u64, pub(crate) timer_frequency: u64, pub(crate) duration_ms: u64, pub(crate) stdout_sha256: String, pub(crate) stderr_sha256: String, pub(crate) event_sha256: String, pub(crate) cleanup: String }

#[rustfmt::skip]
impl JobSummary {
    pub fn validate(&self) -> Result<(), String> {
        macro_rules! require { ($condition:expr,$message:literal) => { if !($condition) { return Err($message.into()); } }; }
        require!(self.generator == concat!("hum-dev ", env!("CARGO_PKG_VERSION")), "summary generator mismatch");
        require!(oid_is_exact(&self.commit), "summary commit identity mismatch");
        require!(oid_is_exact(&self.parent), "summary parent identity mismatch");
        require!(oid_is_exact(&self.tree), "summary tree identity mismatch");
        require!(self.checkout_sha == self.commit, "summary checkout identity mismatch");
        require!(self.profile == "full", "summary profile mismatch");
        require!(self.workflow == "ci", "summary workflow mismatch");
        require!(self.event == "push", "summary event mismatch");
        require!(self.classifier_mode == "full", "summary classifier mode mismatch");
        require!(self.classifier_reason == "no_status_transition", "summary classifier reason mismatch");
        require!(self.anchor.is_empty(), "full summary retained a classifier anchor");
        require!(self.transitions.is_empty(), "full summary retained classifier transitions");
        let expected_target = match self.platform.as_str() { "ubuntu" => "x86_64-unknown-linux-gnu", "windows" => "x86_64-pc-windows-msvc", _ => return Err("unknown summary platform".into()) }; if self.target != expected_target { return Err("platform target mismatch".into()); }
        require!(!self.toolchain.is_empty(), "summary toolchain identity is empty");
        require!(self.run_id > 0, "summary run ID is not positive"); require!(self.run_attempt > 0, "summary run attempt is not positive"); require!(self.job_id > 0, "summary job ID is not positive");
        for (name,value) in [("candidate manifest",&self.candidate_manifest),("Cargo lock",&self.cargo_lock_sha256),("dependency closure",&self.dependency_closure_sha256),("configuration",&self.configuration_sha256),("compiler",&self.compiler_sha256),("producer executable",&self.producer_executable_sha256),("selector ledger",&self.selector_ledger_sha256),("mutation ledger",&self.mutation_ledger_sha256),("stdout",&self.stdout_sha256),("stderr",&self.stderr_sha256),("event stream",&self.event_sha256)] { if !hash_is_exact(value) { return Err(format!("summary {name} SHA-256 identity is malformed")); } }
        for (name,value) in [("raw additions",self.raw_additions),("raw deletions",self.raw_deletions),("whitespace additions",self.whitespace_additions),("whitespace deletions",self.whitespace_deletions)] { if value != 0 { return Err(format!("clean full-anchor {name} is nonzero")); } }
        require!(self.selector_count == JOB_SELECTORS as u64, "summary selector count mismatch"); require!(self.mutation_count == JOB_MUTATIONS as u64, "summary mutation count mismatch");
        require!(self.suite_count > 0, "summary suite count is zero"); require!(self.hygiene_file_count > 0, "summary hygiene file count is zero");
        require!(self.readiness == "ir_ready=1;backend_ready=1", "summary readiness mismatch"); require!(self.claims == "passed", "summary claims mismatch");
        require!(self.nonclaims == "no_semantic_or_publication_authority", "summary nonclaims mismatch"); require!(self.release_version == "0.0.1", "summary release version mismatch");
        require!(self.expected_stages.iter().map(String::as_str).collect::<Vec<_>>() == JOB_STAGES, "summary expected-stage order mismatch"); require!(self.stages.len() == JOB_STAGES.len(), "summary stage count mismatch");
        let binding = self.binding();
        for (index, expected) in JOB_STAGES.iter().enumerate() {
            let stage = &self.stages[index];
            if stage.name != *expected { return Err(format!("stage `{expected}` is missing or reordered")); }
            if stage.disposition != StageDisposition::Passed { return Err(format!("stage `{expected}` is not passed")); }
            if stage.skip_reason.is_some() { return Err(format!("stage `{expected}` retained a skip reason")); }
            if stage.skip_predicate.is_some() { return Err(format!("stage `{expected}` retained a skip predicate")); }
            if stage.binding != binding { return Err(format!("stage `{expected}` has a mixed binding")); }
            if stage.evidence_sha256 != stage_hash(expected, &binding) { return Err(format!("stage `{expected}` evidence identity mismatch")); }
        }
        require!(self.terminal == "success", "summary terminal disposition is not success"); require!(self.exit == 0, "summary process exit is nonzero");
        require!(self.started_ticks > 0, "summary start tick is zero"); require!(self.completed_ticks > self.started_ticks, "summary completion tick is not later than start"); require!(self.timer_frequency > 0, "summary timer frequency is zero");
        require!(self.duration_ms == (self.completed_ticks-self.started_ticks).saturating_mul(1000)/self.timer_frequency, "summary duration does not match monotonic ticks"); require!(self.cleanup == "closed", "summary cleanup is not closed"); Ok(())
    }
    pub fn binding(&self) -> String { invocation_hash(&[JOB_POLICY,&self.generator,&self.commit,&self.parent,&self.tree,&self.candidate_manifest,&self.raw_additions.to_string(),&self.raw_deletions.to_string(),&self.whitespace_additions.to_string(),&self.whitespace_deletions.to_string(),&self.cargo_lock_sha256,&self.dependency_closure_sha256,&self.configuration_sha256,&self.platform,&self.target,&self.toolchain,&self.compiler_sha256,&self.producer_executable_sha256,&self.profile,&self.workflow,&self.event,&self.run_id.to_string(),&self.run_attempt.to_string(),&self.job_id.to_string(),&self.checkout_sha,&self.classifier_mode,&self.classifier_reason,&self.anchor,&self.transitions,&self.selector_ledger_sha256,&self.selector_count.to_string(),&self.mutation_ledger_sha256,&self.mutation_count.to_string(),&self.suite_count.to_string(),&self.readiness,&self.hygiene_file_count.to_string(),&self.claims,&self.nonclaims,&self.release_version,&self.terminal,&self.exit.to_string(),&self.started_ticks.to_string(),&self.completed_ticks.to_string(),&self.timer_frequency.to_string(),&self.duration_ms.to_string(),&self.stdout_sha256,&self.stderr_sha256,&self.event_sha256,&self.cleanup]) }
    pub fn artifact_name(&self) -> String { format!("hum-evidence-summary-v1-{}-{}-{}-{}", self.run_id, self.run_attempt, self.job_id, self.platform) }
    pub fn executable_artifact_name(&self) -> String { format!("hum-dev-executable-transport-v1-{}-{}-{}-{}-{}", self.run_id, self.run_attempt, self.job_id, self.platform, self.producer_executable_sha256) }
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, String> { self.validate()?; let mut out = String::from("{\"schema\":\""); out.push_str(SCHEMA); out.push_str("\",\"policy\":\""); out.push_str(JOB_POLICY); out.push_str("\",\"generator\":"); quoted(&mut out, &self.generator); for (name, value) in [("commit",&self.commit),("parent",&self.parent),("tree",&self.tree),("candidate_manifest",&self.candidate_manifest)] { out.push_str(",\""); out.push_str(name); out.push_str("\":"); quoted(&mut out,value); } out.push_str(&format!(",\"raw_additions\":{},\"raw_deletions\":{},\"whitespace_additions\":{},\"whitespace_deletions\":{}",self.raw_additions,self.raw_deletions,self.whitespace_additions,self.whitespace_deletions)); for (name,value) in [("cargo_lock_sha256",&self.cargo_lock_sha256),("dependency_closure_sha256",&self.dependency_closure_sha256),("configuration_sha256",&self.configuration_sha256),("platform",&self.platform),("target",&self.target),("toolchain",&self.toolchain),("compiler_sha256",&self.compiler_sha256),("producer_executable_sha256",&self.producer_executable_sha256),("profile",&self.profile),("workflow",&self.workflow),("event",&self.event)] { out.push_str(",\""); out.push_str(name); out.push_str("\":"); quoted(&mut out,value); } out.push_str(&format!(",\"run_id\":{},\"run_attempt\":{},\"job_id\":{}",self.run_id,self.run_attempt,self.job_id)); for (name,value) in [("checkout_sha",&self.checkout_sha),("classifier_mode",&self.classifier_mode),("classifier_reason",&self.classifier_reason),("anchor",&self.anchor),("transitions",&self.transitions),("selector_ledger_sha256",&self.selector_ledger_sha256)] { out.push_str(",\""); out.push_str(name); out.push_str("\":"); quoted(&mut out,value); } out.push_str(&format!(",\"selector_count\":{}",self.selector_count)); out.push_str(",\"mutation_ledger_sha256\":"); quoted(&mut out,&self.mutation_ledger_sha256); out.push_str(&format!(",\"mutation_count\":{},\"suite_count\":{}",self.mutation_count,self.suite_count)); for (name,value) in [("readiness",&self.readiness),("claims",&self.claims),("nonclaims",&self.nonclaims),("release_version",&self.release_version)] { out.push_str(",\""); out.push_str(name); out.push_str("\":"); quoted(&mut out,value); } out.push_str(&format!(",\"hygiene_file_count\":{}",self.hygiene_file_count)); out.push_str(",\"expected_stages\":"); string_array(&mut out,&self.expected_stages); out.push_str(",\"stages\":["); for (index,stage) in self.stages.iter().enumerate() { if index != 0 { out.push(','); } out.push_str("{\"name\":"); quoted(&mut out,&stage.name); out.push_str(",\"disposition\":\"passed\",\"skip_reason\":null,\"skip_predicate\":null,\"binding\":"); quoted(&mut out,&stage.binding); out.push_str(",\"evidence_sha256\":"); quoted(&mut out,&stage.evidence_sha256); out.push('}'); } out.push_str("],\"terminal\":"); quoted(&mut out,&self.terminal); out.push_str(&format!(",\"exit\":{},\"started_ticks\":{},\"completed_ticks\":{},\"timer_frequency\":{},\"duration_ms\":{}",self.exit,self.started_ticks,self.completed_ticks,self.timer_frequency,self.duration_ms)); for (name,value) in [("stdout_sha256",&self.stdout_sha256),("stderr_sha256",&self.stderr_sha256),("event_sha256",&self.event_sha256),("cleanup",&self.cleanup)] { out.push_str(",\""); out.push_str(name); out.push_str("\":"); quoted(&mut out,value); } out.push_str("}\n"); Ok(out.into_bytes()) }
    pub fn parse_canonical(bytes: &[u8]) -> Result<Self,String> { if bytes.starts_with(&[0xef,0xbb,0xbf]) || bytes.last()!=Some(&b'\n') || bytes.contains(&b'\r') { return Err("summary framing is not canonical UTF-8/LF".into()); } std::str::from_utf8(bytes).map_err(|_|"summary is not UTF-8")?; let mut r=Reader::new(bytes); macro_rules! s {($name:literal)=>{{r.expect(concat!(",\"",$name,"\":").as_bytes())?;r.string()?}};} macro_rules! p {($name:literal)=>{{r.expect(concat!(",\"",$name,"\":").as_bytes())?;r.positive()?}};} macro_rules! u {($name:literal)=>{{r.expect(concat!(",\"",$name,"\":").as_bytes())?;r.unsigned()?}};} r.expect(b"{\"schema\":")?; if r.string()?!=SCHEMA{return Err("unknown summary schema".into())} r.expect(b",\"policy\":")?; if r.string()?!=JOB_POLICY{return Err("unknown summary policy".into())} let generator=s!("generator"); let commit=s!("commit"); let parent=s!("parent"); let tree=s!("tree"); let candidate_manifest=s!("candidate_manifest"); let raw_additions=u!("raw_additions"); let raw_deletions=u!("raw_deletions"); let whitespace_additions=u!("whitespace_additions"); let whitespace_deletions=u!("whitespace_deletions"); let cargo_lock_sha256=s!("cargo_lock_sha256"); let dependency_closure_sha256=s!("dependency_closure_sha256"); let configuration_sha256=s!("configuration_sha256"); let platform=s!("platform"); let target=s!("target"); let toolchain=s!("toolchain"); let compiler_sha256=s!("compiler_sha256"); let producer_executable_sha256=s!("producer_executable_sha256"); let profile=s!("profile"); let workflow=s!("workflow"); let event=s!("event"); let run_id=p!("run_id"); let run_attempt=p!("run_attempt"); let job_id=p!("job_id"); let checkout_sha=s!("checkout_sha"); let classifier_mode=s!("classifier_mode"); let classifier_reason=s!("classifier_reason"); let anchor=s!("anchor"); let transitions=s!("transitions"); let selector_ledger_sha256=s!("selector_ledger_sha256"); let selector_count=p!("selector_count"); let mutation_ledger_sha256=s!("mutation_ledger_sha256"); let mutation_count=p!("mutation_count"); let suite_count=p!("suite_count"); let readiness=s!("readiness"); let claims=s!("claims"); let nonclaims=s!("nonclaims"); let release_version=s!("release_version"); let hygiene_file_count=p!("hygiene_file_count"); r.expect(b",\"expected_stages\":")?; let expected_stages=r.strings()?; r.expect(b",\"stages\":")?; let stages=r.stages()?; let terminal=s!("terminal"); r.expect(b",\"exit\":")?; let exit=r.number()?; let started_ticks=p!("started_ticks"); let completed_ticks=p!("completed_ticks"); let timer_frequency=p!("timer_frequency"); let duration_ms=p!("duration_ms"); let stdout_sha256=s!("stdout_sha256"); let stderr_sha256=s!("stderr_sha256"); let event_sha256=s!("event_sha256"); let cleanup=s!("cleanup"); r.expect(b"}\n")?; if r.at!=bytes.len(){return Err("trailing summary bytes".into())} let value=Self{generator,commit,parent,tree,candidate_manifest,raw_additions,raw_deletions,whitespace_additions,whitespace_deletions,cargo_lock_sha256,dependency_closure_sha256,configuration_sha256,platform,target,toolchain,compiler_sha256,producer_executable_sha256,profile,workflow,event,run_id,run_attempt,job_id,checkout_sha,classifier_mode,classifier_reason,anchor,transitions,selector_ledger_sha256,selector_count,mutation_ledger_sha256,mutation_count,suite_count,readiness,hygiene_file_count,claims,nonclaims,release_version,expected_stages,stages,terminal,exit,started_ticks,completed_ticks,timer_frequency,duration_ms,stdout_sha256,stderr_sha256,event_sha256,cleanup}; value.validate()?; if value.canonical_bytes()?!=bytes{return Err("summary is valid JSON but not canonical bytes".into())} Ok(value) }
    pub fn from_environment(executable:&[u8],job_id:u64,candidate:&CandidateIdentity,cargo_lock_sha256:String)->Result<Self,String>{let env=|name:&str|std::env::var(name).map_err(|_|format!("missing authenticated summary input `{name}`"));let n=|name:&str|env(name)?.parse::<u64>().map_err(|_|format!("invalid summary input `{name}`"));let started_ticks=n("HUM_STARTED_TICKS")?;let completed_ticks=n("HUM_COMPLETED_TICKS")?;let timer_frequency=n("HUM_TIMER_FREQUENCY")?;let mut value=Self{generator:env!("CARGO_PKG_NAME").to_string()+" "+env!("CARGO_PKG_VERSION"),commit:candidate.commit.clone(),parent:candidate.parents[0].clone(),tree:candidate.tree.clone(),candidate_manifest:candidate.binding().state_sha256,raw_additions:candidate.raw_additions,raw_deletions:candidate.raw_deletions,whitespace_additions:candidate.whitespace_additions,whitespace_deletions:candidate.whitespace_deletions,cargo_lock_sha256,dependency_closure_sha256:env("HUM_DEPENDENCY_CLOSURE_SHA256")?,configuration_sha256:env("HUM_CONFIGURATION_SHA256")?,platform:env("HUM_PLATFORM")?,target:env("HUM_TARGET")?,toolchain:env("HUM_TOOLCHAIN")?,compiler_sha256:env("HUM_COMPILER_SHA256")?,producer_executable_sha256:digest_hex(executable),profile:"full".into(),workflow:env("HUM_WORKFLOW")?,event:env("GITHUB_EVENT_NAME")?,run_id:n("GITHUB_RUN_ID")?,run_attempt:n("GITHUB_RUN_ATTEMPT")?,job_id,checkout_sha:env("GITHUB_SHA")?,classifier_mode:env("HUM_CLASSIFIER_MODE")?,classifier_reason:env("HUM_CLASSIFIER_REASON")?,anchor:env("HUM_CLASSIFIER_ANCHOR").unwrap_or_default(),transitions:env("HUM_CLASSIFIER_TRANSITIONS").unwrap_or_default(),selector_ledger_sha256:env("HUM_SELECTOR_LEDGER_SHA256")?,selector_count:n("HUM_SELECTOR_COUNT")?,mutation_ledger_sha256:env("HUM_MUTATION_LEDGER_SHA256")?,mutation_count:n("HUM_MUTATION_COUNT")?,suite_count:n("HUM_SUITE_COUNT")?,readiness:env("HUM_READINESS")?,hygiene_file_count:n("HUM_HYGIENE_FILE_COUNT")?,claims:env("HUM_CLAIMS")?,nonclaims:"no_semantic_or_publication_authority".into(),release_version:env("HUM_RELEASE_VERSION")?,expected_stages:JOB_STAGES.iter().map(|v|(*v).into()).collect(),stages:Vec::new(),terminal:"success".into(),exit:0,started_ticks,completed_ticks,timer_frequency,duration_ms:(completed_ticks-started_ticks).saturating_mul(1000)/timer_frequency,stdout_sha256:env("HUM_STDOUT_SHA256")?,stderr_sha256:env("HUM_STDERR_SHA256")?,event_sha256:env("HUM_EVENT_SHA256")?,cleanup:"closed".into()};let binding=value.binding();value.stages=JOB_STAGES.iter().map(|name|StageRecord{name:(*name).into(),disposition:StageDisposition::Passed,skip_reason:None,skip_predicate:None,binding:binding.clone(),evidence_sha256:stage_hash(name,&binding)}).collect();value.validate()?;Ok(value)}
}

pub fn authenticate_platform_pair(ubuntu: &JobSummary, windows: &JobSummary) -> Result<(), String> {
    ubuntu.validate()?;
    windows.validate()?;
    if ubuntu.platform != "ubuntu" || windows.platform != "windows" {
        return Err("platform summary ordering mismatch".into());
    }
    for (name, a, b) in [
        ("generator", &ubuntu.generator, &windows.generator),
        ("commit", &ubuntu.commit, &windows.commit),
        ("parent", &ubuntu.parent, &windows.parent),
        ("tree", &ubuntu.tree, &windows.tree),
        (
            "candidate",
            &ubuntu.candidate_manifest,
            &windows.candidate_manifest,
        ),
        (
            "cargo_lock",
            &ubuntu.cargo_lock_sha256,
            &windows.cargo_lock_sha256,
        ),
        (
            "dependency_closure",
            &ubuntu.dependency_closure_sha256,
            &windows.dependency_closure_sha256,
        ),
        (
            "configuration",
            &ubuntu.configuration_sha256,
            &windows.configuration_sha256,
        ),
        ("profile", &ubuntu.profile, &windows.profile),
        ("workflow", &ubuntu.workflow, &windows.workflow),
        ("event", &ubuntu.event, &windows.event),
        ("checkout", &ubuntu.checkout_sha, &windows.checkout_sha),
        (
            "classifier_mode",
            &ubuntu.classifier_mode,
            &windows.classifier_mode,
        ),
        (
            "classifier_reason",
            &ubuntu.classifier_reason,
            &windows.classifier_reason,
        ),
        ("anchor", &ubuntu.anchor, &windows.anchor),
        ("transitions", &ubuntu.transitions, &windows.transitions),
        (
            "selector_ledger",
            &ubuntu.selector_ledger_sha256,
            &windows.selector_ledger_sha256,
        ),
        (
            "mutation_ledger",
            &ubuntu.mutation_ledger_sha256,
            &windows.mutation_ledger_sha256,
        ),
        ("readiness", &ubuntu.readiness, &windows.readiness),
        ("claims", &ubuntu.claims, &windows.claims),
        ("nonclaims", &ubuntu.nonclaims, &windows.nonclaims),
        ("release", &ubuntu.release_version, &windows.release_version),
    ] {
        if a != b {
            return Err(format!("cross-platform `{name}` disagreement"));
        }
    }
    if ubuntu.run_id != windows.run_id
        || ubuntu.run_attempt != windows.run_attempt
        || ubuntu.raw_additions != windows.raw_additions
        || ubuntu.raw_deletions != windows.raw_deletions
        || ubuntu.whitespace_additions != windows.whitespace_additions
        || ubuntu.whitespace_deletions != windows.whitespace_deletions
        || ubuntu.selector_count != windows.selector_count
        || ubuntu.mutation_count != windows.mutation_count
        || ubuntu.suite_count != windows.suite_count
        || ubuntu.hygiene_file_count != windows.hygiene_file_count
        || ubuntu.expected_stages != windows.expected_stages
        || ubuntu.terminal != windows.terminal
        || ubuntu.exit != windows.exit
    {
        return Err("cross-platform numeric or stage disagreement".into());
    }
    Ok(())
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
    fn job(platform:&str)->JobSummary { let mut value=JobSummary{generator:"hum-dev 0.0.1".into(),commit:"a".repeat(40),parent:"b".repeat(40),tree:"c".repeat(40),candidate_manifest:"8".repeat(64),raw_additions:0,raw_deletions:0,whitespace_additions:0,whitespace_deletions:0,cargo_lock_sha256:"d".repeat(64),dependency_closure_sha256:"e".repeat(64),configuration_sha256:"9".repeat(64),platform:platform.into(),target:if platform=="ubuntu"{"x86_64-unknown-linux-gnu"}else{"x86_64-pc-windows-msvc"}.into(),toolchain:if platform=="ubuntu"{"rustc linux"}else{"rustc windows"}.into(),compiler_sha256:"0".repeat(64),producer_executable_sha256:if platform=="ubuntu"{"1"}else{"2"}.repeat(64),profile:"full".into(),workflow:"ci".into(),event:"push".into(),run_id:71,run_attempt:2,job_id:if platform=="ubuntu"{81}else{82},checkout_sha:"a".repeat(40),classifier_mode:"full".into(),classifier_reason:"no_status_transition".into(),anchor:String::new(),transitions:String::new(),selector_ledger_sha256:"3".repeat(64),selector_count:128,mutation_ledger_sha256:"4".repeat(64),mutation_count:7,suite_count:200,readiness:"ir_ready=1;backend_ready=1".into(),hygiene_file_count:584,claims:"passed".into(),nonclaims:"no_semantic_or_publication_authority".into(),release_version:"0.0.1".into(),expected_stages:JOB_STAGES.iter().map(|v|(*v).into()).collect(),stages:Vec::new(),terminal:"success".into(),exit:0,started_ticks:100,completed_ticks:1100,timer_frequency:1000,duration_ms:1000,stdout_sha256:"5".repeat(64),stderr_sha256:"6".repeat(64),event_sha256:"7".repeat(64),cleanup:"closed".into()};let binding=value.binding();value.stages=JOB_STAGES.iter().map(|name|StageRecord{name:(*name).into(),disposition:StageDisposition::Passed,skip_reason:None,skip_predicate:None,binding:binding.clone(),evidence_sha256:stage_hash(name,&binding)}).collect();value }
    #[test] fn cross_platform_status_agreement_is_exact(){
        let ubuntu=job("ubuntu");let windows=job("windows");authenticate_platform_pair(&ubuntu,&windows).unwrap();
        for field in ["generator","commit","parent","tree","candidate","accounting","lock","closure","configuration","platform","target","toolchain","compiler","producer","profile","workflow","event","run","attempt","job","checkout","classifier","selector_order","selector_hash","selector_count","mutation_order","mutation_hash","suite","readiness","hygiene","claims","nonclaims","release","expected_stages","terminal","timing","stdout","stderr","event_stream","cleanup"]{
            let mut changed=windows.clone();match field{
                "generator"=>changed.generator="foreign generator".into(),"commit"=>changed.commit="f".repeat(40),"parent"=>changed.parent="f".repeat(40),"tree"=>changed.tree="f".repeat(40),"candidate"=>changed.candidate_manifest="f".repeat(64),"accounting"=>changed.raw_additions=1,"lock"=>changed.cargo_lock_sha256="f".repeat(64),"closure"=>changed.dependency_closure_sha256="f".repeat(64),"configuration"=>changed.configuration_sha256="f".repeat(64),"platform"=>changed.platform="ubuntu".into(),"target"=>changed.target="x86_64-unknown-linux-gnu".into(),"toolchain"=>changed.toolchain="foreign toolchain".into(),"compiler"=>changed.compiler_sha256="f".repeat(64),"producer"=>changed.producer_executable_sha256="f".repeat(64),"profile"=>changed.profile="status".into(),"workflow"=>changed.workflow="xx".into(),"event"=>changed.event="pull".into(),"run"=>changed.run_id+=1,"attempt"=>changed.run_attempt+=1,"job"=>changed.job_id+=1,"checkout"=>changed.checkout_sha="f".repeat(40),"classifier"=>changed.anchor="f".repeat(40),"selector_order"|"selector_hash"=>changed.selector_ledger_sha256="f".repeat(64),"selector_count"=>changed.selector_count-=1,"mutation_order"|"mutation_hash"=>changed.mutation_ledger_sha256="f".repeat(64),"suite"=>changed.suite_count=0,"readiness"=>changed.readiness="ir_ready=1;backend_ready=0".into(),"hygiene"=>changed.hygiene_file_count=0,"claims"=>changed.claims="failed".into(),"nonclaims"=>changed.nonclaims.clear(),"release"=>changed.release_version="0.0.2".into(),"expected_stages"=>changed.expected_stages.swap(0,1),"terminal"=>changed.terminal="failed".into(),"timing"=>changed.duration_ms+=1,"stdout"=>changed.stdout_sha256="f".repeat(64),"stderr"=>changed.stderr_sha256="f".repeat(64),"event_stream"=>changed.event_sha256="f".repeat(64),_=>changed.cleanup="failed".into()}
            assert!(changed.validate().is_err(),"{field} summary corruption authenticated");
        }
        for stage_case in 1..=6 { let mut changed=windows.clone();match stage_case{1=>{changed.stages.remove(0);},2=>changed.stages.push(changed.stages[0].clone()),3=>changed.stages[0].name="plausible_unknown".into(),4=>changed.stages[0].skip_reason=Some("false".into()),5=>changed.stages[0].binding="foreign".into(),_=>changed.stages[0].evidence_sha256="f".repeat(64)}assert!(changed.validate().is_err(),"S0{stage_case} false closure authenticated"); }
        let bytes=ubuntu.canonical_bytes().unwrap();assert_eq!(JobSummary::parse_canonical(&bytes).unwrap(),ubuntu);let mut schema=bytes.clone();schema[11]=b'X';assert!(JobSummary::parse_canonical(&schema).is_err());let policy=String::from_utf8(bytes.clone()).unwrap().replace(JOB_POLICY,"wo25.unit_x.v1");assert!(JobSummary::parse_canonical(policy.as_bytes()).is_err());let mut trailing=bytes;trailing.push(b'x');assert!(JobSummary::parse_canonical(&trailing).is_err());
        let fixture_ubuntu=JobSummary::parse_canonical(include_bytes!("../../../fixtures/evidence/job_summary_ubuntu.v1.json")).unwrap();let fixture_windows=JobSummary::parse_canonical(include_bytes!("../../../fixtures/evidence/job_summary_windows.v1.json")).unwrap();authenticate_platform_pair(&fixture_ubuntu,&fixture_windows).unwrap();assert_eq!(fixture_ubuntu.artifact_name(),"hum-evidence-summary-v1-71-2-81-ubuntu");assert_eq!(fixture_windows.executable_artifact_name(),format!("hum-dev-executable-transport-v1-71-2-82-windows-{}","2".repeat(64)));
        let matrix=include_str!("../../../fixtures/evidence/summary_corruption_cases.v1.json");for id in (1..=6).map(|n|format!("S{n:02}")).chain((1..=43).map(|n|format!("J{n:02}"))).chain((1..=16).map(|n|format!("T{n:02}"))){assert_eq!(matrix.matches(&format!("\"id\":\"{id}\"")).count(),1,"missing or duplicate corruption {id}");}
    }
}
