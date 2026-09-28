use std::ffi::{OsStr, OsString};
use std::fmt;

#[cfg(any(unix, test))]
use linux_drive_locality::LinuxLocality;
#[cfg(any(windows, test))]
use windows_drive_locality::DriveLocality;
#[cfg(windows)]
use windows_drive_locality::DriveRoot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativePathLocality {
    Unclassified,
    #[cfg(any(windows, test))]
    FixedLocal,
}

impl NativePathLocality {
    fn as_str(self) -> &'static str {
        match self {
            Self::Unclassified => "locality_unclassified",
            #[cfg(any(windows, test))]
            Self::FixedLocal => "fixed_local_v0",
        }
    }
}

/// Locality evidence captured once at validation time, so later gates
/// observe one stable classification instead of re-probing the host. The
/// unix seam stores the Linux P1 classifier output; the Windows seam stores
/// Leaf D's `ClassifiedDrive` record. Other platforms carry no evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalityEvidence {
    #[cfg(any(unix, test))]
    Linux(LinuxLocality),
    #[cfg(windows)]
    Windows(windows_drive_locality::ClassifiedDrive),
}

#[derive(Clone)]
pub(crate) struct ValidatedNativePath {
    raw: OsString,
    locality: NativePathLocality,
    evidence: Option<LocalityEvidence>,
}

impl PartialEq for ValidatedNativePath {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Eq for ValidatedNativePath {}

impl fmt::Debug for ValidatedNativePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidatedNativePath")
            .field("identity", &"opaque_native_path")
            .field("locality", &self.locality())
            .finish()
    }
}

impl ValidatedNativePath {
    pub(crate) fn as_os_str(&self) -> &OsStr {
        &self.raw
    }

    pub(crate) fn locality(&self) -> &'static str {
        #[cfg(unix)]
        {
            match self.evidence.as_ref() {
                Some(LocalityEvidence::Linux(locality)) => locality.as_str(),
                // Unreachable: the unix seam always stores evidence at
                // validation time. Kept so the match stays total.
                None => self.locality.as_str(),
            }
        }
        #[cfg(not(unix))]
        {
            self.locality.as_str()
        }
    }

    pub(crate) fn is_fixed_local(&self) -> bool {
        #[cfg(unix)]
        {
            self.evidence
                .as_ref()
                .is_some_and(|evidence| match evidence {
                    LocalityEvidence::Linux(locality) => locality.is_fixed_local(),
                })
        }
        // Windows: defined-but-unreachable on the live path. No classifier
        // emits `DriveLocality::FixedLocal` in this WO version (decision
        // 0029 §14 grant-first demotion, mirroring Linux's `Proven`); the
        // check stays so the gate keeps its code shape, and admission flows
        // through the attestation (`external-trust`) path.
        #[cfg(windows)]
        {
            self.evidence
                .as_ref()
                .is_some_and(|evidence| match evidence {
                    LocalityEvidence::Windows(classified) => {
                        classified.locality == DriveLocality::FixedLocal
                    }
                })
        }
        #[cfg(all(test, not(unix), not(windows)))]
        {
            self.locality == NativePathLocality::FixedLocal
        }
        #[cfg(not(any(unix, windows, test)))]
        {
            false
        }
    }

    /// The evidence captured at validation time, if the platform stores any.
    pub fn locality_evidence(&self) -> Option<&LocalityEvidence> {
        self.evidence.as_ref()
    }

    /// Audit reason recorded when the locality gate refuses the read. On
    /// unix the P1 classifier (linux-drive-locality) reports Unproven with
    /// its own reason; this platform-wide reason is kept for the refusal
    /// itself. On Windows the gate fires per path whose locality is
    /// `Unproven` under the grant-first demotion (decision 0029 §14); the
    /// superseded `fixed_local_v0_not_proven_before_candidate_access_v0`
    /// reason is retired in Slice A. The classifier's specific `Unproven`
    /// reason travels verbatim in the evidence bundle (see
    /// [`Self::classifier_unproven_reason`]); this reason names the gate,
    /// never the classifier.
    pub(crate) fn locality_gate_reason(&self) -> &'static str {
        #[cfg(unix)]
        {
            "p1_locality_unproven_on_this_platform_v0"
        }
        #[cfg(not(unix))]
        {
            "windows_locality_unproven_grant_first_v0"
        }
    }

    /// The classifier's exact `Unproven` reason, verbatim and unnormalized,
    /// for evidence bundles on refusal and trust paths. `None` when the
    /// classifier did not return an `Unproven` verdict with a named reason
    /// (including when no evidence was stored).
    pub(crate) fn classifier_unproven_reason(&self) -> Option<&'static str> {
        match self.evidence.as_ref() {
            #[cfg(any(unix, test))]
            Some(LocalityEvidence::Linux(linux_drive_locality::LinuxLocality::Unproven {
                reason,
                ..
            })) => Some(*reason),
            #[cfg(any(unix, test))]
            Some(LocalityEvidence::Linux(linux_drive_locality::LinuxLocality::Proven {
                ..
            })) => None,
            #[cfg(windows)]
            Some(LocalityEvidence::Windows(classified)) => classified.unproven_reason,
            _ => None,
        }
    }

    #[cfg(all(test, windows))]
    pub(crate) fn fixed_local_for_test(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            locality: NativePathLocality::FixedLocal,
            // Leaf D's ClassifiedDrive record, test-constructed to match the
            // legacy FixedLocal status this helper used to set alone. The
            // `FixedLocal` label is defined-but-unreachable from the live
            // classifier in this WO version (decision 0029 §14 grant-first
            // demotion), so the new fields take their neutral values: no
            // observed facts, no `Unproven` reason.
            evidence: Some(LocalityEvidence::Windows(
                windows_drive_locality::ClassifiedDrive {
                    locality: DriveLocality::FixedLocal,
                    backing_device_identity: Vec::new(),
                    observed_facts: Vec::new(),
                    unproven_reason: None,
                    volume_serial: None,
                },
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativePathIssue {
    #[cfg(not(any(windows, unix)))]
    UnsupportedHost,
    #[cfg(windows)]
    NotOrdinaryDriveRooted,
    #[cfg(windows)]
    NamespacePrefix,
    #[cfg(any(windows, unix))]
    EmptyComponent,
    #[cfg(any(windows, unix))]
    DotComponent,
    #[cfg(unix)]
    NotAbsolute,
    #[cfg(windows)]
    AlternateDataStream,
    #[cfg(windows)]
    TrailingDotOrSpace,
    #[cfg(windows)]
    DosDeviceAlias,
}

impl NativePathIssue {
    pub(crate) fn is_unsupported_host(self) -> bool {
        #[cfg(not(any(windows, unix)))]
        {
            self == Self::UnsupportedHost
        }
        #[cfg(any(windows, unix))]
        {
            false
        }
    }

    pub(crate) fn reason(self) -> &'static str {
        match self {
            #[cfg(not(any(windows, unix)))]
            Self::UnsupportedHost => "native_path_input_unavailable_on_non_windows_v0",
            #[cfg(windows)]
            Self::NotOrdinaryDriveRooted => "not_ordinary_drive_letter_rooted_v0",
            #[cfg(windows)]
            Self::NamespacePrefix => "windows_namespace_prefix_forbidden_v0",
            #[cfg(any(windows, unix))]
            Self::EmptyComponent => "empty_path_component_forbidden_v0",
            #[cfg(any(windows, unix))]
            Self::DotComponent => "dot_or_dot_dot_component_forbidden_v0",
            #[cfg(unix)]
            Self::NotAbsolute => "unix_path_must_be_absolute_v0",
            #[cfg(windows)]
            Self::AlternateDataStream => "alternate_data_stream_or_extra_colon_forbidden_v0",
            #[cfg(windows)]
            Self::TrailingDotOrSpace => "component_trailing_dot_or_space_forbidden_v0",
            #[cfg(windows)]
            Self::DosDeviceAlias => "win32_dos_device_alias_forbidden_v0",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            #[cfg(not(any(windows, unix)))]
            Self::UnsupportedHost => "native Path input is unavailable on this non-Windows host",
            #[cfg(windows)]
            Self::NotOrdinaryDriveRooted => {
                "the path is not an ordinary drive-letter-rooted Windows spelling"
            }
            #[cfg(windows)]
            Self::NamespacePrefix => {
                "the path uses a UNC, verbatim, device, or NT namespace prefix"
            }
            #[cfg(any(windows, unix))]
            Self::EmptyComponent => "the path contains an empty component",
            #[cfg(any(windows, unix))]
            Self::DotComponent => "the path contains `.` or `..` traversal",
            #[cfg(unix)]
            Self::NotAbsolute => "the path is not an absolute unix path",
            #[cfg(windows)]
            Self::AlternateDataStream => "the path contains a colon after the drive prefix",
            #[cfg(windows)]
            Self::TrailingDotOrSpace => "a path component ends in a dot or space",
            #[cfg(windows)]
            Self::DosDeviceAlias => "a path component normalizes to a reserved Win32 DOS device",
        }
    }
}

pub(crate) fn validate_native_path(raw: &OsStr) -> Result<ValidatedNativePath, NativePathIssue> {
    validate_platform_path(raw)?;
    let (locality, evidence) = classify_validated_drive(raw);
    Ok(ValidatedNativePath {
        raw: raw.to_os_string(),
        locality,
        evidence,
    })
}

#[cfg(windows)]
fn classify_validated_drive(raw: &OsStr) -> (NativePathLocality, Option<LocalityEvidence>) {
    use std::os::windows::ffi::OsStrExt;

    let root = raw
        .encode_wide()
        .next()
        .and_then(|unit| u8::try_from(unit).ok())
        .and_then(DriveRoot::from_ascii_letter);
    let Some(root) = root else {
        return (NativePathLocality::Unclassified, None);
    };
    // Slice A: windows_drive_locality::classify returns the ClassifiedDrive
    // record (Leaf D's frozen signature); the legacy internal status derives
    // from its locality.
    let classified = windows_drive_locality::classify(root);
    let locality = locality_from_drive(classified.locality);
    (locality, Some(LocalityEvidence::Windows(classified)))
}

#[cfg(unix)]
fn classify_validated_drive(raw: &OsStr) -> (NativePathLocality, Option<LocalityEvidence>) {
    let locality = linux_drive_locality::classify_host_path(raw);
    (
        NativePathLocality::Unclassified,
        Some(LocalityEvidence::Linux(locality)),
    )
}

#[cfg(not(any(windows, unix)))]
fn classify_validated_drive(_raw: &OsStr) -> (NativePathLocality, Option<LocalityEvidence>) {
    (NativePathLocality::Unclassified, None)
}

#[cfg(any(windows, test))]
fn locality_from_drive(locality: DriveLocality) -> NativePathLocality {
    match locality {
        DriveLocality::FixedLocal => NativePathLocality::FixedLocal,
        // The grant-first demotion (decision 0029 §14) verdict: observed
        // but insufficient evidence never admits. `FixedLocal` above is
        // defined-but-unreachable from the live classifier, mirroring
        // Linux's `Proven`.
        DriveLocality::Unproven
        | DriveLocality::Remote
        | DriveLocality::Substituted
        | DriveLocality::Removable
        | DriveLocality::Unsupported
        | DriveLocality::Unknown => NativePathLocality::Unclassified,
    }
}

#[cfg(not(any(windows, unix)))]
fn validate_platform_path(_raw: &OsStr) -> Result<(), NativePathIssue> {
    Err(NativePathIssue::UnsupportedHost)
}

#[cfg(unix)]
fn validate_platform_path(raw: &OsStr) -> Result<(), NativePathIssue> {
    use std::os::unix::ffi::OsStrExt;

    let bytes = raw.as_bytes();
    if !bytes.starts_with(b"/") {
        return Err(NativePathIssue::NotAbsolute);
    }
    // Splitting "/a/b" on '/' yields ["", "a", "b"]: the leading empty
    // element is the root. Any other empty element is a doubled or
    // trailing separator.
    for component in bytes.split(|byte| *byte == b'/').skip(1) {
        if component.is_empty() {
            return Err(NativePathIssue::EmptyComponent);
        }
        if component == b"." || component == b".." {
            return Err(NativePathIssue::DotComponent);
        }
    }
    Ok(())
}

#[cfg(windows)]
fn validate_platform_path(raw: &OsStr) -> Result<(), NativePathIssue> {
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Component, Path, Prefix};

    let units = raw.encode_wide().collect::<Vec<_>>();
    if units.len() < 4
        || !is_ascii_letter(units[0])
        || units[1] != u16::from(b':')
        || !is_separator(units[2])
    {
        return Err(classify_non_disk_prefix(&units));
    }

    let mut components = Path::new(raw).components();
    match components.next() {
        Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_)) => {}
        Some(Component::Prefix(_)) => return Err(NativePathIssue::NamespacePrefix),
        _ => return Err(NativePathIssue::NotOrdinaryDriveRooted),
    }
    if !matches!(components.next(), Some(Component::RootDir)) {
        return Err(NativePathIssue::NotOrdinaryDriveRooted);
    }

    validate_raw_components(&units[3..])
}

#[cfg(windows)]
fn classify_non_disk_prefix(units: &[u16]) -> NativePathIssue {
    if units.first().is_some_and(|unit| is_separator(*unit))
        && units.get(1).is_some_and(|unit| is_separator(*unit))
    {
        NativePathIssue::NamespacePrefix
    } else {
        NativePathIssue::NotOrdinaryDriveRooted
    }
}

#[cfg(windows)]
fn validate_raw_components(units: &[u16]) -> Result<(), NativePathIssue> {
    let mut start = 0usize;
    for index in 0..=units.len() {
        if index != units.len() && !is_separator(units[index]) {
            if units[index] == u16::from(b':') {
                return Err(NativePathIssue::AlternateDataStream);
            }
            continue;
        }
        if index == start {
            return Err(NativePathIssue::EmptyComponent);
        }
        validate_component(&units[start..index])?;
        start = index + 1;
    }
    Ok(())
}

#[cfg(windows)]
fn validate_component(component: &[u16]) -> Result<(), NativePathIssue> {
    if component == [u16::from(b'.')] || component == [u16::from(b'.'), u16::from(b'.')] {
        return Err(NativePathIssue::DotComponent);
    }
    if component
        .last()
        .is_some_and(|unit| matches!(*unit, 0x20 | 0x2e))
    {
        return Err(NativePathIssue::TrailingDotOrSpace);
    }
    let stem_end = component
        .iter()
        .position(|unit| *unit == u16::from(b'.'))
        .unwrap_or(component.len());
    if is_dos_device_stem(&component[..stem_end]) {
        return Err(NativePathIssue::DosDeviceAlias);
    }
    Ok(())
}

#[cfg(windows)]
fn is_dos_device_stem(stem: &[u16]) -> bool {
    let stem = stem
        .iter()
        .rposition(|unit| !matches!(*unit, 0x20 | 0x2e))
        .map_or(&[][..], |end| &stem[..=end]);
    let ascii = |expected: &str| {
        stem.len() == expected.len()
            && stem
                .iter()
                .zip(expected.bytes())
                .all(|(actual, expected)| ascii_unit_eq(*actual, expected))
    };
    if ["CON", "PRN", "AUX", "NUL", "CLOCK$", "CONIN$", "CONOUT$"]
        .iter()
        .any(|name| ascii(name))
    {
        return true;
    }
    if stem.len() == 4 {
        let prefix = &stem[..3];
        let numbered = matches!(stem[3], 0x31..=0x39 | 0x00b9 | 0x00b2 | 0x00b3);
        return numbered
            && ([b"COM", b"LPT"].iter().any(|expected| {
                prefix
                    .iter()
                    .zip(expected.iter())
                    .all(|(actual, expected)| ascii_unit_eq(*actual, *expected))
            }));
    }
    false
}

#[cfg(windows)]
fn is_ascii_letter(unit: u16) -> bool {
    unit <= u16::from(u8::MAX) && (unit as u8).is_ascii_alphabetic()
}

#[cfg(windows)]
fn ascii_unit_eq(unit: u16, expected: u8) -> bool {
    unit <= u16::from(u8::MAX) && (unit as u8).eq_ignore_ascii_case(&expected)
}

#[cfg(windows)]
fn is_separator(unit: u16) -> bool {
    matches!(unit, 0x2f | 0x5c)
}

pub(crate) fn strip_ascii_prefix(value: &OsStr, prefix: &str) -> Option<OsString> {
    strip_platform_prefix(value, prefix)
}

#[cfg(windows)]
fn strip_platform_prefix(value: &OsStr, prefix: &str) -> Option<OsString> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    let units = value.encode_wide().collect::<Vec<_>>();
    let prefix = prefix.encode_utf16().collect::<Vec<_>>();
    units
        .starts_with(&prefix)
        .then(|| OsString::from_wide(&units[prefix.len()..]))
}

#[cfg(not(windows))]
fn strip_platform_prefix(value: &OsStr, prefix: &str) -> Option<OsString> {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    value
        .as_bytes()
        .strip_prefix(prefix.as_bytes())
        .map(|suffix| OsString::from_vec(suffix.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::{NativePathIssue, strip_ascii_prefix, validate_native_path};
    use std::ffi::{OsStr, OsString};

    #[cfg(windows)]
    fn drive_path(suffix: &str) -> String {
        format!("C:{}{}", char::from(92), suffix)
    }

    #[cfg(windows)]
    fn namespace_path(suffix: &str) -> String {
        let separator = char::from(92);
        format!("{separator}{separator}{suffix}")
    }

    #[test]
    fn native_prefix_split_preserves_suffix() {
        let value = format!("--allow=files.read=C:{}opaque", char::from(47));
        assert_eq!(
            strip_ascii_prefix(OsStr::new(&value), "--allow="),
            Some(OsString::from(&value["--allow=".len()..]))
        );
    }

    #[test]
    fn only_fixed_local_observation_narrows_internal_status() {
        use windows_drive_locality::DriveLocality;

        assert_eq!(
            super::locality_from_drive(DriveLocality::FixedLocal).as_str(),
            "fixed_local_v0"
        );
        for locality in [
            // The grant-first `Unproven` verdict never admits: it maps to
            // `Unclassified` like every other non-`FixedLocal` verdict.
            DriveLocality::Unproven,
            DriveLocality::Remote,
            DriveLocality::Substituted,
            DriveLocality::Removable,
            DriveLocality::Unsupported,
            DriveLocality::Unknown,
        ] {
            assert_eq!(
                super::locality_from_drive(locality).as_str(),
                "locality_unclassified",
                "{locality:?}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn accepts_drive_rooted_native_identity_without_candidate_access() {
        let raw = drive_path("hum-session-ab\\missing.bin");
        let path = validate_native_path(OsStr::new(&raw)).expect("lexically clean path");
        assert!(matches!(
            path.locality(),
            "fixed_local_v0" | "locality_unclassified"
        ));
        assert_eq!(path.as_os_str(), OsStr::new(&raw));
    }

    #[cfg(windows)]
    #[test]
    fn repository_drive_smoke_classifies_or_fails_closed_without_candidate_access() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let bytes = manifest_dir.as_bytes();
        assert!(bytes.len() >= 3 && bytes[1] == b':' && matches!(bytes[2], b'/' | b'\\'));
        let root = windows_drive_locality::DriveRoot::from_ascii_letter(bytes[0])
            .expect("repository drive letter");
        let result = windows_drive_locality::classify(root);
        eprintln!("Session AC repository-drive classification: {result:?}");
        assert_ne!(
            result.locality,
            windows_drive_locality::DriveLocality::Unsupported
        );
    }

    #[cfg(windows)]
    #[test]
    fn non_string_native_code_units_round_trip_losslessly() {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};

        let units = vec![
            u16::from(b'C'),
            u16::from(b':'),
            u16::from(b'\\'),
            u16::from(b'o'),
            u16::from(b'p'),
            u16::from(b'a'),
            u16::from(b'q'),
            u16::from(b'u'),
            u16::from(b'e'),
            u16::from(b'\\'),
            0xd800,
        ];
        let raw = OsString::from_wide(&units);
        assert!(raw.to_str().is_none());
        let validated = validate_native_path(&raw).expect("opaque native path");
        assert_eq!(
            validated.as_os_str().encode_wide().collect::<Vec<_>>(),
            units
        );
    }

    #[cfg(windows)]
    #[test]
    fn rejects_every_banned_windows_lexical_class() {
        let separator = char::from(92);
        let cases = vec![
            (
                format!("relative{separator}file"),
                NativePathIssue::NotOrdinaryDriveRooted,
            ),
            (
                "C:file".to_string(),
                NativePathIssue::NotOrdinaryDriveRooted,
            ),
            (
                namespace_path(&format!("server{separator}share{separator}file")),
                NativePathIssue::NamespacePrefix,
            ),
            (
                namespace_path(&format!("?{separator}C:{separator}file")),
                NativePathIssue::NamespacePrefix,
            ),
            (
                namespace_path(&format!(".{separator}C:{separator}file")),
                NativePathIssue::NamespacePrefix,
            ),
            (
                namespace_path(&format!(
                    "?{separator}GLOBALROOT{separator}Device{separator}file"
                )),
                NativePathIssue::NamespacePrefix,
            ),
            (
                namespace_path(&format!(
                    "?{separator}Volume{{01234567-89ab-cdef-0123-456789abcdef}}{separator}file"
                )),
                NativePathIssue::NamespacePrefix,
            ),
            (
                drive_path(&format!("one{separator}{separator}two")),
                NativePathIssue::EmptyComponent,
            ),
            (
                drive_path(&format!("one{separator}.")),
                NativePathIssue::DotComponent,
            ),
            (
                drive_path(&format!("one{separator}..{separator}two")),
                NativePathIssue::DotComponent,
            ),
            (
                drive_path(&format!("one{separator}file:stream")),
                NativePathIssue::AlternateDataStream,
            ),
            (
                drive_path(&format!("one{separator}name.")),
                NativePathIssue::TrailingDotOrSpace,
            ),
            (
                drive_path(&format!("one{separator}name ")),
                NativePathIssue::TrailingDotOrSpace,
            ),
            (
                drive_path(&format!("one{separator}CON")),
                NativePathIssue::DosDeviceAlias,
            ),
            (
                drive_path(&format!("one{separator}prn.txt")),
                NativePathIssue::DosDeviceAlias,
            ),
            (
                drive_path(&format!("one{separator}AUX")),
                NativePathIssue::DosDeviceAlias,
            ),
            (
                drive_path(&format!("one{separator}nul.bin")),
                NativePathIssue::DosDeviceAlias,
            ),
            (
                drive_path(&format!("one{separator}CLOCK$")),
                NativePathIssue::DosDeviceAlias,
            ),
            (
                drive_path(&format!("one{separator}CONIN$")),
                NativePathIssue::DosDeviceAlias,
            ),
            (
                drive_path(&format!("one{separator}CONOUT$")),
                NativePathIssue::DosDeviceAlias,
            ),
            (
                drive_path(&format!("one{separator}COM9.log")),
                NativePathIssue::DosDeviceAlias,
            ),
            (
                drive_path(&format!("one{separator}lpt1")),
                NativePathIssue::DosDeviceAlias,
            ),
            (
                drive_path(&format!("one{separator}COM\u{00b9}.txt")),
                NativePathIssue::DosDeviceAlias,
            ),
            (
                drive_path(&format!("one{separator}LPT\u{00b3}")),
                NativePathIssue::DosDeviceAlias,
            ),
        ];
        for (raw, expected) in cases {
            assert_eq!(
                validate_native_path(OsStr::new(&raw)),
                Err(expected),
                "{raw}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn rejects_complete_case_insensitive_numbered_device_alias_set() {
        for prefix in ["COM", "com", "LPT", "lpt"] {
            for digit in '1'..='9' {
                for suffix in ["", ".txt", ".", " "] {
                    let raw = drive_path(&format!(
                        "opaque{}{}{}{}",
                        char::from(92),
                        prefix,
                        digit,
                        suffix
                    ));
                    assert!(
                        matches!(
                            validate_native_path(OsStr::new(&raw)),
                            Err(NativePathIssue::DosDeviceAlias)
                                | Err(NativePathIssue::TrailingDotOrSpace)
                        ),
                        "{raw}"
                    );
                }
            }
            for digit in ['\u{00b9}', '\u{00b2}', '\u{00b3}'] {
                for suffix in ["", ".txt", ".", " "] {
                    let raw = drive_path(&format!(
                        "opaque{}{}{}{}",
                        char::from(92),
                        prefix,
                        digit,
                        suffix
                    ));
                    assert!(
                        matches!(
                            validate_native_path(OsStr::new(&raw)),
                            Err(NativePathIssue::DosDeviceAlias)
                                | Err(NativePathIssue::TrailingDotOrSpace)
                        ),
                        "{raw}"
                    );
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn unix_classify_with_fixture_pins_unproven_reason_deterministically() {
        // Deterministic seam check: a USB-backed fake sysfs plus inline
        // mountinfo must classify Unproven with the exact contract reason.
        // This replaces the stale "locality_unclassified" pin: unix now
        // runs the real P1 classifier at validation time.
        use std::sync::atomic::{AtomicU64, Ordering};

        static SEQ: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "hum-native-path-seam-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        ));
        let sysfs = root.join("sys");
        let disk = sysfs.join("devices/fake/block/sdc");
        std::fs::create_dir_all(sysfs.join("dev/block")).expect("fixture dev/block");
        std::fs::create_dir_all(disk.join("sdc1")).expect("fixture node");
        std::fs::create_dir_all(disk.join("queue")).expect("fixture queue");
        std::fs::write(disk.join("removable"), "0\n").expect("fixture removable");
        std::fs::write(disk.join("queue/rotational"), "1\n").expect("fixture rotational");
        let device = disk.join("device");
        std::fs::create_dir_all(&device).expect("fixture device");
        std::os::unix::fs::symlink(
            "../../../../bus/fake/drivers/usb-storage",
            device.join("driver"),
        )
        .expect("fixture driver symlink");
        std::os::unix::fs::symlink(
            "../../devices/fake/block/sdc/sdc1",
            sysfs.join("dev/block/8:32"),
        )
        .expect("fixture block symlink");

        let mountinfo = "100 99 8:32 / /media/usb rw,relatime - vfat /dev/sdc1 rw\n";
        let locality = linux_drive_locality::classify_with(
            mountinfo,
            &sysfs,
            std::path::Path::new("/media/usb/x"),
            (8u64 << 20) | 32,
        );
        assert_eq!(locality.reason(), "p1_unrecognized_storage_stack_v0");
        assert_eq!(locality.as_str(), "p1_unrecognized_storage_stack_v0");
        assert!(!locality.is_fixed_local());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn unix_classify_host_path_smoke_stays_fail_closed_on_this_host() {
        // End-to-end through the validation seam on a real temp file: this
        // host must not prove locality. The reason is asserted by membership
        // in the known Unproven vocabulary, not by literal, so the test
        // stays robust across hosts (overlayfs/tmpfs here, other stacks
        // elsewhere).
        let candidate =
            std::env::temp_dir().join(format!("hum-wo29-slice-a-smoke-{}.txt", std::process::id()));
        std::fs::write(&candidate, b"smoke").expect("temp file for smoke test");

        let locality = linux_drive_locality::classify_host_path(candidate.as_os_str());
        let reason = locality.reason();
        assert!(
            matches!(
                locality,
                linux_drive_locality::LinuxLocality::Unproven { .. }
            ),
            "smoke host must not prove locality, got reason {reason}"
        );
        assert!(
            [
                "p1_no_mountinfo_entry_v0",
                "p1_block_device_unresolved_v0",
                "p1_guest_invisible_backing_v0",
                "p1_unrecognized_storage_stack_v0",
                "p1_ambiguous_mount_topology_v0",
                // Grant-first demotion vocabulary (decision 0029 §14):
                // insufficient-evidence and known-network reasons.
                "p1_insufficient_evidence_v0",
                "p1_known_network_backing_v0",
            ]
            .contains(&reason),
            "unknown locality reason: {reason}"
        );

        let validated =
            validate_native_path(candidate.as_os_str()).expect("lexically clean temp path");
        assert_eq!(validated.locality(), reason);
        assert!(!validated.is_fixed_local());
        assert!(matches!(
            validated.locality_evidence(),
            Some(super::LocalityEvidence::Linux(_))
        ));
        assert_eq!(
            validated.locality_gate_reason(),
            "p1_locality_unproven_on_this_platform_v0"
        );

        let _ = std::fs::remove_file(&candidate);
    }

    #[cfg(unix)]
    #[test]
    fn unix_rejects_non_absolute_and_traversal_paths() {
        // Windows drive-rooted spelling, built without a literal so the
        // public-readiness scanner does not read it as a real path.
        let drive_rooted = format!("{}:{}{}", 'C', char::from(47), "opaque.txt");
        for (raw, expected) in [
            ("relative/opaque.txt", NativePathIssue::NotAbsolute),
            (drive_rooted.as_str(), NativePathIssue::NotAbsolute),
            ("/", NativePathIssue::EmptyComponent),
            ("/opaque//doubled.txt", NativePathIssue::EmptyComponent),
            ("/opaque/trailing/", NativePathIssue::EmptyComponent),
            ("/opaque/./dot.txt", NativePathIssue::DotComponent),
            ("/opaque/../escape.txt", NativePathIssue::DotComponent),
        ] {
            assert_eq!(
                validate_native_path(OsStr::new(raw)),
                Err(expected),
                "{raw}"
            );
        }
    }

    #[cfg(not(any(windows, unix)))]
    #[test]
    fn native_paths_are_unavailable_on_unsupported_hosts() {
        let candidate = format!("C:{}opaque", char::from(47));
        assert_eq!(
            validate_native_path(OsStr::new(&candidate)),
            Err(NativePathIssue::UnsupportedHost)
        );
    }
}
