use std::ffi::OsStr;
#[cfg(any(windows, test))]
use std::io::Read;

use crate::native_path::{ValidatedNativePath, validate_native_path};

#[cfg(any(windows, test))]
pub(crate) const FILE_READ_LIMIT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileReadAdapterError {
    #[cfg(any(windows, test))]
    UnsafePath,
    #[cfg(windows)]
    NotFound,
    #[cfg(any(windows, test))]
    NotFile,
    #[cfg(any(windows, test))]
    TooLarge,
    #[cfg(any(windows, test))]
    InvalidUtf8,
    IoFailed,
}

impl FileReadAdapterError {
    pub(crate) fn variant(self) -> &'static str {
        match self {
            #[cfg(any(windows, test))]
            Self::UnsafePath => "unsafe_path",
            #[cfg(windows)]
            Self::NotFound => "not_found",
            #[cfg(any(windows, test))]
            Self::NotFile => "not_file",
            #[cfg(any(windows, test))]
            Self::TooLarge => "too_large",
            #[cfg(any(windows, test))]
            Self::InvalidUtf8 => "invalid_utf8",
            Self::IoFailed => "io_failed",
        }
    }

    pub(crate) fn result_reason(self) -> &'static str {
        match self {
            #[cfg(any(windows, test))]
            Self::UnsafePath => "reparse_or_unsafe_component_rejected_v0",
            #[cfg(windows)]
            Self::NotFound => "candidate_not_found_v0",
            #[cfg(any(windows, test))]
            Self::NotFile => "candidate_is_not_one_regular_file_v0",
            #[cfg(any(windows, test))]
            Self::TooLarge => "one_mibibyte_limit_exceeded_v0",
            #[cfg(any(windows, test))]
            Self::InvalidUtf8 => "strict_utf8_decode_failed_v0",
            Self::IoFailed => "opaque_host_io_failure_v0",
        }
    }
}

pub(crate) trait FileReadAdapter {
    fn read_text(&mut self, path: &OsStr) -> Result<String, FileReadAdapterError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileLocalityError {
    UnsafePath,
    Unavailable,
}

pub(crate) trait FileLocalityAdapter {
    fn revalidate(
        &mut self,
        path: &ValidatedNativePath,
    ) -> Result<ValidatedNativePath, FileLocalityError>;
}

pub(crate) struct HostFileLocalityAdapter;

impl FileLocalityAdapter for HostFileLocalityAdapter {
    fn revalidate(
        &mut self,
        path: &ValidatedNativePath,
    ) -> Result<ValidatedNativePath, FileLocalityError> {
        validate_native_path(path.as_os_str()).map_err(|issue| {
            if issue.is_unsupported_host() {
                FileLocalityError::Unavailable
            } else {
                FileLocalityError::UnsafePath
            }
        })
    }
}

pub(crate) struct HostFileReadAdapter;

#[cfg(not(windows))]
impl FileReadAdapter for HostFileReadAdapter {
    fn read_text(&mut self, _path: &OsStr) -> Result<String, FileReadAdapterError> {
        Err(FileReadAdapterError::IoFailed)
    }
}

#[cfg(windows)]
impl FileReadAdapter for HostFileReadAdapter {
    fn read_text(&mut self, path: &OsStr) -> Result<String, FileReadAdapterError> {
        read_checked_windows_file(path)
    }
}

#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComponentKind {
    Directory,
    File,
    Other,
}

#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ComponentEvidence {
    kind: ComponentKind,
    reparse: bool,
    length: u64,
    final_component: bool,
}

#[cfg(any(windows, test))]
fn validate_component_evidence(evidence: &[ComponentEvidence]) -> Result<(), FileReadAdapterError> {
    let Some((last, parents)) = evidence.split_last() else {
        return Err(FileReadAdapterError::UnsafePath);
    };
    if evidence.iter().any(|entry| entry.reparse)
        || parents
            .iter()
            .any(|entry| entry.kind != ComponentKind::Directory || entry.final_component)
        || !last.final_component
    {
        return Err(FileReadAdapterError::UnsafePath);
    }
    if last.kind != ComponentKind::File {
        return Err(FileReadAdapterError::NotFile);
    }
    if last.length > FILE_READ_LIMIT_BYTES as u64 {
        return Err(FileReadAdapterError::TooLarge);
    }
    Ok(())
}

#[cfg(any(windows, test))]
fn read_validated_content<R, Open>(
    evidence: &[ComponentEvidence],
    open: Open,
) -> Result<String, FileReadAdapterError>
where
    R: Read,
    Open: FnOnce() -> Result<R, FileReadAdapterError>,
{
    validate_component_evidence(evidence)?;
    read_bounded_utf8(open()?)
}

#[cfg(any(windows, test))]
fn read_bounded_utf8<R: Read>(reader: R) -> Result<String, FileReadAdapterError> {
    let mut bytes = Vec::new();
    reader
        .take(FILE_READ_LIMIT_BYTES as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| FileReadAdapterError::IoFailed)?;
    String::from_utf8(bytes).map_err(|_| FileReadAdapterError::InvalidUtf8)
}

#[cfg(windows)]
fn read_checked_windows_file(path: &OsStr) -> Result<String, FileReadAdapterError> {
    use std::fs::{self, File};
    use std::os::windows::fs::MetadataExt;
    use std::path::{Component, Path, PathBuf};

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;

    let path = Path::new(path);
    let mut prefixes = Vec::new();
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match component {
            Component::Prefix(_) | Component::RootDir => {}
            Component::Normal(_) => prefixes.push(current.clone()),
            Component::CurDir | Component::ParentDir => {
                return Err(FileReadAdapterError::UnsafePath);
            }
        }
    }

    let mut evidence = Vec::with_capacity(prefixes.len());
    for (index, prefix) in prefixes.iter().enumerate() {
        let metadata = fs::symlink_metadata(prefix).map_err(map_host_error)?;
        let file_type = metadata.file_type();
        evidence.push(ComponentEvidence {
            kind: if file_type.is_dir() {
                ComponentKind::Directory
            } else if file_type.is_file() {
                ComponentKind::File
            } else {
                ComponentKind::Other
            },
            reparse: file_type.is_symlink()
                || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0,
            length: metadata.len(),
            final_component: index + 1 == prefixes.len(),
        });
    }
    read_validated_content(&evidence, || File::open(path).map_err(map_host_error))
}

#[cfg(windows)]
fn map_host_error(error: std::io::Error) -> FileReadAdapterError {
    match error.kind() {
        std::io::ErrorKind::NotFound => FileReadAdapterError::NotFound,
        std::io::ErrorKind::IsADirectory => FileReadAdapterError::NotFile,
        _ => FileReadAdapterError::IoFailed,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentEvidence, ComponentKind, FILE_READ_LIMIT_BYTES, FileReadAdapter,
        FileReadAdapterError, HostFileReadAdapter, read_bounded_utf8, read_validated_content,
        validate_component_evidence,
    };

    fn directory() -> ComponentEvidence {
        ComponentEvidence {
            kind: ComponentKind::Directory,
            reparse: false,
            length: 0,
            final_component: false,
        }
    }

    fn file(length: u64) -> ComponentEvidence {
        ComponentEvidence {
            kind: ComponentKind::File,
            reparse: false,
            length,
            final_component: true,
        }
    }

    #[test]
    fn component_evidence_rejects_every_reparse_position_and_non_file_final() {
        assert_eq!(
            validate_component_evidence(&[]),
            Err(FileReadAdapterError::UnsafePath)
        );
        assert!(validate_component_evidence(&[directory(), file(5)]).is_ok());

        let mut parent_reparse = directory();
        parent_reparse.reparse = true;
        assert_eq!(
            validate_component_evidence(&[parent_reparse, file(5)]),
            Err(FileReadAdapterError::UnsafePath)
        );

        let mut file_reparse = file(5);
        file_reparse.reparse = true;
        assert_eq!(
            validate_component_evidence(&[directory(), file_reparse]),
            Err(FileReadAdapterError::UnsafePath)
        );

        let mut final_directory = directory();
        final_directory.final_component = true;
        assert_eq!(
            validate_component_evidence(&[final_directory]),
            Err(FileReadAdapterError::NotFile)
        );

        let mut other = file(0);
        other.kind = ComponentKind::Other;
        assert_eq!(
            validate_component_evidence(&[other]),
            Err(FileReadAdapterError::NotFile)
        );
    }

    #[test]
    fn component_evidence_enforces_the_exact_one_mibibyte_bound() {
        assert!(validate_component_evidence(&[file(FILE_READ_LIMIT_BYTES as u64)]).is_ok());
        assert_eq!(
            validate_component_evidence(&[file(FILE_READ_LIMIT_BYTES as u64 + 1)]),
            Err(FileReadAdapterError::TooLarge)
        );
    }

    #[test]
    fn production_read_path_accepts_exact_limit_without_consuming_a_sentinel() {
        use std::cell::Cell;
        use std::io::{Cursor, Read};

        let open_calls = Cell::new(0usize);
        let exact = vec![b'a'; FILE_READ_LIMIT_BYTES];
        let text = read_validated_content(&[file(FILE_READ_LIMIT_BYTES as u64)], || {
            open_calls.set(open_calls.get() + 1);
            Ok(Cursor::new(exact))
        })
        .expect("exactly one MiB is accepted");
        assert_eq!(text.len(), FILE_READ_LIMIT_BYTES);
        assert_eq!(open_calls.get(), 1);

        let oversized_open_calls = Cell::new(0usize);
        let oversized = read_validated_content::<Cursor<Vec<u8>>, _>(
            &[file(FILE_READ_LIMIT_BYTES as u64 + 1)],
            || {
                oversized_open_calls.set(oversized_open_calls.get() + 1);
                Ok(Cursor::new(Vec::new()))
            },
        );
        assert_eq!(oversized, Err(FileReadAdapterError::TooLarge));
        assert_eq!(oversized_open_calls.get(), 0);

        let mut reader = Cursor::new(vec![b'b'; FILE_READ_LIMIT_BYTES + 1]);
        let bounded = read_bounded_utf8(&mut reader).expect("bounded UTF-8 read");
        assert_eq!(bounded.len(), FILE_READ_LIMIT_BYTES);
        assert_eq!(reader.position(), FILE_READ_LIMIT_BYTES as u64);
        let mut sentinel = [0u8; 1];
        reader.read_exact(&mut sentinel).expect("sentinel remains");
        assert_eq!(sentinel, [b'b']);
    }

    #[test]
    fn adapter_source_has_one_read_only_open_and_no_widened_filesystem_surface() {
        let source = include_str!("file_read.rs");
        assert_eq!(source.matches(concat!("File::", "open(")).count(), 1);
        for forbidden in [
            concat!("Open", "Options"),
            concat!("File::", "create("),
            concat!("fs::", "write("),
            concat!("read_", "dir("),
            concat!("canonical", "ize("),
            concat!("remove_", "file("),
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden adapter surface: {forbidden}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn host_adapter_reads_the_checked_in_utf8_fixture_without_writing() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/file_read/session_ad_utf8.txt");
        let mut adapter = HostFileReadAdapter;
        assert_eq!(
            adapter.read_text(path.as_os_str()),
            Ok("Hum reads exact UTF-8: lambda=λ\n".to_string())
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_host_adapter_is_unavailable_without_file_access() {
        let mut adapter = HostFileReadAdapter;
        assert_eq!(
            adapter.read_text(std::ffi::OsStr::new("/not-accessed")),
            Err(FileReadAdapterError::IoFailed)
        );
    }
}
