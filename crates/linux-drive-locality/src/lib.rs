//! Linux P1 (not network-backed) storage classification for WO29 Slice A.
//!
//! The classifier walks a fail-closed pipeline over the guest-visible
//! storage stack:
//!
//! `/proc/self/mountinfo` (octal escapes decoded before matching; strict
//! numeric mount and parent IDs; the unique topmost entry per mountpoint
//! wins — the entry not covered by a child mount at the same mountpoint;
//! hidden child mounts under a covered lower stack are excluded;
//! ambiguous or dangling topology fails closed) -> `/sys/dev/block/<major>:<minor>`
//! (symlink followed, or the node read as a directory) -> the block
//! device's `device/` directory (partition nodes expose no `device/`, so
//! one level of parent walk-up to the containing disk is used) -> the
//! upward driver chain (nearest first; a leading generic `sd` never masks a
//! farther allowlisted HBA) -> HBA allowlist / NVMe / MMC fact extraction ->
//! `removable == "0"`.
//!
//! `queue/rotational` is read for the evidence record only; it never gates.
//!
//! `dev_t` handling: the classifier decodes the userspace `dev_t` from
//! `MetadataExt::dev()` with the Linux userspace `<sys/sysmacros.h>`
//! (glibc) formulas — major `((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff)`,
//! minor `(dev & 0xff) | ((dev >> 12) & !0xff)` — never the kernel
//! `<linux/kdev_t.h>` `MINORBITS=20` packing. `0x801` decodes to `8:1`.
//!
//! Grant-first (decision 0029 §14, amendment 2026-09-27, accepted on
//! PR #58): no classifier in this Work Order version emits `proved`.
//! Guest-visible transport/HBA observations are recorded as observed facts
//! with `Unproven` and the insufficient-evidence reason; automatic proof is
//! deferred, not completed. The disputed predecessor promotion rule is
//! preserved only as quarantined evidence (see `quarantined_promotion`):
//! it is unreachable from the live admission path, and nothing here
//! presents guest-visible observations as proof of invisible host-local
//! backing.
//!
//! Stacked and paravirtual devices never reach `Proven`: the classifier
//! performs NO recursion into `slaves/` — a stacked device whose slaves
//! look local stays `Unproven`. `dm-*`, `md*`, `loop*`, `nbd*`, `rbd*`,
//! `drbd*` yield observed facts with `p1_insufficient_evidence_v0` (a
//! device name alone never justifies the known-network reason); `vd*`,
//! `xvd*`, and the `virtio_blk`/`nvme-tcp`/`storvsc`/`pvscsi` drivers yield
//! `p1_guest_invisible_backing_v0`.
//!
//! Filesystem-type classification: known-network filesystems (`nfs`,
//! `nfs4`, `cifs`, `smb3`, `smb`, `ncpfs`, `afp`, `ceph`) are
//! known-network; `fuse`-prefixed filesystems are unclassifiable
//! (insufficient evidence, never local); host-shared filesystems (`9p`,
//! `virtiofs`) are guest-invisible. Anonymous filesystems (`tmpfs`,
//! `overlay`) stay grant-only guest-invisible, as before.
//!
//! Every other outcome is fail-closed `Unproven` with a stable reason
//! string. Evidence lines record observations only.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// `Unproven` reason vocabulary. Each constant is unique and stable; the
/// emission site in `classify_with` names the constant it returns, and the
/// test suite asserts the exact strings verbatim (BDFL test-plan note,
/// 2026-09-27). Insufficient-evidence, guest-invisible-backing,
/// known-network, and missing-entry reasons are never normalized into one
/// another where the accepted spec distinguishes them.
const REASON_NO_MOUNTINFO_ENTRY: &str = "p1_no_mountinfo_entry_v0";
/// The selected mountinfo entry's device contradicts the path's `st_dev`
/// device: contradictory evidence, never relabeled as missing. The
/// external-trust consumer rejects it before any payload read; it is not
/// attestation-coverable.
const REASON_MOUNTINFO_STAT_CONTRADICTION: &str = "p1_mountinfo_stat_contradiction_v0";
const REASON_BLOCK_DEVICE_UNRESOLVED: &str = "p1_block_device_unresolved_v0";
const REASON_GUEST_INVISIBLE_BACKING: &str = "p1_guest_invisible_backing_v0";
const REASON_UNRECOGNIZED_STORAGE_STACK: &str = "p1_unrecognized_storage_stack_v0";
const REASON_AMBIGUOUS_MOUNT_TOPOLOGY: &str = "p1_ambiguous_mount_topology_v0";
/// Grant-first demotion reason: observed facts recorded, proof deferred.
/// Emitted by the demoted positive branches (NVMe PCIe transport,
/// allowlisted HBA driver, non-removable MMC/SD) and by the re-bucketed
/// stacked/network names (`dm-*`, `md*`, `loop*`, `nbd*`, `rbd*`, `drbd*`).
const REASON_INSUFFICIENT_EVIDENCE: &str = "p1_insufficient_evidence_v0";
/// Emitted only when extra observations establish a network fabric: an
/// NVMe `transport` attribute of `tcp`/`rdma`/`fc`, an iSCSI initiator
/// driver anywhere in the upward driver chain (CORRECTION 1,
/// BDFL-authorized 2026-09-27), or a known-network filesystem type
/// observed in mountinfo. Never emitted from a device name alone
/// (finding 4).
const REASON_KNOWN_NETWORK: &str = "p1_known_network_backing_v0";
/// Filesystem-level network evidence: the mountinfo filesystem type names
/// a network filesystem outright (`nfs`, `nfs4`, `cifs`, `smb3`, `smb`,
/// `ncpfs`, `afp`, `ceph`). Distinct from the block/fabric
/// `p1_known_network_backing_v0`: the observation is the filesystem type,
/// not the device transport.
const REASON_KNOWN_NETWORK_FILESYSTEM: &str = "p1_known_network_filesystem_v0";
/// `fuse`-prefixed filesystem types: the backing is whatever the
/// userspace filesystem server says it is, which the guest cannot
/// observe. Unclassifiable — insufficient evidence, never local.
const REASON_FUSE_FILESYSTEM: &str = "p1_fuse_filesystem_insufficient_evidence_v0";
/// Host-shared filesystem types (`9p`, `virtiofs`): the files are served
/// by the host, invisible to the guest's block layer by construction.
const REASON_HOST_SHARED_FILESYSTEM: &str = "p1_host_shared_filesystem_v0";
/// Removability-naming reason (CORRECTION 2, BDFL-authorized 2026-09-27):
/// emitted only when the `<disk>/removable` attribute genuinely reads `1`.
/// The accepted spec (WORKORDER_29.md) requires "the reason naming
/// removability" but names no literal; this literal is the builder's choice
/// in the existing `p1_*_v0` vocabulary, for review at implementation. It is
/// reserved strictly for a valid `removable=1` observation:
/// missing/unreadable keeps `p1_block_device_unresolved_v0`, and malformed
/// values fail closed with a non-observation reason, never this one.
const REASON_REMOVABLE_MEDIA: &str = "p1_removable_media_v0";
/// Emitted only by `LinuxLocality::Proven::reason()`. The `Proven` variant
/// is unreachable from the live admission path in this Work Order version
/// (grant-first); the only construction site is the quarantined predecessor.
const REASON_PROVED: &str = "proved_local_v0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinuxLocality {
    /// Defined but unreachable from the live admission path in this Work
    /// Order version: grant-first admits nothing as `proved`. The variant
    /// stays so the type still names the full verdict vocabulary the
    /// accepted spec uses, and so the quarantined predecessor preserves its
    /// exact shape as evidence.
    Proven {
        device: (u32, u32),
        evidence: Vec<String>,
    },
    /// `observed_facts` are single-line observation strings: what the guest
    /// saw (mountinfo selection, sysfs nodes, driver/transport identity).
    /// They record observations only — never a locality claim.
    ///
    /// `device` is the backing device identity `(major, minor)` the
    /// classifier observed for this path: the selected mountinfo entry's
    /// device. It is `Some` whenever mount/device selection recorded a
    /// device observation — including the contradictory one, when the
    /// entry's device disagrees with the path's `st_dev` (the consistency
    /// failure keeps the observed device so the consumer can reject the
    /// contradiction). It is `None` only when selection failed before
    /// observing any device: genuinely-missing evidence, which external
    /// trust may cover. The Slice A external-trust consumer binds an
    /// observed identity against the opened handle's device before any
    /// payload read: external trust covers missing observations, never
    /// contradictions.
    Unproven {
        reason: &'static str,
        observed_facts: Vec<String>,
        device: Option<(u32, u32)>,
    },
}

impl LinuxLocality {
    pub fn is_fixed_local(&self) -> bool {
        matches!(self, Self::Proven { .. })
    }

    pub fn reason(&self) -> &'static str {
        match self {
            Self::Proven { .. } => REASON_PROVED,
            Self::Unproven { reason, .. } => reason,
        }
    }

    pub fn as_str(&self) -> &'static str {
        self.reason()
    }

    /// The backing device identity `(major, minor)` the classifier
    /// observed for this path, when mount/device selection recorded one —
    /// including the contradictory observation on a mountinfo/stat
    /// mismatch, so the consumer can reject it. `None` on `Proven`
    /// (unreachable) and on `Unproven` results that failed before
    /// observing any device (genuinely missing).
    pub fn observed_device(&self) -> Option<(u32, u32)> {
        match self {
            Self::Proven { device, .. } => Some(*device),
            Self::Unproven { device, .. } => *device,
        }
    }

    /// Whether this verdict carries contradictory evidence: the selected
    /// mountinfo entry's device disagrees with the path's `st_dev`
    /// (`p1_mountinfo_stat_contradiction_v0`). Contradictions are never
    /// attestation-coverable: the Slice A consumers reject them
    /// independently of any opened-device comparison, before any payload
    /// read.
    pub fn is_contradiction(&self) -> bool {
        self.reason() == REASON_MOUNTINFO_STAT_CONTRADICTION
    }
}

/// Host-bus-adapter driver modules admitted to grant-first observed-fact
/// extraction. USB (`usb-storage`), virtual (`virtio_blk`), fabric
/// (`nvme-tcp`), Hyper-V (`storvsc`), VMware (`pvscsi`) and unknown drivers
/// are absent by design: they fail closed through the `Unproven` reasons
/// instead of yielding local-bus facts. Widening this list is a decision,
/// not an implementation detail.
pub fn hba_allowlist() -> &'static [&'static str] {
    &["ahci", "ata_piix", "mpt2sas", "mpt3sas"]
}

/// Decode a userspace `dev_t` into `(major, minor)` without libc, per the
/// Linux userspace `<sys/sysmacros.h>` (glibc) formulas — the encoding
/// `MetadataExt::dev()` actually returns. This is NOT the kernel
/// `<linux/kdev_t.h>` `MINORBITS=20` packing: `0x801` decodes to `8:1`
/// here, while the kernel packing would read it as `0:0x801`.
///
/// `pub` so the Slice A external-trust consumer can decode an opened
/// handle's `dev_t` with the same function the classifier used — one
/// decoder, no drift.
pub fn decode_dev(dev: u64) -> (u32, u32) {
    let major = (((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff)) as u32;
    let minor = ((dev & 0xff) | ((dev >> 12) & !0xff)) as u32;
    (major, minor)
}

struct MountEntry {
    mount_id: u64,
    parent_id: u64,
    mountpoint: PathBuf,
    fstype: String,
    source: String,
    major: u32,
    minor: u32,
}

/// Malformed mountinfo input: any line that does not carry strict numeric
/// mount/parent IDs and a numeric `major:minor` fails the whole parse —
/// malformed lines are never silently skipped.
fn parse_mountinfo(mountinfo: &str) -> Result<Vec<MountEntry>, &'static str> {
    mountinfo
        .lines()
        .map(parse_mountinfo_line)
        .collect::<Option<Vec<_>>>()
        .ok_or(REASON_NO_MOUNTINFO_ENTRY)
}

fn parse_mountinfo_line(line: &str) -> Option<MountEntry> {
    // mountinfo fields: id parent major:minor root mountpoint options...
    // "-" fstype source super-options. Whitespace inside fields is octal
    // escaped (\040), so splitting on whitespace before decoding is safe.
    // The mount and parent IDs are strict numerics: a malformed ID fails
    // the line, and a failed line fails the parse (never skipped).
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let separator = tokens.iter().position(|token| *token == "-")?;
    let pre = &tokens[..separator];
    let post = &tokens[separator + 1..];
    if pre.len() < 6 || post.len() < 2 {
        return None;
    }
    let mount_id: u64 = pre[0].parse().ok()?;
    let parent_id: u64 = pre[1].parse().ok()?;
    let (major, minor) = pre[2].split_once(':')?;
    Some(MountEntry {
        mount_id,
        parent_id,
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

/// Select the mountinfo entry for `path` using the mount topology.
///
/// Among entries whose mountpoint is a component-wise prefix of `path`,
/// each distinct mountpoint contributes its unique topmost entry: the one
/// that is not covered by another entry at the same mountpoint. An entry
/// is covered when a direct child (by parent ID) is mounted at the same
/// mountpoint — the child is stacked on top and hides it. Numeric mount-ID
/// ordering is never used as a tiebreak.
///
/// A duplicated mount ID means the mount table is internally inconsistent:
/// the index would silently collapse two entries into one, so selection
/// fails closed instead.
///
/// Reachability: an entry is hidden when its ancestry passes through a
/// covered mount that is *not* on the entry's own ancestry line. An
/// ancestor covered by the entry itself or by another ancestor of the
/// entry is the stacking case — a multi-level overmount stack, or a
/// visible descendant of an upper overmount — and does not hide it. A
/// self-parent entry is a namespace root (container setups): the walk
/// stops there, visible. A parent ID absent from the entry set is
/// tolerated — the parent lives outside this process's namespace root —
/// and the walk simply stops there.
///
/// Genuinely ambiguous topology fails closed with
/// `p1_ambiguous_mount_topology_v0`: more than one uncovered entry at the
/// same mountpoint (a dangling duplicate with no parent relationship), a
/// duplicated mount ID, a parent-ID cycle, or no reachable entry at all.
/// There is deliberately no tiebreak.
fn select_mount<'a>(
    entries: &'a [MountEntry],
    path: &Path,
) -> Result<&'a MountEntry, &'static str> {
    // Index by mount ID for the ancestry walk. A duplicated mount ID
    // fails closed: collapsing two entries into one would silently pick
    // a mount the table does not unambiguously name.
    let mut by_id: std::collections::HashMap<u64, &MountEntry> =
        std::collections::HashMap::with_capacity(entries.len());
    for entry in entries {
        if by_id.insert(entry.mount_id, entry).is_some() {
            return Err(REASON_AMBIGUOUS_MOUNT_TOPOLOGY);
        }
    }

    // Covered(entry) <=> some other entry is a direct child of it at the
    // same mountpoint (stacked on top, hiding it).
    let is_covered = |entry: &MountEntry| {
        entries.iter().any(|other| {
            other.mount_id != entry.mount_id
                && other.parent_id == entry.mount_id
                && other.mountpoint == entry.mountpoint
        })
    };

    // The mount covering `entry`, if any: its child at the same mountpoint.
    let coverer = |entry: &MountEntry| {
        entries.iter().find(|other| {
            other.mount_id != entry.mount_id
                && other.parent_id == entry.mount_id
                && other.mountpoint == entry.mountpoint
        })
    };

    // Reachable(entry): every strict ancestor (via parent IDs) is either
    // uncovered or covered by a mount on the entry's own ancestry line.
    // A covered ancestor whose coverer sits on the entry's own line is
    // the stacking case — the entry sits above the covering mount
    // (multi-level overmount stacks, visible descendants of an upper
    // overmount) — and does not hide the entry. A covered ancestor
    // whose coverer is off the entry's line hides the whole subtree
    // below it (the hidden-lower-child case).
    //
    // The walk is bounded: a parent-ID cycle that never reaches a
    // self-parent namespace root or an outside-namespace parent can never
    // resolve to a visible mount, so it fails closed rather than looping.
    let is_reachable = |entry: &MountEntry| {
        // The entry's ancestry line: the entry itself, then each parent
        // by ID, stopping at a self-parent namespace root, an
        // outside-namespace parent, or a repeated ID (cycle guard).
        let mut ancestry: Vec<u64> = Vec::with_capacity(entries.len().saturating_add(1));
        let mut cursor_id = entry.mount_id;
        for _ in 0..entries.len().saturating_add(1) {
            if ancestry.contains(&cursor_id) {
                break;
            }
            ancestry.push(cursor_id);
            let Some(current) = by_id.get(&cursor_id) else {
                break;
            };
            if current.parent_id == cursor_id {
                break;
            }
            cursor_id = current.parent_id;
        }
        // Every strict ancestor must be uncovered or covered from the
        // entry's own line; a foreign coverer hides the entry.
        for ancestor_id in ancestry.iter().skip(1) {
            let Some(ancestor) = by_id.get(ancestor_id) else {
                // Parent outside this namespace: tolerated, walk stops.
                return true;
            };
            if is_covered(ancestor)
                && !coverer(ancestor).is_some_and(|covering| ancestry.contains(&covering.mount_id))
            {
                return false;
            }
        }
        // The ancestry line must terminate at a self-parent namespace
        // root or an outside-namespace parent; stopping anywhere else
        // means a parent-ID cycle, which can never be visible.
        let terminus = ancestry.last().copied().unwrap_or(u64::MAX);
        match by_id.get(&terminus) {
            None => true,
            Some(current) => current.parent_id == current.mount_id,
        }
    };

    // Group prefix-matching entries by mountpoint; each mountpoint keeps
    // its unique uncovered (topmost) entry.
    let mut topmost_by_mountpoint: Vec<&MountEntry> = Vec::new();
    let mut seen_mountpoints: Vec<&Path> = Vec::new();
    for entry in entries {
        if !is_component_prefix(&entry.mountpoint, path) {
            continue;
        }
        if seen_mountpoints.contains(&entry.mountpoint.as_path()) {
            continue;
        }
        seen_mountpoints.push(entry.mountpoint.as_path());
        let at_mountpoint: Vec<&MountEntry> = entries
            .iter()
            .filter(|other| {
                other.mountpoint == entry.mountpoint && is_component_prefix(&other.mountpoint, path)
            })
            .collect();
        let uncovered: Vec<&MountEntry> = at_mountpoint
            .iter()
            .filter(|other| !is_covered(other))
            .copied()
            .collect();
        // Exactly one uncovered entry per mountpoint: zero means a
        // parent-ID cycle, more than one means a dangling duplicate with
        // no stacking relationship. Both fail closed.
        if uncovered.len() != 1 {
            return Err(REASON_AMBIGUOUS_MOUNT_TOPOLOGY);
        }
        topmost_by_mountpoint.push(uncovered[0]);
    }
    if topmost_by_mountpoint.is_empty() {
        return Err(REASON_NO_MOUNTINFO_ENTRY);
    }

    // Longest mountpoint first; the first reachable topmost wins. A
    // topmost entry hidden under a covered lower stack is skipped, never
    // selected.
    topmost_by_mountpoint
        .sort_by_key(|entry| std::cmp::Reverse(entry.mountpoint.components().count()));
    for entry in topmost_by_mountpoint {
        if is_reachable(entry) {
            return Ok(entry);
        }
    }
    Err(REASON_AMBIGUOUS_MOUNT_TOPOLOGY)
}

/// Filesystems with no block device behind them. Their backing is invisible
/// to the guest's block layer by construction, so there is no sysfs node to
/// prove — the honest bucket is guest-invisible, not a lookup failure.
fn is_guest_invisible_fstype(fstype: &str) -> bool {
    matches!(fstype, "tmpfs" | "overlay")
}

/// Known-network filesystem types: the mountinfo filesystem type names a
/// network filesystem outright. The backing traverses the network by
/// definition of the filesystem type — no block-layer observation needed.
fn is_known_network_fstype(fstype: &str) -> bool {
    matches!(
        fstype,
        "nfs" | "nfs4" | "cifs" | "smb3" | "smb" | "ncpfs" | "afp" | "ceph"
    )
}

/// `fuse`-prefixed filesystem types (including `fuseblk` and
/// `fuse.<subtype>`): the backing is whatever the userspace filesystem
/// server provides, which the guest cannot observe through the block
/// layer. Unclassifiable — insufficient evidence, never local.
fn is_fuse_fstype(fstype: &str) -> bool {
    fstype.starts_with("fuse")
}

/// Host-shared filesystem types: the files are served directly by the
/// host (`9p` virtio, `virtiofs`). The guest's block layer never sees the
/// backing — guest-invisible by construction.
fn is_host_shared_fstype(fstype: &str) -> bool {
    matches!(fstype, "9p" | "virtiofs")
}

/// Paravirtual block names whose backing is invisible to the guest by
/// construction: virtio (`vd*`) and Xen (`xvd*`). Checked on the resolved
/// block node itself; the classifier never recurses into `slaves/`.
fn is_guest_invisible_block_name(name: &str) -> bool {
    name.starts_with("vd") || name.starts_with("xvd")
}

/// Stacked or network block names whose device class the guest can observe
/// but whose backing it cannot resolve: device-mapper (`dm-*`), MD RAID
/// (`md*`), loop (`loop*`), and network block devices (`nbd*`, `rbd*`,
/// `drbd*`). Grant-first re-bucketing: these yield observed facts with
/// `REASON_INSUFFICIENT_EVIDENCE` — a device name alone never justifies the
/// known-network reason (finding 4: that needs extra observations such as a
/// network `transport` attribute). Checked on the resolved block node; no
/// recursion into `slaves/`, so a stacked device with local-looking slaves
/// stays `Unproven`.
fn is_unproven_stacked_block_name(name: &str) -> bool {
    name.starts_with("dm-")
        || name.starts_with("md")
        || name.starts_with("loop")
        || name.starts_with("nbd")
        || name.starts_with("rbd")
        || name.starts_with("drbd")
}

/// Device class label for the observed-fact line. The caller guarantees
/// `is_unproven_stacked_block_name(name)`.
fn stacked_device_class(name: &str) -> &'static str {
    if name.starts_with("dm-") {
        "dm"
    } else if name.starts_with("nbd") {
        "nbd"
    } else if name.starts_with("rbd") {
        "rbd"
    } else if name.starts_with("drbd") {
        "drbd"
    } else if name.starts_with("md") {
        "md"
    } else {
        "loop"
    }
}

/// Driver-level paravirtual or fabric frontends whose backing the guest
/// cannot see: virtio-blk, NVMe-oF (`nvme-tcp`), Hyper-V `storvsc`, VMware
/// `pvscsi`. Guest-invisible bucket per the accepted spec.
fn is_guest_invisible_driver(driver: &str) -> bool {
    matches!(driver, "virtio_blk" | "nvme-tcp" | "storvsc" | "pvscsi")
}

/// iSCSI initiator/transport driver modules whose presence anywhere in the
/// upward driver chain identifies iSCSI protocol evidence (CORRECTION 1,
/// BDFL-authorized 2026-09-27). Rationale: each of these is an in-kernel
/// iSCSI initiator — `iscsi_tcp` (the software initiator, the common case),
/// `ib_iser` (iSCSI over RDMA), and the hardware offload initiators
/// `qla4xxx` (QLogic), `bnx2i` (Broadcom), `be2iscsi` (Emulex), `cxgb3i` /
/// `cxgb4i` (Chelsio). Their names appear as `driver` symlink basenames in
/// the device chain of disks whose backing traverses the iSCSI network
/// protocol, so the accepted spec (WORKORDER_29.md sd* bullet) buckets them
/// known-network. This set is the builder's choice, reviewed at
/// implementation; widening it is a decision, not an implementation detail.
/// Matching is exact on the basename. `sd` — the SCSI disk upper driver
/// present on every SCSI-transport disk — is deliberately absent: the
/// upward walk looks past it to the ancestor evidence instead of treating
/// the upper driver as the transport.
fn is_iscsi_initiator_driver(driver: &str) -> bool {
    matches!(
        driver,
        "iscsi_tcp" | "ib_iser" | "qla4xxx" | "bnx2i" | "be2iscsi" | "cxgb3i" | "cxgb4i"
    )
}

/// Admission check for the driver name feeding grant-first observed-fact
/// extraction. The allowlist names HBA controller driver modules; plain
/// NVMe attaches via the in-kernel `nvme` host driver with no HBA in the
/// path, and non-removable MMC/SD cards attach via the in-kernel `mmcblk`
/// block driver. Admission records an observed fact and returns `Unproven` —
/// it establishes nothing about host-local backing. Removability is
/// enforced separately by the `removable == "0"` gate, so an MMC/SD device
/// that reports removable stays `Unproven` without reaching the fact
/// extraction below.
fn is_observed_fact_driver(driver: &str) -> bool {
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

/// Upward driver-chain inspection (CORRECTION 1, BDFL-authorized
/// 2026-09-27): from `device_dir` (`<disk>/device`) upward through parent
/// directories, read each level's `driver` symlink and collect the basename
/// of its target, nearest first. Levels whose `driver` entry is missing,
/// unreadable, or yields no basename are skipped — never fabricated — and
/// the walk continues upward; it stops at the sysfs root or the filesystem
/// root, whichever comes first. Returns the chain, possibly empty.
///
/// Mechanism note: `read_link` does not follow the symlink target, so a
/// dangling `driver` symlink still resolves to its basename; genuinely
/// unreadable means `read_link` itself fails (missing entry, non-symlink,
/// permission). An immediate `sd` upper driver therefore never hides
/// ancestor evidence: the walk looks past it.
fn read_driver_chain(device_dir: &Path, sysfs_root: &Path) -> Vec<String> {
    let mut chain = Vec::new();
    let mut cursor = device_dir.to_path_buf();
    loop {
        if let Some(name) = std::fs::read_link(cursor.join("driver"))
            .ok()
            .and_then(|target| {
                target
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
        {
            chain.push(name);
        }
        let Some(parent) = cursor.parent() else {
            break;
        };
        if parent == cursor || !parent.starts_with(sysfs_root) {
            break;
        }
        cursor = parent.to_path_buf();
    }
    chain
}

pub fn classify_with(
    mountinfo: &str,
    sysfs_root: &Path,
    path: &Path,
    st_dev: u64,
) -> LinuxLocality {
    // dev_t decode per the Linux userspace <sys/sysmacros.h> (glibc)
    // formulas — the encoding MetadataExt::dev() returns. Never the
    // kernel <linux/kdev_t.h> MINORBITS=20 packing.
    let (major, minor) = decode_dev(st_dev);

    let entries = match parse_mountinfo(mountinfo) {
        Ok(entries) => entries,
        Err(reason) => {
            // Malformed mountinfo (including a malformed mount ID) fails
            // the whole parse: no entry is silently skipped, and no
            // device was observed.
            return LinuxLocality::Unproven {
                reason,
                observed_facts: Vec::new(),
                device: None,
            };
        }
    };
    let entry = match select_mount(&entries, path) {
        Ok(entry) => entry,
        Err(reason) => {
            return LinuxLocality::Unproven {
                reason,
                observed_facts: Vec::new(),
                device: None,
            };
        }
    };
    // Consistency: the selected mount entry must describe the path's actual
    // device. A stale mountinfo view (entry device != st_dev device) is
    // CONTRADICTORY evidence, not missing evidence: the reason names the
    // contradiction, the observed facts record both sides, and the
    // mountinfo-observed device is kept so the external-trust consumer
    // binds it against the opened handle and rejects it before any
    // payload read. It is never relabeled as missing and never
    // attestation-coverable.
    if entry.major != major || entry.minor != minor {
        return LinuxLocality::Unproven {
            reason: REASON_MOUNTINFO_STAT_CONTRADICTION,
            observed_facts: vec![format!(
                "mountinfo: selected mountpoint={} dev={}:{} contradicts st_dev dev={}:{} (stale mountinfo view; contradictory evidence, rejected before read)",
                entry.mountpoint.display(),
                entry.major,
                entry.minor,
                major,
                minor,
            )],
            device: Some((entry.major, entry.minor)),
        };
    }
    // From here on the classifier has observed a usable backing device
    // identity: the selected entry's (major, minor), tied to the path's
    // st_dev by the consistency check above. Every Unproven below carries
    // it so the external-trust consumer can bind it against the opened
    // handle.
    let observed_device = Some((entry.major, entry.minor));

    // Filesystem-type classification, before any block-layer walk: the
    // mountinfo fstype alone determines these buckets.
    if is_known_network_fstype(&entry.fstype) {
        return LinuxLocality::Unproven {
            reason: REASON_KNOWN_NETWORK_FILESYSTEM,
            observed_facts: vec![format!(
                "mountinfo: fstype={} names a known-network filesystem for path={} (network by filesystem type; no block device observed)",
                entry.fstype,
                path.display(),
            )],
            device: observed_device,
        };
    }
    if is_fuse_fstype(&entry.fstype) {
        return LinuxLocality::Unproven {
            reason: REASON_FUSE_FILESYSTEM,
            observed_facts: vec![format!(
                "mountinfo: fstype={} is fuse-prefixed for path={} (userspace filesystem server; backing unobservable; insufficient evidence, never local)",
                entry.fstype,
                path.display(),
            )],
            device: observed_device,
        };
    }
    if is_host_shared_fstype(&entry.fstype) {
        return LinuxLocality::Unproven {
            reason: REASON_HOST_SHARED_FILESYSTEM,
            observed_facts: vec![format!(
                "mountinfo: fstype={} is host-shared for path={} (files served by the host; guest block layer sees no backing)",
                entry.fstype,
                path.display(),
            )],
            device: observed_device,
        };
    }
    if is_guest_invisible_fstype(&entry.fstype) {
        return LinuxLocality::Unproven {
            reason: REASON_GUEST_INVISIBLE_BACKING,
            observed_facts: Vec::new(),
            device: observed_device,
        };
    }

    let block_link = sysfs_root
        .join("dev/block")
        .join(format!("{major}:{minor}"));
    let Some(block_dir) = resolve_block_dir(&block_link) else {
        return LinuxLocality::Unproven {
            reason: REASON_BLOCK_DEVICE_UNRESOLVED,
            observed_facts: Vec::new(),
            device: observed_device,
        };
    };

    let block_name = block_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if is_guest_invisible_block_name(block_name) {
        return LinuxLocality::Unproven {
            reason: REASON_GUEST_INVISIBLE_BACKING,
            observed_facts: Vec::new(),
            device: observed_device,
        };
    }
    if is_unproven_stacked_block_name(block_name) {
        // Grant-first re-bucketing (decision 0029 §14): the device class is
        // observed, the backing is unresolved (no slave recursion), and the
        // name alone never justifies the known-network reason (finding 4).
        return LinuxLocality::Unproven {
            reason: REASON_INSUFFICIENT_EVIDENCE,
            observed_facts: vec![format!(
                "sysfs: observed fact {{device_class: {}}} (stacked/network device name observed; backing unresolved; no slave recursion)",
                stacked_device_class(block_name),
            )],
            device: observed_device,
        };
    }

    let Some((disk_dir, device_dir)) = disk_and_device_dirs(&block_dir) else {
        // The block node resolved but its sysfs identity (`device/`) is
        // absent: fail closed at the resolution stage.
        return LinuxLocality::Unproven {
            reason: REASON_BLOCK_DEVICE_UNRESOLVED,
            observed_facts: Vec::new(),
            device: observed_device,
        };
    };

    // Upward driver-chain inspection (CORRECTION 1, BDFL-authorized
    // 2026-09-27): the old code read only the immediate `<device>/driver`
    // symlink, so an `sd` upper driver hid ancestor evidence — notably an
    // iSCSI initiator. The chain is recorded as observed facts on every
    // path below where it was successfully read.
    let driver_chain = read_driver_chain(&device_dir, sysfs_root);
    if driver_chain.is_empty() {
        // No driver resolvable at any level: the guest cannot see the
        // backing. Fail closed with the existing guest-invisible reason; no
        // facts are fabricated for the unreadable chain.
        return LinuxLocality::Unproven {
            reason: REASON_GUEST_INVISIBLE_BACKING,
            observed_facts: Vec::new(),
            device: observed_device,
        };
    }
    let chain_fact = format!(
        "sysfs: driver chain (nearest first): {} (basenames of driver symlink targets walking up from {})",
        driver_chain.join(" -> "),
        device_dir.join("driver").display(),
    );
    // Nearest-first recognition, in the authorized priority order:
    // (a) an iSCSI initiator driver anywhere in the chain is known-network
    // per the accepted spec; (b) otherwise any guest-invisible driver in
    // the chain is guest-invisible; (c) otherwise the effective driver
    // gates the observed-fact extraction below; (d) otherwise the stack is
    // unrecognized. Every route stays Unproven: no automatic proof, no
    // admission widening.
    //
    // iSCSI keeps whole-chain priority: it is checked before
    // guest-invisible drivers, exactly as before.
    if driver_chain.iter().any(|d| is_iscsi_initiator_driver(d)) {
        return LinuxLocality::Unproven {
            reason: REASON_KNOWN_NETWORK,
            observed_facts: vec![
                chain_fact,
                "sysfs: iSCSI initiator driver in chain (iSCSI protocol evidence; known-network, never host-local backing)"
                    .to_string(),
            ],
            device: observed_device,
        };
    }
    if driver_chain.iter().any(|d| is_guest_invisible_driver(d)) {
        return LinuxLocality::Unproven {
            reason: REASON_GUEST_INVISIBLE_BACKING,
            observed_facts: vec![chain_fact],
            device: observed_device,
        };
    }
    // Effective driver: a nearer generic `sd` (the SCSI disk upper driver
    // present on every SCSI-transport disk) must not mask a farther
    // allowlisted HBA. The full chain is searched for the HBA, and the
    // recorded driver skips a leading `sd`; the complete chain stays in
    // the observed facts.
    let hba_driver = driver_chain
        .iter()
        .find(|driver| hba_allowlist().contains(&driver.as_str()));
    let driver: &str = driver_chain
        .iter()
        .find(|driver| driver.as_str() != "sd")
        .map(String::as_str)
        .unwrap_or(&driver_chain[0]);
    if !is_observed_fact_driver(driver) && hba_driver.is_none() {
        return LinuxLocality::Unproven {
            reason: REASON_UNRECOGNIZED_STORAGE_STACK,
            observed_facts: vec![chain_fact],
            device: observed_device,
        };
    }

    // Three-way removable gate (CORRECTION 2, BDFL-authorized 2026-09-27).
    // "0" proceeds to fact extraction; "1" is Unproven with the
    // removability-naming reason; missing/unreadable keeps the existing
    // fail-closed p1_block_device_unresolved_v0; any other (malformed)
    // value fails closed with a non-observation reason — never the
    // removability reason, which is reserved for a genuine removable=1
    // observation. The value is trimmed at read time, as before.
    let removable =
        std::fs::read_to_string(disk_dir.join("removable")).map(|text| text.trim().to_string());
    let Ok(removable) = removable else {
        return LinuxLocality::Unproven {
            reason: REASON_BLOCK_DEVICE_UNRESOLVED,
            observed_facts: vec![chain_fact],
            device: observed_device,
        };
    };
    match removable.as_str() {
        "0" => {}
        "1" => {
            return LinuxLocality::Unproven {
                reason: REASON_REMOVABLE_MEDIA,
                observed_facts: vec![chain_fact],
                device: observed_device,
            };
        }
        _ => {
            return LinuxLocality::Unproven {
                reason: REASON_UNRECOGNIZED_STORAGE_STACK,
                observed_facts: vec![chain_fact],
                device: observed_device,
            };
        }
    }

    // Informational only: read for the evidence record, never a gate.
    let rotational = std::fs::read_to_string(disk_dir.join("queue/rotational"))
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|_| "unread".to_string());

    // The driver chain was read successfully (non-empty, checked above);
    // record it first, then the existing mount/sysfs facts. The recorded
    // driver is the effective driver (a leading generic `sd` skipped); the
    // complete chain stays in the chain fact above.
    let mut observed_facts = vec![
        chain_fact,
        format!(
            "mountinfo: selected mountpoint={} fstype={} source={} dev={major}:{minor} for path={}",
            entry.mountpoint.display(),
            entry.fstype,
            entry.source,
            path.display(),
        ),
        format!(
            "mountinfo: st_dev={st_dev} decodes to {major}:{minor} via <sys/sysmacros.h> (glibc userspace dev_t); selected entry dev={}:{} matches",
            entry.major, entry.minor,
        ),
        format!(
            "sysfs: {} resolves to {}",
            block_link.display(),
            block_dir.display(),
        ),
        format!(
            "sysfs: disk={} driver={driver} (effective driver; basename of {}/driver symlink target, leading sd skipped)",
            disk_dir.display(),
            device_dir.display(),
        ),
        format!(
            "sysfs: removable=0 rotational={rotational} (rotational is informational only, not a gate)",
        ),
    ];

    // Branch selector for fact extraction: plain NVMe first, then the
    // allowlisted HBA found anywhere in the chain (a nearer `sd` or other
    // upper driver never masks it), then non-removable MMC/SD. The
    // admission check above guarantees one of these holds; anything else
    // fails closed.
    let fact_branch: &str = if driver == "nvme" {
        "nvme"
    } else if let Some(hba) = hba_driver {
        hba
    } else if driver == "mmcblk" {
        "mmcblk"
    } else {
        return LinuxLocality::Unproven {
            reason: REASON_UNRECOGNIZED_STORAGE_STACK,
            observed_facts,
            device: observed_device,
        };
    };

    // GRANT-FIRST (decision 0029 §14, amendment 2026-09-27): the branches
    // below extract observed facts and return `Unproven`. Guest-visible
    // transport/HBA observations are facts for the operator's trust
    // decision — never proof of invisible host-local backing. No branch in
    // the live admission path constructs `LinuxLocality::Proven`.
    match fact_branch {
        "nvme" => {
            // Per the accepted spec, for NVMe namespaces the disk's parent
            // device IS the controller (`device_add_disk(ctrl->device,
            // ...)`), so `<disk>/device/transport` is the controller's
            // transport attribute. NVMe multipath heads (disk parented to
            // the subsystem device) expose no `transport` attribute.
            match std::fs::read_to_string(device_dir.join("transport"))
                .map(|text| text.trim().to_string())
            {
                Ok(transport) if matches!(transport.as_str(), "tcp" | "rdma" | "fc") => {
                    // NVMe-oF presents the same `nvme*n*` device nodes over
                    // a network fabric: known-network, established by the
                    // transport attribute — never by the name alone.
                    observed_facts.push(format!(
                        "sysfs: observed fact {{transport: {transport}}} (NVMe-oF fabric transport observed; known-network)"
                    ));
                    return LinuxLocality::Unproven {
                        reason: REASON_KNOWN_NETWORK,
                        observed_facts,
                        device: observed_device,
                    };
                }
                Ok(transport) if transport == "pcie" => {
                    // An emulated PCI NVMe frontend with a network/file
                    // backend presents the same fact: insufficient evidence.
                    observed_facts.push(
                        "sysfs: observed fact {transport: pcie} (PCIe NVMe transport observed; host-local backing not established)"
                            .to_string(),
                    );
                }
                Ok(transport) => {
                    observed_facts.push(format!(
                        "sysfs: observed fact {{transport: {transport}}} (unrecognized NVMe transport value; insufficient evidence)"
                    ));
                }
                Err(_) => {
                    observed_facts.push(
                        "sysfs: transport attribute absent or unreadable (NVMe multipath head or non-PCIe frontend); insufficient evidence"
                            .to_string(),
                    );
                }
            }
        }
        driver if hba_allowlist().contains(&driver) => {
            // `driver` here is the allowlisted HBA found anywhere in the
            // chain (never masked by a nearer `sd`): record the found
            // name. The complete chain stays in the observed facts.
            observed_facts.push(format!(
                "sysfs: observed fact {{local_hba_driver: {driver}}} (allowlisted HBA driver observed in the guest; host-local backing not established)"
            ));
        }
        "mmcblk" => {
            // MMC/SD fact requires the observed `<disk>/device/type` to
            // read `MMC` or `SD`: the driver name alone does not establish
            // the media kind. Any other (or unreadable) value fails
            // closed with the unrecognized reason — the removable gate
            // above already admitted only removable=0.
            let device_type = std::fs::read_to_string(device_dir.join("type"))
                .map(|text| text.trim().to_string())
                .unwrap_or_default();
            if device_type == "MMC" || device_type == "SD" {
                observed_facts.push(
                    "sysfs: observed fact {mmc: non-removable media, removable=0} (MMC/SD fixed media observed; host-local backing not established)"
                        .to_string(),
                );
            } else {
                return LinuxLocality::Unproven {
                    reason: REASON_UNRECOGNIZED_STORAGE_STACK,
                    observed_facts,
                    device: observed_device,
                };
            }
        }
        _ => {
            // Unreachable: the branch selector admitted exactly the arms
            // above. Fail closed rather than panic if the admission set
            // ever drifts.
            return LinuxLocality::Unproven {
                reason: REASON_UNRECOGNIZED_STORAGE_STACK,
                observed_facts,
                device: observed_device,
            };
        }
    }

    LinuxLocality::Unproven {
        reason: REASON_INSUFFICIENT_EVIDENCE,
        observed_facts,
        device: observed_device,
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
            observed_facts: Vec::new(),
            device: None,
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
        observed_facts: Vec::new(),
        device: None,
    }
}

/// QUARANTINED PREDECESSOR — DISPUTED, UNREACHABLE FROM THE LIVE ADMISSION PATH.
///
/// What this is: the positive promotion rule exactly as implemented before
/// the grant-first amendment (decision 0029 §14, accepted on PR #58): an
/// admitted driver (plain NVMe, `mmcblk`, or an allowlisted HBA) promoted
/// the verdict to `Proven` once `removable == "0"` held.
///
/// Why it is quarantined: the BDFL-relayed Codex review (2026-09-27)
/// disputed the rule — guest-visible transport/HBA observations alone do
/// not establish host-local backing (a guest can observe virtual PCIe/NVMe
/// devices and emulated HBAs). The grant-first amendment demoted every
/// positive branch to observed-fact extraction + `Unproven`; automatic proof
/// is deferred, not completed.
///
/// Preservation contract: this module keeps the predecessor's exact shape
/// (its admission check and its `Proven` construction) as review evidence.
/// It is deliberately never called by the live admission path: the
/// `no_proved_emission_across_fixture_matrix` test pins that no live path
/// emits `Proven`, and the quarantine structural test pins that no new
/// caller is added without review. Nothing in this module — comments
/// included — presents guest-visible observations as proof of invisible
/// host-local backing: the `Proven` value it builds is the disputed
/// artifact under review, not a claim.
mod quarantined_promotion {
    use super::{LinuxLocality, REASON_UNRECOGNIZED_STORAGE_STACK, hba_allowlist};

    /// Frozen copy of the predecessor's driver admission, as implemented
    /// before the amendment. Not the live admission check.
    fn predecessor_admission(driver: &str) -> bool {
        driver == "nvme" || driver == "mmcblk" || hba_allowlist().contains(&driver)
    }

    /// The predecessor promotion rule, preserved as evidence. Disputed and
    /// unreachable from the live admission path: never called outside the
    /// quarantine evidence tests.
    #[allow(dead_code)]
    pub fn disputed_predecessor_promotion(
        driver: &str,
        device: (u32, u32),
        evidence: Vec<String>,
    ) -> LinuxLocality {
        if !predecessor_admission(driver) {
            return LinuxLocality::Unproven {
                reason: REASON_UNRECOGNIZED_STORAGE_STACK,
                observed_facts: Vec::new(),
                device: None,
            };
        }
        LinuxLocality::Proven { device, evidence }
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

        /// Writes `<disk>/device/transport` (the NVMe controller transport
        /// attribute; absent for multipath heads). Requires `device_dir =
        /// true` in `add_block`.
        fn set_transport(&self, disk: &str, transport: &str) {
            std::fs::write(
                self.sysfs
                    .join("devices/fake/block")
                    .join(disk)
                    .join("device/transport"),
                transport,
            )
            .expect("fixture transport");
        }

        /// Adds a `driver` symlink at an ancestor level of the device chain:
        /// `levels_up = 1` writes `<disk>/driver` (the parent of
        /// `<disk>/device/`), `levels_up = 2` writes
        /// `<devices/fake/block>/driver`, and so on. Models the upward
        /// device chain the production walk (`read_driver_chain`) inspects:
        /// the immediate `device/driver` symlink comes from `add_block`,
        /// and each ancestor level contributes its own driver basename,
        /// nearest first.
        fn set_chain_driver(&self, disk: &str, levels_up: u32, driver: &str) {
            let mut dir = self
                .sysfs
                .join("devices/fake/block")
                .join(disk)
                .join("device");
            for _ in 0..levels_up {
                dir = dir.parent().expect("fixture chain parent").to_path_buf();
            }
            std::os::unix::fs::symlink(
                format!("../../../../bus/fake/drivers/{driver}"),
                dir.join("driver"),
            )
            .expect("fixture chain driver symlink");
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

        /// Writes `<disk>/device/type` (the MMC/SD media type attribute:
        /// `MMC` or `SD`). Requires `device_dir = true` in `add_block`.
        fn set_device_type(&self, disk: &str, device_type: &str) {
            std::fs::write(
                self.sysfs
                    .join("devices/fake/block")
                    .join(disk)
                    .join("device/type"),
                device_type,
            )
            .expect("fixture device type");
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
        // Fixture oracle: the glibc <sys/sysmacros.h> makedev encoding,
        // matching MetadataExt::dev() on Linux and decode_dev above.
        let major = major as u64;
        let minor = minor as u64;
        ((major & 0xfff) << 8) | ((major & !0xfff) << 32) | (minor & 0xff) | ((minor & !0xff) << 12)
    }

    fn assert_unproven(locality: &LinuxLocality, expected_reason: &str) {
        match locality {
            LinuxLocality::Unproven { reason, .. } => assert_eq!(*reason, expected_reason),
            LinuxLocality::Proven { device, .. } => {
                panic!("expected Unproven({expected_reason}), got Proven({device:?})")
            }
        }
        assert!(!locality.is_fixed_local());
        assert_eq!(locality.as_str(), expected_reason);
    }

    /// Pins the grant-first demotion (decision 0029 §14): `Unproven` with
    /// the exact reason — never `Proven` — and every expected fact
    /// substring present in the observed facts. Reason strings are asserted
    /// verbatim (BDFL test-plan note, 2026-09-27).
    fn assert_demoted<'a>(
        locality: &'a LinuxLocality,
        expected_reason: &str,
        expected_fact_substrings: &[&str],
    ) -> &'a Vec<String> {
        match locality {
            LinuxLocality::Unproven {
                reason,
                observed_facts,
                device,
            } => {
                assert_eq!(*reason, expected_reason);
                for substring in expected_fact_substrings {
                    assert!(
                        observed_facts.iter().any(|fact| fact.contains(substring)),
                        "observed facts must contain {substring:?}; got {observed_facts:?}"
                    );
                }
                for fact in observed_facts {
                    assert!(
                        !fact.contains('\n'),
                        "observed facts are single-line observations: {fact}"
                    );
                }
                assert!(!locality.is_fixed_local());
                assert_eq!(locality.as_str(), expected_reason);
                // The demoted branches observed a usable backing device.
                assert!(
                    device.is_some(),
                    "demoted branch must carry the observed device; got {locality:?}"
                );
                observed_facts
            }
            LinuxLocality::Proven { device, .. } => {
                panic!("expected Unproven({expected_reason}), got Proven({device:?})")
            }
        }
    }

    #[test]
    fn hba_allowlist_pins_expected_drivers() {
        assert_eq!(hba_allowlist(), &["ahci", "ata_piix", "mpt2sas", "mpt3sas"]);
    }

    #[test]
    fn dev_t_decode_uses_sysmacros_not_minorbits_20() {
        // Linux userspace <sys/sysmacros.h> (glibc), NOT the kernel
        // <linux/kdev_t.h> MINORBITS=20 packing: 0x801 decodes to 8:1.
        // The fixture oracle dev() must round-trip through decode_dev.
        assert_eq!(decode_dev(0x801), (8, 1));
        assert_eq!(decode_dev(dev(8, 1)), (8, 1));
        assert_eq!(decode_dev(dev(259, 3)), (259, 3));
        assert_eq!(decode_dev(dev(0, 0)), (0, 0));
        assert_eq!(decode_dev(dev(0, 0xF_FFFF)), (0, 0xF_FFFF));
        // A large major exercises the high bits ((dev >> 32) & !0xfff).
        assert_eq!(decode_dev(dev(4096, 7)), (4096, 7));
    }

    // GRANT-FIRST (decision 0029 §14): the disputed promotion is demoted.
    // NVMe with transport=pcie is the observed fact {transport: pcie} with
    // Unproven + the insufficient-evidence reason — an emulated PCI NVMe
    // frontend with a network/file backend presents the same fact.
    #[test]
    fn grant_first_nvme_pcie_is_observed_fact_not_proof() {
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
        fixture.set_transport("nvme0n1", "pcie\n");
        let mountinfo = fixture.mountinfo(259, 1, "/data", "ext4", "/dev/nvme0n1p1");
        let locality = fixture.classify(&mountinfo, "/data/x", 259, 1);
        let facts = assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &[
                "{transport: pcie}",
                "mountpoint=/data",
                "driver=nvme",
                "driver chain (nearest first): nvme",
            ],
        );
        assert_eq!(facts.len(), 7);
    }

    // GRANT-FIRST (decision 0029 §14): the disputed promotion is demoted.
    // An allowlisted HBA driver in the chain is the observed fact
    // {local_hba_driver: <name>} with Unproven + the insufficient-evidence
    // reason — never a promotion.
    #[test]
    fn grant_first_sata_ahci_is_observed_fact_not_proof() {
        let fixture = FakeSysfs::new("sata");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        let facts = assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &[
                "{local_hba_driver: ahci}",
                "rotational=1",
                "driver chain (nearest first): ahci",
            ],
        );
        assert_eq!(facts.len(), 7);
    }

    // GRANT-FIRST (decision 0029 §14): the disputed promotion is demoted.
    // Non-removable MMC/SD (driver mmcblk, removable=0, device/type=MMC)
    // is an observed fact with Unproven + the insufficient-evidence
    // reason; the removable gate below keeps removable cards Unproven
    // without reaching fact extraction.
    #[test]
    fn grant_first_mmc_nonremovable_is_observed_fact_not_proof() {
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
        fixture.set_device_type("mmcblk0", "MMC\n");
        let mountinfo = fixture.mountinfo(179, 1, "/data", "ext4", "/dev/mmcblk0p1");
        let locality = fixture.classify(&mountinfo, "/data/x", 179, 1);
        let facts = assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &[
                "{mmc: non-removable media, removable=0}",
                "driver=mmcblk",
                "driver chain (nearest first): mmcblk",
            ],
        );
        assert_eq!(facts.len(), 7);
    }

    #[test]
    fn removable_mmc_backing_names_removability() {
        // CORRECTION 2 (BDFL-authorized 2026-09-27): the same mmcblk driver
        // with removable=1 stays Unproven, but the reason now names
        // removability (p1_removable_media_v0, the builder's literal choice)
        // instead of the generic unrecognized-storage-stack reason. The
        // removability gate, not the driver, decides; admission is unchanged
        // (Unproven either way).
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
        assert_unproven(&locality, "p1_removable_media_v0");
    }

    #[test]
    fn removable_malformed_value_fails_closed_without_removability_reason() {
        // CORRECTION 2: a malformed `removable` value (anything other than
        // "0"/"1" after trimming) is not a valid removable observation, so
        // it must not receive the removability-naming reason. It fails
        // closed with the non-observation unrecognized-storage-stack
        // reason — fail closed, never a fabricated observation.
        let fixture = FakeSysfs::new("mmcmalformed");
        fixture.add_block(
            179,
            3,
            "mmcblk2",
            "mmcblk2p1",
            Some("mmcblk"),
            true,
            "2\n",
            "0\n",
        );
        let mountinfo = fixture.mountinfo(179, 3, "/data", "ext4", "/dev/mmcblk2p1");
        let locality = fixture.classify(&mountinfo, "/data/x", 179, 3);
        assert_unproven(&locality, "p1_unrecognized_storage_stack_v0");
        assert_ne!(locality.reason(), "p1_removable_media_v0");
    }

    // CORRECTION 1 (BDFL-authorized 2026-09-27): upward driver-chain
    // inspection. The immediate `sd` upper driver (present on every
    // SCSI-transport disk) must not hide ancestor evidence: with `sd` at the
    // device level and `iscsi_tcp` one level up, the verdict is
    // known-network with the collected chain in the observed facts.
    // Behavior distinction: under the previous immediate-driver-only code
    // this same fixture yields p1_unrecognized_storage_stack_v0 (`sd` was
    // unadmitted); the asserted p1_known_network_backing_v0 literal is
    // unreachable under the old code here.
    #[test]
    fn iscsi_ancestor_recognized_as_known_network_not_hidden_by_sd() {
        let fixture = FakeSysfs::new("iscsichain");
        fixture.add_block(8, 112, "sdh", "sdh1", Some("sd"), true, "0\n", "1\n");
        fixture.set_chain_driver("sdh", 1, "iscsi_tcp");
        let mountinfo = fixture.mountinfo(8, 112, "/data", "ext4", "/dev/sdh1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 112);
        let facts = assert_demoted(
            &locality,
            "p1_known_network_backing_v0",
            &[
                "driver chain (nearest first): sd -> iscsi_tcp",
                "iSCSI initiator driver in chain",
            ],
        );
        assert_eq!(facts.len(), 2);
    }

    // CLARIFICATION (BDFL, 2026-09-27): "nearest-first" describes traversal
    // order only — it must not mean "first recognized driver wins". A
    // nearer allowlisted HBA observation must not mask recognized iSCSI
    // evidence farther along the chain: iSCSI recognition scans the entire
    // collected chain, and the full chain is retained in the observed
    // facts. Behavior distinction: a nearest-wins reading would take the
    // HBA observed-fact path (p1_insufficient_evidence_v0 with a
    // {local_hba_driver: ahci} fact) here; the asserted
    // p1_known_network_backing_v0 literal is unreachable under that
    // reading.
    #[test]
    fn nearer_hba_does_not_mask_farther_iscsi_evidence() {
        let fixture = FakeSysfs::new("hbaandiscsi");
        fixture.add_block(8, 124, "sdk", "sdk1", Some("ahci"), true, "0\n", "1\n");
        fixture.set_chain_driver("sdk", 1, "iscsi_tcp");
        let mountinfo = fixture.mountinfo(8, 124, "/data", "ext4", "/dev/sdk1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 124);
        let facts = assert_demoted(
            &locality,
            "p1_known_network_backing_v0",
            &[
                "driver chain (nearest first): ahci -> iscsi_tcp",
                "iSCSI initiator driver in chain",
            ],
        );
        assert_eq!(facts.len(), 2);
        assert!(
            !facts.iter().any(|fact| fact.contains("local_hba_driver")),
            "the nearer HBA observation must not mask the farther iSCSI evidence: {facts:?}"
        );
    }

    // CORRECTION 1: no driver symlink at any level of the device chain
    // fails closed with the existing guest-invisible reason, and no facts
    // are fabricated for the unreadable chain.
    #[test]
    fn missing_driver_chain_fails_closed_guest_invisible() {
        let fixture = FakeSysfs::new("nochain");
        fixture.add_block(8, 116, "sdi", "sdi1", None, true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 116, "/data", "ext4", "/dev/sdi1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 116);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
        let LinuxLocality::Unproven { observed_facts, .. } = locality else {
            panic!("assert_unproven already established Unproven");
        };
        assert!(
            observed_facts.is_empty(),
            "no facts may be fabricated for an unreadable chain: {observed_facts:?}"
        );
    }

    // CORRECTION 1: a `driver` entry that `read_link` cannot resolve (here a
    // regular file, not a symlink) is skipped — not fabricated — and does
    // not stop the upward walk: the ancestor's `sd` is still collected and
    // the verdict follows the normal rules (unrecognized, Unproven).
    // Documented mechanism note: `read_link` does not follow its target, so
    // a merely dangling symlink still resolves to its basename; genuinely
    // unreadable means `read_link` itself fails (missing entry, non-symlink,
    // permission).
    #[test]
    fn unreadable_mid_chain_driver_is_skipped_not_fabricated() {
        let fixture = FakeSysfs::new("badchain");
        fixture.add_block(8, 120, "sdj", "sdj1", None, true, "0\n", "1\n");
        std::fs::write(
            fixture.sysfs.join("devices/fake/block/sdj/device/driver"),
            "not-a-symlink",
        )
        .expect("fixture non-symlink driver");
        fixture.set_chain_driver("sdj", 1, "sd");
        let mountinfo = fixture.mountinfo(8, 120, "/data", "ext4", "/dev/sdj1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 120);
        let facts = assert_demoted(
            &locality,
            "p1_unrecognized_storage_stack_v0",
            &["driver chain (nearest first): sd"],
        );
        // Exactly the chain fact: nothing fabricated for the unreadable
        // immediate level.
        assert_eq!(facts.len(), 1);
    }

    // GRANT-FIRST (decision 0029 §14): the disputed promotion is demoted.
    // Rotational is recorded in the observed facts but never gates the
    // outcome — the verdict is Unproven either way.
    #[test]
    fn grant_first_rotational_recorded_but_never_gates() {
        for (tag, rotational) in [("rot0", "0\n"), ("rot1", "1\n")] {
            let fixture = FakeSysfs::new(tag);
            fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", rotational);
            let mountinfo = fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1");
            let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
            let expected = format!("rotational={}", rotational.trim());
            assert_demoted(
                &locality,
                "p1_insufficient_evidence_v0",
                &[&expected, "{local_hba_driver: ahci}"],
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
    fn stacked_mount_top_is_selected_not_ambiguous() {
        // A valid stacked mount: entry 101 (parent 100) is stacked on top
        // of entry 100 at the same mountpoint /data. The topmost entry
        // (101, dev 8:2) is the unique visible mount — not ambiguous.
        // Numeric mount-ID ordering is never consulted.
        let fixture = FakeSysfs::new("stackedtop");
        fixture.add_block(8, 2, "sdb", "sdb1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = [
            "100 99 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
            "101 100 8:2 / /data rw,relatime - ext4 /dev/sdb1 rw\n",
        ]
        .concat();
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 2);
        let facts = assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &["mountpoint=/data", "dev=8:2", "{local_hba_driver: ahci}"],
        );
        assert!(facts.iter().any(|fact| fact.contains("dev=8:2")));
    }

    #[test]
    fn hidden_child_mount_is_not_selected() {
        // Entry 102 at /data/sub is a child of entry 100, but entry 100
        // is covered by entry 101 (stacked on top at /data). Entry 102's
        // ancestry passes through the covered mount 100, so 102 is hidden
        // and must not be selected. The visible mount is 101 at /data.
        let fixture = FakeSysfs::new("hiddenchild");
        fixture.add_block(8, 2, "sdb", "sdb1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = [
            "100 99 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
            "101 100 8:2 / /data rw,relatime - ext4 /dev/sdb1 rw\n",
            "102 100 8:3 / /data/sub rw,relatime - ext4 /dev/sdc1 rw\n",
        ]
        .concat();
        // st_dev 8:2 matches the visible topmost entry 101, not the hidden
        // child 102 (8:3).
        let locality = fixture.classify(&mountinfo, "/data/sub/x", 8, 2);
        let facts = assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &["mountpoint=/data", "dev=8:2"],
        );
        assert!(
            !facts
                .iter()
                .any(|fact| fact.contains("mountpoint=/data/sub")),
            "hidden child mount must not be selected; facts: {facts:?}"
        );
    }

    #[test]
    fn multi_level_overmount_stack_selects_topmost() {
        // Three mounts stacked at /data: 100 (ext4 8:1) covered by 101
        // (xfs 8:2) covered by 102 (btrfs 8:3). Distinguishable devices
        // and filesystems per level prove which entry was selected: the
        // topmost (102, dev 8:3). The old reachability rule judged the
        // topmost unreachable (its grandparent was covered by its parent,
        // not by itself) and failed the whole stack as ambiguous.
        let fixture = FakeSysfs::new("multistack");
        fixture.add_block(8, 3, "sdc", "sdc1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = [
            "100 99 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
            "101 100 8:2 / /data rw,relatime - xfs /dev/sdb1 rw\n",
            "102 101 8:3 / /data rw,relatime - btrfs /dev/sdc1 rw\n",
        ]
        .concat();
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 3);
        let facts = assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &["mountpoint=/data", "dev=8:3"],
        );
        assert!(
            !facts.iter().any(|fact| fact.contains("dev=8:1"))
                && !facts.iter().any(|fact| fact.contains("dev=8:2")),
            "only the topmost stack level may be selected; facts: {facts:?}"
        );
    }

    #[test]
    fn visible_descendant_of_upper_overmount_is_selected() {
        // 100 (ext4 8:1) at /data is overmounted by 101 (xfs 8:2); 102
        // (ext4 8:3) at /data/sub is mounted on the UPPER mount. 102 is
        // visible: its covered grandparent 100 is covered by 101, which
        // sits on 102's own ancestry line. The old rule only exempted
        // covering-by-self and hid 102.
        let fixture = FakeSysfs::new("upperchild");
        fixture.add_block(8, 3, "sdc", "sdc1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = [
            "100 99 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
            "101 100 8:2 / /data rw,relatime - xfs /dev/sdb1 rw\n",
            "102 101 8:3 / /data/sub rw,relatime - ext4 /dev/sdc1 rw\n",
        ]
        .concat();
        let locality = fixture.classify(&mountinfo, "/data/sub/x", 8, 3);
        let facts = assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &["mountpoint=/data/sub", "dev=8:3"],
        );
        assert!(
            !facts.iter().any(|fact| fact.contains("dev=8:2")),
            "the upper overmount itself must not be selected for the sub-path; facts: {facts:?}"
        );
    }

    #[test]
    fn self_parent_namespace_root_is_valid() {
        // Container-style mountinfo: the root entry is its own parent
        // (mount ID 1, parent ID 1). A self-parent entry is a namespace
        // root, not a cycle: the walk stops there, visible. The old
        // bounded walk exhausted its iterations and failed every path as
        // ambiguous.
        let fixture = FakeSysfs::new("selfparent");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = [
            "1 1 8:0 / / rw,relatime - ext4 /dev/sda rw\n",
            "100 1 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
        ]
        .concat();
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &["mountpoint=/data", "dev=8:1"],
        );
    }

    #[test]
    fn duplicate_mount_ids_fail_closed() {
        // Two entries share mount ID 100 (at different mountpoints). The
        // table is internally inconsistent: indexing by mount ID would
        // silently collapse them and pick one. Selection fails closed as
        // ambiguous instead.
        let fixture = FakeSysfs::new("dupids");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = [
            "100 99 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
            "100 99 8:2 / /other rw,relatime - ext4 /dev/sdb1 rw\n",
        ]
        .concat();
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        assert_unproven(&locality, "p1_ambiguous_mount_topology_v0");
        assert_eq!(
            locality.observed_device(),
            None,
            "no device may be observed from an inconsistent table"
        );
    }

    #[test]
    fn malformed_mount_id_fails_parse_not_skipped() {
        // A malformed mount ID is a parse failure, not a skipped line: the
        // whole mountinfo is unusable and the classifier fails closed at
        // the mountinfo stage.
        let fixture = FakeSysfs::new("badmountid");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = "XXX 99 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n";
        let locality = fixture.classify(mountinfo, "/data/x", 8, 1);
        assert_unproven(&locality, "p1_no_mountinfo_entry_v0");
        assert_eq!(locality.observed_device(), None);
    }

    #[test]
    fn malformed_parent_id_fails_parse_not_skipped() {
        let fixture = FakeSysfs::new("badparentid");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = "100 YYY 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n";
        let locality = fixture.classify(mountinfo, "/data/x", 8, 1);
        assert_unproven(&locality, "p1_no_mountinfo_entry_v0");
        assert_eq!(locality.observed_device(), None);
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
    fn known_network_filesystems_are_known_network() {
        // The mountinfo fstype names a network filesystem outright: no
        // block-layer observation is needed, and the reason names the
        // filesystem (not the device transport).
        for (tag, fstype) in [
            ("nfs", "nfs"),
            ("nfs4", "nfs4"),
            ("cifs", "cifs"),
            ("smb3", "smb3"),
            ("smb", "smb"),
            ("ncpfs", "ncpfs"),
            ("afp", "afp"),
            ("ceph", "ceph"),
        ] {
            let fixture = FakeSysfs::new(&format!("netfs-{tag}"));
            let mountinfo = fixture.mountinfo(0, 70, "/data", fstype, "server:/share");
            let locality = fixture.classify(&mountinfo, "/data/x", 0, 70);
            assert_unproven(&locality, "p1_known_network_filesystem_v0");
            assert_eq!(locality.observed_device(), Some((0, 70)));
        }
    }

    #[test]
    fn fuse_filesystems_are_insufficient_evidence_never_local() {
        // fuse-prefixed filesystems: the backing is whatever the userspace
        // server provides — unclassifiable, never local.
        for (tag, fstype) in [
            ("fuse", "fuse"),
            ("fuseblk", "fuseblk"),
            ("fusesshfs", "fuse.sshfs"),
        ] {
            let fixture = FakeSysfs::new(&format!("fuse-{tag}"));
            let mountinfo = fixture.mountinfo(0, 71, "/data", fstype, "sshfs#server");
            let locality = fixture.classify(&mountinfo, "/data/x", 0, 71);
            assert_unproven(&locality, "p1_fuse_filesystem_insufficient_evidence_v0");
            assert_eq!(locality.observed_device(), Some((0, 71)));
        }
    }

    #[test]
    fn host_shared_filesystems_are_guest_invisible() {
        // 9p and virtiofs: files served by the host, invisible to the
        // guest's block layer by construction.
        for (tag, fstype) in [("9p", "9p"), ("virtiofs", "virtiofs")] {
            let fixture = FakeSysfs::new(&format!("hostshared-{tag}"));
            let mountinfo = fixture.mountinfo(0, 72, "/data", fstype, "hostshare");
            let locality = fixture.classify(&mountinfo, "/data/x", 0, 72);
            assert_unproven(&locality, "p1_host_shared_filesystem_v0");
            assert_eq!(locality.observed_device(), Some((0, 72)));
        }
    }

    #[test]
    fn nearer_sd_does_not_mask_farther_allowlisted_hba() {
        // Chain ["sd" (nearest), "ahci" (farther)]: the generic SCSI disk
        // upper driver must not mask the allowlisted HBA. The HBA is
        // found by searching the complete chain, and the recorded fact
        // names it.
        let fixture = FakeSysfs::new("sdhba");
        fixture.add_block(8, 1, "sda", "sda1", Some("sd"), true, "0\n", "1\n");
        fixture.set_chain_driver("sda", 1, "ahci");
        let mountinfo = fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        let facts = assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &[
                "{local_hba_driver: ahci}",
                "driver chain (nearest first): sd -> ahci",
            ],
        );
        // The full chain is preserved in the observed facts.
        assert!(facts[0].contains("sd -> ahci"));
    }

    #[test]
    fn mmc_without_observed_device_type_is_unrecognized() {
        // The mmcblk driver alone does not establish MMC/SD media: without
        // an observed <disk>/device/type of MMC or SD, the stack is
        // unrecognized. The removable gate already admitted removable=0.
        let fixture = FakeSysfs::new("mmcnotype");
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
        assert_unproven(&locality, "p1_unrecognized_storage_stack_v0");
        assert_eq!(locality.observed_device(), Some((179, 1)));
    }

    #[test]
    fn mmc_with_unexpected_device_type_is_unrecognized() {
        let fixture = FakeSysfs::new("mmcbadtype");
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
        fixture.set_device_type("mmcblk0", "USB\n");
        let mountinfo = fixture.mountinfo(179, 1, "/data", "ext4", "/dev/mmcblk0p1");
        let locality = fixture.classify(&mountinfo, "/data/x", 179, 1);
        assert_unproven(&locality, "p1_unrecognized_storage_stack_v0");
    }

    #[test]
    fn observed_device_is_carried_on_demoted_branches() {
        // The structured backing-device identity is usable by the
        // external-trust consumer: it matches the selected entry's device.
        let fixture = FakeSysfs::new("observeddev");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        assert_eq!(locality.observed_device(), Some((8, 1)));
    }

    #[test]
    fn observed_device_is_none_when_selection_fails() {
        // No usable identity was observed: selection failed before any
        // device could be tied to the path.
        let fixture = FakeSysfs::new("nodevice");
        let locality = fixture.classify("", "/data/x", 8, 1);
        assert_unproven(&locality, "p1_no_mountinfo_entry_v0");
        assert_eq!(locality.observed_device(), None);
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
    fn nvme_o_f_fabric_transport_is_known_network() {
        // The extra observation (a network `transport` attribute) is what
        // earns the known-network reason — this is the finding-4 contrast
        // case: the name alone never does this.
        for (tag, transport) in [("oFtcp", "tcp"), ("oFrdma", "rdma"), ("oFfc", "fc")] {
            let fixture = FakeSysfs::new(tag);
            fixture.add_block(
                259,
                6,
                "nvme2n1",
                "nvme2n1",
                Some("nvme"),
                true,
                "0\n",
                "0\n",
            );
            fixture.set_transport("nvme2n1", &format!("{transport}\n"));
            let mountinfo = fixture.mountinfo(259, 6, "/data", "ext4", "/dev/nvme2n1");
            let locality = fixture.classify(&mountinfo, "/data/x", 259, 6);
            assert_demoted(
                &locality,
                "p1_known_network_backing_v0",
                &[&format!("{{transport: {transport}}}"), "known-network"],
            );
        }
    }

    #[test]
    fn nvme_missing_transport_is_insufficient_evidence() {
        // NVMe multipath heads (disk parented to the subsystem device) expose
        // no `transport` attribute: fail closed with insufficient evidence,
        // never an optimistic admission.
        let fixture = FakeSysfs::new("multipath");
        fixture.add_block(
            259,
            7,
            "nvme3n1",
            "nvme3n1",
            Some("nvme"),
            true,
            "0\n",
            "0\n",
        );
        let mountinfo = fixture.mountinfo(259, 7, "/data", "ext4", "/dev/nvme3n1");
        let locality = fixture.classify(&mountinfo, "/data/x", 259, 7);
        assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &["transport attribute absent or unreadable"],
        );
    }

    #[test]
    fn vd_and_xvd_names_stay_guest_invisible() {
        // `vd*`/`xvd*` keep the guest-invisible bucket after re-bucketing.
        // The xvda fixture carries an innocuous HBA driver to prove the NAME
        // decides: the block-name check runs before driver resolution.
        let fixture = FakeSysfs::new("xvda");
        fixture.add_block(202, 0, "xvda", "xvda", Some("ahci"), true, "0\n", "0\n");
        let mountinfo = fixture.mountinfo(202, 0, "/", "ext4", "/dev/xvda");
        let locality = fixture.classify(&mountinfo, "/x", 202, 0);
        assert_unproven(&locality, "p1_guest_invisible_backing_v0");
    }

    #[test]
    fn hypervisor_drivers_stay_guest_invisible() {
        // Hyper-V storvsc and VMware PVSCSI are driver-level paravirtual
        // frontends: guest-invisible per the accepted spec.
        for (tag, major, minor, disk, driver) in [
            ("storvsc", 8u32, 80u32, "sdf", "storvsc"),
            ("pvscsi", 8u32, 96u32, "sdg", "pvscsi"),
        ] {
            let fixture = FakeSysfs::new(tag);
            fixture.add_block(major, minor, disk, disk, Some(driver), true, "0\n", "0\n");
            let mountinfo =
                fixture.mountinfo(major, minor, "/data", "ext4", &format!("/dev/{disk}"));
            let locality = fixture.classify(&mountinfo, "/data/x", major, minor);
            assert_unproven(&locality, "p1_guest_invisible_backing_v0");
        }
    }

    #[test]
    fn dm_device_with_local_slaves_is_insufficient_evidence() {
        // No slave recursion: dm-0's slave looks local (ahci) but the stack
        // must stay Unproven. Grant-first re-bucketing: the observed device
        // class is recorded as a fact with the insufficient-evidence reason.
        let fixture = FakeSysfs::new("dmslaves");
        fixture.add_block(252, 0, "dm-0", "dm-0", None, false, "0\n", "0\n");
        fixture.add_block(8, 0, "sda", "sda", Some("ahci"), true, "0\n", "1\n");
        fixture.add_slaves("dm-0", &[("sda", "sda")]);
        let mountinfo = fixture.mountinfo(252, 0, "/", "ext4", "/dev/dm-0");
        let locality = fixture.classify(&mountinfo, "/x", 252, 0);
        assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &["{device_class: dm}", "no slave recursion"],
        );
    }

    #[test]
    fn md_device_with_local_slaves_is_insufficient_evidence() {
        let fixture = FakeSysfs::new("mdslaves");
        fixture.add_block(9, 0, "md0", "md0", None, false, "0\n", "0\n");
        fixture.add_block(8, 0, "sda", "sda", Some("ahci"), true, "0\n", "1\n");
        fixture.add_slaves("md0", &[("sda", "sda")]);
        let mountinfo = fixture.mountinfo(9, 0, "/", "ext4", "/dev/md0");
        let locality = fixture.classify(&mountinfo, "/x", 9, 0);
        assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &["{device_class: md}", "no slave recursion"],
        );
    }

    #[test]
    fn loop_device_with_local_slaves_is_insufficient_evidence() {
        let fixture = FakeSysfs::new("loopslaves");
        fixture.add_block(7, 0, "loop0", "loop0", None, false, "0\n", "0\n");
        fixture.add_block(8, 0, "sda", "sda", Some("ahci"), true, "0\n", "1\n");
        fixture.add_slaves("loop0", &[("sda", "sda")]);
        let mountinfo = fixture.mountinfo(7, 0, "/", "ext4", "/dev/loop0");
        let locality = fixture.classify(&mountinfo, "/x", 7, 0);
        assert_demoted(
            &locality,
            "p1_insufficient_evidence_v0",
            &["{device_class: loop}", "no slave recursion"],
        );
    }

    // Finding 4 (binding, decision 0029 §14): an `nbd`/`rbd` name alone NEVER
    // yields the known-network reason; the stronger reason needs extra
    // observations (e.g. a network `transport` attribute). These fixtures
    // expose a `device/` directory with an innocuous driver to prove the
    // NAME decides — and it decides insufficient-evidence, not
    // known-network.
    #[test]
    fn nbd_and_rbd_names_alone_never_yield_known_network() {
        for (tag, major, disk, class) in [
            ("nbd4", 43u32, "nbd0", "nbd"),
            ("rbd4", 251u32, "rbd0", "rbd"),
            ("drbd4", 147u32, "drbd0", "drbd"),
        ] {
            let fixture = FakeSysfs::new(tag);
            fixture.add_block(major, 0, disk, disk, Some("ahci"), true, "0\n", "0\n");
            let mountinfo = fixture.mountinfo(major, 0, "/data", "ext4", &format!("/dev/{disk}"));
            let locality = fixture.classify(&mountinfo, "/data/x", major, 0);
            assert_demoted(
                &locality,
                "p1_insufficient_evidence_v0",
                &[&format!("{{device_class: {class}}}")],
            );
            assert_ne!(
                locality.reason(),
                "p1_known_network_backing_v0",
                "a device name alone must never yield the known-network reason"
            );
        }
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
    fn removable_backing_names_removability() {
        // CORRECTION 2 (BDFL-authorized 2026-09-27): removable=1 with an
        // admitted driver stays Unproven, but the reason now names
        // removability instead of the generic unrecognized-storage-stack
        // reason. Admission is unchanged (Unproven either way).
        let fixture = FakeSysfs::new("removable");
        fixture.add_block(8, 64, "sde", "sde1", Some("ahci"), true, "1\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 64, "/data", "ext4", "/dev/sde1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 64);
        assert_unproven(&locality, "p1_removable_media_v0");
    }

    #[test]
    fn mountinfo_stat_contradiction_at_root_is_contradiction() {
        // The only covering entry (/) describes a different device (8:0 vs
        // the path's 8:1). An entry WAS selected — it contradicts st_dev —
        // so this is contradictory evidence (rejected before read), never
        // "no mountinfo entry".
        let fixture = FakeSysfs::new("noentry");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 0, "/", "ext4", "/dev/sda");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        assert_unproven(&locality, "p1_mountinfo_stat_contradiction_v0");
        assert_eq!(locality.observed_device(), Some((8, 0)));
    }

    #[test]
    fn genuinely_missing_mountinfo_entry_stays_missing() {
        // No entry's mountpoint prefix-matches the path at all: genuinely
        // missing evidence. The reason stays `p1_no_mountinfo_entry_v0`
        // with no observed device — attestation-coverable, unlike the
        // contradiction above.
        let fixture = FakeSysfs::new("truenoentry");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1");
        let locality = fixture.classify(&mountinfo, "/other/x", 8, 1);
        assert_unproven(&locality, "p1_no_mountinfo_entry_v0");
        assert_eq!(locality.observed_device(), None);
    }

    #[test]
    fn stale_mountinfo_device_mismatch_is_contradiction_not_missing() {
        // The selected entry's device (8:2) disagrees with the path's
        // st_dev device (8:1): the mount view is stale. This is
        // contradictory evidence, never relabeled as missing: the reason
        // names the contradiction, the observed device is kept (so the
        // live consumer binds it against the opened handle and rejects
        // it before any payload read), and the facts record both sides.
        let fixture = FakeSysfs::new("stale");
        fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
        let mountinfo = fixture.mountinfo(8, 2, "/data", "ext4", "/dev/sda2");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        let facts = assert_demoted(
            &locality,
            "p1_mountinfo_stat_contradiction_v0",
            &["mountpoint=/data", "dev=8:2", "dev=8:1"],
        );
        assert_eq!(
            locality.observed_device(),
            Some((8, 2)),
            "the contradictory mountinfo observation stays observable: {facts:?}"
        );
        assert!(
            facts
                .iter()
                .any(|fact| fact.contains("contradictory evidence")),
            "facts must mark the observation contradictory, not missing: {facts:?}"
        );
    }

    #[test]
    fn missing_block_device_node_fails_closed() {
        // The mount entry matches but /sys has no dev/block/<major>:<minor>.
        let fixture = FakeSysfs::new("nonode");
        let mountinfo = fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1");
        let locality = fixture.classify(&mountinfo, "/data/x", 8, 1);
        assert_unproven(&locality, "p1_block_device_unresolved_v0");
    }

    /// Builds the fixture for one sweep case, returning
    /// (mountinfo, path, st_dev).
    fn build_sweep_case(fixture: &FakeSysfs, name: &str) -> (String, String, u64) {
        match name {
            "nvme_pcie" => {
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
                fixture.set_transport("nvme0n1", "pcie\n");
                (
                    fixture.mountinfo(259, 1, "/data", "ext4", "/dev/nvme0n1p1"),
                    "/data/x".to_string(),
                    dev(259, 1),
                )
            }
            "nvme_no_transport" => {
                fixture.add_block(
                    259,
                    5,
                    "nvme0n1",
                    "nvme0n1",
                    Some("nvme"),
                    true,
                    "0\n",
                    "0\n",
                );
                (
                    fixture.mountinfo(259, 5, "/data", "ext4", "/dev/nvme0n1"),
                    "/data/x".to_string(),
                    dev(259, 5),
                )
            }
            "nvme_tcp" => {
                fixture.add_block(
                    259,
                    4,
                    "nvme1n1",
                    "nvme1n1",
                    Some("nvme"),
                    true,
                    "0\n",
                    "0\n",
                );
                fixture.set_transport("nvme1n1", "tcp\n");
                (
                    fixture.mountinfo(259, 4, "/data", "ext4", "/dev/nvme1n1"),
                    "/data/x".to_string(),
                    dev(259, 4),
                )
            }
            "sata_ahci" => {
                fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
                (
                    fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1"),
                    "/data/x".to_string(),
                    dev(8, 1),
                )
            }
            "mmc_fixed" => {
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
                fixture.set_device_type("mmcblk0", "MMC\n");
                (
                    fixture.mountinfo(179, 1, "/data", "ext4", "/dev/mmcblk0p1"),
                    "/data/x".to_string(),
                    dev(179, 1),
                )
            }
            "mmc_removable" => {
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
                (
                    fixture.mountinfo(179, 2, "/data", "ext4", "/dev/mmcblk1p1"),
                    "/data/x".to_string(),
                    dev(179, 2),
                )
            }
            "dm" => {
                fixture.add_block(252, 0, "dm-0", "dm-0", None, false, "0\n", "0\n");
                fixture.add_block(8, 0, "sda", "sda", Some("ahci"), true, "0\n", "1\n");
                fixture.add_slaves("dm-0", &[("sda", "sda")]);
                (
                    fixture.mountinfo(252, 0, "/", "ext4", "/dev/dm-0"),
                    "/x".to_string(),
                    dev(252, 0),
                )
            }
            "md" => {
                fixture.add_block(9, 0, "md0", "md0", None, false, "0\n", "0\n");
                (
                    fixture.mountinfo(9, 0, "/", "ext4", "/dev/md0"),
                    "/x".to_string(),
                    dev(9, 0),
                )
            }
            "loop" => {
                fixture.add_block(7, 0, "loop0", "loop0", None, false, "0\n", "0\n");
                (
                    fixture.mountinfo(7, 0, "/", "ext4", "/dev/loop0"),
                    "/x".to_string(),
                    dev(7, 0),
                )
            }
            "nbd" => {
                fixture.add_block(43, 0, "nbd0", "nbd0", None, false, "0\n", "0\n");
                (
                    fixture.mountinfo(43, 0, "/data", "ext4", "/dev/nbd0"),
                    "/data/x".to_string(),
                    dev(43, 0),
                )
            }
            "rbd" => {
                fixture.add_block(251, 0, "rbd0", "rbd0", None, false, "0\n", "0\n");
                (
                    fixture.mountinfo(251, 0, "/data", "ext4", "/dev/rbd0"),
                    "/data/x".to_string(),
                    dev(251, 0),
                )
            }
            "drbd" => {
                fixture.add_block(147, 0, "drbd0", "drbd0", None, false, "0\n", "0\n");
                (
                    fixture.mountinfo(147, 0, "/data", "ext4", "/dev/drbd0"),
                    "/data/x".to_string(),
                    dev(147, 0),
                )
            }
            "vda" => {
                fixture.add_block(253, 0, "vda", "vda", Some("virtio_blk"), true, "0\n", "1\n");
                (
                    fixture.mountinfo(253, 0, "/", "ext4", "/dev/vda"),
                    "/x".to_string(),
                    dev(253, 0),
                )
            }
            "xvda" => {
                fixture.add_block(202, 0, "xvda", "xvda", Some("ahci"), true, "0\n", "0\n");
                (
                    fixture.mountinfo(202, 0, "/", "ext4", "/dev/xvda"),
                    "/x".to_string(),
                    dev(202, 0),
                )
            }
            "nvme_tcp_driver" => {
                fixture.add_block(
                    259,
                    9,
                    "nvme4n1",
                    "nvme4n1",
                    Some("nvme-tcp"),
                    true,
                    "0\n",
                    "0\n",
                );
                (
                    fixture.mountinfo(259, 9, "/data", "ext4", "/dev/nvme4n1"),
                    "/data/x".to_string(),
                    dev(259, 9),
                )
            }
            "storvsc" => {
                fixture.add_block(8, 80, "sdf", "sdf", Some("storvsc"), true, "0\n", "0\n");
                (
                    fixture.mountinfo(8, 80, "/data", "ext4", "/dev/sdf"),
                    "/data/x".to_string(),
                    dev(8, 80),
                )
            }
            "pvscsi" => {
                fixture.add_block(8, 96, "sdg", "sdg", Some("pvscsi"), true, "0\n", "0\n");
                (
                    fixture.mountinfo(8, 96, "/data", "ext4", "/dev/sdg"),
                    "/data/x".to_string(),
                    dev(8, 96),
                )
            }
            "usb" => {
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
                (
                    fixture.mountinfo(8, 32, "/media/usb", "vfat", "/dev/sdc1"),
                    "/media/usb/x".to_string(),
                    dev(8, 32),
                )
            }
            "megaraid" => {
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
                (
                    fixture.mountinfo(8, 48, "/data", "ext4", "/dev/sdd1"),
                    "/data/x".to_string(),
                    dev(8, 48),
                )
            }
            "removable_ahci" => {
                fixture.add_block(8, 64, "sde", "sde1", Some("ahci"), true, "1\n", "1\n");
                (
                    fixture.mountinfo(8, 64, "/data", "ext4", "/dev/sde1"),
                    "/data/x".to_string(),
                    dev(8, 64),
                )
            }
            "tmpfs" => (
                fixture.mountinfo(0, 50, "/data", "tmpfs", "tmpfs"),
                "/data/x".to_string(),
                dev(0, 50),
            ),
            "no_driver" => {
                fixture.add_block(8, 16, "sdb", "sdb", None, true, "0\n", "1\n");
                (
                    fixture.mountinfo(8, 16, "/data", "ext4", "/dev/sdb"),
                    "/data/x".to_string(),
                    dev(8, 16),
                )
            }
            "no_block_node" => (
                fixture.mountinfo(8, 1, "/data", "ext4", "/dev/sda1"),
                "/data/x".to_string(),
                dev(8, 1),
            ),
            "stale_mount" => {
                fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
                (
                    fixture.mountinfo(8, 2, "/data", "ext4", "/dev/sda2"),
                    "/data/x".to_string(),
                    dev(8, 1),
                )
            }
            "ambiguous" => {
                fixture.add_block(8, 1, "sda", "sda1", Some("ahci"), true, "0\n", "1\n");
                (
                    [
                        "100 99 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
                        "101 99 8:1 / /data rw,relatime - ext4 /dev/sda1 rw\n",
                    ]
                    .concat(),
                    "/data/x".to_string(),
                    dev(8, 1),
                )
            }
            other => panic!("unknown sweep case: {other}"),
        }
    }

    // Grant-first sweep (decision 0029 §14): across the full fixture matrix
    // — every classifier path — the live admission path must never emit
    // `Proven`. Each case also pins its exact `Unproven` reason: the reason
    // strings are unique, stable, and asserted verbatim (BDFL test-plan
    // note, 2026-09-27); insufficient-evidence, guest-invisible-backing,
    // known-network, and missing-entry reasons are never normalized into
    // one another.
    #[test]
    fn no_proved_emission_across_fixture_matrix() {
        let cases: &[(&str, &str)] = &[
            ("nvme_pcie", "p1_insufficient_evidence_v0"),
            ("nvme_no_transport", "p1_insufficient_evidence_v0"),
            ("nvme_tcp", "p1_known_network_backing_v0"),
            ("sata_ahci", "p1_insufficient_evidence_v0"),
            ("mmc_fixed", "p1_insufficient_evidence_v0"),
            ("mmc_removable", "p1_removable_media_v0"),
            ("dm", "p1_insufficient_evidence_v0"),
            ("md", "p1_insufficient_evidence_v0"),
            ("loop", "p1_insufficient_evidence_v0"),
            ("nbd", "p1_insufficient_evidence_v0"),
            ("rbd", "p1_insufficient_evidence_v0"),
            ("drbd", "p1_insufficient_evidence_v0"),
            ("vda", "p1_guest_invisible_backing_v0"),
            ("xvda", "p1_guest_invisible_backing_v0"),
            ("nvme_tcp_driver", "p1_guest_invisible_backing_v0"),
            ("storvsc", "p1_guest_invisible_backing_v0"),
            ("pvscsi", "p1_guest_invisible_backing_v0"),
            ("usb", "p1_unrecognized_storage_stack_v0"),
            ("megaraid", "p1_unrecognized_storage_stack_v0"),
            ("removable_ahci", "p1_removable_media_v0"),
            ("tmpfs", "p1_guest_invisible_backing_v0"),
            ("no_driver", "p1_guest_invisible_backing_v0"),
            ("no_block_node", "p1_block_device_unresolved_v0"),
            ("stale_mount", "p1_mountinfo_stat_contradiction_v0"),
            ("ambiguous", "p1_ambiguous_mount_topology_v0"),
        ];
        assert_eq!(
            cases.len(),
            25,
            "the sweep must cover the full fixture matrix"
        );
        for (name, expected_reason) in cases {
            let fixture = FakeSysfs::new(name);
            let (mountinfo, path, st_dev) = build_sweep_case(&fixture, name);
            let locality = classify_with(&mountinfo, &fixture.sysfs, Path::new(&path), st_dev);
            match &locality {
                LinuxLocality::Proven { device, .. } => {
                    panic!(
                        "case {name}: live path emitted Proven({device:?}) — grant-first violation"
                    )
                }
                LinuxLocality::Unproven { reason, .. } => {
                    assert_eq!(
                        *reason, *expected_reason,
                        "case {name}: wrong Unproven reason"
                    );
                }
            }
            assert!(!locality.is_fixed_local(), "case {name}: is_fixed_local");
            assert_eq!(locality.as_str(), *expected_reason, "case {name}: as_str");
        }
    }

    #[test]
    fn quarantined_predecessor_promotion_preserved_as_evidence() {
        // The quarantined predecessor keeps the exact shape of the disputed
        // rule as evidence: an admitted driver promoted to `Proven` with the
        // evidence lines, a non-admitted driver did not. It is NEVER called
        // by the live admission path — the fixture-matrix sweep above pins
        // that no live path emits `Proven`.
        let promoted = quarantined_promotion::disputed_predecessor_promotion(
            "ahci",
            (8, 1),
            vec!["predecessor evidence line".to_string()],
        );
        let promoted_reason = promoted.reason();
        match promoted {
            LinuxLocality::Proven { device, evidence } => {
                assert_eq!(device, (8, 1));
                assert_eq!(evidence, vec!["predecessor evidence line".to_string()]);
            }
            LinuxLocality::Unproven { .. } => {
                panic!("the quarantined predecessor must preserve the Proven construction")
            }
        }
        assert_eq!(promoted_reason, "proved_local_v0");
        let not_promoted =
            quarantined_promotion::disputed_predecessor_promotion("usb-storage", (8, 32), vec![]);
        assert_unproven(&not_promoted, "p1_unrecognized_storage_stack_v0");
    }

    #[test]
    fn quarantined_predecessor_not_reachable_from_live_path() {
        // Structural pin on the quarantine contract: the only `Proven`
        // construction with a device+evidence payload in this crate is the
        // quarantined predecessor's, and every reference to the quarantined
        // function is its definition, the two calls inside the quarantine
        // evidence test, or this test's own source scan — never a live
        // admission-path caller. (The fixture-matrix sweep separately pins
        // the runtime behavior.)
        let source = include_str!("lib.rs");
        assert_eq!(
            source
                .matches("LinuxLocality::Proven { device, evidence }\n")
                .count(),
            1,
            "the quarantined predecessor must be the only Proven construction site"
        );
        assert_eq!(
            source.matches("disputed_predecessor_promotion").count(),
            4, // definition + two evidence-test calls + this test's own scan
            "the quarantined predecessor must gain no new callers without review"
        );
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
