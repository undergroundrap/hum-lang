use std::{
    fmt, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct CleanupError(String);

impl fmt::Display for CleanupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug)]
pub struct OwnedResource {
    path: PathBuf,
    root: PathBuf,
    prefix: String,
    closed: bool,
}

fn normalized(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
        .trim_end_matches('/')
        .to_string()
}

impl OwnedResource {
    pub fn create(prefix: &str) -> Result<Self, CleanupError> {
        if prefix.is_empty()
            || !prefix
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(CleanupError("owned resource prefix is invalid".into()));
        }
        let root = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| CleanupError(error.to_string()))?
            .as_nanos();
        let path = root.join(format!("hum-dev-{prefix}-{}-{nonce}", std::process::id()));
        fs::create_dir(&path)
            .map_err(|error| CleanupError(format!("owned resource creation failed: {error}")))?;
        Self::authenticate(&root, &path, prefix)?;
        Ok(Self {
            path,
            root,
            prefix: prefix.to_string(),
            closed: false,
        })
    }

    fn authenticate(root: &Path, path: &Path, prefix: &str) -> Result<(), CleanupError> {
        let parent = path
            .parent()
            .ok_or_else(|| CleanupError("owned path has no parent".into()))?;
        if normalized(parent) != normalized(root) {
            return Err(CleanupError("owned path escaped the temporary root".into()));
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| CleanupError("owned path name is not UTF-8".into()))?;
        if !name.starts_with(&format!("hum-dev-{prefix}-")) {
            return Err(CleanupError("owned path prefix mismatch".into()));
        }
        let metadata =
            fs::symlink_metadata(path).map_err(|error| CleanupError(error.to_string()))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(CleanupError(
                "owned path is not an ordinary directory".into(),
            ));
        }
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&self, name: &str, bytes: &[u8]) -> Result<PathBuf, CleanupError> {
        if name.is_empty() || name.contains(['/', '\\']) {
            return Err(CleanupError("owned file name is invalid".into()));
        }
        let path = self.path.join(name);
        fs::write(&path, bytes).map_err(|error| CleanupError(error.to_string()))?;
        Ok(path)
    }

    pub fn close(&mut self) -> Result<(), CleanupError> {
        if self.closed {
            return Ok(());
        }
        Self::authenticate(&self.root, &self.path, &self.prefix)?;
        fs::remove_dir_all(&self.path)
            .map_err(|error| CleanupError(format!("owned cleanup failed: {error}")))?;
        self.closed = true;
        Ok(())
    }
}

impl Drop for OwnedResource {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::OwnedResource;
    use std::{
        fs,
        panic::{AssertUnwindSafe, catch_unwind},
        path::PathBuf,
    };

    struct ResidueReconciler(PathBuf);

    impl Drop for ResidueReconciler {
        fn drop(&mut self) {
            if !self.0.exists() {
                return;
            }
            let metadata = fs::symlink_metadata(&self.0).expect("proof residue metadata");
            assert!(metadata.is_dir() && !metadata.file_type().is_symlink());
            assert_eq!(
                fs::read_dir(&self.0)
                    .expect("proof residue listing")
                    .count(),
                0
            );
            fs::remove_dir(&self.0).expect("proof residue reconciliation");
        }
    }

    #[test]
    fn owned_resources_close_on_every_controlled_terminal_path() {
        let success_path = {
            let mut owned = OwnedResource::create("success").unwrap();
            owned.write("record", b"ok").unwrap();
            let path = owned.path().to_path_buf();
            owned.close().unwrap();
            path
        };
        assert!(!success_path.exists());
        let failure_path = {
            let owned = OwnedResource::create("failure").unwrap();
            let path = owned.path().to_path_buf();
            let _reconciler = ResidueReconciler(path.clone());
            drop(owned);
            if path.exists() {
                eprintln!(
                    "owned_residue_sha256={}",
                    hum_sha256::digest_hex(path.to_string_lossy().as_bytes())
                );
            }
            assert!(
                !path.exists(),
                "authenticated current-run residue survives a controlled disposition"
            );
            path
        };
        assert!(!failure_path.exists());
        let mut panic_path = None;
        let result = catch_unwind(AssertUnwindSafe(|| {
            let owned = OwnedResource::create("panic").unwrap();
            panic_path = Some(owned.path().to_path_buf());
            panic!("controlled");
        }));
        assert!(result.is_err());
        assert!(
            !panic_path.unwrap().exists(),
            "authenticated current-run residue survives a controlled disposition"
        );
        for disposition in ["timeout", "termination"] {
            let mut owned = OwnedResource::create(disposition).unwrap();
            let path = owned.path().to_path_buf();
            owned.close().unwrap();
            assert!(!path.exists());
        }
        let foreign = std::env::temp_dir().join(format!("foreign-resource-{}", std::process::id()));
        fs::create_dir_all(&foreign).unwrap();
        {
            let _owned = OwnedResource::create("foreign-control").unwrap();
        }
        assert!(foreign.exists());
        fs::remove_dir(&foreign).unwrap();
    }
}
