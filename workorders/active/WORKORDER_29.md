# Hum Work Order 29: decision 0029 implementation — per-platform locality proofs and the operator grant

Date: 2026-09-25
Status: DRAFT — pre-issuance review. Not active. WO28 remains the active Work
Order. (The active-workorder marker comment is intentionally absent from this
file; it is added only when this Work Order is activated.)
Decision: 0029 (accepted 2026-09-24, Option D: trusted-local file reads)

## Authorization

Decision 0029 is the sole policy authority for this Work Order. Decision 0015
supplies the evidence vocabulary this Work Order must use: `proved`,
`boundary`, `unproved`, `external-trust`. The locality crates change only by
decision (ledger #17); this Work Order, issued under decision 0029, is that
authorization.

## Mission and queue position

Implement decision 0029's per-platform property proofs (P1–P4) plus the
explicit operator grant, with evidence labelled per decision 0015, and flip
Session AG to the grant-path byte-exact success on both platforms.

Queue: this Work Order executes **after WO28 #7**. WO28 #7 (in progress,
builder lane) ports the read mechanics fail-closed with a `P1 unproven`
refusal. WO29 turns the capability on. #7 builds the mechanics and the
fail-closed refusal against the proof specification in Item 1 below; WO29
supplies the proof itself, the Windows widening, the macOS declaration, the
grant surface, and the Session AG flip.

## What the grant does and does not waive

The operator grant waives **proof of P1** (the backing is not network-backed)
and the physical-swap leg of P3 (no mid-read substitution). It does **not**
waive: P4 (ordinary file, via `is_ordinary_fixed_target`), the component walk,
`O_NOFOLLOW`, the before/after fstat observations, the 1 MiB read bound, or
UTF-8 validation. Those stay wherever the gate ends up (decision 0029, §4).
Evidence on the grant path must say exactly this: locality is
`trusted-not-proven` / `external-trust`; the read hardening still ran.

---

## Item 1 — Linux P1 proof: exact acceptance criteria

Given the canonicalized absolute candidate path P (component walk,
`O_NOFOLLOW`, fstat before/after already performed by the read path), the
Linux classifier proves P1 **only if every step below succeeds**. Any failure
is `Unproven` with a named reason — never an optimistic admission (decision
0029, §3: "Fail-closed is non-negotiable. Unclassifiable storage is refused,
never admitted on optimism").

**Step 1 — mount evidence.** Read `/proc/self/mountinfo`. The kernel documents
the field layout: mount ID, parent ID, `major:minor` device, mount root, mount
point, mount options, then after the `-` separator the filesystem type, mount
source, and super options — and requires parsers to ignore unrecognized
optional fields (source: `Documentation/filesystems/proc.rst`, §3.5,
https://www.kernel.org/doc/Documentation/filesystems/proc.rst).

Select the entry with the **longest mount-point prefix** of P. Record the
filesystem type, the mount source, and the `major:minor` device. Refuse
(`Unproven`, grant path) when the filesystem type is:

- a network protocol: `nfs`, `nfs4`, `cifs`, `smb3`/`smb`, `ncpfs`, `afp`,
  `ceph`; or
- any `fuse`-prefixed type (sshfs and every userspace filesystem — the kernel
  cannot observe the daemon's backing, so locality is unclassifiable from
  kernel evidence); or
- `9p` or `virtiofs` (host-shared into the guest; the backing is invisible to
  the guest — decision 0029 ruling 3).

**Step 2 — block-device identity.** The `major:minor` from mountinfo must equal
P's `st_dev` (consistency check; mismatch is `Unproven`). Resolve
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
  host-locality from guest-visible data.
- **Proven by this Work Order:**
  - `nvme*n*` — NVMe, PCIe-attached local media;
  - `sd*` on a local HBA (SATA/SAS via libata; the `<disk>/device` chain
    terminates in a local ATA/SAS host);
  - `mmcblk*` with `removable == 0` **and** `device/type` of `MMC` (eMMC) or
    `SD` — this incorporates the eMMC/SD P3 research argument (commit
    `c3ed1adcd36b18b2827f202d570edc7662504052`, research lane) by reference:
    welded eMMC or fixed SD that the kernel reports non-removable is
    admissible; `removable == 1` is grant-only (the circular-reasoning
    argument — the removable bit is the only kernel witness to physical
    fixity, so trusting a `1` to mean "fixed" would be the claim proving
    itself).
- **Not proven by this Work Order (grant path):** `dm-*` (device-mapper,
  including LUKS), `md*` (MD RAID), `loop*`, `nbd`, `rbd`, `drbd`, and any
  device the classifier cannot resolve. These are honest `Unproven`, not
  refusals of the concept — a later decision may widen the admission with its
  own evidence.
- **Fail closed:** any mountinfo or sysfs read/parse failure, any unresolvable
  device, any unrecognized layout → `Unknown` → `Unproven`
  (e.g. `p1_evidence_unavailable_v0`) → refuse. The read fails closed with the
  existing `FileReadError.unavailable` unless the operator grant is present.

The evidence bundle for a proven P1 records: the mountinfo fields (filesystem
type, mount source, `major:minor`), the sysfs disk path, the `removable`
value, and the driver/bus identity — one line per property P1–P4, in this
Work Order's format, supplying the end-to-end traceability decision 0029 §8
requires.

What this proof does **not** claim: it does not prove the absence of a
hypervisor — ruling 3 handles that by forcing the grant. It proves the
guest-visible storage stack terminates in locally-attached media.

## Item 2 — Windows: widen proof admission to fixed, non-removable eMMC/SD

The research-lane P3 argument for eMMC/SD is already recorded in commit
`c3ed1adcd36b18b2827f202d570edc7662504052` ("docs(research): eMMC/SD P3
argument"). This Item implements it; it does not re-derive it. The Work Order
incorporates that commit by reference as the evidence for this widening.

Acceptance criteria (all per `c3ed1ad`, §2–§7):

- In `crates/windows-drive-locality/src/lib.rs`, add `BUS_TYPE_SD` (12) and
  `BUS_TYPE_MMC` (13) to the admitted bus-type list alongside the existing
  ATA/SATA/NVMe admission (the gate at `lib.rs:204`). Nothing else in the
  gate changes.
- The `STORAGE_DEVICE_DESCRIPTOR.RemovableMedia` rejection stays exactly as
  is: fixed, non-removable eMMC/SD (`RemovableMedia` false) may be
  proof-admitted; removable SD, USB readers, unreadable removable status, and
  welded media still reported removable fall through to the **grant** — never
  proof (the circular-reasoning argument).
- Unchanged and re-pinned by tests: disk extents → disk numbers, the
  disk-number/dependency binding, the virtual-dependency walk (no virtual
  dependency in the chain), the before/after observation equality
  (`lib.rs:165`), and P4 via `is_ordinary_fixed_target` (`lib.rs:257`).
- The evidence bundle per `c3ed1ad` §8 is emitted for every classification:
  the bus/media class observed (enum value plus name), the removable-flag
  value and which query produced it, the device identity (disk numbers), the
  before/after observation and equality verdict, a one-line P1–P4 mapping, the
  classification with provenance, and the admitted bus list shown in the
  bundle so the next widening is visibly a decision.
- The classification provenance distinguishes `proven (P1–P4 evidenced)` from
  `trusted (operator grant, external-trust)` — the exact two labels from
  `c3ed1ad` §7, consistent with decision 0015.

## Item 3 — macOS: declared unprovable (grant only)

**Recommendation: declare macOS P1 unprovable; macOS file reads proceed only
under the operator grant.** The classifier on macOS returns `Unproven` with a
named reason (builder's choice, e.g. `macos_p1_unprovable_v0`); ordinary-file
and stable-identity checks (P2–P4) still run. This is the honest declaration
decision 0029 §6 explicitly permits ("an honest declaration of
unprovability").

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
- The grant path is the same mechanism on every platform (decision 0029,
  principle 3: no environment special cases), and CI's macOS runs are hosted
  runners — grant regardless. The cost of the honest declaration is one
  visible flag on the command line.

A future decision may admit a coarse macOS proof if its holes are closed;
that widening is a decision, not an implementation detail.

## Item 4 — the operator grant: exact spelling, no ambient construction, labelled evidence

- **Exact CLI spelling (pinned):** `--allow files.read=<path>` and the
  joined form `--allow=files.read=<path>`. These are the only spellings. Both
  exist today (the `--allow` / `--allow=` argv handling in `src/main.rs`
  feeding `OperatorGrantPolicy` in `src/operator_grant.rs`); this Item pins
  them as the grant's surface and forbids synonyms.
- **Per-invocation, per-path, command-line only** (decision 0029, ruling 4):
  the grant is constructed fresh from `argv` on each `hum run` invocation —
  today `OperatorGrantPolicy::allow_os`/`deny_os` in `src/operator_grant.rs`,
  fed from the `--allow` / `--allow=` argv handling in `src/main.rs` (~line
  1956). The granted path must exactly match the requested native path, and
  at most one distinct native `files.read=<path>` grant exists per
  invocation (exact duplicates are idempotent — today's
  `OperatorGrantPolicy` invariant); `--deny files.read` overrides allow
  (today's `exact_deny_overrides_allow_v0` precedence). **No environment
  variable, config file, or persistent setting may confer the grant.**
  Acceptance includes a negative test: setting a plausible env var (e.g.
  `HUM_ALLOW_FILES_READ`) must not grant anything, and the only construction
  site of the grant policy stays argv parsing (grep-level: `allow_os` is
  called only from argv handling).
- **Evidence labels (decision 0015 vocabulary):** the file-read authority
  event's locality classification is one of 0015's classes —
  - `proved` — P1–P4 evidenced by the platform proof, evidence bundle
    attached;
  - `external-trust` — admitted by operator grant; the evidence records
    `operator_allow_present: true`, the classifier's `Unproven` reason, and
    the grant facts: exact path, scope (`one_run`), lifetime
    (`exact_native_path`), and the literal human label below.
  - Human output must contain the literal string `trusted-not-proven`,
    unmissable — e.g. a stderr evidence line
    `files.read <path>: trusted-not-proven (operator grant, external-trust;
    classifier: <reason>)`. Exact line format is the builder's choice; the
    acceptance is the literal string.
  - JSON output must contain the classification value `external-trust` (field
    name builder's choice, value pinned) alongside the grant facts above.

## Item 5 — evidence surfaces (human + JSON)

Today the file-read authority event (`grant_scope`, `grant_lifetime`,
`locality_status`) is test-visible only: `RunReport.authority_events` is
consumed by tests and `hum run` renders nothing of it in ordinary output.
This Item adds the render surface; it does not assume one exists.

Acceptance: the file-read authority event — including the locality
classification label from Item 4 — is rendered in **both** the human and the
JSON run output. Exact UX (new stderr evidence section, extended `hum
evidence`, or a run-evidence flag) is the builder's choice; the pinned
content is the label strings from Item 4. Session AG (Item 6) pins both
surfaces, so this Item's acceptance is the AG test, not a screenshot.

## Item 6 — Session AG end state: byte-exact success under the grant, visibly labelled, both platforms

Replace the current pins: the Windows hosted-runner locality-refusal pin
(from the PR #30 follow-up) and the non-Windows native-path-unavailable pin
(from WO28 #7). The new assertions, on **both** Windows and Ubuntu:

- `hum run examples/tools/wordfreq.hum --allow stdout.write
  --allow=files.read=<fixture path> --args <fixture path>` exits 0, where the
  fixture path is spelled per-OS as the current pins do (repo-relative on
  Ubuntu, absolute on Windows) and the grant path exactly matches the
  requested path.
- Stdout is byte-exact `hum\nlang\nhum\n` (12 bytes) for
  `fixtures/wordfreq/sample.txt` (15 bytes: `hum␣␣lang\n\nhum\n`).
- The evidence is asserted on both surfaces: the human output contains the
  literal `trusted-not-proven`; the JSON output contains the `external-trust`
  classification with the grant facts. On CI both platforms' storage is
  unprovable (Azure runners are virtual/SCSI), so these assertions exercise
  the grant path, not the proof path — the proof paths are covered by unit
  tests with fixtures/mocks.
- The misuse pins stay: without the grant, the read fails closed with
  `FileReadError.denied` (the existing pin is not removed).

The grant flag sits in `check_all.ps1`'s Session AG invocation — a
version-controlled, reviewed file. That satisfies decision 0029 ruling 4's
"in CI it sits visibly in the workflow file": visible means reviewable, not
ambient.

## Resolution of WO28 #7's open semantic questions

WO28 #7's cold-start map (2026-09-25) stopped on four questions. This Work
Order answers all four; #7's STOP is lifted:

- **Q1 (what is the Linux P1 proof?):** Item 1 defines it exactly. #7 builds
  the fail-closed `P1 unproven` refusal against this specification.
- **Q2 (macOS scope):** grant only — Item 3.
- **Q3 (proof-vs-grant ordering):** the proof *definition* lives in WO29; #7
  is mechanical portability plus the honest refusal. Confirmed.
- **Q4 (does Session AG use the Ubuntu grant?):** yes — Item 6.

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
  it. If a platform's proof cannot be implemented as specified, the
  implementation reports that as a finding — it does not weaken the proof.

## Review and evidence requirements

- Independent pre-issuance review of this draft (Claude) before any PR.
- Honesty locks (decision 0014): no output text or doc may claim more than
  the implementation proves. Evidence labelled `proved` must carry the
  bundle; evidence labelled `trusted-not-proven` / `external-trust` must
  carry the grant facts and the classifier's `Unproven` reason.
- Every Item's acceptance criteria are asserted by tests, not by prose.
  Session letters continue the project odometer; Items are implemented in
  dependency order (1 → 4 → 5 → 2 → 3 → 6; Item 2 is platform-independent
  and may run parallel to 1).
