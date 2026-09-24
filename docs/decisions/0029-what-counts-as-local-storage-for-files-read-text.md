# 0029: What counts as "local" storage for `files_read_text`

Date: 2026-09-23
Status: proposed 2026-09-23. BDFL rules and merges.

## Context

`files_read_text(path: Path) -> Result Text, FileReadError` may read only
after the authority and locality gates. On Windows, Session AC's isolated
bootstrap adapter narrows an internal status to threat-scoped
`fixed_local_v0` only when stable drive/mapping observations, an empty
storage dependency result, a complete bounded extent list, and non-removable
ATA/SATA/NVMe descriptors for every backing disk agree — observed
identically before and after inspection. The adapter opens only synthesized
volume/disk device names with desired access zero and never opens the
candidate path. Unsupported or unclassified locality fails closed as
`FileReadError.unavailable`. (Decision 0017, 2026-07-11 amendment;
`docs/LANGUAGE_REFERENCE.md`.)

The code is explicit about what it refuses. `crates/windows-drive-locality`
enumerates every Windows bus type and admits exactly three
(`lib.rs:204`):

- admitted: ATA, SATA, NVMe
- refused: SCSI, SAS, Fibre Channel, iSCSI, RAID, Storage Spaces,
  file-backed virtual, NVMe-over-Fabrics — and, by name, VIRTUAL

GitHub's hosted runners are Azure VMs: virtual/SCSI, network-backed disks.
The chain fails closed there, so wordfreq's success path — the actual file
read — cannot be proven in hosted CI. Only the typed fail-closed behavior
(`unavailable`) is provable. Anyone running Hum in a VM, a cloud dev box, or
hosted CI hits the same wall.

This is live, not theoretical. WO28's done-condition is "wordfreq runs
end-to-end on Windows" (friction ledger #13). On hosted runners that
condition is unsatisfiable even after the loop-variable fix lands: the gate
that blocks today would give way to `unavailable` tomorrow. 0029's answer
determines whether WO28 can be satisfied in hosted CI at all, or whether it
needs a physical Windows machine.

The same question decides friction ledger #7: `files_read_text` is
Windows-only today, and Linux/macOS need their own locality classifier. What
"local" means must be answered per platform, and the answers may
legitimately differ.

This record lays out the options with the security reasoning behind today's
rule. It does not choose — Ocean rules as BDFL; Claude reviews.

## Why "fixed local" matters

Three reasons, in increasing order of how much they hurt to weaken.

**1. Network-backed storage crosses a trust boundary.** The `files.read`
grant is an exact one-file authority: this program may read this file, this
run. The read assumes the bytes sit under the machine's own storage stack.
A network-backed file — SMB mapping, cloud-synced drive, fabric-attached
volume — is served by another party over a channel the machine does not
control. Its content can change, disappear, or be served adversarially
between the grant and the read. "Local file authority" must not be a label
anyone can put on a network read. This is the property the 0017
consequences section names directly: hidden network/device mappings must
never be mistaken for local file authority.

**2. TOCTOU: the backing chain can change mid-inspection.** The adapter
observes the drive type and mapping, inspects extents and backing disks,
closes every handle, then observes the drive type and mapping again — and
only narrows when both observations agree. That before/after pair is the
anti-TOCTOU mechanism: it catches a backing chain that changed while it was
being inspected. Virtual and network-backed disks are exactly the disks most
susceptible to detach, snapshot, live migration, or re-mapping. Any rule
that admits them leans harder on the double observation, not less — and the
record should state its limit honestly: nothing in the adapter detects a
live migration that completes between the two observations.

**3. Trust: the TCB is stated, and virtual disks sit inside it.**
`fixed_local_v0` is explicitly threat-scoped: it holds under a trusted
Windows kernel, trusted storage drivers, and a non-deceptive hypervisor, and
it is not proof against a malicious or deceptive member of that TCB. A
virtual disk puts an abstracting layer — the hypervisor — inside the TCB by
construction: the guest cannot observe the true backing store, so "no
network layer" becomes a claim the guest takes on trust rather than a claim
it verifies. Admitting virtual storage does not just widen a bus-type check;
it weakens the stated TCB assumption. The record must say so out loud, not
smuggle it in.

Non-negotiable across all options: fail-closed is preserved. Unknown,
partial, failed, changing, unsupported, or unknown evidence stays
`locality_unclassified` and the read stays refused. No option turns "could
not tell" into "allow".

## Option A: Keep physical-only

The threat model is the point. `fixed_local_v0` is Hum's one strong,
auditable locality claim — a small adapter, a bounded query chain, a stated
TCB. Diluting it for CI convenience trades the trust property for evidence
convenience, and evidence convenience is the one currency Hum never spends.

The cost, stated plainly: wordfreq's success path cannot be proven in hosted
CI; WO28's done-condition needs a physical Windows machine (self-hosted
runner) or stays unsatisfiable; every VM, cloud dev box, and hosted-CI user
gets `unavailable` on the success path forever.

0027's question cuts both ways here. "Provable in hosted CI" is a general
requirement — every file-reading program wants it, not just wordfreq — so A
is not wordfreq-specific stubbornness. But the security rule is also
general, and A is the only option that changes nothing about it.

## Option B: Admit virtual/SCSI as a distinct, labelled trust level

Admit a second property, e.g. `virtual_local_v0`: `GetDriveTypeW` Fixed,
non-removable, no network/fabric dependency, no file-backed-virtual layer,
complete nonempty extent list, identical facts before and after — but the
backing disk reports VIRTUAL or SCSI, or a VHD dependency exists. The read
proceeds; the evidence records the weaker property by its exact name.

The label must travel. Locality is a pre-read gate, not a value, so the
program-observable surface (`Result Text, FileReadError`) does not change —
but two runs of one program can then carry different trust properties with
identical program behavior. The honesty requirement lands on the evidence
and the grant, which is where Hum already puts authority (0017: source
maximum, reachable task closure, exact operator consent). The decision must
pin the label's exact meaning and forbid eliding it downstream: "virtual
local" may never be summarized as "local".

0015's vocabulary already names this weakening. `external-trust` means
"depends on data, code, or authority outside Hum's proof" — and a virtual
disk's backing store is outside the guest's proof by construction. The
record can say it directly: virtual-local reads are external-trust in 0015's
sense. That is not a new concept; it is the existing one, applied.

TOCTOU under B: the double observation is load-bearing, and the record
should name live migration between the observations as a known limit
(see above). B does not get to quietly inherit A's threat scope; it states
its own: trusted kernel, trusted drivers, trusted hypervisor — with the
hypervisor now explicitly trusted rather than assumed non-deceptive.

Risk: label erosion. Every downstream claim — evidence summaries, program
READMEs, bake-off entries — will want to shorten "virtual-local" to
"local". The decision can forbid it, but only review discipline enforces it.

## Option C: Explicit operator grant flag for non-physical storage

Keep the classifier exactly as strict as today. Add an explicit,
operator-acknowledged widening — a per-invocation flag on the `hum run`
file grant — where the human operator accepts the weaker locality for that
run.

The rationale is 0017's own authority model: "source maximum, reachable
task closure, and exact operator consent." The operator is already in the
TCB for the grant itself; the grant is meaningless without their consent.
Letting the operator explicitly widen the trust is consistent with that
model — the weakening is visible, auditable, per-invocation, and never the
default. C preserves `fixed_local_v0`'s meaning byte-for-byte; nothing about
the classifier or its threat scope changes.

Difference from B: B changes what the classifier accepts by default; C
keeps the default strict and puts the widening in the operator's hands. The
evidence then records "proven under operator-acknowledged non-physical
storage" — honest, and WO28's done-condition becomes satisfiable in hosted
CI (the workflow passes the flag explicitly) without weakening the default
for anyone else.

Open mechanics: the flag must be part of the evidence receipt, since a
program's proof then depends on invocation flags. And the flag's own
semantics need pinning — does it admit virtual/SCSI only, or everything the
classifier refuses (network, removable)? The narrow form (virtual only) is
the one this context motivates; the wide form reopens the network question
that reason 1 above closes.

## Option D: Per-platform differences

Ledger #7 needs a Linux/macOS classifier regardless, and 0029's answer
shapes it. The platform analogues:

- Linux: mount filesystem type (`/proc/mounts` — ext4/xfs/btrfs/tmpfs vs
  nfs/smb/cifs/fuse) and block-device virtuality (`/sys/block/*` —
  virtio-blk vs physical). Note the interesting case virtio-blk raises: it
  is virtual but host-local. Is that "local"? The B-vs-C question replays
  per platform, and Linux can draw the virtual-but-local /
  virtual-and-remote distinction that Windows's bus-type check cannot.
- macOS: no direct bus-type API for the adapter's purposes; statfs plus
  IOKit. The classification will be coarser. The decision should say
  whether coarser is acceptable or macOS stays `unavailable` until a bounded
  adapter exists — "coarse but bounded" vs "nothing until exact".

D is not an alternative to A/B/C; it is the axis along which A/B/C must
each be specified. A physical-only rule on Linux would exclude most cloud
VMs (virtio) — a harsher consequence than on Windows, where physical
hardware is the common dev machine. The ruling should say, per platform,
which option holds, rather than pretending one paragraph covers all three.

## Option E: B + C — labelled tier reachable only via explicit operator grant

The classifier learns to produce `virtual_local_v0`, but Session AD's gate
accepts only `fixed_local_v0` unless the operator passed the explicit flag.
Default-deny, explicit-allow.

This keeps A's strict default, names the weakening honestly with B's label,
and puts the decision with the operator per C's consent model. Hosted CI
passes the flag; evidence records both the label and the flag. Nothing is
smuggled: the default path is exactly today's rule, and the widened path is
visible in the invocation, the evidence, and the trust vocabulary.

Cost: two mechanisms where one might do. 0027's default-to-no asks whether
the complexity is justified — or whether C alone is enough, with the
evidence recording the flag and no new trust level named. E's answer is that
the label matters because the flag alone doesn't distinguish "operator
accepted virtual" from "operator accepted anything"; but if the flag is
pinned narrow (virtual/SCSI only), that distinction may already be carried.

## Open questions for the ruling

1. A vs E: is hosted-CI provability worth any weakening of the locality
   rule, or does the threat model hold the line and WO28 takes a
   self-hosted runner?
2. If any weakening: B's standing label vs C's per-invocation flag vs E's
   combination — which mechanism carries the honesty requirement?
3. Does the program-observable surface change at all (a new `FileReadError`
   variant, a visible trust level), or does all of this live in evidence
   and grants?
4. Per-platform (D): does Linux get the same rule — knowing physical-only
   excludes most cloud VMs — and does macOS get a coarse classifier or
   stay `unavailable`?
5. TOCTOU: is the before/after observation sufficient for virtual backing,
   or must the record state live migration between the observations as a
   known limit of any admitted-virtual option?
