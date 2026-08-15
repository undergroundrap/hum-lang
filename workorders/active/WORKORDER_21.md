# Hum Work Order 21: Organize Active And Closed Work Orders

Date: 2026-08-13
<!-- hum-active-workorder:v1 -->
Status: Work Order 21 planning, Unit A, and Unit B are independently accepted,
committed, published, status-recorded, and terminal-green. The first Unit C
harness-recovery amendment and its publication-status lifecycle are also
terminal-green. Unit C remains unaccepted, and its clean and rejected candidate
stashes remain separately preserved and unchanged.

The Unit C evidence-transport redesign amendment was independently accepted.
It was committed as `2619184469dc43748a42695a80866d0bbd9df1c6`, with parent
`7877e346ffc86418a14a06e5bf00aa7741311eae`, subject
`docs(workorder): redesign unit c evidence transport`, sole path
`workorders/active/WORKORDER_21.md`, blob
`b814d8d41717d6b9bebcc1d9d1f3fe4854b040c9`, and statistics `+952/-40`. It was
published by one normal non-force `main`-only fast-forward.

Workflow `ci`, run `31909076659`, attempt 1, tested exact SHA
`2619184469dc43748a42695a80866d0bbd9df1c6` and concluded `success`. Ubuntu job
`95071313852` and Windows job `95071313802` both selected the exact binding
`mode=full;reason=no_status_transition;anchor=;run_id=0;run_attempt=0;ubuntu_job_id=0;windows_job_id=0;transitions=`.
Both passed the required full preflight, exact 151-invocation and
151-unique-name deterministic classifier inventory, Unit A four-fast/24-full
accounting, binary-safe A11 and separate visible duplicate-marker evidence,
canonical diagnostic-doctrine evidence, text hygiene and public readiness for
533 files, alpha claims, release readiness 0.0.1, and all platform obligations.
Ubuntu passed the 14,226-pair platform-independent Exhaustive producer; Windows
correctly skipped only its duplicate. Both correctly skipped status-only
evidence.

The publication-status record and its exact local commit are the sole current
lifecycle step. Publication of that status commit, terminal-green fast CI, a
fresh explicit BDFL Unit C redesign-implementation signal, clean-stash
restoration, implementation, Fast, independent review, acceptance, closeout,
WO22, broad harness consolidation, semantic-coordinate research, and compiler
work all remain separately gated and unauthorized.
Owner: BDFL (Ocean).

## Durable-consumer audit

The migration design begins with the complete repository audit because Work
Order placement is part of Hum's CI control plane, not cosmetic file cleanup.
The author inspected the published tree at
`b703347996b6803f0d77bc8f0142e26333f0d9af`, including hidden repository
paths, all root Work Orders, the active-marker literal, root/active/closed Work
Order language, and the identifiers `WorkOrderPath`, `ActiveWorkOrder`, and
`ActivePath`.

The audit used repository-wide searches for:

- `WORKORDER*.md` and exact numbered variants;
- `<!-- hum-active-workorder:v1 -->`;
- root, active, and closed Work Order wording;
- `WorkOrderPath`, `ActiveWorkOrder`, and `ActivePath`;
- classifier candidate and Work-Order-like regexes;
- all callers of the status-boundary classifier and its permanent suite; and
- source metadata that names a Work Order as durable doctrine.

The resulting consumer inventory is frozen as follows.

| Consumer | Existing contract | Required unit | Ruling |
| --- | --- | --- | --- |
| `AGENTS.md` | Defines the active Work Order as the sole root `WORKORDER*.md` carrying the marker; cold-start and routine status closure also say root | Unit B | Load-bearing policy consumer; must change with the repository migration |
| `docs/GOVERNANCE.md` | Describes the active Work Order and lifecycle generically, without a root path or filename grammar | inspection only | No edit authorized or required |
| `docs/OCB_METHOD.md` | Contains historical Work Order references and process lessons, not live placement rules | inspection only | No edit authorized or required |
| `README.md` | Has no Work Order path or active-marker control-plane contract | inspection only | No edit authorized or required; no `workorders/README.md` is justified |
| `.github/workflows/ci.yml` | Invokes the classifier and permanent suite but owns no Work Order path grammar | inspection only | No edit authorized or required |
| `tools/check_workorder_status_boundary.ps1` | Recognizes only `^WORKORDER(?:_[1-9][0-9]*)?\.md$`, enumerates the root tree, resolves the sole marker, and requires one unchanged active path for fast status transitions | Units A and C | Load-bearing executable control-plane consumer |
| `tools/test_workorder_status_boundary.ps1` | Models `WORKORDER_10.md` as active and `WORKORDER.md` as inactive and structurally audits the root-only classifier | Units A and C | Load-bearing permanent evidence consumer |
| `tools/check_all.ps1` | Invokes the permanent classifier suite; it does not own candidate paths or marker placement | inspection only | No edit authorized or required |
| `src/diagnostic_catalog.rs` | Fifteen diagnostic-family doctrine lists name the current Work Order 9 path as `WORKORDER.md` | Unit B | Durable source-metadata consumer; all fifteen path literals must follow Work Order 9 and permanent same-file evidence must make resolution load-bearing |
| closed Work Orders and research snapshots | Contain historical root filenames, session descriptions, and prior facts | content preservation only | Historical content must not be rewritten |

The complete audit proves exactly five implementation consumers across the
three units:

1. `tools/check_workorder_status_boundary.ps1`;
2. `tools/test_workorder_status_boundary.ps1`;
3. `AGENTS.md`;
4. `src/diagnostic_catalog.rs`; and
5. the thirteen Work Order rename records frozen below.

The workflow, full-preflight wrapper, governance, OCB method, and README are
callers or path-agnostic doctrine, not missing migration paths. Historical
references remain historical. A direct move before Unit A would make the
published classifier unable to resolve the active Work Order and is forbidden.

### Satisfiability ruling

The migration is satisfiable without a new catalog, workflow edit, governance
rewrite, history rewrite, archive mutation, or compiler semantic change:

- Unit A can recursively inspect committed Git tree entries and accept either
  one internally consistent legacy tree or one internally consistent canonical
  tree while retaining the existing status-projection and CI-evidence logic.
- Unit B can perform thirteen byte-identical Git renames in one commit and edit
  the two proven in-place consumers.
- Unit C can delete the temporary root alternative after the canonical tree and
  nested status-only lifecycle have been published and proven.

No additional implementation path is authorized. Discovery of one is a stop,
not permission to expand a unit.

## Published baseline and issuance package

The frozen planning baseline is:

- branch, `HEAD`, local `main`, cached `origin/main`, and live `main`:
  `b703347996b6803f0d77bc8f0142e26333f0d9af`;
- ahead/behind: `0/0`;
- subject: `docs(workorder): close work order 20`;
- closeout CI: workflow `31743775395`, attempt `1`, tested SHA
  `b703347996b6803f0d77bc8f0142e26333f0d9af`, Ubuntu job
  `94593256902` success, Windows job `94593256658` success, overall success;
- clean worktree, empty index, no untracked files before authorship; and
- sole published marker at `WORKORDER_20.md:4` before the issuance edit.

This planning package performs only two document operations:

1. remove the marker line from closed `WORKORDER_20.md`, preserving every
   other published byte; and
2. create temporary root `WORKORDER_21.md` with the sole marker at line 4.

The package is not the repository migration. Root Work Orders remain in place
until Units A and B pass their separate gates.

### Closed root inventory authenticated before design

All twelve published root documents were read and authenticated as closed.
Their issuance-baseline identities are recorded to prevent substitution.

| Current path | Work Order | Lines | Bytes | SHA-256 | Git blob |
| --- | ---: | ---: | ---: | --- | --- |
| `WORKORDER.md` | 9 | 2,982 | 162,630 | `4efa1f7c4da6c05873fc0305d3bdebd47b3d95c695ad32a27b9520af60640eef` | `467013c89c0abeec867aed68b97868d9729ef28b` |
| `WORKORDER_10.md` | 10 | 6,994 | 394,680 | `4f8d645cbb9b45a20b13fe69314285d7ab3748dc85306ca92e4853713cf5c7b7` | `d25bcfadb040f1daaec3825412f3294f57b13935` |
| `WORKORDER_11.md` | 11 | 1,276 | 62,505 | `c1508ed53e8275691cd2a7981c9b86cd307de01e79a1a94ee72cb26bea291718` | `478626bb3f1fc99c1023642f5269ec0cc1d2af2b` |
| `WORKORDER_12.md` | 12 | 355 | 18,052 | `cd0cbea7596f8f546b33097a0340fde0efc06c6ca42387f2c59fc516c831a260` | `802b4bfbba20fd72b291edb05b5eea436c06fded` |
| `WORKORDER_13.md` | 13 | 803 | 44,914 | `8476300f2eb450ac582786febbbd110cc7ad6148f9c1f6093c9a6f231ee43225` | `73509eda67a0cb872fe8a02feab4d920261cefc0` |
| `WORKORDER_14.md` | 14 | 510 | 27,717 | `0f582ca81616e8c9de2f62ee73aceabf52b78f12aa985682e1a39d523fd8f79a` | `69b5e897d387be29e5223c6c0b9a355b7f2e3eae` |
| `WORKORDER_15.md` | 15 | 1,000 | 57,228 | `944797ef23a04bed177cf984a5c3154a7e185f1213fedd450f8b44052ba6a414` | `628d001050e90db6ded33543da37667e36e1413a` |
| `WORKORDER_16.md` | 16 | 1,637 | 91,386 | `39459956d08c237b62899db1261d5e5253ac23f3ffd3759747522b9ce983ba17` | `bf4b6b5dc0ea60bc7727588dd437e549bfe5c8ca` |
| `WORKORDER_17.md` | 17 | 1,536 | 85,609 | `175304d5e53711fcb12228d2f9bd31e272e6e027c1dbb06ee23c3d80c7c75972` | `b3678efb721a42d5b174cfa6fee892c62d148b5f` |
| `WORKORDER_18.md` | 18 | 944 | 46,050 | `003854ee50cdf29f0ff93fb8a6d647b25fe58ebd0bdfa0d133c79e363b4eaa11` | `71f0f071dc3c2028f2a47f4767c80dab2ba4afec` |
| `WORKORDER_19.md` | 19 | 953 | 42,537 | `225ac91f8209c6b1e668c4b9fb03450033462db2f838da8fc00d09e2a574f9f6` | `14a5af8d97764ff5feade9cddc092b94a9883e0a` |
| `WORKORDER_20.md` | 20 | 3,037 | 162,702 | `f60363b7d995877063bdbe63bae7904af75905c9367bc2dc7601b4efe86e94e1` | `eaf600b2226625321280c1117599bc70ae67d6c5` |

Work Orders 9 through 20 are closed. Work Orders 11, 13, 15, 18, and 20
also retain their published archive evidence where applicable. Closure does not
authorize deletion, condensation, or historical rewriting.

### Protected non-branch state

The four stashes must remain ordered and byte-identical throughout all units:

1. `f9b310902f804a0b8b7a3bf58910c7ec4f639c18`;
2. `303ee9af93696409bea66d3f8a379cb1a8cf8e1a`;
3. `bd6d2722cffa50da8463201204a48f4a7305ae1b`; and
4. `73101039f5e3faf0c802d4f723add1b891c51602`.

The published remote contains eleven refs at issuance. Every existing local,
cached, and live archive ref must remain exact, including
`archive/workorder-20-unit-b-terminal-fast-rejection-2026-08-13` at
`94b32ca95a14072b4a22adf6e56101118650c683`. No unit may add, delete, rename,
force-update, merge, or otherwise mutate an archive.

## Canonical target topology

The temporary issuance path is `WORKORDER_21.md`. Its mandatory canonical path
after Unit B is:

```text
workorders/active/WORKORDER_21.md
```

The complete final active inventory is exactly one file:

```text
workorders/active/WORKORDER_21.md
```

The complete final closed inventory is exactly twelve files:

```text
workorders/closed/WORKORDER_9.md
workorders/closed/WORKORDER_10.md
workorders/closed/WORKORDER_11.md
workorders/closed/WORKORDER_12.md
workorders/closed/WORKORDER_13.md
workorders/closed/WORKORDER_14.md
workorders/closed/WORKORDER_15.md
workorders/closed/WORKORDER_16.md
workorders/closed/WORKORDER_17.md
workorders/closed/WORKORDER_18.md
workorders/closed/WORKORDER_19.md
workorders/closed/WORKORDER_20.md
```

After Unit B, the repository must contain no root `WORKORDER*.md` path. After
Unit C, the classifier must reject every such root path. The canonical layout
does not permit an unnumbered Work Order.

## Global invariants

All three units preserve these requirements:

1. Exactly one active-marker line exists repository-wide across the raw bytes
   of every committed blob entry, independent of Git text/binary
   classification.
2. The marker appears exactly once in the sole valid active Work Order and
   immediately precedes its unique status anchor.
3. Closed Work Orders never carry the active marker.
4. Every Work Order is a regular `100644` UTF-8 file without BOM, uses LF-only
   encoding, and ends in a final LF.
5. Invalid Work-Order-like paths fail closed; adjacent names that are not Work
   Orders do not become candidates.
6. Fast classification remains limited to an exact linear recognized
   status/current-gate transition on the same active path with no other path
   change.
7. Adds, deletes, copies, renames, modes, types, symlinks, submodules, malformed
   bytes, conflict markers, active-path changes, and unrelated edits cannot be
   misclassified as fast.
8. Production decisions are proven through disposable Git repositories and
   observable classifier results. Source occurrence counts may guard structure
   but may not substitute for behavior.
9. PowerShell 5.1, Ubuntu PowerShell, and Windows PowerShell behavior must agree.
10. No unit inherits authorization from the preceding unit.

### Canonical successor issuance after Unit B

Future successor issuance is one exact full-lane semantic operation, not a
byte-identical rename and never a status-only transition. Starting from a
canonical parent with exactly one marked predecessor at
`workorders/active/WORKORDER_N.md`, the child must:

1. move that predecessor to `workorders/closed/WORKORDER_N.md`;
2. remove exactly its single line
   `<!-- hum-active-workorder:v1 -->`;
3. preserve every other predecessor byte exactly;
4. create `workorders/active/WORKORDER_N+1.md` as a regular numbered Work Order
   with the sole repository-wide active marker at line 4; and
5. contain no other Work Order or marker change.

The parent tree must resolve `WORKORDER_N.md` as the exact active Work Order.
The child tree must resolve `WORKORDER_N+1.md` as the exact active Work Order
and the unmarked predecessor as closed. The transition must return exactly
`mode=full` and `reason=no_status_transition`; it is never eligible for fast
status-only evidence. The high-similarity predecessor move may not display as
`R100` because the marker line is intentionally deleted. Git rename display is
supporting evidence; exact old-byte/new-byte reconstruction is authority.

Permanent behavior evidence must reject each distinct corruption:

- retaining the predecessor marker in the closed destination, which creates a
  marker in a closed Work Order and two repository-wide markers;
- removing any predecessor byte other than the one marker line;
- creating the successor without moving and unmarking the predecessor;
- moving/unmarking the predecessor without creating a valid successor; and
- a predecessor source/closed-destination number or path mismatch.

Once this exact transition commits, the closed predecessor is immutable. This
future rule does not change Unit B: WO21 remains active during Unit B, its
temporary-root-to-active move remains byte-identical `R100`, and all twelve
already-closed predecessors remain byte-identical `R100` moves.

## Unit A - dual-layout control-plane support

### Purpose and authorization boundary

Unit A teaches the published status-boundary control plane to understand both
the complete legacy root layout and the complete canonical nested layout. It
does not move, copy, add, delete, or edit any Work Order or policy file.

Unit A may begin only after this planning package is independently accepted,
committed under a separate authorization, published with terminal-green full
CI, status-recorded, published again with terminal-green fast CI, and followed
by a fresh explicit BDFL Unit A signal.

### Exact path envelope and stop

Unit A authorizes exactly two modified paths:

1. `tools/check_workorder_status_boundary.ps1`;
2. `tools/test_workorder_status_boundary.ps1`.

Maximum path count: 2. A third path, an add/delete/rename, a mode/type change,
or any Work Order edit is a mandatory stop and return to the BDFL.

### Budget

| Path | Maximum insertions | Maximum deletions |
| --- | ---: | ---: |
| `tools/check_workorder_status_boundary.ps1` | 260 | 80 |
| `tools/test_workorder_status_boundary.ps1` | 900 | 180 |
| **Unit A total** | **1,160** | **260** |

Whitespace-insensitive totals must remain within the same ceilings. Reformatting
unrelated code, dependency changes, generated output, and proof artifacts are
forbidden. A ceiling breach is a stop, not an invitation to consume the full
budget.

### Required behavior

Unit A must define and exercise three case-sensitive semantic path classes:

- legacy root candidates: `WORKORDER.md` or
  `WORKORDER_[1-9][0-9]*.md` at the repository root;
- canonical active candidates:
  `workorders/active/WORKORDER_[1-9][0-9]*.md`; and
- canonical closed candidates:
  `workorders/closed/WORKORDER_[1-9][0-9]*.md`.

The implementation may refine regex spelling for PowerShell 5.1 but may not
weaken those sets. It must inspect the committed tree recursively using exact
Git path identity, not the host filesystem's case folding.

A tree is valid only when it is entirely one of:

- legacy: all Work Orders are valid root candidates and exactly one root file
  carries the marker; or
- canonical: every Work Order is under the correct canonical directory,
  exactly one numbered file exists under `workorders/active`, that file carries
  the sole marker, and every numbered closed file is unmarked.

Any tree containing both a legacy Work Order and a canonical Work Order is a
mixed layout and fails closed. Unit A's migration-transition test compares a
valid legacy parent with a valid canonical child; each tree is internally
valid, but the active path differs and the commit must select full mode.

Unit A must also understand the exact canonical successor transition. Its
marked-predecessor parent and unmarked-predecessor/marked-successor child are
both independently valid canonical trees, but their active paths differ, so
the transition must select `full/no_status_transition`. A child retaining the
closed predecessor marker is invalid and must fail closed before fast evidence.

Legacy acceptance is temporary and exists only so the planning/status lifecycle
and the Unit B migration remain governable. It is not the final contract.

`Resolve-ActiveWorkOrderBlob` or its reviewed replacement must:

- parse recursive `git ls-tree` inventory fail-closed;
- inspect every tree entry whose object type is `blob` through raw Git object
  plumbing and count exact active-marker-line occurrences per tree entry;
- reject malformed or ambiguous Work-Order-like paths before selecting a file;
- require regular `100644` blobs;
- decode strict UTF-8 without BOM;
- require exactly one valid marker at the recognized location;
- reject a marker in a closed file;
- return exact path, object ID, bytes, and text for the sole active file; and
- preserve all existing status-projection, event-history, diff-hygiene, and
  authenticated Actions-evidence boundaries.

Fast status classification requires parent and child to resolve the same exact
active path. Parent/child disagreement, including a correct migration, is full.

### Exact Unit A case ledger

The published suite contains exactly 123 credited cases before Unit A. Unit A
adds exactly the following 28 cases, producing exactly 151. These names,
ordinals, layouts, changes, results, owned properties, and credit types are
frozen. `eligible_status_chain` and `no_status_transition` are the exact
published classifier reasons; Unit A needs no new reason vocabulary.

| Ordinal | Stable permanent case name | Parent layout and state | Child layout and single relevant change | Mode | Reason | Sole owned property | Type |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A01 | `canonical_nested_header_only_fast` | Valid canonical nested tree; `workorders/active/WORKORDER_21.md` is sole marked active | Same nested path; only the recognized `Status:` header-body interval changes; current-gate body and immutable bytes remain exact | `fast` | `eligible_status_chain` | The recognized header interval remains status-only for the canonical nested active path | positive |
| A02 | `canonical_nested_gate_only_fast` | Valid canonical nested tree; `workorders/active/WORKORDER_21.md` is sole marked active | Same nested path; only the recognized `## Current authorization gate` body changes; Status header body and immutable bytes remain exact | `fast` | `eligible_status_chain` | The recognized current-gate interval remains status-only for the canonical nested active path | positive |
| A03 | `canonical_nested_two_commit_chain_fast` | Valid canonical anchor with `workorders/active/WORKORDER_21.md` sole marked active; first child changes only its Status header body | Second child changes only the same nested path's current-gate body; event range is first child through second child | `fast` | `eligible_status_chain` | Canonical nested suffix scanning retains the pre-chain authenticated anchor and exactly two ordered status-only transitions | positive |
| A04 | `canonical_non_status_full` | Valid canonical tree with exact nested active path resolved | Same nested path; one immutable mandate byte changes | `full` | `no_status_transition` | Canonical immutable-content rejection | positive |
| A05 | `legacy_to_canonical_migration_full` | Complete valid legacy root inventory with marked WO21 | Complete byte-identical canonical relocation of every Work Order | `full` | `no_status_transition` | Valid cross-layout migration is resolvable but never fast | positive |
| A06 | `canonical_successor_issuance_full` | Canonical marked active `WORKORDER_N.md`; all earlier records closed/unmarked | Move N active-to-closed, remove only its marker, create marked active N+1 | `full` | `no_status_transition` | Valid successor issuance resolves both trees and is full | positive |
| A07 | `canonical_adjacent_workordering_ignored` | Valid canonical tree already containing adjacent `WORKORDERING.md` | Only nested active status/current-gate body changes | `fast` | `eligible_status_chain` | Adjacent non-candidate does not poison canonical fast status | positive |
| A08 | `closed_marker_rejected` | Valid canonical tree | Move the sole marker from active WO21 into closed WO20, leaving exactly one marker but in the wrong class | `full` | `no_status_transition` | Marker-to-active-directory binding | adversarial |
| A09 | `closed_copy_cannot_become_active` | Valid canonical tree with closed unmarked WO20 and marked active WO21 | Replace the active file with an exact unmarked copy of closed WO20 at `workorders/active/WORKORDER_20.md` | `full` | `no_status_transition` | Closed bytes copied into active cannot acquire authority | adversarial |
| A10 | `two_active_candidates_rejected` | Valid canonical tree | Add one distinct unmarked numbered active candidate while retaining the sole marked active candidate | `full` | `no_status_transition` | Active-candidate cardinality is exactly one | adversarial |
| A11 | `duplicate_repository_marker_rejected` | Three-state fixture: State 1 is a valid canonical control with the sole legitimate marker, no `hidden.bin`, no other marker occurrence, and its active path proven resolvable; State 2 is a committed corruption anchor derived by adding only regular `100644` `hidden.bin`, whose raw bytes contain at least one NUL and exactly one standalone ASCII marker line, while retaining the legitimate marker unchanged | From the corruption anchor, retain `hidden.bin` byte-identically and change only the recognized `Status:` body in the same canonical active Work Order; commit this State 3 adjacent child; the measured anchor-to-child diff is exactly one `M workorders/active/WORKORDER_21.md` record, and the single classifier invocation measures State 2 to State 3 | `full` | `no_status_transition` | Repository-wide marker cardinality counts exact marker-line occurrences in every committed blob entry regardless of path or Git text/binary classification | adversarial |
| A12 | `mixed_layout_ambiguity_rejected` | Valid legacy root tree with one marked active | Add one unmarked canonical active candidate while retaining the legacy inventory | `full` | `no_status_transition` | A single tree cannot mix legacy and canonical candidates | adversarial |
| A13 | `canonical_unnumbered_active_rejected` | Valid canonical tree | Rename active WO21 to `workorders/active/WORKORDER.md` | `full` | `no_status_transition` | Canonical Work Orders must be numbered | adversarial |
| A14 | `canonical_leading_zero_rejected` | Valid canonical tree | Rename active WO21 to `workorders/active/WORKORDER_021.md` | `full` | `no_status_transition` | Canonical numbers forbid leading zero | adversarial |
| A15 | `active_directory_case_rejected` | Valid canonical tree | Rename directory component `active` to exact `Active` for WO21 | `full` | `no_status_transition` | Active directory identity is case-sensitive | adversarial |
| A16 | `closed_directory_case_rejected` | Valid canonical tree | Rename directory component `closed` to exact `Closed` for WO20 | `full` | `no_status_transition` | Closed directory identity is case-sensitive | adversarial |
| A17 | `extension_case_rejected` | Valid canonical tree | Rename active WO21 extension from `.md` to exact `.MD` | `full` | `no_status_transition` | Canonical extension is case-sensitive | adversarial |
| A18 | `canonical_suffix_rejected` | Valid canonical tree | Rename active WO21 to exact `WORKORDER_21.md.bak` | `full` | `no_status_transition` | Canonical candidates forbid backup/suffix extensions | adversarial |
| A19 | `backslash_separator_rejected` | Valid canonical tree | Git-object plumbing renames active WO21 to literal tree path `workorders\active\WORKORDER_21.md` | `full` | `no_status_transition` | Only `/` canonical separators are accepted | adversarial |
| A20 | `unrecognized_directory_rejected` | Valid canonical tree | Rename active WO21 to exact `workorders/pending/WORKORDER_21.md` | `full` | `no_status_transition` | Work Orders cannot occupy an unrecognized directory | adversarial |
| A21 | `active_symlink_rejected` | Valid canonical tree | Replace active WO21 regular blob with mode `120000` symlink blob | `full` | `no_status_transition` | Active object must be a regular `100644` blob, not symlink | adversarial |
| A22 | `active_submodule_rejected` | Valid canonical tree | Replace active WO21 with mode `160000` commit object | `full` | `no_status_transition` | Active object must be a regular `100644` blob, not submodule | adversarial |
| A23 | `active_deletion_without_successor_full` | Valid canonical tree | Delete active WO21 and create no successor | `full` | `no_status_transition` | Active deletion cannot leave a child without authority | adversarial |
| A24 | `active_rename_without_successor_full` | Valid canonical active WO21 and no closed WO21 | Rename marked active WO21 directly to active WO22, preserve its marker, and do not create closed WO21 | `full` | `no_status_transition` | A bare active-path rename is not valid successor issuance | adversarial |
| A25 | `status_plus_closed_edit_full` | Valid canonical tree | Edit active status body and one immutable byte in closed WO20 | `full` | `no_status_transition` | Closed-file edit disqualifies status-only transition | adversarial |
| A26 | `status_plus_workorder_move_full` | Valid canonical tree | Edit active status body and move closed WO20 to closed WO22 | `full` | `no_status_transition` | Any additional Work Order move disqualifies status-only transition | adversarial |
| A27 | `canonical_active_marker_removed_full` | Valid canonical tree with sole marked active WO21 | Keep active WO21 at the same exact canonical path; remove only its active-marker line; add or move no file and create no successor | `full` | `no_status_transition` | A canonical active candidate without its required marker cannot authorize the child tree even when its path is unchanged | adversarial |
| A28 | `successor_retained_predecessor_marker_rejected` | Valid canonical marked predecessor parent | Perform A06 but retain the marker in closed predecessor N | `full` | `no_status_transition` | Successor child forbids retained closed marker/two markers | adversarial |

A01, A02, A03, A05, A06, and A07 require explicit non-degenerate controls that
resolve the intended exact paths and reach the intended production branch
before asserting the result. A04 likewise proves its valid tree resolves before
the immutable change is classified.

A01 through A03 are new credits because they exercise the newly introduced
canonical nested active-path class; no published case uses a nested active Work
Order. A01 owns the nested header interval, A02 owns the nested current-gate
interval, and A03 owns nested multi-transition suffix traversal and ordering.
A03 must authenticate the pre-chain canonical anchor and report exactly the
ordered first-child and second-child transitions. None may be implemented as an
alias or renamed invocation of an existing root case.

A24 and A27 must reach different production boundaries. A24 changes the active
path by a bare rename, so deleting the active-path identity check must make A24
fail. A27 preserves the exact active path but removes marker authority, so
weakening marker-required resolution must make A27 fail. Neither mutation or
expected assertion may satisfy the other case. The general rule that
unexplained parent/child active-path disagreement selects full remains owned
collectively by A24's invalid bare rename, A05's valid migration, and A06's
valid successor issuance.

Each A01-A28 row must be one direct, individually named case definition and one
credit. A loop or parameter expansion may build disposable support data but may
not earn multiple case credits. Inventory evidence must require exactly 123
published names plus these 28 names, exactly 151 unique names total, no missing,
duplicate, or unexpected name, and all 151 cases executed twice
deterministically.

A01, A02, A03, and A07 are the only new fast cases. Exactly 4 new cases are
`fast/eligible_status_chain`; exactly 24 are `full/no_status_transition`.
Therefore the exact Unit A arithmetic remains `123 + 28 = 151`.

A11 retains its ordinal, stable name, full result, owned repository-wide
marker-cardinality property, and one direct credited invocation. Its amended
three-state binary-blob fixture does not create A29 or another credit. The
published case `duplicate active marker is full` remains separately credited
and continues to cover a visible second marker inside the active Work Order.
A11 alone covers the independent binary-classification bypass described below.

For every adversarial row, the honest control must first reach the intended
resolver/classifier branch; the child then mutates only the row's owned
property. Deleting or weakening the corresponding production check must make
that exact stable case fail. Full results must carry empty/zero binding fields;
fast results must retain the exact authenticated status-chain binding.

The published 123 cases remain continuing coverage for encoding, BOM/invalid
UTF-8, conflict markers, status/owner/current-gate/end-marker corruption,
history replacement, grafts, merges, missing/divergent/reversed ranges, and the
published mode/type/copy/add/delete protections. None of those existing credits
is renamed or recounted as A01-A28 unless the row is the distinct nested-layout
scenario frozen above.

### Platform, review, and lifecycle

- Source and test scripts must parse under Windows PowerShell 5.1.
- Focused tests run under the local Windows environment before review.
- The implementation candidate receives one fresh independent complete-tree
  review with proportional positive/adversarial evidence and one direct Fast
  invocation only if the issued review mandate authorizes it.
- Publication is a normal non-force `main` fast-forward and must run full CI on
  required Ubuntu and Windows jobs. Ubuntu runs the platform-independent
  Exhaustive producer; Windows skips only that duplicate.
- A separately authored status record must project immutably, classify fast,
  publish under separate authority, and reach terminal-green fast CI.
- Unit B remains unauthorized until a fresh BDFL signal after that lifecycle.

Exact proposed Unit A commit subject:

```text
ci(workorders): support nested work order layouts
```

The `ci` type is required because Unit A changes the production CI classifier,
not only its tests.

### Unit A exclusions

No Work Order move or edit; no `AGENTS.md` change; no diagnostic-catalog change;
no workflow or preflight-wrapper change; no compiler/source change; no status
record inside the implementation commit; no archive/stash operation; no Unit B
work.

### Binary-marker recovery amendment (BDFL-directed, 2026-08-14)

The first complete Unit A candidate was independently reviewed at the exact
published baseline `77974e7c576e298a4fb67b6ec8d5f88035582f2a`. The verdict was
`ACCEPT WITH REQUIRED FIX`: P0 and P2 were empty, and the sole P1 was that
repository-wide marker counting used `git grep -I`. Because `-I` ignores
Git-classified binary blobs, a committed `hidden.bin` containing a NUL byte and
one exact standalone marker line admitted an otherwise exact status transition
as `fast/eligible_status_chain` even though two repository-wide marker
occurrences existed. The permanent 151-case suite and every other focused
review probe passed. The reviewer did not invoke Fast, so the single direct
independent-review Fast allowance remains unconsumed.

The first amendment review also returned `ACCEPT WITH REQUIRED FIX`, with no P0
or P2. Its sole P1 was that adding `hidden.bin` in A11's measured parent-to-child
diff independently forced full mode under the classifier's exact one-record
gate. Both the binary-safe implementation and a defective `git grep -I`
mutation therefore returned `full/no_status_transition`, so the two-state
fixture could not make the marker-count boundary load-bearing. The reviewer
independently demonstrated that committing the corruption anchor first and
then measuring its adjacent status-only child yields honest full and weakened
fast behavior. This is a fixture correction only; the accepted raw-byte
production architecture remains unchanged.

The candidate was parked without modification as follows:

- stash commit: `799d4eaa2fb473633b41bbf17ad82e67fe2386a3`;
- first parent: `77974e7c576e298a4fb67b6ec8d5f88035582f2a`;
- complete tree: `ee9a392827a5c707962eaef5626d6b429a992f58`;
- scoped two-path tree: `2ac3a52f056e0c705c1b689c7bc06b4845aa0748`;
- production blob: `64a66f49a40d423cb6aa60d577e77843bd280d15`;
- test blob: `481ae5752671f67610170cebc3f9f730b09f1101`;
- raw statistics: `+677/-55`; and
- whitespace-insensitive statistics: `+674/-52`.

The amendment changes A11's fixture only. It does not add A29, increase the
151-case inventory, rename A11, or transfer its credit. Revised A11 has this
exact three-state contract.

State 1 - valid canonical control:

1. Start from a valid canonical tree whose numbered active Work Order owns the
   sole legitimate marker.
2. `hidden.bin` does not exist, no other exact marker occurrence exists, and
   the exact canonical active path is independently proven to resolve normally.

State 2 - corruption anchor:

3. Starting from State 1, add only one regular `100644` non-Work-Order blob at
   exact path `hidden.bin` and commit that tree as the corruption anchor.
4. `hidden.bin` contains at least one NUL byte and exactly one standalone ASCII
   line `<!-- hum-active-workorder:v1 -->`; the legitimate active marker remains
   unchanged.
5. Raw Git-object inspection independently proves both the NUL byte and exact
   marker line. The binary-safe implementation must reject repository-wide
   marker cardinality at this anchor.

State 3 - adjacent status child:

6. Starting from the corruption anchor, retain `hidden.bin` byte-identically,
   change only the recognized `Status:` body in the same canonical active Work
   Order, and commit that tree as the adjacent child.
7. The measured corruption-anchor-to-adjacent-child diff contains exactly one
   record: `M workorders/active/WORKORDER_21.md`.
8. A11's single credited classifier invocation measures only the corruption
   anchor to the adjacent child.
9. Honest classification is exactly `mode=full` and
   `reason=no_status_transition`, with empty anchor and transitions and zero
   run, attempt, Ubuntu-job, and Windows-job bindings. The full result must be
   attributable to marker cardinality, not diff shape.
10. A11 owns repository-wide exact marker-line counting across every committed
    blob entry regardless of path or Git text/binary classification.

The production correction must not use `git grep -I`, filesystem reads, text
decoding, or Git's text/binary classification as marker-count authority. It
must enumerate the committed tree through exact Git object plumbing and inspect
the raw bytes of every entry whose object type is `blob`, including regular
`100644`, executable `100755`, and symlink `120000` blob entries. Counting is
per tree entry, not per unique blob OID: one blob referenced at two paths
represents two repository occurrences. Submodule `160000` commit entries have
no blob bytes and are not decoded as blobs.

An occurrence counts only when the raw byte line equals the exact ASCII marker
bytes, preceded by start-of-blob or LF and followed by LF or end-of-blob. CR,
prefixes, suffixes, and marker-like substrings do not count. A NUL or other
binary byte elsewhere in the blob cannot hide an exact marker line. Any tree
enumeration, object-type validation, or blob-read failure fails closed. The
existing strict UTF-8, BOM, LF framing, final-LF, and marker-position checks for
the selected Work Order remain unchanged.

The corrected evidence must be load-bearing:

- State 1 resolves normally with exactly one marker;
- State 2 adds only `hidden.bin`, commits the corruption anchor, preserves the
  legitimate marker, and independently proves the NUL and exact marker line;
- State 3 retains `hidden.bin` byte-identically, changes only the recognized
  status body, and produces exactly one measured active-Work-Order `M` record;
- exact A11 measures State 2 to State 3 and returns
  `full/no_status_transition` with empty/zero bindings;
- a disposable mutation that restores `git grep -I`, skips NUL-containing
  blobs, or otherwise excludes Git-classified binary blobs must make both
  measured endpoints incorrectly resolve the same canonical active path and
  make exact A11 return `fast/eligible_status_chain`;
- the weakened result must carry the expected nonzero synthetic run, attempt,
  Ubuntu-job, and Windows-job bindings; its anchor must be the corruption-anchor
  commit and its transitions field must be exactly
  `corruption-anchor>adjacent-child`;
- the mutation must make A11 fail at its expected-full assertion. Because
  `hidden.bin` is present byte-identically in both measured endpoints and its
  addition is outside the measured transition, the only honest/weakened
  difference is whether binary marker bytes participate in repository-wide
  cardinality;
- that mutation reaches the marker-count boundary and does not fail through
  syntax, fixture construction, path resolution, Git plumbing, or unrelated
  behavior; and
- the published visible duplicate-marker case remains green.

Inventory arithmetic remains exact: 123 published cases plus 28 Unit A cases
equals 151 invocations and 151 unique stable names. Unit A remains exactly four
fast and 24 full cases, with A11 among the 24 full cases. Unit C must retain
revised binary-safe A11 and the published visible duplicate-marker case when it
removes the legacy fallback. The Unit C six-for-six rollover and final
`119 + 26 + 6 = 151` composition remain unchanged. No extra credited case,
alias, or renamed credit is permitted.

The existing Unit A ceilings remain authoritative. The parked candidate uses
`+142/-46` production, `+535/-9` tests, and `+677/-55` combined; ignoring
whitespace it uses `+139/-43`, `+535/-9`, and `+674/-52`. The correction must
remain within production `+260/-80`, tests `+900/-180`, and combined
`+1,160/-260`, under both raw and whitespace-insensitive accounting. The
amendment grants no extra path or line budget.

Recovery follows exactly this separately gated lifecycle:

1. Author this amendment in `WORKORDER_21.md` only.
2. Obtain one fresh independent amendment review.
3. Under separate BDFL authority, create one local commit with subject
   `docs(workorder): require binary-safe marker counting`.
4. Separately publish that exact amendment and obtain terminal-green full CI.
5. Create a separate publication-status commit.
6. Separately publish that status commit and obtain terminal-green fast CI.
7. Obtain a fresh explicit BDFL Unit A correction-resumption signal.
8. Apply, but do not pop, stash
   `799d4eaa2fb473633b41bbf17ad82e67fe2386a3`.
9. Authenticate the exact parked two-file candidate.
10. Correct only the two Unit A tool files.
11. Prove revised A11's exact three-state construction, honest full result,
    weakening-mutation fast result with exact synthetic bindings, the 151-case
    inventory, all budgets, and every standing check.
12. Leave the corrected candidate unstaged for a fresh independent review.
13. Preserve the single direct reviewer Fast allowance for that fresh review;
    the prior review did not consume it.
14. Keep Unit B unauthorized until the complete Unit A
    commit/publication/status lifecycle finishes and a new BDFL signal is
    issued.

## Unit B - content-preserving repository migration

### Entry gate

Unit B begins only after Unit A implementation acceptance, its exact commit and
publication, terminal-green full CI, separately committed/published status
record, terminal-green fast CI, and a fresh explicit BDFL Unit B signal.

Before mutation, the implementer must re-authenticate `main`, live remote refs,
all stashes/archives, the exact Work Order inventory, every source blob, the
sole marker, and the accepted Unit A classifier behavior. Identity values may
advance through authorized status records; the Unit B handoff must freeze the
actual published input identities before staging.

### Exact path envelope and stop

Unit B authorizes exactly fifteen logical Git change records:

1. `WORKORDER.md` -> `workorders/closed/WORKORDER_9.md`;
2. `WORKORDER_10.md` -> `workorders/closed/WORKORDER_10.md`;
3. `WORKORDER_11.md` -> `workorders/closed/WORKORDER_11.md`;
4. `WORKORDER_12.md` -> `workorders/closed/WORKORDER_12.md`;
5. `WORKORDER_13.md` -> `workorders/closed/WORKORDER_13.md`;
6. `WORKORDER_14.md` -> `workorders/closed/WORKORDER_14.md`;
7. `WORKORDER_15.md` -> `workorders/closed/WORKORDER_15.md`;
8. `WORKORDER_16.md` -> `workorders/closed/WORKORDER_16.md`;
9. `WORKORDER_17.md` -> `workorders/closed/WORKORDER_17.md`;
10. `WORKORDER_18.md` -> `workorders/closed/WORKORDER_18.md`;
11. `WORKORDER_19.md` -> `workorders/closed/WORKORDER_19.md`;
12. `WORKORDER_20.md` -> `workorders/closed/WORKORDER_20.md`;
13. `WORKORDER_21.md` -> `workorders/active/WORKORDER_21.md`;
14. modify `AGENTS.md`; and
15. modify `src/diagnostic_catalog.rs`.

Those records mention exactly 28 authorized endpoint names: 13 old rename
paths, 13 new rename paths, and 2 in-place files. Maximum logical path count is
15. A sixteenth record, a twenty-ninth endpoint, or any unlisted path is a
mandatory stop. Directory creation is represented only by the listed tracked
destinations; no placeholder or `workorders/README.md` is authorized.

### Content-preserving rename contract

All thirteen renames must be Git `R100` records with identical source and
destination blob IDs, byte counts, SHA-256 values, LF/CR counts, and final-LF
state. In particular:

- `WORKORDER.md` becomes numbered `WORKORDER_9.md` without editing its title,
  history, status, or any other byte;
- closed Work Orders 10 through 20 retain every byte;
- the then-current published root WO21 moves to the active directory without
  changing its marker, status, history, or any other byte; and
- no closed document is updated merely to describe its new path.

Git's rename display is evidence, not authority by itself. The implementation
must compare source blobs from the exact parent tree to destination blobs from
the exact child tree and reconstruct the complete mapping independently.

### In-place policy consumer: `AGENTS.md`

The in-place policy edit must make these live rules exact:

- the active Work Order is the sole regular numbered Markdown file under
  `workorders/active` carrying the active marker;
- cold-start finds it by the marker and canonical active path, not the root;
- routine status-only closure permits exactly that active marked path and no
  other path;
- new Work Orders are authored under `workorders/active`;
- closed Work Orders are immutable and live under `workorders/closed`;
- when a successor is issued, the predecessor moves active-to-closed, loses
  exactly its one marker line, preserves every other byte, and becomes
  immutable while the numbered successor is created active with the sole
  marker at line 4;
- both successor parent and child trees must resolve, and the issuance
  transition is `full/no_status_transition`, never status-only; and
- Git history preserves lifecycle without accumulating root files.

Historical observations may remain historical, but no live instruction may
still identify a root Work Order as current authority.

### In-place metadata consumer: `src/diagnostic_catalog.rs`

All fifteen diagnostic-family doctrine references currently equal to
`WORKORDER.md` must become exactly
`workorders/closed/WORKORDER_9.md`. No diagnostic code, family range, owner,
stage, status, precedence, public schema, or compiler decision may change.

The same file must add or strengthen permanent test evidence that:

- every Markdown doctrine path resolves from `CARGO_MANIFEST_DIR` to a regular
  repository file;
- the canonical Work Order 9 path occurs in all fifteen expected family
  doctrine sets;
- the root literal is absent from live diagnostic metadata; and
- mutating one canonical doctrine path to the old root, wrong number, wrong
  directory, or missing file makes the focused test fail.

This is a path-metadata repair and its test, not authorization to change Hum
diagnostic semantics. No other source file is permitted.

### Budget

The thirteen `R100` renames have an exact content budget of `+0/-0`.

| In-place path | Maximum insertions | Maximum deletions |
| --- | ---: | ---: |
| `AGENTS.md` | 45 | 18 |
| `src/diagnostic_catalog.rs` | 45 | 18 |
| **Unit B non-rename total** | **90** | **36** |

The raw and whitespace-insensitive in-place diff must stay within these
ceilings. Moving or reformatting unrelated source/policy text is forbidden.

### Required positive evidence

Unit B must prove:

1. exact parent and child inventories from committed Git trees;
2. 13 `R100` mappings and exactly 2 in-place modifications;
3. identical source/destination blob and byte identities for every rename;
4. exactly one active file and marker at
   `workorders/active/WORKORDER_21.md`;
5. exactly twelve unmarked closed files with numbers 9 through 20;
6. zero root `WORKORDER*.md` paths and zero unnumbered canonical paths;
7. Work Order 9 bytes at the numbered destination equal the exact parent
   `WORKORDER.md` bytes;
8. no lost, duplicated, edited, or misnumbered Work Order;
9. AGENTS live policy contains only canonical placement rules;
10. all fifteen diagnostic doctrine paths resolve to closed Work Order 9;
11. the migration commit selects full mode with
    `reason=no_status_transition`;
12. a subsequent exact nested WO21 status transition selects fast with the
    authenticated status-chain binding; and
13. full Ubuntu and Windows CI pass, with platform-independent Exhaustive run
    once on Ubuntu and skipped only as a duplicate on Windows.

The future AGENTS rule must also match A06 exactly: after Unit B, a canonical
successor transition removes only the predecessor marker while moving it to
closed, creates the numbered marked successor, resolves both trees, and
classifies full. This policy requirement does not add a Unit B rename or
endpoint and does not alter any of the current thirteen `R100` moves.

### Required adversarial evidence

The permanent or review evidence must reject independently:

- one missing rename, duplicate destination, swapped number, edited byte, or
  wrong destination directory;
- a retained root Work Order;
- marker retained in closed WO20 or copied into any closed file;
- two active files or two markers;
- active WO21 copied rather than renamed;
- symlink, submodule, executable mode, or non-blob substitution;
- stale root policy in any live AGENTS instruction;
- one stale or foreign diagnostic doctrine path;
- a migration commit incorrectly preselected as fast;
- a migration plus an unrelated sixteenth change record; and
- any attempt to hide content change behind rename similarity.

Candidate corruption must operate on real parent/child tree inventories. A
preselected failure enum, edited expected string, or source occurrence count
alone is insufficient.

### Platform, review, and lifecycle

- Windows PowerShell 5.1 must parse every touched PowerShell consumer (none are
  expected in Unit B) and run the accepted classifier against the candidate.
- Focused Rust evidence is limited to the same-file diagnostic metadata test;
  normal format/check/warnings-denied checks and full preflight remain required
  by the implementation/review mandate.
- A fresh independent complete-tree reviewer must verify all 15 records, every
  rename identity, policy/metadata scope, classification, and adversarial
  evidence before commit authority.
- The exact accepted commit publishes separately by normal non-force main-only
  fast-forward and must receive terminal-green full Ubuntu and Windows CI.
- The nested WO21 publication status record is a separate one-path edit,
  commit, push, and terminal-green fast-lane CI chain.
- Unit C requires a fresh explicit BDFL signal after that chain.

Exact proposed Unit B commit subject:

```text
chore(workorders): organize active and closed records
```

### Unit B exclusions

No classifier edit; no status-test edit; no workflow/preflight/governance/OCB/
README edit; no `workorders/README.md`; no closed Work Order byte change; no
compiler semantic change; no dependency/generated output; no archive/stash or
history mutation; no Unit C work.

## Unit C - remove the legacy root fallback

### Entry gate

Unit C begins only after Unit B implementation acceptance, exact commit and
publication, terminal-green full CI, nested WO21 status record publication,
terminal-green fast CI, and a fresh explicit BDFL Unit C signal.

### Exact path envelope and stop

Unit C authorizes exactly two modified paths:

1. `tools/check_workorder_status_boundary.ps1`;
2. `tools/test_workorder_status_boundary.ps1`.

Maximum path count: 2. A third path, a Work Order edit/move, an add/delete, or
any other change is a mandatory stop.

### Budget

| Path | Maximum insertions | Maximum deletions |
| --- | ---: | ---: |
| `tools/check_workorder_status_boundary.ps1` | 100 | 180 |
| `tools/test_workorder_status_boundary.ps1` | 600 | 520 |
| **Unit C total** | **700** | **700** |

These ceilings include removal of transitional legacy branches, the exact
six-for-six inventory rollover below, canonicalization of retained valid-layout
positives, and preservation of rooted negative fixtures whose owned property is
final root-path rejection. Unrelated refactoring or formatting is forbidden.

### Final path contract

Only these case-sensitive classes are valid:

- active: `^workorders/active/WORKORDER_[1-9][0-9]*\.md$`;
- closed: `^workorders/closed/WORKORDER_[1-9][0-9]*\.md$`.

The implementation may use equivalent PowerShell 5.1-safe regex composition,
but the semantic sets are exact. There is no unnumbered Work Order and no root
fallback.

The final resolver must recursively inventory committed tree paths and require:

- exactly one valid numbered active candidate;
- exactly one marker on that active candidate;
- zero markers in all closed candidates;
- zero root or wrong-directory Work-Order-like paths;
- regular `100644` strict UTF-8 LF-framed files; and
- exact same-path parent/child identity for a fast status transition.

### Exact Unit C 151-case replacement ledger

Unit C enters with exactly 151 declared cases: the 123 published base cases plus
A01 through A28. It preserves exactly 151 cases through one exact six-for-six
rollover. No published or Unit A case may be renamed, redirected, or converted
and still retain credit when this ledger retires it.

The only six retired stable names are:

1. published `header interval alone is fast`;
2. published `current gate interval alone is fast`;
3. published `two consecutive status commits retain one anchor`;
4. published `rapid status push after canceled fast run remains fast`;
5. A05, `legacy_to_canonical_migration_full`; and
6. A12, `mixed_layout_ambiguity_rejected`.

The first four are valid historical root-layout evidence, but Unit C removes
their permanent credit. The two published two-commit names use the same
repository, base, head, evidence factory, expected anchor, mode, and reason;
both retire, and neither may remain as an alias of A03. A01 alone preserves
nested header-only coverage, A02 alone preserves nested gate-only coverage, and
A03 alone preserves nested two-commit suffix traversal and ordering.

Published `one full anchor plus exact header and gate update is fast` remains
credited. Its valid fixture becomes canonical and continues to own one combined
header-and-gate single-transition property, distinct from A01 and A02. Existing
published root-negative/path-corruption cases may remain rooted when their
property is precisely final root-path rejection; they are not converted into
valid canonical layouts.

A05 required a legacy parent to remain a valid transitional layout. A12
classified one mixed tree through dual-layout ambiguity. After Unit C, any root
Work-Order-like path is invalid directly and legacy is no longer a recognized
alternative layout. Keeping either old owned property would falsely claim that
the final resolver still accepts legacy inventory semantics.

The only six added replacement cases are:

| Replacement | Parent | Child mutation or child | Mode | Reason | Sole owned property |
| --- | --- | --- | --- | --- | --- |
| C01 `canonical_active_basename_case_rejected` | Valid canonical-only tree with sole marked `workorders/active/WORKORDER_21.md` | Change only the active basename to `workorders/active/workorder_21.md`; preserve directory, extension, contents, and marker | `full` | `no_status_transition` | The canonical `WORKORDER_` basename prefix is case-sensitive independently of directory and extension case |
| C02 `canonical_zero_number_rejected` | Valid canonical-only tree with active `WORKORDER_21.md` | Change only the active path to `workorders/active/WORKORDER_0.md`; preserve contents and marker | `full` | `no_status_transition` | Canonical Work Order number zero is forbidden |
| C03 `canonical_signed_number_rejected` | Valid canonical-only tree with active `WORKORDER_21.md` | Change only the active path to `workorders/active/WORKORDER_-21.md`; preserve contents and marker | `full` | `no_status_transition` | Canonical Work Order numbers are unsigned decimal integers |
| C04 `canonical_spaced_number_rejected` | Valid canonical-only tree with active `WORKORDER_21.md` | Change only the active path to `workorders/active/WORKORDER_21 .md`; preserve contents and marker | `full` | `no_status_transition` | Canonical Work Order numbers and extensions permit no embedded or trailing whitespace |
| C05 `canonical_root_reintroduction_rejected` | Valid canonical-only tree with sole marked nested active WO21 and every closed Work Order under `workorders/closed` | Preserve the valid canonical inventory; add one unmarked regular root path `WORKORDER_22.md` | `full` | `no_status_transition` | After fallback removal, any root Work-Order-like path is forbidden even when canonical authority otherwise remains valid |
| C12 `legacy_parent_rejected_after_fallback_removal` | Legacy-root layout that Unit A previously accepted transitionally | Valid canonical nested child representing the former migration shape | `full` | `no_status_transition` | The canonical-only classifier no longer resolves a legacy-root parent; historical migration cannot reopen the removed fallback |

C01 through C04 must be direct, individually named case declarations and
invocations. They may not be loop-generated credits or aliases. Their
load-bearing mutations are exact:

- making only the basename-prefix comparison case-insensitive must fail C01
  without satisfying A15, A16, or A17, which own active-directory,
  closed-directory, and extension case;
- allowing exactly zero while retaining rejection of leading-zero positive
  integers must fail C02 without satisfying A14's `WORKORDER_021.md` case;
- allowing an optional minus sign while preserving all other rules must fail
  C03; and
- allowing whitespace immediately before `.md` while preserving all other
  path rules must fail C04.

The exact retained Unit A disposition is frozen: A01-A04 remain; A06-A11
remain; A13-A28 remain; A05 and A12 alone retire. The exact final composition
is 119 retained published cases, 26 retained Unit A cases, and C01, C02, C03,
C04, C05, and C12. The arithmetic is exactly `151 - 6 + 6 = 151` and
`119 + 26 + 6 = 151`. The final inventory must contain exactly 151 direct
invocations and 151 unique stable names.

The retained A11 is the amended binary-safe
`duplicate_repository_marker_rejected` fixture. Unit C must preserve its raw
per-entry blob scan and the three-state corruption-anchor-to-adjacent-child
weakening proof. The separately credited published
`duplicate active marker is full` case remains the visible same-Work-Order
duplicate fixture; neither credit aliases or replaces the other.

Unit C's permanent report must identify the four retired published names
verbatim, retired Unit A names A05 and A12, added names C01 through C05 and C12,
starting count 151, retired count 6, added count 6, final invocation count 151,
and final unique-name count 151. The inventory gate must reject any unexpected
retirement, retention, alias, or addition. It must fail if any retired
published name, A05, or A12 remains credited; any C case is missing; a retired
name is redirected to a canonical fixture; A01-A03 are counted beside retired
published aliases; either published two-commit alias remains; any C case lacks
one direct declaration and invocation; or either final count differs from 151.

Unit C review must prove non-overlap explicitly. A01 replaces the retired root
header case, A02 replaces the retired root gate case, and A03 replaces both
retired root two-commit aliases; only the A credit survives canonical-only
authority. C01 owns nested basename-prefix case, C02 owns exact zero, A14 owns
a positive integer with a leading zero, C03 owns a signed number, C04 owns
embedded/trailing whitespace, C05 owns a root Work-Order-like path beside valid
canonical authority, and C12 owns a legacy-root parent after fallback removal.
Each requires a targeted one-property mutation capable of admitting only its
owned invalid class while preserving the other frozen rejections. A broad regex
weakening may fail several cases but cannot substitute for these targeted
mutations.

### Required positive evidence

Permanent tests must prove:

1. A01, A02, and A03 remain the sole credited canonical nested header-only,
   gate-only, and two-commit status positives;
2. canonical non-status changes classify full;
3. the published 9-through-20 closed inventory plus active WO21 resolves
   deterministically;
4. full-lane evidence fields remain empty/zero and fast bindings authenticate
   the exact nested chain;
5. adjacent non-Work-Order names remain ignored;
6. A06's canonical successor parent and child both resolve and the issuance
   remains `full/no_status_transition` after legacy support is removed;
7. retained published `one full anchor plus exact header and gate update is
   fast` uses a canonical fixture and owns only its combined single-transition
   property;
8. the complete declared inventory follows the exact six-for-six ledger above,
   remains exactly 151 cases and 151 unique stable names, and runs twice
   deterministically with zero omissions or duplicates; and
9. the permanent report names all six exact retirements and six exact additions
   and reports starting 151, retired 6, added 6, final invocation count 151,
   and final unique-name count 151.

### Required adversarial evidence

Unit C must permanently reject:

- `WORKORDER.md`, `WORKORDER_21.md`, and every other root Work Order;
- C01's lowercase active basename prefix independently of directory/extension
  case;
- C02's exact zero number independently of A14's leading-zero positive number;
- C03's signed number;
- C04's whitespace before `.md`;
- C05's unmarked regular root `WORKORDER_22.md` added beside an otherwise valid
  canonical inventory;
- C12's legacy-root parent after fallback removal, even when its child is the
  valid canonical shape formerly used for migration;
- an unnumbered canonical file;
- leading-zero, zero, signed, spaced, or otherwise malformed number;
- wrong-case directory, basename, or extension;
- nested backup/suffix and extra-extension file;
- path containing traversal-like segments, backslashes, repeated separators,
  or separator ambiguity;
- Work Order in the wrong active/closed directory;
- closed file carrying a marker;
- closed file copied into active;
- two active numbered files or two active markers;
- active deletion or rename without a successor;
- removal of the sole active marker while keeping the exact active path;
- successor child retaining the predecessor marker;
- successor child removing any predecessor byte beyond the marker;
- successor creation without closing/unmarking the predecessor;
- predecessor closure/unmarking without a valid successor;
- predecessor source/closed-destination number mismatch;
- active status edit plus unrelated closed edit;
- status edit plus any Work Order move;
- parent/child disagreement about active path;
- symlink, submodule, mode, invalid UTF-8, BOM, or conflict substitution; and
- weakening/deleting the canonical-only check while merely editing expected
  test text.

At least one direct mutation must reintroduce the precise legacy root branch in
production and make C05 or C12 fail. Weakening the root-path ban while changing
only expected text must fail. At least one mutation must relax the active/closed
directory distinction and fail at its intended case. The targeted C01-C04
mutations above must each admit only their owned invalid class while preserving
the other case-sensitive and numeric path rejections.

### Platform, review, and lifecycle

- PowerShell 5.1 parsing and behavior are mandatory on Windows; Ubuntu and
  Windows CI must agree.
- Focused local classifier tests and source/configuration audits precede a
  fresh independent complete-tree review.
- That review must authenticate all four exact published retirements, exact
  A05/A12 retirement, exact C01-C05/C12 additions, the `119 + 26 + 6 = 151`
  composition, the 151-name report, every non-overlap ruling, and every
  load-bearing replacement mutation; no alternate replacement or discretionary
  substitution is allowed.
- The accepted implementation commit publishes separately and must receive
  full terminal-green Ubuntu/Windows CI.
- Its closure/status record is separately authored, projected, committed,
  published, and proven by fast CI before WO21 can close or a successor can be
  issued.
- After WO21 closes, successor issuance still requires a separately authorized
  full-lane transition following the exact one-marker-line removal contract;
  closure does not imply creation of WO22.

Exact proposed Unit C commit subject:

```text
ci(workorders): require canonical work order paths
```

### Unit C exclusions

No Work Order move or content edit; no policy/source/governance/workflow/
preflight edit; no archive/stash cleanup; no branch deletion; no successor Work
Order; no compiler work.

### Unit C harness-recovery amendment (BDFL-directed, 2026-08-15)

#### Terminal evidence record

Unit C began only after its full entry lifecycle completed. The exact status
commit `29a6c011490f9bc4698a885d05321a3c69135840` was published, and workflow
`31860185064`, attempt 1, tested that exact SHA and concluded `success`.
Ubuntu job `94952085370` and Windows job `94952085413` both completed the fast
status-only lane successfully. The BDFL then issued the required fresh Unit C
signal.

The Unit C implementation sequence completed red at three distinct evidence-
harness boundaries. None was a production-classifier semantic failure:

1. The first candidate-matrix run stopped before case 3 because the published
   closed-inventory fixture compared Git's lexicographic tree order with
   numeric Work Order 9-through-20 order. The bounded correction made that
   exact inventory assertion order-independent while retaining exact ordinal,
   case-sensitive set equality, count 12, and rejection of missing, extra,
   duplicate, substituted, wrong-case, wrong-directory, malformed, and wrong-
   number paths.
2. The second candidate-matrix run passed cases 1 through 124 and stopped at
   A07 because `New-CanonicalTestRepository -WithAdjacentPath` wrote
   `workorders/active/WORKORDERING.md` after the returned anchor was committed.
   The measured child therefore contained both the adjacent-path addition and
   the intended active-status edit. The bounded correction committed the
   adjacent fixture first, returned that commit as A07's anchor, retained it
   byte-identically in the child, and restored the exact one-record measured
   diff `M workorders/active/WORKORDER_21.md`.
3. The restarted matrix and initial mutation sequence then stopped because the
   mutation harness did not transport the exact disposable repository's
   process-local `safe.directory` context into a nested child that invoked
   `git show`. Git rejected the nested access before the intended mutation
   assertion was reached. This is an evidence-context transport failure, not a
   production-classifier or canonical-path failure.

The corrected permanent matrix itself exited 0. Exactly 151 direct invocations
and 151 case-sensitive unique stable names passed twice deterministically. The
composition was exactly 119 retained published cases, 26 retained Unit A
cases, and six Unit C cases: `119 + 26 + 6 = 151`. All six retirements and six
additions matched the frozen ledger. Binary-safe A11 and the separately
credited visible duplicate-marker case passed. Initial C01-C04, active/closed
directory-distinction, and C05 mutation evidence passed before the third stop.

The remaining legacy-root-branch mutation and A11 binary-excluding mutation
were not reached. Text hygiene, public readiness, alpha claims, release
readiness, and Fast were not run on this candidate. The single direct Unit C
Fast allowance remains unconsumed. No green credit is claimed for an
unreached check or mutation.

#### Parked candidate identity

The stopped candidate is preserved without correction at `stash@{0}` with:

- stash commit: `b9093901b8c92c626c3c23ee1a52366d2e54f698`;
- complete tree: `3342bcda2baabbb416b6797b5ed3346ec9e2c0c9`;
- first parent: `29a6c011490f9bc4698a885d05321a3c69135840`;
- index parent: `b5d5909ebe36e7b5b97802ac76083ea9e7c2e090`;
- first-parent and index-parent tree:
  `78c0ab10681128779224aaca2674927c9bd9b017`;
- independently reconstructed scoped two-path tree:
  `700eaf0dc131f546295ff244646325eae50ab5ce`;
- raw statistics: `+265/-102`; and
- whitespace-insensitive statistics: `+263/-100`.

Its only changed paths are regular tracked `100644` files:

| Path | Blob | SHA-256 | Raw statistics | Whitespace-insensitive |
| --- | --- | --- | ---: | ---: |
| `tools/check_workorder_status_boundary.ps1` | `62b3d964e116a1c19190945550151b456ad5aa99` | `c5edf187b8a799d361a6a02dd67bf572711328d96b3267846e1352a673ea950b` | `+1/-26` | `+1/-26` |
| `tools/test_workorder_status_boundary.ps1` | `21dbcd830a3384a91c98b563e437c1f8e9793f20` | `83e3d13dd97d9bd70e822f957b9127324e543f0e5ff7133cedb47a00561adaaf` | `+264/-76` | `+262/-74` |

Recovery must apply this exact stash by commit identity, never pop it. The five
older stash commits remain byte-identical and retain their relative order:

1. `799d4eaa2fb473633b41bbf17ad82e67fe2386a3`;
2. `f9b310902f804a0b8b7a3bf58910c7ec4f639c18`;
3. `303ee9af93696409bea66d3f8a379cb1a8cf8e1a`;
4. `bd6d2722cffa50da8463201204a48f4a7305ae1b`; and
5. `73101039f5e3faf0c802d4f723add1b891c51602`.

#### Frozen recovery architecture

The production candidate
`tools/check_workorder_status_boundary.ps1` must remain byte-identical at blob
`62b3d964e116a1c19190945550151b456ad5aa99` throughout recovery. Only
`tools/test_workorder_status_boundary.ps1` may receive the bounded evidence-
context correction. The canonical-only resolver, six-for-six ledger, A11
binary-safe behavior, A07 correction, and order-independent inventory
correction do not change.

The recovery must use one repository-owned Git invocation helper for every
direct and nested Git command used by Unit C mutation evidence. An exact
equivalent organization is acceptable only if one shared contract remains
mechanically observable. The helper must:

1. resolve the requested disposable repository with
   `System.IO.Path.GetFullPath`, require that exact directory to exist, reject
   NUL, CR, LF, wildcard, and ambiguous input, and derive trust only from that
   resolved path;
2. normalize the exact trust value for Git without widening path identity and
   require a post-invocation `rev-parse --show-toplevel` result to match the
   same resolved repository under the host's path-comparison rules;
3. invoke Git with command-scoped, process-local trust equivalent to
   `git --no-replace-objects -c safe.directory=<exact-repository> -C
   <exact-repository> ...`;
4. quote every native argument with a PowerShell-5.1-safe algorithm, including
   empty values, whitespace, quotes, and trailing backslashes;
5. capture native stdout and stderr as distinct complete byte streams, retain
   the exact child exit code, drain both streams without deadlock, and emit
   exactly one structured result object without helper or task-return objects
   leaking into the PowerShell pipeline;
6. make nonzero exit, unexpected stderr, and required-but-missing stdout
   independently observable and incapable of satisfying a success assertion;
7. require nested PowerShell probes to receive the exact repository path
   explicitly and to invoke Git through the same helper and validation
   contract as direct probes, never through a raw ambient `git show`; and
8. leave caller environment variables and Git configuration byte-for-byte
   unchanged after success or failure.

Trusting `*`, a parent directory, another disposable repository, or any other
broader/different value is forbidden. Global or system `safe.directory`
mutation, persisted repository or user configuration, credential changes,
parent-directory trust, environment leakage, and unrelated PATH or shell-tool
repair are forbidden. If a process-local environment transport is used as an
exact equivalent rather than command-scoped `-c`, every preexisting value must
be authenticated, only the exact repository value may be added, and every
value must be restored in `finally`; malformed, orphaned, broader, or
mismatched configuration fails closed.

#### Disposable projection and recovery budget

A disposable external projection established this shape without applying the
stash or changing either candidate script. Its final proof script was 309
LF-only lines and 10,177 bytes, with SHA-256
`d5a0c7023f5fe0b5557a6e38759eee2a2d37e6f2136112a9a4bc7c377248b488`
and Git-style blob identity `1e7b7b8d7541b2172439b97001d048f8d35407e6`.
Windows PowerShell 5.1 and the installed current PowerShell each exited 0 and
reproduced the same payload SHA-256
`46581d19ee1ff1c2b7006377d0f486a108fc90346db020495699ad055b4869aa`.
The projection is evidence, not a required repository artifact, and must be
removed after authorship.

The projection includes standalone repository creation and assertions that the
permanent harness already owns. The smallest credible integration ceiling is
therefore an additional `+300/-100` in
`tools/test_workorder_status_boundary.ps1` relative to parked blob
`21dbcd830a3384a91c98b563e437c1f8e9793f20`, under both raw and whitespace-
insensitive accounting. That yields final conservative ceilings relative to
the Unit C parent of `+564/-176` for the test script and `+565/-202` combined
with the unchanged production candidate. The existing Unit C ceilings of
test `+600/-520` and combined `+700/-700` remain authoritative; no revised
unit budget, third path, or production allowance is needed. The recovery delta
is a ceiling, not an insertion target, and unrelated refactoring remains
forbidden.

#### Load-bearing recovery evidence

Before the complete matrix restarts, focused disposable evidence must prove:

1. an honest nested `git show` succeeds while assuming a different owner only
   because the exact resolved disposable repository receives scoped trust;
2. removing only trust propagation reaches the recorded dubious-ownership
   failure before any mutation assertion, while the honest control remains
   green;
3. substituting a different disposable directory for the trust value fails;
4. wildcard, parent-directory, and other broader trust are rejected by the
   permanent harness rather than merely avoided by the honest fixture;
5. nonzero exit, stderr, and missing stdout remain separately captured and
   each fails the appropriate success contract;
6. direct and nested invocations traverse the same helper and return byte-
   identical output for the same `git show` request;
7. process environment and local, global, and system Git configuration are
   unchanged after every successful and failing probe; and
8. the remaining precise legacy-root-branch mutation and A11 binary-excluding
   mutation both reach their intended classifier assertions without weakening
   or rewriting those assertions.

The trust-removal proof must fail for the same reason recorded by the third
completed-red sequence. A syntax error, missing Git executable, malformed
fixture, missing commit, path-resolution error, hook/PATH failure, or changed
expected result earns no credit. The legacy and binary-excluding mutations
remain distinct load-bearing requirements and must not be combined.

#### Exact recovery lifecycle

Recovery follows this sequence with no implied next step:

1. Author this Work Order amendment only.
2. Obtain one fresh independent amendment review.
3. Under separate BDFL authority, create one local documentation commit with
   subject `docs(workorder): define unit c harness recovery`.
4. Separately publish that exact amendment by normal non-force `main`
   fast-forward and obtain terminal-green full Ubuntu and Windows CI.
5. Create a separate immutable publication-status record.
6. Separately publish that status record and obtain terminal-green fast Ubuntu
   and Windows CI.
7. Obtain a fresh explicit BDFL Unit C recovery signal.
8. Apply, but do not pop, stash
   `b9093901b8c92c626c3c23ee1a52366d2e54f698`.
9. Authenticate the exact parked two-path candidate, blobs, scoped tree, and
   raw and whitespace-insensitive statistics.
10. Preserve the production script byte-for-byte and apply only the frozen
    test-harness context correction.
11. Run the focused trust/context mutations above.
12. Restart the complete 151-case matrix and the required mutation sequence
    from their beginning; no prior partial run contributes green credit.
13. Run every standing Unit C check and the single still-unconsumed Fast
    allowance in the mandated order.
14. Leave the exact two-path candidate unstaged for one fresh independent
    complete-tree review.
15. Continue implementation commit, publication, status, and WO21 closeout only
    through their existing separate gates.

A failure at any recovery step stops without repair, rerun, scope expansion,
fallback, assertion weakening, alternate trust, or later-work authority. This
amendment authorizes no stash application, candidate edit, matrix execution,
mutation, Fast, commit, push, closeout, WO22, global harness consolidation,
compiler work, semantic-coordinate research, archive mutation, or stash
cleanup.

### Unit C evidence-transport redesign amendment (BDFL-directed, 2026-08-15)

This amendment supersedes only the first recovery amendment's evidence-
transport architecture, disposable projection, recovery-delta ceiling,
load-bearing transport evidence, and implementation base/lifecycle. The Unit C
production semantics, two-path envelope, canonical-only resolver, six-for-six
ledger, 151-case accounting, A07 correction, binary-safe A11, mutation matrix,
unit budgets, commit subject, and exclusions remain byte-for-byte authoritative.
It is not a migration rollback and does not authorize broad harness
consolidation.

#### Terminal `REDESIGN` disposition

The independent scope/budget review found no defect in the frozen Unit C
production blob or permanent classifier matrix. The candidate matrix had
already passed all 151 direct invocations twice deterministically, and the
single Unit C Fast allowance remained unconsumed. The terminal disposition was
`REDESIGN`, not a budget increase, because the added mutation-evidence
transport had three P1 defects:

1. In rejected test blob
   `9f629012439af424e5364d4b202f7f4afd03c940`, line 692 begins the native
   quote probe and line 693 passes
   `-Arguments @('-NoProfile', '-File', $QuoteProbePath) + $QuoteValues`.
   Windows PowerShell 5.1 binds `+ $QuoteValues` as the next parameter unless
   the complete array concatenation is parenthesized. The assertion therefore
   did not exercise the intended argument vector.
2. The direct and nested probes executed as the same repository owner. Exact
   `safe.directory` text was present, but removing it did not activate Git's
   dubious-ownership boundary, so the trust evidence was not load-bearing.
3. Failed Git invocations retained exit status and stderr, but repository
   validation ran only after a successful operation. A failed child therefore
   did not independently prove which repository produced the failure.

The review attributed the rejected 404-line recovery addition as follows:

| Rejected addition category | Lines |
| --- | ---: |
| Native stdout/stderr/exit capture | 68 |
| PowerShell 5.1 native quoting | 36 |
| Repository and trust validation | 79 |
| Shared direct/nested transport | 43 |
| Environment and Git-configuration restoration | 32 |
| Focused assertions | 112 |
| Integration glue | 26 |
| Mechanically duplicate byte-hash block | 8 |
| **Total** | **404** |

Only the eight-line hash block was mechanically redundant. Removing it did not
make the old `+300` recovery ceiling satisfiable, and compaction could not cure
the three P1s. The budget-only amendment path is closed.

#### Rejected and clean parked identities

The rejected candidate is preserved at `stash@{0}` as commit
`44262ceec1e895a3120133e8676387f2786ae3d0`, complete tree
`aa67b506da93174273eb4bdfdbd74e1ed259f90c`, first parent
`7877e346ffc86418a14a06e5bf00aa7741311eae`, and index parent
`7a2247f97bb1c1b97d42d7dc914f19bb60639a0e`. Its independently reconstructed
two-path scoped tree is `103acfb4f5b83f56699140363164fb25c6275dc3`.
It contains only regular `100644` paths:

| Path | Blob | SHA-256 | Raw | Whitespace-insensitive |
| --- | --- | --- | ---: | ---: |
| `tools/check_workorder_status_boundary.ps1` | `62b3d964e116a1c19190945550151b456ad5aa99` | `c5edf187b8a799d361a6a02dd67bf572711328d96b3267846e1352a673ea950b` | `+1/-26` | `+1/-26` |
| `tools/test_workorder_status_boundary.ps1` | `9f629012439af424e5364d4b202f7f4afd03c940` | `39701cf2902abc396442b3459d52af1a6272389d2cac3a508d4b93ae6904c692` | `+664/-111` | `+662/-109` |
| **Combined** | | | **`+665/-137`** | **`+663/-135`** |

The clean semantic candidate remains at `stash@{1}` as commit
`b9093901b8c92c626c3c23ee1a52366d2e54f698`, complete tree
`3342bcda2baabbb416b6797b5ed3346ec9e2c0c9`, and scoped tree
`700eaf0dc131f546295ff244646325eae50ab5ce`. Its production blob is the same
`62b3d964e116a1c19190945550151b456ad5aa99`; its test blob is
`21dbcd830a3384a91c98b563e437c1f8e9793f20`, SHA-256
`83e3d13dd97d9bd70e822f957b9127324e543f0e5ff7133cedb47a00561adaaf`.
Its combined raw and whitespace-insensitive statistics remain respectively
`+265/-102` and `+263/-100`. Only this clean stash is eligible as a future
implementation base. The rejected stash is immutable evidence and must never
be applied as the redesign implementation.

#### Selected ownership-adversary architecture

The selected design uses one repository-owned synchronous native-process
primitive and one mutation-process transport. It does not retain the old
direct-versus-nested equivalence assertion: direct and nested execution under
one owner did not constrain trust, and the redesign has only one mutation child
path. The transport contract is:

1. Resolve an absolute disposable non-bare worktree path with
   `System.IO.Path.GetFullPath`; require the exact worktree and its `.git`
   directory to exist; reject NUL, CR, LF, wildcard, relative, missing, and
   ambiguous paths; normalize only directory separators for Git's
   `safe.directory` value.
2. Require one lowercase 40-hex expected commit. Before launching the mutation
   child, independently execute
   `git --no-replace-objects --git-dir=<resolved-repository>/.git rev-parse
   --verify <expected>^{commit}` and require exit 0, zero stderr, and exactly
   that commit on stdout. This object-database operation does not perform the
   later worktree discovery and therefore authenticates canonical path plus
   immutable commit even when the worktree operation fails at ownership.
3. Fail closed unless the installed Git's controlled
   `GIT_TEST_ASSUME_DIFFERENT_OWNER=1` adversary is executable: the same-owner
   control without it must succeed, the adversarial child without added trust
   must fail nonzero with `dubious ownership` on stderr, and adding only exact
   repository trust must make the same operation succeed. If any platform's
   Git lacks or ignores this adversary, Unit C stops; no weaker substitute is
   allowed.
4. Authenticate the complete inherited `GIT_CONFIG_COUNT` vector. Its count
   must be canonical nonnegative decimal; every indexed key and value must
   exist; orphaned indexed entries fail closed. The mutation child alone
   receives `GIT_TEST_ASSUME_DIFFERENT_OWNER=1` and one appended config entry:
   `GIT_CONFIG_COUNT=N+1`, `GIT_CONFIG_KEY_N=safe.directory`, and
   `GIT_CONFIG_VALUE_N=<exact-normalized-repository>`. Trust removal deletes
   only that appended entry. Parent environment and local, global, and system
   Git configuration never change.
5. Require the supplied trust path to resolve exactly to the selected worktree
   under the host's path-comparison rules before any child launch. Wildcard
   trust is rejected syntactically. Parent, broader, and different-repository
   trust fail exact equality. Git's own acceptance of `safe.directory=*` is
   not evidence and must never be reached through the helper.
6. Quote every native argument with the Windows command-line algorithm, and
   pass concatenated argument vectors only as a fully parenthesized expression.
   Empty strings, whitespace, embedded quotes, and trailing backslashes are
   mandatory focused cases under Windows PowerShell 5.1.
7. Capture the mutation PowerShell child's stdout and stderr as separate
   complete byte streams with asynchronous draining, close stdin, wait for the
   child and both drain tasks, retain the exact exit code, and return one
   structured result containing canonical repository path, authenticated
   commit, exit, stdout bytes, and stderr bytes.
8. A success contract requires authenticated identity, exit 0, zero stderr,
   and exactly `<expected-commit>\n` stdout. Nonzero exit, any stderr, missing
   stdout, malformed stdout, extra stdout, wrong identity, launch failure, or
   incomplete drain is independently observable and fails.

No global, system, user, or repository Git configuration write is permitted.
No credential, PATH, askpass, shell, or persistent environment repair belongs
to this design. The transport may be used only by Unit C's focused mutation
evidence; it is not a general subprocess framework.

#### Executable disposable projection

The author evaluated both a real Windows SID boundary and the portable
controlled adversary. A disposable repo owned by `OCEAN\ocean` was accessed by
`OCEAN\CodexSandboxOffline`: no trust exited 128 at `dubious ownership`;
independent `--git-dir` identity returned exact commit
`a280aaff99dfc30280d1f3406f84cadefefd1ec3`; exact forward-slash-normalized
repository trust returned that commit with exit 0. Parent trust remained red.
Git's wildcard trust succeeded, confirming why the helper must reject it before
launch rather than merely omit it.

Git `2.45.1.windows.1` also proved the controlled adversary in a same-owner
repo: normal access returned exact commit
`1ff2df6fe62e7ce93a944b26849d9e33853f8e17` with exit 0, while adding only
`GIT_TEST_ASSUME_DIFFERENT_OWNER=1` exited 128 at dubious ownership. The parent
environment was restored after that probe.

The final integrated projection preserves production blob
`62b3d964e116a1c19190945550151b456ad5aa99` byte-for-byte and projects the test
script to 2,155 LF-only lines, 110,883 bytes, zero CR, final LF, SHA-256
`d473bfd9102ddc1e988bf9f62c751ff0e3875aa0797193bb833afe6f9e0c3f90`, and
Git-style blob `8d28cc9854863255719b7ccd55a42ea304afe6cf`. It parses with zero errors. Its
exact diffs are:

| Comparison base | Test raw | Test whitespace-insensitive | Combined raw | Combined whitespace-insensitive |
| --- | ---: | ---: | ---: | ---: |
| Published Unit C parent scripts | `+593/-76` | `+591/-74` | `+594/-102` | `+592/-100` |
| Clean parked candidate `b9093901...` | `+329/-0` | `+329/-0` | `+329/-0` | `+329/-0` |
| Rejected candidate `44262cee...` | `+275/-311` | `+273/-309` | `+275/-311` | `+273/-309` |

The exact 329-line recovery addition is attributable without overlap:

| Selected addition category | Lines |
| --- | ---: |
| PowerShell 5.1 native argument quoting | 32 |
| Synchronous native stdout/stderr/exit capture | 49 |
| Canonical repository, trust, and independent identity binding | 84 |
| Inherited Git-config vector authentication | 24 |
| Exact success-result contract | 12 |
| Environment and Git-configuration fingerprint | 24 |
| Focused adversary, corruption, and reachability fixtures/assertions | 103 |
| Permanent-suite integration call | 1 |
| **Total** | **329** |

An authoring-only focus entry exercised the same selected functions and was
removed from the integrated projection. Windows PowerShell 5.1.26100.9168 and
PowerShell 7.6.4 each exited 0 against the same fixture identity
`aac5844e02c0829a6f1e7a4971f8bfe751dc1ed2`. In both hosts the honest child
exited 0 with 41 stdout bytes, SHA-256
`d1f70ce144864470f386b4ab1d9668efd7e89561d4b3aeb3ff4c3cadff045556`, and
zero stderr bytes, SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
Trust removal preserved the same independently authenticated identity, exited
128, emitted zero stdout bytes with the empty SHA-256, and emitted 334 stderr
bytes containing the precise dubious-ownership failure. The PS5.1 stderr
SHA-256 was `ebd2f3a49b9ab7da632006a8b1a8cb74255d35bcab9bd1f6a41d9c3146e1cc0d`;
the PowerShell 7.6.4 stderr SHA-256 was
`51474e760bc6f214f7ea150195341b2ff0407c015d2ef00ff31be19fe18f5502`.
Host newline transport explains the byte-hash difference; both retained the
same Git failure, exit, zero stdout, and authenticated repository identity.

Both hosts passed the exact empty/space/embedded-quote/trailing-backslash
argument vector, rejected different, parent, and wildcard trust, rejected
nonzero, missing, and malformed result shapes, preserved separate streams, and
reached distinct `legacy-assertion-reached` and
`binary-assertion-reached` child assertions. The author environment contained
a valid inherited two-entry Git config vector; the projection appended one
child-only entry and proved the complete parent environment plus local, global,
and system config fingerprints unchanged afterward.

The Windows hosts and real Windows SID boundary were exercised. Ubuntu/Linux
was not available locally and remains mandatory full-CI evidence; absence or
different behavior of `GIT_TEST_ASSUME_DIFFERENT_OWNER` there is a mandatory
stop, not permission to select another ownership simulation. The actual
legacy-root and binary-marker classifier mutations were intentionally not run
during document authorship; their exact future assertions remain mandatory.

#### Projection reproducibility correction (BDFL-directed, 2026-08-15)

The first fresh review accepted the redesign behaviorally but could not
authenticate its exact projected bytes because only the target identities were
retained. This correction changes no architecture, test meaning, line
attribution, budget, or lifecycle rule. It freezes the exact transformation
from clean parked test blob
`21dbcd830a3384a91c98b563e437c1f8e9793f20` to projected test blob
`8d28cc9854863255719b7ccd55a42ea304afe6cf` as one canonical full-index Git
patch.

The fresh review of that retained artifact reproduced the patch, target,
architecture, attribution, and budgets, but found one decoder-ordering P1:
`ReadAllLines` normalized CRLF framing before the envelope checks could reject
it. The bounded correction below authenticates the single raw Work Order byte
buffer before any line-oriented operation or disposable side effect, then
derives every parsed line from that same strictly decoded buffer. It changes no
payload or projected implementation byte.

The decoded patch identity is exact:

- source blob:
  `21dbcd830a3384a91c98b563e437c1f8e9793f20`;
- target blob:
  `8d28cc9854863255719b7ccd55a42ea304afe6cf`;
- byte count: 14,210;
- LF count: 347;
- CR count: 0;
- final LF: present; and
- SHA-256:
  `ce54a995e6ddaad260059f2b935899846055d0fa3f488958c46299354f724934`.

The RFC 4648 Base64 representation is canonical and frozen as exactly 18,948
characters on 250 lines: lines 1 through 249 contain exactly 76 characters,
line 250 contains exactly 24 characters, and no other whitespace is permitted.
The final encoded line carries the only padding. Decoding must reject invalid
alphabet characters, malformed or noncanonical padding, a wrong line count or
line length, extra whitespace, a re-encoding mismatch, or any decoded
length/hash/newline mismatch.

The following PowerShell 5.1-compatible procedure is normative. Run it from
the canonical repository root with no real stash application. It materializes
the two clean projected input blobs in one disposable repository through the
canonical object database, authenticates the clean stash path, writes the
decoded patch only inside that disposable repository, requires
`git apply --check --index` before applying, and restores the caller's
alternate-object environment in `finally`.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Repository = (Resolve-Path '.').Path
$SafeRepository = $Repository.Replace('\', '/')
$WorkOrder = Join-Path $Repository 'workorders/active/WORKORDER_21.md'
$Disposable = [IO.Path]::GetFullPath(
  (Join-Path ([IO.Path]::GetTempPath()) 'hum-wo21-unit-c-projection')
)
$TestPath = 'tools/test_workorder_status_boundary.ps1'
$ProductionPath = 'tools/check_workorder_status_boundary.ps1'
$PublishedHead = '7877e346ffc86418a14a06e5bf00aa7741311eae'
$BaseTest = '21dbcd830a3384a91c98b563e437c1f8e9793f20'
$TargetTest = '8d28cc9854863255719b7ccd55a42ea304afe6cf'
$PublishedTest = '762ef996b926a3f9c5bdf69c8abee9e13cafb67e'
$PublishedProduction = '6b7d421d7d153e8ae660e70a3fccb634983df7a3'
$ProjectedProduction = '62b3d964e116a1c19190945550151b456ad5aa99'
$PatchSha = 'ce54a995e6ddaad260059f2b935899846055d0fa3f488958c46299354f724934'
$BeginLiteral = '<!-- wo21-unit-c-projection-patch-base64:begin -->'
$EndLiteral = '<!-- wo21-unit-c-projection-patch-base64:end -->'
$StrictUtf8 = New-Object Text.UTF8Encoding($false, $true)

function Get-Wo21BytesSha256 {
  param([byte[]] $Bytes)
  $Sha = [Security.Cryptography.SHA256]::Create()
  try {
    return ([BitConverter]::ToString($Sha.ComputeHash($Bytes)) -replace '-', '').ToLowerInvariant()
  } finally {
    $Sha.Dispose()
  }
}

function Get-Wo21ExactLine {
  param([string[]] $Lines, [string] $Text)
  $Matches = @(
    for ($Index = 0; $Index -lt $Lines.Count; $Index += 1) {
      if ($Lines[$Index] -ceq $Text) { $Index }
    }
  )
  if ($Matches.Count -ne 1) { throw "expected one exact line: $Text" }
  return [int]$Matches[0]
}

$DocumentBytes = [IO.File]::ReadAllBytes($WorkOrder)
if ($DocumentBytes.Length -eq 0) {
  throw 'Work Order raw byte stream is empty'
}
if ([Array]::IndexOf($DocumentBytes, [byte]13) -ge 0) {
  throw 'Work Order raw byte stream contains CR'
}
if ($DocumentBytes[$DocumentBytes.Length - 1] -ne 10) {
  throw 'Work Order raw byte stream lacks its final LF'
}
try {
  $DocumentText = $StrictUtf8.GetString($DocumentBytes)
} catch {
  throw 'Work Order raw byte stream is not strict UTF-8'
}
$DocumentLines = @(
  $DocumentText.Substring(0, $DocumentText.Length - 1).Split([char]10)
)
$Begin = Get-Wo21ExactLine $DocumentLines $BeginLiteral
$End = Get-Wo21ExactLine $DocumentLines $EndLiteral
if (
  $End - $Begin -ne 253 -or
  $DocumentLines[$Begin + 1] -cne '```text' -or
  $DocumentLines[$End - 1] -cne '```'
) {
  throw 'projection Base64 envelope is malformed'
}
$EncodedLines = @($DocumentLines[($Begin + 2)..($End - 2)])
if ($EncodedLines.Count -ne 250) {
  throw 'projection Base64 line count changed'
}
for ($Index = 0; $Index -lt 249; $Index += 1) {
  if (
    $EncodedLines[$Index].Length -ne 76 -or
    $EncodedLines[$Index] -cnotmatch '^[A-Za-z0-9+/]{76}$'
  ) {
    throw "projection Base64 line $($Index + 1) is noncanonical"
  }
}
if (
  $EncodedLines[249].Length -ne 24 -or
  $EncodedLines[249] -cnotmatch '^[A-Za-z0-9+/]{23}=$'
) {
  throw 'projection Base64 final line is noncanonical'
}
$Encoded = @($EncodedLines) -join ''
if (
  $Encoded.Length -ne 18948 -or
  $Encoded -cnotmatch '^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$'
) {
  throw 'projection Base64 payload is malformed'
}
try {
  $PatchBytes = [Convert]::FromBase64String($Encoded)
} catch {
  throw 'projection Base64 decoding failed'
}
if ([Convert]::ToBase64String($PatchBytes) -cne $Encoded) {
  throw 'projection Base64 is not canonical RFC 4648'
}
if (
  $PatchBytes.Length -ne 14210 -or
  (Get-Wo21BytesSha256 $PatchBytes) -cne $PatchSha -or
  @($PatchBytes | Where-Object { $_ -eq 10 }).Count -ne 347 -or
  @($PatchBytes | Where-Object { $_ -eq 13 }).Count -ne 0 -or
  $PatchBytes[$PatchBytes.Length - 1] -ne 10
) {
  throw 'decoded projection patch identity changed'
}

if (Test-Path -LiteralPath $Disposable) {
  throw "disposable projection path already exists: $Disposable"
}
$null = New-Item -ItemType Directory -Path $Disposable
$Git = (Get-Command git.exe -ErrorAction Stop).Source
& $Git -C $Disposable init --quiet
if ($LASTEXITCODE -ne 0) { throw 'disposable git init failed' }
& $Git -C $Disposable config core.autocrlf false
if ($LASTEXITCODE -ne 0) { throw 'disposable core.autocrlf setup failed' }

$StashBase = @(
  & $Git --no-replace-objects -c "safe.directory=$SafeRepository" -C $Repository `
    rev-parse "b9093901b8c92c626c3c23ee1a52366d2e54f698`:$TestPath"
)
if ($LASTEXITCODE -ne 0 -or $StashBase.Count -ne 1 -or $StashBase[0] -cne $BaseTest) {
  throw 'clean stash test blob does not match the frozen base'
}

$OldAlternate = [Environment]::GetEnvironmentVariable(
  'GIT_ALTERNATE_OBJECT_DIRECTORIES',
  'Process'
)
$HadAlternate = $null -ne $OldAlternate
$SourceObjects = [IO.Path]::GetFullPath((Join-Path $Repository '.git/objects'))
$NewAlternate = if ([string]::IsNullOrEmpty($OldAlternate)) {
  $SourceObjects
} else {
  $OldAlternate + [IO.Path]::PathSeparator + $SourceObjects
}
try {
  [Environment]::SetEnvironmentVariable(
    'GIT_ALTERNATE_OBJECT_DIRECTORIES',
    $NewAlternate,
    'Process'
  )
  & $Git -C $Disposable update-ref refs/heads/main $PublishedHead
  if ($LASTEXITCODE -ne 0) { throw 'published projection ref setup failed' }
  & $Git -C $Disposable symbolic-ref HEAD refs/heads/main
  if ($LASTEXITCODE -ne 0) { throw 'published projection HEAD setup failed' }
  & $Git -C $Disposable read-tree $PublishedHead
  if ($LASTEXITCODE -ne 0) { throw 'published projection tree setup failed' }
  & $Git -C $Disposable checkout-index -a
  if ($LASTEXITCODE -ne 0) { throw 'published projection checkout failed' }

  & $Git -C $Disposable update-index --add --cacheinfo `
    "100644,$BaseTest,$TestPath"
  if ($LASTEXITCODE -ne 0) { throw 'clean test blob indexing failed' }
  & $Git -C $Disposable update-index --add --cacheinfo `
    "100644,$ProjectedProduction,$ProductionPath"
  if ($LASTEXITCODE -ne 0) { throw 'production blob indexing failed' }
  & $Git -C $Disposable checkout-index --force -- $TestPath $ProductionPath
  if ($LASTEXITCODE -ne 0) { throw 'projected script checkout failed' }
  & $Git -C $Disposable update-index --refresh
  if ($LASTEXITCODE -ne 0) { throw 'clean projection index refresh failed' }

  $BaseFile = Join-Path $Disposable $TestPath
  $ObservedBase = @(& $Git -C $Disposable hash-object -- $BaseFile)
  if (
    $LASTEXITCODE -ne 0 -or
    $ObservedBase.Count -ne 1 -or
    $ObservedBase[0] -cne $BaseTest
  ) {
    throw 'materialized clean test blob changed'
  }

  $PatchPath = Join-Path $Disposable 'wo21-unit-c-projection.patch'
  [IO.File]::WriteAllBytes($PatchPath, $PatchBytes)
  & $Git -C $Disposable apply --check --index $PatchPath
  if ($LASTEXITCODE -ne 0) { throw 'projection git apply check failed' }
  & $Git -C $Disposable apply --index $PatchPath
  if ($LASTEXITCODE -ne 0) { throw 'projection git apply failed' }

  $TargetFile = Join-Path $Disposable $TestPath
  $TargetBytes = [IO.File]::ReadAllBytes($TargetFile)
  $ObservedTarget = @(& $Git -C $Disposable hash-object -- $TargetFile)
  if (
    $LASTEXITCODE -ne 0 -or
    $ObservedTarget.Count -ne 1 -or
    $ObservedTarget[0] -cne $TargetTest -or
    $TargetBytes.Length -ne 110883 -or
    (Get-Wo21BytesSha256 $TargetBytes) -cne
      'd473bfd9102ddc1e988bf9f62c751ff0e3875aa0797193bb833afe6f9e0c3f90' -or
    @($TargetBytes | Where-Object { $_ -eq 10 }).Count -ne 2155 -or
    @($TargetBytes | Where-Object { $_ -eq 13 }).Count -ne 0 -or
    $TargetBytes[$TargetBytes.Length - 1] -ne 10
  ) {
    throw 'reconstructed projected test identity changed'
  }

  $CleanStat = @(& $Git -C $Disposable diff --numstat $BaseTest $TargetTest)
  $PublishedStat = @(
    & $Git -C $Disposable diff --numstat $PublishedTest $TargetTest
  )
  $ProductionStat = @(
    & $Git -C $Disposable diff --numstat `
      $PublishedProduction $ProjectedProduction
  )
  if (
    $LASTEXITCODE -ne 0 -or
    $CleanStat.Count -ne 1 -or
    $CleanStat[0] -cnotmatch "^329`t0`t" -or
    $PublishedStat.Count -ne 1 -or
    $PublishedStat[0] -cnotmatch "^593`t76`t" -or
    $ProductionStat.Count -ne 1 -or
    $ProductionStat[0] -cnotmatch "^1`t26`t"
  ) {
    throw 'reconstructed projection statistics changed'
  }
} finally {
  if ($HadAlternate) {
    [Environment]::SetEnvironmentVariable(
      'GIT_ALTERNATE_OBJECT_DIRECTORIES',
      $OldAlternate,
      'Process'
    )
  } else {
    [Environment]::SetEnvironmentVariable(
      'GIT_ALTERNATE_OBJECT_DIRECTORIES',
      $null,
      'Process'
    )
  }
}
```

Excluding its Markdown fences, the corrected normative procedure is exactly
8,503 bytes on 236 LF-only lines, has zero CR and a final LF, and has SHA-256
`a25ecb032641a053ed4e2c3de48c05093a931f85275ec203e618db11c629dc0f`.

The decoder ordering is load-bearing and must be reviewed before target
reconstruction. For each required shell, a reviewer must bind an extracted
copy of only the `$WorkOrder` and `$Disposable` declarations to one mutated
external Work Order and one unique, predeclared, absent scratch path. No other
procedure byte may change. These six sentinel path names are frozen relative
to one fresh external review root:

- `sentinel-payload-crlf-ps51` and `sentinel-payload-crlf-pwsh`;
- `sentinel-outside-crlf-ps51` and `sentinel-outside-crlf-pwsh`; and
- `sentinel-missing-final-lf-ps51` and
  `sentinel-missing-final-lf-pwsh`.

The bounded correction executed all six probes. In Windows PowerShell
5.1.26100.9168 and PowerShell 7.6.4 alike, changing one Base64 payload LF to
CRLF and inserting one CR before an LF outside the payload each exited 1 with
`Work Order raw byte stream contains CR`; removing only the Work Order's final
LF exited 1 with `Work Order raw byte stream lacks its final LF`. Every probe
produced zero stdout bytes, and its unique disposable sentinel was absent both
before and after the process. No cleanup of a created sentinel received credit.
The honest LF-only Work Order must still complete the retained reconstruction
under both shells before this evidence is accepted.

The reconstructed target itself fixes the attribution boundaries. A fresh
review must read its LF-delimited lines and require these exact, nonoverlapping
ranges:

| Category | Exact reconstructed line range | Lines |
| --- | --- | ---: |
| PowerShell 5.1 native argument quoting | 114-145 | 32 |
| Synchronous native stdout/stderr/exit capture | 146-194 | 49 |
| Canonical repository/trust/identity binding | 195-227 and 252-302 | 84 |
| Inherited Git-config vector authentication | 228-251 | 24 |
| Exact success-result contract | 303-314 | 12 |
| Environment/configuration fingerprint | 315-338 | 24 |
| Focused adversary/corruption/reachability evidence | 339-441 | 103 |
| Permanent-suite integration call | exact line 1051 | 1 |
| **Total** | | **329** |

The declaration boundaries at lines 114, 146, 195, 219, 228, 252, 303, 315,
339, and 442 must respectively be
`ConvertTo-RecoveryNativeArgument`, `Invoke-RecoveryNativeProcess`,
`Resolve-RecoveryRepository`, `Test-RecoveryPathEqual`,
`Get-RecoveryAmbientGitConfigCount`, `Invoke-RecoveryMutationProcess`,
`Assert-RecoveryMutationSuccess`, `Get-RecoveryStateSha256`,
`Assert-RecoveryTransportProjection`, and `Invoke-TestGit`. The integration
line must occur exactly once as
`Assert-RecoveryTransportProjection $TestRoot`. These boundaries prove the
category counts directly from the reconstructed bytes; a source-text table
alone is not evidence.

After reconstruction, execute the disposable target with its materialized
production blob under Windows PowerShell 5.1.26100.9168 and PowerShell 7.6.4.
Both executions must pass the focused ownership-adversary projection and the
complete 151-case deterministic suite. They must prove exact-trust success,
trust-removal dubious-ownership failure with retained repository identity,
fully parenthesized argument-vector fidelity, non-exact trust rejection,
malformed/missing-output rejection, caller environment/configuration
restoration, and distinct legacy-root and binary-marker assertion reachability.
Remove the entire disposable repository afterward.

The reconstructed target must still reproduce test `+329/-0` against the clean
stash blob and test `+593/-76` against the published parent. The unchanged
production projection remains `+1/-26`, so the combined parent-relative result
is exactly `+594/-102`. The attribution therefore continues to ground the
existing `+333/-4` recovery ceiling, `+597/-80` final test ceiling, and
`+598/-106` final combined ceiling with the same four insertion and four
deletion lines of honest integration headroom.

The canonical patch follows. Its sentinels, fence lines, payload line count,
wrapping, and bytes are immutable parts of this amendment.

<!-- wo21-unit-c-projection-patch-base64:begin -->
```text
ZGlmZiAtLWdpdCBhL3Rvb2xzL3Rlc3Rfd29ya29yZGVyX3N0YXR1c19ib3VuZGFyeS5wczEgYi90
b29scy90ZXN0X3dvcmtvcmRlcl9zdGF0dXNfYm91bmRhcnkucHMxCmluZGV4IDIxZGJjZDgzMGEz
Mzg0YTkxYzk4YjU2M2U0MzdjMWY4ZTk3OTNmMjAuLjhkMjhjYzk4NTQ4NjMyNTU3MTliN2NjZDU1
YTQyZWEzMDRhZmU2Y2YgMTAwNjQ0Ci0tLSBhL3Rvb2xzL3Rlc3Rfd29ya29yZGVyX3N0YXR1c19i
b3VuZGFyeS5wczEKKysrIGIvdG9vbHMvdGVzdF93b3Jrb3JkZXJfc3RhdHVzX2JvdW5kYXJ5LnBz
MQpAQCAtMTExLDYgKzExMSwzMzQgQEAgZnVuY3Rpb24gQXNzZXJ0LVB1Ymxpc2hlZENsb3NlZElu
dmVudG9yeVJlamVjdGVkIHsKICAgQXNzZXJ0LUJvdW5kYXJ5VGVzdCAkUmVqZWN0ZWQgIiROYW1l
IGludmVudG9yeSBjb3JydXB0aW9uIHdhcyBhY2NlcHRlZCIKIH0KIAorZnVuY3Rpb24gQ29udmVy
dFRvLVJlY292ZXJ5TmF0aXZlQXJndW1lbnQgeworICBwYXJhbShbQWxsb3dFbXB0eVN0cmluZygp
XVtzdHJpbmddICRBcmd1bWVudCkKKworICBpZiAoJEFyZ3VtZW50Lkxlbmd0aCAtZXEgMCkgeyBy
ZXR1cm4gJyIiJyB9CisgIGlmICgkQXJndW1lbnQgLW5vdG1hdGNoICdbXHMiXScpIHsgcmV0dXJu
ICRBcmd1bWVudCB9CisgICRCdWlsZGVyID0gTmV3LU9iamVjdCBTeXN0ZW0uVGV4dC5TdHJpbmdC
dWlsZGVyCisgIFt2b2lkXSRCdWlsZGVyLkFwcGVuZCgnIicpCisgICRCYWNrc2xhc2hlcyA9IDAK
KyAgZm9yZWFjaCAoJENoYXJhY3RlciBpbiAkQXJndW1lbnQuVG9DaGFyQXJyYXkoKSkgeworICAg
IGlmICgkQ2hhcmFjdGVyIC1lcSAnXCcpIHsKKyAgICAgICRCYWNrc2xhc2hlcyArPSAxCisgICAg
ICBjb250aW51ZQorICAgIH0KKyAgICBpZiAoJENoYXJhY3RlciAtZXEgJyInKSB7CisgICAgICBb
dm9pZF0kQnVpbGRlci5BcHBlbmQoKCgnXCcgKiAoKCRCYWNrc2xhc2hlcyAqIDIpICsgMSkpIC1q
b2luICcnKSkKKyAgICAgIFt2b2lkXSRCdWlsZGVyLkFwcGVuZCgnIicpCisgICAgICAkQmFja3Ns
YXNoZXMgPSAwCisgICAgICBjb250aW51ZQorICAgIH0KKyAgICBpZiAoJEJhY2tzbGFzaGVzIC1n
dCAwKSB7CisgICAgICBbdm9pZF0kQnVpbGRlci5BcHBlbmQoKCgnXCcgKiAkQmFja3NsYXNoZXMp
IC1qb2luICcnKSkKKyAgICAgICRCYWNrc2xhc2hlcyA9IDAKKyAgICB9CisgICAgW3ZvaWRdJEJ1
aWxkZXIuQXBwZW5kKCRDaGFyYWN0ZXIpCisgIH0KKyAgaWYgKCRCYWNrc2xhc2hlcyAtZ3QgMCkg
eworICAgIFt2b2lkXSRCdWlsZGVyLkFwcGVuZCgoKCdcJyAqICgkQmFja3NsYXNoZXMgKiAyKSkg
LWpvaW4gJycpKQorICB9CisgIFt2b2lkXSRCdWlsZGVyLkFwcGVuZCgnIicpCisgIHJldHVybiAk
QnVpbGRlci5Ub1N0cmluZygpCit9CisKK2Z1bmN0aW9uIEludm9rZS1SZWNvdmVyeU5hdGl2ZVBy
b2Nlc3MgeworICBwYXJhbSgKKyAgICBbc3RyaW5nXSAkRmlsZU5hbWUsCisgICAgW3N0cmluZ1td
XSAkQXJndW1lbnRzLAorICAgIFtzdHJpbmddICRXb3JraW5nRGlyZWN0b3J5LAorICAgIFtoYXNo
dGFibGVdICRDaGlsZEVudmlyb25tZW50ID0gQHt9CisgICkKKworICAkU3RhcnRJbmZvID0gTmV3
LU9iamVjdCBTeXN0ZW0uRGlhZ25vc3RpY3MuUHJvY2Vzc1N0YXJ0SW5mbworICAkU3RhcnRJbmZv
LkZpbGVOYW1lID0gJEZpbGVOYW1lCisgICRTdGFydEluZm8uQXJndW1lbnRzID0gKEAoJEFyZ3Vt
ZW50cyB8IEZvckVhY2gtT2JqZWN0IHsKKyAgICBDb252ZXJ0VG8tUmVjb3ZlcnlOYXRpdmVBcmd1
bWVudCAoW3N0cmluZ10kXykKKyAgfSkgLWpvaW4gJyAnKQorICAkU3RhcnRJbmZvLldvcmtpbmdE
aXJlY3RvcnkgPSAkV29ya2luZ0RpcmVjdG9yeQorICAkU3RhcnRJbmZvLlVzZVNoZWxsRXhlY3V0
ZSA9ICRmYWxzZQorICAkU3RhcnRJbmZvLkNyZWF0ZU5vV2luZG93ID0gJHRydWUKKyAgJFN0YXJ0
SW5mby5SZWRpcmVjdFN0YW5kYXJkSW5wdXQgPSAkdHJ1ZQorICAkU3RhcnRJbmZvLlJlZGlyZWN0
U3RhbmRhcmRPdXRwdXQgPSAkdHJ1ZQorICAkU3RhcnRJbmZvLlJlZGlyZWN0U3RhbmRhcmRFcnJv
ciA9ICR0cnVlCisgIGZvcmVhY2ggKCROYW1lIGluIEAoJENoaWxkRW52aXJvbm1lbnQuS2V5cykp
IHsKKyAgICAkU3RhcnRJbmZvLkVudmlyb25tZW50VmFyaWFibGVzW1tzdHJpbmddJE5hbWVdID0g
W3N0cmluZ10kQ2hpbGRFbnZpcm9ubWVudFskTmFtZV0KKyAgfQorICAkUHJvY2VzcyA9IE5ldy1P
YmplY3QgU3lzdGVtLkRpYWdub3N0aWNzLlByb2Nlc3MKKyAgJFByb2Nlc3MuU3RhcnRJbmZvID0g
JFN0YXJ0SW5mbworICBpZiAoLW5vdCAkUHJvY2Vzcy5TdGFydCgpKSB7CisgICAgJFByb2Nlc3Mu
RGlzcG9zZSgpCisgICAgdGhyb3cgJ3JlY292ZXJ5IHByb2Nlc3MgZGlkIG5vdCBzdGFydCcKKyAg
fQorICAkU3Rkb3V0ID0gTmV3LU9iamVjdCBTeXN0ZW0uSU8uTWVtb3J5U3RyZWFtCisgICRTdGRl
cnIgPSBOZXctT2JqZWN0IFN5c3RlbS5JTy5NZW1vcnlTdHJlYW0KKyAgdHJ5IHsKKyAgICAkU3Rk
b3V0VGFzayA9ICRQcm9jZXNzLlN0YW5kYXJkT3V0cHV0LkJhc2VTdHJlYW0uQ29weVRvQXN5bmMo
JFN0ZG91dCkKKyAgICAkU3RkZXJyVGFzayA9ICRQcm9jZXNzLlN0YW5kYXJkRXJyb3IuQmFzZVN0
cmVhbS5Db3B5VG9Bc3luYygkU3RkZXJyKQorICAgICRQcm9jZXNzLlN0YW5kYXJkSW5wdXQuQ2xv
c2UoKQorICAgICRQcm9jZXNzLldhaXRGb3JFeGl0KCkKKyAgICAkU3Rkb3V0VGFzay5XYWl0KCkK
KyAgICAkU3RkZXJyVGFzay5XYWl0KCkKKyAgICByZXR1cm4gW3BzY3VzdG9tb2JqZWN0XUB7Cisg
ICAgICBFeGl0Q29kZSA9ICRQcm9jZXNzLkV4aXRDb2RlCisgICAgICBTdGRvdXRCeXRlcyA9ICRT
dGRvdXQuVG9BcnJheSgpCisgICAgICBTdGRlcnJCeXRlcyA9ICRTdGRlcnIuVG9BcnJheSgpCisg
ICAgfQorICB9IGZpbmFsbHkgeworICAgICRTdGRvdXQuRGlzcG9zZSgpCisgICAgJFN0ZGVyci5E
aXNwb3NlKCkKKyAgICAkUHJvY2Vzcy5EaXNwb3NlKCkKKyAgfQorfQorCitmdW5jdGlvbiBSZXNv
bHZlLVJlY292ZXJ5UmVwb3NpdG9yeSB7CisgIHBhcmFtKFtzdHJpbmddICRQYXRoKQorCisgIGlm
ICgKKyAgICBbc3RyaW5nXTo6SXNOdWxsT3JXaGl0ZVNwYWNlKCRQYXRoKSAtb3IKKyAgICAtbm90
IFtTeXN0ZW0uSU8uUGF0aF06OklzUGF0aFJvb3RlZCgkUGF0aCkgLW9yCisgICAgJFBhdGguSW5k
ZXhPZihbY2hhcl0wKSAtZ2UgMCAtb3IKKyAgICAkUGF0aC5JbmRleE9mKCJgciIsIFtTdHJpbmdD
b21wYXJpc29uXTo6T3JkaW5hbCkgLWdlIDAgLW9yCisgICAgJFBhdGguSW5kZXhPZigiYG4iLCBb
U3RyaW5nQ29tcGFyaXNvbl06Ok9yZGluYWwpIC1nZSAwIC1vcgorICAgIFtTeXN0ZW0uTWFuYWdl
bWVudC5BdXRvbWF0aW9uLldpbGRjYXJkUGF0dGVybl06OkNvbnRhaW5zV2lsZGNhcmRDaGFyYWN0
ZXJzKCRQYXRoKQorICApIHsgdGhyb3cgJ3JlY292ZXJ5IHJlcG9zaXRvcnkgcGF0aCBpcyBhbWJp
Z3VvdXMgb3IgdW5zYWZlJyB9CisgICRSZXNvbHZlZCA9IFtTeXN0ZW0uSU8uUGF0aF06OkdldEZ1
bGxQYXRoKCRQYXRoKS5UcmltRW5kKCdcJywgJy8nKQorICAkR2l0RGlyZWN0b3J5ID0gSm9pbi1Q
YXRoICRSZXNvbHZlZCAnLmdpdCcKKyAgaWYgKAorICAgIC1ub3QgW1N5c3RlbS5JTy5EaXJlY3Rv
cnldOjpFeGlzdHMoJFJlc29sdmVkKSAtb3IKKyAgICAtbm90IFtTeXN0ZW0uSU8uRGlyZWN0b3J5
XTo6RXhpc3RzKCRHaXREaXJlY3RvcnkpCisgICkgeyB0aHJvdyAncmVjb3ZlcnkgcmVwb3NpdG9y
eSBpcyBub3QgYW4gZXhhY3Qgbm9uLWJhcmUgR2l0IHdvcmt0cmVlJyB9CisgIHJldHVybiBbcHNj
dXN0b21vYmplY3RdQHsKKyAgICBQYXRoID0gJFJlc29sdmVkCisgICAgR2l0RGlyZWN0b3J5ID0g
W1N5c3RlbS5JTy5QYXRoXTo6R2V0RnVsbFBhdGgoJEdpdERpcmVjdG9yeSkKKyAgICBTYWZlRGly
ZWN0b3J5ID0gJFJlc29sdmVkLlJlcGxhY2UoJ1wnLCAnLycpCisgIH0KK30KKworZnVuY3Rpb24g
VGVzdC1SZWNvdmVyeVBhdGhFcXVhbCB7CisgIHBhcmFtKFtzdHJpbmddICRGaXJzdCwgW3N0cmlu
Z10gJFNlY29uZCkKKworICAkQ29tcGFyaXNvbiA9IGlmICgkZW52Ok9TIC1jZXEgJ1dpbmRvd3Nf
TlQnKSB7CisgICAgW1N0cmluZ0NvbXBhcmlzb25dOjpPcmRpbmFsSWdub3JlQ2FzZQorICB9IGVs
c2UgeyBbU3RyaW5nQ29tcGFyaXNvbl06Ok9yZGluYWwgfQorICByZXR1cm4gW3N0cmluZ106OkVx
dWFscygkRmlyc3QsICRTZWNvbmQsICRDb21wYXJpc29uKQorfQorCitmdW5jdGlvbiBHZXQtUmVj
b3ZlcnlBbWJpZW50R2l0Q29uZmlnQ291bnQgeworICAkRW52aXJvbm1lbnQgPSBbRW52aXJvbm1l
bnRdOjpHZXRFbnZpcm9ubWVudFZhcmlhYmxlcygnUHJvY2VzcycpCisgICRSYXdDb3VudCA9IFtz
dHJpbmddJEVudmlyb25tZW50WydHSVRfQ09ORklHX0NPVU5UJ10KKyAgaWYgKFtzdHJpbmddOjpJ
c051bGxPckVtcHR5KCRSYXdDb3VudCkpIHsgJENvdW50ID0gMCB9CisgIGVsc2VpZiAoJFJhd0Nv
dW50IC1jbWF0Y2ggJ14oPzowfFsxLTldWzAtOV0qKSQnKSB7ICRDb3VudCA9IFtpbnRdJFJhd0Nv
dW50IH0KKyAgZWxzZSB7IHRocm93ICdyZWNvdmVyeSBhbWJpZW50IEdJVF9DT05GSUdfQ09VTlQg
aXMgbWFsZm9ybWVkJyB9CisgIGlmICgkbnVsbCAtbmUgJEVudmlyb25tZW50WydHSVRfVEVTVF9B
U1NVTUVfRElGRkVSRU5UX09XTkVSJ10pIHsKKyAgICB0aHJvdyAncmVjb3Zlcnkgb3duZXJzaGlw
IGFkdmVyc2FyeSBpcyBhbHJlYWR5IGFjdGl2ZScKKyAgfQorICBmb3JlYWNoICgkTmFtZSBpbiBA
KCRFbnZpcm9ubWVudC5LZXlzKSkgeworICAgIGlmIChbc3RyaW5nXSROYW1lIC1jbWF0Y2ggJ15H
SVRfQ09ORklHXyg/OktFWXxWQUxVRSlfKFswLTldKykkJykgeworICAgICAgJEluZGV4ID0gW2lu
dF0kTWF0Y2hlc1sxXQorICAgICAgaWYgKCRJbmRleCAtZ2UgJENvdW50KSB7IHRocm93ICdyZWNv
dmVyeSBhbWJpZW50IEdpdCBjb25maWcgY29udGFpbnMgYW4gb3JwaGFuJyB9CisgICAgfQorICB9
CisgIGZvciAoJEluZGV4ID0gMDsgJEluZGV4IC1sdCAkQ291bnQ7ICRJbmRleCArPSAxKSB7Cisg
ICAgaWYgKAorICAgICAgJG51bGwgLWVxICRFbnZpcm9ubWVudFsiR0lUX0NPTkZJR19LRVlfJElu
ZGV4Il0gLW9yCisgICAgICAkbnVsbCAtZXEgJEVudmlyb25tZW50WyJHSVRfQ09ORklHX1ZBTFVF
XyRJbmRleCJdCisgICAgKSB7IHRocm93ICdyZWNvdmVyeSBhbWJpZW50IEdpdCBjb25maWcgaXMg
aW5jb21wbGV0ZScgfQorICB9CisgIHJldHVybiAkQ291bnQKK30KKworZnVuY3Rpb24gSW52b2tl
LVJlY292ZXJ5TXV0YXRpb25Qcm9jZXNzIHsKKyAgcGFyYW0oCisgICAgW3N0cmluZ10gJFJlcG9z
aXRvcnlQYXRoLAorICAgIFtzdHJpbmddICRFeHBlY3RlZENvbW1pdCwKKyAgICBbc3RyaW5nXSAk
VHJ1c3RQYXRoLAorICAgIFtzdHJpbmdbXV0gJFBvd2VyU2hlbGxBcmd1bWVudHMsCisgICAgW3N3
aXRjaF0gJE9taXRUcnVzdAorICApCisKKyAgJEFtYmllbnRDb3VudCA9IEdldC1SZWNvdmVyeUFt
YmllbnRHaXRDb25maWdDb3VudAorICBpZiAoJEV4cGVjdGVkQ29tbWl0IC1jbm90bWF0Y2ggJ15b
MC05YS1mXXs0MH0kJykgeworICAgIHRocm93ICdyZWNvdmVyeSBleHBlY3RlZCBjb21taXQgaXMg
aW52YWxpZCcKKyAgfQorICAkUmVwb3NpdG9yeSA9IFJlc29sdmUtUmVjb3ZlcnlSZXBvc2l0b3J5
ICRSZXBvc2l0b3J5UGF0aAorICAkVHJ1c3QgPSBSZXNvbHZlLVJlY292ZXJ5UmVwb3NpdG9yeSAk
VHJ1c3RQYXRoCisgIGlmICgtbm90IChUZXN0LVJlY292ZXJ5UGF0aEVxdWFsICRSZXBvc2l0b3J5
LlBhdGggJFRydXN0LlBhdGgpKSB7CisgICAgdGhyb3cgJ3JlY292ZXJ5IHRydXN0IGRvZXMgbm90
IGV4YWN0bHkgbWF0Y2ggdGhlIHNlbGVjdGVkIHJlcG9zaXRvcnknCisgIH0KKyAgJEdpdCA9IChH
ZXQtQ29tbWFuZCBnaXQuZXhlIC1FcnJvckFjdGlvbiBTdG9wKS5Tb3VyY2UKKyAgJEFkdmVyc2Fy
eSA9IEB7IEdJVF9URVNUX0FTU1VNRV9ESUZGRVJFTlRfT1dORVIgPSAnMScgfQorICAkSWRlbnRp
dHkgPSBJbnZva2UtUmVjb3ZlcnlOYXRpdmVQcm9jZXNzIC1GaWxlTmFtZSAkR2l0IC1Xb3JraW5n
RGlyZWN0b3J5ICRSZXBvc2l0b3J5LlBhdGggYAorICAgIC1DaGlsZEVudmlyb25tZW50ICRBZHZl
cnNhcnkgLUFyZ3VtZW50cyBAKAorICAgICAgJy0tbm8tcmVwbGFjZS1vYmplY3RzJywgIi0tZ2l0
LWRpcj0kKCRSZXBvc2l0b3J5LkdpdERpcmVjdG9yeSkiLAorICAgICAgJ3Jldi1wYXJzZScsICct
LXZlcmlmeScsICIkRXhwZWN0ZWRDb21taXRgXntjb21taXR9IgorICAgICkKKyAgJFV0ZjggPSBO
ZXctT2JqZWN0IFN5c3RlbS5UZXh0LlVURjhFbmNvZGluZygkZmFsc2UsICR0cnVlKQorICBpZiAo
CisgICAgJElkZW50aXR5LkV4aXRDb2RlIC1uZSAwIC1vcgorICAgICRJZGVudGl0eS5TdGRlcnJC
eXRlcy5MZW5ndGggLW5lIDAgLW9yCisgICAgJFV0ZjguR2V0U3RyaW5nKCRJZGVudGl0eS5TdGRv
dXRCeXRlcykuVHJpbUVuZChbY2hhcltdXSJgcmBuIikgLWNuZSAkRXhwZWN0ZWRDb21taXQKKyAg
KSB7IHRocm93ICdyZWNvdmVyeSByZXBvc2l0b3J5IGlkZW50aXR5IGF1dGhlbnRpY2F0aW9uIGZh
aWxlZCcgfQorCisgICRDaGlsZEVudmlyb25tZW50ID0gQHsgR0lUX1RFU1RfQVNTVU1FX0RJRkZF
UkVOVF9PV05FUiA9ICcxJyB9CisgIGlmICgtbm90ICRPbWl0VHJ1c3QpIHsKKyAgICAkQ2hpbGRF
bnZpcm9ubWVudC5HSVRfQ09ORklHX0NPVU5UID0gW3N0cmluZ10oJEFtYmllbnRDb3VudCArIDEp
CisgICAgJENoaWxkRW52aXJvbm1lbnRbIkdJVF9DT05GSUdfS0VZXyRBbWJpZW50Q291bnQiXSA9
ICdzYWZlLmRpcmVjdG9yeScKKyAgICAkQ2hpbGRFbnZpcm9ubWVudFsiR0lUX0NPTkZJR19WQUxV
RV8kQW1iaWVudENvdW50Il0gPSAkUmVwb3NpdG9yeS5TYWZlRGlyZWN0b3J5CisgIH0KKyAgJFBv
d2VyU2hlbGwgPSAoR2V0LVByb2Nlc3MgLUlkICRQSUQpLlBhdGgKKyAgJFJlc3VsdCA9IEludm9r
ZS1SZWNvdmVyeU5hdGl2ZVByb2Nlc3MgLUZpbGVOYW1lICRQb3dlclNoZWxsIGAKKyAgICAtV29y
a2luZ0RpcmVjdG9yeSAkUmVwb3NpdG9yeS5QYXRoIC1DaGlsZEVudmlyb25tZW50ICRDaGlsZEVu
dmlyb25tZW50IGAKKyAgICAtQXJndW1lbnRzICRQb3dlclNoZWxsQXJndW1lbnRzCisgIHJldHVy
biBbcHNjdXN0b21vYmplY3RdQHsKKyAgICBSZXBvc2l0b3J5UGF0aCA9ICRSZXBvc2l0b3J5LlBh
dGgKKyAgICBJZGVudGl0eSA9ICRFeHBlY3RlZENvbW1pdAorICAgIEV4aXRDb2RlID0gJFJlc3Vs
dC5FeGl0Q29kZQorICAgIFN0ZG91dEJ5dGVzID0gJFJlc3VsdC5TdGRvdXRCeXRlcworICAgIFN0
ZGVyckJ5dGVzID0gJFJlc3VsdC5TdGRlcnJCeXRlcworICB9Cit9CisKK2Z1bmN0aW9uIEFzc2Vy
dC1SZWNvdmVyeU11dGF0aW9uU3VjY2VzcyB7CisgIHBhcmFtKFtvYmplY3RdICRSZXN1bHQsIFtz
dHJpbmddICRFeHBlY3RlZENvbW1pdCkKKworICAkVXRmOCA9IE5ldy1PYmplY3QgVGV4dC5VVEY4
RW5jb2RpbmcoJGZhbHNlLCAkdHJ1ZSkKKyAgaWYgKAorICAgICRSZXN1bHQuSWRlbnRpdHkgLWNu
ZSAkRXhwZWN0ZWRDb21taXQgLW9yCisgICAgJFJlc3VsdC5FeGl0Q29kZSAtbmUgMCAtb3IKKyAg
ICAkUmVzdWx0LlN0ZGVyckJ5dGVzLkxlbmd0aCAtbmUgMCAtb3IKKyAgICAkVXRmOC5HZXRTdHJp
bmcoJFJlc3VsdC5TdGRvdXRCeXRlcykgLWNuZSAiJEV4cGVjdGVkQ29tbWl0YG4iCisgICkgeyB0
aHJvdyAncmVjb3ZlcnkgbXV0YXRpb24gcmVzdWx0IGlzIG1pc3NpbmcsIG1hbGZvcm1lZCwgb3Ig
dW5zdWNjZXNzZnVsJyB9Cit9CisKK2Z1bmN0aW9uIEdldC1SZWNvdmVyeVN0YXRlU2hhMjU2IHsK
KyAgcGFyYW0oW3N0cmluZ10gJFJlcG9zaXRvcnlQYXRoKQorCisgICRSb3dzID0gTmV3LU9iamVj
dCBTeXN0ZW0uQ29sbGVjdGlvbnMuR2VuZXJpYy5MaXN0W3N0cmluZ10KKyAgJEVudmlyb25tZW50
ID0gW0Vudmlyb25tZW50XTo6R2V0RW52aXJvbm1lbnRWYXJpYWJsZXMoJ1Byb2Nlc3MnKQorICBm
b3JlYWNoICgkTmFtZSBpbiBAKCRFbnZpcm9ubWVudC5LZXlzIHwgU29ydC1PYmplY3QpKSB7Cisg
ICAgJEJ5dGVzID0gW1RleHQuRW5jb2RpbmddOjpVVEY4LkdldEJ5dGVzKFtzdHJpbmddJEVudmly
b25tZW50WyROYW1lXSkKKyAgICAkUm93cy5BZGQoImVudjokTmFtZT0kKFtDb252ZXJ0XTo6VG9C
YXNlNjRTdHJpbmcoJEJ5dGVzKSkiKQorICB9CisgICRHaXQgPSAoR2V0LUNvbW1hbmQgZ2l0LmV4
ZSAtRXJyb3JBY3Rpb24gU3RvcCkuU291cmNlCisgIGZvcmVhY2ggKCRTY29wZSBpbiBAKCdsb2Nh
bCcsICdnbG9iYWwnLCAnc3lzdGVtJykpIHsKKyAgICAkQXJndW1lbnRzID0gaWYgKCRTY29wZSAt
Y2VxICdsb2NhbCcpIHsKKyAgICAgIEAoIi0tZ2l0LWRpcj0kUmVwb3NpdG9yeVBhdGhcLmdpdCIs
ICdjb25maWcnLCAnLS1sb2NhbCcsICctLW51bGwnLCAnLS1saXN0JykKKyAgICB9IGVsc2UgeyBA
KCdjb25maWcnLCAiLS0kU2NvcGUiLCAnLS1udWxsJywgJy0tbGlzdCcpIH0KKyAgICAkUmVzdWx0
ID0gSW52b2tlLVJlY292ZXJ5TmF0aXZlUHJvY2VzcyAkR2l0ICRBcmd1bWVudHMgJFJlcG9zaXRv
cnlQYXRoCisgICAgJFJvd3MuQWRkKCIke1Njb3BlfTokKCRSZXN1bHQuRXhpdENvZGUpOiQoW0Nv
bnZlcnRdOjpUb0Jhc2U2NFN0cmluZygkUmVzdWx0LlN0ZG91dEJ5dGVzKSk6JChbQ29udmVydF06
OlRvQmFzZTY0U3RyaW5nKCRSZXN1bHQuU3RkZXJyQnl0ZXMpKSIpCisgIH0KKyAgJFNoYSA9IFtT
ZWN1cml0eS5DcnlwdG9ncmFwaHkuU0hBMjU2XTo6Q3JlYXRlKCkKKyAgdHJ5IHsKKyAgICAkQnl0
ZXMgPSBbVGV4dC5FbmNvZGluZ106OlVURjguR2V0Qnl0ZXMoKEAoJFJvd3MpIC1qb2luICJgbiIp
ICsgImBuIikKKyAgICByZXR1cm4gKFtCaXRDb252ZXJ0ZXJdOjpUb1N0cmluZygkU2hhLkNvbXB1
dGVIYXNoKCRCeXRlcykpIC1yZXBsYWNlICctJywgJycpLlRvTG93ZXJJbnZhcmlhbnQoKQorICB9
IGZpbmFsbHkgeyAkU2hhLkRpc3Bvc2UoKSB9Cit9CisKK2Z1bmN0aW9uIEFzc2VydC1SZWNvdmVy
eVRyYW5zcG9ydFByb2plY3Rpb24geworICBwYXJhbShbc3RyaW5nXSAkUm9vdCkKKworICAkUmVw
b3NpdG9yeSA9IE5ldy1UZXN0UmVwb3NpdG9yeSAkUm9vdAorICAkT3RoZXIgPSBOZXctVGVzdFJl
cG9zaXRvcnkgJFJvb3QKKyAgJFByb2JlUGF0aCA9IEpvaW4tUGF0aCAkUm9vdCAncmVjb3Zlcnkt
Y2hpbGQucHMxJworICBXcml0ZS1UZXN0VGV4dCAkUHJvYmVQYXRoIEAnCitwYXJhbShbc3RyaW5n
XSAkUmVwb3NpdG9yeSwgW3N0cmluZ10gJENvbW1pdCwgW3N0cmluZ10gJE1vZGUpCitpZiAoJE1v
ZGUgLWNlcSAnbm9uemVybycpIHsgW0NvbnNvbGVdOjpFcnJvci5Xcml0ZSgncHJvYmUtZXJyb3In
KTsgZXhpdCAyMyB9CitpZiAoJE1vZGUgLWNlcSAnbWlzc2luZycpIHsgZXhpdCAwIH0KK2lmICgk
TW9kZSAtY2VxICdtYWxmb3JtZWQnKSB7IFtDb25zb2xlXTo6T3V0LldyaXRlKCJtYWxmb3JtZWRg
biIpOyBleGl0IDAgfQorJEdpdCA9IChHZXQtQ29tbWFuZCBnaXQuZXhlIC1FcnJvckFjdGlvbiBT
dG9wKS5Tb3VyY2UKKyRPdXRwdXQgPSBAKCYgJEdpdCAtLW5vLXJlcGxhY2Utb2JqZWN0cyAtQyAk
UmVwb3NpdG9yeSBzaG93ICctLWZvcm1hdD0lSCcgLS1uby1wYXRjaCAkQ29tbWl0KQorJENvZGUg
PSAkTEFTVEVYSVRDT0RFCitpZiAoJENvZGUgLW5lIDApIHsgZXhpdCAkQ29kZSB9CitbQ29uc29s
ZV06Ok91dC5Xcml0ZSgoQCgkT3V0cHV0KSAtam9pbiAiYG4iKSArICJgbiIpCitpZiAoJE1vZGUg
LWNlcSAnbGVnYWN5JykgeyBbQ29uc29sZV06Ok91dC5Xcml0ZSgibGVnYWN5LWFzc2VydGlvbi1y
ZWFjaGVkYG4iKSB9CitpZiAoJE1vZGUgLWNlcSAnYmluYXJ5JykgeyBbQ29uc29sZV06Ok91dC5X
cml0ZSgiYmluYXJ5LWFzc2VydGlvbi1yZWFjaGVkYG4iKSB9CisnQAorICAkUXVvdGVQYXRoID0g
Sm9pbi1QYXRoICRSb290ICdyZWNvdmVyeS1xdW90ZS5wczEnCisgIFdyaXRlLVRlc3RUZXh0ICRR
dW90ZVBhdGggQCcKKyRSb3dzID0gQCgkYXJncyB8IEZvckVhY2gtT2JqZWN0IHsKKyAgW0NvbnZl
cnRdOjpUb0Jhc2U2NFN0cmluZyhbVGV4dC5FbmNvZGluZ106OlVURjguR2V0Qnl0ZXMoW3N0cmlu
Z10kXykpCit9KQorW0NvbnNvbGVdOjpPdXQuV3JpdGUoJFJvd3MgLWpvaW4gJzsnKQorJ0AKKyAg
JFF1b3RlVmFsdWVzID0gQCgnJywgJ3R3byB3b3JkcycsICdlbWJlZGRlZCJxdW90ZScsICd0cmFp
bGluZ1wnKQorICAkUXVvdGVSZXN1bHQgPSBJbnZva2UtUmVjb3ZlcnlOYXRpdmVQcm9jZXNzIChH
ZXQtUHJvY2VzcyAtSWQgJFBJRCkuUGF0aCBgCisgICAgKEAoJy1Ob1Byb2ZpbGUnLCAnLUZpbGUn
LCAkUXVvdGVQYXRoKSArICRRdW90ZVZhbHVlcykgJFJvb3QKKyAgJEV4cGVjdGVkUXVvdGVzID0g
QCgkUXVvdGVWYWx1ZXMgfCBGb3JFYWNoLU9iamVjdCB7CisgICAgW0NvbnZlcnRdOjpUb0Jhc2U2
NFN0cmluZyhbVGV4dC5FbmNvZGluZ106OlVURjguR2V0Qnl0ZXMoJF8pKQorICB9KSAtam9pbiAn
OycKKyAgQXNzZXJ0LUJvdW5kYXJ5VGVzdCAoCisgICAgJFF1b3RlUmVzdWx0LkV4aXRDb2RlIC1l
cSAwIC1hbmQKKyAgICAkUXVvdGVSZXN1bHQuU3RkZXJyQnl0ZXMuTGVuZ3RoIC1lcSAwIC1hbmQK
KyAgICBbVGV4dC5FbmNvZGluZ106OlVURjguR2V0U3RyaW5nKCRRdW90ZVJlc3VsdC5TdGRvdXRC
eXRlcykgLWNlcSAkRXhwZWN0ZWRRdW90ZXMKKyAgKSAnUG93ZXJTaGVsbCA1LjEgcmVjb3Zlcnkg
YXJndW1lbnQgY29uc3RydWN0aW9uIGNoYW5nZWQgYW4gYXJndW1lbnQnCisgICRQb3dlclNoZWxs
QXJndW1lbnRzID0gQCgKKyAgICAnLU5vUHJvZmlsZScsICctRmlsZScsICRQcm9iZVBhdGgsCisg
ICAgJy1SZXBvc2l0b3J5JywgJFJlcG9zaXRvcnkuUGF0aCwKKyAgICAnLUNvbW1pdCcsICRSZXBv
c2l0b3J5LkFuY2hvciwKKyAgICAnLU1vZGUnLCAnaG9uZXN0JworICApCisgICRTdGF0ZSA9IEdl
dC1SZWNvdmVyeVN0YXRlU2hhMjU2ICRSZXBvc2l0b3J5LlBhdGgKKyAgJEJhc2VsaW5lID0gSW52
b2tlLVJlY292ZXJ5TmF0aXZlUHJvY2VzcyAoR2V0LVByb2Nlc3MgLUlkICRQSUQpLlBhdGggYAor
ICAgICRQb3dlclNoZWxsQXJndW1lbnRzICRSZXBvc2l0b3J5LlBhdGgKKyAgQXNzZXJ0LUJvdW5k
YXJ5VGVzdCAoJEJhc2VsaW5lLkV4aXRDb2RlIC1lcSAwKSAnc2FtZS1vd25lciBjb250cm9sIGRp
ZCBub3Qgc3VjY2VlZCcKKworICAkSG9uZXN0ID0gSW52b2tlLVJlY292ZXJ5TXV0YXRpb25Qcm9j
ZXNzICRSZXBvc2l0b3J5LlBhdGggJFJlcG9zaXRvcnkuQW5jaG9yIGAKKyAgICAkUmVwb3NpdG9y
eS5QYXRoICRQb3dlclNoZWxsQXJndW1lbnRzCisgICRVdGY4ID0gTmV3LU9iamVjdCBUZXh0LlVU
RjhFbmNvZGluZygkZmFsc2UsICR0cnVlKQorICAkRXhwZWN0ZWQgPSAiJCgkUmVwb3NpdG9yeS5B
bmNob3IpYG4iCisgIEFzc2VydC1SZWNvdmVyeU11dGF0aW9uU3VjY2VzcyAkSG9uZXN0ICRSZXBv
c2l0b3J5LkFuY2hvcgorCisgICROb1RydXN0ID0gSW52b2tlLVJlY292ZXJ5TXV0YXRpb25Qcm9j
ZXNzICRSZXBvc2l0b3J5LlBhdGggJFJlcG9zaXRvcnkuQW5jaG9yIGAKKyAgICAkUmVwb3NpdG9y
eS5QYXRoICRQb3dlclNoZWxsQXJndW1lbnRzIC1PbWl0VHJ1c3QKKyAgQXNzZXJ0LUJvdW5kYXJ5
VGVzdCAoCisgICAgJE5vVHJ1c3QuSWRlbnRpdHkgLWNlcSAkUmVwb3NpdG9yeS5BbmNob3IgLWFu
ZAorICAgICROb1RydXN0LkV4aXRDb2RlIC1uZSAwIC1hbmQKKyAgICAkTm9UcnVzdC5TdGRvdXRC
eXRlcy5MZW5ndGggLWVxIDAgLWFuZAorICAgICRVdGY4LkdldFN0cmluZygkTm9UcnVzdC5TdGRl
cnJCeXRlcykuQ29udGFpbnMoJ2R1YmlvdXMgb3duZXJzaGlwJykKKyAgKSAnb3duZXJzaGlwIGFk
dmVyc2FyeSBkaWQgbm90IGZhaWwgc3BlY2lmaWNhbGx5IGF0IGR1YmlvdXMgb3duZXJzaGlwJwor
CisgIGZvcmVhY2ggKCRUcnVzdCBpbiBAKCRPdGhlci5QYXRoLCAkUm9vdCwgIiQoJFJlcG9zaXRv
cnkuUGF0aCkqIikpIHsKKyAgICAkUmVqZWN0ZWQgPSAkZmFsc2UKKyAgICB0cnkgeworICAgICAg
W3ZvaWRdKEludm9rZS1SZWNvdmVyeU11dGF0aW9uUHJvY2VzcyAkUmVwb3NpdG9yeS5QYXRoICRS
ZXBvc2l0b3J5LkFuY2hvciBgCisgICAgICAgICRUcnVzdCAkUG93ZXJTaGVsbEFyZ3VtZW50cykK
KyAgICB9IGNhdGNoIHsgJFJlamVjdGVkID0gJHRydWUgfQorICAgIEFzc2VydC1Cb3VuZGFyeVRl
c3QgJFJlamVjdGVkICJub24tZXhhY3QgcmVjb3ZlcnkgdHJ1c3Qgd2FzIGFjY2VwdGVkOiAkVHJ1
c3QiCisgIH0KKworICBmb3JlYWNoICgkTW9kZSBpbiBAKCdub256ZXJvJywgJ21pc3NpbmcnLCAn
bWFsZm9ybWVkJywgJ2xlZ2FjeScsICdiaW5hcnknKSkgeworICAgICRBcmd1bWVudHMgPSBAKCRQ
b3dlclNoZWxsQXJndW1lbnRzKQorICAgICRBcmd1bWVudHNbJEFyZ3VtZW50cy5Db3VudCAtIDFd
ID0gJE1vZGUKKyAgICAkUmVzdWx0ID0gSW52b2tlLVJlY292ZXJ5TXV0YXRpb25Qcm9jZXNzICRS
ZXBvc2l0b3J5LlBhdGggJFJlcG9zaXRvcnkuQW5jaG9yIGAKKyAgICAgICRSZXBvc2l0b3J5LlBh
dGggJEFyZ3VtZW50cworICAgICRUZXh0ID0gJFV0ZjguR2V0U3RyaW5nKCRSZXN1bHQuU3Rkb3V0
Qnl0ZXMpCisgICAgaWYgKCRNb2RlIC1jaW4gQCgnbm9uemVybycsICdtaXNzaW5nJywgJ21hbGZv
cm1lZCcpKSB7CisgICAgICAkUmVqZWN0ZWQgPSAkZmFsc2UKKyAgICAgIHRyeSB7IEFzc2VydC1S
ZWNvdmVyeU11dGF0aW9uU3VjY2VzcyAkUmVzdWx0ICRSZXBvc2l0b3J5LkFuY2hvciB9IGNhdGNo
IHsgJFJlamVjdGVkID0gJHRydWUgfQorICAgICAgQXNzZXJ0LUJvdW5kYXJ5VGVzdCAkUmVqZWN0
ZWQgIiRNb2RlIHJlY292ZXJ5IHJlc3VsdCBzYXRpc2ZpZWQgdGhlIHN1Y2Nlc3MgY29udHJhY3Qi
CisgICAgICBpZiAoJE1vZGUgLWNlcSAnbm9uemVybycpIHsKKyAgICAgICAgQXNzZXJ0LUJvdW5k
YXJ5VGVzdCAoJFJlc3VsdC5FeGl0Q29kZSAtZXEgMjMgLWFuZCAkUmVzdWx0LlN0ZGVyckJ5dGVz
Lkxlbmd0aCAtZ3QgMCkgYAorICAgICAgICAgICdub256ZXJvIHJlY292ZXJ5IHJlc3VsdCB3YXMg
bm90IHByZXNlcnZlZCcKKyAgICAgIH0gZWxzZWlmICgkTW9kZSAtY2VxICdtaXNzaW5nJykgewor
ICAgICAgICBBc3NlcnQtQm91bmRhcnlUZXN0ICgkUmVzdWx0LkV4aXRDb2RlIC1lcSAwIC1hbmQg
JFJlc3VsdC5TdGRvdXRCeXRlcy5MZW5ndGggLWVxIDApIGAKKyAgICAgICAgICAnbWlzc2luZyBy
ZWNvdmVyeSBvdXRwdXQgd2FzIG5vdCBvYnNlcnZhYmxlJworICAgICAgfQorICAgIH0gZWxzZSB7
CisgICAgICBBc3NlcnQtQm91bmRhcnlUZXN0ICgKKyAgICAgICAgJFJlc3VsdC5FeGl0Q29kZSAt
ZXEgMCAtYW5kCisgICAgICAgICRSZXN1bHQuU3RkZXJyQnl0ZXMuTGVuZ3RoIC1lcSAwIC1hbmQK
KyAgICAgICAgJFRleHQuQ29udGFpbnMoIiRNb2RlLWFzc2VydGlvbi1yZWFjaGVkYG4iKQorICAg
ICAgKSAiJE1vZGUgbXV0YXRpb24gYXNzZXJ0aW9uIHdhcyBub3QgcmVhY2hlZCIKKyAgICB9Cisg
IH0KKyAgQXNzZXJ0LUJvdW5kYXJ5VGVzdCAoCisgICAgKEdldC1SZWNvdmVyeVN0YXRlU2hhMjU2
ICRSZXBvc2l0b3J5LlBhdGgpIC1jZXEgJFN0YXRlCisgICkgJ3JlY292ZXJ5IHByb2plY3Rpb24g
Y2hhbmdlZCBwYXJlbnQgZW52aXJvbm1lbnQgb3IgR2l0IGNvbmZpZ3VyYXRpb24nCisgIFdyaXRl
LUhvc3QgIlVuaXQgQyByZWNvdmVyeSByZWRlc2lnbiBwYXNzZWQ6IGlkZW50aXR5PSQoJFJlcG9z
aXRvcnkuQW5jaG9yKTtxdW90ZXM9ZXhhY3Q7bGVnYWN5PXJlYWNoYWJsZTtiaW5hcnk9cmVhY2hh
YmxlIgorfQorCiBmdW5jdGlvbiBJbnZva2UtVGVzdEdpdCB7CiAgIHBhcmFtKAogICAgIFtzdHJp
bmddICRSZXBvUGF0aCwKQEAgLTcyMCw2ICsxMDQ4LDcgQEAgQXNzZXJ0LUJvdW5kYXJ5VGVzdCAk
VGVzdFJvb3QuU3RhcnRzV2l0aCgkVGVtcEJhc2UsIFtTeXN0ZW0uU3RyaW5nQ29tcGFyaXNvbl06
Ok8KIFt2b2lkXVtTeXN0ZW0uSU8uRGlyZWN0b3J5XTo6Q3JlYXRlRGlyZWN0b3J5KCRUZXN0Um9v
dCkKIAogdHJ5IHsKKyAgQXNzZXJ0LVJlY292ZXJ5VHJhbnNwb3J0UHJvamVjdGlvbiAkVGVzdFJv
b3QKICAgQXNzZXJ0LVByb2R1Y3Rpb25TZWFtSXNDbG9zZWQKICAgQXNzZXJ0LUhpc3RvcmljYWxB
bWVuZG1lbnRJc0Z1bGwKIAo=
```
<!-- wo21-unit-c-projection-patch-base64:end -->

#### Revised recovery ceiling and one implementation path

The smallest passing integrated projection adds 329 lines and deletes none
relative to clean candidate `b9093901...`. The redesign permits exactly four
lines of insertion and four lines of deletion headroom for platform integration
without architectural change. The recovery delta ceiling is therefore
`+333/-4`, under both raw and whitespace-insensitive accounting. This derives
from measured `+329/-0`, not from a round target.

With that ceiling, final test-script limits relative to the published Unit C
parent are `+597/-80`; final combined limits with the unchanged production
candidate are `+598/-106`. These remain below the already authoritative Unit C
ceilings of test `+600/-520` and combined `+700/-700`; no unit-level budget is
raised. Production remains exactly `+1/-26`, blob
`62b3d964e116a1c19190945550151b456ad5aa99`. Unused deletion capacity is not
refactoring authority.

There is one future implementation path:

1. Obtain a fresh independent review of this exact amendment. Only an
   unqualified `ACCEPT` may return it to the BDFL.
2. Under separate authority, create one documentation commit with exact subject
   `docs(workorder): redesign unit c evidence transport`.
3. Separately publish that commit and require terminal-green full Ubuntu and
   Windows CI.
4. Separately author, commit, publish, and prove its immutable publication-
   status record through terminal-green fast CI.
5. Obtain a fresh explicit BDFL Unit C redesign-implementation signal.
6. Apply, never pop, clean stash commit
   `b9093901b8c92c626c3c23ee1a52366d2e54f698`; never apply rejected stash
   `44262ceec1e895a3120133e8676387f2786ae3d0`.
7. Reauthenticate the clean two-path candidate, preserve the production blob,
   and implement only the selected 329-line architecture within `+333/-4`.
8. Before restarting the matrix, prove adversary availability, independent
   identity on honest and failed calls, exact scoped-trust success, trust-
   removal dubious-ownership failure, non-exact trust rejection, PS5.1 quoting,
   separate streams/exit, result-shape rejection, and state restoration.
9. Run the precise legacy-root and binary-excluding mutations through this one
   transport and require their original classifier assertions to be reached
   and fail under their one-property production weakenings.
10. Restart the complete 151-case matrix and every remaining Unit C mutation
    from the beginning, then run all standing document gates and the single
    still-unconsumed Fast allowance in their frozen order.
11. Leave exactly the two candidate paths unstaged for one fresh independent
    complete-tree review. Commit, publication, status, closeout, and successor
    work remain separately gated.

A third path, production-blob drift, Work Order edit during implementation,
rejected-stash application, malformed/orphaned ambient config, unavailable or
non-load-bearing ownership adversary, identity mismatch, broader trust reaching
Git, different trust satisfying the child, incomplete capture, environment or
config drift, mutation assertion not reached, matrix failure, budget breach,
platform disagreement, or Fast failure is a mandatory stop. No correction,
alternate adversary, budget increase, rerun, compaction, assertion weakening,
or broad harness consolidation follows implicitly.

This amendment authorizes no candidate restoration or edit, matrix, mutation,
Fast, Exhaustive, Cargo/compiler evidence, commit, push, CI, closeout, WO22,
stash mutation, archive mutation, global harness consolidation,
semantic-coordinate research, or compiler work.

## Complete mutation matrix

Every required property has an honest control and one-property corruption. The
matrix freezes minimum ownership; unit-specific sections may add cases but may
not delete or combine these rows.

| Mutation or positive | Unit | Owning permanent/review evidence | Required result |
| --- | --- | --- | --- |
| Four published root header/gate/two-commit positives | A, C | historical Unit A evidence; exact Unit C retirement ledger | fast during transition, then retire all four credits |
| Canonical nested header/gate/two-commit positives | A, C | exact cases A01, A02, and A03 | fast; sole surviving canonical credits |
| Combined header-and-gate single transition | A, C | retained published `one full anchor plus exact header and gate update is fast` | fast; distinct combined property |
| Complete legacy-to-canonical migration | A, B | classifier suite and tree ledger | full, `no_status_transition` |
| Valid canonical successor issuance | A, C | exact case A06 | full, `no_status_transition` |
| Marker placed in `workorders/closed` | A, C | classifier suite | fail closed/full |
| Closed file copied into active | A, B, C | classifier/tree evidence | fail closed/full |
| Two active numbered files | A, C | classifier suite | fail closed/full |
| Visible second marker in the active Work Order | A, B, C | published `duplicate active marker is full` | fail closed/full |
| Exact marker line retained byte-identically in a Git-classified binary non-Work-Order blob across a measured status-only transition | A, C | revised three-state binary-safe A11 and its weakening mutation | honest fail closed/full; binary-excluding mutation fast |
| Mixed legacy/canonical tree | A | classifier suite | fail closed/full |
| Root Work Order reintroduced beside valid canonical authority | C | exact replacement case C05 | fail closed/full |
| Legacy-root parent after fallback removal | C | exact replacement case C12 | full, `no_status_transition` |
| Unnumbered canonical file | A, C | classifier suite | fail closed/full |
| Leading-zero positive number | A, C | exact case A14 | fail closed/full |
| Exact zero number | C | exact replacement case C02 | fail closed/full |
| Signed number | C | exact replacement case C03 | fail closed/full |
| Whitespace before extension | C | exact replacement case C04 | fail closed/full |
| Wrong-case active basename prefix | C | exact replacement case C01 | fail closed/full |
| Wrong-case active directory | A, C | exact case A15 | fail closed/full |
| Wrong-case closed directory | A, C | exact case A16 | fail closed/full |
| Wrong-case extension | A, C | exact case A17 | fail closed/full |
| Nested backup/suffix file | A, C | classifier suite | fail closed/full |
| Traversal-like/separator ambiguity | A, C | classifier suite | fail closed/full |
| Symlink or submodule substitution | A, B, C | classifier/tree evidence | fail closed/full |
| Active-file deletion | A, C | classifier suite | full |
| Active rename without successor | A, C | exact case A24 and active-path identity mutation | full |
| Active marker removed at unchanged active path | A, C | exact case A27 and marker-required resolution mutation | full |
| Status edit plus unrelated closed edit | A, C | classifier suite | full |
| Status edit plus Work Order move | A, C | classifier suite | full |
| Parent/child active-path disagreement | A, C | exact cases A05, A06, and A24 distinguish migration, successor issuance, and bare rename | full |
| Successor retains predecessor marker | A, C | exact case A28 | full, `no_status_transition` |
| Successor removes another predecessor byte | A, B, C | exact predecessor byte reconstruction | reject candidate/full |
| Successor created without closing/unmarking predecessor | A, C | successor-tree mutation | full, `no_status_transition` |
| Predecessor closed/unmarked without valid successor | A, C | successor-tree mutation | full, `no_status_transition` |
| Predecessor source/closed destination number mismatch | A, B, C | exact tree/path ledger | reject candidate/full |
| One missing/swapped/edited rename | B | exact tree ledger | reject candidate |
| Work Order 9 destination differs by one byte | B | blob/byte identity | reject candidate |
| Stale AGENTS root rule | B | live-policy audit | reject candidate |
| Stale/foreign diagnostic doctrine path | B | focused same-file test | reject candidate |
| Migration gains a sixteenth change record | B | exact envelope audit | mandatory stop |
| Production check deleted/relaxed | A, C | candidate mutation | corresponding case fails |
| Any of six retired credits retained, any C01-C05/C12 missing, or any alias substituted | C | exact 151-name invocation/uniqueness report | reject candidate |
| Adjacent `WORKORDERING.md` path | A, C | classifier suite | remains non-candidate |

No case may be satisfied by substituting a preselected failure value, by
changing only expected output, or by counting source text when executable
behavior is available.

## Review and evidence boundary for every unit

Each unit follows this exact lifecycle, with no implied next step:

1. BDFL issues an explicit unit implementation signal.
2. Implementer authenticates baseline/envelope and leaves a frozen candidate
   unstaged and uncommitted.
3. Fresh independent reviewer inspects the complete candidate, real control
   plane, positives, mutations, platforms, budgets, and final state read-only.
4. Only unqualified acceptance may return for a separately authorized local
   commit with the frozen subject.
5. Publication is a separate main-only normal non-force fast-forward gate.
6. Required Ubuntu and Windows full CI run through terminal completion.
7. Publication evidence is recorded only in a separately authorized mutable
   status/current-gate edit.
8. That status commit is separately published and both required jobs reproduce
   the exact fast status-chain binding.
9. Only a fresh BDFL signal may begin the next unit.

Any red job, classifier disagreement, identity drift, unexpected path, budget
breach, lost mutation, archive/stash change, or review finding stops the unit.
No retry, correction, amendment, commit, push, or next unit follows implicitly.

## Aggregate sustainability budget

Across implementation units, authorized non-rename ceilings are:

| Category | Insertions | Deletions |
| --- | ---: | ---: |
| Unit A classifier and tests | 1,160 | 260 |
| Unit B policy and metadata | 90 | 36 |
| Unit C classifier and tests | 700 | 700 |
| **Maximum aggregate** | **1,950** | **996** |

The thirteen Unit B renames remain exact `+0/-0` content changes. No unit may
borrow another unit's path or line budget. Review must report raw and
whitespace-insensitive totals, per-path statistics, and whether any change is
mechanical relocation versus new behavior/evidence.

## Explicit exclusions

WO21 authorizes none of the following unless a later unit gate explicitly says
otherwise:

- compiler, language, runtime, IR, backend, artifact, or schema behavior;
- any retry, redesign, restoration, or reuse of WO20 Unit B authority;
- evidence-harness consolidation implementation;
- stash apply/pop/drop/reorder/cleanup;
- archive deletion, rename, update, or publication;
- Git history rewriting, reset, rebase, graft, replace ref, or force push;
- branch deletion or repository-setting change;
- decision-record rewrite;
- semantic-coordinate or canonical cognitive-layout implementation;
- replacement compiler Work Order;
- `workorders/README.md` or a manually synchronized catalog;
- unrelated documentation cleanup, formatting, generated output, dependency
  changes, release/tag work, or later planning; and
- moving any Work Order during planning authorship or Unit A.

## Queued successor priorities

These are recorded without implementation or inherited authority:

1. Evidence-harness consolidation is the first compiler-process planning
   question after repository organization.
2. The semantic-coordinate/canonical cognitive-layout advisory remains queued
   for representative-program research.
3. A retry or redesign of WO20 Unit B requires a new Work Order and new evidence
   authority. WO21 inherits no implementation, stash, artifact, verifier, or
   review authority from WO20.

## Planning-package validation

Document authorship must run only:

- `git diff --check`;
- a fail-closed no-index whitespace check for untracked `WORKORDER_21.md`;
- two complete independent executions of the 123-case status classifier suite,
  each internally twice deterministic;
- raw byte comparison of both stdout streams and both zero-length stderr
  streams, with byte counts and SHA-256 identities recorded;
- text hygiene and public readiness for the new repository file count;
- alpha claims;
- release readiness for `0.0.1`;
- LF-only/final-LF checks;
- exact proof of the sole marker at `WORKORDER_21.md:4`; and
- exact proof that `WORKORDER_20.md` differs from the published blob only by
  deletion of its marker line.

No Cargo, Rust selector, Fast, full preflight, Exhaustive, CI, performance
probe, or migration simulation is authorized during planning authorship.

## Current authorization gate

The sole current authorization is this one-file immutable publication-status
record and its exact local commit with subject
`docs(workorder): record unit c redesign publication`. Publication of that
status commit and terminal-green fast Ubuntu and Windows CI remain separately
gated and require a fresh explicit BDFL signal.

Even after terminal-green status CI, restoration of clean candidate stash
`b9093901b8c92c626c3c23ee1a52366d2e54f698` and Unit C implementation require a
fresh explicit BDFL Unit C redesign-implementation signal. Rejected stash
`44262ceec1e895a3120133e8676387f2786ae3d0` remains preservation-only and may not
be applied. No stash apply/pop/drop/reorder, candidate edit, production or test
script edit, matrix or mutation execution, Cargo, Fast, Exhaustive, CI, push,
archive mutation, Unit C acceptance, closeout, successor issuance, WO22, broad
harness consolidation, compiler work, semantic-coordinate research, or later
activity follows implicitly.

<!-- workorder-current-authorization-gate:end -->
