# Hum Work Order 29: decision 0029 implementation — per-platform locality proofs and the operator grant

Date: 2026-09-25
Status: DRAFT — pre-issuance review. Not active. WO30 is the active Work
Order; WO28 is closed. (The active-workorder marker comment is intentionally
absent from this file; it is added only when this Work Order is activated.)
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
Session AG to the trust-path byte-exact success on both platforms.

Queue: this Work Order is specified to execute **after WO30 closes**, once
activated; preparation of this draft proceeds now. WO28 #7 (portable unix
read mechanics) merged as `8b0afae` under the earlier ruling; this draft did
not govern #7. A draft Work Order changes nothing until it is activated;
the proof specification in Item 1 below is proposed for WO29 and is not in
force for #7.

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
- **Queue:** WO29 follows WO30 closure; preparation proceeds now.

### The read pipeline: classification → opened identity → enforcement → bounded read → evidence

Every file read under this Work Order traverses one ordered pipeline.
Authority ordering is preserved end to end; nothing below reorders, skips,
or weakens it:

1. **Authority/audit open** — the file-read authority event opens first, as
   today.
2. **Capability consent (0017)** — a matching `--allow files.read=<path>`
   is required before anything else is evaluated.
3. **Locality classification** — the platform proof (Items 1–3) runs; where
   it cannot prove, the operator attestation (Item 4) is consulted.
   Refusal here never opens the file. Failure is fail-closed (`Unproven`
   with a named reason), never optimistic.
4. **Component walk + open** — the existing hardening is unchanged:
   `symlink_metadata` per component, links never followed during the walk,
   `.`/`..` rejected, then `File::open` of the final component.
5. **Opened-identity binding** — the file's identity is the opened handle's
   own identity: fstat `(dev, ino)` on unix; the platform-defined device
   identity on Windows (disk numbers, per the existing `c3ed1ad` §8
   vocabulary — unix terminology is not imported onto Windows). The
   pre-read walked-vs-handle comparison
   (`opened_file_matches_walked_target`) stays as the P2/P3-walk check.
   Pathname equality is not a file-identity proof and is never used as
   one.
6. **Evidence-vs-object enforcement** — before any payload byte is
   consumed, the opened object's observed identity/device is checked
   against the identity/device recorded in the locality classification
   evidence from step 3. A mismatch rejects the read fail-closed (named
   reason, builder's choice, pinned by test) — it is never merely logged.
   Propagation without this check is not enforcement.
7. **Bounded read** — the 1 MiB bound and strict UTF-8 validation, unchanged.
8. **Evidence emission** — `AuthorityAuditEvent` carries the bound identity
   from step 5 together with the classification and the P1–P4 lines
   (Item 5). The bundle carries identity and classification evidence, not
   file contents; payload byte-exactness is pinned by the output
   assertions (Items 5–6), not by the bundle.

**Specified interface changes (not comments; none of this exists today):**
(a) the read entry point (`read_checked_unix_file`; the Windows read path
symmetrically) returns the opened identity bound to the bytes it read —
the identity is threaded through the return value, not re-derived at the
consumer and not documented-only; (b) the locality classifier returns the
observed device identity alongside the classification — on Windows this is
a new return-type change owned by `crates/windows-drive-locality` (the
classifier currently returns only a label, and the Windows reader does not
currently provide the opened identity; the audit-corrected plumbing is
specified here, not assumed). The walked-vs-handle comparison stays where
it is. P1–P4 are not weakened: classification still fails closed, and the
trust path still records the `Unproven` reason with the attestation facts.
These corrections require no new semantic decision; the remaining open
choices (reason strings, exit codes, field names) use this draft's
established builder's-choice-reviewed pattern.

**Acceptance — enforcement first, propagation second:** with injected
adapters through the real pipeline, (a) *enforcement* — the classification
evidence's device identity is made to disagree with the opened object;
the test asserts the pipeline REJECTS before payload consumption (no bytes
read, fail-closed refusal with the named reason). A run that logs the
substituted identity and proceeds fails this control; (b) *emitter
consumption* (separate, propagation only) — the threaded identity is
substituted before emission; the test asserts the emitted bundle carries
the substituted value, proving the emitter consumed the threaded value
rather than re-deriving it from the path. (b) does not prove enforcement;
(a) does.

### Platform effects of the shared attestation and rendering (Items 4–5)

Items 4 and 5 are platform-shared; their effects are stated here, separately
from the platform classifier changes (Items 1–3):

- **Windows:** the attestation admits reads on unprovable Windows storage
  (virtual/SCSI, hosted runners) with classification `external-trust`, the
  `trusted-not-proven` stderr literal, and `external-trust` in the JSON
  channel. Item 4 changes nothing about the Windows proof classifier — the
  ATA/SATA/NVMe admission, the before/after observation equality, and
  `is_ordinary_fixed_target` are Item 2's (and only Item 2's) concern.
- **macOS:** the classifier always returns `Unproven` (Item 3); the
  attestation is the only admission path. Rendering is identical to every
  other platform.
- **Rendering:** the stderr evidence line and the JSON channel behave
  identically on all platforms. Items 4–5 introduce no per-platform
  rendering differences and no new classifier behavior.

---

## Item 1 — Linux P1 proof: exact acceptance criteria

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
comment is refreshed). The Linux classifier proves P1 **only if every step below
succeeds**. Any failure
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
(`Unproven`, trust path) when the filesystem type is:

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
  - `nvme*n*` **and** the controller's `transport` attribute reads exactly
    `pcie`. NVMe-oF (tcp/rdma/fc) presents the same `nvme*n*` device nodes
    and is network storage, so the name pattern alone is not proof. For NVMe
    namespaces the disk's parent device IS the controller
    (`device_add_disk(ctrl->device, ...)` in the kernel's
    `drivers/nvme/host/core.c`), so read `<disk>/device/transport` directly
    — not the parent of the `device` symlink target. Source for the
    attribute: the stable sysfs ABI (`Documentation/ABI/stable/sysfs-nvme`:
    "transport: Shows the transport type string. Possible values: 'pcie',
    'tcp', 'rdma', 'fc', 'loop'"),
    https://docs.kernel.org/admin-guide/abi-stable.html. A missing or
    unreadable `transport` attribute is `Unproven` — fail closed. (This
    explicitly includes NVMe multipath heads: their disk is parented to the
    subsystem device, not a controller
    (`device_add_disk(&head->subsys->dev, ...)` in
    `drivers/nvme/host/multipath.c`), so they expose no `transport`
    attribute and are `Unproven`.)
  - `sd*` on a positively-identified local host bus adapter: walk the sysfs
    device chain from `<disk>/device` upward, collecting driver names from
    the `driver` symlinks. P1 is proven **iff** a driver in the chain is in
    the positive allowlist of local HBA drivers — seeded with the libata
    family (e.g. `ahci`, `ata_piix`) and named local SAS HBA drivers. The
    exact list is the builder's choice, reviewed at implementation; widening
    it is a decision, not an implementation detail. **Everything else is
    `Unproven`**, explicitly including virtio-scsi, iSCSI, Fibre Channel,
    USB mass storage, Hyper-V storvsc, and VMware PVSCSI.
  - `mmcblk*` with `removable == 0` **and** `device/type` of `MMC` (eMMC) or
    `SD` — this incorporates the eMMC/SD P3 research argument (commit
    `c3ed1adcd36b18b2827f202d570edc7662504052`, research lane) by reference:
    welded eMMC or fixed SD that the kernel reports non-removable is
    admissible; `removable == 1` is grant-only (the circular-reasoning
    argument — the removable bit is the only kernel witness to physical
    fixity, so trusting a `1` to mean "fixed" would be the claim proving
    itself).
- **Not proven by this Work Order (trust path):** `dm-*` (device-mapper,
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
value, the driver/bus identity, and the walked-vs-handle identity observation
(the `(dev, ino)` pair and the equality verdict) — one line per property
P1–P4, in this Work Order's format, supplying the end-to-end traceability
decision 0029 §8 requires.

What this proof does **not** claim: it does not prove the absence of a
hypervisor — ruling 3 handles that by forcing the grant. It proves the
guest-visible storage stack terminates in locally-attached media.

**Stated limitations** (decision 0029 §8: limitations written down, not
discovered in an incident): anonymous-device filesystems — overlay, tmpfs —
whose mountinfo device is `0:0` or otherwise does not resolve under
`/sys/dev/block` fail step 2 and are grant-only; btrfs spanning multiple
devices fails the single-device identity check and is grant-only.

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
  bundle so the next widening is visibly a decision. The bundle shape rides
  the shared Item 5 surface (available from Slice A); this Item widens
  admission, not the API.
- The classification provenance distinguishes `proven (P1–P4 evidenced)` from
  `trusted (operator grant, external-trust)` — the exact two labels from
  `c3ed1ad` §7, consistent with decision 0015.

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
  - `proved` — P1–P4 evidenced by the platform proof, evidence bundle
    attached;
  - `external-trust` — admitted by operator attestation; the evidence
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
  classification (`proved` | `external-trust`), the P1–P4 evidence lines on
  proof paths, and the attestation facts plus the classifier's `Unproven`
  reason on trust paths. Field names are the builder's choice; the values
  above are pinned.
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
    the literal `trusted-not-proven`. Proof path: the line carries the
    label `proved`. Exact surrounding format is the builder's choice; the
    pinned content is the label strings from Item 4.
- **JSON channel (new surface, authorized by this Item):** one new `hum run`
  output mode (flag name builder's choice; recommended `--format json`).
  In JSON mode stdout carries **only** the JSON envelope — no human outcome
  text, no fallback rendering, no replay of the human channel. The envelope
  embeds the program's output bytes byte-exactly (encoding builder's
  choice; the round-trip is byte-exact) and the `authority_events` array
  with the Item 4 values (`locality_classification`: `external-trust` on
  the trust path, `proved` on the proof path). Failure behavior: the
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

The harness correction rides with the slice that needs it — it is not
deferred:

- **Unix (with the Item 1 slice):** once the Linux proof exists, the no-grant
  refusal pin is hardware-dependent — on proof-capable Linux the proof can
  genuinely admit the read. The unix AG pin becomes provability-aware in the
  same slice: it branches once on the observed no-grant outcome. Exit 1 with
  `FileReadError.unavailable` asserts the refusal shape (unprovable
  storage); exit 0 asserts the proven-path shape. The native harness keeps
  two negative fixtures distinct, with no permission-changing fixtures: an
  OS-readable owned file with no Hum `--allow` → `FileReadError.denied`
  (Hum consent enforcement, independent of OS readability), and the same
  file with matching `--allow` on unprovable storage but no trust flag →
  `FileReadError.unavailable` with the classifier's `Unproven` reason
  (attestation missing). Missing consent and missing attestation are
  different refusals and are pinned separately. Where the pin branches on
  the observed hardware outcome, the exit-0 branch validates the required
  bound evidence, not just the label: byte-exact stdout **and** the
  evidence bundle carrying the fixture file's opened `(dev, ino)` identity,
  the `proved` classification, and the P1–P4 lines. The trust-flag run then
  asserts exit 0, byte-exact stdout, and the evidence label matching the
  observed admission class (`external-trust` where unprovable, `proved`
  where the proof outranks trust).
- **Windows (later slice, with Item 2):** the Windows classifier is unchanged
  until Item 2, so the Windows refusal pin stays valid and its flip waits.

The new assertions, on **both** Windows and Ubuntu (final state):

- `hum run examples/tools/wordfreq.hum --allow stdout.write
  --allow=files.read=<fixture path> --trust-locality
  files.read=<fixture path> --args <fixture path>` exits 0, where the
  fixture path is absolute and repo-root-joined on both platforms (as the
  current pins spell it) and the attested path exactly matches the
  requested path.
- Stdout is byte-exact `hum\nlang\nhum\n` (12 bytes) for
  `fixtures/wordfreq/sample.txt` (15 bytes).
- The evidence is asserted on both surfaces: the human output contains the
  literal `trusted-not-proven`; the JSON output contains the `external-trust`
  classification with the attestation facts. On CI both platforms' storage is
  unprovable (Azure runners are virtual/SCSI), so these assertions exercise
  the trust path, not the proof path — the proof paths are covered by unit
  tests with fixtures/mocks.
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

- **Q1 (what is the Linux P1 proof?):** Item 1 defines it exactly.
- **Q2 (macOS scope):** grant only — Item 3.
- **Q3 (proof-vs-grant ordering):** the proof *definition* belongs to WO29;
  #7 is mechanical portability plus the honest refusal.
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
  it. If a platform's proof cannot be implemented as specified, the
  implementation reports that as a finding — it does not weaken the proof.

## Review and evidence requirements

- Independent pre-issuance review of this draft (Codex) before acceptance
  and issuance. Draft PR publication for review is already authorized
  (PR #57) and is neither acceptance nor issuance.
- Honesty locks (decision 0014): no output text or doc may claim more than
  the implementation proves. Evidence labelled `proved` must carry the
  bundle; evidence labelled `trusted-not-proven` / `external-trust` must
  carry the grant facts and the classifier's `Unproven` reason.
- Every Item's acceptance criteria are asserted by tests, not by prose.
  Session letters continue the project odometer.

## Implementation slices

Dependency order 1 → 4 → 5 → 2 → 3 → 6, in three slices sized for one
review sitting each:

- **Slice A — Items 1 + 4 + 5 + the unix AG harness correction.** The Linux
  proof must not land without the operator recourse (Item 4), the surfaces
  that observe both paths (Item 5), and the provability-aware unix pin
  (Item 6's unix half) — otherwise the refusal pin is knowingly invalid on
  proof-capable Linux. Admission policy on Windows/macOS is unchanged in
  this slice, but Items 4–5 are platform-shared: the attestation, the
  stderr evidence line, and the JSON channel land on all platforms in
  Slice A, with platform checks covering the shared effects (Windows: the
  trust-path assertions on hosted runners; macOS: the grant-only path).
  The audit-corrected Windows plumbing also lands in Slice A: the
  classifier currently returns only a label and the Windows reader does
  not currently provide the opened identity, so the minimal future changes
  are specified here — `crates/windows-drive-locality` extends the
  classifier return to carry the observed device identity (disk numbers,
  per the existing `c3ed1ad` §8 vocabulary; no unix terminology imported),
  and the Windows read path captures and threads the opened handle's
  device identity for the enforcement check. Owners:
  `crates/windows-drive-locality` for the classifier return type;
  `src/file_read.rs` and `src/run.rs` for the threading, enforcement, and
  emission. Slice B widens only the *admitted bus list* (the gate at
  `lib.rs:204`). Plumbing availability and admission widening are separate
  changes. No new framework is introduced — the new controls ride the
  existing unit and CLI test files.
- **Slice B — Items 2 + 3.** Windows SD/MMC widening; macOS declared
  unproven. Platform-gated; no CLI or rendering changes.
- **Slice C — Item 6 remainder.** The Windows AG flip to the trust-path
  assertions (the Windows classifier is unchanged until Slice B, so its
  refusal pin stays valid meanwhile).

Item 2 is platform-independent and may run parallel to Slice A.

## Affected-file scope (complete)

New: `crates/linux-drive-locality/` (manifest, `src/lib.rs`, fixture trees
under `tests/`); `tests/cli_wo29_trust_locality.rs`.

Modified: `Cargo.toml` (workspace members, dependency); `Cargo.lock` (the new
workspace member; mechanical — no version changes beyond what the new member
requires); `src/native_path.rs` (unix classification seam and the
proven-local label); `src/operator_grant.rs` (the attestation field, setter,
accessors); `src/main.rs` (argv arms, run-only gating, usage); `src/run.rs`
(gate order, `AuthorityAuditEvent` bundle fields, the threaded opened
identity consumed at emission, the evidence-vs-object enforcement check,
render paths); `src/file_read.rs` (the read entry point's return interface
— opened identity bound to the returned bytes — and the stale P1-unproven
doc comment; the walk/open/identity mechanics are unchanged);
`crates/windows-drive-locality` (classifier return type carries the observed
device identity — Slice A; the bus-list admission gate — Slice B);
`tools/check_all.ps1` (Session AG pins: unix correction in Slice A, Windows
flip in Slice C);
`tools/test_ci_policy.ps1` (mechanically affected: the SHA-256 pins over any
edited `check_all.ps1` function bodies are recomputed by the builder and
verified at review; no policy change).

Untouched: `docs/` (research lane), all fixtures.

## Focused acceptance plan

- **Crate unit tests** (`linux-drive-locality`, fixture-driven): mountinfo
  longest-prefix selection and optional-field tolerance; the refuse-by-fstype
  matrix (nfs/nfs4/cifs/smb/fuse/9p/virtiofs/overlay); the sysfs matrix
  (nvme+pcie proven; nvme+tcp/rdma/fc/loop, multipath heads, and missing
  transport refused; sd+ahci/ata_piix proven; virtio-scsi/iscsi/usb refused;
  mmcblk non-removable MMC/SD proven, removable refused; vd*/xvd*/storvsc
  refused; dm/md/loop refused; st_dev mismatch refused; unreadable sysfs
  refused). Every `Unproven` pins its exact reason string.
- **Policy/CLI/gate tests** (injected adapters, no host): the attestation
  setter accepts `files.read=<abs path>`, rejects empty/non-`files.read`
  payloads and second distinct paths, exact duplicates idempotent, no host
  access on rejection; space form, joined form, missing-value error,
  non-`run` rejection, post-`--args` ordering; the gate admits on proof
  (`proved`) and on attestation (`external-trust`), refuses otherwise with
  the adapter never called; `HUM_TRUST_LOCALITY` set with no flag still
  refuses.
- **CLI tests** (Linux VM, overlay/tmpfs → deterministically `Unproven`):
  the trust-path run exits 0 with byte-exact stdout, the `trusted-not-proven`
  stderr literal, and `external-trust` in the JSON channel; `--allow` alone
  refuses; mismatched attestation/allow paths refuse.
- **Harness (Slice A):** the unix AG no-grant pin branches on the observed
  outcome (refusal shape vs proven-path shape); the exit-0 branch validates
  the bound evidence bundle per Item 6; the two negative fixtures (missing
  Hum consent vs missing attestation, no permission changes) are pinned
  separately; the enforcement control proves mismatched evidence rejects
  before payload consumption and the emitter-consumption control proves
  propagation; the trust-flag run asserts exit 0, byte-exact stdout, and
  the evidence label matching the observed admission class.
  **Harness (Slice C):** the Windows AG flip.
- **Fixture evidence vs native hardware proof:** everything above is
  fixture-testable and deterministic. Native hardware proof — a real
  `Proven` admission on NVMe/PCIe, SATA/AHCI, and eMMC iron — needs
  bare-metal machines and is not fixture-provable; CI runners exercise only
  the trust path. The NVMe-oF negative is unit-pinned
  (`transport=tcp → Unproven`) but not hardware-observed end-to-end; that
  residual is stated, not solved. If any accepted property cannot be met as
  specified, the implementation reports that concrete finding — it does not
  invent a replacement policy.
