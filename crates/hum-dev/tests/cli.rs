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
    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
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
        let direct = Command::new("pwsh")
            .current_dir(root())
            .args([
                "-NoLogo",
                "-NoProfile",
                "-File",
                "tools/check_workorder_status_boundary.ps1",
            ])
            .output()
            .unwrap();
        let actual = Command::new(binary())
            .env("HUM_DEV_LEGACY_EQUIVALENCE_PROBE", "1")
            .args(["evidence", "status"])
            .output()
            .unwrap();
        assert!(direct.status.success());
        assert_eq!(actual.status.code(), direct.status.code());
        assert_eq!(actual.stdout, direct.stdout);
        assert_eq!(actual.stderr, direct.stderr);
        let alpha = b"Alpha claims check passed.";
        assert!(
            !actual
                .stdout
                .windows(alpha.len())
                .any(|bytes| bytes == alpha)
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
