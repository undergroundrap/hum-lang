use std::ffi::{OsStr, OsString};
use std::fmt;

use windows_drive_locality::{DriveLocality, DriveRoot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativePathLocality {
    Unclassified,
    FixedLocal,
}

impl NativePathLocality {
    fn as_str(self) -> &'static str {
        match self {
            Self::Unclassified => "locality_unclassified",
            Self::FixedLocal => "fixed_local_v0",
        }
    }
}

#[derive(Clone)]
pub(crate) struct ValidatedNativePath {
    raw: OsString,
    locality: NativePathLocality,
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
            .field("locality", &self.locality.as_str())
            .finish()
    }
}

impl ValidatedNativePath {
    #[cfg(all(test, windows))]
    pub(crate) fn as_os_str(&self) -> &OsStr {
        &self.raw
    }

    #[cfg(all(test, windows))]
    pub(crate) fn locality(&self) -> &'static str {
        self.locality.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativePathIssue {
    #[cfg(not(windows))]
    UnsupportedHost,
    #[cfg(windows)]
    NotOrdinaryDriveRooted,
    #[cfg(windows)]
    NamespacePrefix,
    #[cfg(windows)]
    EmptyComponent,
    #[cfg(windows)]
    DotComponent,
    #[cfg(windows)]
    AlternateDataStream,
    #[cfg(windows)]
    TrailingDotOrSpace,
    #[cfg(windows)]
    DosDeviceAlias,
}

impl NativePathIssue {
    pub(crate) fn reason(self) -> &'static str {
        match self {
            #[cfg(not(windows))]
            Self::UnsupportedHost => "native_path_input_unavailable_on_non_windows_v0",
            #[cfg(windows)]
            Self::NotOrdinaryDriveRooted => "not_ordinary_drive_letter_rooted_v0",
            #[cfg(windows)]
            Self::NamespacePrefix => "windows_namespace_prefix_forbidden_v0",
            #[cfg(windows)]
            Self::EmptyComponent => "empty_path_component_forbidden_v0",
            #[cfg(windows)]
            Self::DotComponent => "dot_or_dot_dot_component_forbidden_v0",
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
            #[cfg(not(windows))]
            Self::UnsupportedHost => "native Path input is unavailable on this non-Windows host",
            #[cfg(windows)]
            Self::NotOrdinaryDriveRooted => {
                "the path is not an ordinary drive-letter-rooted Windows spelling"
            }
            #[cfg(windows)]
            Self::NamespacePrefix => {
                "the path uses a UNC, verbatim, device, or NT namespace prefix"
            }
            #[cfg(windows)]
            Self::EmptyComponent => "the path contains an empty component",
            #[cfg(windows)]
            Self::DotComponent => "the path contains `.` or `..` traversal",
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
    Ok(ValidatedNativePath {
        raw: raw.to_os_string(),
        locality: classify_validated_drive(raw),
    })
}

#[cfg(windows)]
fn classify_validated_drive(raw: &OsStr) -> NativePathLocality {
    use std::os::windows::ffi::OsStrExt;

    let letter = raw
        .encode_wide()
        .next()
        .and_then(|unit| u8::try_from(unit).ok())
        .and_then(DriveRoot::from_ascii_letter);
    letter
        .map(windows_drive_locality::classify)
        .map_or(NativePathLocality::Unclassified, locality_from_drive)
}

#[cfg(not(windows))]
fn classify_validated_drive(_raw: &OsStr) -> NativePathLocality {
    NativePathLocality::Unclassified
}

fn locality_from_drive(locality: DriveLocality) -> NativePathLocality {
    match locality {
        DriveLocality::FixedLocal => NativePathLocality::FixedLocal,
        DriveLocality::Remote
        | DriveLocality::Substituted
        | DriveLocality::Removable
        | DriveLocality::Unsupported
        | DriveLocality::Unknown => NativePathLocality::Unclassified,
    }
}

#[cfg(not(windows))]
fn validate_platform_path(_raw: &OsStr) -> Result<(), NativePathIssue> {
    Err(NativePathIssue::UnsupportedHost)
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
        assert_ne!(result, windows_drive_locality::DriveLocality::Unsupported);
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

    #[cfg(not(windows))]
    #[test]
    fn native_paths_are_unavailable_on_non_windows_hosts() {
        let candidate = format!("C:{}opaque", char::from(47));
        assert_eq!(
            validate_native_path(OsStr::new(&candidate)),
            Err(NativePathIssue::UnsupportedHost)
        );
    }
}
