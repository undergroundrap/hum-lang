use crate::{
    cleanup::OwnedResource,
    summary::{JobSummary, authenticate_platform_pair},
    workorder,
};
use hum_sha256::digest_hex;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

const SUMMARY_FILE: &str = "hum-evidence-summary.v2.json";
const MAX_REVIEW: Duration = Duration::from_secs(120);
const FORBIDDEN_STATUS_COMMANDS: &[&str] = &[
    "--log",
    "cargo",
    "rustc",
    "check_all",
    "Fast",
    "Exhaustive",
    "cache",
    "local-copy",
    "build",
    "workflow run",
    "rerun",
    "powershell",
    "pwsh",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusRequest {
    pub repository: String,
    pub run_id: u64,
    pub run_attempt: u64,
    pub ubuntu_job_id: u64,
    pub windows_job_id: u64,
    pub anchor: String,
    pub anchor_tree: String,
    pub gh: PathBuf,
    pub workorder: PathBuf,
}

pub fn request_from_environment() -> Result<StatusRequest, String> {
    let value = |name: &str| {
        std::env::var(name).map_err(|_| format!("missing authenticated status input `{name}`"))
    };
    Ok(StatusRequest {
        repository: value("HUM_STATUS_REPOSITORY")?,
        run_id: parse_positive(&value("HUM_STATUS_RUN_ID")?, "run ID")?,
        run_attempt: parse_positive(&value("HUM_STATUS_RUN_ATTEMPT")?, "run attempt")?,
        ubuntu_job_id: parse_positive(&value("HUM_STATUS_UBUNTU_JOB_ID")?, "Ubuntu job ID")?,
        windows_job_id: parse_positive(&value("HUM_STATUS_WINDOWS_JOB_ID")?, "Windows job ID")?,
        anchor: value("HUM_STATUS_ANCHOR")?,
        anchor_tree: value("HUM_ANCHOR_TREE")?,
        gh: value("HUM_STATUS_GH_PATH")?.into(),
        workorder: value("HUM_STATUS_WORKORDER")?.into(),
    })
}

#[derive(Debug, Clone)]
struct Artifact {
    id: u64,
    name: String,
    size: u64,
    expired: bool,
    digest: String,
}

fn ordinary_file(path: &Path) -> Result<Vec<u8>, String> {
    crate::shell::ordinary_file(path, "status_file")?;
    let meta = fs::symlink_metadata(path).map_err(|e| format!("file metadata unavailable: {e}"))?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err("path is not one ordinary file".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err("path is a Windows reparse point".into());
        }
    }
    fs::read(path).map_err(|e| format!("file read failed: {e}"))
}
fn parse_positive(value: &str, name: &str) -> Result<u64, String> {
    if value.is_empty()
        || value == "0"
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(format!("{name} is not canonical positive decimal"));
    }
    value.parse().map_err(|_| format!("{name} overflows"))
}
fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

trait Transport {
    fn run(&mut self, args: &[&str]) -> Result<Vec<u8>, String>;
    fn calls(&self) -> &[String];
}
struct Gh {
    path: PathBuf,
    calls: Vec<String>,
}

pub fn resolve_current_job_id() -> Result<u64, String> {
    let value = |name: &str| {
        std::env::var(name).map_err(|_| format!("missing authenticated producer input `{name}`"))
    };
    let gh: PathBuf = value("HUM_GH_PATH")?.into();
    ordinary_file(&gh)?;
    let repository = value("GITHUB_REPOSITORY")?;
    let run = parse_positive(&value("GITHUB_RUN_ID")?, "run ID")?;
    let attempt = parse_positive(&value("GITHUB_RUN_ATTEMPT")?, "run attempt")?;
    let sha = value("GITHUB_SHA")?;
    let platform = value("HUM_PLATFORM")?;
    let endpoint = format!("repos/{repository}/actions/runs/{run}/jobs?per_page=100");
    let mut transport = Gh {
        path: gh,
        calls: Vec::new(),
    };
    let records = lines(transport.run(&[
        "api",
        &endpoint,
        "--paginate",
        "--jq",
        ".jobs[] | [.id,.run_attempt,.head_sha,.status,.name] | @tsv",
    ])?)?;
    let matched = records
        .iter()
        .filter_map(|line| {
            let f = line.split('\t').collect::<Vec<_>>();
            (f.len() == 5
                && f[1] == attempt.to_string()
                && f[2] == sha
                && f[3] == "in_progress"
                && f[4]
                    == if platform == "ubuntu" {
                        "preflight (ubuntu-latest)"
                    } else {
                        "preflight (windows-latest)"
                    })
            .then_some(f[0])
        })
        .collect::<Vec<_>>();
    if matched.len() != 1 {
        return Err(format!(
            "current producer job cardinality is {}",
            matched.len()
        ));
    }
    parse_positive(matched[0], "numeric job ID")
}
impl Transport for Gh {
    fn run(&mut self, args: &[&str]) -> Result<Vec<u8>, String> {
        if args
            .iter()
            .any(|value| matches!(*value, "--log" | "workflow" | "rerun"))
            || FORBIDDEN_STATUS_COMMANDS
                .iter()
                .any(|forbidden| args.join(" ").contains(forbidden))
        {
            return Err("forbidden full-log, dispatch, or rerun transport".into());
        }
        self.calls.push(args.join(" "));
        let output = Command::new(&self.path)
            .args(args)
            .output()
            .map_err(|e| format!("GitHub CLI launch failed: {e}"))?;
        if !output.status.success() || !output.stderr.is_empty() {
            return Err(format!(
                "GitHub CLI failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        if !output.stderr.is_empty() {
            return Err("GitHub CLI emitted unexpected stderr".into());
        }
        Ok(output.stdout)
    }
    fn calls(&self) -> &[String] {
        &self.calls
    }
}

fn lines(bytes: Vec<u8>) -> Result<Vec<String>, String> {
    let text = String::from_utf8(bytes).map_err(|_| "GitHub metadata is not UTF-8")?;
    Ok(text
        .lines()
        .map(str::to_string)
        .filter(|line| !line.is_empty())
        .collect())
}
fn one(bytes: Vec<u8>, label: &str) -> Result<String, String> {
    let values = lines(bytes)?;
    if values.len() != 1 {
        return Err(format!("{label} metadata cardinality is {}", values.len()));
    }
    Ok(values[0].clone())
}
fn artifact(line: &str) -> Result<Artifact, String> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 5 {
        return Err("artifact metadata shape mismatch".into());
    }
    let digest = fields[4]
        .strip_prefix("sha256:")
        .ok_or("artifact digest algorithm mismatch")?
        .to_string();
    if !hash(&digest) {
        return Err("artifact digest is not lowercase SHA-256".into());
    }
    Ok(Artifact {
        id: parse_positive(fields[0], "artifact ID")?,
        name: fields[1].into(),
        size: parse_positive(fields[2], "artifact size")?,
        expired: match fields[3] {
            "true" => true,
            "false" => false,
            _ => return Err("artifact expiry is not Boolean".into()),
        },
        digest,
    })
}

fn authenticate_run<T: Transport>(
    transport: &mut T,
    request: &StatusRequest,
) -> Result<(), String> {
    let endpoint = format!(
        "repos/{}/actions/runs/{}",
        request.repository, request.run_id
    );
    let record = one(
        transport.run(&[
            "api",
            &endpoint,
            "--jq",
            "[.id,.run_attempt,.head_sha,.event,.path,.name,.status,.conclusion] | @tsv",
        ])?,
        "run",
    )?;
    let f = record.split('\t').collect::<Vec<_>>();
    if f.len() != 8 {
        return Err("run metadata shape mismatch".into());
    }
    if parse_positive(f[0], "run ID")? != request.run_id {
        return Err("run ID mismatch".into());
    }
    if parse_positive(f[1], "run attempt")? != request.run_attempt {
        return Err("run attempt mismatch".into());
    }
    for (name, actual, expected) in [
        ("run SHA", f[2], request.anchor.as_str()),
        ("run event", f[3], "push"),
        ("workflow path", f[4], ".github/workflows/ci.yml"),
        ("workflow name", f[5], "ci"),
        ("run status", f[6], "completed"),
        ("run conclusion", f[7], "success"),
    ] {
        if actual != expected {
            return Err(format!("{name} mismatch"));
        }
    }
    Ok(())
}
fn authenticate_job<T: Transport>(
    transport: &mut T,
    request: &StatusRequest,
    id: u64,
    platform: &str,
) -> Result<(), String> {
    let endpoint = format!("repos/{}/actions/jobs/{id}", request.repository);
    let record = one(
        transport.run(&[
            "api",
            &endpoint,
            "--jq",
            "[.id,.run_id,.run_attempt,.head_sha,.status,.conclusion,.name] | @tsv",
        ])?,
        "job",
    )?;
    let f = record.split('\t').collect::<Vec<_>>();
    if f.len() != 7 {
        return Err(format!("{platform} job metadata shape mismatch"));
    }
    if parse_positive(f[0], "job ID")? != id {
        return Err(format!("{platform} job ID mismatch"));
    }
    if parse_positive(f[1], "job run ID")? != request.run_id {
        return Err(format!("{platform} job run ID mismatch"));
    }
    if parse_positive(f[2], "job attempt")? != request.run_attempt {
        return Err(format!("{platform} job attempt mismatch"));
    }
    let expected_name = if platform == "ubuntu" {
        "preflight (ubuntu-latest)"
    } else {
        "preflight (windows-latest)"
    };
    for (name, actual, expected) in [
        ("SHA", f[3], request.anchor.as_str()),
        ("status", f[4], "completed"),
        ("conclusion", f[5], "success"),
        ("name", f[6], expected_name),
    ] {
        if actual != expected {
            return Err(format!("{platform} job {name} mismatch"));
        }
    }
    Ok(())
}
fn summaries<T: Transport>(
    transport: &mut T,
    request: &StatusRequest,
) -> Result<Vec<Artifact>, String> {
    let endpoint = format!(
        "repos/{}/actions/runs/{}/artifacts?per_page=100",
        request.repository, request.run_id
    );
    lines(transport.run(&[
        "api",
        &endpoint,
        "--paginate",
        "--jq",
        ".artifacts[] | [.id,.name,.size_in_bytes,.expired,.digest] | @tsv",
    ])?)?
    .iter()
    .map(|line| artifact(line))
    .collect()
}
fn select<'a>(values: &'a [Artifact], name: &str) -> Result<&'a Artifact, String> {
    let matches = values
        .iter()
        .filter(|value| value.name == name)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!(
            "summary artifact `{name}` cardinality is {}",
            matches.len()
        ));
    }
    let value = matches[0];
    if value.expired || value.size == 0 {
        return Err(format!("summary artifact `{name}` is expired or empty"));
    }
    Ok(value)
}
fn download<T: Transport>(
    transport: &mut T,
    request: &StatusRequest,
    artifact: &Artifact,
    root: &Path,
    platform: &str,
) -> Result<JobSummary, String> {
    let archive_endpoint = format!(
        "repos/{}/actions/artifacts/{}/zip",
        request.repository, artifact.id
    );
    if digest_hex(&transport.run(&["api", &archive_endpoint])?) != artifact.digest {
        return Err(format!("{platform} summary GitHub digest mismatch"));
    }
    let directory = root.join(platform);
    fs::create_dir(&directory).map_err(|e| format!("summary directory creation failed: {e}"))?;
    let run = request.run_id.to_string();
    let destination = directory.to_str().ok_or("summary path is not UTF-8")?;
    transport.run(&[
        "run",
        "download",
        &run,
        "--repo",
        &request.repository,
        "--name",
        &artifact.name,
        "--dir",
        destination,
    ])?;
    let entries = fs::read_dir(&directory)
        .map_err(|e| format!("summary directory read failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if entries.len() != 1 || entries[0].file_name() != SUMMARY_FILE {
        return Err("summary archive does not contain exactly the canonical file".into());
    }
    JobSummary::parse_canonical(&ordinary_file(&entries[0].path())?)
}

fn authenticate_summary(
    value: &JobSummary,
    request: &StatusRequest,
    job: u64,
    artifact: &Artifact,
) -> Result<(), String> {
    value.validate()?;
    if value.run_id != request.run_id {
        return Err(format!("{} summary run ID mismatch", value.platform));
    }
    if value.run_attempt != request.run_attempt {
        return Err(format!("{} summary run attempt mismatch", value.platform));
    }
    if value.job_id != job {
        return Err(format!("{} summary job ID mismatch", value.platform));
    }
    if value.commit != request.anchor {
        return Err(format!("{} summary commit mismatch", value.platform));
    }
    if value.tree != request.anchor_tree {
        return Err(format!("{} summary tree mismatch", value.platform));
    }
    if value.artifact_name() != artifact.name {
        return Err(format!("{} summary artifact-name mismatch", value.platform));
    }
    if !hash(&artifact.digest) {
        return Err(format!(
            "{} summary GitHub digest shape mismatch",
            value.platform
        ));
    }
    if artifact.id == 0 {
        return Err(format!("{} summary artifact ID is zero", value.platform));
    }
    if artifact.size == 0 {
        return Err(format!("{} summary artifact size is zero", value.platform));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct RuntimeIdentity {
    sha256: String,
    target: String,
    toolchain: String,
}
#[derive(Debug, Clone)]
struct ToolIdentity {
    path: PathBuf,
    sha256: String,
    version_sha256: String,
}

pub fn run(request: &StatusRequest) -> Result<String, String> {
    if !request.gh.is_absolute() {
        return Err("GitHub CLI path is not absolute".into());
    }
    let gh_bytes = ordinary_file(&request.gh)?;
    let gh_version = Command::new(&request.gh)
        .arg("--version")
        .output()
        .map_err(|e| format!("GitHub CLI version launch failed: {e}"))?;
    if !gh_version.status.success()
        || !gh_version.stderr.is_empty()
        || gh_version.stdout.is_empty()
        || std::str::from_utf8(&gh_version.stdout).is_err()
    {
        return Err("GitHub CLI version authentication failed".into());
    }
    let self_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let runtime = RuntimeIdentity {
        sha256: digest_hex(&ordinary_file(&self_path)?),
        target: option_env!("HUM_BUILD_TARGET")
            .ok_or("running executable has no authenticated build target")?
            .into(),
        toolchain: option_env!("HUM_BUILD_TOOLCHAIN")
            .ok_or("running executable has no authenticated build toolchain")?
            .into(),
    };
    let tool = ToolIdentity {
        path: request.gh.clone(),
        sha256: digest_hex(&gh_bytes),
        version_sha256: digest_hex(&gh_version.stdout),
    };
    let mut transport = Gh {
        path: request.gh.clone(),
        calls: Vec::new(),
    };
    run_with_transport(&mut transport, request, &runtime, &tool)
}

fn run_with_transport<T: Transport>(
    transport: &mut T,
    request: &StatusRequest,
    runtime: &RuntimeIdentity,
    tool: &ToolIdentity,
) -> Result<String, String> {
    let start = Instant::now();
    authenticate_run(transport, request)?;
    authenticate_job(transport, request, request.ubuntu_job_id, "ubuntu")?;
    authenticate_job(transport, request, request.windows_job_id, "windows")?;
    let artifacts = summaries(transport, request)?;
    let ubuntu_name = format!(
        "hum-evidence-summary-v2-{}-{}-{}-ubuntu",
        request.run_id, request.run_attempt, request.ubuntu_job_id
    );
    let windows_name = format!(
        "hum-evidence-summary-v2-{}-{}-{}-windows",
        request.run_id, request.run_attempt, request.windows_job_id
    );
    let ubuntu_artifact = select(&artifacts, &ubuntu_name)?;
    let windows_artifact = select(&artifacts, &windows_name)?;
    if ubuntu_artifact.id == windows_artifact.id {
        return Err("summary artifacts have duplicate IDs".into());
    }
    let mut owned = OwnedResource::create("status-summary").map_err(|e| e.to_string())?;
    let result = (|| {
        let ubuntu = download(transport, request, ubuntu_artifact, owned.path(), "ubuntu")?;
        let windows = download(
            transport,
            request,
            windows_artifact,
            owned.path(),
            "windows",
        )?;
        authenticate_summary(&ubuntu, request, request.ubuntu_job_id, ubuntu_artifact)?;
        authenticate_summary(&windows, request, request.windows_job_id, windows_artifact)?;
        if ubuntu.schema != crate::summary::JOB_SCHEMA_V2
            || windows.schema != crate::summary::JOB_SCHEMA_V2
        {
            return Err("Unit C status requires an exact v2 summary pair".into());
        }
        authenticate_platform_pair(&ubuntu, &windows)?;
        let current = if cfg!(windows) { &windows } else { &ubuntu };
        if runtime.sha256 != current.producer_executable_sha256 {
            return Err("running executable and producer summary disagree".into());
        }
        if current.target != runtime.target {
            return Err("running executable and producer target disagree".into());
        }
        if current.toolchain != runtime.toolchain {
            return Err("running executable and producer toolchain disagree".into());
        }
        let workorder = ordinary_file(&request.workorder)?;
        let immutable = workorder::immutable_projection(&workorder)?;
        let endpoint = format!(
            "repos/{}/contents/workorders/active/WORKORDER_25.md?ref={}",
            request.repository, request.anchor
        );
        let anchor_workorder = transport.run(&[
            "api",
            "-H",
            "Accept: application/vnd.github.raw+json",
            &endpoint,
        ])?;
        if workorder::immutable_projection(&anchor_workorder)? != immutable {
            return Err("Work Order immutable-region projection mismatch".into());
        }
        if start.elapsed() >= MAX_REVIEW {
            return Err("status-review performance boundary exceeded".into());
        }
        if transport.calls().iter().any(|call| {
            call.contains("cargo")
                || FORBIDDEN_STATUS_COMMANDS
                    .iter()
                    .any(|value| call.contains(value))
        }) {
            return Err("forbidden status-stage sentinel recorded access".into());
        }
        Ok(format!(
            "status_authenticated|anchor={}|tree={}|run_id={}|run_attempt={}|ubuntu_job_id={}|windows_job_id={}|ubuntu_artifact_id={}|windows_artifact_id={}|gh_path={}|gh_version_sha256={}|gh_sha256={}|immutable_projection_sha256={}|elapsed_ms={}\n",
            request.anchor,
            request.anchor_tree,
            request.run_id,
            request.run_attempt,
            request.ubuntu_job_id,
            request.windows_job_id,
            ubuntu_artifact.id,
            windows_artifact.id,
            tool.path.display(),
            tool.version_sha256,
            tool.sha256,
            digest_hex(&immutable),
            start.elapsed().as_millis()
        ))
    })();
    owned
        .close()
        .map_err(|e| format!("summary cleanup failed: {e}"))?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[rustfmt::skip]
    #[derive(Clone)]
    struct Exchange { argv: Vec<String>, stdout: Vec<u8>, stderr: Vec<u8>, exit: i32, file: Option<Vec<u8>> }
    #[rustfmt::skip]
    struct Controlled { calls: Vec<String>, queue: VecDeque<Exchange> }
    #[rustfmt::skip]
    impl Transport for Controlled {
        fn run(&mut self, args: &[&str]) -> Result<Vec<u8>, String> {
            let joined=args.join(" "); self.calls.push(joined.clone());
            if FORBIDDEN_STATUS_COMMANDS.iter().any(|value|joined.contains(value)){return Err("forbidden controlled transport request".into())}
            let expected=self.queue.pop_front().ok_or("missing controlled transcript record")?; let mut actual=args.iter().map(|v|(*v).to_string()).collect::<Vec<_>>();
            if let Some(bytes)=expected.file.as_ref(){let at=actual.iter().position(|v|v=="--dir").ok_or("download destination is absent")?+1;let path=PathBuf::from(&actual[at]);let platform=expected.argv[at].strip_prefix("$owned/").ok_or("controlled destination placeholder is invalid")?;if path.file_name().and_then(|v|v.to_str())!=Some(platform)||!path.is_absolute()||!path.is_dir(){return Err("owned download destination mismatch".into())}actual[at]=expected.argv[at].clone();if actual!=expected.argv{return Err("controlled transport argv mismatch".into())}fs::write(path.join(SUMMARY_FILE),bytes).map_err(|e|e.to_string())?;}else if actual!=expected.argv{return Err("controlled transport argv mismatch".into())}
            if expected.exit!=0{return Err(format!("controlled transport exit {}",expected.exit))}
            if !expected.stderr.is_empty(){return Err("controlled transport stderr is nonempty".into())}Ok(expected.stdout)
        }
        fn calls(&self)->&[String]{&self.calls}
    }
    #[rustfmt::skip]
    fn words(values:&[&str])->Vec<String>{values.iter().map(|v|(*v).to_string()).collect()}
    #[rustfmt::skip]
    fn exchange(argv:Vec<String>,stdout:Vec<u8>)->Exchange{Exchange{argv,stdout,stderr:vec![],exit:0,file:None}}
    #[rustfmt::skip]
    fn summary_bytes(name:&str,fallback:&'static [u8])->Vec<u8>{std::env::var_os(name).map(fs::read).transpose().unwrap().unwrap_or_else(||fallback.to_vec())}
    #[rustfmt::skip]
    #[derive(Clone,Copy)] enum Corruption{None,Missing,Reordered,Duplicated,Extra,Red,Malformed,Forbidden,Stderr,Exit}

    #[rustfmt::skip]
    fn controlled_case(corruption:Corruption)->Result<String,String>{
        let ub=summary_bytes("HUM_TEST_STATUS_UBUNTU_SUMMARY",include_bytes!("../../../fixtures/evidence/job_summary_ubuntu.v2.json"));let wb=summary_bytes("HUM_TEST_STATUS_WINDOWS_SUMMARY",include_bytes!("../../../fixtures/evidence/job_summary_windows.v2.json"));let u=JobSummary::parse_canonical(&ub)?;let w=JobSummary::parse_canonical(&wb)?;let workorder=include_bytes!("../../../workorders/active/WORKORDER_25.md");let mut owned=OwnedResource::create("controlled-transcript").map_err(|e|e.to_string())?;let path=owned.write("WORKORDER_25.md",workorder).map_err(|e|e.to_string())?;let request=StatusRequest{repository:"example/hum-lang".into(),run_id:u.run_id,run_attempt:u.run_attempt,ubuntu_job_id:u.job_id,windows_job_id:w.job_id,anchor:u.commit.clone(),anchor_tree:u.tree.clone(),gh:"/controlled/gh".into(),workorder:path};let uraw=b"ubuntu archive".to_vec();let wraw=b"windows archive".to_vec();let artifacts=format!("91\t{}\t{}\tfalse\tsha256:{}\n92\t{}\t{}\tfalse\tsha256:{}\n",u.artifact_name(),ub.len(),digest_hex(&uraw),w.artifact_name(),wb.len(),digest_hex(&wraw)).into_bytes();let run=format!("{}\t{}\t{}\tpush\t.github/workflows/ci.yml\tci\tcompleted\tsuccess\n",u.run_id,u.run_attempt,u.commit).into_bytes();let uj=format!("{}\t{}\t{}\t{}\tcompleted\tsuccess\tpreflight (ubuntu-latest)\n",u.job_id,u.run_id,u.run_attempt,u.commit).into_bytes();let wj=format!("{}\t{}\t{}\t{}\tcompleted\tsuccess\tpreflight (windows-latest)\n",w.job_id,w.run_id,w.run_attempt,w.commit).into_bytes();let run_id=u.run_id.to_string();let ujid=u.job_id.to_string();let wjid=w.job_id.to_string();let run_endpoint=format!("repos/{}/actions/runs/{}",request.repository,run_id);let uj_endpoint=format!("repos/{}/actions/jobs/{}",request.repository,ujid);let wj_endpoint=format!("repos/{}/actions/jobs/{}",request.repository,wjid);let artifacts_endpoint=format!("repos/{}/actions/runs/{}/artifacts?per_page=100",request.repository,run_id);let ua=format!("repos/{}/actions/artifacts/91/zip",request.repository);let wa=format!("repos/{}/actions/artifacts/92/zip",request.repository);let remote=format!("repos/{}/contents/workorders/active/WORKORDER_25.md?ref={}",request.repository,request.anchor);let mut records=vec![exchange(words(&["api",&run_endpoint,"--jq","[.id,.run_attempt,.head_sha,.event,.path,.name,.status,.conclusion] | @tsv"]),run),exchange(words(&["api",&uj_endpoint,"--jq","[.id,.run_id,.run_attempt,.head_sha,.status,.conclusion,.name] | @tsv"]),uj),exchange(words(&["api",&wj_endpoint,"--jq","[.id,.run_id,.run_attempt,.head_sha,.status,.conclusion,.name] | @tsv"]),wj),exchange(words(&["api",&artifacts_endpoint,"--paginate","--jq",".artifacts[] | [.id,.name,.size_in_bytes,.expired,.digest] | @tsv"]),artifacts),exchange(words(&["api",&ua]),uraw),Exchange{argv:words(&["run","download",&run_id,"--repo",&request.repository,"--name",&u.artifact_name(),"--dir","$owned/ubuntu"]),stdout:vec![],stderr:vec![],exit:0,file:Some(ub)},exchange(words(&["api",&wa]),wraw),Exchange{argv:words(&["run","download",&run_id,"--repo",&request.repository,"--name",&w.artifact_name(),"--dir","$owned/windows"]),stdout:vec![],stderr:vec![],exit:0,file:Some(wb)},exchange(words(&["api","-H","Accept: application/vnd.github.raw+json",&remote]),workorder.to_vec())];match corruption{Corruption::None=>{},Corruption::Missing=>{records.pop();},Corruption::Reordered=>records.swap(0,1),Corruption::Duplicated=>records.insert(0,records[0].clone()),Corruption::Extra=>records.push(records[0].clone()),Corruption::Red=>records[0].stdout=String::from_utf8(records[0].stdout.clone()).unwrap().replace("success","failure").into_bytes(),Corruption::Malformed=>records[0].stdout=vec![0xff],Corruption::Forbidden=>records[0].argv.push("--log".into()),Corruption::Stderr=>records[0].stderr=b"unexpected".to_vec(),Corruption::Exit=>records[0].exit=1};let current=if cfg!(windows){&w}else{&u};let runtime=RuntimeIdentity{sha256:current.producer_executable_sha256.clone(),target:current.target.clone(),toolchain:current.toolchain.clone()};let tool=ToolIdentity{path:request.gh.clone(),sha256:"a".repeat(64),version_sha256:"b".repeat(64)};let mut transport=Controlled{calls:vec![],queue:records.into()};let result=run_with_transport(&mut transport,&request,&runtime,&tool).and_then(|value|if transport.queue.is_empty(){Ok(value)}else{Err("leftover controlled transcript record".into())});owned.close().map_err(|e|e.to_string())?;result
    }

    #[rustfmt::skip]
    fn delegation_contract(source:&str)->Result<(),String>{let marker="run_with_transport(&mut transport, request, &runtime, &tool)";if source.matches(marker).count()!=1{return Err("public status delegation cardinality mismatch".into())}for forbidden in ["HUM_TEST_STATUS_","Controlled","controlled_case("]{if source.contains(forbidden){return Err("production contains a controlled-transport selector".into())}}Ok(())}

    #[rustfmt::skip]
    #[test]
    fn job_summary_binds_run_attempt_job_sha_tree_and_platform(){let request=StatusRequest{repository:"example/hum-lang".into(),run_id:71,run_attempt:2,ubuntu_job_id:81,windows_job_id:82,anchor:"a".repeat(40),anchor_tree:"c".repeat(40),gh:"gh".into(),workorder:"workorder".into()};let endpoint="repos/example/hum-lang/actions/runs/71";let mut changed=Controlled{calls:vec![],queue:vec![exchange(words(&["api",endpoint,"--jq","[.id,.run_attempt,.head_sha,.event,.path,.name,.status,.conclusion] | @tsv"]),format!("70\t2\t{}\tpush\t.github/workflows/ci.yml\tci\tcompleted\tsuccess\n",request.anchor).into_bytes())].into()};assert!(authenticate_run(&mut changed,&request).is_err(),"run substitution authenticated");assert!(controlled_case(Corruption::None).unwrap().starts_with("status_authenticated|"));for corruption in [Corruption::Missing,Corruption::Reordered,Corruption::Duplicated,Corruption::Extra,Corruption::Red,Corruption::Malformed,Corruption::Forbidden,Corruption::Stderr,Corruption::Exit]{assert!(controlled_case(corruption).is_err(),"controlled transport corruption authenticated")}}

    #[rustfmt::skip]
    #[test]
    fn status_review_consumes_summaries_without_full_logs(){let production=include_str!("status.rs").split_once("#[cfg(test)]").unwrap().0;delegation_contract(production).unwrap();let marker="run_with_transport(&mut transport, request, &runtime, &tool)";assert!(delegation_contract(&production.replacen(marker,"Err(\"diverted\".into())",1)).is_err());assert!(production.contains("matches!(*value, \"--log\" | \"workflow\" | \"rerun\")"),"the no-log-download sentinel records forbidden access");assert!(production.contains("call.contains(\"cargo\")"),"a forbidden stage appears without rejection");}
}
