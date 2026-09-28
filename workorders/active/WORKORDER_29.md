# Hum Work Order 29: decision 0029 implementation — per-platform locality observation and the operator grant

Date: 2026-09-25
<!-- hum-active-workorder:v1 -->
Status: ACTIVE (issued 2026-09-27). Supersedes WO30 as the active Work Order;
WO30 is closed with its full mission complete — closure evidence is recorded
in the transition record below, and the predecessor is preserved byte-for-byte
(minus its active marker) at `workorders/closed/WORKORDER_30.md`. Issuance
authorized by the BDFL (Ocean); this transition PR is draft pending Codex
review. WO29 implementation requires a separate go signal after the
transition is reviewed and integrated.
Decision: 0029 (accepted 2026-09-24, Option D: trusted-local file reads; amended 2026-09-27 — grant-first admission, decision record §14)

## Authorization

Decision 0029 is the sole policy authority for this Work Order. Decision 0015
supplies the evidence vocabulary this Work Order must use: `proved`,
`boundary`, `unproved`, `external-trust`. The locality crates change only by
decision (ledger #17); this Work Order, issued under decision 0029, is that
authorization.

## Mission and queue position

Implement decision 0029's per-platform observed-fact classifiers (P1–P4
evidence) plus the explicit operator grant — grant-first admission per the
2026-09-27 amendment (decision record §14): guest-visible transport,
driver, bus, and removable-media observations are useful facts but do not
alone establish invisible backing, so no classifier in this WO version
emits `proved`; automatic proof and proof-widening are explicitly deferred,
not completed — with evidence labelled per decision 0015, and flip Session
AG to the trust-path byte-exact success on both platforms.

Queue: WO30 closed 2026-09-27 (see transition record below); this Work Order
is now active. WO28 #7 (portable unix read mechanics) merged as `8b0afae`
under the earlier ruling; this draft did not govern #7.

## Transition record — WO30 → WO29 (2026-09-27)

WO30 ("the checker enforces declared task signatures") closed with its
accepted implementation on main. Its mission is distinct from decision
0030 (what `hum check` reports); the decision's own integration and
close-out followed separately. Closure evidence:

- WO30 implementation lineage, all merged and ancestors of the issuance
  base `5171373ec052d5a3e89b04b005c9428cbdc47a1c`: PR #49 (`90f04780`,
  signature table and H0640 arity), PR #50 (`59b2050b`, integer-literal
  range/seal handling, H0011/H0012), PR #52 (`7e791d1b`, argument types,
  H0641 and retired-code migration), PR #53 (`500e6eaa`, negative UInt
  literals, H0642).
- Subsequent decision 0030 integration/close-out, retained: PR #54
  (`fc4ac487`, check-pipeline implementation), PR #55 (`e217a062`,
  documentation close-out); PR #56 (`3e2663c2`) was the separately
  authorized H0642 set-target follow-up.
- Main health at close: push run `36336467387` (Full profile, head
  `9faa850c`, the red-main recovery) completed success 2026-09-27.
- Distinguished — not WO30 evidence: run `36344250017` (head
  `5171373ec052d5a3e89b04b005c9428cbdc47a1c`, the PR #57 squash-merge
  push) completed success at Language profile. It validated the WO29
  draft merge onto main, not WO30's implementation; the two runs must
  not be confused.
- This transition moves `workorders/active/WORKORDER_30.md` to
  `workorders/closed/WORKORDER_30.md`, removing only its active marker
  (every other predecessor byte preserved), and activates WO29 here. No
  WO30 implementation work remains open.

The classifier specification in Item 1 below governs WO29 now that it is active;
it was not in force for #7.

## The trust attestation: what it waives and what it does not

Decision 0029 ruling 4 requires a distinct, unmistakable operator
attestation. Reusing the 0017 capability-consent flag (`--allow
files.read=<path>`) for locality would make every existing read permission
silently waive P1 — the smuggling 0029 §3.1 warns about ("Admitting it
silently would smuggle an external-trust dependency into a grant the
operator believes is local"). Ocean's ruling: the locality attestation is a
**separate flag**, `--trust-locality files.read=<path>`.

The `--trust-locality` attestation waives **proof of P1** (the backing is not
network-backed) and the storage-substitution leg of P3 (the physical medium
being swapped mid-read — unobservable from the guest). The walk/open/identity
mechanics that defeat path rebinding stay enforced.
In this WO29 version (amendment 2026-09-27) the attestation is the sole
locality admission path: no platform classifier emits `proved`.
It does **not** waive: the 0017 capability consent (a matching `--allow
files.read=<path>` for the same path is still required — the trust flag is
valid only with it, never instead of it), P2 (stable identity, via the
pre-read handle-identity comparison), P4 (ordinary file — the unix
component-kind and reparse checks, Windows `is_ordinary_fixed_target`), the
component walk (`symlink_metadata` per component, links never followed during
the walk), the 1 MiB read bound, or strict UTF-8 validation.
Those stay wherever the gate ends up (decision 0029, §4). Without the trust
flag, unproven storage is refused exactly as today. Evidence on the trust
path must say exactly this: locality is `trusted-not-proven` /
`external-trust`; the read hardening still ran.

### P4 resolution (BDFL, 2026-09-28) — superseding clarification

The P4 named in the non-waiver list above ("ordinary file — the unix
component-kind and reparse checks, Windows `is_ordinary_fixed_target`") was
genuinely ambiguous about whether it additionally required a fixed-volume
backing class on the trust path. It did not unambiguously say so, and this
draft does not recast it as having done so. The BDFL resolution (decision
0029 §15, 2026-09-28) is:

- P4 requires **file-object ordinariness** (decision 0029 §4: not a symlink,
  reparse point, device, FIFO, or pipe), not a fixed-volume backing class.
- The Windows `is_ordinary_fixed_target` predicate is **backing
  observation**, not a sufficient file-object check and not an additional
  non-waivable fixed-volume admission requirement.
- Matching trust may cover absent/unproven locality and the already
  specified storage-substitution assumption. It never waives capability
  consent, exact-path matching, the existing lexical/component/reparse
  checks, ordinary-file enforcement, file-identity binding,
  contradictory-evidence rejection, the 1 MiB read bound, or strict UTF-8
  validation. No CLI path syntax widens.

The actual Windows ordinary-file enforcement owners are the open phase, not
the classifier: `src/file_read.rs` (`open_checked_windows_file` — the
component-walk reparse rejection, the non-file final-component rejection,
and the opened-handle `is_file()` check, all before payload consumption —
walked identity is captured earlier and is distinct from the opened
handle's identity), invoked unconditionally by `src/run.rs` Step 4 on
every admitted path including `external-trust`.

Report corrections recorded alongside (precision, not policy): direct UNC
syntax is rejected at path validation — only drive-letter-rooted paths pass
Windows validation; a valid drive-letter-rooted path may pass lexical
validation, and classification then depends on observed mapping and drive
type. Remote observations and substituted mappings remain distinct — both
fail closed without the grant, and the grant covers unproven locality
without guaranteeing successful admission. Native-hardware SD/MMC
observation is unavailable, not impossible. Unexecuted cross-target checks
receive no credit as evidence.

### BDFL rulings incorporated (2026-09-27)

- **CLI spelling:** the space form `--trust-locality files.read=<path>` is
  mandatory; the joined form `--trust-locality=files.read=<path>` remains
  the builder's delegated choice (the `--allow` precedent admits both
  forms).
- **Workflow visibility:** the explicit flag in the reviewed,
  version-controlled script invoked by CI (`tools/check_all.ps1`) satisfies
  decision 0029 ruling 4's "in CI it sits visibly in the workflow file" —
  visible means reviewable, not ambient. No ambient attestation exists or
  is permitted.
- **macOS:** remains unproven/grant-only for this Work Order (Item 3).
- **Grant-first admission (amendment, BDFL, 2026-09-27):** guest-visible
  transport, driver, bus, and removable-media observations remain useful
  facts but do not alone establish invisible backing. No platform
  classifier in this WO version emits `proved`; locality admission is
  grant-first on Linux and Windows alike (macOS remains grant-only). The
  BDFL explicitly accepts the compatibility cost: currently automatic
  Windows admission requires matching per-path `--allow` and
  `--trust-locality`. Automatic proof and proof-widening are explicitly
  deferred, not completed. This authorizes the specification change, not
  its implementation.
- **Queue:** WO29 follows WO30 closure; preparation proceeds now.

### The read pipeline: classification → file identity → observed-identity binding → bounded read → evidence

Every file read under this Work Order traverses one ordered pipeline.
Authority ordering is preserved end to end; nothing below reorders, skips,
or weakens it:

1. **Authority/audit open** — the file-read authority event opens first, as
   today.
2. **Capability consent (0017)** — a matching `--allow files.read=<path>`
   is required before anything else is evaluated.
3. **Locality classification** — the platform classifier (Items 1–3) runs,
   producing observed backing-device facts and an `Unproven` verdict with
   a named reason; no classifier in this WO version emits `proved`
   (automatic proof explicitly deferred — amendment 2026-09-27). The
   operator attestation (Item 4) is then consulted. Refusal here never
   opens the file. Failure is fail-closed (`Unproven` with a named
   reason), never optimistic.
4. **Component walk + open** — the existing hardening is unchanged:
   `symlink_metadata` per component, links never followed during the walk,
   `.`/`..` rejected, then `File::open` of the final component.
5. **File-object identity binding** — the validated file object is bound to
   the opened handle before payload consumption. On unix this is the
   opened handle's fstat `(dev, ino)` together with the pre-read
   walked-vs-handle comparison (`opened_file_matches_walked_target`);
   pathname equality is not a file-identity proof and is never used as
   one. On Windows the opened handle must provide a file/volume identity
   (a volume identity plus a per-file identifier on that volume) — disk
   numbers identify the *device*, not the *file*, and are not file
   identity. The exact OS primitive is builder's choice, reviewed; the
   requirement is specified here. Missing required identity, or an
   identity mismatch, rejects the read fail-closed — the grant never
   covers file identity. The unix-only
   comparison does not provide Windows identity and is not claimed to.
6. **Observed-identity binding** — the *available* observed backing-device
   facts are bound to the opened object: the facts' device identity must
   match the opened object's observed device identity. *Unavailable*
   backing observations are the attested path: an explicit matching trust
   grant covers the unproven P1, and the absence and its reason are honestly
   recorded (`Unproven` with the named reason, `external-trust`
   classification). The grant covers missing locality/backing observations
   only — usable file identity remains mandatory (step 5). The grant
   fabricates no evidence and waives nothing else: file identity, the
   ordinary-file check, and the path safeguards apply to every admitted
   read, including external-trust. Available contradictory binding
   evidence — observed device ≠ opened object device — rejects fail-closed
   before payload consumption, grant or no grant; it is never
   silently ignored.
7. **Bounded read** — the 1 MiB bound and strict UTF-8 validation run only
   after steps 5–6 pass; unchanged.
8. **Evidence emission** — `AuthorityAuditEvent` carries the bound identity
   from step 5 together with the classification and the P1–P4 lines
   (Item 5). The bundle carries identity and classification evidence, not
   file contents; payload byte-exactness is pinned by the output
   assertions (Items 5–6), not by the bundle.

**Specified interface changes (not comments; none of this exists today):**
(a) the read entry point separates open from read: open yields the opened
handle with its identity; the file-identity binding (step 5) and the
observed-identity binding (step 6) run on the opened handle BEFORE the
bounded read. A design that reads the payload and returns bytes before
these checks run violates this contract — enforcement is not post-hoc
validation of returned bytes; (b) the unix read entry point
(`read_checked_unix_file`) threads the opened handle's fstat `(dev, ino)`
through the open phase; (c) the Windows read path captures the opened
handle's file/volume identity and threads it through the open phase —
disk numbers remain device evidence for the observation binding, never file
identity; (d) the locality classifier returns the observed backing-device
identity alongside the classification (the classification is `Unproven`
with a named reason in this WO version — `proved` remains defined but
unreachable) — on Windows this is a new
return-type change owned by `crates/windows-drive-locality` (the
classifier currently returns only a label, and the Windows reader does not
currently provide the opened identity; the audit-corrected plumbing is
specified here, not assumed). The walked-vs-handle comparison stays where
it is. P1–P4 are not weakened: classification still fails closed, and the
trust path still records the `Unproven` reason with the attestation facts.
These corrections require no new semantic decision; the remaining open
choices (reason strings, exit codes, field names, the exact Windows OS
identity primitive) use this draft's established builder's-choice-reviewed
pattern.

**Complete minimal future owners:** `src/file_read.rs` (unix and Windows
open-phase identity capture and threading); `crates/windows-drive-locality`
(classifier verdict demotion to grant-first and return type; Windows
file/volume identity provision);
`src/native_path.rs` (unix classification seam: backing-device evidence);
`src/run.rs` (file-identity binding, observed-identity binding, and
enforcement before payload consumption; evidence emission);
`src/operator_grant.rs` (attestation — shape unchanged, already specified).

**Acceptance — focused future controls** (specified criteria, not
permission to execute now): with injected adapters through the real
pipeline, (a) *honest observation binding* — *available* observed
backing-device facts bound to the opened object; the control asserts the threaded facts appear
in the bundle and match the opened object's device; contradictory binding
evidence (observed device ≠ opened object device) rejects fail-closed
before payload consumption — never logged-and-ignored, grant or no grant
(the grant covers the *absence* of observations, not a contradiction among
available ones); (b) *honest
attested admission* — trust grant with unavailable P1/backing observations:
admission under `external-trust` with the absence and reason honestly
recorded; the control asserts no fabricated proof evidence appears; that
file identity, ordinary-file, and path safeguards still reject — missing
usable file identity rejects even on the attested path — and that
contradictory binding evidence still rejects (the grant covers absence,
not contradiction);
(c) *reason distinction* — known-network evidence (e.g. `transport=tcp`)
and merely-insufficient evidence (e.g. `transport=pcie` with no further
proof) yield distinct `Unproven` reasons, pinned; both fail closed without
the grant; (d) *no proved emission* — across injected adapters covering
every classifier path, the classifier never emits `proved` in this WO
version; (e) *same-disk file substitution* — same device identity,
different file identity rejects, proving device identity is not file
identity; (f) *missing required file identity* — the opened handle
provides no usable file identity → reject fail-closed. Separately,
(g) *emitter consumption* (propagation only) — the threaded identity
substituted before emission appears in the bundle, proving the emitter
consumed the threaded value; (g) does not prove enforcement.

### Platform effects of the shared attestation and rendering (Items 4–5)

Items 4 and 5 are platform-shared; their effects are stated here, separately
from the platform classifier changes (Items 1–3):

- **Windows:** the attestation is the sole locality admission path in this
  WO version: bus-type evidence (ATA/SATA/NVMe, and SD/MMC after Item 2),
  `RemovableMedia`, disk extents, the dependency walk, and the
  before/after observation are recorded as observed facts with an
  `Unproven` verdict — none independently earns `proved` (amendment
  2026-09-27). `is_ordinary_fixed_target` (P4) is unchanged and still
  enforced on every admitted read; it is an ordinary-file check, not a
  locality proof. Item 2 widens which bus types yield local-bus *facts*,
  not the verdict.
- **macOS:** the classifier always returns `Unproven` (Item 3); the
  attestation is the only admission path. Rendering is identical to every
  other platform.
- **Rendering:** the stderr evidence line and the JSON channel behave
  identically on all platforms. Items 4–5 introduce no per-platform
  rendering differences and no new classifier behavior.

---

## Item 1 — Linux P1 observed facts: exact acceptance criteria

Given the canonicalized absolute candidate path P, the read path
(`src/file_read.rs`, `read_checked_unix_file`) performs: a component walk
with `symlink_metadata` per component (links never followed during the walk;
`.`/`..` rejected); `File::open` of the final component; then a pre-read
handle-identity comparison (`opened_file_matches_walked_target`) — the
handle's fstat `(dev, ino)` must equal the walked final component's
`(dev, ino)`, else `UnsafePath`. There is no explicit `O_NOFOLLOW` and no
before/after-read verification by those names; the walk/open/identity
mechanics above are the P2/P3-walk evidence and are unchanged by this Work
Order (the interface change is the threaded opened identity specified in
the pipeline section above, and the stale "P1 remains unproven on unix" doc
comment is refreshed). The Linux classifier extracts observed
backing-device facts per the steps below and returns `Unproven` with a
named reason: guest-visible transport, driver, and removable-media
observations are useful facts but do not alone establish invisible backing
(amendment 2026-09-27) — there is no automatic P1 proof in this WO version.
Any failure
is `Unproven` with a named reason — never an optimistic admission (decision
0029, §3: "Fail-closed is non-negotiable. Unclassifiable storage is refused,
never admitted on optimism").

**Step 1 — mount evidence.** Read `/proc/self/mountinfo`. The kernel documents
the field layout: mount ID, parent ID, `major:minor` device, mount root, mount
point, mount options, then after the `-` separator the filesystem type, mount
source, and super options — and requires parsers to ignore unrecognized
optional fields (source: `Documentation/filesystems/proc.rst`, §3.5,
https://www.kernel.org/doc/Documentation/filesystems/proc.rst).

Select the entry per the mount-topology oracle: unescape mountinfo path
escaping (octal escapes such as `\040`) before component comparison; group
entries by mount-point location; at each location the effective (topmost)
entry is the one that is **not the parent of any other entry at the same
location** (mount IDs may be reused after umount, so ID ordering is not the
tiebreak — `proc_pid_mountinfo(5)`); then take the longest component-aware
mount-point prefix of P among *accessible* entries — a mount point recorded
under a covered (non-topmost) entry is hidden by the stacked mount and is
not selected (e.g. an overmount at `/a` hides a `/a/b` recorded under the
covered `/a`: the overmount's parent is the covered mount, so the covered
mount is the parent of another entry at the same location and is not
topmost). If no unique topmost exists at a location on P's path, or parent
references dangle ambiguously, selection is `Unproven` — never guessed. A
parent ID with no corresponding record because the parent lies outside the
process's root (chroot) is the documented legitimate omission
(`proc_pid_mountinfo(5)`) and is tolerated, not treated as ambiguity.
Record the filesystem type, the mount source, and the `major:minor` device.
Refuse (`Unproven`, trust path) when the filesystem type is:

- a network protocol: `nfs`, `nfs4`, `cifs`, `smb3`/`smb`, `ncpfs`, `afp`,
  `ceph` — positively network-backed; or
- any `fuse`-prefixed type (sshfs and every userspace filesystem — the kernel
  cannot observe the daemon's backing, so locality is unclassifiable from
  kernel evidence); or
- `9p` or `virtiofs` (host-shared into the guest; the backing is invisible to
  the guest — decision 0029 ruling 3).

Refusals for positively network-backed types carry a **known-network**
reason, distinct from the **insufficient-evidence** reason below; the
distinction is pinned by tests (exact reason strings are the builder's
choice, reviewed).

**Step 2 — block-device identity.** The `major:minor` from mountinfo must equal
P's `st_dev` (consistency check; mismatch is `Unproven`). This primarily
validates that the oracle selected the right mount record — it is not
independent locality evidence. Resolve
`/sys/dev/block/<major>:<minor>` to its canonical `/sys/block/<disk>` whole
disk (partitions resolve through their parent directory). Sources: the stable
sysfs block ABI (`Documentation/ABI/stable/sysfs-block`,
https://mjmwired.net/kernel/Documentation/ABI/stable/sysfs-block) and the
kernel's sysfs access rules — sysfs is always mounted at `/sys`
(https://docs.kernel.org/6.7/admin-guide/sysfs-rules.html). The `removable`
attribute is the stable-ABI witness for `GENHD_FL_REMOVABLE`
(https://www.kernel.org/doc/html/v5.12/block/capability.html).

Read `<disk>/removable` and identify the driver via the `<disk>/device`
symlink target and the disk name pattern.

- **Always grant, never proof (decision 0029, ruling 3):** if the device
  resolves to virtio-blk (`vd*`), Xen (`xvd*`), Hyper-V storvsc, VMware
  PVSCSI, or any device whose backing the guest cannot observe, the classifier
  returns `Unproven` (e.g. `p1_guest_invisible_backing_v0`) and the read is
  grant-only. The classifier must *detect and decline*; it must never argue
  host-locality from guest-visible data. Retained and extended by the
  2026-09-27 amendment: guest-visible-but-hypervisor-invisible backing is
  likewise unprovable from the guest.
- **Observed facts (not proof) in this WO version** (amendment 2026-09-27 —
  automatic P1 proof explicitly deferred; these observations are useful
  facts for the operator's trust decision and for future proof-widening):
  - `nvme*n*`: read the controller's `transport` attribute
    (`<disk>/device/transport` directly — for NVMe namespaces the disk's
    parent device IS the controller, `device_add_disk(ctrl->device, ...)`
    in `drivers/nvme/host/core.c`; source for the attribute: the stable
    sysfs ABI, https://docs.kernel.org/admin-guide/abi-stable.html).
    `transport` of `tcp`/`rdma`/`fc` is **known-network** (`Unproven` with
    the known-network reason — NVMe-oF presents the same `nvme*n*` device
    nodes and is network storage). `transport` of `pcie` is recorded as the
    observed fact `{transport: pcie}` with `Unproven` and the
    insufficient-evidence reason — an emulated PCI NVMe frontend with a
    network/file backend presents the same fact. A missing or unreadable
    `transport` attribute is `Unproven` (insufficient evidence) — fail
    closed. NVMe multipath heads (disk parented to the subsystem device,
    `device_add_disk(&head->subsys->dev, ...)`, no `transport` attribute)
    are `Unproven` (insufficient evidence).
  - `sd*`: walk the sysfs device chain from `<disk>/device` upward,
    collecting driver names from the `driver` symlinks, and record the
    chain as observed facts. A driver in the chain matching the positive
    allowlist of local HBA drivers — seeded with the libata family (e.g.
    `ahci`, `ata_piix`) and named local SAS HBA drivers — is the observed
    fact `{local_hba_driver: <name>}` with `Unproven` and the
    insufficient-evidence reason. The exact list is the builder's choice,
    reviewed at implementation; widening it is a decision, not an
    implementation detail. virtio-scsi, Fibre Channel, USB mass storage,
    Hyper-V storvsc, and VMware PVSCSI are `Unproven` (guest-invisible or
    insufficient evidence, reason naming the cause); iSCSI is
    **known-network**.
  - `mmcblk*` with `device/type` of `MMC` (eMMC) or `SD` — this incorporates
    the eMMC/SD P3 research argument (commit
    `c3ed1adcd36b18b2827f202d570edc7662504052`, research lane) by reference.
    `removable == 0` is recorded as the observed fact with `Unproven` and
    the insufficient-evidence reason; `removable == 1` is `Unproven` with
    the reason naming removability (the circular-reasoning argument is
    unchanged — the removable bit is the only kernel witness to physical
    fixity, so trusting a `1` to mean "fixed" would be the claim proving
    itself).
- **`Unproven` in this WO version (trust path):** `dm-*` (device-mapper,
  including LUKS), `md*` (MD RAID), `loop*`, `nbd`, `rbd`, `drbd`, and any
  device the classifier cannot resolve. These are honest `Unproven` with the
  insufficient-evidence reason — recorded as observed facts (device class
  observed, backing unresolved), not refusals of the concept. A device or
  protocol name alone (`nbd`, `rbd`, or similar) never justifies the
  known-network reason; the stronger reason requires additional observations
  (e.g. a `tcp`/`rdma`/`fc` transport attribute). A later decision may widen
  the admission with its own evidence.
- **Fail closed:** any mountinfo or sysfs read/parse failure, any unresolvable
  device, any unrecognized layout → `Unknown` → `Unproven`
  (e.g. `p1_evidence_unavailable_v0`) → refuse. The read fails closed with the
  existing `FileReadError.unavailable` unless the operator grant is present.

The evidence bundle records the observed facts: the mountinfo fields
(filesystem type, mount source, `major:minor`), the sysfs disk path, the
`removable` value, the driver/bus identity, the `Unproven` reason (naming
known-network vs insufficient evidence vs guest-invisible backing), and the
walked-vs-handle identity observation (the `(dev, ino)` pair and the
equality verdict) — one line per property P1–P4, in this Work Order's
format, supplying the end-to-end traceability decision 0029 §8 requires.

**What these observations do not establish** (amendment 2026-09-27): they
do not establish invisible backing. An emulated PCI frontend (QEMU
NVMe/AHCI) with file- or network-backed block storage presents the same
guest-visible transport/driver facts; the operator grant covers that
residual. The observations establish guest-visible stack termination in
media of the stated class — useful facts for the trust decision and for
future proof-widening — not P1.

**Stated limitations** (decision 0029 §8: limitations written down, not
discovered in an incident): anonymous-device filesystems — overlay, tmpfs —
whose mountinfo device is `0:0` or otherwise does not resolve under
`/sys/dev/block` fail step 2 and are grant-only; btrfs spanning multiple
devices fails the single-device identity check and is grant-only.

## Item 2 — Windows: widen observed local-bus facts to fixed, non-removable eMMC/SD

The research-lane P3 argument for eMMC/SD is already recorded in commit
`c3ed1adcd36b18b2827f202d570edc7662504052` ("docs(research): eMMC/SD P3
argument"). This Item implements it; it does not re-derive it. The Work Order
incorporates that commit by reference as the evidence for this widening.

Acceptance criteria (all per `c3ed1ad`, §2–§7):

- In `crates/windows-drive-locality/src/lib.rs`, add `BUS_TYPE_SD` (12) and
  `BUS_TYPE_MMC` (13) to the observed local-bus list alongside ATA/SATA/NVMe
  (the gate at `lib.rs:204` becomes an observation gate: which bus types
  yield local-bus facts). The verdict demotion (grant-first admission: no
  bus-type observation independently earns `proved`) lands in Slice A per
  the slice description below — it is not this Item's change. This Item
  widens only the observed bus list; observation-list work stays distinct
  from admission changes. Nothing else in the
  gate changes.
- The `STORAGE_DEVICE_DESCRIPTOR.RemovableMedia` handling stays exactly as
  is: fixed, non-removable eMMC/SD (`RemovableMedia` false) yields the
  local-bus observed fact; removable SD, USB readers, unreadable removable
  status, and welded media still reported removable yield `Unproven` with
  the reason naming the cause (the circular-reasoning argument is
  unchanged). Neither earns `proved`.
- Unchanged and re-pinned by tests as observed-fact extraction: disk extents
  → disk numbers, the disk-number/dependency binding, the
  virtual-dependency walk (a virtual dependency in the chain is positively
  identified and fails closed), the before/after observation equality
  (`lib.rs:165`), and P4 via `is_ordinary_fixed_target` (`lib.rs:257`) —
  recorded facts and negative observations, not proof.
- The evidence bundle per `c3ed1ad` §8 is emitted for every classification:
  the bus/media class observed (enum value plus name), the removable-flag
  value and which query produced it, the device identity (disk numbers), the
  before/after observation and equality verdict, a one-line P1–P4 mapping, the
  classification with provenance, and the observed bus list shown in the
  bundle so the next widening is visibly a decision. The bundle shape rides
  the shared Item 5 surface (available from Slice A); this Item widens the
  observed bus list, not the API.
- The classification provenance distinguishes observed-but-unproven (with
  the `Unproven` reason) from `trusted` (operator grant, external-trust) —
  consistent with decision 0015. The `proven` label remains defined but is
  unreachable in this WO version (automatic proof explicitly deferred).

## Item 3 — macOS: declared unprovable (grant only)

**Ruling (BDFL, 2026-09-27): macOS P1 is declared unprovable; macOS file
reads proceed only under the operator grant.** The classifier on macOS
returns `Unproven` with a named reason (builder's choice, e.g.
`macos_p1_unprovable_v0`); ordinary-file and stable-identity checks (P2–P4)
still run. This is the honest declaration decision 0029 §6 explicitly
permits ("an honest declaration of unprovability").

Reasoning (the Work Order records the losing alternative so the choice is
auditable):

- The only *stable, documented* kernel locality signal on macOS is the
  `MNT_LOCAL` flag in `statfs(2)`'s `f_flags` — documented as "File system is
  stored locally" (man page: `fstatfs(2)`; e.g.
  https://www.unix.com/man-page/osx/2/fstatfs64/). It is
  filesystem-granularity, not device-granularity, and has known
  silent-admission holes: a disk image (`dmg`) mounted from an NFS share
  presents as local APFS/HFS with `MNT_LOCAL` set while its bytes arrive over
  the network; FUSE filesystems' locality depends on their userspace daemon,
  which the kernel cannot observe; `virtiofs` (Apple Silicon VM host shares)
  is host-backed and guest-invisible. A proof with a known false positive
  admits on optimism, which decision 0029 §3 forbids ("Fail-closed is
  non-negotiable").
- The fine-grained alternative — resolving `st_dev` to a BSD disk name and
  walking the IOKit device plane for physical-interconnect evidence — has no
  stability contract (driver-reported keys change across releases), is fragile
  under sandboxing, and collapses to the grant on VMs per ruling 3 anyway: it
  buys proof only on bare-metal Macs at high maintenance cost.
- The trust path is the same mechanism on every platform (decision 0029,
  principle 3: no environment special cases), and CI's macOS runs are hosted
  runners — grant regardless. The cost of the honest declaration is one
  visible flag on the command line.

A future decision may admit a coarse macOS proof if its holes are closed;
that widening is a decision, not an implementation detail.

## Item 4 — the trust attestation: exact spelling, no ambient construction, labelled evidence

The locality attestation is a **distinct flag**, not a reuse of the 0017
capability consent. Ocean's ruling: `--trust-locality files.read=<path>`.

- **Exact CLI spelling (pinned):** `--trust-locality files.read=<path>`
  (space form, mandatory). Ruling (BDFL, 2026-09-27): the joined form
  `--trust-locality=files.read=<path>` remains the builder's delegated
  choice (the `--allow` precedent admits both forms); the space form must
  work regardless.
- **Per-invocation, per-path, command-line only** (decision 0029, ruling 4):
  the attestation is given fresh on `argv` on each `hum run` invocation, for
  one exact native path. It is **valid only with a matching `--allow
  files.read=<path>` for the same path** — capability consent (0017) and
  locality attestation (0029) are separate facts, and the read requires
  both. The trust flag is never a substitute for `--allow`. **No
  environment variable, config file, or persistent setting may confer the
  attestation.** Acceptance includes negative tests: setting a plausible env
  var (e.g. `HUM_TRUST_LOCALITY`) must not attest anything, and `--allow`
  alone on unprovable storage still refuses exactly as today (no silent P1
  waiver — the smuggling 0029 §3.1 warns about).
- **Evidence labels (decision 0015 vocabulary):** the file-read authority
  event's locality classification is one of 0015's classes —
  - `proved` — defined by decision 0015; unreachable in this WO version
    (no platform classifier emits it; automatic proof explicitly deferred
    — amendment 2026-09-27);
  - `external-trust` — the sole admission classification in this WO version,
    admitted by operator attestation; the evidence
    records `trust_locality_present: true`, the attested path, the matching
    `--allow` path, the classifier's `Unproven` reason, and the literal
    human label below.
  - Human output must contain the literal string `trusted-not-proven`,
    unmissable — e.g. a stderr evidence line
    `files.read <path>: trusted-not-proven (operator attestation,
    external-trust; classifier: <reason>)`. Exact line format is the
    builder's choice; the acceptance is the literal string.
  - JSON output must contain the classification value `external-trust`
    (field name builder's choice, value pinned) alongside the attestation
    facts above. The JSON channel itself is specified in Item 5 — no JSON
    run output exists today.

## Item 5 — evidence surfaces (human + JSON): one minimal contract

Today the file-read authority event is test-visible only:
`RunReport.authority_events` is consumed by tests and `hum run` renders
nothing of it in ordinary output. No JSON run output exists today (`hum run`
has no `--format` flag; `render_run_command_execution` prints outcome text
only). This Item specifies exactly one minimal contract — not a general
reporting framework. `hum evidence` is not extended; no event bus is
introduced.

The contract:

- **In-process surface (unchanged, completed):** `RunReport.authority_events`
  remains the test-visible machine-readable channel. `AuthorityAuditEvent`
  is the named carrier of the per-read evidence bundle: the locality
  classification (`external-trust`; `proved` is defined but unreachable in
  this WO version), the observed-fact lines, and the attestation facts plus
  the classifier's `Unproven` reason on trust paths. Field names are the
  builder's choice; the values above are pinned.
- **Human channel — two layers, stated separately:**
  - *Program output (pinned byte-exact):* the bytes the app wrote are
    emitted exactly as written. WO29 pins this and changes nothing else
    about it.
  - *Task-result rendering (existing behavior, unchanged):* hum's own
    outcome rendering keeps its established behavior, including its
    existing trailing-newline convention. That behavior predates this Work
    Order; this Work Order neither extends nor re-pins it beyond
    "unchanged". Byte-exactness is asserted on the program's bytes, where
    the app's trailing newline and the renderer's newline are distinguished
    by the byte count.
  - *Stderr evidence line:* exactly one stderr line per file-read exercise
    that carries a classification, and only when a file read was exercised
    (existing empty-stderr pins stay green). Trust path: the line contains
    the literal `trusted-not-proven`. There is no proof path in this WO
    version. Exact surrounding format is the builder's choice; the
    pinned content is the label strings from Item 4.
- **JSON channel (new surface, authorized by this Item):** one new `hum run`
  output mode (flag name builder's choice; recommended `--format json`).
  In JSON mode stdout carries **only** the JSON envelope — no human outcome
  text, no fallback rendering, no replay of the human channel. The envelope
  embeds the program's output bytes byte-exactly (encoding builder's
  choice; the round-trip is byte-exact) and the `authority_events` array
  with the Item 4 values (`locality_classification`: `external-trust`;
  `proved` is defined but unreachable in this WO version). Failure behavior: the
  envelope is still emitted on failure; the exit code is unchanged and
  authoritative; a refused read appears as an event carrying its exact
  refusal reason. *Envelope-construction failure* (the envelope itself
  cannot be built) is a different case from ordinary program failure: no
  program-output replay and no human-stdout fallback — the run exits
  nonzero as a reporting failure (exact code builder's choice, pinned by
  test, reviewed), diagnostics go to stderr, and any earlier execution
  error is preserved in those diagnostics rather than masked. Ordinary
  program failure keeps the constructed envelope with its failure event
  and the program's exit code.
  Acceptance for this case is bounded. A *reachable, recoverable*
  envelope-construction failure must exhibit the behavior above
  (nonzero exit, stderr-only diagnostics, no human-stdout fallback). If
  the production construction path has no recoverable failure channel,
  acceptance does not require inventing one — no artificial `Result`,
  dead error arm, or synthetic failure solely to satisfy the criterion.
  Instead the builder provides independent verification of that fact
  from the source (the actual construction code and its type/error
  paths, reviewed), plus the actual-command JSON success and
  ordinary-failure controls, which remain mandatory. A test of a
  diagnostics helper is not production-path failure evidence and does
  not satisfy this criterion. Allocation failure, process termination,
  missing captured output, and stdout write failure are distinct failure
  modes; proving the construction path unreachable tests none of them
  and must not be claimed as handling them. This clarifies the
  acceptance criterion; it is not acceptance of any implementation.

Acceptance: Session AG (Item 6) pins both the stderr literal and the JSON
`external-trust` value, so this Item's acceptance is the AG test, not a
screenshot. Focused controls (in the CLI test file): empty program output
(zero bytes embedded, envelope still well-formed); output ending in a
newline (byte-exactness distinguishes the app's newline from the
renderer's); partial output on failure (the envelope carries the bytes
captured before failure plus the failure event); nonzero exit (envelope
present, exit code unchanged).

## Item 6 — Session AG end state: byte-exact success under the trust attestation, visibly labelled, both platforms

Current pins (as shipped): the Windows hosted-runner locality-refusal pin
(exit 1, `FileReadError.unavailable`, reason
`fixed_local_v0_not_proven_before_candidate_access_v0`) and the non-Windows
P1-unproven refusal pin (exit 1, `FileReadError.unavailable`, reason
`p1_locality_unproven_on_this_platform_v0`). The fixture path is absolute
and repo-root-joined on both platforms.

The harness correction rides Slice A — it is not deferred:

- **Unix and Windows (Slice A):** no classifier in this WO version emits
  `proved`, so the no-grant refusal pin is hardware-independent: the read
  is refused fail-closed on every platform. The Windows admission demotion
  lands in Slice A — the ATA/SATA/NVMe branches become observed-fact
  extraction with `Unproven` verdicts — so the shipped Windows refusal
  pin's reason is superseded in Slice A, and both platforms' no-grant pins
  assert the demoted refusal shape. The native harness keeps two negative
  fixtures distinct, with no permission-changing fixtures: an OS-readable
  owned file with no Hum `--allow` → `FileReadError.denied` (Hum consent
  enforcement, independent of OS readability), and the same file with
  matching `--allow` on unprovable storage but no trust flag →
  `FileReadError.unavailable` with the classifier's `Unproven` reason
  (attestation missing). Missing consent and missing attestation are
  different refusals and are pinned separately. The trust-flag run asserts
  exit 0, byte-exact stdout, **and** the required bound evidence: the
  evidence bundle carrying the fixture file's opened file identity — unix
  `(dev, ino)`, Windows volume/file identity — the `external-trust`
  classification, and the classifier's `Unproven` reason honestly recorded.
  The guarantees are identical on both platforms; the identity fields are
  platform-correct, not identical.

The new assertions, on **both** Windows and Ubuntu (Slice A):

- `hum run examples/tools/wordfreq.hum --allow stdout.write
  --allow=files.read=<fixture path> --trust-locality
  files.read=<fixture path> --args <fixture path>` exits 0, where the
  fixture path is absolute and repo-root-joined on both platforms (as the
  current pins spell it) and the attested path exactly matches the
  requested path.
- Stdout is byte-exact `hum\nlang\nhum\nhum: 2\nlang: 1\n` (28 UTF-8 bytes,
  LF newlines) for `fixtures/wordfreq/sample.txt` (15 bytes). The 28 bytes
  are the program's bytes per Item 5's program-output layer — the
  task-result rendering's own trailing newline is distinguished by the
  byte count and is not part of the asserted program output.
- The evidence is asserted on both surfaces: the human output contains the
  literal `trusted-not-proven`; the JSON output contains the `external-trust`
  classification with the attestation facts. On CI both platforms' storage
  yields `Unproven` (Azure runners are virtual/SCSI), so these assertions
  exercise the trust path — the classifier's observed-fact branches are
  covered by unit tests with fixtures/mocks.
- The misuse pins stay, plus the new one: without the trust flag, the read
  refuses exactly as today (`FileReadError.unavailable` on unprovable
  storage) even when `--allow` is present; without `--allow` it stays
  `FileReadError.denied` (the existing no-grant pin is not removed).

The trust flag sits in `check_all.ps1`'s Session AG invocation — a
version-controlled, reviewed file. Ruling (BDFL, 2026-09-27): that satisfies
decision 0029 ruling 4's "in CI it sits visibly in the workflow file" —
visible means reviewable, not ambient.

## Proposed answers to WO28 #7's open semantic questions

WO28 #7's cold-start map (2026-09-25) stopped on four questions. This draft
**proposes** answers for WO29; they take effect only if and when WO29 is
activated. #7 merged as `8b0afae` under the earlier ruling and WO28 is
closed — a draft Work Order changes nothing until it is activated:

- **Q1 (what is the Linux P1 specification?):** Item 1 defines it exactly —
  observed-fact extraction; automatic proof is deferred, not completed.
- **Q2 (macOS scope):** grant only — Item 3.
- **Q3 (classifier-vs-grant ordering):** the classifier *definition* belongs
  to WO29; #7 is mechanical portability plus the honest refusal.
- **Q4 (does Session AG use the Ubuntu grant?):** yes — Item 6, with the
  distinct trust flag.

## Lane assignments and STOP conditions

- **Research lane (this draft + incorporated `c3ed1ad`):** the Work Order
  text, the Linux P1 evidence specification (Item 1), the macOS
  recommendation (Item 3), and the Windows widening evidence argument
  (Item 2, via `c3ed1ad`).
- **Builder lane:** the classifier implementation in the locality crates,
  the CLI surface, the evidence rendering, the Session AG flip, and the
  tests. The locality crates change only by decision; this Work Order under
  decision 0029 is that authorization.
- **STOP:** if implementing any Item requires new language surface or an
  unmade semantic choice, STOP and report to the BDFL instead of inventing
  it. If a platform's classifier cannot be implemented as specified, the
  implementation reports that as a finding — it does not weaken the
  specification.

## Review and evidence requirements

- Independent pre-issuance review of this draft (Codex) before acceptance
  and issuance. Draft PR publication for review is already authorized
  (PR #57) and is neither acceptance nor issuance.
- Honesty locks (decision 0014): no output text or doc may claim more than
  the implementation proves. Evidence labelled `proved` must carry the
  bundle; evidence labelled `trusted-not-proven` / `external-trust` must
  carry the grant facts and the classifier's `Unproven` reason. The trust
  grant covers unavailable locality/backing observations only; it fabricates
  no evidence and waives no safeguard — usable file identity remains
  mandatory, and the ordinary-file check and the path safeguards apply to
  every admitted read.
- Every Item's acceptance criteria are asserted by tests, not by prose.
  Session letters continue the project odometer.

## Implementation slices

Dependency order 1 → 4 → 5 → 2 → 3, in two slices sized for one
review sitting each. Item 6's AG harness correction rides Slice A — it
depends on Items 1 + 4 + 5 only, not on the Item 2/3 observation work.

- **Slice A — Items 1 + 4 + 5 + 6 (AG harness correction).** The Linux
  classifier must not land without the operator recourse (Item 4), the
  surfaces that observe both paths (Item 5), and the grant-first AG pins
  (Item 6) — otherwise the Item 1 observations have no admission path and
  the pins describe behavior that does not exist yet.
  Windows admission is demoted to grant-first in this slice: the
  ATA/SATA/NVMe observation branches become observed-fact extraction with
  `Unproven` verdicts — no bus-type observation independently earns
  `proved` — and the attestation is the sole locality admission path on
  Windows from this slice on. macOS remains grant-only, unchanged. Items
  4–5 are platform-shared: the attestation, the stderr evidence line, and
  the JSON channel land on all platforms in Slice A, with platform checks
  covering the shared effects (Windows: the trust-path assertions on
  hosted runners; macOS: the grant-only path).
  The audit-corrected Windows plumbing also lands in Slice A: the
  classifier currently returns only a label and the Windows reader does
  not currently provide the opened identity, so the minimal future changes
  are specified here — `crates/windows-drive-locality` extends the
  classifier return to carry the observed backing-device identity (disk
  numbers, per the existing `c3ed1ad` §8 vocabulary; no unix terminology
  imported), and the Windows read path captures and threads the opened
  handle's file/volume identity (volume identity plus per-file
  identifier — disk numbers identify the device, never the file). Owners:
  `crates/windows-drive-locality` for the classifier verdict demotion,
  the return type, and the file/volume identity provision;
  `src/file_read.rs` and `src/run.rs` for the capture, threading, pre-read
  enforcement, and emission. Slice B widens only the *observed bus list*
  (the gate at `lib.rs:204`) — observation facts, not admission: no
  bus-type observation independently earns `proved`.
  Plumbing availability and admission widening are separate changes. No new
  framework is introduced — the new controls ride the existing unit and
  CLI test files.
- **Slice B — Items 2 + 3.** Windows SD/MMC observation widening; macOS
  declared unproven (grant-only). Platform-gated; no CLI, rendering, or
  admission changes.

Item 2 is Windows-specific, not platform-independent: it shares the Windows
owners with Slice A's Windows plumbing (`crates/windows-drive-locality`,
the Windows read path in `src/file_read.rs`, `src/run.rs`). It does not run
alongside Slice A — the accepted dependency order (1 → 4 → 5 → 2 → 3)
and the slice sequence stand, and this draft remains the sole writer of the
Work Order text (one-writer rule preserved).

## Affected-file scope (complete)

New: `crates/linux-drive-locality/` (manifest, `src/lib.rs`, fixture trees
under `tests/`); `tests/cli_wo29_trust_locality.rs`.

Modified: `Cargo.toml` (workspace members, dependency); `Cargo.lock` (the new
workspace member; mechanical — no version changes beyond what the new member
requires); `src/native_path.rs` (unix classification seam, the
backing-device evidence, and the locality classification label (the
`proved` label is defined but unreachable in this WO version);
`src/operator_grant.rs` (the attestation field, setter,
accessors); `src/main.rs` (argv arms, run-only gating, usage); `src/run.rs`
(gate order, `AuthorityAuditEvent` bundle fields, the threaded opened
identity consumed at enforcement and emission, the file-identity and
observed-identity binding checks before payload consumption, render paths);
`src/file_read.rs` (the read entry point separates open from read — the open
phase yields the handle with its identity for pre-read enforcement — and
the stale P1-unproven doc comment; the Windows path gains the file/volume
identity capture; identity mechanics change wherever the new checks
require it);
`crates/windows-drive-locality` (classifier verdict demotion to grant-first,
return type carrying the observed backing-device identity, and Windows
file/volume identity provision — Slice A; the observed bus-list widening —
Slice B);
`tools/check_all.ps1` (Session AG pins: both-platforms harness correction in
Slice A);
`tools/test_ci_policy.ps1` (mechanically affected: the SHA-256 pins over any
edited `check_all.ps1` function bodies are recomputed by the builder and
verified at review; no policy change).

Untouched: `docs/` (research lane), all fixtures.

## Focused acceptance plan

- **Crate unit tests** (`linux-drive-locality`, fixture-driven): mountinfo
  longest-prefix selection and optional-field tolerance; the refuse-by-fstype
  matrix (nfs/nfs4/cifs/smb/fuse/9p/virtiofs/overlay); the sysfs matrix
  (nvme+pcie recorded as the observed `{transport: pcie}` fact, `Unproven`
  with the insufficient-evidence reason; nvme+tcp/rdma/fc refused as
  known-network; nvme+loop recorded as observed fact, `Unproven`
  insufficient-evidence; multipath heads and missing transport refused as
  insufficient-evidence; sd+local-HBA-driver recorded as observed fact,
  `Unproven` insufficient-evidence; virtio-scsi/usb/storvsc refused as
  guest-invisible or insufficient-evidence; iscsi refused as known-network;
  nbd/rbd device-name-only observations recorded as observed facts,
  `Unproven` with the insufficient-evidence reason (the name alone never
  justifies known-network);
  mmcblk non-removable MMC/SD recorded as observed fact, `Unproven`
  insufficient-evidence; removable refused with the removability reason;
  vd*/xvd*/storvsc refused; dm/md/loop refused; st_dev mismatch refused;
  unreadable sysfs refused). Every `Unproven` pins its exact reason string.
- **Policy/CLI/gate tests** (injected adapters, no host): the attestation
  setter accepts `files.read=<abs path>`, rejects empty/non-`files.read`
  payloads and second distinct paths, exact duplicates idempotent, no host
  access on rejection; space form, joined form, missing-value error,
  non-`run` rejection, post-`--args` ordering; the gate admits on
  attestation (`external-trust`) and refuses otherwise with the adapter
  never called; `HUM_TRUST_LOCALITY` set with no flag still
  refuses.
- **CLI tests** (Linux VM, overlay/tmpfs → deterministically `Unproven`):
  the trust-path run exits 0 with byte-exact stdout, the `trusted-not-proven`
  stderr literal, and `external-trust` in the JSON channel; `--allow` alone
  refuses; mismatched attestation/allow paths refuse.
- **Harness (Slice A, both platforms):** the no-grant pins assert the
  refusal shape on unix and Windows (hardware-independent — no classifier
  in this WO version emits `proved`); the trust-flag runs assert exit 0,
  byte-exact stdout, and the bound evidence bundle per Item 6
  (`external-trust` with the `Unproven` reason; platform-correct identity
  fields — unix `(dev, ino)`, Windows volume/file identity); the two
  negative fixtures (missing Hum consent vs missing attestation, no
  permission changes) are pinned separately; the focused future controls are
  specified per the pipeline section — honest observation binding, honest
  attested admission, reason distinction, no proved emission, contradictory
  binding evidence, same-disk file substitution, missing required file
  identity — plus the emitter-consumption propagation check.
- **Fixture evidence vs native hardware observation:** everything above is
  fixture-testable and deterministic. This WO version defers automatic
  proof, so no native-hardware `Proven` admission is an acceptance
  requirement here — CI runners exercise only the trust path. The NVMe-oF
  negative is unit-pinned (`transport=tcp → Unproven` with the
  known-network reason) but not hardware-observed end-to-end; that residual
  is stated, not solved. This defers, it does not forbid: future
  proof-widening (a decision, not an implementation detail) may require
  native-hardware observation. If any accepted property cannot be met as
  specified, the implementation reports that concrete finding — it does not
  invent a replacement policy.
