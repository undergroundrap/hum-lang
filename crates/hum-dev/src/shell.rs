use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub(crate) use crate::summary::PwshIdentity;

const ENVIRONMENT_KEYS: &str = "CARGO_HOME,COMSPEC,HOME,OS,PATH,PATHEXT,ProgramFiles,ProgramFiles(x86),ProgramData,PSModulePath,RUNNER_TEMP,RUSTUP_HOME,RUSTUP_TOOLCHAIN,SystemRoot,TEMP,TMP,TMPDIR,USERPROFILE,WINDIR,CI,GITHUB_ACTIONS,GITHUB_ACTION,GITHUB_ACTOR,GITHUB_API_URL,GITHUB_ENV,GITHUB_EVENT_NAME,GITHUB_EVENT_PATH,GITHUB_GRAPHQL_URL,GITHUB_JOB,GITHUB_OUTPUT,GITHUB_PATH,GITHUB_REF,GITHUB_REPOSITORY,GITHUB_RUN_ATTEMPT,GITHUB_RUN_ID,GITHUB_SERVER_URL,GITHUB_SHA,GITHUB_STEP_SUMMARY,GITHUB_TOKEN,GITHUB_WORKFLOW,GITHUB_WORKSPACE,RUNNER_ARCH,RUNNER_OS,RUNNER_TOOL_CACHE,HUM_CANONICAL_SEAL_EVIDENCE_TIER,HUM_EVIDENCE_RECEIPT,HUM_BUILD_TARGET,HUM_BUILD_TOOLCHAIN,INCLUDE,LIB,LIBPATH,VCINSTALLDIR,VCToolsInstallDir,VSCMD_ARG_HOST_ARCH,VSCMD_ARG_TGT_ARCH,WindowsSdkDir,WindowsSDKVersion,GIT_CONFIG_COUNT,GIT_CONFIG_KEY_0,GIT_CONFIG_VALUE_0";
const WINDOWS_TOOLCHAIN_KEYS: &str = "INCLUDE,LIB,LIBPATH,VCINSTALLDIR,VCToolsInstallDir,VSCMD_ARG_HOST_ARCH,VSCMD_ARG_TGT_ARCH,WindowsSdkDir,WindowsSDKVersion";
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellEnvironment(pub(crate) BTreeMap<OsString, OsString>);

impl ShellEnvironment {
    pub fn from_process(repository: &Path) -> Result<Self, String> {
        let mut values = BTreeMap::new();
        for key in ENVIRONMENT_KEYS.split(',') {
            if let Some(value) = std::env::var_os(key) {
                values.insert(OsString::from(key), value);
            }
        }
        for required in ["PATH", "PSModulePath"] {
            if !values.contains_key(OsStr::new(required)) {
                return Err(format!("environment_required: missing {required}"));
            }
        }
        values.insert("GIT_CONFIG_COUNT".into(), "1".into());
        values.insert("GIT_CONFIG_KEY_0".into(), "safe.directory".into());
        values.insert("GIT_CONFIG_VALUE_0".into(), repository.as_os_str().into());
        Ok(Self(values))
    }
    pub(crate) fn get(&self, key: &str) -> Result<&OsStr, String> {
        self.0
            .get(OsStr::new(key))
            .map(OsString::as_os_str)
            .ok_or_else(|| format!("environment_required: missing {key}"))
    }
    pub(crate) fn authenticate(&self, repository: &Path) -> Result<(), String> {
        let allowed = ENVIRONMENT_KEYS.split(',').collect::<BTreeSet<_>>();
        let mut folded = BTreeSet::new();
        for key in self.0.keys() {
            let key = key
                .to_str()
                .ok_or_else(|| "environment_key: value is not UTF-8".to_string())?;
            let comparison = if cfg!(windows) {
                key.to_ascii_lowercase()
            } else {
                key.to_string()
            };
            if !folded.insert(comparison) {
                return Err(format!("environment_key: duplicate case-variant {key}"));
            }
        }
        for (key, value) in &self.0 {
            let key = key
                .to_str()
                .ok_or_else(|| "environment_key: value is not UTF-8".to_string())?;
            if !allowed.contains(key) {
                return Err(format!("environment_key: unsupported {key}"));
            }
            let value = value
                .to_str()
                .ok_or_else(|| format!("environment_value: {key} is not UTF-8"))?;
            if value.bytes().any(|b| matches!(b, 0 | b'\r' | b'\n')) {
                return Err(format!("environment_value: {key} contains CR, LF, or NUL"));
            }
        }
        self.get("PATH")?;
        self.get("PSModulePath")?;
        if let Some(token) = self.0.get(OsStr::new("GITHUB_TOKEN"))
            && token.is_empty()
        {
            return Err("credential_identity: GITHUB_TOKEN is empty".into());
        }
        if self.get("GIT_CONFIG_COUNT")? != "1"
            || self.get("GIT_CONFIG_KEY_0")? != "safe.directory"
            || Path::new(self.get("GIT_CONFIG_VALUE_0")?) != repository
        {
            return Err("repository_trust: command-local safe.directory binding differs".into());
        }
        self.authenticate_windows_toolchain()?;
        Ok(())
    }
    #[cfg(not(windows))]
    fn authenticate_windows_toolchain(&self) -> Result<(), String> {
        let os = OsStr::new("OS");
        let forged = self.0.get(os).is_some_and(|v| v == "Windows_NT");
        let mut keys = WINDOWS_TOOLCHAIN_KEYS.split(',');
        let tools = keys.any(|k| self.0.contains_key(OsStr::new(k)));
        if forged || tools {
            return Err("windows_toolchain: bindings supplied on non-Windows path".into());
        }
        Ok(())
    }
    #[cfg(windows)]
    fn authenticate_windows_toolchain(&self) -> Result<(), String> {
        if self.get("OS")? != "Windows_NT" {
            return Err("windows_toolchain: OS must be Windows_NT".into());
        }
        for key in WINDOWS_TOOLCHAIN_KEYS.split(',') {
            self.get(key)
                .map_err(|_| format!("windows_toolchain: missing {key}"))?;
        }
        for key in ["INCLUDE", "LIB", "LIBPATH"] {
            for path in std::env::split_paths(self.get(key)?) {
                if !path.is_absolute() || !path.is_dir() {
                    return Err(format!("windows_toolchain: invalid {key} path"));
                }
            }
        }
        for key in ["VCINSTALLDIR", "VCToolsInstallDir", "WindowsSdkDir"] {
            let path = Path::new(self.get(key)?);
            if !path.is_absolute() || !path.is_dir() {
                return Err(format!("windows_toolchain: invalid {key}"));
            }
        }
        if !Path::new(self.get("VCToolsInstallDir")?)
            .starts_with(Path::new(self.get("VCINSTALLDIR")?))
        {
            return Err("windows_toolchain: substituted toolchain directory".into());
        }
        for key in ["VSCMD_ARG_HOST_ARCH", "VSCMD_ARG_TGT_ARCH"] {
            if self.get(key)? != "x64" {
                return Err(format!("windows_toolchain: {key} must be x64"));
            }
        }
        let expected = Path::new(self.get("VCToolsInstallDir")?)
            .join("bin/Hostx64/x64/link.exe")
            .canonicalize()
            .map_err(|_| "windows_toolchain: link.exe absent".to_string())?;
        ordinary_file(&expected, "windows_linker")?;
        let mut effective = None;
        for dir in std::env::split_paths(self.get("PATH")?) {
            let candidate = dir.join("link.exe");
            if candidate.exists() {
                ordinary_file(&candidate, "windows_linker")?;
                effective = Some(candidate);
                break;
            }
        }
        let effective = effective.ok_or("windows_toolchain: link.exe unreachable")?;
        if !same_ordinary_file(&expected, &effective)? {
            return Err("windows_toolchain: substituted effective link.exe".into());
        }
        Ok(())
    }
}
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)] pub(crate) struct ExecutableBinding { path: PathBuf, identity: String, length: usize, sha256: String, predicate: &'static str }
#[rustfmt::skip]
impl ExecutableBinding {
    pub(crate) fn authenticate(path: PathBuf, predicate: &'static str) -> Result<Self, String> {
        ordinary_file(&path, predicate)?; let initial_identity = stable_file_identity(&path)?; let initial_bytes = fs::read(&path).map_err(|e| format!("{predicate}: {e}"))?; let path = path.canonicalize().map_err(|e| format!("{predicate}: {e}"))?; ordinary_file(&path, predicate)?; let identity = stable_file_identity(&path)?; let bytes = fs::read(&path).map_err(|e| format!("{predicate}: {e}"))?; if initial_identity != identity { return Err(format!("{predicate}: file identity changed during canonicalization")); } else if initial_bytes.len() != bytes.len() { return Err(format!("{predicate}: size changed during canonicalization")); } else if hum_sha256::digest_hex(&initial_bytes) != hum_sha256::digest_hex(&bytes) { return Err(format!("{predicate}: digest changed during canonicalization")); }
        let binding = Self { path, identity, length: bytes.len(), sha256: hum_sha256::digest_hex(&bytes), predicate }; binding.reauthenticate()?; Ok(binding) }
    pub(crate) fn reauthenticate(&self) -> Result<(), String> {
        ordinary_file(&self.path, self.predicate)?; let identity = stable_file_identity(&self.path)?; let bytes = fs::read(&self.path).map_err(|e| format!("{}: {e}", self.predicate))?;
        if bytes.len() != self.length { return Err(format!("{}: size changed before launch", self.predicate)); } else if identity != self.identity || stable_file_identity(&self.path)? != identity { return Err(format!("{}: file identity changed before launch", self.predicate)); } else if hum_sha256::digest_hex(&bytes) != self.sha256 { return Err(format!("{}: digest changed before launch", self.predicate)); } Ok(()) }
    pub(crate) fn path(&self) -> &Path { &self.path } pub(crate) fn predicate(&self) -> &str { self.predicate } pub(crate) fn sha256(&self) -> &str { &self.sha256 } pub(crate) fn command(&self) -> Result<Command,String> { self.reauthenticate()?; Ok(Command::new(&self.path)) }
}
pub(crate) fn same_ordinary_file(left: &Path, right: &Path) -> Result<bool, String> {
    ordinary_file(left, "windows_linker")?;
    ordinary_file(right, "windows_linker")?;
    let left = left.canonicalize().map_err(|e| e.to_string())?;
    let right = right.canonicalize().map_err(|e| e.to_string())?;
    let folded = left
        .to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy());
    Ok((left == right || cfg!(windows) && folded)
        && stable_file_identity(&left)? == stable_file_identity(&right)?)
}

pub(crate) fn stable_file_identity(path: &Path) -> Result<String, String> {
    ordinary_file(path, "executable_identity")?;
    #[cfg(windows)]
    return Ok(format!("{:?}", &windows_information(path, true)?[7..]));
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = fs::metadata(path).map_err(|e| format!("executable_identity: {e}"))?;
        Ok(format!("{}:{}", metadata.dev(), metadata.ino()))
    }
}

fn ordinary_windows_attributes(attributes: u32, links: u32) -> bool {
    attributes & 0x400 == 0 && links == 1
}

#[cfg(windows)]
fn windows_information(path: &Path, is_file: bool) -> Result<[u32; 13], String> {
    use std::os::windows::io::AsRawHandle;
    unsafe extern "system" {
        fn GetFileInformationByHandle(handle: *mut std::ffi::c_void, information: *mut u32) -> i32;
    }
    if !is_file {
        return Ok([0; 13]);
    }
    let file = fs::File::open(path).map_err(|e| format!("executable_identity: {e}"))?;
    let mut information = [0_u32; 13];
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) } == 0 {
        Err("executable_identity: file identity unavailable".into())
    } else {
        Ok(information)
    }
}

#[cfg(windows)]
pub(crate) fn fixed_system_command() -> Result<PathBuf, String> {
    use std::os::windows::{ffi::OsStringExt, fs::MetadataExt};
    unsafe extern "system" {
        fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    }
    let mut buffer = [0_u16; 32768];
    let length = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) } as usize;
    if length == 0 || length >= buffer.len() {
        return Err("system_command: native system directory unavailable".into());
    }
    let path = PathBuf::from(OsString::from_wide(&buffer[..length])).join("cmd.exe");
    let metadata = fs::symlink_metadata(&path).map_err(|e| format!("system_command: {e}"))?;
    let attributes = metadata.file_attributes();
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || attributes & 0x400 != 0
    {
        return Err("system_command: path is not a fixed ordinary file".into());
    }
    let read = || fs::read(&path).map_err(|e| format!("system_command: {e}"));
    let first = windows_information(&path, true)?;
    let first_digest = hum_sha256::digest(&read()?);
    let second = windows_information(&path, true)?;
    let second_digest = hum_sha256::digest(&read()?);
    if first[7..] != second[7..] || first_digest != second_digest {
        return Err("system_command: file identity changed before launch".into());
    }
    Ok(path)
}

#[cfg(windows)]
pub(crate) fn known_folder(folder: i32, label: &str) -> Result<PathBuf, String> {
    use std::os::windows::{ffi::OsStringExt, fs::MetadataExt};
    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHGetFolderPathW(
            owner: *mut std::ffi::c_void,
            folder: i32,
            token: *mut std::ffi::c_void,
            flags: u32,
            path: *mut u16,
        ) -> i32;
    }
    let mut buffer = [0_u16; 260];
    if unsafe {
        SHGetFolderPathW(
            std::ptr::null_mut(),
            folder,
            std::ptr::null_mut(),
            0,
            buffer.as_mut_ptr(),
        )
    } != 0
    {
        return Err(format!("{label}: native known folder unavailable"));
    }
    let length = buffer
        .iter()
        .position(|unit| *unit == 0)
        .ok_or_else(|| format!("{label}: native path unterminated"))?;
    let path = PathBuf::from(OsString::from_wide(&buffer[..length]));
    let metadata = fs::symlink_metadata(&path).map_err(|e| format!("{label}: {e}"))?;
    if !path.is_absolute() || !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 {
        return Err(format!(
            "{label}: known folder is not an ordinary absolute directory"
        ));
    }
    Ok(path)
}

#[rustfmt::skip]
pub(crate) fn ordinary_file(path: &Path, predicate: &str) -> Result<(), String> {
    if !path.is_absolute() { return Err(format!("{predicate}: path is not absolute")); }
    for component in path.ancestors().skip(1).collect::<Vec<_>>().into_iter().rev() {
        let metadata = fs::symlink_metadata(component).map_err(|e| format!("{predicate}: {e}"))?;
        #[cfg(windows)] let linked = std::os::windows::fs::MetadataExt::file_attributes(&metadata) & 0x400 != 0;
        #[cfg(unix)] let linked = metadata.file_type().is_symlink();
        if !metadata.is_dir() || linked { return Err(format!("{predicate}: path component is not an ordinary directory")); }
    }
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{predicate}: {e}"))?;
    #[cfg(windows)]
    let ordinary = ordinary_windows_attributes(std::os::windows::fs::MetadataExt::file_attributes(&metadata), windows_information(path, metadata.file_type().is_file())?[10]);
    #[cfg(unix)]
    let ordinary = std::os::unix::fs::MetadataExt::nlink(&metadata) == 1;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() || !ordinary { return Err(format!("{predicate}: path is not an ordinary file")); }
    Ok(())
}

#[cfg(all(test, windows))]#[rustfmt::skip]mod fixed_tests{use super::{fixed_system_command,ordinary_file,windows_information};use std::fs;#[test]fn native_cmd_is_fixed_while_substitutable_files_remain_single_link(){let command=fixed_system_command().unwrap();let information=windows_information(&command,true).unwrap();assert!(information[10]>=1);if information[10]>1{assert!(ordinary_file(&command,"substitutable").is_err())}let scratch=std::env::temp_dir().join(format!("hum-fixed-command-{}",std::process::id()));assert!(!scratch.exists());fs::create_dir(&scratch).unwrap();let copy=scratch.join("cmd.exe");fs::copy(&command,&copy).unwrap();let hard=scratch.join("hard-cmd.exe");fs::hard_link(&copy,&hard).unwrap();assert_ne!(fixed_system_command().unwrap(),copy);assert_ne!(fixed_system_command().unwrap(),hard);fs::remove_file(hard).unwrap();fs::remove_file(copy).unwrap();fs::remove_dir(scratch).unwrap();}}
#[cfg(all(test, windows))]#[rustfmt::skip]mod stability_tests{use super::ExecutableBinding;use std::fs;#[test]fn timestamps_are_not_identity_but_stable_fields_and_digest_are(){let honest=[0_u32;13];let mut timestamp=honest;timestamp[1]=1;assert_eq!(&honest[7..],&timestamp[7..]);let mut identity=honest;identity[11]=1;assert_ne!(&honest[7..],&identity[7..]);let digest=[0_u8;32];let mut changed_digest=digest;changed_digest[0]=1;assert_ne!(digest,changed_digest);let source=include_str!("shell.rs").split_once("#[cfg(all(test, windows))]").unwrap().0;assert!(source.contains("first[7..] != second[7..] || first_digest != second_digest"));let owned=std::env::temp_dir().join(format!("hum-binding-parent-{}",std::process::id()));assert!(!owned.exists());fs::create_dir(&owned).unwrap();let parent=owned.join("direct");fs::create_dir(&parent).unwrap();let executable=parent.join("pwsh.exe");fs::copy(std::env::current_exe().unwrap(),&executable).unwrap();let binding=ExecutableBinding::authenticate(executable.clone(),"pwsh_executable").unwrap();let target=owned.join("target");fs::rename(&parent,&target).unwrap();let junction=std::process::Command::new(std::env::var_os("ComSpec").unwrap()).args(["/d","/c","mklink","/J"]).arg(&parent).arg(&target).output().unwrap();assert!(junction.status.success(),"{}",String::from_utf8_lossy(&junction.stderr));assert_eq!(binding.reauthenticate().unwrap_err(),"pwsh_executable: path component is not an ordinary directory");fs::remove_dir(&parent).unwrap();fs::rename(&target,&parent).unwrap();binding.reauthenticate().unwrap();drop(binding);fs::remove_file(executable).unwrap();fs::remove_dir(parent).unwrap();fs::remove_dir(owned).unwrap();let ordered=|text:&str|match(text.find("ordinary_file(&path, predicate)?"),text.find("let initial_identity"),text.find("path.canonicalize()"),text.find("let binding = Self { path, identity")){(Some(a),Some(b),Some(c),Some(d))=>a<b&&b<c&&c<d&&text.contains("ordinary_file(&self.path, self.predicate)?"),_=>false};assert!(ordered(source));for removed in ["ordinary_file(&path, predicate)?","let initial_identity = stable_file_identity(&path)?;","let path = path.canonicalize().map_err(|e| format!(\"{predicate}: {e}\"))?;","ordinary_file(&self.path, self.predicate)?;"]{assert!(!ordered(&source.replacen(removed,"",1)));}}}
#[cfg(test)]#[rustfmt::skip]pub fn resolve_pwsh7(environment:&ShellEnvironment,current:&Path)->Result<PathBuf,String>{let mut matches=std::collections::BTreeSet::new();for entry in std::env::split_paths(environment.get("PATH")?){if entry.as_os_str().is_empty()||!entry.is_absolute()||entry==current{return Err("pwsh_path: empty, relative, or current-directory PATH entry".into())}let candidate=entry.join(if cfg!(windows){"pwsh.exe"}else{"pwsh"});if candidate.exists(){ordinary_file(&candidate,"pwsh_executable")?;matches.insert(candidate.canonicalize().map_err(|e|e.to_string())?);}}if matches.len()!=1{return Err(format!("pwsh_resolution: expected one executable, found {}",matches.len()))}Ok(matches.into_iter().next().unwrap())}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PwshRequest {
    pub executable: ExecutableBinding,
    pub repository: PathBuf,
    pub script: PathBuf,
    pub arguments: Vec<OsString>,
    pub environment: ShellEnvironment,
}

impl PwshRequest {
    pub fn authenticate(&self) -> Result<(), String> {
        self.executable.reauthenticate()?;
        if !self.repository.is_absolute() || !self.repository.is_dir() {
            return Err("repository_identity: repository is not an absolute directory".into());
        }
        self.environment.authenticate(&self.repository)?;
        ordinary_file(&self.script, "script_identity")?;
        if !self.script.starts_with(&self.repository) {
            return Err("script_identity: script escapes repository".into());
        }
        for argument in &self.arguments {
            let text = argument
                .to_str()
                .ok_or_else(|| "argument_identity: value is not UTF-8".to_string())?;
            if text.is_empty() || text.bytes().any(|b| matches!(b, 0 | b'\r' | b'\n')) {
                return Err("argument_identity: empty or contains CR, LF, or NUL".into());
            }
        }
        Ok(())
    }
    pub fn launch(&self) -> Result<Output, String> {
        self.authenticate()?;
        let identity = crate::summary::authenticate_pwsh7(&self.executable, &self.environment)?;
        let script = self
            .script
            .strip_prefix(&self.repository)
            .map_err(|_| "script_identity: script escapes repository".to_string())?;
        let mut command = self.executable.command()?;
        command.env_clear().envs(&self.environment.0);
        let output = command
            .current_dir(&self.repository)
            .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-File"])
            .arg(script)
            .args(&self.arguments)
            .output()
            .map_err(|e| format!("pwsh_launch: {e}"))?;
        if crate::summary::authenticate_pwsh7(&self.executable, &self.environment)? != identity {
            return Err("pwsh_executable: identity changed across launch".into());
        }
        Ok(output)
    }
}

#[cfg(test)]#[rustfmt::skip]mod tests{use super::{ordinary_file,ordinary_windows_attributes,resolve_pwsh7,same_ordinary_file,ExecutableBinding,PwshRequest,ShellEnvironment};use std::{ffi::{OsStr,OsString},fs,ops::Deref,path::{Path,PathBuf},sync::atomic::{AtomicU64,Ordering}};struct Scratch(PathBuf);impl Scratch{fn new(tag:&str)->Self{static N:AtomicU64=AtomicU64::new(0);let path=std::env::temp_dir().join(format!("hum-pwsh-adversary-{tag}-{}-{}",std::process::id(),N.fetch_add(1,Ordering::Relaxed)));assert!(!path.exists());fs::create_dir(&path).unwrap();Self(path)}}impl Deref for Scratch{type Target=Path;fn deref(&self)->&Path{&self.0}}impl Drop for Scratch{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}fn exercise_cleanup(mode:&str)->PathBuf{let owned=Scratch::new(mode);let path=owned.0.clone();match mode{"panic"=>assert!(std::panic::catch_unwind(move||{let _guard=owned;panic!("owned adversary")}).is_err()),"error"=>{let result:Result<(),()>=Err(());drop(owned);assert!(result.is_err())},_=>drop(owned)}path}const WINDOWS_TOOLCHAIN_KEYS:&[&str]=&["INCLUDE","LIB","LIBPATH","VCINSTALLDIR","VCToolsInstallDir","VSCMD_ARG_HOST_ARCH","VSCMD_ARG_TGT_ARCH","WindowsSdkDir","WindowsSDKVersion"];#[cfg(windows)] use std::os::windows::ffi::OsStringExt;fn root()->PathBuf{PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()}#[cfg(windows)]#[test]fn malformed_environment_encoding_is_owned(){let root=root();let honest=ShellEnvironment::from_process(&root).unwrap();let malformed=OsString::from_wide(&[0xd800]);let mut changed=honest.clone();changed.0.insert(malformed.clone(),"x".into());assert!(changed.authenticate(&root).unwrap_err().contains("environment_key"));let mut changed=honest.clone();changed.0.insert("PATH".into(),malformed);assert!(changed.authenticate(&root).unwrap_err().contains("environment_value"));assert_eq!(honest,ShellEnvironment::from_process(&root).unwrap());}#[test]fn pwsh7_adapter_is_thin_declarative_and_environment_bound(){let source=include_str!("shell.rs");let forbidden=concat!("let ","windows =");assert!(source.contains("#[cfg(windows)]\n    fn authenticate_windows_toolchain"));assert!(!source.contains(forbidden));let corrupt=source.replacen("impl ShellEnvironment {",&format!("impl ShellEnvironment {{ {forbidden} false;"),1);assert_ne!(corrupt,source);assert!(corrupt.contains(forbidden));assert!(ordinary_windows_attributes(0,1));assert!(!ordinary_windows_attributes(0x400,1));assert!(!ordinary_windows_attributes(0x480,1));assert!(!ordinary_windows_attributes(0,2));let root=root();let environment=ShellEnvironment::from_process(&root).unwrap();let executable=ExecutableBinding::authenticate(resolve_pwsh7(&environment,&root).unwrap(),"pwsh_executable").unwrap();let honest=PwshRequest{executable,repository:root.clone(),script:root.join("tools/check_alpha_claims.ps1"),arguments:vec![],environment};honest.authenticate().unwrap();let mut changed=honest.clone();changed.environment.0.remove(OsStr::new("PATH"));assert_eq!(changed.authenticate().unwrap_err(),"environment_required: missing PATH");let mut changed=honest.clone();changed.environment.0.insert("GIT_CONFIG_VALUE_0".into(),"*".into());assert!(changed.authenticate().unwrap_err().contains("repository_trust"));let mut changed=honest.clone();changed.arguments.push(OsString::from("bad\0argument"));assert!(changed.authenticate().unwrap_err().contains("argument_identity"));let mut changed=honest.clone();changed.executable.path=PathBuf::from("pwsh");assert!(changed.authenticate().unwrap_err().contains("pwsh_executable"));if cfg!(windows){let mut changed=honest.clone();changed.environment.0.remove(OsStr::new("OS"));assert!(changed.authenticate().unwrap_err().contains("missing OS"));let mut changed=honest.clone();changed.environment.0.insert("OS".into(),"Linux".into());assert!(changed.authenticate().unwrap_err().contains("OS must be Windows_NT"));for remove_os in [false,true]{let mut changed=honest.clone();if remove_os{changed.environment.0.remove(OsStr::new("OS"));}for key in WINDOWS_TOOLCHAIN_KEYS{changed.environment.0.remove(OsStr::new(key));}assert!(changed.authenticate().unwrap_err().contains(if remove_os{"missing OS"}else{"missing INCLUDE"}));}for key in WINDOWS_TOOLCHAIN_KEYS{let mut changed=honest.clone();changed.environment.0.remove(OsStr::new(key));assert!(changed.authenticate().unwrap_err().contains(key));}let mut changed=honest.clone();changed.environment.0.insert("Path".into(),honest.environment.get("PATH").unwrap().into());assert!(changed.authenticate().unwrap_err().contains("duplicate case-variant"));let mut changed=honest.clone();changed.environment.0.insert("BOGUS".into(),"x".into());changed.environment.0.insert("bogus".into(),"y".into());assert!(changed.authenticate().unwrap_err().contains("duplicate case-variant"));let mut changed=honest.clone();changed.environment.0.remove(OsStr::new("PATH"));changed.environment.0.insert("Path".into(),"x".into());assert!(changed.authenticate().unwrap_err().contains("unsupported Path"));let mut changed=honest.clone();changed.environment.0.insert("UNRELATED_AMBIENT".into(),"x".into());assert!(changed.authenticate().unwrap_err().contains("unsupported"));let mut changed=honest.clone();changed.environment.0.insert("VCToolsInstallDir".into(),"relative".into());assert!(changed.authenticate().unwrap_err().contains("invalid VCToolsInstallDir"));let mut changed=honest.clone();changed.environment.0.insert("VSCMD_ARG_TGT_ARCH".into(),"x86".into());assert!(changed.authenticate().unwrap_err().contains("must be x64"));}else{let mut changed=honest.clone();changed.environment.0.insert("OS".into(),"Windows_NT".into());assert!(changed.authenticate().unwrap_err().contains("non-Windows"));let mut changed=honest.clone();changed.environment.0.insert("Path".into(),"x".into());assert!(changed.authenticate().unwrap_err().contains("unsupported Path"));let mut changed=honest.clone();changed.environment.0.insert("VCINSTALLDIR".into(),root.clone().into());assert!(changed.authenticate().unwrap_err().contains("non-Windows"));}for disposition in ["success","error","panic"]{assert!(!exercise_cleanup(disposition).exists());}let scratch=Scratch::new("identity");let left=scratch.join("Identity.exe");let other=scratch.join("other.exe");fs::copy(std::env::current_exe().unwrap(),&left).unwrap();fs::copy(std::env::current_exe().unwrap(),&other).unwrap();let hard=scratch.join("hard.exe");fs::hard_link(&left,&hard).unwrap();assert!(ordinary_file(&left,"hard_link").unwrap_err().contains("ordinary file"));fs::remove_file(&hard).unwrap();ordinary_file(&left,"hard_link").unwrap();assert!(same_ordinary_file(&left,&left).unwrap());if cfg!(windows){assert!(same_ordinary_file(&left,scratch.join("IDENTITY.EXE").as_path()).unwrap())}else{assert!(same_ordinary_file(&left,scratch.join("IDENTITY.EXE").as_path()).is_err())}assert!(!same_ordinary_file(&left,&other).unwrap());assert!(same_ordinary_file(&left,&scratch).is_err());assert!(same_ordinary_file(&left,&scratch.join("missing.exe")).is_err());let original=honest.environment.get("PATH").unwrap().to_owned();if cfg!(windows){fs::copy(std::env::current_exe().unwrap(),scratch.join("link.exe")).unwrap();let expected=PathBuf::from(honest.environment.get("VCToolsInstallDir").unwrap()).join("bin/Hostx64/x64");for entries in [std::env::split_paths(&original).chain(std::iter::once(expected.clone())).collect::<Vec<_>>(),std::env::split_paths(&original).chain(std::iter::once(scratch.0.clone())).collect::<Vec<_>>()]{let mut changed=honest.clone();changed.environment.0.insert("PATH".into(),std::env::join_paths(entries).unwrap());changed.authenticate().unwrap();}let mut changed=honest.clone();changed.environment.0.insert("PATH".into(),std::env::join_paths(std::iter::once(scratch.0.clone()).chain(std::env::split_paths(&original))).unwrap());assert!(changed.authenticate().unwrap_err().contains("substituted effective"));let mut changed=honest.clone();changed.environment.0.insert("PATH".into(),scratch.as_os_str().to_owned());assert!(changed.authenticate().unwrap_err().contains("substituted effective"));}for path in [OsString::from("."),scratch.as_os_str().to_owned()]{let mut changed=honest.environment.clone();changed.0.insert("PATH".into(),path);assert!(resolve_pwsh7(&changed,&scratch).is_err());}let candidate=scratch.join(if cfg!(windows){"pwsh.exe"}else{"pwsh"});fs::create_dir(&candidate).unwrap();let mut changed=honest.environment.clone();changed.0.insert("PATH".into(),scratch.as_os_str().to_owned());let error=resolve_pwsh7(&changed,&root).unwrap_err();assert!(error.contains("ordinary file"),"{error}");fs::remove_dir(&candidate).unwrap();fs::copy(std::env::current_exe().unwrap(),&candidate).unwrap();let fake=resolve_pwsh7(&changed,&root).unwrap();let mut request=honest.clone();request.executable=ExecutableBinding::authenticate(fake,"pwsh_executable").unwrap();request.environment=honest.environment.clone();request.authenticate().unwrap();assert_eq!(request.launch().unwrap_err(),"pwsh_version: runtime is not authenticated PowerShell 7");let entries=std::iter::once(scratch.0.clone()).chain(std::env::split_paths(&original));changed.0.insert("PATH".into(),std::env::join_paths(entries).unwrap());assert!(resolve_pwsh7(&changed,&root).unwrap_err().contains("found 2"));}}
