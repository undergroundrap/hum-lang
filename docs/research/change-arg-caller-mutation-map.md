# Bounded repair map: caller-visible mutation through user-task `change` arguments

Date: 2026-09-29 (correction pass). Role: Builder (planning only).
Pinned audit source: `0e215d31e072f14db7f6e68e2e8782ecd86cb77d`
(verified present on disk; `git cat-file -t` = commit; zero `src/` delta
vs baseline `0cd5e4630b9259c600a7b16de0e63bd628351032`). All line numbers
below are from the pinned head, re-read 2026-09-29 via read-only git
object inspection. No repository files or worktrees were changed; no
builds, tests, Work Order activation, commits, pushes, CI, or publication
were performed. All existing worktrees preserved untouched.

No implementation is authorized. This map is revised and the task stops.

Evidence limitation (standing): "Claude's retained D1 evidence" was not
locatable as a distinct accessible artifact in this agent's context; a
terminated search only found the string `D1` inside
`~/workspace/hum-research/notes/design_decisions_evaluation.md`, which was
not inspected. Do not search indefinitely. Any reproducer recreated later
is **new evidence**, never recovered Claude evidence, and must be labeled
as such.

Withdrawn claim: the earlier map's "copy-in/copy-out demonstrated
equivalent" (§5) is withdrawn in full. A final-value comparison cannot
reconstruct the mutation/effect history (no-op calls, write-then-restore,
invalidation timing, nested forwarding, arg-eval side effects). §4 below
replaces it with an explicit mutation/effect mechanism through the real
call owner.

## 1. The defect (what is wrong today)

In `src/run.rs`, `eval_primary`'s user-task call branch (arg loop at
3143–3160, inside the call dispatch at 3085–3167): `change` arguments are
stripped of the keyword by `strip_borrow_or_change_argument` (5410) and
evaluated as ordinary *expressions*. The resulting `Value` is passed by
copy into `execute_task(task, values: Vec<Value>)` (2040).
`execute_task_body` (2072) builds a fresh per-call `Env` and inserts
callee param bindings *by value* (`RuntimeBinding::parameter`, 372).
Every callee-side mutation (`set`, `list_append`, field writes, writable
aliases) applies only to the callee's private env. There is no write-back
path. Net effect: on user-task calls, `change` is semantically a no-op
for the caller — contradicting decision 0014 ("`change` grants bounded
exclusive mutation authority") and `docs/LANGUAGE_REFERENCE.md:391`
("Use `change` when the task may write through the parameter"). The
checker does not close the gap: no static validator
(`full_type_check.rs`, `ownership_check.rs`, `callable.rs`) inspects
call-argument `change`/`borrow`/`consume` keywords against callee
parameter permissions; the interpreter's call site is the enforcement
point. The honest record: this repair is a **first implementation** of
the documented write-through semantic, claim-locked by 0014's honesty
locks — not a restoration of previously-correct behavior.

## 2. Source owners (pinned line numbers)

- **Argument owner:** `eval_primary` user-task call branch (3085–3167);
  the argument loop proper is 3143–3160: arity check (3137–3142) ->
  `consume_argument_root` (5404) + `read_consume_value` (4754) +
  `mark_moved` (4826) on the consume path -> `strip_borrow_or_change_argument`
  (5410) + `eval_expr` on the borrow/change/ordinary path, with early
  return on `Evaluated::Failure` / `Evaluated::ContractViolation`
  (3153–3157).
- **Place owners:** `strip_keyword`, `place_root` (5505),
  `field_place::split_field_place` (src/field_place.rs — single level
  only; `a.b.c` -> None), `element_place::split_element_place`
  (src/element_place.rs — bare-root lists only; `r.s[0]` -> None).
- **Environment owner:** `Env = BTreeMap<String, RuntimeBinding>`;
  `RuntimeBinding::parameter` (372; `writable = permission != Borrow`);
  `RuntimeViewKind::{Field, Element}` and `RuntimeView` (views name only
  single field places or bare-root element places);
  `resolve_writable_alias_place` (4738); `ensure_can_set` (4780);
  `write_place` (4803 — whole-root replacement and direct `root.field`
  replacement only; no element path; deeper places trap fail-closed);
  `mark_moved` (4826 — only `Local`/`Consume` permission bindings are
  marked); `ensure_linear_closed_on_exit` (4840);
  `active_iteration_for` (4661) + `iteration_mutation_trap` (4884,
  emits H0806); `invalidate_field_views` (5445 — no-op unless the place
  is a single field place; invalidates only exact-`source_place` field
  views); `invalidate_element_views_for_growth` (5463 — invalidates
  element views whose `place_root(source_place)` equals the list root).
- **Call owner:** `execute_task` (2040 — establishes active task
  routing, then `execute_task_body`); `execute_task_body` (2072–2130):
  `does:` required -> body analysis -> writable-alias analysis +
  `preflight_writable_aliases` (H0808/H0809, **before** env creation) ->
  fresh `Env`, params inserted by value -> `needs:` contract (failure ->
  `TaskResult::ContractViolation`, body never ran) ->
  `capture_old_contract_values` (2306, after `needs:`, before body) ->
  body eval -> exit dispatch: `Return` -> `ensure_return_dependency` ->
  `finish_success` (2294, evaluates `ensures:` against a cloned exit
  env); `Fail` -> `TaskResult::Failed` directly (2126, no second linear
  check); fallthrough -> linear-close check (2128) -> `finish_success`
  with `Unit`.
- **Direct-fail linear behavior (corrected):** explicit typed `fail`
  runs `ensure_linear_closed_on_exit(..., "fail", ...)` **before**
  producing `Flow::Fail` (2664–2665); a typed failure produced while
  evaluating a `return` expression also checks linear closure first
  (2649–2651); successful explicit return checks closure in the
  statement evaluator (~2642). The earlier map's "linear-close check is
  skipped on the fail path" was wrong: the check happens at the fail
  *statement* site, and `execute_task_body` maps the resulting
  `Flow::Fail` straight through.
- **Exit-path owners:** `TaskResult::{Returned, Failed,
  ContractViolation}` rendered by `run_program` (562) as
  `RunOutcome::{...}`; `Err(String)` -> `RunOutcome::Trap`
  (program-fatal); exit codes in main.rs.
- **Failure-value owner:** `FailureValue` / `Flow::Fail` /
  `typed_failure` module; `try` shape restriction (Session W):
  `let value = try named_call(...)` with ordinary value arguments only
  — `borrow`, `change`, `consume`, nested calls, and operators remain
  unsupported there (typed_failure.rs ~932). Preserved unchanged; no new
  try/catch surface is authorized.
- **Value equality:** `Value` derives `PartialEq, Eq` (285–286) — a
  final==initial comparison is available, but §3.4 explains why it is
  not the mechanism.

## 3. The complete mutation/effect path (what the repair must preserve)

### 3.1 Root and field writes

`eval_set` (2756): resolve writable-alias place -> `place_root` ->
`ensure_can_set` (unknown place -> plain trap; moved ->
`use_after_move_invariant` plain trap; `Borrow` permission ->
`borrow_mutation_trap`, H0802 diagnostic; not writable -> plain
"cannot set immutable place" trap) -> evaluate RHS -> `write_place` ->
**unconditional** `invalidate_field_views`. Two facts the write-back
must mirror: (a) invalidation runs on *every* successful write, even
when the written value equals the old value; (b) `write_place` traps
fail-closed on shape mismatch ("`{root}` is not a record", "record
`{root}` has no field `{field}`", "unsupported set place `{place}`")
and clears `moved_at`/`moved_by` on success. The write-back reuses
`write_place` and `invalidate_field_views` — no new write or
invalidation machinery except the bounded depth-2 case in §4.5.

### 3.2 List-growth invalidation

`eval_list_append` (4550): requires `change list`; rejects mutation
during active iteration (`active_iteration_for` -> H0806); evaluates
the item; appends directly to the root binding; calls
`invalidate_element_views_for_growth`. Element views can only name
bare-root lists (`xs[0]`; `r.s[0]` is not a valid element place), so a
grown list field (`r.s`) has no element views to invalidate — the value
write alone suffices there. The write-back reuses
`invalidate_element_views_for_growth` only when the caller place is a
bare root; calling it with `place_root` of a field place would
over-invalidate sibling lists' views and is forbidden.

### 3.3 No-op calls

A callee that never writes its `change` parameter must produce **zero**
caller-visible effects: no write, no invalidation. In particular, a
caller field/element view taken before the call must remain valid.
This is why the design (§4.4) tracks whether the callee actually wrote
instead of comparing final vs initial values.

### 3.4 Writes restoring the original value

A callee that writes a different value and later restores the original
**did perform writes**: `eval_set`'s unconditional write+invalidate
already ran inside the callee env. The write-back must perform a real
`write_place` and real invalidation even when final == initial.
Value comparison alone would silently drop the effect history — this is
the concrete reason the withdrawn copy-in/copy-out equivalence fails.

### 3.5 Nested forwarding

`outer(change r)` -> `middle(change r)` -> `inner(change r)`: each frame
owns its env; each frame records its own caller place (`r` in its
caller's env). Copy-in reads the post-inner-eval value at each level;
write-back flows the final value back frame by frame, preserving the
original caller place and authority. Invalidation is applied per frame
iff that frame's callee actually wrote (§4.4). No frame can observe
another frame's env; the chain composes through the values alone.

### 3.6 Side effects during argument evaluation

Caller argument expressions evaluate **left-to-right before**
`execute_task` (3143–3160). Each argument's `eval_expr` may contain
nested calls, and under the repair each nested call completes its own
copy-in/call/write-back inline, in order. Consequences the design
preserves:

- A nested call's write-back to a place the outer call later uses as a
  `change` place is visible in the outer copy-in snapshot (phase 2
  reads post-eval state).
- `consume` arguments mark moved **during** arg evaluation, exactly as
  today (4826); a later pre-body rejection does not undo the mark.
- On `Evaluated::Failure` / `Evaluated::ContractViolation` from any
  argument, the loop returns early exactly as today (3153–3157): the
  outer call never happens, so there is no outer write-back; effects
  already produced (earlier consumes, completed nested-call
  write-backs) stand. Callee pre-body rejection is therefore always
  separated from prior argument-evaluation effects — rejection prevents
  the callee body from running but never rewrites history.

### 3.7 What write-back must not do

No silent loss (every recorded callee write reaches the caller place or
traps fail-closed — never dropped); no over-invalidation (only the
existing helpers, only on the exact places they already cover;
unrelated views stay valid); no blanket view bans to simplify the
mechanism (borrow views live through the call and are invalidated
iff the callee actually wrote, via the existing H0807 path).

## 4. Repair design (replaces the withdrawn equivalence claim)

### 4.1 Mechanism, not inference

The implementation direction is an explicit mutation/effect mechanism
through the real call owner: the user-task call site records per-`change`
argument (caller place, arg index, span); `execute_task` returns the
callee's final parameter outcomes (value + recorded write effects);
the call site performs the write-back with `write_place` +
`invalidate_field_views` / `invalidate_element_views_for_growth`.
Final-value *differences* are never inferred into effects.

### 4.2 Two-phase argument loop (eval_primary call branch)

- **Phase 0 — static validation, before any arg evaluation** (extends
  the existing arity check at 3137–3142, same fail-fast precedent):
  D5 keyword/permission matrix (§4.7); change-place shape validation
  (root or single field only; element places -> D2 rejection; deeper or
  non-place expressions -> rejection); overlap detection on resolved
  places (§4.6). Deterministic first-failure in arg order.
- **Phase 1 — evaluate left-to-right** (current loop, minus change
  args): consume path unchanged (`read_consume_value` + `mark_moved`
  immediately); ordinary/`borrow` args via `eval_expr` unchanged;
  `change` args are recorded, not evaluated. Early return on
  `Failure`/`ContractViolation` unchanged.
- **Phase 2 — per change arg, in order, after all arg evals:**
  writability via the existing `ensure_can_set` on the caller env
  (unknown/moved/borrow/immutable -> existing traps, §6 reuse table) +
  the existing iteration trap (`active_iteration_for` ->
  `iteration_mutation_trap`, H0806) for change args on actively
  iterated roots; then copy-in snapshot read (root or single-field
  read mirroring `eval_primary`'s read path — *not*
  `read_consume_value`, which is consume-specific). A place consumed by
  an earlier argument traps here via the moved check.

### 4.3 Threading final parameter outcomes

`execute_task` (2040) gains a second result: the callee's final
parameter outcomes in param order — value plus recorded write effects
(§4.4). The four call sites: entry path (1837), expression-application
(3047), direct application (3077) destructure and ignore it; only the
user-task call site (3161) consumes it for write-back. Inside
`execute_task_body`, outcomes are captured from the callee env at each
exit point: `Return`/fallthrough (env in scope), `Fail` (env in
scope at 2126), `needs:`-`ContractViolation` (body never ran — no
write-back; outcomes empty), `ensures:`-`ContractViolation` inside
`finish_success` (write-back per the carried D1-sub recommendation:
completed mutations survive postcondition failure).

### 4.4 Write-effect tracking (why comparison is insufficient)

`RuntimeBinding` gains additive write-effect records, populated by the
existing mutation points: `write_place` records the written sub-place
(`""` for the root, the field name for a direct field write);
`eval_list_append` records list growth. The write-back consults these
records, never a value diff:

- no records -> skip entirely (no write, no invalidation): §3.3.
- records present -> real `write_place` + real invalidation even when
  final == initial: §3.4.

### 4.5 Write-back algorithm (per change argument)

Let P be the caller place after `resolve_writable_alias_place`
(P is a root `r` or a single field `r.s`; deeper is impossible by §2),
and let the callee param's final root value be Vf with recorded
sub-places W and growth flag G:

- If `""` in W: `write_place(caller_env, P, Vf)` (subsumes field
  writes); `invalidate_field_views(caller_env, P)` (exact-match; no-op
  for bare roots, correct for field places).
- Else for each field `f` in W: write Vf's field `f` to the effective
  place P.f — via `write_place` when P is a root, via one bounded
  depth-2 descent (new, write-back-only, ~20 lines mirroring
  `write_place`'s record descent) when P is `r.s`; then
  `invalidate_field_views` on P and on P.f (the P.f call is a harmless
  no-op when deeper than one level, since no view can name it; the P
  call catches views of the changed record).
- If G and P is a bare root: `invalidate_element_views_for_growth(P)`;
  if P is a field place, value write only (no element views can name
  it — §3.2).
- Any shape mismatch traps fail-closed via `write_place`'s existing
  errors. No recovery is invented.

### 4.6 Overlap rejection (exclusive mutation, precisely)

Forbidden overlap is **rejected**; left-to-right copy-back order is not
an alternative and is not offered. Checked in phase 0 on resolved
places (alias-resolved, so `change a1`/`change a2` aliasing the same
field is caught):

- two `change` args with overlapping places (same place, or one a
  field-prefix of the other: `change r` vs `change r.s`) -> reject.
  New narrow code **H0810** "overlapping change arguments",
  family `ownership_borrowing` — H0808 ("writable alias overlap") is
  not reused here because its message/help is written for `let alias =
  change` declarations; conflating the constructs would mislead. Full
  allocation checklist applies (§8).
- `change` arg overlapping a `borrow` arg's place -> reject (H0810).
  Two `borrow` args on the same place stay allowed (shared read,
  existing behavior).
- `change` arg overlapping a **live caller writable alias** ->
  reject by reusing **H0808**: the call creates a second live writer,
  which is exactly what H0808's "live overlapping access" rule
  forbids. No new code.
- `change` arg overlapping a live caller **borrow view** -> allowed;
  the view is invalidated iff the callee actually wrote, via the
  existing H0807 path (§3.7 — no blanket view ban).

### 4.7 Keyword/permission matrix (D5, scoped to change cells)

- `change x` -> `change` param: the repair (write-back).
- `change x` -> `borrow` param: reject (the caller grants write
  authority the callee cannot exercise; silently ignoring the keyword
  is the old bug in a new shape).
- `change x` -> `consume` param: reject (write-back is impossible; the
  value is moved).
- ordinary `x` -> `change` param: reject (without the keyword there is
  no write-back obligation; silently dropping callee mutations is the
  defect being repaired — the keyword is the explicit exclusive-write
  grant). Zero blast radius: every existing fixture/example that calls
  a `change`-param task already uses the keyword
  (`session_o_complete_item_field_place.hum:27`,
  `session_t_wrong_swap_contract.hum:26`); entry tasks take CLI args,
  not keywords (§4.8).
- `borrow`/`consume` arg cells and non-change params: existing behavior
  preserved as-is (out of this repair's scope).

### 4.8 Exemptions (settled, unchanged)

- **Entry-task args** (`run_task_with_args`, 1837; `--args` values):
  no caller place exists -> no write-back possible or needed. Pinned by
  `hum run fixtures/ownership_check/session_j_change_pass.hum --entry
  increment --args 7` -> `8`.
- **Callable-application paths** (3047, 3077): statically restricted to
  ordinary arguments (Session AL: `session_al_permission_argument_fail`
  -> H1401 at every surface) -> never carry change places.
- **`try` expressions:** Session W restriction preserved exactly
  (ordinary value arguments only); no write-back surface is added to
  `try`, and no new try/catch surface is authorized.

## 5. Settled rules / unsupported shapes / genuine policy choices

### Settled (from 0014/0016, existing behavior, this audit)

- `change` = bounded exclusive mutation authority, write-through at
  the boundary (0014; LANGUAGE_REFERENCE:391 already correct).
- No implicit rollback: linear resources are the exactly-once protocol
  (0014 §5); the interpreter keeps no journal (0010's explicit state
  model). Completed mutations survive ordinary typed failure and (per
  the carried recommendation) postcondition failure.
- Typed failure is causal (0016); the linear-close check runs at the
  fail/return statement sites (§2), and `Flow::Fail` maps directly to
  `TaskResult::Failed`.
- Existing H08x rules unchanged: H0801 (use-after-move), H0802
  (borrowed write), H0803/H0804 (linear), H0805 (returned view), H0806
  (iteration mutation), H0807 (stale views), H0808/H0809 (writable
  aliases). `try` keeps its Session W restriction.
- `needs:`-failure: body never ran -> no write-back (nothing to
  preserve). Pre-body rejection never undoes prior arg-eval effects
  (§3.6).
- Place vocabulary is single-level everywhere (`set`, views, aliases,
  `write_place`): deeper places stay rejected.
- Entry args, callable applications, and `try` are exempt (§4.8).

### Unsupported shapes (rejected, no new machinery)

- Deeper places (`a.b.c`, `xs[0].f`) as change args: rejected (consistent
  with every other place consumer).
- Element places (`xs[0]`) as change args: rejected at the call site
  (D2, recommended) — `write_place` has no element path, and adding one
  would change `set xs[0] = v` semantics (today it replaces the whole
  root binding — a separate latent hazard, out of scope).
- Non-place expressions (`change 5`, `change (a+b)`, nested calls
  `change g(x)`): rejected — no caller place exists. Plain runtime
  trap (call-shape errors like arity/unknown-task are plain traps
  today; no catalog allocation justified — §6).

### Genuine policy choices (need Ocean/Codex ruling)

- **D1:** completed mutations survive ordinary typed failure; no
  implicit rollback. Carried as recommendation pending approval
  (follows from no-journal + 0014 §5).
- **D1-sub:** write-back on `ensures:`-failure (`ContractViolation`
  after the body ran). Carried as recommendation pending approval
  (uniformity: the body ran, its mutations are facts).
- **D2:** element-place change args — reject at call site
  (recommended) vs extend `write_place` with element writes (wider
  than this repair).
- **D3:** overlap rejection code — new narrow **H0810** for
  change/change and change/borrow-arg overlap (recommended, checklist
  in §8) vs stretching H0808 (rejected: message/help would mislead).
  Live-writable-alias overlap reuses H0808 (settled, §4.6).
- **D4:** non-place change args — reject (required); plain trap
  (recommended, §6 justification) — no open point beyond ruling D4
  itself.
- **D5:** keyword/permission mismatch rejection per §4.7 (recommended).
- **D6:** `try` with change args stays unsupported (preserve Session
  W).

## 6. Diagnostic reuse justification (before any allocation)

| New rejection | Existing mechanism reused | New code? |
|---|---|---|
| change arg on `let`-bound (immutable) root | `ensure_can_set` -> plain "cannot set immutable place" trap (same as `set`) | No |
| change arg on borrow-permission root | `ensure_can_set` -> `borrow_mutation_trap`, **H0802** (same authority violation as in-body write through a borrow) | No |
| change arg on moved root | `ensure_can_set` -> `use_after_move_invariant` plain trap (checker preflight is the primary guard; runtime is the backstop) | No |
| change arg on actively iterated root | `active_iteration_for` + `iteration_mutation_trap`, **H0806** (same structural-mutation-during-iteration conflict) | No |
| change arg overlapping live caller writable alias | **H0808** (the call creates the second live writer H0808 exists to forbid) | No |
| change/change or change/borrow-arg overlap in one call | — (no existing code names this construct; H0808's wording is alias-specific) | **H0810** proposed |
| non-place change arg (`change 5`, `change (a+b)`, `change g(x)`) | plain trap — consistent with arity ("expects N arguments, got M") and unknown-task call-shape errors, which are traps, not diagnostics | No |
| keyword/permission mismatch (§4.7) | plain trap — same call-shape family as arity | No |
| element-place change arg | plain trap (shape limitation, mirrors `write_place`'s "unsupported set place") | No |

H0810 ("overlapping change arguments", family `ownership_borrowing`,
owner `ownership_check` — runtime emission has precedent: H0802's
`borrow_mutation_trap` is runtime-emitted) is the **only** proposed
allocation, and only because no existing code names the construct.
Checklist per standing practice: `diagnostic_causes!` entry,
`diagnostic_code_allocations!` entry (next free key index),
`historical_public_ordinal` match arm, `DIAGNOSTICS` detail entry —
continued in the tail completion below.

> **Tail completion — newly authored 2026-09-29 (builder lane). Not recovered text.**
> The revised map as received ended mid-sentence at "`diagnostic_code_allocations!`
> entry (next free key index), " with the literal final line shown below. That line
> is preserved verbatim as evidence; the ~8.6KB of intended tail it stood in for is
> unrecoverable (no second copy exists in the workspace; memory holds no record of
> it). What follows completes the missing tail — H0810 allocation checklist,
> affected-file inventory, acceptance controls, validation plan/platform limits, and
> stop boundary — from the pinned source `0e215d31e072f14db7f6e68e2e8782ecd86cb77d`
> and standing repository requirements. It does not recover the lost bytes, does not
> repeat the broad audit, and claims no prior execution evidence. H0810 stays
> **proposed, not approved or allocated**; D1–D6 stay open pending Ocean/Codex
> approval.
>
> Evidence — original final line, verbatim: `...[truncated 8629 chars]`

## 7. Affected-file inventory (only if the repair is approved)

| File | Change | Why |
|---|---|---|
| `src/run.rs` | Two-phase argument loop at the user-task call site (§4.2, at 3143–3160); thread final parameter outcomes through `execute_task` (2040) / `execute_task_body` (2072); write-back at the call site through the real call owner (§4.5); pre-evaluation overlap rejection (§4.6) | The defect and every owner live here (§2) |
| `src/diagnostic_catalog.rs` | **Only if H0810 is accepted:** `diagnostic_causes!` entry, `diagnostic_code_allocations!` entry, `historical_public_ordinal` arm, `DIAGNOSTICS` detail entry, hardcoded-count updates | §8 checklist |
| `docs/DIAGNOSTICS.md` | Mirror row, **only if H0810 is accepted** (test-enforced via `validate_human_projection`; catalog mirror, not prose) | Standing catalog-mirror rule |
| `tools/check_all.ps1`, `tools/test_ci_policy.ps1` | Pinned diagnostic count + `$CompilerBodies` digest updates, **only if the catalog changes** | Standing pin practice (decision 0028) |
| `fixtures/` | New `.hum` fixtures + session tests exercised **through the real user-task call path**, covering §3.1–§3.6 and the §6 rejection table | Positive-evidence rule: fixtures must observe the effect, not merely declare the form |

Out of scope for the repair: every other `src/` file, the CLI surface (no new flags), Work Order edits, and any try/catch machinery (D6 / Session W preserved).

## 8. H0810 allocation checklist (proposed — not approved, not allocated)

H0810 ("overlapping change arguments", family `ownership_borrowing`, owning stage
`ownership_check`) is the only proposed allocation, and only because no existing code
names the change/change and change/borrow-argument overlap construct (§6). Runtime
emission has precedent: H0802's `borrow_mutation_trap` is runtime-emitted. If — and
only if — Codex/Ocean accept the allocation, the standing multi-site change is
(decision 0028 checklist; indexes verified read-only at pinned `0e215d31`):

1. `diagnostic_causes!` entry — next free cause key **194** (max 193 at the pinned commit).
2. `diagnostic_code_allocations!` entry — next free key index **100** (max 99 = H0643 at
   the pinned commit); spelling `"H0810"`, title `"overlapping change arguments"`,
   family `OWNERSHIP_BORROWING`, owner `"ownership_borrowing"`, stage `"ownership_check"`.
   Range check: H0810 sits inside the family's `H0800–H0899` range (`DIAGNOSTIC_FAMILIES`,
   verified at the pinned commit).
3. `historical_public_ordinal` match arm for key 100 — current arms end at 99; without
   the arm the ordinal defaults to `u16::MAX` and validation fails `InvalidPublicOrdinal`.
4. `DIAGNOSTICS` detail entry — blame-style: name the call site, the overlapping
   argument positions and caller places, and the fix (pass disjoint caller places).
5. Hardcoded counts: `DIAGNOSTIC_CAUSES.len()`, `summary.active_codes`, `all().len()`,
   `catalog.len()`, `"Hum diagnostics (N codes)"`, `"\"count\": N"` in `diagnostics.rs`.
6. `docs/DIAGNOSTICS.md` mirror row.
7. `tools/check_all.ps1` pinned count + `tools/test_ci_policy.ps1` `$CompilerBodies` digest.

Until acceptance, none of the above is done; the map's §6 reuse table stands as the
complete diagnostic story.

## 9. Acceptance controls (standing repository requirements)

- **Honesty locks (decision 0014):** no output text or doc may claim more than the checker
  proves. Fixtures must observe the effect (the changed caller value, the fired
  contract/trap), not merely declare the form.
- **One pen at a time; branch + PR; Ocean merges.** Never push `main`; one task per branch.
- **Pre-push gates:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --bin hum`, `tools/test_ci_policy.ps1`, hand-run the exact commands behind any
  added/changed Session AB assertions, `tools/check_public_readiness.ps1`, text hygiene.
- **Draft until CI Full green**, then mark ready and report the run ID. Red on a draft is
  contained; nothing merges on red.
- **Codex review verdict** before ready (architect-reviewer role: independent probes,
  scope check, diagnostic quality, honesty locks).
- **Commit identity:** Ocean's GitHub noreply identity; no agent attribution anywhere.

## 10. Validation plan and platform limits

An implementation PR would need, at minimum:

- Fixtures through the **real user-task call path** (§4.2 two-phase loop), covering §3.1–§3.6:
  root/field writes, list-growth invalidation, no-op calls, write-then-restore, nested
  forwarding, and side effects during argument evaluation.
- Adversarial fixtures: change/change and change/borrow-arg overlap (H0810, if accepted);
  overlap with a live caller writable alias (H0808); element-place, non-place, and
  keyword/permission-mismatch rejections (plain traps per §6); `try` with `change` args
  stays unsupported.
- Per-OS Full CI. Windows exhaustive canonical-seal evidence is intentionally skipped per
  the standing Ocean rule — not a failure; only Ubuntu's is required.
- **Platform limits (not negotiable in the repair):** Session W preserved — no new
  try/catch surface (D6); element-place `change` args stay rejected, no `write_place`
  widening (D2); no blanket view ban — selective H0807 invalidation only (§4.6);
  forbidden overlap is rejected, never legalized by copy-back order.
- **Evidence labeling:** locally constructed fixtures are labeled as such; they are not
  independent execution evidence. Acceptance evidence is Codex's independent run through
  the production seam.

## 11. Stop boundary

- This document is **planning-only**. This delivery authorizes finishing the planning
  document; it does not authorize implementing its recommendations.
- D1, D1-sub, D2, D3, D4, D5, D6 remain **open** pending Ocean/Codex approval. H0810
  remains **proposed, not approved or allocated**.
- Any implementation needs a Work Order before branch work begins.
- Builder lane for this delivery: one documentation file, no source/tooling changes, no
  builds, no test campaigns, no CI dispatch, no Work Order activation, no merge, no `main`
  push. Delivery ends at the draft PR.
- §§7–11 above (from the tail-completion notice onward) are newly authored completion,
  not recovered historical text.
