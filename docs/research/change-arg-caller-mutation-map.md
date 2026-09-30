# Bounded repair map: caller-visible mutation through user-task `change` arguments

Date: 2026-09-29 (delegated-ruling reconciliation). Role: Builder (planning only).
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

Delegated-ruling reconciliation (2026-09-29, Codex): the live proposal
now specifies final-value transfer to each exact change argument place
with conservative call-access invalidation of overlapping caller views
(§4.5 authoritative). The W/G write-effect design of the previous
revision is superseded but preserved in git history — no history
rewrite. D1/D1-sub are ruled: transfer and invalidation precede
propagation of Returned, Failed, and ContractViolation at every frame.
Fatal Err traps bypass the boundary with no rollback guarantee. The
proposed test journal is withdrawn; tests observe actual caller state
through the real evaluator. No source implementation, builds, tests,
test campaign, or Work Order activation is authorized by this
reconciliation.

> **Counterexample fold-in — 2026-09-29 (Codex direction; builder
> lane). Not recovered text.** Claude's counterexamples are folded
> into the existing correction — no restart, no alternative-design
> exercise: (1) a change argument requires writable caller authority
> even for a no-op call, with shared static/runtime rejection and a
> designated diagnostic — `examples/probes/transaction_once.hum`'s
> `let txn` → `change txn` shape and its unchanged `ok` assertion
> are carried in the §7 inventory with no zero-compatibility-impact
> claim; (2) phase-2 validation and snapshots run after all
> argument evaluation, with controls preserving a nested call's
> completed update (`f(change r, g(change r.x))`) and rejecting
> `change r.x` combined with `consume r` / `consume r.z` so the
> transfer never resurrects a moved root — consume stays
> root-granular, and the new admission failures get designated
> diagnostics, never generic invariant traps; (3) consuming a
> Change/Borrow parameter rejects through shared static/runtime
> enforcement with a truthful designated diagnostic — mutation
> authority is not ownership-transfer authority, with nested
> forwarding covered by the same consume-branch guard, and no
> silent moves or copy-back of consumed resources; (4) Session W's
> public permission-bearing `try` stays unsupported while private
> tests may retain the caller Env and drive the real evaluator/call
> boundary, inspecting actual post-transfer state after
> `ContractViolation` or `Failed` — internal coverage labeled
> separately from publicly admitted CLI behavior, no journal, no
> synthetic replay. The affected inventory and diagnostic proposals
> are reconciled once (§§6–7, §15).

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

**Ruled (delegated ruling 2026-09-29):** an admitted `change` call
is treated as potentially modifying its argument place whether or
not the callee wrote. A no-op callee still performs the §4.5
transfer — the final value is copied to the exact argument place —
and the conservative call-access invalidation applies: a whole-root
arg invalidates descendant caller views; a field-place arg
preserves disjoint siblings. The earlier requirement ("zero
caller-visible effects for a no-op call") is superseded; the
conservative rule deliberately over-invalidates relative to
observed writes, and the invalidation reason is recorded truthfully
as call-access, not as an observed write. Writable caller authority
is required even for a no-op call (§4.2).

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
original caller place and authority. Invalidation is applied per frame by the §4.5 call-access rule —
whether or not that frame's callee wrote. No frame can observe
another frame's env; each boundary independently copies the callee's
final value into its own caller place and invalidates overlapping
caller views — no effect records cross frames.

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

### 3.7 What the transfer must not do

No silent loss (the final value reaches the exact argument place or
traps fail-closed — never dropped); no silent resurrection (a change
arg whose root was consumed during argument evaluation is rejected
at phase 2 with its designated diagnostic — the transfer never
writes to a moved root); no silent moves (consuming a Change/Borrow
parameter is rejected with a truthful designated diagnostic — never
a silent no-op, and the transfer never copies back a consumed
resource); no over-invalidation beyond the argument's place scope
(whole-root arg invalidates descendants, field-place arg preserves
disjoint siblings — §4.5; unrelated views stay valid); no blanket
view bans to simplify the mechanism (borrow views live through the
call and are invalidated per the conservative call-access rule via
the existing H0807 path, including no-op calls).

## 4. Repair design (replaces the withdrawn equivalence claim)

### 4.1 Mechanism, not inference

The implementation direction is an explicit transfer mechanism
through the real call owner: the user-task call site records per-`change`
argument (caller place, arg index, span); `execute_task` returns the
callee's final parameter values; the call site performs the §4.5
transfer — final-value copy to the exact argument place plus
conservative call-access invalidation. Final-value *differences* are
never inferred into effects.

### 4.2 Two-phase argument loop (eval_primary call branch)

- **Phase 0 — admission, before any arg evaluation.** Primary
  admission is static, in the `ownership_check` stage (§6): D5
  keyword/permission matrix (§4.7); change-place shape validation
  (root or single field only; element places -> D2 rejection;
  deeper or non-place expressions -> D4 rejection); overlap
  detection on syntactic caller places (§4.6). The call branch at
  3137–3142 retains only fail-fast arity/unknown-task checks and
  the overlap backstop (same diagnostic). Deterministic
  first-failure in arg order, per the §6 admission order.
- **Phase 1 — evaluate left-to-right** (current loop, minus change
  args): consume path unchanged (`read_consume_value` + `mark_moved`
  immediately); ordinary/`borrow` args via `eval_expr` unchanged;
  `change` args are recorded, not evaluated. Early return on
  `Failure`/`ContractViolation` unchanged.
- **Phase 2 — per change arg, in order, after all arg evals:**
  authority via the §6 admission checks on the caller env: unknown
  place, immutable place (writable authority is required **even for
  a no-op** change call — the conservative invalidation treats
  every admitted call as a potential write, so authority cannot be
  waived when the callee happens not to write), borrow-permission
  root, and a root consumed during argument evaluation (the
  transfer must never resurrect a moved root — §4.9 control). Each
  of these failures carries its §6 designated diagnostic, never a
  generic invariant trap; plus the existing iteration trap
  (`active_iteration_for` -> `iteration_mutation_trap`, H0806) for
  change args on actively iterated roots; then copy-in snapshot
  read (root or single-field read mirroring `eval_primary`'s read
  path — *not* `read_consume_value`, which is consume-specific).
  Snapshots read post-argument-evaluation state, so a completed
  nested call's update is preserved: in
  `f(change r, g(change r.x))`, g's boundary transfer to `r.x`
  already applied during phase 1, so f's copy-in of `r` includes
  g's completed update — the nested call is complete, not
  concurrent, and is not an overlap rejection. Phase 2 rejects
  `change r.x` combined with `consume r` (or `consume r.z`, which
  is root-granular) because r's root is marked moved during phase
  1; the rejection precedes any transfer.

### 4.3 Threading final parameter values

`execute_task` (2040) gains a second result: the callee's final
parameter values in param order (no effect records). The four call
sites: entry path (1837), expression-application (3047), direct
application (3077) destructure and ignore it; only the user-task call
site (3161) consumes it for the §4.5 transfer. Inside
`execute_task_body`, final values are captured from the callee env at
each exit point: `Return`/fallthrough (env in scope), `Fail` (env in
scope at 2126), `needs:`-`ContractViolation` (body never ran — no
transfer; outcomes empty), `ensures:`-`ContractViolation` inside
`finish_success` (transfer per the delegated ruling: completed
mutations survive postcondition failure).

### 4.4 No per-mutation history (delegated ruling 2026-09-29)

The W/G write-effect records are removed from the live proposal (the
design is superseded but preserved in git history). `RuntimeBinding`
gains nothing; `write_place` and `eval_list_append` are not
instrumented. The transfer consults only the callee param's final
value Vf, and invalidation follows the conservative call-access rule
(§4.5) — never a value diff, never a write log. Rationale: the value
outcomes are identical with or without history; the history existed
only to preserve view precision in the no-op and disjoint-sibling
cases, which the ruling trades for implementation and review
simplicity.

### 4.5 Transfer algorithm — authoritative (per change argument; delegated ruling 2026-09-29)

This section is the plan's **single** transfer specification. §12's
cases and §12.1's traces are walkthroughs applying it; no other
section defines transfer. It supersedes the W/G design of the
previous revision (preserved in git history). Let P be the caller
place (a root `r` or a single field `r.s`; deeper is impossible by
§2). Writable aliases cannot reach the call site —
H0809/`PassedToCall` at the caller's preflight (§4.6a); the existing
`resolve_writable_alias_place` guard (4738) is retained fail-closed,
not widened. Let the callee param's final root value be Vf.

**Value transfer** (what the caller's binding holds after the call):
copy the callee param's final value to the exact argument place —
`write_place(caller_env, P, Vf)` when P is a root; the bounded
depth-2 descent when P is `r.s`. No per-mutation history is
consulted; no value comparison is performed. The **append-only value
transfer** is explicit: `list_append` grows the callee's list in
place (4587) — the caller's snapshot predates that growth — so
without this write the append would be silently lost. Fields not
written by the callee are unchanged from copy-in, so the copy cannot
clobber caller-side concurrent state beyond the argument place.

**Effect transfer — conservative call-access invalidation.** An
admitted `change` call is treated as potentially modifying its
argument place. Invalidate caller views overlapping P:

- P is a whole root `r`: invalidate all descendant views — field
  views of `r.f` for any `f` and element views of `r` — **even when
  the callee never wrote** (no-op calls included; ruled).
- P is a field place `r.s`: invalidate views of exactly `r.s`;
  disjoint siblings (`r.t`) are preserved (ruled).
- Implementation: reuse `invalidate_field_views` /
  `invalidate_element_views_for_growth` where they apply, plus one
  new overlap sweep for the root-descendant case (the existing field
  helper is a no-op on bare roots, 5445–5449).

The invalidation reason is call-access — "the place was passed as
`change` to an admitted call" — recorded truthfully. It does **not**
expand the existing H0807 write-based causes, and no catalog code is
allocated for it. A later use of an invalidated view still fires
`stale_view_trap` (H0807) through the existing path.

**Forwarding** (every frame boundary): each boundary copies the
callee's final value into its own caller place and invalidates the
caller env's overlapping views independently — no effect records
cross frames. `outer(change r)` → `middle(change r)` →
`inner(change r)`: inner's boundary writes inner's final into
middle's `p` and invalidates middle's overlapping views; middle's
boundary writes middle's final into outer's place, and so on. Values
compose by construction; no frame observes another frame's env.

**Per-frame ordering (ruled):** after body execution, transfer and
invalidation precede propagation of `Returned`, `Failed`, and
`ContractViolation` at every frame — D1/D1-sub decided (see §5, §14).
The violation/failure still propagates uncaught afterward. Pre-body
rejection (D4/D5/overlap/authority): neither transfer nor
invalidation runs — the call never happened; prior
argument-evaluation effects stand (§3.6). A callee that traps via
`Err(String)` bypasses the boundary entirely (`?` at 3165 exits
before outcomes are consumed); no rollback of prior effects is
guaranteed.

Any shape mismatch traps fail-closed via `write_place`'s existing
errors. No recovery is invented.

### 4.6 Overlap rejection (exclusive mutation, precisely)

Forbidden overlap is **rejected**; left-to-right copy-back order is not
an alternative and is not offered. Admitted **statically** by the
`ownership_check` stage on the call's syntactic caller places
(§6); the runtime call site retains the check in phase 0 as a
fail-closed backstop emitting the same diagnostic. Writable aliases
cannot reach the call: the caller body's preflight rejects any call
mentioning an alias with H0809 (see §4.6a), so the overlap check
never sees alias names. Authority-before-overlap (§6): a change arg
that both violates authority and overlaps reports the authority
diagnostic.

- two `change` args with overlapping places (same place, or one a
  field-prefix of the other: `change r` vs `change r.s`) -> reject.
  New narrow code **H0810** "overlapping change arguments",
  family `ownership_borrowing` — H0808 ("writable alias overlap") is
  not reused here because its message/help is written for `let alias =
  change` declarations; conflating the constructs would mislead. Full
  allocation checklist applies (§8).
- `change` arg overlapping a **borrow-declaring** argument's place ->
  reject (H0810). Borrow-declaring = the explicit `borrow x` keyword
  **or** an ordinary argument to an implicit-Borrow (default)
  parameter: the param's permission declares read authority for the
  call duration either way (`f(change r.s, r.s)` with the second param
  default-Borrow declares the same read as `f(change r.s, borrow
  r.s)`). Allowing the ordinary-argument form while rejecting the
  keyword form would make exclusivity keyword-sensitive; the
  declarations conflict regardless of the value-copy implementation.
  Two borrow-declaring args on the same place with no `change` arg
  stay allowed (shared read, existing behavior).
- `change` arg overlapping a **live caller writable alias** ->
  reject by reusing **H0808**: the call creates a second live writer,
  which is exactly what H0808's "live overlapping access" rule
  forbids. No new code.
- `change` arg overlapping a live caller **borrow view** -> allowed;
  the view is invalidated by the conservative call-access rule
  (§4.5 — even for a no-op call), via the existing H0807 path on
  later use (§3.7 — no blanket view ban).

### 4.6a H0809 boundary — complete (accepted; not expanded)

H0809 (`UNSUPPORTED_WRITABLE_ALIAS`) is the writable-alias
preflight's rejection of alias forms outside Session V's
direct-field, straight-line, non-escaping slice. Its complete
accepted cause set (kind `Unsupported`, `src/writable_field_alias.rs`
at pinned `0e215d31`): `ShapeOutsideDirectFieldSlice` (163),
`RebindsItsOwner` (164), `BindingRebinding` (165),
`BindingInsideControlFlow` (166), `OwnerRebinding` (167),
`LifetimeCrossesControlFlow` (168), `UseInsideControlFlow` (169),
`Rebinding` (170), `Storage` (171), `UnsupportedUse` (172),
`PermissionWrapper` (173), `AliasToAliasBinding` (174),
`NestedOrElementUse` (175), `PassedToCall` (176),
`OutsideTaskBody` (177), `Escape` (117). H0808's overlap causes
(116, 158, 159, 160) are separate and unchanged.

For the change-argument plan this boundary means:

- **Alias passing:** `f(change a1)` where `a1` is a writable alias is
  rejected at the *caller's* body preflight —
  `preflight_writable_aliases` (run.rs:2135–2170) emits H0809 with
  cause `PassedToCall` (176) before the call site evaluates anything.
  The change-arg mechanism never receives alias names; no alias
  resolution is specified at the call site beyond the existing
  fail-closed `resolve_writable_alias_place` guard (4738), which is
  retained as-is, not widened.
- **Alias escape:** `return a1` / `fail a1` (`Escape`, 117),
  `save_in_store a1` (`Storage`, 171), nesting or rebinding the alias
  (`NestedOrElementUse` 175, `Rebinding` 170,
  `AliasToAliasBinding` 174) are rejected at the defining body's
  preflight. Aliases cannot escape into a callee frame.
- No change-argument case routes to H0809, and H0809's meaning is
  preserved exactly — the plan neither reuses it for call-shape
  errors nor narrows its accepted causes.

### 4.7 Permission matrix (D5 — complete; replaced 2026-09-29, Codex-findings audit)

Dimensions (source-anchored): argument keyword ∈ {ordinary, `borrow`,
`change`, `consume`} (the loop strips `borrow`/`change` at 5410 and
takes the move path on `consume` at 5404); parameter permission ∈
{Borrow, Change, Consume} (`ParamPermission`, ast.rs:575). **A param
with no explicit permission defaults to Borrow** (parser.rs:10338–
10350). "Implicit borrows" in this matrix means the two
implicit-borrow phenomena in the source: (a) live caller borrow views
created by `let v = borrow <place>` (run.rs:2728, via
`borrowed_view_source` at 5416 — Field views for single field places,
Element views for bare-root element places); (b) the default-Borrow
param permission. A call-site `borrow` argument does **not** create a
view — the keyword is stripped and the place is evaluated as a value
(3148).

Timing tags: **P0** = phase-0 static check, before any argument
evaluation, per argument in arg order (D5 → place-shape → overlap
against earlier args); **P2** = phase-2 post-evaluation writability
check (`ensure_can_set`, 4780) in change-arg order; **EVAL** =
existing evaluation behavior, unchanged by the repair.

| # | Argument | Param permission | Disposition | Timing | Mechanism / owner |
|---|---|---|---|---|---|
| 1 | ordinary `x` | Borrow (explicit or default) | pass value | EVAL | existing loop (3148) |
| 2 | ordinary `x` | Change | **reject** (D5) | P0 | proposed designated diagnostic, static admission (§6) |
| 3 | ordinary `x` | Consume | pass value, no move mark | EVAL | existing behavior preserved — move marking happens only on the `consume` keyword path (4826); out of this repair's scope |
| 4 | `borrow x` | Borrow | pass value (no view created) | EVAL | existing loop (3148) |
| 5 | `borrow x` | Change | **reject** (D5) | P0 | proposed designated diagnostic — a read grant cannot satisfy a write grant; static admission (§6) |
| 6 | `borrow x` | Consume | pass value, no move mark | EVAL | existing behavior preserved — the param consumes the value copy; no caller place is moved |
| 7 | `change x` | Change | the repair: §4.2 two-phase admission, copy-in, §4.5 transfer | P0 (shape/overlap) → P2 (authority) | §4.2, §4.5, §12 |
| 8 | `change x` | Borrow | **reject** (D5) | P0 | proposed designated diagnostic — write grant the callee cannot exercise; static admission (§6) |
| 9 | `change x` | Consume | **reject** (D5) | P0 | proposed designated diagnostic — transfer impossible; the value is moved; static admission (§6) |
| 10 | `consume x` | Borrow | mark moved, pass value | EVAL | existing (3143–3147) |
| 11 | `consume x` | Change | **reject** (D5) | P0 | proposed designated diagnostic — the place is moved before any transfer could run; static admission (§6) |
| 12 | `consume x` | Consume | mark moved, pass value | EVAL | existing (3143–3147) |
| 13 | `change x` where `x` (or an overlapping place) was consumed by an earlier argument | Change | reject, phase 2 designated diagnostic (never a generic invariant trap — §4.2, §6) | P2 | proposed `H0xxx` change argument on consumed/moved place |
| 14 | `change` arg overlapping a live caller borrow view (`let v = borrow r.s`, `let w = borrow xs[0]`) | Change | **allowed**; the view is invalidated by the conservative call-access rule (§4.5) — even when the callee never wrote | §4.5 transfer | existing H0807 path: `stale_view_trap` (4918) emits STALE_FIELD_VIEW diagnostic + trap on later *use* of the invalidated view; the invalidation reason is call-access, truthfully recorded, not an observed write; no blanket view ban (§3.7) |
| 15 | borrow-declaring arg overlapping a `change` arg's place: explicit `borrow x`, or ordinary `x` to an implicit-Borrow (default) param | Borrow (explicit or default) on the overlapping arg; Change on the change arg | **reject** | P0 | proposed H0810 (§4.6) |
| 16 | two borrow-declaring args on the same place, no `change` arg involved | Borrow | allowed (shared read) | — | existing behavior (§4.6) |

Zero blast radius for the new P0 rejections: every existing
fixture/example that calls a `change`-param task already uses the
keyword (`session_o_complete_item_field_place.hum:27`,
`session_t_wrong_swap_contract.hum:26`); entry tasks take CLI args,
not keywords (§4.8).

**H0809 is not in this matrix.** Its complete accepted boundary —
every `Unsupported` cause (163–177, 117), including alias passing
(`PassedToCall`, 176) and alias escape (`Escape`, 117) — is stated in
§4.6a. No change-argument case routes to H0809; its meaning is
preserved exactly.

**Authority/overlap precedence** (recommended, deterministic):
[Corrected 2026-09-29, Codex re-review — the earlier
"overlap-before-authority" order is withdrawn.] The §6 admission
order — shape (D4) → keyword/permission (D5) → authority →
overlap — holds identically on the static and runtime sides, and
the first failure wins: a change arg that is both
authority-violating and overlapping reports the authority
diagnostic, never the overlap one. The shared catalog precedence
specs (`authority_over_ownership_v0`, 2113;
`effect_failure_over_ownership_v0`, 2129) continue to govern
checker-emitted diagnostic suppression; the admission order governs
which diagnostic is produced. No new precedence spec beyond the
admission order is proposed.

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

### 4.9 Consume controls: mutation authority is not ownership-transfer authority

A `change` (or `borrow`) parameter grants mutation (or read)
authority — never the right to move the caller's place. Consuming
a Change or Borrow parameter must reject through shared
static/runtime enforcement with truthful designated diagnostics:

- **Static:** `ownership_check` gains a branch rejecting `consume`
  of a Change/Borrow parameter with a proposed designated
  diagnostic (code TBD at allocation; §8-style checklist). Existing
  coverage trace: `is_movable_root` (ownership_check.rs:2924)
  accepts immutable/mutable locals and `Consume` params only —
  Change/Borrow params are not movable roots today, so the new
  branch closes a silent gap rather than changing an accepted
  shape.
- **Runtime:** the call-arg consume branch (~3144) rejects when
  the root's binding permission is `Change`/`Borrow` with the same
  designated diagnostic, before `read_consume_value`/`mark_moved`.
  This covers direct `consume p` args and nested forwarding
  (`outer(change p) { inner(consume p) }`) identically — the nested
  call's arg loop is the same code path. Hazard closed:
  `mark_moved` (4826) matches `Local | Consume` only, so consuming
  a Change param today is a silent no-op (value cloned, nothing
  marked) — the plan replaces the silence with rejection.
- **No silent moves, no copy-back of consumed resources:** a
  rejected consume never moves the caller's place; the §4.5
  transfer applies to `change` args only and never runs for a
  consumed resource.
- **Consume stays root-granular:** `consume_argument_root` (5404)
  reduces `consume r.z` to root `r` via `place_root` — no
  field-move expansion is proposed; `change r.x` combined with
  `consume r.z` is therefore the same rejection as `change r.x` +
  `consume r` (§4.2 phase 2).

## 5. Settled rules / unsupported shapes / genuine policy choices

### Settled (from 0014/0016, existing behavior, this audit)

- `change` = bounded exclusive mutation authority, write-through at
  the boundary (0014; LANGUAGE_REFERENCE:391 already correct).
- No implicit rollback machinery exists: linear resources are the
  exactly-once protocol (0014 §5); the interpreter keeps no journal
  (0010's explicit state model). **Ruled (Codex, delegated
  2026-09-29):** completed mutations survive ordinary typed failure
  (D1) and postcondition failure (D1-sub) — transfer and invalidation
  precede propagation of `Returned`, `Failed`, and `ContractViolation`
  at every frame. `needs:`-violation (body never ran) still transfers
  nothing; fatal `Err(String)` traps bypass the boundary with no
  rollback guarantee.
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
  `change g(x)`): rejected — no caller place exists. Designated
  diagnostic, **proposed** allocation via static admission (D4,
  §6); the old "plain trap, no catalog allocation" justification
  is withdrawn (Codex re-review 2026-09-29).

### Genuine policy choices (need Ocean/Codex ruling)

- **D2:** element-place change args — reject at call site
  (recommended) vs extend `write_place` with element writes (wider
  than this repair).
- **D3:** overlap rejection code — new narrow **H0810** for
  change/change and change/borrow-declaring-arg overlap (recommended,
  checklist in §8) vs stretching H0808 (rejected: message/help would
  mislead). Live-writable-alias overlap reuses H0808 (settled, §4.6).
- **D4:** non-place change args — reject (required); designated
  diagnostic, **proposed** allocation via static admission (§6) —
  generic traps are not accepted for new rejections. No open point
  beyond ruling D4 itself.
- **D5:** keyword/permission mismatch rejection per §4.7 (recommended).
- **D6:** `try` with change args stays unsupported (preserve Session
  W).

## 6. Rejection plan: shared static/runtime admission (Codex re-review correction 2026-09-29; replaces the runtime-only premise)

[Corrected 2026-09-29, Codex re-review:] the earlier "runtime-only,
generic-trap" premise is withdrawn — it is not accepted. Every new
rejection gets a **designated diagnostic** (never a generic trap),
and admission is **shared** between the checking pipeline and the
runtime call site, by decidability:

- **Static admission** — the `ownership_check` stage
  (`src/ownership_check.rs`, the stage that owns the H08x family):
  everything decidable from the call expression plus the callee's
  declared signature — D4 argument shape, D5 keyword/permission
  match, and overlap on syntactic caller places.
- **Runtime admission** — the user-task call site in `src/run.rs`
  (the §2 argument owner): everything that depends on runtime env
  state — authority (mutable/moved/iteration/borrow-permission via
  the existing `ensure_can_set` family), live-alias/view overlap
  (H0808 — liveness is runtime state), the overlap backstop, and
  the §4.5 transfer itself.

**Admission order** (deterministic; one diagnostic per call — the
first failure wins): shape (D4) → keyword/permission (D5) →
authority → overlap. In particular **authority-before-overlap**
holds identically on both sides: the static side emits the
authority diagnostic (not overlap) when it can prove the authority
violation (e.g. `change` on a `let`-bound root); the runtime side
runs `ensure_can_set` before the overlap backstop. Combined causes
produce exactly one diagnostic — the earliest in this order — on
both sides.

Rejections are observed through one of two **existing** channels.
The repair's *mechanism* is run.rs-centered; its *designated
surface* is not: diagnostic identity (code, cause key, ordinal,
family), the human-projection mirror, and the pinned-count/policy
pins have designated owners named explicitly in §7. No AGENTS.md
exception is approved: those owners are identified, not touched, by
this planning delivery. No new consumers, and no catalog allocation
except proposals (H0810 and the D4/D5/D2 shapes below stay
**proposed, not approved or allocated**):

- **Trap channel** (existing traps — unknown place, arity, fatal
  paths): the interpreter returns `Err(String)` → `run_program`
  maps it to `RunOutcome::Trap(message)` (run.rs:898) → main.rs
  prints `runtime trap: {message}` on **stderr** and exits **2**
  (`run_outcome_exit_code`, main.rs:1639–1646; the test seam mirrors
  it at run.rs:742/767). Consumers: CLI users (stderr + exit 2);
  `tools/check_all.ps1` via `Read-NativeOutputWithExit` asserting exit
  code and output on `hum run fixtures/… --entry …`; `src/run.rs`
  internal tests via `run_program_with_adapters` (597) asserting
  `RunOutcome::Trap`.
- **Diagnostic+trap channel** (H0802/H0806/H0808 rows): the `*_trap`
  helper pushes a `Diagnostic::error` onto the interpreter's
  diagnostics — collected into `RunReport.diagnostics` (run.rs:899–
  904), surfaced by reporters, asserted by internal tests — **and**
  returns `Err("H0xxx <title>")`, which takes the trap channel above.
  Precedent: `borrow_mutation_trap` (4856),
  `iteration_mutation_trap` (4884), the H0808/H0809 emission in
  `preflight_writable_aliases` (2154–2170).

| Rejection | Admission owner | Channel | CLI-observable | Files touched (by intent) |
|---|---|---|---|---|
| non-place change arg (`change 5`, `change (a+b)`, `change g(x)`) — D4 | **Static:** `ownership_check` stage (`src/ownership_check.rs`); syntactic shape | diagnostic+trap, **proposed** (`H0xxx` non-place change argument; code TBD at allocation) | diagnostic + trap text, exit 2 | `src/ownership_check.rs` + — **iff accepted** — `src/diagnostic_catalog.rs` (§8-style checklist), `docs/DIAGNOSTICS.md`, `tools/check_all.ps1`, `tools/test_ci_policy.ps1` |
| D5 keyword/permission mismatch (§4.7 cells 2, 5, 8, 9, 11) | **Static:** `ownership_check` stage; call keyword × declared param permission | diagnostic+trap, **proposed** (`H0xxx` change-argument permission mismatch; code TBD at allocation) | diagnostic + trap text, exit 2 | `src/ownership_check.rs` + catalog checklist iff accepted (as above) |
| change/change or change/borrow-declaring-arg overlap | **Static primary:** `ownership_check` stage on syntactic caller places; **runtime backstop:** call-site branch, phase 0 (~3143), emitting the same diagnostic | diagnostic+trap, **H0810 proposed** | `H0810 overlapping change arguments` diagnostic + trap text, exit 2 | `src/ownership_check.rs` + `src/run.rs` (backstop) + — **iff accepted** — `src/diagnostic_catalog.rs` (§8 checklist), `docs/DIAGNOSTICS.md` (mirror row), `tools/check_all.ps1` (pinned count in `Invoke-HumCompilerFrontChecks`), `tools/test_ci_policy.ps1` (`$CompilerBodies` re-pin for that function) |
| change arg overlapping a live caller writable alias | **Runtime:** call-site branch, phase 0 (~3143); liveness is runtime state | diagnostic+trap, **H0808** (existing meaning: the call creates the second live writer H0808 forbids) | H0808 diagnostic + trap, exit 2 | `src/run.rs` only |
| change arg on `let`-bound (immutable) root | **Static:** `ownership_check` stage; the static side emits the authority diagnostic (not overlap) when it proves the violation (authority-before-overlap); **runtime:** phase 2 writability check (4780) with the same diagnostic | diagnostic+trap, **proposed** (`H0xxx` change argument on immutable place; code TBD at allocation) | diagnostic + trap text, exit 2 | `src/ownership_check.rs` + `src/run.rs` + — **iff accepted** — `src/diagnostic_catalog.rs` (§8-style checklist), `docs/DIAGNOSTICS.md` (mirror row), `tools/check_all.ps1` (fixture assertions) |
| change arg on borrow-permission root | **Runtime:** phase 2 `ensure_can_set` → `borrow_mutation_trap` (4856) | diagnostic+trap, **H0802** (same authority violation as an in-body write through a borrow) | H0802 diagnostic + `runtime trap: H0802 …`, exit 2 | `src/run.rs` only |
| change arg whose root is moved — before the call or consumed during argument evaluation (`change r.x` + `consume r`, or `consume r.z` which is root-granular) | **Static:** `ownership_check` stage; syntactic consume+change on the same root in one call → the designated diagnostic (authority-before-overlap); **runtime:** phase 2 moved check with the same diagnostic — the transfer never resurrects a moved root | diagnostic+trap, **proposed** (`H0xxx` change argument on consumed/moved place; code TBD at allocation) | diagnostic + trap text, exit 2 | `src/ownership_check.rs` + `src/run.rs` + — **iff accepted** — `src/diagnostic_catalog.rs` (§8-style checklist), `docs/DIAGNOSTICS.md` (mirror row), `tools/check_all.ps1` (fixture assertions) |
| consume of a Change/Borrow parameter (direct arg or nested forwarding `inner(consume p)` where `p` is outer's change param) | **Static:** `ownership_check` new branch (Change/Borrow params are not movable roots — `is_movable_root` 2924); **runtime:** call-arg consume branch (~3144) rejects Change/Borrow-permission bindings before `read_consume_value`/`mark_moved` — closes the current silent no-op (`mark_moved` 4826 matches `Local \| Consume` only) | diagnostic+trap, **proposed** (`H0xxx` consuming a non-ownership parameter; code TBD at allocation) | diagnostic + trap text, exit 2 | `src/ownership_check.rs` + `src/run.rs` + — **iff accepted** — `src/diagnostic_catalog.rs` (§8-style checklist), `docs/DIAGNOSTICS.md` (mirror row), `tools/check_all.ps1` (fixture assertions) |
| change arg on actively iterated root | **Runtime:** phase 2 `active_iteration_for` (4661) → `iteration_mutation_trap` (4884) | diagnostic+trap, **H0806** (same structural-mutation-during-iteration conflict) | H0806 diagnostic + trap, exit 2 | `src/run.rs` only |
| element-place change arg (`change xs[0]`) — D2 | **Static:** `ownership_check` stage (D2 recommended; syntactic shape) | diagnostic+trap, **proposed** (`H0xxx` element-place change argument; code TBD at allocation) | diagnostic + trap text, exit 2 | `src/ownership_check.rs` + catalog checklist iff accepted (as above) |

Why designated diagnostics, not generic traps, for the new
rejections: the repository's fail-closed probe requires every
unsupported shape of a new form to produce its designated diagnostic
— a validated line that traps generically is a defect — and
stretching H0802/H0806/H0808/H0809 wording to cover call-shape
mismatches would silently expand their approved meanings, which is
forbidden. The D4/D5/D2 shapes, the immutable-place authority
failure, the moved/consumed-root change-arg failure, and the
consume-of-Change/Borrow-parameter failure therefore get proposed
designated diagnostics (codes TBD at allocation; H0810's §8
checklist is the allocation template, one checklist per code). The
borrow-permission-root change arg keeps H0802 explicitly: passing a
borrow-permission root as `change` requests write authority through
a borrow — the same authority violation `borrow_mutation_trap`
names, not an expansion. Precedence among the catalog rows is the §6
admission order (shape → keyword/permission → authority →
overlap), applied identically on the static and runtime sides; the
existing `DIAGNOSTIC_PRECEDENCE` specs
(`authority_over_ownership_v0`, `effect_failure_over_ownership_v0`)
continue to govern checker-emitted suppression. If H0810 is
accepted, the §8 checklist gains one item: decide whether cause key
194 joins `OWNERSHIP_CAUSES` so those existing specs cover it — no
new precedence spec is proposed beyond the admission order.

H0810 ("overlapping change arguments", family `ownership_borrowing`,
owning stage `ownership_check` — runtime emission has precedent:
H0802's `borrow_mutation_trap` is runtime-emitted) remains
**proposed, not approved or allocated**, as are the three new
designated diagnostics above; no existing code names any of these
constructs. They all stay **proposed, not approved or allocated**;
D1/D1-sub are ruled; D2–D6 stay open.

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

## 7. Affected-file inventory — single, reconciled (only if the repair is approved)

This is the plan's one inventory; §15 keeps no second copy. The
mechanism lives in `src/run.rs`; diagnostic identity, mirrors, and
pins have designated owners named here. No AGENTS.md exception is
approved — those owners are identified, not touched, by this
planning delivery.

| File | Change | Why |
|---|---|---|
| `src/run.rs` | Two-phase argument loop at the user-task call site (§4.2, at 3143–3160); thread final parameter values through `execute_task` (2040) / `execute_task_body` (2072); the §4.5 final-value transfer + call-access invalidation at the call site through the real call owner (every exit, per the D1/D1-sub ruling); runtime admission — authority via `ensure_can_set` family, live-alias overlap (H0808), overlap backstop (§6); the new overlap sweep for root-descendant invalidation (§4.5); no per-mutation instrumentation | The defect and the runtime mechanism owner live here (§2) |
| `src/diagnostic_catalog.rs` | **Only if a proposed diagnostic is accepted:** §8-style checklist per accepted code — `diagnostic_causes!` entry (next free cause key), `diagnostic_code_allocations!` entry (next free allocation key), `historical_public_ordinal` arm, `DIAGNOSTICS` detail entry, the exact count literals in §8 item 5 — plus the open ruling whether H0810's cause key 194 joins `OWNERSHIP_CAUSES` | Designated diagnostic-identity owner |
| `src/ownership_check.rs` | **Only if the repair is approved:** static admission of D4/D5/D2 shapes, syntactic overlap, the authority branches (immutable-place change arg, syntactic consume+change on one root), and the consume-of-Change/Borrow-parameter branch (§6) — new branches in the `ownership_check` stage emitting the proposed designated diagnostics; authority-before-overlap deferral where the stage proves an authority violation | Proposed static admission owner (by intent); the stage that owns the H08x family |
| `src/diagnostics.rs` | **No hand edit proposed.** `"Hum diagnostics (N codes)"` and `"\"count\": N"` derive from `diagnostic_catalog::all()` via `format!`; only the *test* literals in §8 item 5 change, and only if the catalog changes | `hum diagnostics` contract; affected consumer |
| `docs/DIAGNOSTICS.md` | Mirror row, **only if H0810 is accepted** (test-enforced via `validate_human_projection`; catalog mirror, not prose) | Standing catalog-mirror rule |
| `tools/check_all.ps1` | Pinned count 100→101 inside `Invoke-HumCompilerFrontChecks` (**only if the catalog changes**); new `Read-NativeOutput[WithExit]` assertions for the new fixtures; the `transaction_once` `ok` assertion (4196–4197) must keep passing unchanged after the example's implementation-time migration | Standing pin practice; acceptance assertions live here |
| `tools/test_ci_policy.ps1` | `$CompilerBodies` digest re-pin for `Invoke-HumCompilerFrontChecks` (**only if that function's body changes** — i.e. only if the pinned count above changes; a `src/run.rs`-only repair does **not** trip it). The digest pins the body bytes of four shared functions (`Invoke-HumCompilerFrontChecks`, `Invoke-HumCompilerCorpusChecks`, `Invoke-HumUseAfterMoveRuntimeCheck`, `Invoke-HumUseAfterMoveProjectionCheck`); the failure message names the function and prints the new digest | Standing pin practice (decision 0028) |
| `fixtures/` | New `.hum` fixtures + session tests exercised **through the real user-task call path** — the §12 traces, the §6 rejection table, the §14 exit rows — placed by family (`fixtures/ownership_check/`, `fixtures/run/`). Counterexample fixtures: `f(change r, g(change r.x))` nested-call preservation; `change r.x` + `consume r` / `consume r.z` rejection (no resurrection); `consume` of a Change/Borrow param (direct + nested forwarding) rejection; immutable-place `change` rejection | Positive-evidence rule: fixtures must observe the effect, not merely declare the form |
| `examples/probes/transaction_once.hum` | **Not edited during planning.** Carried in the inventory as the writable-authority compatibility witness: `let txn` (immutable, line 66) passed as `change txn` (lines 68, 73) is the shape the item-1 authority rule newly rejects — even though `record_debit`/`record_credit` never write. The implementation-time migration is `let txn` → `let mut txn`; the `hum run … --entry transfer --args 10` → `ok` assertion (`tools/check_all.ps1`:4196–4197) must keep passing unchanged. **No zero-compatibility-impact claim is made** — the example as written today exercises the newly-rejected shape, and the migration is verified at implementation time, not assumed here | Compatibility witness for the authority rule |
| `src/run.rs` `mod tests` (5716) | Internal tests through `run_program_with_adapters` asserting the §14 observation points (post-call reads, stale-view trap-or-success) through the real evaluator — no journal | §14 seam distinction |

Out of scope for the repair: every other `src/` file, the CLI surface (no new flags), Work Order edits, and any try/catch machinery (D6 / Session W preserved). The parser already admits `change`-argument syntax — no syntax or admission change is proposed anywhere.

## 8. H0810 allocation checklist (proposed — not approved, not allocated)

H0810 ("overlapping change arguments", family `ownership_borrowing`, owning stage
`ownership_check`) is the only proposed allocation, and only because no existing code
names the change/change and change/borrow-declaring-argument overlap construct (§6). Runtime
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
5. Count literals — exact at pinned `0e215d31` (each N → N+1 when one
   code is added): `src/diagnostic_catalog.rs` tests —
   `assert_eq!(summary.active_codes, 97)` (two sites),
   `assert_eq!(DIAGNOSTIC_CAUSES.len(), 193)`,
   `assert_eq!(all().len(), 100)`; `src/diagnostics.rs` tests —
   `assert_eq!(catalog.len(), 100)`,
   `assert!(text.starts_with("Hum diagnostics (100 codes)\n"))`,
   `assert!(json.contains("\"count\": 100"))`. The production code in
   `src/diagnostics.rs` derives both strings via `format!` — no
   literal edits there.
6. `docs/DIAGNOSTICS.md` mirror row.
7. `tools/check_all.ps1` pinned count (the `100` inside
   `Invoke-HumCompilerFrontChecks`'s canonical-catalog assertion) +
   `tools/test_ci_policy.ps1` `$CompilerBodies` digest re-pin for
   that function (the digest pins that function's body bytes; the
   failure message names the function and prints the new digest).

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
- **Two test seams** (§14): actual-command tests (`hum run fixtures/…`, stdout + exit
  code) prove the observable contract; internal tests
  (`run_program_with_adapters`) prove the mechanism through the §14
  observation points (post-call reads, stale-view trap-or-success)
  through the real evaluator — no journal. Both execute the real
  interpreter; neither invents evidence. `RunReport` does not expose
  the caller `Env`.
- Adversarial fixtures: change/change and change/borrow-declaring-arg overlap (H0810, if accepted);
  overlap with a live caller writable alias (H0808); element-place, non-place, and
  keyword/permission-mismatch rejections (proposed designated diagnostics per §6); `try` with `change` args
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

> **Codex-findings correction pass — newly authored 2026-09-29 (builder lane). Not recovered text.**
> Addresses Codex's five findings on this map through a source-backed satisfiability
> audit at pinned `0e215d31` (`src/` identical at branch head `7bbc885c`; all line
> numbers below re-verified read-only in the worktree at that head):
> (1) §12 makes value propagation and effect propagation independently complete
> across six cases, and corrects the §4.5 root-replacement invalidation gap the audit
> found; (2) §4.7 is replaced with the complete permission matrix — every cell
> involving a change parameter or argument, implicit borrows, H0809 preserved,
> authority/overlap precedence, and argument-evaluation timing; (3) §6 is replaced
> with a repository-compliant rejection plan naming actual producers, channels,
> consumers, shared precedence, and required files — H0810 stays proposed, no
> approved meaning is expanded; (4) §14 covers every task exit including propagated
> contract failure, naming the observable result and real owner per acceptance case,
> and distinguishes actual-command tests from internal caller-env tests — Session W
> preserved, no catch/recovery syntax invented; (5) §15 reconciles the complete
> inventory including `src/diagnostics.rs` and affected harnesses/tests, separated
> into accepted facts, recommendations, and open rulings. Planning-only; D1–D6 and
> H0810 stay open/proposed. No implementation, builds, tests, or CI were run.

> **Satisfiability-audit reconciliation — newly authored 2026-09-29 (builder lane). Not recovered text.**
> Replaces contradictory sections with one consistent proposed plan (no new layer):
> (1) the run.rs-only premise is dropped where designated diagnostics, mirrors, and
> pins genuinely require other owners — `src/diagnostic_catalog.rs`,
> `docs/DIAGNOSTICS.md`, `tools/check_all.ps1`, `tools/test_ci_policy.ps1` are named
> in the single §7 inventory (no AGENTS.md exception approved); (2) §12.1 traces
> concrete caller/callee env states through the mechanism at every frame boundary —
> append-only and field-write/root-replacement with another forwarding return —
> showing value and effect as two applications of the same write records;
> (3) implicit-Borrow overlap is covered (§4.6, §4.7 cell 15) and the complete
> accepted H0809 boundary — all `Unsupported` causes incl. `PassedToCall` (176) and
> `Escape` (117) — is stated in §4.6a, with the corrected explicit cells retained;
> (4) the real test observation point is the caller frame's own subsequent behavior
> through the actual call owner (`RunReport` exposes no caller `Env`); per-frame
> write-back on the failure paths — incl. propagated contract failure — is covered
> via explicitly proposed `#[cfg(test)]` write-back journal plumbing;
> (5) the §8 count literals are corrected to the exact assertions at the pinned
> commit, `$CompilerBodies` digest ownership is stated exactly (four pinned
> `check_all.ps1` function bodies; re-pin only if a body changes), and the
> contradictory §7/§15 inventories are replaced by the single §7 inventory.
> D1–D6 and H0810 stay open/proposed. Planning-only.

## 12. Value/effect propagation independence (the six cases)

For each case, the **value outcome** (what the caller's binding holds
after the call) and the **effect outcome** (which caller views are
invalidated — hence which later uses fire H0807 via `stale_view_trap`,
4918) are stated independently. The value outcome is the callee
param's final value copied to the exact argument place (§4.5); the
effect outcome follows the conservative call-access rule — an
admitted change call invalidates overlapping caller views whether or
not the callee wrote — never a value diff, never a write log.
Invalidation vocabulary (source): `invalidate_field_views` marks
Field views whose `source_place` exactly equals the written
single-field place (5445–5460; no-op on bare roots);
`invalidate_element_views_for_growth` marks Element views whose
place-root equals the grown list root (5463–5475); the new overlap
sweep (§4.5) covers root-descendant views. Views are snapshot bindings
created by `let v = borrow <place>` (2728).

Notation: P = caller place after `resolve_writable_alias_place`
(a root `r` or a single field `r.s`); Vf = the callee param's final
root value.

1. **Append-only** (`change xs`; callee only runs
   `list_append(change xs, v)`): value — caller `xs` is the grown
   list **because the transfer writes it**: the append mutates the
   callee's root binding `List` in place (4587) — the caller's
   snapshot predates that growth — so the final-value transfer runs
   `write_place(caller_env, P, Vf)` with Vf = the grown list. Without
   that write the append would be silently lost. Effect — when P is
   a bare root, the overlap sweep + `invalidate_element_views_for_growth(caller_env, P)`:
   caller Element views of `xs` (snapshots from `borrow xs[0]`) are
   marked ListAppend-invalidated; a later use fires H0807. When P is a
   field place `r.s` holding the list: value write only, no element
   invalidation (no Element view can name `r.s[0]` — §3.2).

2. **Field-write followed by root replacement** (callee runs
   `set p.x = 1` then `set p = <record>`; Vf = the final record):
   value — caller P holds Vf via `write_place(caller_env, P, Vf)`;
   the root replacement subsumes the field write. Effect — P is a
   whole root, so the overlap sweep invalidates all descendant
   views, including caller Field views of `P.x`: the field write
   really ran inside the callee, and the conservative rule covers it
   without needing a per-field record. The value outcome does not
   determine the effect outcome.

3. **No-op** (callee never writes): value — `write_place` copies
   the equal final value (real but harmless). Effect — overlapping
   views are still invalidated (delegated ruling). This is the
   deliberate precision trade of the conservative rule: a no-op
   change call costs the caller its overlapping views.

4. **Write-restore** (callee writes then restores the original
   value; Vf == initial): value — `write_place` runs with Vf. Effect
   — full invalidation per the conservative rule, exactly as if the
   value had changed. Final==initial never suppresses effects.
   Write-restore and no-op are now intentionally indistinguishable.

5. **Disjoint fields, whole-root arg** (`change r`; callee writes
   `r.x` and `r.y`): value — the full final value is copied to `r`;
   fields the callee did not write are unchanged from copy-in, so no
   caller-side concurrent state is clobbered. Effect — the whole-root
   rule invalidates all descendant views: views of the written
   fields die, **and views of unwritten siblings (e.g. a snapshot
   from `borrow r.z`) die too** (ruled precision loss).

5b. **Field-place arg** (`change r.s`): value — Vf copied to
   `r.s`. Effect — only views of exactly `r.s` are invalidated;
   disjoint siblings (`r.t`) are preserved (ruled).

6. **Nested forwarding** (`outer(change r)` →
   `middle(change r)` → `inner(change r)`; inner writes `q.x`):
   value — each boundary copies its callee's final value into its
   own caller place (§3.5, §4.5); values compose frame by frame with
   no frame observing another frame's env. Effect — invalidation is
   applied per frame by the call-access rule: the middle frame's
   boundary marks outer views overlapping `p`; the outer frame's
   boundary marks main views overlapping `r`. A write deep in the
   chain invalidates the original caller's overlapping views
   because every boundary invalidates — no history records are
   needed.

### 12.1 Frame-boundary traces — concrete states through the mechanism

Planned-mechanism walkthroughs (not execution evidence): the exact
caller/callee env states at each frame boundary, showing value and
effect transfer as two separate applications of the §4.5 transfer
(final-value copy + call-access invalidation — no write records
exist under the delegated ruling). Notation: env entries show binding → value; views show
`invalidated_by`.

**Trace A — append-only call** (`list_append`; no field writes,
no root replacement):

```hum
task append_one(change xs) {
  list_append(change xs, 2)
}
task main() {
  let mut xs = [1]
  let e = borrow xs[0]
  append_one(change xs)
}
```

- t0 (main env): `xs → [1]` (mutable let); `e →
  view{source_place: "xs[0]", kind: Element, invalidated_by: None}`.
- Admission: shape ok (root `xs`); keyword/permission ok; authority
  ok (mutable); no overlap (§6 order).
- Inward boundary (copy-in): callee env `xs → param(change, [1])`
  — value only. No views cross inward; caller env untouched.
- Callee body: `list_append(change xs, 2)` → in-place `push` on the
  callee's list (4587); no write records exist under the delegated
  ruling; callee-local `invalidate_element_views_for_growth` finds
  no views. Callee env: `xs → [1, 2]` = Vf.
- Outward boundary (§4.5, P = `xs`): final-value transfer →
  `write_place(main, "xs", [1, 2])` → main `xs → [1, 2]`
  (**value transfer** — the append-only write; without it the
  caller's `[1]` snapshot would survive and the append would be
  silently lost); overlap sweep +
  `invalidate_element_views_for_growth(main, "xs")` → `e.invalidated_by
  = Some(ListAppend)` (**effect transfer** — call-access
  invalidation, reasoned truthfully as call-access, not as an
  observed write).
- t1 (main env): `xs → [1, 2]`; `e` invalidated. A later read of `e`
  → `stale_view_trap` (H0807) → runtime trap.

Why this trace matters: it is the case the explicit final-value
transfer exists for. A transfer specified only for root/field writes
would transfer nothing here — the caller's list would keep `[1]`
while the callee grew `[1, 2]`. The value transfer is explicit, not
inferred.

**Trace B — field-write/root-replacement, then another forwarding
return** (the audit's exact case):

```hum
task inner(change q) {
  set q.count = q.count + 1
  set q = { count: 99 }
}
task outer(change p) { inner(change p) }
task main() {
  let mut r = { count: 0 }
  let v = borrow r.count
  outer(change r)
}
```

- t0 (main env): `r → {count: 0}`; `v → view{r.count, valid}`.
- main→outer inward: outer env `p → {count: 0}`.
- outer body calls `inner(change p)`: phase 0 ok; inward: inner env
  `q → {count: 0}`.
- inner body: `set q.count = q.count + 1` → `q → {count: 1}`;
  then `set q = {count: 99}` → `q → {count: 99}` = Vf. No write
  records exist under the delegated ruling.
- inner→outer outward (first frame boundary), P = `p`, Vf =
  `{count: 99}`: `write_place(outer, "p", {count: 99})` → outer
  `p → {count: 99}` (value); the whole-root call-access rule
  invalidates outer's descendant views of `p` (none exist in
  outer's env). No records cross the boundary.
- outer returns with no further writes; outer's `p` is the final
  value `{count: 99}`.
- outer→main outward (second frame boundary), P = `r`:
  `write_place(main, "r", {count: 99})` → main `r → {count: 99}`
  (value); the whole-root call-access rule invalidates all
  descendant views of `r` → `v.invalidated_by = Some(CallAccess)`
  (effect — the field write is covered by the descendant rule, not
  by a per-field record).
- t1 (main env): `r → {count: 99}`; `v` invalidated; reading `v`
  traps (H0807).

Why the explicit final-value transfer matters here: at the
inner→outer boundary the final value of `q` is `{count: 99}` — a
comparison-based scheme might see "p changed" but cannot establish
the field-view effect; in the write-restore variant (inner writes
`q.count = 1` then back to `0` before the root replacement) the
final value equals the initial and a diff would transfer nothing at
all — while the explicit transfer still writes Vf and the
call-access rule still invalidates `v`.

## 13. Shared precedence, producers, and consumers (finding 3, continued)

Producers — shared static/runtime (§6): **static:**
`src/ownership_check.rs` (new admission branches for D4/D5/D2
shapes, syntactic overlap, the authority branches, and the
consume-of-Change/Borrow-parameter branch, emitting the proposed
designated diagnostics); **runtime:** the phase-0 call-shape checks
(~3143, new — overlap backstop only), the phase-2 authority checks
emitting the proposed designated diagnostics for immutable-place
and moved/consumed-root change args (diagnostic+trap, same channel
as H0802/H0806), the consume-branch guard (~3144) emitting the
proposed consuming-a-parameter diagnostic, `ensure_can_set` (4780)
with `borrow_mutation_trap` (4856) for borrow-permission roots
(H0802 reuse), and `active_iteration_for` (4661) with
`iteration_mutation_trap` (4884); `preflight_writable_aliases`
(2135) for the existing H0808/H0809 emissions. No new producer
module is created; the repair adds admission branches to the two
existing owners, not a new diagnostic subsystem.

Designated (non-`run.rs`) owners for the diagnostic surface are
named once in §7; the producer list below covers both sides.

Consumers: (a) the CLI — stderr `runtime trap: …`, exit 2 for traps
(main.rs:1528–1531, 1639–1646); diagnostics via `RunReport.diagnostics`
for the diagnostic+trap rows; (b) `hum diagnostics`
(`src/diagnostics.rs`: `diagnostics_text`/`diagnostics_json`, deriving
`"Hum diagnostics (N codes)"` and `"\"count\": N"` from
`diagnostic_catalog::all()` — its output changes iff the catalog
changes, and tests assert it); (c) `docs/DIAGNOSTICS.md`, the
test-enforced human mirror (iff H0810 is accepted); (d)
`tools/check_all.ps1` `Read-NativeOutput[WithExit]` assertions on
`hum run fixtures/…`; (e) `src/run.rs` internal tests asserting
`RunOutcome` and `report.diagnostics`.

Shared precedence: the §6 admission order — shape (D4) →
keyword/permission (D5) → authority → overlap — determines which
diagnostic is produced, identically on the static and runtime
sides (authority-before-overlap). `DIAGNOSTIC_PRECEDENCE`
(diagnostic_catalog.rs:2041) — e.g. `authority_over_ownership_v0`
(2113, AUTHORITY_CAUSES dominant over OWNERSHIP_CAUSES) and
`effect_failure_over_ownership_v0` (2129) — continues to govern
checker-emitted diagnostic suppression. The only allocation-time
precedence question is whether a future H0810 cause key 194 joins
`OWNERSHIP_CAUSES` (§6).

## 14. Task-exit coverage (every exit, observable result, real owner)

Exit vocabulary (source): `Flow::{Continue, Return{..},
Fail(FailureValue), ContractViolation}` (run.rs:306);
`TaskResult::{Returned(Value), Failed(FailureValue),
ContractViolation}` (1775). The entry seam maps
`Ok((TaskResult, is_app))` → `RunOutcome` (885–898): Returned →
`Success(display)` (exit 0; Trap if display fails); Failed →
`Failure(value.render())` (exit 1); ContractViolation →
`ContractViolation` (exit 1); `Err(DIAGNOSTIC_PREFLIGHT_REJECTED)`
→ `PreflightRejected` (exit 2); any other `Err(message)` →
`Trap(message)` → stderr `runtime trap: {message}`, exit 2
(main.rs:1528–1531). Diagnostics ride in `RunReport.diagnostics`
(899–904). Inside a call, `TaskResult` maps to `Evaluated`
(3166–3170): Returned → Value, Failed → Failure, ContractViolation
→ ContractViolation.

| Acceptance case | Callee exit | Caller-observable result | Real owner |
|---|---|---|---|
| success with writes | `Returned(v)` via `finish_success` (2294) | final-value transfer + call-access invalidation applied per §4.5; the call evaluates to v | call-site transfer (~3161) + `Evaluated::Value` (3166) |
| success, no writes | `Returned(v)` | value copy of the equal value + conservative call-access invalidation per §4.5 (no-op call rule); the call evaluates to v | §4.5 (unconditional) |
| typed `fail` after writes | `Failed(fv)` (2126; the linear-close check ran at the fail statement site, 2664–2665) | transfer + invalidation applied (D1 **ruled**, delegated 2026-09-29), then `Evaluated::Failure(fv)` propagates outward | §4.3 capture at 2126; per-frame call sites |
| `needs:` violation | `ContractViolation`, body never ran (2114–2116) | no transfer, no invalidation — the body never ran; nothing to preserve; `Evaluated::ContractViolation` propagates | 2114–2116 |
| `ensures:` violation after the body ran | `ContractViolation` from `finish_success` (2302) + ENSURES_CONTRACT_VIOLATION diagnostic (2406) | transfer + invalidation applied (D1-sub **ruled**, delegated 2026-09-29), then `Evaluated::ContractViolation` propagates carrying the diagnostic | diagnostic owner 2395–2412; per-frame call sites |
| **propagated contract failure** (the callee's own callee violated) | inner `ContractViolation` → outer `Evaluated::ContractViolation` (3168) → outer `Flow::ContractViolation` (2559/2574/2653/2732/2752) → outer `TaskResult::ContractViolation` (2131) | each frame's own transfer + invalidation was already applied at its call site before the violation propagated past it; the violation itself carries no value | per-frame call sites; propagation is by early return — it is never caught |
| trap inside the callee (`Err(String)`) | `result?` propagates (3165) | **no transfer, no invalidation**: the `?` exits before outcomes are consumed; the trap unwinds to the entry seam → `Trap`, exit 2. No rollback of prior effects is guaranteed | 3165, 898 |
| arity / unknown task at the call | `Err` before `execute_task` | the call never happens; no transfer, no invalidation; exit 2 | 3131–3141 |
| argument-evaluation failure / contract violation | early return (3153–3157) | the outer call never happens — pre-body rejection performs neither transfer nor invalidation; effects already produced (earlier consumes, completed nested-call transfers) stand — §3.6 | 3153–3157 |

Session W preserved: `try` accepts only
`let value = try named_call(...)` (or `… or fail …`) with ordinary
value arguments — `borrow`, `change`, `consume`, nested calls, and
operators remain unsupported (typed_failure.rs:932–940). No
catch/recovery syntax is invented: `Evaluated::Failure` inside `try`
uses the existing try machinery, and `ContractViolation` is **not**
catchable — it propagates as `Flow::ContractViolation`, never
converted to a value. Acceptance fixtures must not use `try` to
observe change-argument outcomes.

**Test-seam distinction and the real observation point** (required
by the audit; journal withdrawn by the delegated ruling
2026-09-29). `RunReport` exposes `outcome`, `diagnostics`, and
`authority_events` — it does **not** expose the caller `Env`, so no
CLI-level test can assert `invalidated_by` markers directly. The
real observation point is the caller frame's own subsequent
behavior, executed through the actual call owner (the call-site
transfer in `src/run.rs`):

- *Value transfer* is observed by the caller's own later read of the
  place: a fixture reads `r.count` after the call and the test
  asserts the value from `RunOutcome::Success` output.
- *Effect transfer* is observed by a pre-call borrow view plus a
  post-call use: `let v = borrow r.count` before the call, then read
  `v` after. If the write-back invalidated `v`, `read_value` fires
  `stale_view_trap` (H0807) → `RunOutcome::Trap`; if not, the read
  succeeds. The trap-or-success **is** the effect observation —
  through the actual call owner, with zero new plumbing.
- *Write-restore vs no-op*: under the conservative rule both
  invalidate overlapping views — the test asserts the trap fires in
  both cases, and post-call reads confirm the final value (no-op:
  the unchanged value copied back; write-restore: the restored
  value copied back). Effects are intentionally not distinguished
  by mutation history.
- *Field-write-then-root-replacement* (§4.5, §12 trace B): `v` must
  trap — the whole-root call-access rule invalidates all descendant
  views, so the field write is covered even though the final value
  arrived via root replacement.
- *Field-place disjoint siblings* (§4.5, §12 case 5b): `change r.s`
  must leave a view of `r.t` valid — the test asserts the read
  succeeds.

*Actual-command tests* (`hum run fixtures/… --entry …`, asserting
stdout + exit code via the `tools/check_all.ps1`
`Read-NativeOutput[WithExit]` pattern) prove the CLI-observable
contract above. *Internal tests* (`cargo test` via
`run_program_with_adapters`, 597) run the same fixture programs
through the real interpreter and assert `RunOutcome` +
`report.diagnostics`. Both seams execute the real call owner;
neither invents evidence.

**Propagated contract failure — explicit, tested through the real
evaluator.** [Revised 2026-09-29, delegated ruling.] Final parameter
values are captured at **every** task exit — `Returned`, `Failed`,
`ContractViolation` — and the §4.5 transfer runs before the outcome
propagates past the call site: on `ContractViolation` the violation
still propagates uncaught (never converted to a value; early-return
chain 3168 → 2559/2574/2653/2732/2752 → 2131), but each frame's
transfer precedes it. D1/D1-sub are ruled — the transfer applies on
`Failed` and `ContractViolation` exits.

The proposed `#[cfg(test)]` test journal is withdrawn. Tests observe
actual caller state through the real evaluator — no new plumbing, no
journal:

- *Internal tests* (`cargo test` via `run_program_with_adapters`,
  597) run the fixture programs through the real interpreter and
  assert the §14 observation points: post-call reads for values,
  trap-or-success for effects — through the actual call owner.
- *Unwinding paths* (typed `fail` after writes, `ensures:`-violation,
  propagated contract failure): fixtures are shaped so the
  post-transfer caller state is observable before the program
  unwinds — e.g. the inner callee writes, the transfer at its
  boundary is exercised through the real call-site function, and
  the test asserts the actual post-transfer caller values and
  invalidated view markers via the production code path. A fixture
  where the inner callee writes and then violates `ensures:` must
  leave both frames' transfers (inner→outer, outer→main) applied
  before the violation propagates past them — asserted through the
  real evaluator, not a replay log.
- *Caller-Env tests* (internal coverage, not public behavior):
  Session W's public permission-bearing `try` remains unsupported
  and no public CLI surface gains try/write-back. Private tests may
  retain the caller Env and invoke the real evaluator/call boundary
  directly, inspecting actual post-transfer caller state after
  `ContractViolation` or `Failed` — through the production code
  path, with no journal and no synthetic replay. This internal
  coverage is labeled separately from publicly admitted CLI
  behavior (only shapes admitted by the production pipeline are
  claimed as public execution).

Only shapes admitted by the production pipeline are claimed as
public execution.

## 15. Reconciled inventory (accepted facts / recommendations / open rulings)

**Accepted facts** (pinned source; need no ruling): the defect, all
owners, and the single-level place vocabulary (§§1–2); the delegated
transfer ruling (2026-09-29): final-value transfer to each exact
change argument place, conservative call-access invalidation of
overlapping caller views (whole-root arg invalidates descendants even
on no-op calls; field-place arg preserves disjoint siblings),
transfer + invalidation preceding propagation of Returned, Failed,
and ContractViolation at every frame; pre-body rejection performs
neither; fatal Err traps bypass with no rollback guarantee; the
invalidation reason is truthful call-access wording — no expansion of
the existing H0807 causes, no catalog code allocated (§§4.4–4.5, §14);
`eval_set` / `eval_list_append` / `write_place` / invalidation
semantics (§§3.1–3.2, §12); the exit-code and `RunOutcome` mapping
(§14 head); H0802/H0806/H0807/H0808/H0809 keep their approved
meanings, and the `*_trap` helpers emit diagnostic+trap (§6); the
Session W `try` restriction (typed_failure.rs:932); the
entry/callable-application exemptions (§4.8); default param
permission is Borrow (parser.rs:10338); call-site `borrow` args
create no views (3148) while `let`-bound `borrow <place>` creates
them (2728); the counterexample fold-in (2026-09-29, Codex):
writable caller authority is required for every `change` argument
even for a no-op call; phase-2 validation/snapshots run after all
argument evaluation — a completed nested call's update
(`f(change r, g(change r.x))`) is preserved and the nested call is
not concurrent overlap, while `change r.x` + `consume r`/`consume
r.z` is rejected before any transfer so a moved root is never
resurrected (consume stays root-granular — `consume_argument_root`
5404 reduces to the root); consuming a Change/Borrow parameter is
rejected — mutation authority is not ownership-transfer authority
(the current silent no-op: `mark_moved` 4826 matches
`Local | Consume` only); Session W's public permission-bearing
`try` stays unsupported while private caller-Env tests may drive
the real evaluator/call boundary, labeled separately from public
CLI behavior (§14).

**Recommendations** (argued by the map; need Ocean/Codex approval):
D2 (reject element-place change args via static admission), D4
(reject non-place change args with a proposed designated diagnostic
via static admission), D5 (the complete §4.7 matrix, admitted
statically with proposed designated diagnostics), D6 (Session W
preserved); D3 (overlap rejection, with H0810 proposed **only** for
change/change and change/borrow-declaring-argument overlap —
live-writable-alias overlap reuses H0808 as a settled reuse, not an
expansion); the shared static/runtime admission split with
authority-before-overlap (§6); the two-phase loop and the §4.5
transfer algorithm; the §4.9 consume controls (shared
static/runtime rejection of consuming a Change/Borrow parameter
with a truthful proposed designated diagnostic — mutation authority
is not ownership-transfer authority); the phase-2 controls (§4.2)
with designated diagnostics for the immutable-place, moved-root,
and consume-combination failures — never generic invariant traps;
the `transaction_once.hum` witness carried unedited in §7 with no
zero-compatibility-impact claim.

**Open rulings** (the map makes no recommendation; Ocean/Codex must
choose): whether H0810, the D4/D5/D2 designated diagnostics, and
the three new counterexample diagnostics (immutable-place change
argument, change argument on a moved/consumed root, consuming a
Change/Borrow parameter) are allocated (if yes: the §8-style
checklist per code, plus deciding whether H0810's cause key 194
joins `OWNERSHIP_CAUSES`); the §6 admission order
(shape → keyword/permission → authority → overlap) is recommended
but remains a ruling if Codex wants different determinism.

**File inventory:** single and reconciled in §7 — no second copy is
kept here (the earlier duplicate, which contradicted §7 on
`$CompilerBodies` digest ownership, is removed). Out of scope
(unchanged): every other `src/` file, the CLI surface (no new
flags), Work Order edits, and any try/catch machinery (D6 /
Session W preserved).
