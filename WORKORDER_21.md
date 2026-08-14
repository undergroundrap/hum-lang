# Hum Work Order 21: Organize Active And Closed Work Orders

Date: 2026-08-13
<!-- hum-active-workorder:v1 -->
Status: Work Order 21 planning is independently accepted, committed, published,
and terminal-green. The accepted planning package was committed as
`a0ff6466b43f46afe3341e3e8453313dc954845c` and published by a normal
non-force fast-forward of `main` only. Workflow `ci`, run `31762233813`, attempt
1, tested that exact SHA and concluded `success`. Ubuntu job `94650838755` and
Windows job `94650838773` both succeeded, selected `mode=full` with
`reason=no_status_transition`, and completed the required full preflight. Unit
A is the next planned unit but remains unauthorized.
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

1. Exactly one active-marker line exists repository-wide.
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
| A11 | `duplicate_repository_marker_rejected` | Valid canonical tree | Add a second marker line to the sole active file | `full` | `no_status_transition` | Repository-wide marker cardinality is exactly one | adversarial |
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
| Two active markers | A, B, C | classifier/tree evidence | fail closed/full |
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

The sole current authorization is creation of this local planning-publication
status commit after the required document-only evidence. Nothing may be pushed
under this gate.

Unit A is the next planned unit but remains unauthorized. It may begin only
after all three separate gates complete:

1. this exact status commit is separately authorized for publication;
2. its required fast-lane CI reaches terminal-green on Ubuntu and Windows; and
3. the BDFL issues a fresh explicit Unit A resumption signal.

Unit B, Unit C, migration activity, successor planning, evidence-harness
consolidation, compiler work, Work Order movement, classifier or policy edits,
stash operations, archive mutation, history rewriting, tags, and all later work
remain unauthorized. This status record changes no planning requirement,
consumer audit, case ledger, budget, topology, subject, exclusion, or stop
condition.

The next eligible gate is separately authorized publication of the exact local
status commit followed by terminal fast-lane CI. That eligibility is not
publication authority and does not authorize Unit A automatically.

<!-- workorder-current-authorization-gate:end -->
