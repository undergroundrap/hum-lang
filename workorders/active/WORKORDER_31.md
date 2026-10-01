# Hum Work Order 31: caller-mutation repair — declaration-time lexical-scope restoration and change-call admission

Date: 2026-09-30
Status: DRAFT — pre-issuance review. Not active. WO29 remains the active Work
Order. (The active-workorder marker comment is intentionally absent from this
file; it is added only when this Work Order is activated.)

## Authorization

Decision 0014 (ownership model) is the policy authority. Codex's
consolidated ruling on the caller-mutation (`change` argument) repair map —
draft PR #63, head `a8b9246` — is recorded in decision 0014 as accepted
under delegated authority (BDFL veto open); the frozen map is this Work
Order's planning baseline. The map's correction history is not copied
here — the frozen map is the reference.

The semantic choices D2–D6 and the diagnostic identities are decided
under delegated authority (BDFL veto open), recorded in decision 0014's
consolidated ruling: whole roots and direct fields only (indexed,
deeper, non-place shapes rejected); the reviewed permission matrix,
with explicit `change` arguments for Change parameters; Session W's
permission-bearing `try` restriction stays; admission precedence
shape → permission → authority → overlap with whole-call stage sweeps;
conservative CallAccess invalidation including no-op calls; the narrow
non-owning resource escapes rejected on source-place declared type ×
root authority. Approved identities: H0810 overlapping change
arguments, H0811 unsupported change-argument place, H0812
change-argument permission mismatch, H0813 change argument on immutable
place, H0814 ownership transfer from non-owning parameter, H0815
recognized linear-resource escape from non-owning authority;
moved-place admission reuses H0801 as a proper diagnostic at the new
admission site; H0802/H0806/H0807/H0808/H0809 identities preserved.
Catalog implementation remains unauthorized. This draft carries no
implementation authorization and no active marker; a draft Work Order
changes nothing until it is activated.

## Mission and queue position

Repair caller-visible mutation for `change` arguments in two parts:

- **Item A — declaration-time lexical-scope restoration.** The map's §4.3
  credited repair: `eval_binding` saves the displaced complete
  `RuntimeBinding` after initializer evaluation, immediately before the
  executed declaration replaces it, on all insertion paths; `eval_block`
  restores only declarations actually executed in that scope via
  `restore_binding`, replacing the entry-time `block_binding_names`
  precomputation; loop-body declarations saved/restored per iteration.
  Completed writes, moved/view metadata, and early-exit behavior are
  preserved; parameter identity through `definition_id`; final-value
  capture from the actual body `env`, not the postcondition `exit_env`.
- **Item B — change-call admission, transfer, invalidation, and the narrow
  ownership-escape controls.** The map's §§4.2/4.5/4.7/4.9/4.9a: the
  two-phase argument loop at the user-task call site; phase-0
  pre-authority admission (arity/unknown-task fail-fast, D4 shape, D5
  keyword/permission) and phase-2 authority (`ensure_can_set` family) →
  overlap (H0810 backstop after the authority sweep, then live-alias
  H0808) → snapshots; the §4.5 final-value transfer plus the
  `CallAccess` overlap sweep at the call site through the real call
  owner on every task exit (`Returned`, `Failed`, `ContractViolation` —
  D1/D1-sub ruled); the narrow §4.9a ownership-escape rejection keyed on
  source-place declared type crossed with root permission.

Queue: the consolidated semantic/diagnostic decision is recorded in
decision 0014 (delegated authority, BDFL veto open); this Work Order
executes after WO29 closes and on Ocean's activation call. Session
letters are assigned from the project odometer at issuance (the odometer
stands at AG as of WO29); these items take the next letters in sequence.

## Evidence base

The frozen repair map (PR #63, `a8b9246`); decision 0014 with its
honesty lock; decision 0015's evidence vocabulary (`proved`,
`boundary`, `unproved`, `external-trust`). Executed probes cited in the
map were run by Claude; walk mechanics are source inspection at the
pinned commit, not reruns — the distinction is preserved in the
evidence requirements below.

## Item A — declaration-time lexical-scope restoration

The repair from the map's §4.3, credited and unchanged:

- `eval_binding` saves the displaced complete `RuntimeBinding` after
  initializer evaluation and immediately before an executed declaration
  replaces it. All insertion paths are covered.
- `eval_block` restores only declarations actually executed in that
  scope via `restore_binding`, replacing entry-time
  `block_binding_names` precomputation.
- Loop-body declarations are saved/restored per iteration.
- Completed writes, moved/view metadata, and early-exit behavior are
  preserved.
- Parameter identity through `definition_id` remains required, but
  does not itself repair rollback.

Standalone acceptance controls (effects reachable without Item B;
fixtures observe the effect, not the form): `set` to an outer binding
whose shadowing declaration never executes (write survives);
nested-block shadowing restored on return/fail/contract-violation
paths; for-each per-iteration isolation. Caller-mutating-initializer
coverage lives in Item B's integrated coverage, with explicit `change`
arguments where the permission rules require them.

## Item B — change-call admission, transfer, invalidation, and the narrow ownership-escape controls

From the map's §§4.2/4.5/4.7/4.9/4.9a:

- Two-phase argument loop at the user-task call site, with the decided
  admission precedence shape → permission → authority → overlap. Phase 0
  runs shape (H0811 for indexed, deeper, and non-place shapes) then
  permission (H0812 per the reviewed matrix — Change parameters
  require explicit `change` arguments); phase 2 runs authority
  (`ensure_can_set` family; H0813 for immutable places; moved-place
  admission reuses H0801 as a proper diagnostic at the new admission
  site) → overlap (H0810 backstop after the authority sweep, then
  live-alias H0808) → snapshots. Each stage sweeps the whole call
  before the next stage begins: a later argument's authority failure
  beats an earlier argument's overlap. New consume-branch guard and
  ordinary/`borrow`-path guard for the §4.9 non-ownership controls
  (H0814 for ownership transfer from a non-owning parameter).
- Final-value transfer (§4.5): thread final parameter values through
  `execute_task`/`execute_task_body`; final values are captured from
  the actual body `env`, not the postcondition `exit_env`; the
  transfer plus the `CallAccess` overlap sweep runs at the call site
  through the real call owner on every task exit (`Returned`,
  `Failed`, `ContractViolation` — D1/D1-sub ruled). New H0807 `(Field,
  CallAccess)` / `(Element, CallAccess)` trap arms on the existing
  `stale_view_trap` path (`FieldWrite`/`ListAppend` arms unchanged).
  No per-mutation instrumentation, no replay journal.
- Narrow ownership-escape rejection (§4.9a, H0815): ownership escapes
  from Borrow/Change authority over recognized linear resource types
  are *rejected* through annotated/unannotated bindings,
  resource-valued field/view copies, and returns — keyed on the
  source-place declared type (`Param.ty` for whole parameters; the
  field's declared type from `TypeDef` `Field.ty` for direct fields)
  crossed with root permission in {Borrow, Change}.
  `is_linear_resource_type` is unchanged; declared-type threading to
  the runtime consumers is the Builder's reviewed implementation
  choice. No implicit moves, no additional owned snapshot, no general
  copyability or linear-safety claim.
  Acceptance: `change t: Transaction` → `let y = t` / `return t`
  rejected; `change t: Transaction` → `let i = t.id` / `return t.id`
  (`UInt` field) permitted; non-linear record's `Transaction` field →
  owned copy/return rejected; non-linear `Counter` copying permitted.
- Integrated coverage (Items A+B): caller-mutating-initializer
  controls use explicit `change` arguments where the permission rules
  require them (e.g. `change x = bump(change x)`) — the callee's write
  to the outer place is preserved by the Item A save before the
  shadowing declaration replaces the binding. The write survives, and
  post-block reads observe the shadowed value.
- `examples/probes/transaction_once.hum` is carried as the
  writable-authority compatibility witness: no zero-compatibility-impact
  claim is made — the example as written today exercises the newly
  rejected shape, and the `let txn` → `change txn: Transaction`
  migration is verified at implementation time, not assumed here. The
  `hum run … --entry transfer --args 10` → `ok` assertion
  (`tools/check_all.ps1`) must keep passing unchanged.

## Affected-file inventory (proposed envelope — only if the repair is approved)

The map's §7 is this plan's one inventory. Line numbers below are at the
map's pinned commit `0e215d3` (historical source pin — main has since moved).

| File | Change | Why |
|---|---|---|
| `src/run.rs` | Two-phase argument loop at the user-task call site (§4.2); thread final parameter values through `execute_task`/`execute_task_body`; the §4.5 final-value transfer + `CallAccess` overlap sweep at the call site through the real call owner on every exit; runtime admission (phase 0 pre-authority, phase 2 authority → overlap → snapshots); §4.9 non-ownership guards; new H0807 `(Field, CallAccess)` / `(Element, CallAccess)` trap arms; Item A declaration-time lexical-scope repair (`eval_binding` save, `eval_block` restore via `restore_binding`, per-iteration re-save); §4.9a escape rejection — runtime consumers `eval_binding` and the `return` arm, with the parameter binding carrying the declared-type facts `RuntimeBinding::parameter` drops today | The defect and the runtime mechanism owner live here |
| `src/main.rs` | Crate-root declaration only: add `mod change_arg_admission;`. Named as the consumer adapter the plan stays consistent with: `hum ownership-check` report path, `hum run --native` preflight admission gate, trap printing + exit 2, the `hum check` stage pipeline (no new stage; the D3 `stages` listing stays truthful) | Consumer adapter; stages truthfulness |
| `src/diagnostic.rs` | No production change proposed. The new H0807 `CallAccess` trap arms construct related spans through the existing `with_related_span` builder | Consumed by the new H0807 arms; no change needed |
| `src/diagnostic_catalog.rs` | Diagnostic identities H0810–H0815 are approved; catalog implementation remains unauthorized — the §8-style allocation checklist per code stays undone until separately authorized. Regardless of catalog authorization: the H0807 detail entry's explanation/repair prose covers call-access invalidation — prose only, code row unchanged, no cause key added | Designated diagnostic-identity owner |
| `src/ownership_check.rs` | Only if the repair is approved: static admission of the decided shapes, syntactic overlap, authority branches, Known Borrow → H0802 ordered before overlap, and the non-ownership→Consume-parameter branch — new per-call admission branches calling the shared producer, invoked for every user-task call expression, emitting the approved designated diagnostics into the existing occurrence set | Proposed static admission owner; the H08x family's stage |
| `src/change_arg_admission.rs` | New, narrow — only if the repair is approved: the pure shared static admission producer (call expression + callee/caller declared permissions + caller binding mutability → ordered admissions, first failure wins) plus binding/return admission branches keyed on declared-type × source-permission, plus shared message/help builders. Emits no diagnostics itself; consumed by the `ownership_check` branches and the `full_type_check` adapter | Single static admission producer; shared text so codes, spans, and precedence cannot drift |
| `src/full_type_check.rs` | Only if the repair is approved: the narrow `hum check` adapter at `check_stage_outcome` — deliberately not inside the shared `build_report_with` walk that effect checking consumes via `with_full_type_for_effect`. No new stage; no `main.rs` change; the D3 `stages` listing is unchanged | Ordinary-check consumer of the shared producer |
| `src/diagnostics.rs` | No hand edit proposed. The `hum diagnostics` count strings derive from `diagnostic_catalog::all()` via `format!`; only the test literals change, and only if the catalog changes | `hum diagnostics` contract; affected consumer |
| `docs/DIAGNOSTICS.md` | Mirror rows for accepted codes (test-enforced via `validate_human_projection`; catalog mirror, not prose). Regardless of acceptances: the H0807 mirror row is updated to cover call-access invalidation — prose only | Standing catalog-mirror rule |
| `tools/check_all.ps1` | Pinned count inside `Invoke-HumCompilerFrontChecks` (only when the catalog implementation is authorized); new `Read-NativeOutput[WithExit]` assertions for the new fixtures; the `transaction_once` `ok` assertion must keep passing unchanged after the example's implementation-time migration | Standing pin practice; acceptance assertions live here |
| `tools/test_ci_policy.ps1` | `$CompilerBodies` digest re-pin for `Invoke-HumCompilerFrontChecks` (only if that function's body changes; a `src/run.rs`-only repair does not trip it) | Standing pin practice (decision 0028) |
| `fixtures/` | New `.hum` fixtures + session tests exercised through the real user-task call path — placed by family. Counterexample fixtures: nested-call preservation; `change` + `consume` rejection (no resurrection); `consume` of a Change/Borrow param rejection; immutable-place `change` rejection. No-change-keyword controls per the §4.9 rows. Item A lexical-scope controls and Item B §4.9a escape controls per the acceptance rows above | Positive-evidence rule: fixtures must observe the effect, not merely declare the form |
| `examples/probes/transaction_once.hum` | Not edited during planning. Carried as the writable-authority compatibility witness (see Item B) | Compatibility witness for the authority rule |
| `src/run.rs` `mod tests` | Internal tests through `run_program_with_adapters` asserting the observation points (post-call reads, stale-view trap-or-success) through the real evaluator — no journal | Test-seam distinction |

Out of scope: every other `src/` file, the CLI surface (no new flags), Work Order edits, and any try/catch machinery (D6 preserved). The parser already admits `change`-argument syntax — no syntax change is proposed.

## Evidence and acceptance requirements

- **Honesty locks (decision 0014):** no output text or doc may claim more
  than the checker proves. Evidence vocabulary per decision 0015:
  `proved`, `boundary`, `unproved`, `external-trust`.
- **Actual-command vs internal evidence:** actual-command tests (`hum run
  fixtures/… --entry …`, asserting stdout + exit code via the
  `tools/check_all.ps1` `Read-NativeOutput[WithExit]` pattern) prove the
  CLI-observable contract. Internal tests (`cargo test` via
  `run_program_with_adapters`) run the same fixture programs through the
  real interpreter and assert `RunOutcome` + `report.diagnostics`. Both
  seams execute the real call owner; neither invents evidence. No
  journal. Only shapes admitted by the production pipeline are claimed
  as public execution.
- **Private real-evaluator/call-boundary tests are required:** private
  tests retain the caller Env and invoke the real evaluator/call
  boundary directly, inspecting actual post-transfer caller values and
  invalidated-view markers after `Failed` and `ContractViolation`
  exits — through the production code path. Public CLI evidence is
  labeled separately from this internal coverage; Session W's
  permission-bearing `try` restriction is unchanged. No journal, no
  synthetic replay.
- **Positive-evidence rule:** a passing fixture must observe the effect
  (the changed caller value, the fired contract/trap), not merely
  declare the form.
- **Pre-push gates:** `cargo fmt --all -- --check`, `cargo clippy
  --workspace --all-targets -- -D warnings`, `cargo test --bin hum`,
  `tools/test_ci_policy.ps1`, hand-run the exact commands behind any
  added/changed Item A/Item B acceptance assertions, `tools/check_public_readiness.ps1`,
  text hygiene.
- **Draft until separately authorized.** A green CI Full run never
  marks the draft ready by itself: readiness requires the Codex review
  verdict (architect-reviewer role: independent probes, scope check,
  diagnostic quality, honesty locks) and the owner's separate
  authorization to mark ready. Report the run ID. Red on a draft is
  contained; nothing merges on red.
- **Commit identity:** Ocean's GitHub noreply identity; no agent
  attribution anywhere.

## Lane assignments and STOP conditions

- Builder lane implements; research lane owns docs. Lane separation is
  absolute; conflicts are reported to Ocean, never resolved across lanes.
- STOP: missing language surface is a STOP, never a workaround (decision
  0027). Implementation requires Work Order activation — not granted by
  this draft.

## Review and evidence requirements

Pre-issuance review: this draft gets an independent review pass before
issuance — authority validity (the sequence respects accepted decisions
and governance; no honesty lock is overstated), session sizing (every
session is review-sized when tightly pinned; split any that are not),
and evidence linkage (every mandate traces to production code, a focused
observation, or accepted strategy, not to momentum) — by a capable actor
who did not author or edit the deliverable. This draft PR is published
for that independent review.
