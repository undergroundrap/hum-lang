# eMMC/SD P3 argument — Work Order evidence for decision 0029

Date: 2026-09-24. Lane: research.
Assignment (decision 0029 §13 Q2, pinned): this argument is research-lane
work, written as evidence in the implementing Work Order; the proof code is
builder-lane. This note *is* that evidence.

**Claim.** Fixed, non-removable eMMC/SD storage satisfies decision 0029's
P1–P4 and may be admitted by proof (Option B). Removable media cannot be
admitted by proof; it falls to the operator grant (Option C). The dividing
line is not the bus — it is the *removable* flag, and the load-bearing
property is P3.

Sections are labelled by standing: **(pinned)** = verifiable in the repo
now; **(external)** = verifiable in public documentation/standards;
**(inference)** = this note's argument. The recommended reading of a claim
follows its label.

## 1. What the current proof already observes (pinned)

The Windows adapter (`crates/windows-drive-locality/src/lib.rs`) already
gates on removability before bus type:

- `DiskObservation` carries `removable: bool` (`:102`), read from
  `STORAGE_DEVICE_DESCRIPTOR.removable_media` (`:640, :659–664, :771`).
- Classification refuses a disk when `disk.removable ||
  !matches!(disk.bus_type, BUS_TYPE_ATA | BUS_TYPE_SATA | BUS_TYPE_NVME)`
  (`:204`). Removability is already a first-class input.
- The bus constants stop at `BUS_TYPE_NVME` (`:310`); there is no
  `BUS_TYPE_SD` (12) or `BUS_TYPE_MMC` (13) constant anywhere in the file.
  eMMC/SD is refused today purely because the bus value is unlisted.
- The before/after double observation survives classification: if
  `evidence.before != evidence.after`, the drive goes `Unknown` (`:168`).
- P4 is media-agnostic: `is_ordinary_fixed_target` requires the resolved
  target on `\Device\HarddiskVolume<N>` (`:256`), which applies to eMMC
  user volumes identically to SATA ones.

So the eMMC/SD widening is exactly: add the two bus constants to the
admission list *without weakening the `disk.removable` rejection*.
Everything else in the adapter — extents, disk-number binding, dependency
walk, before/after — is unchanged. That smallness is a feature: the proof
changes in one place, reviewable under the existing "locality crate changes
only by decision" control.

## 2. P1 — not network-backed (inference)

eMMC/SD sits behind the machine's own storage stack (SDHCI/eMMC host
controller, kernel block layer). No remote party serves, alters, or observes
the bytes mid-read. The P1 argument is the same one the ATA/SATA/NVMe list
was proxying for — it is not media-specific.

Caveat (inference): an SD card behind a network-attached reader
(USB-over-IP, iSCSI-backed virtual card) would fail P1 — but such a setup
presents as `BusTypeUsb`/virtual with `removable` set, so it cannot pass
the observable gates in §5. P1 is enforced through those gates, not by
arguing about the card.

## 3. P2 — stable file identity (inference)

The existing proof pattern applies unchanged: resolve the candidate path,
open it, and require the identity to be stable across the before/after
observations. The media-specific hazard is a card swap between open and
read: the name still resolves, the object is different. That hazard is
exactly what the removability gate excludes (see §4), which is why P2 and
P3 share the same load-bearing fact.

## 4. P3 — no mid-read substitution: the core (inference)

The hazard is physical: a removable SD card ejected and replaced (with a
different card, or the same card rewritten elsewhere) between the two
observations. The backing chain observed before access is then not the
backing chain during it, and the evidence's before/after equality is
meaningless — it compared two states of the same *name*, not the same
*media*.

**eMMC.** eMMC is soldered to the board (external: JEDEC JESD84); physical
swap requires rework, which is not a runtime vector. With the physical swap
vector removed, P3 reduces to the software surface: block-device identity,
volume binding, and the before/after double observation — the same threat
shape as a fixed SATA drive. The existing `:168` check is the P3 mechanism;
eMMC needs no new one.

**Removable SD.** Here the swap vector is real and *no OS-level observation
can close it*: the two observations are only of the same card if the card
hasn't been swapped, which is the claim the observations were supposed to
establish. The reasoning is circular, so removable media is **unprovable**
and must take the operator grant — even if the card is present and quiet
at both observation points.

**The fixed/removable distinction is the load-bearing fact.** "Fixed" must
mean both: the media class is eMMC (soldered by construction) or an SD
class the OS reports non-removable, *and* the platform's removable flag
(`removable_media = 0` on Windows, `/sys/block/*/removable = 0` on Linux)
says so. Either leg failing means grant, never proof.

**Welded-card honesty (inference).** A medical device may weld or glue an
SD card into its slot — physically fixed, but the OS still reports
`removable_media = 1` because the descriptor describes the device's design,
not this unit's assembly. That fixity is invisible to the OS and therefore
unprovable by any platform observation; it takes the grant. This is the
right call: the proof's bar is *OS-observable* non-removability, not
physical fact the machine cannot see.

## 5. What the platform can observe (pinned + external)

**Windows (pinned: read path exists; external: the enum values).**

- `STORAGE_DEVICE_DESCRIPTOR.BusType` — the proof needs `BusTypeSd` (12)
  and `BusTypeMmc` (13) admitted (external: WinDDK `STORAGE_BUS_TYPE`;
  absent from the crate today, §1).
- `STORAGE_DEVICE_DESCRIPTOR.RemovableMedia` — must be 0. The crate
  already reads and gates on it (`:640, :659–664, :771, :204`); the
  widening keeps that gate intact.
- Unchanged remainder: volume extents → disk numbers, dependency walk
  (no virtual dependency), before/after equality (`:168`).

An SD card in a USB reader presents as `BusTypeUsb` with removable set —
it cannot satisfy these gates and falls to the grant (see §7).

**Linux (external: sysfs ABI; the proof itself is future ledger-#7 work,
but the observable surface is what this note must name).**

- `/sys/block/mmcblk*/device/type` — the device class: `MMC` (eMMC) vs
  `SD`; distinguishes the media class the same way `BusType` does.
- `/sys/block/mmcblk*/removable` — must be `0` for proof; `1` means
  grant. This is the Linux leg of the load-bearing fact.
- `/sys/block/mmcblk*/device/name`, `cid`, `manfid` — stable device
  identity for the before/after comparison, analogous to the Windows
  disk-number binding.
- sysfs is kernel-reported: the proof inherits the trusted-kernel leg of
  the threat-scoped TCB (§6).

## 6. Threat scope (pinned + inference)

The proof inherits decision 0029 §3.3's threat-scoped TCB unchanged:
trusted kernel, trusted storage drivers, non-deceptive hypervisor. The
media widening adds two stated limits:

1. **Firmware reports hardware truthfully (inference).** A host that
   reports `removable_media = 0` for a removable SD slot is lying, and no
   guest-side check can catch it. This is not a new hole: a deceptive
   platform defeats *every* local proof, including today's ATA list. It is
   a stated limitation of the TCB, not a flaw in the eMMC/SD argument.
2. **Physical access is out of scope (inference).** Hands-on-the-board
   defeats any storage-locality claim — desoldering a NAND chip beats
   SATA's locality exactly as it beats eMMC's. Same bar as today.
3. **No live-migration claim (inference).** The target is bare-metal
   embedded boards. A VM whose backing is eMMC is guest-invisible backing
   and stays on the grant path (§7; the §13 Q3 ruling for virtio-blk
   applies by analogy).

Fail-closed is preserved: anything the proof cannot observe classifies
`Unknown` and is refused, exactly as today.

## 7. What stays an operator grant (inference)

- Removable SD (`removable_media = 1` / sysfs `removable = 1`) —
  **always grant, never proof** (§4: circular reasoning).
- SD behind a USB reader (`BusTypeUsb`) — removable by construction;
  grant.
- Any storage where the removable flag cannot be read — `Unknown` in the
  classifier, grant path in the Work Order.
- Physically welded/fixed cards the OS still reports as removable —
  grant (unobservable fixity, §4).
- VM guests with guest-invisible backing — always grant, never proof
  (§13 Q3).

## 8. Evidence the builder's proof code must emit (inference)

The evidence bundle for an eMMC/SD admission must contain, so the
proven/trusted distinction (§5.2 of the decision) stays auditable:

- **Bus/media class observed:** enum value and name (`BusTypeMmc`/`BusTypeSd`
  on Windows; `device/type` on Linux).
- **Removable flag value:** `0`, from which platform query.
- **Device identity:** Windows disk number(s) from extents, dependency
  observation; Linux `mmcblk` device `name`/`cid`/`manfid`.
- **Before/after observations and their equality verdict** — the P3
  evidence.
- **P1–P4 mapping:** one line per property naming the evidence that
  establishes it (P4: the `\Device\HarddiskVolume<N>` check result).
- **Classification with provenance:** `proven (P1–P4 evidenced)` vs
  `trusted (operator grant, external-trust)`. The bundle must show which
  bus types the proof admitted and the `removable = 0` gate, so the next
  person to widen the list knows it is a decision — not a constant.

## 9. Sources and standing

- **Pinned (repo, verifiable now):** `crates/windows-drive-locality/
  src/lib.rs` — `:102` removable field, `:204` the removable-or-bus gate,
  `:640/:659–664/:771` the `removable_media` read, `:168` before/after,
  `:256` `is_ordinary_fixed_target`; the absence of any `BUS_TYPE_SD`/
  `BUS_TYPE_MMC` constant; decision 0029 §§1–13 (accepted, Option D).
- **External:** Windows `STORAGE_BUS_TYPE` values and `RemovableMedia`
  semantics (WinDDK/MSDN); JEDEC JESD84 eMMC soldered construction;
  Linux sysfs `mmcblk` attributes (`device/type`, `removable`, `cid`,
  `manfid`); IEC 62304 traceability concepts (applied by analogy, as in
  the decision's §8).
- **Inference (this note's):** the P1/P2/P3 application to eMMC/SD; the
  circular-reasoning claim for removable media; the welded-card grant
  rule; the threat-scope limits; the evidence-bundle schema in §8.
