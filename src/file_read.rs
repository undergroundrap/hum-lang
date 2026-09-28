use std::ffi::OsStr;
#[cfg(any(windows, unix, test))]
use std::io::Read;

use crate::native_path::{ValidatedNativePath, validate_native_path};

#[cfg(any(windows, unix, test))]
pub(crate) const FILE_READ_LIMIT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileReadAdapterError {
    #[cfg(any(windows, unix, test))]
    UnsafePath,
    #[cfg(any(windows, unix))]
    NotFound,
    #[cfg(any(windows, unix, test))]
    NotFile,
    #[cfg(any(windows, unix, test))]
    TooLarge,
    #[cfg(any(windows, unix, test))]
    InvalidUtf8,
    IdentityUnavailable,
    MissingProofEvidence,
    ContradictoryBackingEvidence,
    IoFailed,
}

impl FileReadAdapterError {
    pub(crate) fn variant(self) -> &'static str {
        match self {
            #[cfg(any(windows, unix, test))]
            Self::UnsafePath => "unsafe_path",
            #[cfg(any(windows, unix))]
            Self::NotFound => "not_found",
            #[cfg(any(windows, unix, test))]
            Self::NotFile => "not_file",
            #[cfg(any(windows, unix, test))]
            Self::TooLarge => "too_large",
            #[cfg(any(windows, unix, test))]
            Self::InvalidUtf8 => "invalid_utf8",
            Self::IdentityUnavailable => "identity_unavailable",
            Self::MissingProofEvidence => "missing_proof_evidence",
            Self::ContradictoryBackingEvidence => "contradictory_backing_evidence",
            Self::IoFailed => "io_failed",
        }
    }

    pub(crate) fn result_reason(self) -> &'static str {
        match self {
            #[cfg(any(windows, unix, test))]
            Self::UnsafePath => "reparse_or_unsafe_component_rejected_v0",
            #[cfg(any(windows, unix))]
            Self::NotFound => "candidate_not_found_v0",
            #[cfg(any(windows, unix, test))]
            Self::NotFile => "candidate_is_not_one_regular_file_v0",
            #[cfg(any(windows, unix, test))]
            Self::TooLarge => "one_mibibyte_limit_exceeded_v0",
            #[cfg(any(windows, unix, test))]
            Self::InvalidUtf8 => "strict_utf8_decode_failed_v0",
            Self::IdentityUnavailable => "file_identity_unavailable_before_read_v0",
            Self::MissingProofEvidence => "proof_evidence_missing_before_read_v0",
            Self::ContradictoryBackingEvidence => "backing_evidence_contradicts_opened_object_v0",
            Self::IoFailed => "opaque_host_io_failure_v0",
        }
    }
}

/// A failed `open_checked`: the error plus explicit progress recording
/// how far the open got before failing. Failure evidence must report
/// only established progress — a late `IdentityUnavailable` happens
/// after the component walk completed, the open succeeded, and the
/// ordinary-file check ran — so the failure result carries that fact
/// instead of letting the consumer infer progress from a missing
/// identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpenCheckedFailure {
    pub(crate) error: FileReadAdapterError,
    pub(crate) progress: OpenProgress,
}

impl OpenCheckedFailure {
    pub(crate) fn new(error: FileReadAdapterError, progress: OpenProgress) -> Self {
        Self { error, progress }
    }
}

/// How far `open_checked` progressed before failing: which of the
/// evidence-relevant checks actually ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpenProgress {
    /// The component walk completed: `symlink_metadata` per component,
    /// no links followed, evidence validated.
    pub(crate) walk_completed: bool,
    /// The ordinary-file check ran on the opened handle — whether it
    /// passed (a later stage failed) or rejected the object (`NotFile`).
    pub(crate) ordinary_file_check_ran: bool,
}

impl OpenProgress {
    /// Nothing established: the walk did not complete.
    pub(crate) const NONE: Self = Self {
        walk_completed: false,
        ordinary_file_check_ran: false,
    };
    /// The walk completed — every component's metadata was collected and
    /// the walk's enforcement ran — but the open did not reach the
    /// ordinary-file check on the handle (validation, open, or metadata
    /// failed).
    pub(crate) const WALK_ONLY: Self = Self {
        walk_completed: true,
        ordinary_file_check_ran: false,
    };
    /// The open reached the identity stage: the walk completed and the
    /// ordinary-file check ran. Used when the caller holds an opened
    /// identity (any failure came after the open).
    pub(crate) const COMPLETE: Self = Self {
        walk_completed: true,
        ordinary_file_check_ran: true,
    };
}

/// Stable identity of the file object behind an opened handle.
///
/// WO29 Slice A: the read gate binds the walked path's identity to the
/// opened handle's identity *before* any payload byte is read, so a
/// component swapped between the component walk and the open cannot
/// redirect the read undetected.
///
/// The Windows variant is the documented stable identity contract:
/// `FILE_ID_INFO`'s 64-bit volume serial plus the 128-bit file reference
/// number (`GetFileInformationByHandleEx` with `FileIdInfo`) — unique and
/// stable on ReFS, unlike the legacy 64-bit `nFileIndex`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileObjectIdentity {
    // Never constructed on Windows (the Windows identity is
    // `WindowsVolumeFile`); still matched in `render`, so the allowance
    // is scoped to this platform only.
    #[cfg_attr(windows, allow(dead_code))]
    UnixDeviceInode { dev: u64, ino: u64 },
    #[cfg(windows)]
    WindowsVolumeFile {
        volume_serial: u64,
        file_id: [u8; 16],
    },
}

impl FileObjectIdentity {
    pub(crate) fn render(&self) -> String {
        match self {
            Self::UnixDeviceInode { dev, ino } => format!("dev={dev} ino={ino}"),
            #[cfg(windows)]
            Self::WindowsVolumeFile {
                volume_serial,
                file_id,
            } => {
                let mut hex = String::with_capacity(32);
                for byte in file_id {
                    hex.push_str(&format!("{byte:02x}"));
                }
                format!("volume_serial={volume_serial} file_id={hex}")
            }
        }
    }
}

/// The output of [`FileReadAdapter::open_checked`]: an opened, checked file
/// whose payload has NOT been read yet. `walked_identity` is the identity
/// observed during the component walk (`Some` on unix and on Windows, where
/// the final component's `FILE_ID_INFO` identity is read without following
/// reparse points); `opened_identity` is always present: a missing identity
/// surfaces as `Err`, never as a sentinel. `opened_volume_disks` is the
/// physical disk-number observation for the opened file's volume (`Some`
/// on Windows when the observation succeeded, `None` elsewhere): the live
/// backing bind consumes it against the classifier's recorded disk
/// numbers. A missing observation is trust-coverable; a mismatch rejects.
#[derive(Debug)]
pub(crate) struct OpenedCheckedFile {
    pub(crate) handle: Option<std::fs::File>,
    pub(crate) walked_identity: Option<FileObjectIdentity>,
    pub(crate) opened_identity: FileObjectIdentity,
    /// Windows-only: one coherent observation of the volume actually
    /// containing the opened file — the full 64-bit `FILE_ID_INFO`
    /// serial plus the complete extent set, resolved from the opened
    /// handle's own volume-GUID final path. On unix the (major, minor)
    /// device bind covers the backing identity, so there is no field at
    /// all (not even a `None`).
    #[cfg(windows)]
    pub(crate) opened_volume: Option<windows_drive_locality::OpenedVolumeObservation>,
}

/// Open/read split file adapter (WO29 Slice A). `open_checked` performs the
/// component walk, evidence validation, open, and handle checks WITHOUT
/// reading the payload; `read_opened` performs the bounded UTF-8 read on an
/// already-opened file. run.rs binds walked-to-opened identity between the
/// two calls, before any read. A failed open carries explicit progress
/// (`OpenCheckedFailure`): the evidence must report only established
/// progress, never infer it from a missing identity.
pub(crate) trait FileReadAdapter {
    fn open_checked(&mut self, path: &OsStr) -> Result<OpenedCheckedFile, OpenCheckedFailure>;
    fn read_opened(&mut self, opened: OpenedCheckedFile) -> Result<String, FileReadAdapterError>;
}

/// Step 5 of the Slice A gate: bind the walked identity to the opened
/// identity BEFORE any read.
///
/// Unix arm: the walk must have produced an identity and it must equal the
/// opened handle's identity (via [`opened_file_matches_walked_target`]),
/// else `UnsafePath`. Windows arm: the walk produces the final component's
/// 128-bit `FILE_ID_INFO` identity (read without following reparse points)
/// and it must equal the opened handle's identity, else `UnsafePath`. A
/// missing walked identity fails closed: there is nothing to bind against.
pub(crate) fn bind_walked_to_opened(
    walked: Option<FileObjectIdentity>,
    opened: FileObjectIdentity,
) -> Result<(), FileReadAdapterError> {
    #[cfg(unix)]
    {
        let Some(FileObjectIdentity::UnixDeviceInode {
            dev: walked_dev,
            ino: walked_ino,
        }) = walked
        else {
            return Err(FileReadAdapterError::UnsafePath);
        };
        let FileObjectIdentity::UnixDeviceInode {
            dev: opened_dev,
            ino: opened_ino,
        } = opened;
        if opened_file_matches_walked_target((walked_dev, walked_ino), (opened_dev, opened_ino)) {
            Ok(())
        } else {
            Err(FileReadAdapterError::UnsafePath)
        }
    }
    #[cfg(windows)]
    {
        let Some(FileObjectIdentity::WindowsVolumeFile {
            volume_serial: walked_serial,
            file_id: walked_id,
        }) = walked
        else {
            return Err(FileReadAdapterError::UnsafePath);
        };
        // A Unix identity on the Windows arm is a caller fabrication, not a
        // bindable opened file: fail closed.
        let FileObjectIdentity::WindowsVolumeFile {
            volume_serial: opened_serial,
            file_id: opened_id,
        } = opened
        else {
            return Err(FileReadAdapterError::UnsafePath);
        };
        if walked_serial == opened_serial && walked_id == opened_id {
            Ok(())
        } else {
            Err(FileReadAdapterError::UnsafePath)
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (walked, opened);
        // Unsupported host: no identity comparison is possible; fail closed
        // with the always-present variant (`UnsafePath` is cfg-gated away
        // here).
        Err(FileReadAdapterError::IdentityUnavailable)
    }
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

#[cfg(not(any(windows, unix)))]
impl FileReadAdapter for HostFileReadAdapter {
    fn open_checked(&mut self, _path: &OsStr) -> Result<OpenedCheckedFile, OpenCheckedFailure> {
        Err(OpenCheckedFailure::new(
            FileReadAdapterError::IoFailed,
            OpenProgress::NONE,
        ))
    }

    fn read_opened(&mut self, _opened: OpenedCheckedFile) -> Result<String, FileReadAdapterError> {
        Err(FileReadAdapterError::IoFailed)
    }
}

#[cfg(windows)]
impl FileReadAdapter for HostFileReadAdapter {
    fn open_checked(&mut self, path: &OsStr) -> Result<OpenedCheckedFile, OpenCheckedFailure> {
        open_checked_windows_file(path)
    }

    fn read_opened(&mut self, opened: OpenedCheckedFile) -> Result<String, FileReadAdapterError> {
        read_opened_handle(opened)
    }
}

#[cfg(unix)]
impl FileReadAdapter for HostFileReadAdapter {
    fn open_checked(&mut self, path: &OsStr) -> Result<OpenedCheckedFile, OpenCheckedFailure> {
        open_checked_unix_file(path)
    }

    fn read_opened(&mut self, opened: OpenedCheckedFile) -> Result<String, FileReadAdapterError> {
        read_opened_handle(opened)
    }
}

/// Read half of the split: the shared bounded UTF-8 read on the taken
/// handle (1 MiB, strict UTF-8). A missing handle means no opened file was
/// ever produced, so the identity required before the read is unavailable.
#[cfg(any(windows, unix))]
fn read_opened_handle(opened: OpenedCheckedFile) -> Result<String, FileReadAdapterError> {
    let handle = opened
        .handle
        .ok_or(FileReadAdapterError::IdentityUnavailable)?;
    read_bounded_utf8(handle)
}

#[cfg(any(windows, unix, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComponentKind {
    Directory,
    File,
    Other,
}

#[cfg(any(windows, unix, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ComponentEvidence {
    kind: ComponentKind,
    reparse: bool,
    length: u64,
    final_component: bool,
}

#[cfg(any(windows, unix, test))]
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

#[cfg(test)]
fn read_validated_content<R, Open>(
    evidence: &[ComponentEvidence],
    open: Open,
) -> Result<String, FileReadAdapterError>
where
    R: Read,
    Open: FnOnce() -> Result<R, FileReadAdapterError>,
{
    validate_component_evidence(evidence)?;
    // Open/read split: the open half hands back the reader; the read half is
    // the shared bounded UTF-8 decode. Production exposes the same split as
    // FileReadAdapter::open_checked / FileReadAdapter::read_opened.
    let reader = open()?;
    read_bounded_utf8(reader)
}

#[cfg(any(windows, unix, test))]
fn read_bounded_utf8<R: Read>(reader: R) -> Result<String, FileReadAdapterError> {
    let mut bytes = Vec::new();
    reader
        .take(FILE_READ_LIMIT_BYTES as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| FileReadAdapterError::IoFailed)?;
    String::from_utf8(bytes).map_err(|_| FileReadAdapterError::InvalidUtf8)
}

/// Windows open half of the split (WO29 Slice A): component walk with
/// `symlink_metadata` (never follows links), reparse-point rejection, then
/// open with NO payload read. Slice A adds the ordinary-file check on the
/// opened handle (fstat-style handle metadata: no path lookup, no symlink
/// following) plus capture of the opened handle's 128-bit `FILE_ID_INFO`
/// identity and the opened volume's backing-disk observation; a missing
/// identity fails closed with `IdentityUnavailable`.
#[cfg(windows)]
fn open_checked_windows_file(path: &OsStr) -> Result<OpenedCheckedFile, OpenCheckedFailure> {
    use std::fs::{self, File};
    use std::os::windows::fs::MetadataExt;
    use std::os::windows::io::AsRawHandle;
    use std::path::{Component, Path, PathBuf};

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;

    // Every failure below carries the progress established so far: the
    // evidence lines must report only what ran, never infer it.
    let fail = |error: FileReadAdapterError, progress: OpenProgress| {
        OpenCheckedFailure::new(error, progress)
    };

    let path = Path::new(path);
    let mut prefixes = Vec::new();
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match component {
            Component::Prefix(_) | Component::RootDir => {}
            Component::Normal(_) => prefixes.push(current.clone()),
            Component::CurDir | Component::ParentDir => {
                return Err(fail(FileReadAdapterError::UnsafePath, OpenProgress::NONE));
            }
        }
    }

    let mut evidence = Vec::with_capacity(prefixes.len());
    // The walked identity: the final component's volume serial and file
    // index, read WITHOUT following reparse points (the
    // `FILE_FLAG_OPEN_REPARSE_POINT` open in `walked_file_identity`). The
    // reparse rejection in `validate_component_evidence` guarantees the
    // final component is a real file, so this is the file's own identity.
    // Stable Rust's `MetadataExt` does not expose these (unstable
    // `windows_by_handle`); the FFI lives in the audited
    // `windows-drive-locality` crate. A missing identity fails closed:
    // without a walked identity there is nothing to bind the opened
    // handle against.
    let mut walked_identity: Option<FileObjectIdentity> = None;
    for (index, prefix) in prefixes.iter().enumerate() {
        let metadata = fs::symlink_metadata(prefix)
            .map_err(|error| fail(map_host_error(error), OpenProgress::NONE))?;
        let file_type = metadata.file_type();
        if index + 1 == prefixes.len() {
            use std::os::windows::ffi::OsStrExt;
            let mut wide: Vec<u16> = prefix.as_os_str().encode_wide().collect();
            wide.push(0);
            let identity =
                windows_drive_locality::walked_file_identity(&wide).ok_or_else(|| {
                    fail(
                        FileReadAdapterError::IdentityUnavailable,
                        OpenProgress::NONE,
                    )
                })?;
            walked_identity = Some(FileObjectIdentity::WindowsVolumeFile {
                volume_serial: identity.volume_serial,
                file_id: identity.file_id,
            });
        }
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
    validate_component_evidence(&evidence).map_err(|error| fail(error, OpenProgress::WALK_ONLY))?;

    let file =
        File::open(path).map_err(|error| fail(map_host_error(error), OpenProgress::WALK_ONLY))?;
    let opened_metadata = file
        .metadata()
        .map_err(|error| fail(map_host_error(error), OpenProgress::WALK_ONLY))?;
    if !opened_metadata.is_file() {
        return Err(fail(FileReadAdapterError::NotFile, OpenProgress::COMPLETE));
    }
    let identity =
        windows_drive_locality::opened_file_identity(file.as_raw_handle()).ok_or_else(|| {
            fail(
                FileReadAdapterError::IdentityUnavailable,
                OpenProgress::COMPLETE,
            )
        })?;
    // The opened volume's coherent observation (full 64-bit serial plus
    // complete extent set), taken from the opened object itself via its
    // volume-GUID final path — never re-derived from the walk's drive
    // letter. The live backing bind consumes it against the classifier's
    // recorded serial and disk numbers. A missing observation is
    // trust-coverable; it never invents a serial or disks.
    let opened_volume = windows_drive_locality::opened_volume_observation(file.as_raw_handle());
    Ok(OpenedCheckedFile {
        handle: Some(file),
        // The walked identity comes from the final component's
        // reparse-point-blind open above; run.rs binds it against the
        // opened handle's identity before any payload read.
        walked_identity,
        opened_identity: FileObjectIdentity::WindowsVolumeFile {
            volume_serial: identity.volume_serial,
            file_id: identity.file_id,
        },
        opened_volume,
    })
}

/// Unix open half of the split (WO29 Slice A): component walk with
/// `symlink_metadata` (never follows links), reparse/pipe/device rejection,
/// then open with NO payload read. The (dev, ino) identity is captured for
/// both the walked final component and the opened handle (`File::metadata`
/// is fstat on the open handle: no path lookup, no symlink following, no
/// TOCTOU on the final open). The walked-vs-opened COMPARISON lives in
/// run.rs (`bind_walked_to_opened`, step 5 of the Slice A gate) — this
/// function only captures, keeping the comparison pure and unit-testable.
/// The native path was already lexically validated, so `.`/`..` components
/// are unreachable; the walk keeps the guard anyway so the evidence chain
/// stays total.
#[cfg(unix)]
fn open_checked_unix_file(path: &OsStr) -> Result<OpenedCheckedFile, OpenCheckedFailure> {
    use std::fs::{self, File};
    use std::os::unix::fs::MetadataExt;
    use std::path::{Component, Path, PathBuf};

    // Every failure below carries the progress established so far: the
    // evidence lines must report only what ran, never infer it.
    let fail = |error: FileReadAdapterError, progress: OpenProgress| {
        OpenCheckedFailure::new(error, progress)
    };

    let path = Path::new(path);
    let mut prefixes = Vec::new();
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match component {
            Component::Prefix(_) | Component::RootDir => {}
            Component::Normal(_) => prefixes.push(current.clone()),
            Component::CurDir | Component::ParentDir => {
                return Err(fail(FileReadAdapterError::UnsafePath, OpenProgress::NONE));
            }
        }
    }

    let mut evidence = Vec::with_capacity(prefixes.len());
    let mut walked_identity: Option<(u64, u64)> = None;
    for (index, prefix) in prefixes.iter().enumerate() {
        let metadata = fs::symlink_metadata(prefix)
            .map_err(|error| fail(map_host_error(error), OpenProgress::NONE))?;
        let file_type = metadata.file_type();
        let final_component = index + 1 == prefixes.len();
        if final_component {
            walked_identity = Some((metadata.dev(), metadata.ino()));
        }
        evidence.push(ComponentEvidence {
            kind: if file_type.is_dir() {
                ComponentKind::Directory
            } else if file_type.is_file() {
                ComponentKind::File
            } else {
                ComponentKind::Other
            },
            // P4: on unix a symlink is the reparse point; devices, FIFOs,
            // and sockets land in `Other` and are rejected as non-files.
            reparse: file_type.is_symlink(),
            length: metadata.len(),
            final_component,
        });
    }
    let walked = walked_identity
        .ok_or_else(|| fail(FileReadAdapterError::UnsafePath, OpenProgress::NONE))?;
    validate_component_evidence(&evidence).map_err(|error| fail(error, OpenProgress::WALK_ONLY))?;

    let file =
        File::open(path).map_err(|error| fail(map_host_error(error), OpenProgress::WALK_ONLY))?;
    let opened = file
        .metadata()
        .map_err(|error| fail(map_host_error(error), OpenProgress::WALK_ONLY))?;
    if !opened.is_file() {
        return Err(fail(FileReadAdapterError::NotFile, OpenProgress::COMPLETE));
    }
    Ok(OpenedCheckedFile {
        handle: Some(file),
        walked_identity: Some(FileObjectIdentity::UnixDeviceInode {
            dev: walked.0,
            ino: walked.1,
        }),
        opened_identity: FileObjectIdentity::UnixDeviceInode {
            dev: opened.dev(),
            ino: opened.ino(),
        },
    })
}

/// Pure (dev, ino) comparison behind the unix bind: the handle is the walked
/// file only when both halves of the identity match. Kept pure so the
/// comparison itself is unit-testable; a real race cannot be tested
/// deterministically.
#[cfg(unix)]
fn opened_file_matches_walked_target(walked: (u64, u64), opened: (u64, u64)) -> bool {
    walked == opened
}

#[cfg(any(windows, unix))]
fn map_host_error(error: std::io::Error) -> FileReadAdapterError {
    match error.kind() {
        std::io::ErrorKind::NotFound => FileReadAdapterError::NotFound,
        std::io::ErrorKind::IsADirectory => FileReadAdapterError::NotFile,
        _ => FileReadAdapterError::IoFailed,
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    #[cfg(unix)]
    use super::OpenProgress;
    #[cfg(unix)]
    use super::bind_walked_to_opened;
    #[cfg(unix)]
    use super::opened_file_matches_walked_target;
    #[cfg(any(windows, unix))]
    use super::read_opened_handle;
    use super::{
        ComponentEvidence, ComponentKind, FILE_READ_LIMIT_BYTES, FileObjectIdentity,
        FileReadAdapter, FileReadAdapterError, HostFileReadAdapter, OpenedCheckedFile,
        read_bounded_utf8, read_validated_content, validate_component_evidence,
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

    #[cfg(any(windows, unix))]
    fn read_all_via_split(
        adapter: &mut HostFileReadAdapter,
        path: &OsStr,
    ) -> Result<String, FileReadAdapterError> {
        let opened = adapter
            .open_checked(path)
            .map_err(|failure| failure.error)?;
        adapter.read_opened(opened)
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
    fn file_object_identity_renders_canonical_forms() {
        assert_eq!(
            FileObjectIdentity::UnixDeviceInode { dev: 8, ino: 4242 }.render(),
            "dev=8 ino=4242"
        );
        #[cfg(windows)]
        assert_eq!(
            FileObjectIdentity::WindowsVolumeFile {
                volume_serial: 0x1234_5678,
                file_id: [
                    0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, //
                    0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78,
                ],
            }
            .render(),
            // 0x1234_5678 = 305419896; the file ID renders as 32 lowercase
            // hex digits.
            "volume_serial=305419896 file_id=9abcdef0123456789abcdef012345678"
        );
    }

    #[test]
    fn identity_unavailable_has_stable_variant_and_reason() {
        assert_eq!(
            FileReadAdapterError::IdentityUnavailable.variant(),
            "identity_unavailable"
        );
        assert_eq!(
            FileReadAdapterError::IdentityUnavailable.result_reason(),
            "file_identity_unavailable_before_read_v0"
        );
    }

    #[test]
    fn missing_and_contradictory_proof_evidence_are_distinguishable() {
        // Control (h), pure half: missing vs contradictory proof evidence
        // must never collapse into one variant (or into UnsafePath). The
        // fail-closed-before-read half lives with bind_proof_evidence in
        // run.rs.
        let missing = FileReadAdapterError::MissingProofEvidence;
        let contradictory = FileReadAdapterError::ContradictoryBackingEvidence;
        assert_eq!(missing.variant(), "missing_proof_evidence");
        assert_eq!(
            missing.result_reason(),
            "proof_evidence_missing_before_read_v0"
        );
        assert_eq!(contradictory.variant(), "contradictory_backing_evidence");
        assert_eq!(
            contradictory.result_reason(),
            "backing_evidence_contradicts_opened_object_v0"
        );
        assert_ne!(missing.variant(), contradictory.variant());
        assert_ne!(missing.result_reason(), contradictory.result_reason());
        assert_ne!(
            missing.variant(),
            FileReadAdapterError::UnsafePath.variant()
        );
        assert_ne!(
            contradictory.variant(),
            FileReadAdapterError::UnsafePath.variant()
        );
    }

    #[cfg(any(windows, unix))]
    #[test]
    fn read_opened_without_a_handle_is_identity_unavailable() {
        // A missing handle means no opened file was ever produced (test
        // double or caller bug): the read half fails closed instead of
        // inventing an identity.
        let opened = OpenedCheckedFile {
            handle: None,
            walked_identity: None,
            opened_identity: FileObjectIdentity::UnixDeviceInode { dev: 8, ino: 4242 },
            #[cfg(windows)]
            opened_volume: None,
        };
        assert_eq!(
            read_opened_handle(opened),
            Err(FileReadAdapterError::IdentityUnavailable)
        );
    }

    #[test]
    fn adapter_source_has_two_read_only_opens_and_no_widened_filesystem_surface() {
        let source = include_str!("file_read.rs");
        // One per platform open half: the Windows reparse-point walk and the
        // unix symlink walk. Both are read-only opens with no payload read.
        assert_eq!(source.matches(concat!("File::", "open(")).count(), 2);
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

    #[cfg(any(windows, unix))]
    #[test]
    fn host_adapter_reads_the_checked_in_utf8_fixture_without_writing() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/file_read/session_ad_utf8.txt");
        let mut adapter = HostFileReadAdapter;
        assert_eq!(
            read_all_via_split(&mut adapter, path.as_os_str()),
            Ok("Hum reads exact UTF-8: lambda=λ\n".to_string())
        );
    }

    #[cfg(unix)]
    #[test]
    fn p2_identity_comparison_requires_matching_dev_and_ino() {
        // The handle is the walked file: both halves of the identity match.
        assert!(opened_file_matches_walked_target((8, 4242), (8, 4242)));
        // Swapped file on the same device: the inode differs.
        assert!(!opened_file_matches_walked_target((8, 4242), (8, 4243)));
        // Same inode number on a different device: the device differs.
        assert!(!opened_file_matches_walked_target((8, 4242), (9, 4242)));
    }

    #[cfg(unix)]
    #[test]
    fn bind_walked_to_opened_requires_equal_unix_identities() {
        let walked = FileObjectIdentity::UnixDeviceInode { dev: 8, ino: 4242 };
        let opened = FileObjectIdentity::UnixDeviceInode { dev: 8, ino: 4242 };
        assert_eq!(bind_walked_to_opened(Some(walked), opened), Ok(()));
        // No walked identity: nothing to bind against.
        assert_eq!(
            bind_walked_to_opened(None, opened),
            Err(FileReadAdapterError::UnsafePath)
        );
        // Same device, swapped file: the inode differs.
        let swapped = FileObjectIdentity::UnixDeviceInode { dev: 8, ino: 4243 };
        assert_eq!(
            bind_walked_to_opened(Some(walked), swapped),
            Err(FileReadAdapterError::UnsafePath)
        );
        // Same inode number on a different device: the device differs.
        let other_device = FileObjectIdentity::UnixDeviceInode { dev: 9, ino: 4242 };
        assert_eq!(
            bind_walked_to_opened(Some(walked), other_device),
            Err(FileReadAdapterError::UnsafePath)
        );
    }

    #[cfg(windows)]
    #[test]
    fn bind_walked_to_opened_rejects_fabricated_walked_identity() {
        use super::bind_walked_to_opened;

        let opened = FileObjectIdentity::WindowsVolumeFile {
            volume_serial: 1,
            file_id: [2; 16],
        };
        // A Unix walked identity against a Windows opened identity is a
        // caller fabrication, not a bindable opened file: fail closed.
        let fabricated = FileObjectIdentity::UnixDeviceInode { dev: 8, ino: 4242 };
        assert_eq!(
            bind_walked_to_opened(Some(fabricated), opened),
            Err(FileReadAdapterError::UnsafePath)
        );
        // A missing walked identity is also fail-closed: there is nothing
        // to bind against. Production always captures the final
        // component's reparse-point-blind identity, so `None` here means a
        // test double skipped the walk.
        assert_eq!(
            bind_walked_to_opened(None, opened),
            Err(FileReadAdapterError::UnsafePath)
        );
    }

    #[cfg(windows)]
    #[test]
    fn bind_walked_to_opened_requires_full_file_id_equality() {
        use super::bind_walked_to_opened;

        let walked = FileObjectIdentity::WindowsVolumeFile {
            volume_serial: 0xC0DE_1234,
            file_id: [0x42; 16],
        };
        // Matching serial and full 128-bit file ID: the bind admits.
        assert_eq!(bind_walked_to_opened(Some(walked), walked), Ok(()));
        // Same volume serial, different file (different 128-bit ID): the
        // bind rejects. The serial alone never identifies the file.
        let other_file = FileObjectIdentity::WindowsVolumeFile {
            volume_serial: 0xC0DE_1234,
            file_id: [0x43; 16],
        };
        assert_eq!(
            bind_walked_to_opened(Some(walked), other_file),
            Err(FileReadAdapterError::UnsafePath)
        );
        // Different volume serial: the bind rejects.
        let other_volume = FileObjectIdentity::WindowsVolumeFile {
            volume_serial: 0xC0DE_1235,
            file_id: [0x42; 16],
        };
        assert_eq!(
            bind_walked_to_opened(Some(walked), other_volume),
            Err(FileReadAdapterError::UnsafePath)
        );
    }

    #[cfg(unix)]
    #[test]
    fn unix_adapter_rejects_symlinks_and_non_file_candidates() {
        use std::os::unix::fs::symlink;

        let scratch = std::env::temp_dir().join(format!("hum-unix-read-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).expect("scratch dir");
        // Symlink target is the checked-in UTF-8 fixture (absolute), so the
        // test creates no files of its own.
        let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/file_read/session_ad_utf8.txt");
        let link = scratch.join("link.txt");
        symlink(&target, &link).expect("symlink");

        let mut adapter = HostFileReadAdapter;
        // P4: a symlink is a reparse point on unix and is rejected even
        // though its target is an ordinary file.
        assert_eq!(
            read_all_via_split(&mut adapter, link.as_os_str()),
            Err(FileReadAdapterError::UnsafePath)
        );
        // A directory is not an ordinary file.
        assert_eq!(
            read_all_via_split(&mut adapter, scratch.as_os_str()),
            Err(FileReadAdapterError::NotFile)
        );
        assert_eq!(
            read_all_via_split(&mut adapter, target.as_os_str()),
            Ok("Hum reads exact UTF-8: lambda=λ\n".to_string())
        );
        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[cfg(unix)]
    #[test]
    fn unix_adapter_reports_missing_candidates_without_panic() {
        let mut adapter = HostFileReadAdapter;
        assert_eq!(
            read_all_via_split(&mut adapter, OsStr::new("/hum-definitely-absent-opaque")),
            Err(FileReadAdapterError::NotFound)
        );
    }

    /// Production-connected progress: the real adapter's failures carry
    /// explicit progress. A missing path fails in the walk (`NotFound`,
    /// nothing established); a directory completes the walk but is
    /// rejected by the walk's own enforcement (`NotFile`, walk done,
    /// handle never opened so the handle check did not run). The
    /// evidence layer must report exactly this.
    #[cfg(unix)]
    #[test]
    fn production_open_failures_carry_explicit_progress() {
        let mut adapter = HostFileReadAdapter;
        let missing = adapter
            .open_checked(OsStr::new("/hum-definitely-absent-opaque"))
            .expect_err("missing path must fail");
        assert_eq!(missing.error, FileReadAdapterError::NotFound);
        assert_eq!(missing.progress, OpenProgress::NONE);

        let dir = std::env::temp_dir();
        let not_file = adapter
            .open_checked(dir.as_os_str())
            .expect_err("directory must fail the walk's file enforcement");
        assert_eq!(not_file.error, FileReadAdapterError::NotFile);
        assert_eq!(not_file.progress, OpenProgress::WALK_ONLY);
    }

    #[cfg(not(any(windows, unix)))]
    #[test]
    fn unsupported_host_adapter_is_unavailable_without_file_access() {
        let mut adapter = HostFileReadAdapter;
        // `OpenedCheckedFile` is not `PartialEq` (it may own a live handle),
        // so the rejection is pinned with `matches!`, not `assert_eq!`.
        assert!(matches!(
            adapter.open_checked(OsStr::new("/not-accessed")),
            Err(failure) if failure.error == FileReadAdapterError::IoFailed
                && failure.progress == OpenProgress::NONE
        ));
        let opened = OpenedCheckedFile {
            handle: None,
            walked_identity: None,
            opened_identity: FileObjectIdentity::UnixDeviceInode { dev: 0, ino: 0 },
        };
        assert_eq!(
            adapter.read_opened(opened),
            Err(FileReadAdapterError::IoFailed)
        );
    }
}
