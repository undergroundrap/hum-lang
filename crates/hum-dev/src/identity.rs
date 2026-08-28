use hum_sha256::digest_hex;
use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    process::{Command, Output},
};

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)] pub struct IndexEntry { pub path: String, pub mode: String, pub oid: String, pub stage: u8, pub intent_to_add: bool }
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)] pub struct RefIdentity { pub name: String, pub oid: String }
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)] pub struct PathIdentity { pub path: String, pub head_mode: Option<String>, pub head_oid: Option<String>, pub worktree_kind: String, pub worktree_mode: Option<String>, pub worktree_sha256: Option<String>, pub bytes: u64, pub untracked: bool, pub index: Vec<IndexEntry> }
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)] pub struct CandidateIdentity { pub commit: String, pub parents: Vec<String>, pub tree: String, pub head_ref: Option<String>, pub refs: Vec<RefIdentity>, pub index_entries: Vec<IndexEntry>, pub index_clean: bool, pub worktree_clean: bool, pub untracked_clean: bool, pub paths: Vec<PathIdentity>, pub raw_additions: u64, pub raw_deletions: u64, pub whitespace_additions: u64, pub whitespace_deletions: u64 }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateBinding {
    pub state_sha256: String,
}
impl CandidateBinding {
    pub fn matches(&self, expected: &Self) -> bool {
        self == expected
    }
}

#[rustfmt::skip]
fn repository_argument(repository: &Path) -> String { repository.to_string_lossy().replace('\\', "/").trim_start_matches("//?/").into() }
#[rustfmt::skip]
fn git(repository: &Path, arguments: &[&str]) -> Result<Output, String> {
    let executable = std::env::var_os("HUM_DEV_GIT").unwrap_or_else(|| "git".into());
    Command::new(executable).current_dir(repository).arg("-c").arg(format!("safe.directory={}", repository_argument(repository))).args(arguments).output().map_err(|error| format!("git launch failed: {error}"))
}
#[rustfmt::skip]
fn git_success(repository: &Path, arguments: &[&str]) -> Result<Vec<u8>, String> {
    let output = git(repository, arguments)?;
    if !output.status.success() { return Err(format!("git {} failed: {}", arguments.join(" "), String::from_utf8_lossy(&output.stderr).trim())); }
    Ok(output.stdout)
}
#[rustfmt::skip]
fn one_line(repository: &Path, arguments: &[&str]) -> Result<String, String> {
    let bytes = git_success(repository, arguments)?; let text = std::str::from_utf8(&bytes).map_err(|_| "git emitted non-UTF-8 identity")?; let lines: Vec<_> = text.lines().collect();
    if lines.len() != 1 || lines[0].is_empty() { return Err(format!("git {} did not emit exactly one record", arguments.join(" "))); } Ok(lines[0].into())
}
#[rustfmt::skip]
fn nul_paths(bytes: &[u8]) -> Result<Vec<String>, String> { bytes.split(|byte| *byte == 0).filter(|field| !field.is_empty()).map(|field| Ok(std::str::from_utf8(field).map_err(|_| "Git path is not UTF-8")?.replace('\\', "/"))).collect() }
#[rustfmt::skip]
fn parse_index(bytes: &[u8], ita: &BTreeSet<String>) -> Result<Vec<IndexEntry>, String> {
    let mut entries = Vec::new();
    for record in bytes.split(|byte| *byte == 0).filter(|record| !record.is_empty()) {
        let tab = record.iter().position(|byte| *byte == b'\t').ok_or("index record missing path")?; let meta = std::str::from_utf8(&record[..tab]).map_err(|_| "index metadata is not UTF-8")?; let fields: Vec<_> = meta.split(' ').collect();
        if fields.len() != 3 { return Err("index metadata field count mismatch".into()); }
        let path = std::str::from_utf8(&record[tab + 1..]).map_err(|_| "index path is not UTF-8")?.replace('\\', "/");
        entries.push(IndexEntry { path: path.clone(), mode: fields[0].into(), oid: fields[1].into(), stage: fields[2].parse().map_err(|_| "index stage is invalid")?, intent_to_add: ita.contains(&path) });
    } Ok(entries)
}
#[rustfmt::skip]
fn parse_refs(bytes: &[u8]) -> Result<Vec<RefIdentity>, String> { bytes.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()).map(|line| { let fields: Vec<_> = line.split(|byte| *byte == 0).collect(); if fields.len() != 2 { return Err("ref record field count mismatch".into()); } Ok(RefIdentity { name: std::str::from_utf8(fields[0]).map_err(|_| "ref name is not UTF-8")?.into(), oid: std::str::from_utf8(fields[1]).map_err(|_| "ref OID is not UTF-8")?.into() }) }).collect() }
#[rustfmt::skip]
fn parse_numstat(bytes: &[u8]) -> Result<(u64, u64), String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "numstat is not UTF-8")?; let mut result = (0, 0);
    for line in text.lines().filter(|line| !line.is_empty()) { let fields: Vec<_> = line.splitn(3, '\t').collect(); if fields.len() != 3 || fields[0] == "-" || fields[1] == "-" { return Err("invalid or binary numstat record".into()); } result.0 += fields[0].parse::<u64>().map_err(|_| "invalid numstat addition")?; result.1 += fields[1].parse::<u64>().map_err(|_| "invalid numstat deletion")?; } Ok(result)
}
fn line_count(bytes: &[u8]) -> u64 {
    bytes.iter().filter(|byte| **byte == b'\n').count() as u64
        + u64::from(!bytes.is_empty() && bytes.last() != Some(&b'\n'))
}
fn normalized_lines(bytes: &[u8]) -> Vec<Vec<u8>> {
    bytes
        .split_inclusive(|byte| *byte == b'\n')
        .map(|line| {
            line.iter()
                .filter(|byte| !byte.is_ascii_whitespace())
                .copied()
                .collect()
        })
        .collect()
}
fn whitespace_diff(base: &[u8], candidate: &[u8]) -> (u64, u64) {
    let base = normalized_lines(base);
    let candidate = normalized_lines(candidate);
    let mut previous = vec![0_u64; candidate.len() + 1];
    for base_line in &base {
        let mut current = vec![0_u64; candidate.len() + 1];
        for (index, candidate_line) in candidate.iter().enumerate() {
            current[index + 1] = if base_line == candidate_line {
                previous[index] + 1
            } else {
                previous[index + 1].max(current[index])
            };
        }
        previous = current;
    }
    let common = previous[candidate.len()];
    (candidate.len() as u64 - common, base.len() as u64 - common)
}
#[rustfmt::skip]
fn head_entry(repository: &Path, path: &str) -> Result<(Option<String>, Option<String>), String> {
    let bytes = git_success(repository, &["ls-tree", "-z", "HEAD", "--", path])?; if bytes.is_empty() { return Ok((None, None)); }
    let tab = bytes.iter().position(|byte| *byte == b'\t').ok_or("tree record missing path")?; let meta = std::str::from_utf8(&bytes[..tab]).map_err(|_| "tree record is not UTF-8")?; let fields: Vec<_> = meta.split(' ').collect();
    if fields.len() != 3 { return Err("tree record field count mismatch".into()); } Ok((Some(fields[0].into()), Some(fields[2].into())))
}
fn put(out: &mut Vec<u8>, value: &[u8]) {
    out.extend_from_slice(&(value.len() as u64).to_be_bytes());
    out.extend_from_slice(value);
}
fn put_string(out: &mut Vec<u8>, value: &str) {
    put(out, value.as_bytes());
}
fn put_option(out: &mut Vec<u8>, value: &Option<String>) {
    out.push(u8::from(value.is_some()));
    if let Some(value) = value {
        put_string(out, value);
    }
}

#[rustfmt::skip]
impl CandidateIdentity {
    pub fn read(repository: &Path) -> Result<Self, String> {
        let repository = fs::canonicalize(repository).map_err(|error| format!("repository path failed: {error}"))?;
        let head = one_line(&repository, &["rev-list", "--parents", "-n", "1", "HEAD"])?; let mut head_fields = head.split_whitespace(); let commit = head_fields.next().ok_or("HEAD record is empty")?.to_string(); let parents = head_fields.map(str::to_string).collect(); let tree = one_line(&repository, &["rev-parse", "HEAD^{tree}"])?;
        let symbolic = git(&repository, &["symbolic-ref", "--quiet", "HEAD"])?; let head_ref = if symbolic.status.success() { Some(std::str::from_utf8(&symbolic.stdout).map_err(|_| "symbolic HEAD is not UTF-8")?.trim_end_matches(['\r', '\n']).into()) } else if symbolic.status.code() == Some(1) { None } else { return Err(String::from_utf8_lossy(&symbolic.stderr).trim().into()); };
        let refs = parse_refs(&git_success(&repository, &["for-each-ref", "--format=%(refname)%00%(objectname)", "refs/heads"])?)?;
        let visible: BTreeSet<_> = nul_paths(&git_success(&repository, &["diff", "--cached", "--name-only", "-z", "--ita-visible-in-index", "HEAD", "--"])?)?.into_iter().collect(); let invisible: BTreeSet<_> = nul_paths(&git_success(&repository, &["diff", "--cached", "--name-only", "-z", "--ita-invisible-in-index", "HEAD", "--"])?)?.into_iter().collect(); let ita: BTreeSet<_> = visible.difference(&invisible).cloned().collect();
        let index_entries = parse_index(&git_success(&repository, &["ls-files", "--stage", "-z"])?, &ita)?; let index_status = git(&repository, &["diff", "--cached", "--quiet"])?; let worktree_status = git(&repository, &["diff", "--quiet"])?;
        if !matches!(index_status.status.code(), Some(0 | 1)) || !matches!(worktree_status.status.code(), Some(0 | 1)) { return Err("Git cleanliness probe failed".into()); }
        let untracked: BTreeSet<_> = nul_paths(&git_success(&repository, &["ls-files", "--others", "--exclude-standard", "-z"])?)?.into_iter().collect(); let mut paths: BTreeSet<_> = nul_paths(&git_success(&repository, &["diff", "--name-only", "-z", "HEAD", "--"])?)?.into_iter().collect(); paths.extend(untracked.iter().cloned()); paths.extend(ita.iter().cloned()); paths.extend(index_entries.iter().filter(|entry| entry.stage != 0).map(|entry| entry.path.clone()));
        let mut identities = Vec::new(); let mut untracked_raw = 0; let mut untracked_ws = 0;
        for path in paths {
            let absolute = repository.join(&path); let metadata = fs::symlink_metadata(&absolute).ok();
            let (kind, bytes, mode) = match metadata { None => ("missing".into(), None, None), Some(meta) if meta.file_type().is_symlink() => ("symlink".into(), Some(fs::read_link(&absolute).map_err(|e| e.to_string())?.to_string_lossy().as_bytes().to_vec()), None), Some(meta) if meta.is_file() => { #[cfg(unix)] let mode = { use std::os::unix::fs::PermissionsExt; Some(if meta.permissions().mode() & 0o111 == 0 { "100644" } else { "100755" }.into()) }; #[cfg(not(unix))] let mode = None; ("file".into(), Some(fs::read(&absolute).map_err(|error| format!("candidate read `{path}` failed: {error}"))?), mode) }, Some(meta) if meta.is_dir() => ("directory".into(), None, None), Some(_) => ("other".into(), None, None) };
            if untracked.contains(&path) { untracked_raw += bytes.as_deref().map_or(0, line_count); untracked_ws += bytes.as_deref().map_or(0, |candidate| whitespace_diff(&[], candidate).0); }
            let (head_mode, head_oid) = head_entry(&repository, &path)?; identities.push(PathIdentity { path: path.clone(), head_mode, head_oid, worktree_kind: kind, worktree_mode: mode, worktree_sha256: bytes.as_deref().map(digest_hex), bytes: bytes.as_ref().map_or(0, |value| value.len() as u64), untracked: untracked.contains(&path), index: index_entries.iter().filter(|entry| entry.path == path).cloned().collect() });
        }
        let (mut raw_additions, raw_deletions) = parse_numstat(&git_success(&repository, &["diff", "--no-renames", "--numstat", "HEAD", "--"])?)?; let (mut whitespace_additions, whitespace_deletions) = parse_numstat(&git_success(&repository, &["diff", "-w", "--no-renames", "--numstat", "HEAD", "--"])?)?; raw_additions += untracked_raw; whitespace_additions += untracked_ws;
        Ok(Self { commit, parents, tree, head_ref, refs, index_entries, index_clean: index_status.status.success(), worktree_clean: worktree_status.status.success(), untracked_clean: untracked.is_empty(), paths: identities, raw_additions, raw_deletions, whitespace_additions, whitespace_deletions })
    }
    fn state_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new(); put_string(&mut out, &self.commit); for value in &self.parents { put_string(&mut out, value); } out.push(0xff); put_string(&mut out, &self.tree); put_option(&mut out, &self.head_ref);
        for item in &self.refs { put_string(&mut out, &item.name); put_string(&mut out, &item.oid); } out.push(0xfe); for item in &self.index_entries { put_string(&mut out, &item.path); put_string(&mut out, &item.mode); put_string(&mut out, &item.oid); out.extend([item.stage, u8::from(item.intent_to_add)]); } out.push(0xfd); out.extend([u8::from(self.index_clean), u8::from(self.worktree_clean), u8::from(self.untracked_clean)]);
        for item in &self.paths { put_string(&mut out, &item.path); put_option(&mut out, &item.head_mode); put_option(&mut out, &item.head_oid); put_string(&mut out, &item.worktree_kind); put_option(&mut out, &item.worktree_mode); put_option(&mut out, &item.worktree_sha256); out.extend_from_slice(&item.bytes.to_be_bytes()); out.push(u8::from(item.untracked)); for index in &item.index { put_string(&mut out, &index.mode); put_string(&mut out, &index.oid); out.extend([index.stage, u8::from(index.intent_to_add)]); } out.push(0xfc); }
        for value in [self.raw_additions, self.raw_deletions, self.whitespace_additions, self.whitespace_deletions] { out.extend_from_slice(&value.to_be_bytes()); } out
    }
    pub fn binding(&self) -> CandidateBinding { CandidateBinding { state_sha256: digest_hex(&self.state_bytes()) } }
}

#[rustfmt::skip]
#[cfg(test)] mod tests {
    use super::{CandidateBinding, digest_hex, parse_index}; use std::collections::BTreeSet;
    #[test] fn candidate_identity_binds_commit_tree_index_and_paths() {
        let fixture = include_str!("../../../fixtures/evidence/status_candidate.v1.json"); for field in ["\"parents\": []", "\"head_state\": \"detached\"", "\"refs\": [", "\"index\": [", "\"intent_to_add\": false", "\"worktree_kind\": \"file\""] { assert!(fixture.contains(field)); }
        let expected = CandidateBinding { state_sha256: digest_hex(b"complete-state") }; let foreign = CandidateBinding { state_sha256: digest_hex(b"different-index-ref-worktree") };
        assert!(expected.matches(&expected)); assert!(!foreign.matches(&expected), "foreign or dirty candidate authenticates");
        let mut ita = BTreeSet::new(); ita.insert("intent.txt".into()); let entries = parse_index(b"100644 1111111111111111111111111111111111111111 1\tconflict.txt\0\
100644 2222222222222222222222222222222222222222 2\tconflict.txt\0\
100644 3333333333333333333333333333333333333333 3\tconflict.txt\0\
100644 e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 0\tintent.txt\0", &ita).unwrap(); assert_eq!(entries.iter().map(|entry| entry.stage).collect::<Vec<_>>(), [1, 2, 3, 0]); assert!(entries[3].intent_to_add);
    }
}
