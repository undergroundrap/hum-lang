//! Linux P1 (not network-backed) storage classification for WO29 Slice A.
//!
//! The classifier walks a fail-closed pipeline over the guest-visible
//! storage stack:
//!
//! `/proc/self/mountinfo` (octal escapes decoded before matching; longest
//! component-wise mount prefix wins; duplicate identical mount points fail
//! closed) -> `/sys/dev/block/<major>:<minor>` (symlink followed, or the
//! node read as a directory) -> the block device's `device/` directory
//! (partition nodes expose no `device/`, so one level of parent walk-up to
//! the containing disk is used) -> the `driver` symlink basename -> HBA
//! allowlist check -> `removable == "0"`.
//!
//! `queue/rotational` is read for the evidence record only; it never gates.
//!
//! DISPUTED (BDFL-relayed Codex review, 2026-09-27): the positive promotion
//! to `Proven` on guest-visible PCIe/NVMe transport or an allowlisted HBA
//! driver is an unresolved specification dependency — guest-visible
//! transport/HBA alone does not establish host-local backing. The rule is
//! implemented as drafted, pending specification resolution; see the
//! annotation at the `Proven` construction site. Nothing here presents the
//! promotion as settled.
//!
//! Virtual and stacked devices (dm-*, md*, loop, virtio-blk, nvme-tcp,
//! vda/xvd, ...) never reach `Proven`: the classifier performs NO recursion
//! into `slaves/` — a stacked device whose slaves look local is still
//! `Unproven` with `p1_guest_invisible_backing_v0`.
//!
//! Every other outcome is fail-closed `Unproven` with a stable reason
//! string. Evidence lines record observations only.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// dev_t packing width per `<linux/kdev_t.h>`: `MINORBITS = 20`.
const DEV_T_MINOR_BITS: u32 = 20;
/// dev_t minor mask per `<linux/kdev_t.h>`: the low 20 bits.
const DEV_T_MINOR_MASK: u64 = 0xF_FFFF;

const REASON_NO_MOUNTINFO_ENTRY: &str = "p1_no_mountinfo_entry_v0";
const REASON_BLOCK_DEVICE_UNRESOLVED: &str = "p1_block_device_unresolved_v0";
const REASON_GUEST_INVISIBLE_BACKING: &str = "p1_guest_invisible_backing_v0";
const REASON_UNRECOGNIZED_STORAGE_STACK: &str = "p1_unrecognized_storage_stack_v0";
const REASON_AMBIGUOUS_MOUNT_TOPOLOGY: &str = "p1_ambiguous_mount_topology_v0";
const REASON_PROVED: &str = "proved_local_v0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinuxLocality {
    Proven {
        device: (u32, u32),
        evidence: Vec<String>,
    },
    Unproven {
        reason: &'static str,
    },
}

impl LinuxLocality {
    pub fn is_fixed_local(&self) -> bool {
        matches!(self, Self::Proven { .. })
    }

    pub fn reason(&self) -> &'static str {
        match self {
            Self::Proven { .. } => REASON_PROVED,
            Self::Unproven { reason } => reason,
        }
    }

    pub fn as_str(&self) -> &'static str {
        self.reason()
    }
}

/// Host-bus-adapter driver modules admitted by the (disputed) promotion
/// rule. USB (`usb-storage`), virtual (`virtio_blk`), fabric (`nvme-tcp`)
/// and unknown drivers are absent by design: they fail closed through the
/// `Unproven` reasons instead of promoting.
pub fn hba_allowlist() -> &'static [&'static str] {
    &["ahci", "ata_piix", "mpt2sas", "mpt3sas"]
}

/// Decode a `dev_t` into `(major, minor)` without libc, per
/// `<linux/kdev_t.h>`: `MINORBITS = 20`, so the device number packs as
/// `(major << 20) | minor`.
fn decode_dev(dev: u64) -> (u32, u32) {
    let major = (dev >> DEV_T_MINOR_BITS) as u32;
    let minor = (dev & DEV_T_MINOR_MASK) as u32;
    (major, minor)
}

struct MountEntry {
    mountpoint: PathBuf,
    fstype: String,
    source: String,
    major: u32,
    minor: u32,
}

fn parse_mountinfo(mountinfo: &str) -> Vec<MountEntry> {
    mountinfo.lines().filter_map(parse_mountinfo_line).collect()
}

fn parse_mountinfo_line(line: &str) -> Option<MountEntry> {
    // mountinfo fields: id parent major:minor root mountpoint options...
    // "-" fstype source super-options. Whitespace inside fields is octal
    // escaped (\040), so splitting on whitespace before decoding is safe.
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let separator = tokens.iter().position(|token| *token == "-")?;
    let pre = &tokens[..separator];
    let post = &tokens[separator + 1..];
    if pre.len() < 6 || post.len() < 2 {
        return None;
    }
    let (major, minor) = pre[2].split_once(':')?;
    Some(MountEntry {
        mountpoint: PathBuf::from(unescape_mountinfo_field(pre[4])),
        fstype: unescape_mountinfo_field(post[0]),
        source: unescape_mountinfo_field(post[1]),
        major: major.parse().ok()?,
        minor: minor.parse().ok()?,
    })
}

/// Decode mountinfo octal escapes (`\040` -> space, `\011` -> tab,
/// `\012` -> newline, `\134` -> backslash) BEFORE matching, so an escaped
/// mount point compares equal to the real path.
fn unescape_mountinfo_field(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\'
            && let Some(octal) = bytes.get(index + 1..index + 4)
            && octal
                .iter()
                .all(|byte| byte.is_ascii_digit() && *byte < b'8')
        {
            let value = ((octal[0] - b'0') << 6) | ((octal[1] - b'0') << 3) | (octal[2] - b'0');
            out.push(value);
            index += 4;
            continue;
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Component-wise prefix test: mount point `/data` must never prefix-match
/// path `/data2/x`. Compares path components after splitting on `/`, never
/// raw string prefixes.
fn is_component_prefix(mountpoint: &Path, path: &Path) -> bool {
    let mut mount_components = mountpoint.components();
    let mut path_components = path.components();
    loop {
        match (mount_components.next(), path_components.next()) {
            (None, _) => return true,
            (Some(expected), Some(actual)) => {
                if expected != actual {
                    return false;
                }
            }
            (Some(_), None) => return false,
        }
    }
}

/// Select the mountinfo entry for `path`: the longest component-wise prefix
/// wins (documented topology: the most specific mount is the topmost).
/// Duplicate identical mount points — or any otherwise ambiguous selection —
/// fail closed with `p1_ambiguous_mount_topology_v0`; there is deliberately
/// no tiebreak, in particular never by numeric mount ID.
fn select_mount<'a>(
    entries: &'a [MountEntry],
    path: &Path,
) -> Result<&'a MountEntry, &'static str> {
    let mut best: Option<&MountEntry> = None;
    let mut best_components = 0usize;
    let mut ambiguous = false;
    for entry in entries {
        if !is_component_prefix(&entry.mountpoint, path) {
            continue;
        }
        let components = entry.mountpoint.components().count();
        if best.is_none() || components > best_components {
            best = Some(entry);
            best_components = components;
            ambiguous = false;
        } else if components == best_components {
            // Same component length and both prefix `path` means identical
            // mount points: the selection is ambiguous, never tiebreak.
            ambiguous = true;
        }
    }
    match (best, ambiguous) {
        (None, _) => Err(REASON_NO_MOUNTINFO_ENTRY),
        (Some(_), true) => Err(REASON_AMBIGUOUS_MOUNT_TOPOLOGY),
        (Some(entry), false) => Ok(entry),
    }
}

/// Filesystems with no block device behind them. Their backing is invisible
/// to the guest's block layer by construction, so there is no sysfs node to
/// prove — the honest bucket is guest-invisible, not a lookup failure.
fn is_guest_invisible_fstype(fstype: &str) -> bool {
    matches!(fstype, "tmpfs" | "overlay")
}

/// Stacked or paravirtual block names whose backing the guest cannot see:
/// device-mapper (`dm-*`), MD RAID (`md*`), loop, virtio (`vd*`), Xen
/// (`xvd*`), and network block devices (`nbd*`, `rbd*`). Checked on the
/// resolved block node itself; the classifier never recurses into `slaves/`,
/// so a stacked device with local-looking slaves stays guest-invisible.
fn is_guest_invisible_block_name(name: &str) -> bool {
    name.starts_with("dm-")
        || name.starts_with("md")
        || name.starts_with("loop")
        || name.starts_with("vd")
        || name.starts_with("xvd")
        || name.starts_with("nbd")
        || name.starts_with("rbd")
}

fn is_guest_invisible_driver(driver: &str) -> bool {
    matches!(driver, "virtio_blk" | "nvme-tcp")
}

/// Admission check for the driver name feeding the disputed promotion rule.
/// The allowlist names HBA controller driver modules; plain NVMe attaches
/// via the in-kernel `nvme` host driver with no HBA in the path, and
/// non-removable MMC/SD cards attach via the in-kernel `mmcblk` block
/// driver. Both are admitted alongside the allowlist; removability is
/// enforced separately by the `removable == "0"` gate below, so an MMC/SD
/// device that reports removable stays Unproven.
///
/// NOTE: this admission feeds the DISPUTED promotion rule (see the
/// annotation at the `Proven` construction site): guest-visible PCIe/NVMe
/// or an allowlisted HBA alone does not establish host-local backing.
/// Implemented as drafted, pending specification resolution (BDFL-relayed
/// Codex review, 2026-09-27).
fn is_direct_local_driver(driver: &str) -> bool {
    driver == "nvme" || driver == "mmcblk" || hba_allowlist().contains(&driver)
}

/// Resolve `<sysfs>/dev/block/<major>:<minor>`: follow the symlink when it
/// is one (lexically, so fixture trees work), otherwise read the node as a
/// directory. Returns the block directory only if it exists.
fn resolve_block_dir(link: &Path) -> Option<PathBuf> {
    if let Ok(target) = std::fs::read_link(link) {
        let base = link.parent().unwrap_or(link);
        let resolved = lexical_normalize(&base.join(target));
        return resolved.is_dir().then_some(resolved);
    }
    link.is_dir().then_some(link.to_path_buf())
}

/// Collapse `.` and `..` lexically, without touching the filesystem.
fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            _ => out.push(component.as_os_str()),
        }
    }
    out
}

/// Whole-disk sysfs nodes expose `device/` directly. Partition nodes
/// (`sda/sda1`) do not, so walk up one level to the containing disk.
/// Returns `(disk_dir, device_dir)`.
fn disk_and_device_dirs(block_dir: &Path) -> Option<(PathBuf, PathBuf)> {
    let direct = block_dir.join("device");
    if dir_or_symlink(&direct) {
        return Some((block_dir.to_path_buf(), direct));
    }
    let parent = block_dir.parent()?;
    let via_parent = parent.join("device");
    if dir_or_symlink(&via_parent) {
        return Some((parent.to_path_buf(), via_parent));
    }
    None
}

fn dir_or_symlink(path: &Path) -> bool {
    path.is_dir()
        || std::fs::symlink_metadata(path)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
}

pub fn classify_with(
    mountinfo: &str,
    sysfs_root: &Path,
    path: &Path,
    st_dev: u64,
) -> LinuxLocality {
    // dev_t decode per <linux/kdev_t.h> MINORBITS=20, no libc.
    let (major, minor) = decode_dev(st_dev);

    let entries = parse_mountinfo(mountinfo);
    let entry = match select_mount(&entries, path) {
        Ok(entry) => entry,
        Err(reason) => return LinuxLocality::Unproven { reason },
    };
    // Consistency: the selected mount entry must describe the path's actual
    // device. A stale mountinfo view (entry device != st_dev device) fails
    // closed at the mountinfo stage.
    if entry.major != major || entry.minor != minor {
        return LinuxLocality::Unproven {
            reason: REASON_NO_MOUNTINFO_ENTRY,
        };
    }

    if is_guest_invisible_fstype(&entry.fstype) {
        return LinuxLocality::Unproven {
            reason: REASON_GUEST_INVISIBLE_BACKING,
        };
    }

    let block_link = sysfs_root
        .join("dev/block")
        .join(format!("{major}:{minor}"));
    let Some(block_dir) = resolve_block_dir(&block_link) else {
        return LinuxLocality::Unproven {
            reason: REASON_BLOCK_DEVICE_UNRESOLVED,
        };
    };

    if block_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(is_guest_invisible_block_name)
    {
        return LinuxLocality::Unproven {
            reason: REASON_GUEST_INVISIBLE_BACKING,
        };
    }

    let Some((disk_dir, device_dir)) = disk_and_device_dirs(&block_dir) else {
        // The block node resolved but its sysfs identity (`device/`) is
        // absent: fail closed at the resolution stage.
        return LinuxLocality::Unproven {
            reason: REASON_BLOCK_DEVICE_UNRESOLVED,
        };
    };

    let driver = std::fs::read_link(device_dir.join("driver"))
        .ok()
        .and_then(|target| {
            target
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        });
    let Some(driver) = driver else {
        // A missing driver link means the guest cannot see the backing:
        // the guest-invisible bucket, per the contract.
        return LinuxLocality::Unproven {
            reason: REASON_GUEST_INVISIBLE_BACKING,
        };
    };

    if is_guest_invisible_driver(&driver) {
        return LinuxLocality::Unproven {
            reason: REASON_GUEST_INVISIBLE_BACKING,
        };
    }
    if !is_direct_local_driver(&driver) {
        return LinuxLocality::Unproven {
            reason: REASON_UNRECOGNIZED_STORAGE_STACK,
        };
    }

    let removable =
        std::fs::read_to_string(disk_dir.join("removable")).map(|text| text.trim().to_string());
    let Ok(removable) = removable else {
        return LinuxLocality::Unproven {
            reason: REASON_BLOCK_DEVICE_UNRESOLVED,
        };
    };
    if removable != "0" {
        return LinuxLocality::Unproven {
            reason: REASON_UNRECOGNIZED_STORAGE_STACK,
        };
    }

    // Informational only: read for the evidence record, never a gate.
    let rotational = std::fs::read_to_string(disk_dir.join("queue/rotational"))
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|_| "unread".to_string());

    let evidence = vec![
        format!(
            "mountinfo: selected mountpoint={} fstype={} source={} dev={major}:{minor} for path={}",
            entry.mountpoint.display(),
            entry.fstype,
            entry.source,
            path.display(),
        ),
        format!(
            "mountinfo: st_dev={st_dev} decodes to {major}:{minor} via MINORBITS=20 (<linux/kdev_t.h>); selected entry dev={}:{} matches",
            entry.major, entry.minor,
        ),
        format!(
            "sysfs: {} resolves to {}",
            block_link.display(),
            block_dir.display(),
        ),
        format!(
            "sysfs: disk={} driver={driver} (basename of {}/driver symlink target)",
            disk_dir.display(),
            device_dir.display(),
        ),
        format!(
            "sysfs: removable=0 rotational={rotational} (rotational is informational only, not a gate)",
        ),
    ];

    // DISPUTED PROMOTION RULE (BDFL-relayed Codex review, 2026-09-27): the
    // promotion to Proven here is an unresolved specification dependency.
    // Guest-visible PCIe/NVMe transport or an allowlisted HBA driver alone
    // does not establish host-local backing (a guest can observe virtual
    // PCIe/NVMe devices and emulated HBAs). This rule is implemented as
    // drafted, pending specification resolution — do not treat Proven as a
    // settled host-locality claim.
    LinuxLocality::Proven {
        device: (major, minor),
        evidence,
    }
}

/// Classify a host path using the live `/proc/self/mountinfo` and `/sys`.
/// Any failure to observe the pipeline fails closed with the honest reason.
#[cfg(unix)]
pub fn classify_host_path(path: &OsStr) -> LinuxLocality {
    use std::os::unix::fs::MetadataExt;

    let mountinfo = std::fs::read_to_string("/proc/self/mountinfo").unwrap_or_default();
    let Ok(metadata) = std::fs::metadata(path) else {
        // Without metadata there is no st_dev to decode, so no mountinfo
        // entry can be tied to this path: fail closed at the mountinfo stage.
        return LinuxLocality::Unproven {
            reason: REASON_NO_MOUNTINFO_ENTRY,
        };
    };
    classify_with(
        &mountinfo,
        Path::new("/sys"),
        Path::new(path),
        metadata.dev(),
    )
}

/// Non-unix hosts have no `/proc/self/mountinfo` and no Linux sysfs, so the
/// mountinfo stage yields no entry. The classifier still compiles and fails
/// closed with the mountinfo-stage reason rather than inventing a new
/// platform literal.
#[cfg(not(unix))]
pub fn classify_host_path(_path: &OsStr) -> LinuxLocality {
    LinuxLocality::Unproven {
        reason: REASON_NO_MOUNTINFO_ENTRY,
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

    /// Fake sysfs tree under a unique temp dir, removed on drop.
    struct FakeSysfs {
        root: PathBuf,
        sysfs: PathBuf,
    }

    impl FakeSysfs {
        fn new(tag: &str) -> Self {
            let sequence = FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!(
                "hum-linux-locality-{tag}-{}-{sequence}",
                std::process::id()
            ));
            let sysfs = root.join("sys");
            std::fs::create_dir_all(sysfs.join("dev/block")).expect("fixture dev/block");
            Self { root, sysfs }
        }

        /// Adds block node `<major>:<minor>` as a symlink to
        /// `devices/fake/block/<disk>/<node>`. `driver = None` omits the
        /// `device/driver` symlink; `device_dir = false` omits `device/`
        /// entirely (virtual stacked devices like dm-0 expose none).
        #[allow(clippy::too_many_arguments)]
        fn add_block(
            &self,
            major: u32,
            minor: u32,
            disk: &str,
            node: &str,
            driver: Option<&str>,
            device_dir: bool,
            removable: &str,
            rotational: &str,
        ) {
            let disk_dir = self.sysfs.join("devices/fake/block").join(disk);
            std::fs::create_dir_all(disk_dir.join(node)).expect("fixture node dir");
            std::fs::create_dir_all(disk_dir.join("queue")).expect("fixture queue dir");
            std::fs::write(disk_dir.join("removable"), removable).expect("fixture removable");
            std::fs::write(disk_dir.join("queue/rotational"), rotational)
                .expect("fixture rotational");
            if device_dir {
                let device = disk_dir.join("device");
                std::fs::create_dir_all(&device).expect("fixture device dir");
                if let Some(driver) = driver {
                    std::os::unix::fs::symlink(
                        format!("../../../../bus/fake/drivers/{driver}"),
                        device.join("driver"),
                    )
                    .expect("fixture driver symlink");
                }
            }
            std::os::unix::fs::symlink(
                format!("../../devices/fake/block/{disk}/{node}"),
                self.sysfs.join(format!("dev/block/{major}:{minor}")),
            )
            .expect("fixture block symlink");
        }

        /// Adds `slaves/<link>` symlinks under a stacked device, pointing at
        /// other fake disks. The classifier must never consult these (no
        /// slave recursion); they exist only to pin that a local-looking
        /// slave does not promote the stack.
        fn add_slaves(&self, disk: &str, slaves: &[(&str, &str)]) {
            let slaves_dir = self
                .sysfs
                .join("devices/fake/block")
                .join(disk)
                .join("slaves");
            std::fs::create_dir_all(&slaves_dir).expect("fixture slaves dir");
            for (link, target_disk) in slaves {
                std::os::unix::fs::symlink(format!("../../{target_disk}"), slaves_dir.join(link))
                    .expect("fixture slave symlink");
            }
        }

        fn mountinfo(
            &self,
            major: u32,
            minor: u32,
            mountpoint: &str,
            fstype: &str,
            source: &str,
        ) -> String {
            format!("100 99 {major}:{minor} / {mountpoint} rw,relatime - {fstype} {source} rw\n")
        }

        fn classify(&self, mountinfo: &str, path: &str, major: u32, minor: u32) -> LinuxLocality {
            classify_with(mountinfo, &self.sysfs, Path::new(path), dev(major, minor))
        }
    }

    impl Drop for FakeSysfs {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn dev(major: u32, minor: u32) -> u64 {
        ((major as u64) << DEV_T_MINOR_BITS) | (minor as u64)
    }

    fn assert_unproven(locality: &LinuxLocality, expected_reason: &str) {
        match locality {
            LinuxLocality::Unproven { reason } => assert_eq!(*reason, expected_reason),
            LinuxLocality::Proven { device, .. } => {
                panic!("expected Unproven({expected_reason}), got Proven({device:?})")
            }
        }
        assert!(!locality.is_fixed_local());
        assert_eq!(locality.as_str(), expected_reason);
    }

    /// Pins the disputed promotion rule AS IMPLEMENTED (BDFL-relayed Codex
    /// review, 2026-09-27) — not as an accepted claim. Guest-visible
    /// PCIe/NVMe or an allowlisted HBA alone does not establish host-local
    /// backing; implemented as drafted, pending specification resolution.
    fn assert_proven_as_implemented(
        locality: &LinuxLocality,
        major: u32,
        minor: u32,
    ) -> Vec<String> {
        match locality {
            LinuxLocality::Proven { device, evidence } => {
                assert_eq!(*device, (major, minor));
                assert!(
                    !evidence.is_empty(),
                    "Proven carries observational P1 evidence lines"
                );
                for line in evidence {
                    assert!(
                        !line.contains('\n'),
                        "evidence lines are single-line observations: {line}"
                    );
                }
                assert!(locality.is_fixed_local());
                assert_eq!(locality.reason(), "proved_local_v0");
                assert_eq!(locality.as_str(), "proved_local_v0");
                evidence.clone()
            }
            LinuxLocality::Unproven { reason } => {
                panic!("expected Proven (disputed, as-implemented), got Unproven({reason})")
            }
        }
    }

    #[test]
    fn hba_allowlist_pins_expected_drivers() {
        assert_eq!(hba_allowlist(), &["ahci", "ata_piix", "mpt2sas", "mpt3sas"]);
    }

    #[test]
    fn dev_t_decode_uses_minorbits_20() {
        // Per <linux/kdev_t.h>: MINORBITS=20, device = (major << 20) | minor.
        assert_eq!(decode_dev((8u64 << 20) | 1), (8, 1));
        assert_eq!(decode_dev((259u64 << 20) | 3), (259, 3));
        assert_eq!(decode_dev(0), (0, 0));
        assert_eq!(decode_dev(0xF_FFFF), (0, 0xF_FFFF));
    }

    // DISPUTED (BDFL-relayed Codex review, 2026-09-27): pins the promotion
    // as implemented, not as an accepted claim.
    #[test]
    fn disputed_promotion_nvme_pinned_as_implemented() {
        let fixture = FakeSysfs::new("nvme");
        fixture.add_block(
            259,
            1,
            "nvme0n1",
            "nvme0n1p1",
            Some("nvme"),
            true,
            "0\n",
            "0\n",
        );
        let mountinfo = fixture.mountinfo(259, 1, "/data", "ext4", "/dev/nvme0n1p1");
        let locality = fixture.classify(&mountinfo, "/data/x", 259, 1);
        let evidence = assert_proven_as_implemented(&locality, 259, 1);
        assert_eq!(evidence.len(), 5);
        assert!(evidence[0].contains("mountpoint=/data"));
        assert!(evidence[3].contains("driver=nvme"));
    }

    // DISPUTED (BDFL-relayed Codex review, 2026-09-27): pins the promotion
    // as implemented, not as an accepted claim.
    #[test]
    fn disputed_promotion_sata_ahci_pinned_as_implemented() {
        let fixture = FakeSysfs::new("sata");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        let evidence = assert_proven_as_implemented(&locality, 8, 1);
        assert!(evidence[4].contains("rotational=1"));
    }

    // DISPUTED (BDFL-relayed Codex review, 2026-09-27): pins the promotion
    // as implemented, not as an accepted claim. Non-removable MMC/SD
    // (driver mmcblk, removable=0) promotes; the removable gate below keeps
    // removable cards Unproven.
    #[test]
    fn disputed_promotion_mmc_pinned_as_implemented() {
        let fixture = FakeSysfs::new("mmc");
        fixture.add_block(
            179,
            1,
            "mmcblk0",
            "mmcblk0p1",
            Some("mmcblk"),
            true,
            "0\n",
            "0\n",
        );
        let mountinfo = fixture.mountinfo(179, 1, "/data", "ext4", "/dev/mmcblk0p1");
        let locality = fixture.classify(&mountinfo, "/data/x", 179, 1);
        let evidence = assert_proven_as_implemented(&locality, 179, 1);
        assert!(evidence[3].contains("driver=mmcblk"));
    }

    #[test]
    fn removable_mmc_backing_is_unrecognized() {
        // The same mmcblk driver with removable=1 stays Unproven: the
        // removability gate, not the driver, decides.
        let fixture = FakeSysfs::new("mmcremovable");
        fixture.add_block(
            179,
            2,
            "mmcblk1",
            "mmcblk1p1",
            Some("mmcblk"),
            true,
            "1\n",
            "0\n",
        );
        let mountinfo = fixture.mountinfo(179, 2, "/data", "ext4", "/dev/mmcblk1p1");
        let locality = fixture.classify(&mountinfo, "/data/x", 179, 2);
        assert_unproven(&locality, "p1_unrecognized_storage_stack_v0");
    }

    // DISPUTED (BDFL-relayed Codex review, 2026-09-27): pins the promotion
    // as implemented, not as an accepted claim. Rotational is recorded in
    // the evidence but never gates the outcome.
    #[test]
    fn disputed_promotion_rotational_recorded_but_never_gates() {
        for (tag, rotational) in [("rot0", "0\n"), ("rot1", "1\n")] {
            let fixture = FakeSysfs::new(tag);
            fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", rotational);
            let mountinfo = fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1");
            let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
            let evidence = assert_proven_as_implemented(&locality, 8, 1);
            let expected = format!("rotational={}", rotational.trim());
            assert!(
                evidence.iter().any(|line| line.contains(&expected)),
                "evidence records {expected}"
            );
        }
    }

    #[test]
    fn mountinfo_octal_escapes_decoded_before_match() {
        // The mount point carries an escaped space (\040); it must decode
        // before matching, so /mnt/my\040data matches path /mnt/my data/x.
        // tmpfs keeps the outcome Unproven: the test pins decoding, not the
        // disputed promotion.
        let fixture = FakeSysfs::new("escape");
        let mountinfo = "100 99 0:50 / /mnt/my\\040data rw,relatime - tmpfs tmpfs rw\n";
        let locality = fixture.classify(mountinfo, "/mnt/my data/x", 0, 50);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn mount_point_component_prefix_rejects_data2() {
        // Mount point /data must never string-prefix-match path /data2/x.
        let fixture = FakeSysfs::new("components");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1");
        let locality = fixture.classify(&mountinfo, "/data2/x", 8, 1);
        assert_unproven(&locality, "p1_no_mountinfo_entry_v0");
    }

    #[test]
    fn mount_stacking_selects_longest_component_prefix() {
        // /, /data, /data/sub all prefix /data/sub/x; the longest
        // (/data/sub, tmpfs) must win. Guest-invisible proves /data/sub was
        // selected: selecting /data would fail on the device check instead.
        let fixture = FakeSysfs::new("stacking");
        let mountinfo = [
            "10 9 8:0 / / rw,relatime - ext4 /dev/sda rw\n",
            "11 10 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
            "12 11 0:50 / /data/sub rw,relatime - tmpfs tmpfs rw\n",
        ]
        .concat();
        let locality = fixture.classify(&mountinfo, "/data/sub/x", 0, 50);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn duplicate_mount_points_fail_closed_ambiguous() {
        // Two entries, identical mount points, different numeric mount IDs
        // (100 vs 101): the selection is ambiguous and must fail closed —
        // never tiebreak by mount ID.
        let fixture = FakeSysfs::new("duplicate");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = [
            "100 99 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
            "101 99 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
        ]
        .concat();
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        assert_unproven(&locality, "p1_ambiguous_mount_topology_v0");
    }

    #[test]
    fn tmpfs_backing_is_guest_invisible() {
        let fixture = FakeSysfs::new("tmpfs");
        let mountinfo = fixture.mountinfo(0, 50, "/data", "tmpfs", "tmpfs");
        let locality = fixture.classify(&mountinfo, "/data/x", 0, 50);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn overlay_backing_is_guest_invisible() {
        let fixture = FakeSysfs::new("overlay");
        let mountinfo = fixture.mountinfo(0, 60, "/", "overlay", "overlay");
        let locality = fixture.classify(&mountinfo, "/x", 0, 60);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn virtio_blk_backing_is_guest_invisible() {
        let fixture = FakeSysfs::new("virtio");
        fixture.add_block(253, 0, "vda", "vda", Some("virtio_blk"), true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(253, 0, "/", "ext4", "/dev/vda");
        let locality = fixture.classify(&mountinfo, "/x", 253, 0);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn nvme_tcp_backing_is_guest_invisible() {
        // Driver-name arm: the block name (nvme1n1) is not in the
        // guest-invisible name set, so the nvme-tcp driver must decide.
        let fixture = FakeSysfs::new("nvmetcp");
        fixture.add_block(
            259,
            4,
            "nvme1n1",
            "nvme1n1",
            Some("nvme-tcp"),
            true,
            "0\n",
            "0\n",
        );
        let mountinfo = fixture.mountinfo(259, 4, "/data", "ext4", "/dev/nvme1n1");
        let locality = fixture.classify(&mountinfo, "/data/x", 259, 4);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn dm_device_with_local_slaves_stays_guest_invisible() {
        // No slave recursion: dm-0's slave looks local (ahci) but the stack
        // must stay guest-invisible.
        let fixture = FakeSysfs::new("dmslaves");
        fixture.add_block(252, 0, "dm-0", "dm-0", None, false, "0\n", "0\n");
        fixture.add_block(8, 0, "sda", "sda", Some("ahci"), true, "0\n", "1\n");
        fixture.add_slaves("dm-0", &[("sda", "sda")]);
        let mountinfo = fixture.mountinfo(252, 0, "/", "ext4", "/dev/dm-0");
        let locality = fixture.classify(&mountinfo, "/x", 252, 0);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn md_device_with_local_slaves_stays_guest_invisible() {
        let fixture = FakeSysfs::new("mdslaves");
        fixture.add_block(9, 0, "md0", "md0", None, false, "0\n", "0\n");
        fixture.add_block(8, 0, "sda", "sda", Some("ahci"), true, "0\n", "1\n");
        fixture.add_slaves("md0", &[("sda", "sda")]);
        let mountinfo = fixture.mountinfo(9, 0, "/", "ext4", "/dev/md0");
        let locality = fixture.classify(&mountinfo, "/x", 9, 0);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn loop_device_with_local_slaves_stays_guest_invisible() {
        let fixture = FakeSysfs::new("loopslaves");
        fixture.add_block(7, 0, "loop0", "loop0", None, false, "0\n", "0\n");
        fixture.add_block(8, 0, "sda", "sda", Some("ahci"), true, "0\n", "1\n");
        fixture.add_slaves("loop0", &[("sda", "sda")]);
        let mountinfo = fixture.mountinfo(7, 0, "/", "ext4", "/dev/loop0");
        let locality = fixture.classify(&mountinfo, "/x", 7, 0);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn missing_driver_symlink_is_guest_invisible() {
        // The device/ directory exists but the driver symlink is absent:
        // the guest cannot see the backing ("missing driver" bucket).
        let fixture = FakeSysfs::new("nodriver");
        fixture.add_block(8, 16, "sdb", "sdb", None, true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 16, "/data", "ext4", "/dev/sdb");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 16);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn usb_storage_backing_is_unrecognized() {
        let fixture = FakeSysfs::new("usb");
        fixture.add_block(
            8,
            32,
            "sdc",
            "sdc1",
            Some("usb-storage"),
            true,
            "0\n",
            "1\n",
        );
        let mountinfo = fixture.mountinfo(8, 32, "/media/usb", "vfat", "/dev/sdc1");
        let locality = fixture.classify(&mountinfo, "/media/usb/x", 8, 32);
        assert_unproven(&locality, "p1_unrecognized_storage_stack_v0");
    }

    #[test]
    fn non_allowlisted_driver_is_unrecognized() {
        let fixture = FakeSysfs::new("driver");
        fixture.add_block(
            8,
            48,
            "sdd",
            "sdd1",
            Some("megaraid_sas"),
            true,
            "0\n",
            "1\n",
        );
        let mountinfo = fixture.mountinfo(8, 48, "/data", "ext4", "/dev/sdd1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 48);
        assert_unproven(&locality, "p1_unrecognized_storage_stack_v0");
    }

    #[test]
    fn removable_backing_is_unrecognized() {
        let fixture = FakeSysfs::new("removable");
        fixture.add_block(8, 64, "sde", "sde1", Some("ahci"), true, "1\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 64, "/data", "ext4", "/dev/sde1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 64);
        assert_unproven(&locality, "p1_unrecognized_storage_stack_v0");
    }

    #[test]
    fn missing_mountinfo_entry_fails_closed() {
        // The only covering entry (/) describes a different device (8:0 vs
        // the path's 8:1): no mountinfo entry ties this path to its device.
        let fixture = FakeSysfs::new("noentry");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 0, "/", "ext4", "/dev/sda");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        assert_unproven(&locality, "p1_no_mountinfo_entry_v0");
    }

    #[test]
    fn stale_mountinfo_device_mismatch_fails_closed() {
        // The selected entry's device (8:2) disagrees with the path's
        // st_dev device (8:1): the mount view is stale, fail closed.
        let fixture = FakeSysfs::new("stale");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 2, "/data", "ext4", "/dev/sda2");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        assert_unproven(&locality, "p1_no_mountinfo_entry_v0");
    }

    #[test]
    fn missing_block_device_node_fails_closed() {
        // The mount entry matches but /sys has no dev/block/<major>:<minor>.
        let fixture = FakeSysfs::new("nonode");
        let mountinfo = fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        assert_unproven(&locality, "p1_block_device_unresolved_v0");
    }
}

#[cfg(all(test, not(unix)))]
mod non_unix_tests {
    use super::*;

    #[test]
    fn non_unix_classify_host_path_fails_closed() {
        // Documented: off Linux there is no mountinfo/sysfs pipeline, so the
        // classifier fails closed with the mountinfo-stage reason.
        let locality = classify_host_path(OsStr::new("/opaque"));
        assert_eq!(locality.reason(), "p1_no_mountinfo_entry_v0");
        assert!(!locality.is_fixed_local());
    }
}
