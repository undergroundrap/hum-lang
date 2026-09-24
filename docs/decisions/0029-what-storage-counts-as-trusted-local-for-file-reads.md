# Decision 0029 — What storage counts as trusted-local for file reads

Status: accepted 2026-09-24 (BDFL ruling on review at 5c51053), Option D.

> **Standing of this record.** This decision is *accepted*. It records the
> BDFL's ruling (§13) and the recommendation it adopted. The locality crate
> is a security policy and changes only by decision (see ledger entry 17).
> This record supersedes the 2026-09-23 draft on this branch.

## 1. The finding

Ledger entry 17 (pinned fact): WO28 #13 worked — the wordfreq APP entry now
runs past the full type-check gate on Windows and reaches the file read, and
then the read is refused with `FileReadError.unavailable` (exit 1, typed
`WordfreqError.read` chain on stderr). Cause:
`crates/windows-drive-locality/src/lib.rs:204` classifies a drive as
fixed-local only when its bus type is ATA, SATA, or NVMe; GitHub's Azure
runner disks are virtual/SCSI and network-backed, so the drive classifies
`DriveLocality::Unknown` → `NativePathLocality::Unclassified` and the
fixed-local-not-proven branch fires
(`fixed_local_v0_not_proven_before_candidate_access_v0` in `run.rs`). The
Session AG assertion pins this refusal. Consequence: wordfreq's success path
cannot be proven on hosted CI, and WO28 #13's done-condition was rewritten
honestly — "the app entry executes end-to-end through the type gate; the
byte-exact success read is blocked on hosted runners by the locality policy
(decision 0029 pending)".

## 2. The proxy problem

The bus-type list is a **Windows-specific proxy** for "local", not a
definition of it. Two failures follow, and both sit directly on Ocean's
stated goals — medical devices and game engines:

- **Embedded medical boards.** The allowlist names only ATA, SATA, NVMe.
  There is no SD/eMMC constant in the crate at all — unlisted bus types are
  refused. Storage on embedded medical boards is typically eMMC or SD:
  genuinely local, genuinely fixed, refused by the proxy.
- **VM/cloud build farms.** Game studios build on virtualized farms whose
  disks present as virtual/SCSI. The proxy refuses them exactly as it
  refuses Azure runners.

So the current rule is simultaneously **over-strict** (refuses storage that
*is* local and fixed) and **platform-bound** (it says nothing at all on
Linux/macOS, where `files_read_text` does not exist yet — ledger entry 7,
which this decision should shape). A bus-type list cannot be the definition
of trusted-local; at best it is one platform's evidence for a property the
list never states.

## 3. The security reasoning behind today's rule

This section exists so the rule's protections are not lost in a rewrite.
Condensed from the 2026-09-23 draft; the analysis is unchanged.

1. **Network-backed storage crosses a trust boundary.** `files_read_text`
   (Decision 0017) grants a program authority to treat file bytes as local
   fact. A network-backed file is served by another party: its content can
   be altered, withheld, or observed mid-read by someone outside the
   machine's trust boundary. Admitting it silently would smuggle an
   external-trust dependency (Decision 0015's vocabulary) into a grant the
   operator believes is local.
2. **TOCTOU, and what the before/after observation buys.** The Windows
   adapter inspects the drive, resolves the candidate path, and re-inspects
   before access. That double observation narrows the window in which the
   backing chain can change between classification and read, but it cannot
   *establish* every virtual-storage transition: it only sees the states at
   the two observation points.
3. **The threat-scoped TCB.** `fixed_local_v0` relies on a trusted kernel,
   trusted storage drivers, and a non-deceptive hypervisor. Virtual storage
   places the true backing outside guest observation — the guest's
   classification is a claim about something it cannot see. This is why
   virtual/SCSI classifies `Unknown` rather than `Local`: the adapter
   refuses to assert what it cannot observe.
4. **Fail-closed is non-negotiable.** Unclassifiable storage is refused,
   never admitted on optimism. Any option below that weakens this must say
   so explicitly and be judged on that basis.

## 4. The property a trusted read needs

*Design proposal (inference — see §11).* The rule should state the property,
and each platform should prove it its own way. A trusted-local read needs:

- **P1 — Not network-backed.** The bytes are served by the machine's own
  storage stack. No remote party can alter, withhold, or observe content
  mid-read. (This is the property the ATA/SATA/NVMe list was proxying for.)
- **P2 — Stable file identity for the duration of the read.** The file
  opened is the file read: the path still names the same object at read
  time, not a rebound name.
- **P3 — No mid-read substitution (TOCTOU).** The backing chain observed
  before the candidate access is the backing chain during it; nothing
  remaps or rebinds between inspection and read.
- **P4 — Ordinary file.** The target is an ordinary file: not a symlink,
  reparse point, device, FIFO, or pipe. (Already checked today:
  `is_ordinary_fixed_target` in
  `crates/windows-drive-locality/src/lib.rs:257` requires the resolved
  target to sit on `\Device\HarddiskVolume<N>` — an ordinary fixed volume,
  excluding reparse targets and non-file devices.)

Already in the design and noted for completeness (not locality properties):
the read is bounded (1 MiB cap), read-only, and single-file. They stay
wherever the gate ends up.

Bus-type lists are **evidence for P1 on one platform**, not the definition
of P1. The definition is what travels across platforms; the evidence is
what each platform supplies.

## 5. Governing principles

From the tasking; the options in §6 are judged against these:

1. **Property first.** Define what a trusted read needs (P1–P4, §4); let
   each platform prove it its own way. Bus-type lists are not the
   definition.
2. **Explicit and labelled where unprovable.** Where the property cannot be
   proven, trust is never silent: an operator grant, recorded in evidence
   as *trusted* (operator-attested) rather than *proven* (mechanically
   established), using Decision 0015's vocabulary
   (proved / external-trust / boundary).
3. **Hard rule: no environment special cases.** "If CI, skip the check" is
   rejected outright. It would make the evidence lie about what was
   established, and it is the specific-hack shape Decision 0027 exists to
   reject (§10).

## 6. Options

### Option A — Keep the physical-only proxy

The rule stays as it is: fixed-local requires ATA, SATA, or NVMe on
Windows; Linux/macOS remain without the capability.

- For: zero new mechanism; the existing security analysis applies
  unchanged; nothing new to audit.
- Against: fails both of Ocean's goal bars — embedded medical boards
  (eMMC/SD) and game-studio VM farms cannot read files at all; hosted CI
  cannot prove wordfreq's success path (the ledger #17 wall stands);
  ledger #7 (Linux/macOS) gets no shape from this decision.

### Option B — Property-based proof per platform

Redefine the gate as P1–P4. Each platform supplies its own proof; the
Windows adapter becomes *one proof of the property*, not the definition of
the rule. Concretely:

- **Windows:** the existing adapter (bus types + extents + dependency walk
  + before/after) is retained as a proven proof for ATA/SATA/NVMe. It can
  be *widened* to eMMC/SD with stated evidence, because eMMC is fixed local
  media — P1 holds. One wrinkle the proof must handle: SD can be removable
  media, and removable media can be swapped — a P3 concern. The proof must
  show fixed/non-removable, or the storage falls through to the grant path.
  Lane assignment (ruling): the eMMC/SD P3 argument is research-lane work —
  written as evidence in the implementing Work Order; the proof code is
  builder-lane.
- **Linux:** the proof is new work — mount/filesystem evidence plus block
  device identity. The hard question is virtio-blk: virtual-but-host-local.
  P1 holds *if* the host disk is local, but the guest cannot observe the
  host's disk. A guest-side proof of host-locality may be impossible in
  principle (the same observation limit as §3.3); if so, virtio-blk is
  unprovable from inside and falls through to the grant path. Ruling:
  virtio-blk and other guest-invisible backing are **always grant, never
  proof** — no guest-side argument can establish host-locality.
- **macOS:** the classification surface is coarse. Either a coarse proof
  exists or the platform honestly declares the property unprovable and the
  grant path is the only route.

Under B alone, unprovable storage stays refused — the hosted-CI wall stands,
honestly.

### Option C — Explicit operator grant for unprovable storage

The classifier stays strict (whatever A or B defines). A separate,
per-invocation operator grant admits storage the classifier cannot prove
local. The grant is recorded in the evidence bundle as **external-trust**
(Decision 0017's operator-consent leg; Decision 0015's vocabulary) —
*trusted, not proven* — and the label travels with the evidence. It is the
same mechanism on a developer laptop and on a CI runner (principle 3: no
environment special cases).

- For: unblocks every environment without weakening the classifier; the
  honesty machinery (0015 labels, evidence bundles) already exists to carry
  the distinction.
- Against: grant fatigue — operators rubber-stamp flags until the label is
  noise. Mitigation is the label itself: evidence that says
  "trusted-by-operator" cannot be mistaken for "proven-local" by anyone
  reading it later, including an auditor (§8). The grant must also be
  unmistakable at the CLI surface so it cannot be mistaken for a default.
- Grant acceptance criteria (ruling): the grant is **per-invocation and
  per-path, command-line only** — NO environment variable, config file, or
  persistent setting. In CI it therefore sits visibly in the workflow file
  where review sees it; there is no silent ambient way to enable it.

### Option D — B + C: proofs where provable, labelled grants where not

The property gate (B) admits what each platform can genuinely prove —
desktop NVMe, fixed eMMC, and any other storage with real evidence —
while the explicit grant (C) covers what cannot be proven: Azure's
network-backed disks, unclassifiable macOS storage, virtio-blk if
host-locality proves unobservable. No silent trust anywhere: every
admission is either proven (P1–P4 evidenced) or labelled trusted
(operator-attested, external-trust).

(The 2026-09-23 draft's "Option D: per-platform differences" is folded into
B — platform difference is the axis along which B is specified, not a
competing policy. The old Option E (labelled tier via grant) is this
record's Option C.)

## 7. Evaluation: options against environments

| Environment | A: proxy | B: property proofs | C: grant only | D: B + C |
|---|---|---|---|---|
| Embedded eMMC/SD (medical board) | Refused — the bar Ocean set fails | Admitted **by proof** (fixed media implies P1; proof must handle removable-SD P3) | Admitted **by grant**, labelled trusted | Proven where the proof holds; grant where it doesn't |
| VM/cloud build farm (game studio) | Refused — the bar fails | Admitted only if the platform can prove host-local backing; otherwise refused | Admitted by grant, labelled | Proof or labelled grant; no silent admission |
| Desktop NVMe (developer laptop) | Works (unchanged) | Works (unchanged — existing adapter is the proof) | Works (grant unnecessary) | Works |
| Hosted CI (Azure, network-backed) | Refused — wordfreq unprovable (ledger #17) | Refused — P1 unprovable, honestly | Admitted by grant, labelled trusted-not-proven | Labelled grant: CI can produce honest evidence |
| Linux/macOS (ledger #7) | No capability; no shape | **This decision defines each platform's proof or its honest unprovability** | Grant path available once the capability exists | Proof-first, grant as the fallback |

## 8. What a medical-device auditor would need

*External research applied by analogy (inference — see §11).* An
IEC 62304-style audit does not ask whether the software is clever; it asks
whether each claim is **traceable and its limits stated**:

- **A documented requirement.** The property P1–P4 (§4) is auditable; a
  bus-type list is not — no auditor can trace "SATA" to a safety claim, but
  "not network-backed, stable identity, no mid-read substitution" traces
  directly to the hazard it controls.
- **Traceability, end to end.** Requirement (this decision) → design (the
  per-platform proof) → implementation (the adapter code) → verification
  (tests, fixtures, CI evidence). Each link must exist on paper, not just
  in the author's head.
- **Risk analysis of the failure mode.** The hazard: network-backed storage
  mistaken for local implies untrusted bytes treated as trusted input (the
  Decision 0017 authority argument). In a device context the severity is
  concrete — misread calibration or configuration data acted on as local
  fact. The analysis must name this hazard and show which option controls
  it.
- **Stated limitations.** Auditors penalize unstated assumptions more than
  stated ones. The threat scope (§3.3: trusted kernel/drivers/hypervisor)
  and the live-migration limit (a VM can move mid-read — P3's boundary)
  must be written down as limitations, not discovered in an incident.
- **Proven vs trusted, distinguished.** The proved/trusted distinction
  (§5.2) is exactly the shape auditors expect: which claims are verified
  and which are assumed/attested must be separable in the evidence. An
  operator grant recorded as external-trust is an auditable assumption; a
  silent admission is an unauditable one.
- **Change control.** The existing practice — the locality crate changes
  only by decision (ledger entry 17) — is already the control an auditor
  wants to see; this decision should keep it explicit.

Under this lens, Option A is auditable but fails the product bar; Option C
alone is auditable only if the grant label is truly unmissable in evidence;
Option D gives the auditor both: proofs where they exist, labelled
attestations where they don't, limitations stated.

## 9. Performance notes (Decision 0024)

Honest finding: **performance does not discriminate between the options.**

- The locality check runs once per `hum run` invocation (per grant), not
  per byte: a handful of device queries on Windows, mount/block-device
  reads on Linux. Property-based proofs do not change that shape — the
  per-invocation cost stays negligible under every option.
- The 0024-relevant cost is **process, not compiler**: under A (and B
  alone), success-path evidence for wordfreq requires physical or
  self-hosted machines — a velocity cost paid in CI provisioning, not in
  check time. Under D, hosted CI can produce *labelled* (trusted, not
  proven) evidence — cheaper to gather, honestly labelled.
- Recorded so the ruling does not need to re-derive it: no option moves
  check time or CI wall-clock per run; the difference is where evidence
  *can* be gathered.

## 10. Decision 0027's general-vs-specific test

- **General.** The proxy fails independently of wordfreq: medical boards
  (eMMC/SD), game-studio VM farms, hosted CI, and all of Linux/macOS hit
  it. The property definition (§4) is the general fix — it states what
  every file-reading program on every platform needs.
- **Specific (rejected).** A CI carve-out — "if CI, skip the check" — would
  fix wordfreq's ledger entry and nothing else, while making the evidence
  lie (principle 3). Option C's grant is the same mechanism on a laptop
  and a runner; that sameness is what makes it general rather than a
  wordfreq-shaped hole.

## 11. Sources and standing

- **Pinned facts (repo, verifiable now):** `lib.rs:204` and the ATA/SATA/
  NVMe allowlist (`BUS_TYPE_ATA`/`SATA`/`NVME` constants; no SD constant
  exists in the file); the refusal chain
  `DriveLocality::Unknown` → `NativePathLocality::Unclassified` →
  `fixed_local_v0_not_proven_before_candidate_access_v0`; ledger entry 17's
  finding text; WO28 #13's rewritten done-condition; the Session AG
  refusal pin; Decision 0017's authority/TCB amendment; Decision 0015's
  classification vocabulary; Decision 0024's verification-speed clause;
  Decision 0027's general-vs-specific rule.
- **External research:** Azure hosted runners use virtual/SCSI,
  network-backed disks (Claude's tasking and ledger entry 17; widely
  documented); Windows `STORAGE_BUS_TYPE` values (SD/eMMC report outside
  the allowlisted set); IEC 62304 traceability and risk-management concepts
  (general software-lifecycle knowledge, applied here by analogy — not a
  clause-by-clause claim).
- **Inference (this record's proposals):** the P1–P4 property formulation;
  the platform proof sketches (Windows eMMC widening, Linux virtio-blk,
  macOS coarseness); the auditor-needs application in §8; the evaluation
  judgments in §7; the recommendation in §12.

## 12. Recommendation (adopted by the ruling, §13)

**Recommend Option D: property-based proofs per platform (B), with an
explicit, labelled operator grant (C) for storage the property cannot be
proven for.**

- It keeps everything §3 protects: fail-closed by default, no silent
  trust, the classifier strict.
- It fixes the proxy's over-strictness by *proof* rather than by exception:
  fixed eMMC/SD is admitted because P1 holds for it, evidenced — the same
  bar a medical-device auditor would ask for (§8).
- It gives hosted CI an honest path: Azure disks are network-backed, P1 is
  unprovable there, so CI evidence carries the trusted-not-proven label
  instead of either being impossible (A/B) or lying (environment
  special-case).
- It shapes ledger #7: Linux/macOS each get a proof to write or an honest
  declaration of unprovability, rather than inheriting a Windows bus list.
- The main risk is grant fatigue under C; it is controlled by making the
  label unmissable in evidence and the grant unmistakable at the CLI — both
  should be acceptance criteria on the implementing work order.

## 13. Ruling (2026-09-24)

**Option D is accepted: property-based proofs per platform (B), with an
explicit, labelled operator grant (C) for storage the property cannot be
proven for.** Pre-issuance review passed at 5c51053. The ruling answers the
open questions as follows:

1. **P4 added.** The target must be an ordinary file (not symlink, reparse
   point, device, FIFO, or pipe) — §4. Today's hardening already checks
   this (`is_ordinary_fixed_target`, `lib.rs:257`).
2. **eMMC/SD P3 argument is research-lane**, written as evidence in the
   implementing Work Order; the proof code is builder-lane.
3. **virtio-blk and other guest-invisible backing are always grant, never
   proof.** No guest-side argument can establish host-locality.
4. **Grant acceptance criteria:** per-invocation and per-path, command-line
   only — no environment variable, config file, or persistent setting — so
   in CI it sits visibly in the workflow file where review sees it.
5. **Labelled evidence satisfies WO28 #13.** The test proves wordfreq's
   behaviour; under a grant the only trusted element is storage locality,
   and the evidence must say so. Byte-exact output remains proven. This
   resolves the "decision 0029 pending" caveat in §1's done-condition.
