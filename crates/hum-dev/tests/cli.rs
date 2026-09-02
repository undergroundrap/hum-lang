mod cli {
    use std::{
        fs,
        ops::Deref,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Scratch(PathBuf);
    impl Deref for Scratch {
        type Target = Path;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl AsRef<Path> for Scratch {
        fn as_ref(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn binary() -> &'static str {
        env!("CARGO_BIN_EXE_hum-dev")
    }
    fn pwsh() -> PathBuf {
        let output = Command::new("pwsh")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-Command",
                "[Environment]::ProcessPath",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
            .canonicalize()
            .unwrap()
    }
    fn scratch(name: &str) -> Scratch {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Scratch(
            std::env::temp_dir().join(format!("hum-dev-cli-{name}-{}-{nonce}", std::process::id())),
        )
    }
    #[cfg(windows)]
    fn file_identity(path: &Path) -> (u64, u64, u64) {
        use std::{fs::File, os::windows::io::AsRawHandle};
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetFileInformationByHandle(
                handle: *mut std::ffi::c_void,
                information: *mut u32,
            ) -> i32;
        }
        let file = File::open(path).unwrap();
        let mut information = [0_u32; 13];
        assert_ne!(
            unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) },
            0
        );
        (
            information[7] as u64,
            ((information[11] as u64) << 32) | information[12] as u64,
            information[10] as u64,
        )
    }
    #[cfg(unix)]
    fn file_identity(path: &Path) -> (u64, u64, u64) {
        use std::os::unix::fs::MetadataExt;
        let metadata = fs::metadata(path).unwrap();
        (metadata.dev(), metadata.ino(), metadata.nlink())
    }
    fn physical_binary_copy(name: &str) -> (Scratch, PathBuf) {
        let root = scratch(name);
        assert!(!root.exists());
        fs::create_dir(&*root).unwrap();
        let source = Path::new(binary()).canonicalize().unwrap();
        let copied = root.join(if cfg!(windows) { "pwsh.exe" } else { "pwsh" });
        fs::copy(&source, &copied).unwrap();
        let copied_metadata = fs::symlink_metadata(&copied).unwrap();
        assert!(copied_metadata.file_type().is_file());
        assert!(!copied_metadata.file_type().is_symlink());
        #[cfg(windows)]
        assert_eq!(
            std::os::windows::fs::MetadataExt::file_attributes(&copied_metadata) & 0x400,
            0
        );
        let (source_volume, source_file, _) = file_identity(&source);
        let (copied_volume, copied_file, copied_links) = file_identity(&copied);
        assert_eq!(copied_links, 1);
        assert_ne!((source_volume, source_file), (copied_volume, copied_file));
        let source_bytes = fs::read(source).unwrap();
        let copied_bytes = fs::read(&copied).unwrap();
        assert_eq!(copied_bytes, source_bytes);
        assert_eq!(
            hum_sha256::digest_hex(&copied_bytes),
            hum_sha256::digest_hex(&source_bytes)
        );
        (root, copied)
    }
    fn git(repository: &Path, arguments: &[&str]) {
        let executable = std::env::var_os("HUM_DEV_GIT").unwrap_or_else(|| "git".into());
        let output = Command::new(executable)
            .current_dir(repository)
            .arg("-c")
            .arg(format!(
                "safe.directory={}",
                repository.to_string_lossy().replace('\\', "/")
            ))
            .args(arguments)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fn init(repository: &Path, second_commit: bool) {
        fs::create_dir(repository).unwrap();
        git(repository, &["init", "-q"]);
        fs::write(repository.join("tracked.txt"), b"root\n").unwrap();
        git(repository, &["add", "--", "tracked.txt"]);
        git(
            repository,
            &[
                "-c",
                "user.name=Hum Test",
                "-c",
                "user.email=hum-test@example.invalid",
                "-c",
                "core.hooksPath=",
                "commit",
                "-qm",
                "root",
            ],
        );
        if second_commit {
            fs::write(repository.join("tracked.txt"), b"base\n").unwrap();
            git(repository, &["add", "--", "tracked.txt"]);
            git(
                repository,
                &[
                    "-c",
                    "user.name=Hum Test",
                    "-c",
                    "user.email=hum-test@example.invalid",
                    "-c",
                    "core.hooksPath=",
                    "commit",
                    "-qm",
                    "base",
                ],
            );
        }
    }
    fn identity(repository: &Path) -> String {
        let mut command = Command::new(binary());
        command
            .args(["candidate", "identity", "--repository"])
            .arg(repository);
        if let Some(git) = std::env::var_os("HUM_DEV_GIT") {
            command.env("HUM_DEV_GIT", git);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        String::from_utf8(output.stdout).unwrap()
    }

    #[test]
    fn legacy_probe_environment_cannot_replace_the_canonical_status_mapping() {
        let actual = Command::new(binary())
            .env("HUM_DEV_LEGACY_EQUIVALENCE_PROBE", "1")
            .args(["evidence", "status"])
            .output()
            .unwrap();
        assert_eq!(actual.status.code(), Some(2));
        assert!(actual.stdout.is_empty());
        let stderr = String::from_utf8(actual.stderr).unwrap();
        assert!(stderr.contains("missing authenticated status input"));
        assert!(!stderr.contains("check_workorder_status_boundary.ps1"));
        assert!(!stderr.contains("Alpha claims check passed."));
    }

    #[test]
    fn preflight_repairs_stop_before_state_changing_launch() {
        let missing = scratch("missing-message");
        let output = Command::new(binary())
            .args(["commit-message", "check", "--file"])
            .arg(missing.join("message"))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("message_read")
        );
        assert!(!missing.exists(), "validation failure created state");
        let accepted = Command::new(binary())
            .args(["commit-message", "check", "--subject", "docs(scope): exact"])
            .output()
            .unwrap();
        assert_eq!(accepted.status.code(), Some(0));
        assert_eq!(accepted.stdout, b"accepted|subject_sha256=d4e2af7fb6c98315b7442bee716ac277f27cfaa4d8152d822aaa7285f39b5df6\n");
        assert!(accepted.stderr.is_empty());
        let rejected = Command::new(binary())
            .args(["commit-message", "check", "--subject", "docs: unscoped"])
            .output()
            .unwrap();
        assert_eq!(rejected.status.code(), Some(2));
        assert!(rejected.stdout.is_empty());
        assert_eq!(
            rejected.stderr,
            b"hum-dev: subject_scoped: scope is required\n"
        );
        let help = Command::new(binary())
            .args(["commit-message", "check"])
            .output()
            .unwrap();
        assert_eq!(help.status.code(), Some(2));
        assert!(help.stdout.is_empty());
        assert_eq!(help.stderr, b"hum-dev: usage: hum-dev evidence <focused|full|exhaustive> --pwsh ABSOLUTE_PATH | evidence status | evidence summarize --output PATH --pwsh ABSOLUTE_PATH | commit-message check <--subject TEXT|--file PATH> | candidate identity [--repository PATH] | cleanup verify | workorder status-facts --input PATH --base-sha256 HASH --status-body-file PATH --gate-body-file PATH --output PATH\n");
    }

    #[test]
    fn public_focused_evidence_resolves_authenticated_pwsh7() {
        let (executable_root, executable) = physical_binary_copy("public-focused");
        let executable_root_path = executable_root.0.clone();
        let output = Command::new(executable)
            .args(["evidence", "focused", "--pwsh"])
            .arg(pwsh())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        for selector in [
            "shell::tests::pwsh7_adapter_is_thin_declarative_and_environment_bound",
            "commit_message::tests::legacy_hook_corpus_matches_portable_rule",
            "cli::preflight_repairs_stop_before_state_changing_launch",
        ] {
            assert!(
                stdout.contains(selector),
                "public focused path missed {selector}"
            );
        }
        drop(executable_root);
        assert!(!executable_root_path.exists());
    }

    #[test]
    #[rustfmt::skip]
    fn explicit_pwsh_binding_is_required_and_fail_closed() {
        let run = |args: &[&str]| Command::new(binary()).args(args).output().unwrap();
        let missing = run(&["evidence", "focused"]);
        assert_eq!(missing.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&missing.stderr).contains("explicit --pwsh"));
        let duplicate = run(&[
            "evidence",
            "focused",
            "--pwsh",
            binary(),
            "--pwsh",
            binary(),
        ]);
        assert_eq!(duplicate.status.code(), Some(2));
        let relative = run(&["evidence", "focused", "--pwsh", "pwsh"]);
        assert_eq!(relative.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&relative.stderr).contains("explicit absolute --pwsh"));
        let ignored = Command::new(binary())
            .args(["evidence", "focused"])
            .env("HUM_DEV_PWSH", pwsh())
            .output()
            .unwrap();
        assert_eq!(ignored.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&ignored.stderr).contains("explicit --pwsh"));
        let directory = Command::new(binary())
            .args(["evidence", "focused", "--pwsh"])
            .arg(std::env::temp_dir())
            .output()
            .unwrap();
        assert_eq!(directory.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&directory.stderr).contains("ordinary file"));
        let (copied_root, copied) = physical_binary_copy("wrong-pwsh");
        let copied_root_path = copied_root.0.clone();
        let wrong = Command::new(binary())
            .args(["evidence", "focused", "--pwsh"])
            .arg(&copied)
            .output()
            .unwrap();
        assert_eq!(wrong.status.code(), Some(2));
        assert!(wrong.stdout.is_empty());
        let wrong_stderr = String::from_utf8_lossy(&wrong.stderr);
        assert_eq!(
            wrong_stderr, "hum-dev: pwsh_version: runtime is not authenticated PowerShell 7\n",
            "wrong executable stderr: {wrong_stderr}"
        );
        #[cfg(windows)] {
            let native = PathBuf::from(std::env::var_os("ProgramFiles(x86)").unwrap()).canonicalize().unwrap(); let case_variant = PathBuf::from(native.to_string_lossy().to_ascii_uppercase()); let source = native.join("Microsoft Visual Studio/Installer/vswhere.exe").canonicalize().unwrap();
            let poison = scratch("program-files-x86"); let poison_path = poison.0.clone(); let copied_vswhere = poison.join("Microsoft Visual Studio/Installer/vswhere.exe"); fs::create_dir_all(copied_vswhere.parent().unwrap()).unwrap(); fs::copy(&source, &copied_vswhere).unwrap();
            let root_metadata = fs::symlink_metadata(&*poison).unwrap(); let copy_metadata = fs::symlink_metadata(&copied_vswhere).unwrap(); use std::os::windows::fs::MetadataExt; assert!(root_metadata.is_dir() && root_metadata.file_attributes() & 0x400 == 0); assert!(copy_metadata.is_file() && copy_metadata.file_attributes() & 0x400 == 0); assert_eq!(copy_metadata.len(), fs::metadata(&source).unwrap().len());
            let source_identity = file_identity(&source); let copied_identity = file_identity(&copied_vswhere); assert_eq!(copied_identity.2, 1); assert_ne!((source_identity.0, source_identity.1), (copied_identity.0, copied_identity.1)); let source_bytes = fs::read(&source).unwrap(); let copied_bytes = fs::read(&copied_vswhere).unwrap(); assert_eq!(source_bytes, copied_bytes); assert_eq!(hum_sha256::digest_hex(&source_bytes), hum_sha256::digest_hex(&copied_bytes));
            let prebootstrap = |path: &Path| Command::new(binary()).args(["evidence", "focused", "--pwsh"]).arg(path).env("ProgramFiles(x86)", poison.as_os_str()).env_remove("PATH").env_remove("Path").env_remove("PSModulePath").output().unwrap();
            let nonexistent = prebootstrap(&poison.join("missing-pwsh.exe")); assert_eq!(nonexistent.status.code(), Some(2)); assert!(nonexistent.stdout.is_empty()); assert!(String::from_utf8_lossy(&nonexistent.stderr).starts_with("hum-dev: pwsh_executable:"));
            let hard = copied_root.join("hard-pwsh.exe"); fs::hard_link(&copied, &hard).unwrap(); let hard_output = prebootstrap(&hard); assert_eq!(hard_output.stderr, b"hum-dev: pwsh_executable: path is not an ordinary file\n"); fs::remove_file(&hard).unwrap();
            let reparse = copied_root.join("reparse-pwsh.exe"); let junction = Command::new(std::env::var_os("ComSpec").unwrap()).args(["/d", "/c", "mklink", "/J"]).arg(&reparse).arg(poison.as_ref()).output().unwrap(); assert!(junction.status.success(), "{}", String::from_utf8_lossy(&junction.stderr)); assert_ne!(fs::symlink_metadata(&reparse).unwrap().file_attributes() & 0x400, 0); let reparse_output = prebootstrap(&reparse); assert_eq!(reparse_output.stderr, b"hum-dev: pwsh_executable: path is not an ordinary file\n"); fs::remove_dir(&reparse).unwrap(); let parent_owner = scratch("pwsh-parent-junction"); let parent_owner_path = parent_owner.0.clone(); fs::create_dir(&*parent_owner).unwrap(); let parent_alias = parent_owner.join("alias"); let pwsh = pwsh(); let parent_junction = Command::new(std::env::var_os("ComSpec").unwrap()).args(["/d", "/c", "mklink", "/J"]).arg(&parent_alias).arg(pwsh.parent().unwrap()).output().unwrap(); assert!(parent_junction.status.success(), "{}", String::from_utf8_lossy(&parent_junction.stderr)); assert_ne!(fs::symlink_metadata(&parent_alias).unwrap().file_attributes() & 0x400, 0); let parent_output = prebootstrap(&parent_alias.join(pwsh.file_name().unwrap())); assert_eq!(parent_output.status.code(), Some(2)); assert!(parent_output.stdout.is_empty()); assert_eq!(parent_output.stderr, b"hum-dev: pwsh_executable: path component is not an ordinary directory\n"); let case_output = prebootstrap(&PathBuf::from(pwsh.to_string_lossy().to_ascii_uppercase())); assert_eq!(case_output.stderr, b"hum-dev: environment_required: missing PATH\n"); fs::remove_dir(&parent_alias).unwrap(); drop(parent_owner); assert!(!parent_owner_path.exists());
            let combined = Command::new(binary()).args(["evidence", "focused", "--pwsh", "pwsh"]).env("ProgramFiles(x86)", poison.as_os_str()).env_remove("PATH").env_remove("Path").env_remove("PSModulePath").output().unwrap(); assert_eq!(combined.status.code(), Some(2)); assert!(combined.stdout.is_empty()); assert_eq!(combined.stderr, b"hum-dev: pwsh_executable: explicit absolute --pwsh path is required\n");
            let run = |binding: Option<&std::ffi::OsStr>| { let mut command = Command::new(binary()); command.args(["evidence", "focused", "--pwsh"]).arg(&copied); if let Some(value) = binding { command.env("ProgramFiles(x86)", value); } else { command.env_remove("ProgramFiles(x86)"); } command.output().unwrap() };
            for (name, binding, expected) in [("missing", None, "pwsh_version"), ("matching", Some(native.as_os_str()), "pwsh_version"), ("case-equivalent", Some(case_variant.as_os_str()), "pwsh_version"), ("relative", Some(std::ffi::OsStr::new("relative")), "program_files_x86: ambient binding differs from native known folder")] { let output = run(binding); assert_eq!(output.status.code(), Some(2), "{name}"); assert!(output.stdout.is_empty(), "{name}"); assert!(String::from_utf8_lossy(&output.stderr).contains(expected), "{name}: {}", String::from_utf8_lossy(&output.stderr)); }
            let poisoned = run(Some(poison.as_os_str())); assert_eq!(poisoned.status.code(), Some(2)); assert!(poisoned.stdout.is_empty()); assert_eq!(poisoned.stderr, b"hum-dev: program_files_x86: ambient binding differs from native known folder\n");
            let production = include_str!("../src/command.rs").split_once("#[cfg(test)]").unwrap().0; assert_eq!(production.matches("known_folder(0x2a").count(), 1); assert!(!production.contains("PathBuf::from(env.get(\"ProgramFiles(x86)\")")); assert!(production.find("ambient binding differs from native known folder").unwrap() < production.find("clean_stdout(&vswhere").unwrap());
            let launch = include_str!("../src/main.rs").split_once("fn launch_legacy").unwrap().1.split_once("#[rustfmt::skip]").unwrap().0; let ordered = |source: &str| source.matches("ExecutableBinding::authenticate").count() == 1 && source.matches("production_environment").count() == 1 && source.find("ExecutableBinding::authenticate").unwrap() < source.find("production_environment").unwrap() && source.contains("PwshRequest {\n        executable,"); assert!(ordered(launch)); assert!(!ordered(&launch.replacen("ExecutableBinding::authenticate", "removed binding", 1))); let reordered = launch.replacen("production_environment", "WO25_ORDER_SWAP", 1).replacen("ExecutableBinding::authenticate", "production_environment", 1).replacen("WO25_ORDER_SWAP", "ExecutableBinding::authenticate", 1); assert!(!ordered(&reordered)); let rebound = launch.replacen("shell::PwshRequest {", "let executable = shell::ExecutableBinding::authenticate(resolved.to_owned(), \"pwsh_executable\")?;\n    shell::PwshRequest {", 1); assert!(!ordered(&rebound));
            drop(poison); assert!(!poison_path.exists()); let panic_path = { let owned = scratch("program-files-x86-panic"); fs::create_dir(&*owned).unwrap(); let path = owned.0.clone(); assert!(std::panic::catch_unwind(move || { let _owned = owned; panic!("owned native-folder adversary") }).is_err()); path }; assert!(!panic_path.exists()); let junction_cleanup = |mode: &str| { let owned = scratch(&format!("parent-junction-{mode}")); fs::create_dir(&*owned).unwrap(); let target = owned.join("target"); fs::create_dir(&target).unwrap(); let alias = owned.join("alias"); let created = Command::new(std::env::var_os("ComSpec").unwrap()).args(["/d", "/c", "mklink", "/J"]).arg(&alias).arg(&target).output().unwrap(); assert!(created.status.success(), "{}", String::from_utf8_lossy(&created.stderr)); let path = owned.0.clone(); if mode == "error" { let failed: Result<(), ()> = { let _guard = owned; Err(()) }; assert!(failed.is_err()); } else { assert!(std::panic::catch_unwind(move || { let _guard = owned; panic!("owned junction adversary") }).is_err()); } assert!(!path.exists()); }; junction_cleanup("error"); junction_cleanup("panic");
        }
        drop(copied_root);
        assert!(!copied_root_path.exists());
        let output = scratch("summarize-output").join("summary.json");
        let output = output.to_str().unwrap();
        for args in [
            vec!["evidence", "summarize", "--output", output],
            vec!["evidence", "summarize", "--pwsh", "pwsh"],
            vec![
                "evidence",
                "summarize",
                "--pwsh",
                "pwsh",
                "--output",
                output,
            ],
            vec![
                "evidence",
                "summarize",
                "--output",
                output,
                "--output",
                output,
            ],
            vec![
                "evidence",
                "summarize",
                "--output",
                output,
                "--pwsh",
                "pwsh",
                "extra",
            ],
        ] {
            let rejected = run(&args);
            assert_eq!(rejected.status.code(), Some(2));
            assert!(rejected.stdout.is_empty());
            assert!(
                !Path::new(output).exists(),
                "invalid summarize form created output"
            );
        }
        let command_source = include_str!("../src/command.rs");
        let main_source = include_str!("../src/main.rs");
        assert!(!command_source.contains(concat!("EvidenceSummarize", "(PathBuf)")));
        assert!(!main_source.contains(concat!("Command::EvidenceSummarize", "(")));
        let bound = main_source.find("Command::EvidenceSummarizeBound").unwrap();
        let authenticate = main_source[bound..].find("authenticate_pwsh7").unwrap(); let bind = main_source[bound..].find("ExecutableBinding::authenticate").unwrap(); let environment = main_source[bound..].find("production_environment").unwrap(); assert!(bind < environment && environment < authenticate); assert!(main_source[bound..].contains("authenticate_pwsh7(&pwsh"));
        let write = main_source[bound..].find("fs::write(output").unwrap();
        assert!(
            authenticate < write,
            "summary output preceded orchestration authentication"
        );
    }

    #[test]
    fn candidate_identity_distinguishes_clean_indexed_untracked_and_dirty_state() {
        let root_repo = scratch("root");
        init(&root_repo, false);
        let root_identity = identity(&root_repo);
        assert!(root_identity.contains("\"parents\":[]"));
        assert!(root_identity.contains("\"head_state\":\"symbolic\""));
        let repository = scratch("identity");
        init(&repository, true);
        let clean = identity(&repository);
        assert!(
            clean.contains("\"index_clean\":true,\"worktree_clean\":true,\"untracked_clean\":true")
        );
        git(&repository, &["branch", "identity-side"]);
        let extra_ref = identity(&repository);
        assert_ne!(extra_ref, clean);
        git(&repository, &["branch", "-D", "identity-side"]);
        assert_eq!(identity(&repository), clean);
        fs::write(repository.join("tracked.txt"), b"index-one\n").unwrap();
        git(&repository, &["add", "--", "tracked.txt"]);
        fs::write(repository.join("tracked.txt"), b"worktree-fixed\n").unwrap();
        let first_index = identity(&repository);
        fs::write(repository.join("tracked.txt"), b"index-two\n").unwrap();
        git(&repository, &["add", "--", "tracked.txt"]);
        fs::write(repository.join("tracked.txt"), b"worktree-fixed\n").unwrap();
        let second_index = identity(&repository);
        assert_ne!(first_index, second_index);
        fs::write(repository.join("tracked.txt"), b"base\n").unwrap();
        git(&repository, &["add", "--", "tracked.txt"]);
        assert_eq!(identity(&repository), clean);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                repository.join("tracked.txt"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            assert!(identity(&repository).contains("\"worktree_mode\":\"100755\""));
            fs::set_permissions(
                repository.join("tracked.txt"),
                fs::Permissions::from_mode(0o644),
            )
            .unwrap();
        }
        fs::write(repository.join("untracked.txt"), b"one\n").unwrap();
        let untracked_one = identity(&repository);
        #[cfg(windows)]
        assert!(untracked_one.contains("\"worktree_mode\":null"));
        fs::write(repository.join("untracked.txt"), b"two\n").unwrap();
        let untracked_two = identity(&repository);
        assert_ne!(untracked_one, untracked_two);
        fs::write(repository.join("untracked.txt"), b" \t\n").unwrap();
        let whitespace = identity(&repository);
        assert!(whitespace.contains("\"raw\":{\"additions\":1"));
        assert!(whitespace.contains("\"whitespace\":{\"additions\":1"));
        fs::remove_file(repository.join("untracked.txt")).unwrap();
        fs::write(repository.join("tracked.txt"), b" \tbase \n").unwrap();
        let tracked_whitespace = identity(&repository);
        assert!(tracked_whitespace.contains("\"raw\":{\"additions\":1,\"deletions\":1"));
        assert!(tracked_whitespace.contains("\"whitespace\":{\"additions\":0,\"deletions\":0"));
        fs::write(repository.join("tracked.txt"), b"base\n").unwrap();
        fs::write(repository.join("intent.txt"), b"intent\n").unwrap();
        git(&repository, &["add", "-N", "--", "intent.txt"]);
        let intent = identity(&repository);
        assert!(intent.contains("\"intent_to_add\":true"));
        git(&repository, &["add", "--", "intent.txt"]);
        let addition = identity(&repository);
        assert_ne!(addition, intent);
        fs::remove_file(repository.join("tracked.txt")).unwrap();
        git(&repository, &["add", "-u", "--", "tracked.txt"]);
        let deletion = identity(&repository);
        assert!(deletion.contains("\"worktree_kind\":\"missing\""));
        assert_ne!(deletion, addition);
        let detached = scratch("detached");
        init(&detached, true);
        git(&detached, &["checkout", "--detach", "-q"]);
        assert!(identity(&detached).contains("\"head_state\":\"detached\",\"head_ref\":null"));
    }
}
