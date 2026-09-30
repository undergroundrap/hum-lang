# Bounded repair map: caller-visible mutation through user-task `change` arguments

Date: 2026-09-29 (delegated-ruling reconciliation). Correction pass: 2026-09-30
(Codex's five findings reconciled — findings 1–5 per the authorization; §§4.5–4.9,
§6, §7, §8, §11, §13 rewritten or expanded). Role: Builder (planning only).
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

> **Partially superseded 2026-09-30:** point (3) now covers all three keyword
> forms — `consume p`, ordinary `p`, and `borrow p` into a `Consume` parameter
> (§4.9) — not only the `consume p` keyword form. The dated notice below is
> preserved verbatim as evidence.
>
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
`write_place` — no new write machinery. Invalidation uses the new §4.5
call-access overlap sweep (new `CallAccess` cause); the existing
`invalidate_field_views` / `invalidate_element_views_for_growth`
keep their exact write/growth behavior at real write sites and are
not called at the transfer site.

### 3.2 List-growth invalidation

`eval_list_append` (4550): requires `change list`; rejects mutation
during active iteration (`active_iteration_for` -> H0806); evaluates
the item; appends directly to the root binding; calls
`invalidate_element_views_for_growth`. Element views can only name
bare-root lists (`xs[0]`; `r.s[0]` is not a valid element place), so a
grown list field (`r.s`) has no element views to invalidate — the value
write alone suffices there. At the transfer site the write-back does
not call `invalidate_element_views_for_growth`: element views of a
whole-root change argument are invalidated by the §4.5 call-access
sweep with the `CallAccess` cause (truthful — the caller observes the
call, not the growth). Calling the growth helper with `place_root`
of a field place would over-invalidate sibling lists' views and is
forbidden.

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

- **Phase 0 — pre-evaluation checks only.** The call branch at
  3137–3142 retains fail-fast arity/unknown-task checks plus the
  pre-authority admission stages: D4 shape validation (root or
  single field only; element places -> D2 rejection; deeper or
  non-place expressions -> D4 rejection) and the D5
  keyword/permission matrix (§4.7). No authority check and no
  overlap check runs at phase 0: the previous revision's phase-0
  overlap backstop is withdrawn — a phase-0 overlap rejection
  cannot precede the authority decision the §6 order says wins.
  The static side (`ownership_check` stage) runs all four stages —
  shape → keyword/permission → authority → overlap — in one pass
  (§6). Deterministic first-failure in arg order, per the §6
  admission order.
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
  change args on actively iterated roots; then the phase-2 overlap
  sweep, per change arg in arg order: the syntactic overlap
  backstop (H0810 — the same diagnostic the static side emits)
  followed by the live writable-alias check (H0808).
  Authority-before-overlap is structural here: a change arg that
  fails authority never reaches the overlap sweep. A phase-2
  overlap rejection preserves phase-1 effects — consumes and
  completed nested-call transfers stand, no rollback — and it runs
  before any snapshot or transfer, so it never resurrects a moved
  root; then copy-in snapshot
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
parameter values in param order (no effect records).

**Prerequisite — declaration-time block restoration.** Each call
restores the callee's declaration-time block before the body runs:
`execute_task_body` (2072) builds a fresh `Env::new()` and binds
each parameter as `RuntimeBinding::parameter(value, permission,
definition_id)` (run.rs:372–384 — permission, definition_id,
linear, writable metadata). Reliable mutation transfer depends on
this restoration and on preserving, through body execution, all
four of: (1) **initializer effects** — body `let`/`change`
initializers evaluate with their effects; the transfer never
reorders or skips them; (2) **executed shadows** — nested blocks
save and restore shadowed bindings on every path (WO28 #16,
`eval_block` 2530–2548: if-block `let`/`change` bindings are
scoped like for-each binders), so a parameter shadowed inside a
nested block is the restored parameter binding at exit; (3)
**per-iteration scope** — for-each binders are block-scoped the
same way and never disturb parameter bindings; (4) **binding
metadata** — permission, `definition_id`, and `linear` survive to
exit. The transfer therefore identifies actual parameter bindings
**by metadata** (`definition_id.is_some()` — the existing
`binding_by_definition_id`, run.rs:4695), never by bare name, so
an executed shadow cannot be mistaken for the parameter.

The four call sites: entry path (1837), expression-application (3047), direct
application (3077) destructure and ignore it; only the user-task call
site (3161) consumes it for the §4.5 transfer. Inside
`execute_task_body`, final values are captured from the callee env at
each exit point: `Return`/fallthrough (env in scope), `Fail` (env in
scope at 2126), `needs:`-`ContractViolation` (body never ran — no
transfer; outcomes empty), `ensures:`-`ContractViolation` —
captured from the actual `env`, **not** from the postcondition
`exit_env` (§4.3a; transfer per the delegated ruling: completed
mutations survive postcondition failure).

### 4.3a Capture site: actual parameter bindings, not the postcondition environment

`finish_success` (2294) clones the body-exit `env` into `exit_env`
and inserts a synthetic `result` local
(`RuntimeBinding::local(value, false)`) before evaluating
`ensures:`; `capture_old_contract_values` (2306) additionally
inserts `old(...)` locals into the pre/postcondition environments.
The §4.5 transfer **must capture from the actual `env`** — the
parameter bindings as the body left them, identified by metadata
(§4.3) — never from `exit_env`: `exit_env`'s synthetic `result`
and `old(...)` locals are postcondition machinery, and a body
binding legitimately named `result` would be indistinguishable
there. Concretely, the parameter snapshot is taken from the `env`
passed to `finish_success` **before** the `exit_env` clone, so
the transfer reads actual parameter bindings on every path —
including `ensures:`-`ContractViolation`, where completed
mutations survive per the delegated ruling.

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
`write_place(caller_env, P, Vf)` for both root and single-field P.
`write_place` (4803) already resolves the exact single-field place
via `field_place::split_field_place` — no descent machinery is
proposed. No per-mutation history is
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
  views with `place_root(source_place) == r` for any field, and
  element views with `place_root(source_place) == r` — **even when
  the callee never wrote** (no-op calls included; ruled).
- P is a field place `r.s`: invalidate field views with
  `source_place == "r.s"` exactly; disjoint siblings (`r.t`) are
  preserved (ruled). Element views cannot name `r.s[0]` (§3.2), so
  no element selection is needed here.
- Implementation: one new overlap sweep at the transfer site. It
  stamps the new invalidation-reason variant
  `RuntimeViewInvalidationKind::CallAccess`
  (with the change-argument call span) on each overlapping caller
  view whose `invalidated_by` is still none — the selection above,
  covering both field and element views. The existing helpers
  (`invalidate_field_views`, 5445;
  `invalidate_element_views_for_growth`, 5463) are **not** called
  at the transfer site: their `FieldWrite` / `ListAppend` causes
  keep their exact write/growth meanings at real write sites,
  unchanged.

The invalidation reason is call-access — "the place was passed as
`change` to an admitted call" — recorded truthfully in the new
`RuntimeViewInvalidationKind::CallAccess` **variant**. The variant
is a runtime invalidation reason, not a registered diagnostic
cause and not a code allocation: **70 is H0807's code-allocation
key** (the `diagnostic_code_allocations!` row: 70, STALE_FIELD_VIEW,
"H0807", "stale view", family `ownership_borrowing`, stage
`ownership_check`) — not a cause key. H0807's registered causes
for the existing emissions are cause keys **115**
(`field_view_invalidated_by_exact_field_write_v0`) and **157**
(`element_view_invalidated_by_list_growth_v0`) — actual field
writes and list growth — and they keep those exact meanings: the
`FieldWrite`/`ListAppend` variants, helpers, and the existing
`stale_view_trap` arms are unchanged. No new cause key, no new
allocation key, and no new public H-code is allocated for
CallAccess. A later use of a CallAccess-invalidated view fires
`stale_view_trap` (H0807, 4914) through the existing path, with two
new match arms: `(Field, CallAccess)` — message `field view
{view_name} was used after {source_place} was passed as a change
argument`, help naming the borrow site and the change-argument
call site with the fix (re-borrow after the call or copy the value
before the call); `(Element, CallAccess)` — the element-view
analogue naming the list root. The projection needs no new
plumbing: `stale_view_trap` already receives `invalidation:
&RuntimeViewInvalidation` and matches `(view.kind,
invalidation.kind)` — the new arms read the invalidating call site
from the existing `invalidation.span` and the borrow site from
`view.bound_at`, rendered through the existing H0807
(STALE_FIELD_VIEW) diagnostic identity. Both arms attach related
sites via the existing `with_related_span` builder
(`src/diagnostic.rs`:244): the borrow site (`view.bound_at`) and
the invalidating call site (`invalidation.span`). Catalog
consumers: the H0807 **code row** (allocation key 70, title,
family, stage) is unchanged, and `hum diagnostics` output derives
from the catalog and is unchanged in shape; the H0807
**explanation/repair prose** — the `DIAGNOSTICS` detail entry and
the `docs/DIAGNOSTICS.md` mirror row (test-enforced via
`validate_human_projection`) — is updated by the builder at
implementation to document call-access invalidation as a third
invalidation reason. Cause-count and public-code-count implications
are separated in §8 item 5: CallAccess moves neither.

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
(§6); the runtime call site runs the backstop in **phase 2**, after the
phase-2 authority sweep, as a fail-closed backstop emitting the
same diagnostic (authority-before-overlap — §6). Writable aliases
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

Timing tags: **S** = static side (`ownership_check` stage) — shape →
keyword/permission → authority → overlap in one pass per call, first
failure wins (authority proven ⇒ overlap deferred); **R0** = runtime
phase 0, pre-evaluation (arity/unknown-task fail-fast + D4 shape + D5
keyword/permission — the pre-authority stages); **R2** = runtime phase
2, post-evaluation (authority sweep → overlap sweep → snapshots, per
change arg in arg order); **P1** = runtime phase 1,
argument-evaluation position (the consume-branch guard and the
ordinary/`borrow`-path guard run in arg-eval order); **EVAL** =
existing evaluation behavior, unchanged by the repair.

| # | Argument | Param permission | Disposition | Timing | Mechanism / owner |
|---|---|---|---|---|---|
| 1 | ordinary `x` | Borrow (explicit or default) | pass value | EVAL | existing loop (3148) |
| 2 | ordinary `x` | Change | **reject** (D5) | S + R0 | proposed designated diagnostic |
| 3a | ordinary `x`, caller binding an immutable/mutable local | Consume | pass value, no move mark | EVAL | existing behavior preserved — the param consumes a fresh copy; no caller place is moved; the caller owns the local, so no ownership is minted from another party |
| 3b | ordinary `p`, caller binding a Change/Borrow parameter | Consume | **reject** | S + P1 | proposed designated diagnostic — one diagnostic for all three keyword forms (§4.9): the caller cannot transfer ownership it does not hold; static branch + ordinary-path guard (3148) |
| 4 | `borrow x` | Borrow | pass value (no view created) | EVAL | existing loop (3148) |
| 5 | `borrow x` | Change | **reject** (D5) | S + R0 | proposed designated diagnostic — a read grant cannot satisfy a write grant |
| 6a | `borrow x`, caller binding a local | Consume | pass value, no move mark | EVAL | existing behavior preserved — as row 3a |
| 6b | `borrow p`, caller binding a Change/Borrow parameter | Consume | **reject** | S + P1 | same designated diagnostic as 3b; static branch + `borrow`-path guard (3148) |
| 7 | `change x` | Change | the repair: §4.2 two-phase admission, copy-in, §4.5 transfer | S (all four stages) + R0 (shape/keyword) → R2 (authority → overlap → snapshots) | §4.2, §4.5, §12 |
| 8 | `change x` | Borrow | **reject** (D5) | S + R0 | proposed designated diagnostic — write grant the callee cannot exercise |
| 9 | `change x` | Consume | **reject** (D5) | S + R0 | proposed designated diagnostic — transfer impossible; the value is moved |
| 10 | `consume x` | Borrow | mark moved, pass value | P1 | existing (3143–3147); the §4.9 guard rejects first when `x` is a Change/Borrow parameter |
| 11 | `consume x` | Change | **reject** (D5) | S + R0 | proposed designated diagnostic — the place is moved before any transfer could run |
| 12 | `consume x` | Consume | mark moved, pass value | P1 | existing (3143–3147); the §4.9 guard rejects first when `x` is a Change/Borrow parameter |
| 13 | `change x` where `x` (or an overlapping place) was consumed by an earlier argument | Change | reject, phase-2 designated diagnostic (never a generic invariant trap — §4.2, §6) | S + R2 | proposed `H0xxx` change argument on consumed/moved place (authority-before-overlap) |
| 14 | `change` arg overlapping a live caller borrow view (`let v = borrow r.s`, `let w = borrow xs[0]`) | Change | **allowed**; the view is invalidated by the conservative call-access rule (§4.5) — even when the callee never wrote | R2 transfer | existing H0807 path: `stale_view_trap` (4914) emits STALE_FIELD_VIEW diagnostic + trap on later *use* of the invalidated view; the invalidation reason is the new `CallAccess` variant, truthfully recorded, not an observed write; no blanket view ban (§3.7) |
| 15 | borrow-declaring arg overlapping a `change` arg's place: explicit `borrow x`, or ordinary `x` to an implicit-Borrow (default) param | Borrow (explicit or default) on the overlapping arg; Change on the change arg | **reject** | S + R2 | proposed H0810 (§4.6) |
| 16 | two borrow-declaring args on the same place, no `change` arg involved | Borrow | allowed (shared read) | — | existing behavior (§4.6) |

Zero blast radius for the new S/R0 rejections: every existing
fixture/example that calls a `change`-param task already uses the
keyword (`fixtures/run/session_o_complete_item_field_place.hum:27`,
`fixtures/run/session_t_wrong_swap_contract.hum:26`); entry tasks take
CLI args, not keywords (§4.8). The finding-2 guards (rows 3b/6b) touch
no existing fixture or example — checked at the pinned head, no task
forwards a Change/Borrow parameter into a `Consume` parameter in any
keyword form (§4.9).

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
authority — never the right to move the caller's place, and never
the right to mint ownership for a callee's `Consume` parameter.
Offering a Change/Borrow parameter for ownership transfer must
reject through shared static/runtime enforcement with one truthful
designated diagnostic, regardless of the call keyword:

- **Forms covered (one diagnostic):** `consume p` (keyword form),
  ordinary `p`, and `borrow p` — where `p` is the caller's
  Change/Borrow parameter and the callee declares the corresponding
  parameter `Consume`. The hazard is identical in all three: the
  caller holds only non-ownership authority, so the callee's
  `Consume` parameter would claim ownership the caller never had.
  No general copyability model is proposed — the rule is
  permission-based, not type-based: an immutable/mutable **local**
  passed as ordinary/`borrow` to a `Consume` parameter keeps
  existing behavior (matrix rows 3a/6a — the caller owns the local
  and the param consumes a fresh copy; no caller place is moved).
  A Consume-parameter source passed as an ordinary argument to a
  `Consume` parameter likewise keeps existing behavior — out of
  this repair's scope, stated explicitly (§5).
- **Static:** `ownership_check` gains a branch rejecting all three
  forms with the proposed designated diagnostic (code TBD at
  allocation; §8-style checklist). Existing coverage trace:
  `is_movable_root` (ownership_check.rs:2924) accepts
  immutable/mutable locals and `Consume` params only —
  Change/Borrow params are not movable roots today, so the new
  branch closes a silent gap rather than changing an accepted
  shape.
- **Runtime:** the call-arg consume branch (~3144) rejects when
  the root's binding permission is `Change`/`Borrow` with the
  designated diagnostic, before `read_consume_value`/`mark_moved`.
  This covers direct `consume p` args and nested forwarding
  (`outer(change p) { inner(consume p) }`) identically — the nested
  call's arg loop is the same code path. The ordinary/`borrow`
  argument path (3148) gains the same guard: when the target
  parameter permission is `Consume` and the argument is a caller
  place whose root binding permission is `Change`/`Borrow`, reject
  with the same diagnostic before evaluation — covering
  `outer(change p) { inner(p) }` and `outer(borrow p) {
  inner(borrow p) }` forwarding. Hazard closed: `mark_moved`
  (4826) matches `Local | Consume` only, so consuming a Change
  param today is a silent no-op (value cloned, nothing marked) —
  the plan replaces the silence with rejection.
- **No silent moves, no copy-back of consumed resources:** a
  rejected transfer never moves the caller's place; the §4.5
  transfer applies to `change` args only and never runs for a
  consumed resource.
- **Consume stays root-granular:** `consume_argument_root` (5404)
  reduces `consume r.z` to root `r` via `place_root` — no
  field-move expansion is proposed; `change r.x` combined with
  `consume r.z` is therefore the same rejection as `change r.x` +
  `consume r` (§4.2 phase 2).
- **Compatibility:** checked at the pinned head (2026-09-30) — no
  fixture or example forwards a Change/Borrow parameter into a
  `Consume` parameter in any of the three keyword forms;
  `transaction_once.hum`'s `rollback(consume txn)` /
  `commit(consume txn)` take `txn` from a `let` local, not a
  parameter — unaffected. No zero-compatibility-impact claim beyond
  this checked inventory.

### 4.9a Ownership-laundering closure (narrow; Claude finding 2026-09-30)

**Laundering** = a Change/Borrow parameter's value reaching a
`Consume` parameter through an intermediate binding the §4.9 guards
do not see. The narrow closure is mapped here in full — direct,
one-hop local, and return paths:

- **L1 — direct** (`consume p`, `p`, `borrow p` where `p` is a
  Change/Borrow parameter): the true laundering path — the frame
  mints ownership it never had. Closed by §4.9 (rejection; today
  it is the silent no-op: `read_consume_value` clones at 4777
  while `mark_moved` skips non-`Local`/`Consume` bindings at
  4826).
- **L2 — one-hop unannotated local copy** (`let y = p`, then
  `sink(consume y)` / `sink(y)` / `sink(borrow y)`): the
  intermediate is an **unannotated** local, so neither the static
  §4.9 branch (which keys on declared parameter permissions) nor
  the runtime guard (which keys on binding permission) sees
  through it. Severed by value semantics: `eval_binding`
  (2698–2755) inserts `RuntimeBinding::local(value, linear=false)`
  for unannotated `let` — **non-linear** (`linear` is set only
  when the annotation names a linear resource type,
  `annotation.is_some_and(is_linear_resource_type)` at 2740) —
  and the value is a **deep copy** (`Value: Clone` over owned
  `BTreeMap`/`Vec`, run.rs:286–296; `read_value` clones at
  4692). The frame legitimately owns the copy; consuming `y`
  moves the copy (`mark_moved` marks `Local`), never the caller's
  place; the §4.5 transfer still reads `p`'s binding by metadata
  (§4.3). No aliasing path exists.
- **L2b — borrow-view local** (`let v = borrow p.x`, then
  `sink(consume v)`): `eval_binding` inserts a
  `RuntimeBinding::view` (permission `Local`, snapshot value +
  `source_place`; `borrowed_view_source` 5416) — the snapshot is a
  clone taken at view creation, and invalidation is logical
  (H0807), not memory aliasing. Consuming the view moves the
  snapshot; the source place is untouched.
- **L3 — return** (`return p`; the caller binds the returned value
  and consumes it): the returned value is a value, not a place —
  the caller's fresh local owns it, consuming it is ordinary, and
  the callee's exit transfer is independent of the return.

**Non-linear copying is preserved:** `let y = p` stays legal in
all positions — the repair targets the *consume* (L1), never the
copy. No taint tracking through locals is proposed: the closure is
narrow because value semantics sever L2/L2b/L3, not because the
checker follows them.

**Explicit ruling identification (as required): no additional
semantic ruling is needed.** The closure is fully determined:
L1 → §4.9 rejection (its proposed diagnostic joins the §4.9
allocation ruling); L2/L2b/L3 → severed by settled value
semantics (deep copies, snapshot views, owned return values —
run.rs:286–296, 2698–2755, 4692, 5416). This determination is
stated here explicitly so no reviewer must re-derive it; if
Ocean/Codex disagree, that disagreement is itself the ruling to
record.

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

[Corrected 2026-09-30, Codex re-review:] the earlier "runtime-only,
generic-trap" premise is withdrawn — it is not accepted. Every new
rejection gets a **designated diagnostic** (never a generic trap),
and admission is **shared** between the checking pipeline and the
runtime call site, by decidability. The producer→consumer map is
authoritative in §13; this section states the split:

- **Static admission** — one narrow shared admission producer with
  two static consumers. The producer is a pure function over the
  call expression, the callee's declared parameter permissions, the
  caller's declared parameter permissions, and caller binding
  mutability — all present in the parsed AST
  (`CanonicalExpressionKind::Call` with
  `Permission(ParamPermission)` argument wrappers, ast.rs:351;
  `permission: ParamPermission` on declared parameters, ast.rs:83;
  `Binding { mutable, .. }`, ast.rs). It decides everything
  provable without runtime state — D4 argument shape, D5
  keyword/permission match, provable authority violations
  (`change` on a `let`-bound root, syntactic consume+change on one
  root, and **Known Borrow**: a `change` argument whose root is
  statically known to be borrow-permissioned — a caller-declared
  `borrow` parameter or a `let`-bound borrow view — emitting H0802),
  the non-ownership→Consume controls (§4.9), and overlap on
  syntactic caller places — in a single pass, first failure wins
  (authority-before-overlap is structural: the producer stops at
  the first failing stage, and Known Borrow authority precedes
  overlap — a `change` on a known borrow root that also overlaps
  another change argument emits H0802, never H0810). The producer lives in a new narrow
  module `src/change_arg_admission.rs` and emits no diagnostics
  itself; it also owns the shared message/help builders, so both
  static consumers — and the runtime `*_trap` helpers where the
  text must be identical — cannot drift. (The earlier "no new
  producer module" premise is corrected here: the new module is a
  pure analyzer, not a diagnostic subsystem — emission stays in
  the two existing owners.) Consumer A is `ownership_check`
  (`src/ownership_check.rs`, the stage that owns the H08x family):
  per-call admission branches invoked from the ownership walk,
  emitting the proposed designated diagnostics into the existing
  occurrence set via `diagnostic_occurrence_set`
  (ownership_check.rs:822). Consumer B is the ordinary `hum check`
  pipeline: a narrow adapter at `check_stage_outcome`
  (full_type_check.rs:252) — the `hum check`-only entry — emitting
  through `CheckStageOutcome.diagnostics`, the same channel as the
  H0640/H0641 call probes. The adapter is deliberately **not**
  inside the shared `build_report_with` per-statement walk: that
  construction is shared with the effect-check pipeline via
  `with_full_type_for_effect` (744; consumed at
  effect_check.rs:440, 508), so check-only admission placed inside
  it would leak change-arg diagnostics into effect checking. The
  adapter runs after `build_report` returns: it walks the
  program's parsed body statements through the shared producer and
  appends the resulting diagnostics to the outcome.
  **The walk visits every user-task call expression** — call statements, binding initializers, return
  expressions, and nested argument positions — **regardless of
  which argument keywords the call carries**: D5 mismatches and
  the §4.9 non-ownership→Consume controls fire on calls with no
  `change` keyword at all (rows 2, 3b, 6b — e.g. `inner(p)`,
  `inner(borrow p)`, `inner(consume p)` where `p` is a caller
  Change/Borrow parameter into a Consume parameter).
- **Runtime admission** — the user-task call site in `src/run.rs`
  (the §2 argument owner): phase 0 runs the pre-authority stages
  fail-fast (arity/unknown-task, D4 shape, D5 keyword/permission);
  phase 2 runs authority (mutable/moved/iteration/
  borrow-permission — Known Borrow — via the existing
  `ensure_can_set` family) →
  overlap (the H0810 backstop, then the live-alias H0808 check —
  liveness is runtime state) → snapshots; then the §4.5 transfer
  itself.

**Admission order** (deterministic; one diagnostic per call — the
first failure wins): shape (D4) → keyword/permission (D5) →
authority (immutable-place, moved/consumed-root, **Known
Borrow**) → overlap. **Static deferral:** when the
`ownership_check` stage proves an authority violation for a call,
it emits the authority diagnostic and defers — does not emit — the
overlap diagnostic for that call. **Runtime timing:** phase 0 runs
only the pre-authority stages (all decidable before evaluation);
phase 2 runs authority → overlap → snapshots per change arg in arg
order. The previous revision's phase-0 overlap backstop is
withdrawn: a phase-0 overlap rejection cannot precede the authority
decision the order says wins. A phase-2 overlap rejection preserves
phase-1 effects (consumes and completed nested-call transfers
stand — no rollback) and runs before any snapshot or transfer, so
it never resurrects a moved root.

Rejections are observed through the existing surfaces — no new CLI
surface is proposed. The repair's *mechanism* is run.rs-centered;
its *designated surface* is not: diagnostic identity (code, cause
key, ordinal, family), the human-projection mirror, and the
pinned-count/policy pins have designated owners named explicitly in
§7. No AGENTS.md exception is approved: those owners are
identified, not touched, by this planning delivery. No catalog
allocation except proposals (H0810 and the D4/D5/D2/authority/
consume shapes below stay **proposed, not approved or
allocated**):

- **ownership-check report** (static side): `hum ownership-check`
  (main.rs:993–1025) prints the stage report via
  `ownership_check::ownership_check_text/json` wrapped in
  `callable_text_report`/`callable_json_report` (human and
  `--format json`), exit 1 on errors. `hum run --native` gates on
  `ownership_check::ownership_check_has_errors` in the preflight
  admission block (main.rs:1355–1380), exit 1 with the
  ownership-check text. New static diagnostics flow into the
  existing occurrence set via `diagnostic_occurrence_set`
  (ownership_check.rs:822) — no new plumbing.
- **trap channel** (runtime): the interpreter returns `Err(String)`
  → `run_program` maps it to `RunOutcome::Trap(message)`
  (run.rs:898) → main.rs prints `runtime trap: {message}` on
  **stderr** and exits **2** (`run_outcome_exit_code`,
  main.rs:1639–1646; the test seam mirrors it at run.rs:742/767).
  Consumers: CLI users (stderr + exit 2);
  `tools/check_all.ps1` via `Read-NativeOutputWithExit` asserting
  exit code and output on `hum run fixtures/… --entry …`;
  `src/run.rs` internal tests via `run_program_with_adapters`
  (597) asserting `RunOutcome::Trap`.
- **diagnostic+trap channel** (runtime; H0802/H0806/H0808 rows and
  the proposed designated diagnostics): the `*_trap` helper pushes
  a `Diagnostic::error` onto the interpreter's diagnostics —
  collected into `RunReport.diagnostics` (run.rs:899–904),
  surfaced by reporters, asserted by internal tests — **and**
  returns `Err("H0xxx <title>")`, which takes the trap channel
  above. Precedent: `borrow_mutation_trap` (4856),
  `iteration_mutation_trap` (4884), the H0808/H0809 emission in
  `preflight_writable_aliases` (2154–2170).

`hum check` does **not** run `ownership_check`: its pipeline
(main.rs:440–560) is parse → source_check → app_entry →
path_boundary → callable → capability_root → resolve → type_check
→ full_type_check (each gated on no earlier errors; the resolve,
type_check, and full_type_check check-stage diagnostics run only
for the `check` command — main.rs:496–530). The new static
admission reaches `hum check` through Consumer B, the narrow
adapter at `check_stage_outcome` (full_type_check.rs:252) — not
through `ownership_check`, and not inside the shared
`build_report_with` walk (see §6: that walk is shared with effect
checking via `with_full_type_for_effect`). Adapter mechanics (all
source-backed at the pinned commit): the adapter walks the
program's parsed body statements' canonical expressions in
pre-order exactly like the H0640/H0641 probes —
`call_shape_issue` (1997) and `call_argument_type_issue` (2253)
recurse through `canonical_child_expressions` across `Return`,
`Binding`, and `Other` statements, so calls inside bindings,
returns, and nested expressions are all visited;
`strip_permission_expression` (2560) already strips
`borrow`/`change`/`consume` argument keywords. **Probe
attribution:** the H0640/H0641 call probes were executed by
Claude; the walk mechanics above are source inspection at the
pinned commit, not a rerun of those probes.
Gating: the adapter runs the producer only for arity-clean calls
whose callee resolves builtin-first through `task_signatures` —
unknown callees are skipped (the resolver owns H0601) and arity
mismatches stay with the H0640 probe, mirroring both existing
walks. Caller parameter permissions and binding mutability come
from the parsed task declarations and `Binding { mutable, .. }`,
not from full_type_check's type-only `TaskSignature`. Entrypoint
and callable-application calls stay exempt (§4.8): they resolve
outside the user-task table the adapter consults. Diagnostic
handling: emissions travel the stage's ordinary channel,
`CheckStageOutcome.diagnostics` (full_type_check.rs:248–250
carries diagnostics only), exactly like H0640/H0641 — no new
plumbing, and no cross-consumer duplicates, because the two static
consumers are command-disjoint: `hum check` never runs
`ownership_check`, while the native preflight and explicit `hum
ownership-check` never run the check pipeline's `full_type_check`.
D3 reporting stays truthful: `check_stages` is pushed only for
stages that actually ran (main.rs:448–450, 529), and the adapter is
a sub-pass of the `full_type_check` stage — no new stage name is
added, so the `stages` list is unchanged. CLI controls: no new
flags. The adapter rides the existing `hum check` invocation and
decides only the statically-decidable subset; anything needing
runtime liveness (borrow-permission roots, moved/consumed marks,
live aliases, active iteration) stays runtime-only.

**Execution-path mapping** (the existing ownership gate is
native-only): `hum check` → static admission via the
full_type_check adapter only (nothing executes, so no runtime
admission applies). Ordinary interpreted `hum run` → no static
change-arg admission (it runs neither `ownership_check` nor the
check-only stages) plus the interpreter's phase-0/phase-2 runtime
admission in `src/run.rs`. `hum run --native` → static admission
via the `ownership_check` consumer inside the existing native
preflight gate (`native_admission_requested`, main.rs:1357–1380;
exit 1 with the stage text on failure); the interpreter's
phase-0/phase-2 checks do not execute on the compiled path —
preserving the admission semantics in the native backend is an
implementation-time verification item, not a claim made here.
Explicit `hum ownership-check` → the `ownership_check` consumer
only.

In the rejection table below, "Static side" means the shared
producer as consumed by **both** static consumers (the
`ownership_check` branches and the `hum check` adapter — same
admissions, same order, same designated diagnostics); "Files
touched" additionally implies `src/change_arg_admission.rs`
(producer) and `src/full_type_check.rs` (adapter) wherever a
static-side cell appears.

| Rejection | Static side | Runtime side | Designated diagnostic | Files touched (by intent) |
|---|---|---|---|---|
| non-place change arg (`change 5`, `change (a+b)`, `change g(x)`) — D4 | `ownership_check` syntactic shape → ownership-check report, exit 1 | phase-0 shape check (~3143) → diagnostic+trap, exit 2 | proposed `H0xxx` non-place change argument (code TBD at allocation) | `src/ownership_check.rs` + `src/run.rs` + catalog checklist iff accepted (`src/diagnostic_catalog.rs` §8-style, `docs/DIAGNOSTICS.md` mirror row, `tools/check_all.ps1` fixture assertions) |
| D5 keyword/permission mismatch (§4.7 cells 2, 5, 8, 9, 11) | `ownership_check` (call keyword × declared param permission) → report, exit 1 | phase-0 D5 check → diagnostic+trap, exit 2 | proposed `H0xxx` change-argument permission mismatch (code TBD at allocation) | as above |
| element-place change arg (`change xs[0]`) — D2 | `ownership_check` syntactic shape → report, exit 1 | phase-0 shape check (fail-closed for `hum run`) → diagnostic+trap, exit 2 | proposed `H0xxx` element-place change argument (code TBD at allocation) | as above |
| change/change or change/borrow-declaring-arg overlap | **Static primary:** `ownership_check` on syntactic caller places → report, exit 1 | **Backstop:** phase-2 overlap sweep (~3143+), after the authority sweep, same diagnostic → diagnostic+trap, exit 2 | **H0810 proposed** | `src/ownership_check.rs` + `src/run.rs` (backstop) + catalog checklist iff accepted (`src/diagnostic_catalog.rs` §8 checklist, `docs/DIAGNOSTICS.md` mirror row, `tools/check_all.ps1` pinned count in `Invoke-HumCompilerFrontChecks`, `tools/test_ci_policy.ps1` `$CompilerBodies` re-pin for that function) |
| change arg overlapping a live caller writable alias | — (liveness is runtime state) | phase-2 live-alias check → diagnostic+trap, exit 2 | **H0808** (existing meaning: the call creates the second live writer H0808 forbids) | `src/run.rs` only |
| change arg on immutable place | `ownership_check` emits the authority diagnostic when it proves the violation (authority-before-overlap deferral) → report, exit 1 | phase-2 `ensure_can_set` authority sweep → diagnostic+trap, exit 2 | proposed `H0xxx` change argument on immutable place (code TBD at allocation) | `src/ownership_check.rs` + `src/run.rs` + catalog checklist iff accepted (as above) |
| change arg on borrow-permission root | **Known Borrow:** `ownership_check`/adapter emit H0802 when the root is statically known borrow-permissioned (caller `borrow` param / borrow-view binding) — ordered **before** overlap (authority-before-overlap) → report, exit 1 | phase-2 `ensure_can_set` → `borrow_mutation_trap` (4856) → diagnostic+trap, exit 2 | **H0802** (same authority violation as an in-body write through a borrow) | `src/run.rs` + producer/adapter per the implication above |
| change arg whose root is moved — before the call or consumed during argument evaluation (`change r.x` + `consume r`, or `consume r.z` which is root-granular) | `ownership_check`: syntactic consume+change on the same root → designated diagnostic (authority-before-overlap) → report, exit 1 | phase-2 moved check → diagnostic+trap, exit 2 — the transfer never resurrects a moved root | proposed `H0xxx` change argument on consumed/moved place (code TBD at allocation) | `src/ownership_check.rs` + `src/run.rs` + catalog checklist iff accepted (as above) |
| non-ownership resource into a Consume parameter — `consume p`, ordinary `p`, or `borrow p` where `p` is a Change/Borrow parameter (§4.9) | `ownership_check` branch (Change/Borrow params are not movable roots — `is_movable_root` 2924) → report, exit 1 | consume-branch guard (~3144) for the keyword form; ordinary/`borrow`-path guard (3148) for the other two → diagnostic+trap, exit 2 | proposed `H0xxx` — **one** designated diagnostic for all three keyword forms (code TBD at allocation) | `src/ownership_check.rs` + `src/run.rs` + catalog checklist iff accepted (as above) |
| change arg on actively iterated root | — | phase-2 `active_iteration_for` (4661) → `iteration_mutation_trap` (4884) → diagnostic+trap, exit 2 | **H0806** (same structural-mutation-during-iteration conflict) | `src/run.rs` only |

Why designated diagnostics, not generic traps, for the new
rejections: the repository's fail-closed probe requires every
unsupported shape of a new form to produce its designated diagnostic
— a validated line that traps generically is a defect — and
stretching H0802/H0806/H0808/H0809 wording to cover call-shape
mismatches would silently expand their approved meanings, which is
forbidden. The D4/D5/D2 shapes, the immutable-place authority
failure, the moved/consumed-root change-arg failure, and the
non-ownership-resource-into-Consume-parameter failure (all three
keyword forms — §4.9) therefore get proposed designated diagnostics
(codes TBD at allocation; the §8 checklist is the allocation
template, one checklist per code). The
borrow-permission-root change arg keeps H0802 explicitly: passing a
borrow-permission root as `change` requests write authority through
a borrow — the same authority violation `borrow_mutation_trap`
names, not an expansion. **Known Borrow takes precedence over
overlap:** a `change` argument on a statically-known borrow root
that also overlaps another change argument emits H0802 (authority),
never H0810 — on both sides, per the §6 admission order.
Precedence among the catalog rows is the §6
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
**proposed, not approved or allocated**, as do the designated
diagnostics above; no existing code names any of these
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
| `src/run.rs` | Two-phase argument loop at the user-task call site (§4.2, at 3143–3160); thread final parameter values through `execute_task` (2040) / `execute_task_body` (2072); the §4.5 final-value transfer + `CallAccess` overlap sweep at the call site through the real call owner (every exit, per the D1/D1-sub ruling); runtime admission — phase 0 runs the pre-authority stages (arity/unknown-task, D4 shape, D5 keyword/permission), phase 2 runs authority (`ensure_can_set` family) → overlap (H0810 backstop after the authority sweep, then live-alias H0808) → snapshots; new consume-branch guard (~3144) and ordinary/`borrow`-path guard (3148) for the §4.9 non-ownership controls; new H0807 `(Field, CallAccess)` / `(Element, CallAccess)` trap arms (existing `stale_view_trap` path, `FieldWrite`/`ListAppend` arms unchanged); no per-mutation instrumentation | The defect and the runtime mechanism owner live here (§2) |
| `src/main.rs` | **Crate-root declaration only:** add `mod change_arg_admission;` (alphabetical — between `mod capability_root;` and `mod check;`, main.rs:10–11) for the new narrow producer module; this is the sole production change in this file. Named as the consumer adapter the plan must stay consistent with: `hum ownership-check` report path (993–1025, `ownership_check_text`/`ownership_check_json` via `callable_text_report`/`callable_json_report`); `hum run --native` preflight admission gate on `ownership_check_has_errors` (1355–1380); trap printing + exit 2 (1528–1531, 1639–1646); the `hum check` stage pipeline (440–560) — no new stage is added, so the D3 `stages` listing stays truthful. Affected CLI tests (by intent, at implementation): `hum check` stages assertions, ownership-check text/JSON assertions for the new diagnostics, `validate_aq_diagnostic_occurrences` tests | Consumer adapter; stages truthfulness (§6) |
| `src/diagnostic.rs` | **No production change proposed.** The new H0807 `CallAccess` trap arms construct their related spans (borrow site, call-access site) through the existing `with_related_span` builder (244) | Consumed by the new H0807 arms; no change needed |
| `src/diagnostic_catalog.rs` | **Only if a proposed diagnostic is accepted:** §8-style checklist per accepted code — `diagnostic_causes!` entry (next free cause key), `diagnostic_code_allocations!` entry (next free allocation key), `historical_public_ordinal` arm, `DIAGNOSTICS` detail entry, the exact count literals in §8 item 5 — plus the open ruling whether H0810's cause key 194 joins `OWNERSHIP_CAUSES`. **Regardless of acceptances:** the H0807 `DIAGNOSTICS` detail entry's explanation/repair prose covers call-access invalidation (§4.5) — prose only; the code row (allocation key 70) is unchanged and no cause key is added | Designated diagnostic-identity owner |
| `src/ownership_check.rs` | **Only if the repair is approved:** static admission of D4/D5/D2 shapes, syntactic overlap, the authority branches (immutable-place change arg, syntactic consume+change on one root), and the non-ownership→Consume-parameter branch (§4.9) — new per-call admission branches in the `ownership_check` stage **calling the shared producer** (`src/change_arg_admission.rs`), invoked for every user-task call expression (not only change-keyword calls), emitting the proposed designated diagnostics into the existing occurrence set; authority-before-overlap deferral is structural in the producer | Proposed static admission owner (by intent); the stage that owns the H08x family |
| `src/change_arg_admission.rs` | **New, narrow — only if the repair is approved:** the pure shared static admission producer (call expression + callee/caller declared permissions + caller binding mutability → ordered admissions, first failure wins) plus the shared message/help builders for the proposed designated diagnostics. Emits no diagnostics itself; consumed by the `ownership_check` branches and the `full_type_check` adapter. (Corrects the earlier "no new producer module" premise: the module is a pure analyzer, not a diagnostic subsystem.) | Single static admission producer; shared text so codes, spans, and precedence cannot drift |
| `src/full_type_check.rs` | **Only if the repair is approved:** the narrow `hum check` adapter — invoke the shared producer from the per-statement call walk (alongside `call_shape_issue`/`call_argument_type_issue`: `Return`/`Binding`/`Other` statements, nested calls via `canonical_child_expressions`, builtin-first callee resolution, unknown callees skipped), emitting through `CheckStageOutcome.diagnostics`. No new stage; no `main.rs` change; the D3 `stages` listing is unchanged | Ordinary-check consumer of the shared producer (§6) |
| `src/diagnostics.rs` | **No hand edit proposed.** `"Hum diagnostics (N codes)"` and `"\"count\": N"` derive from `diagnostic_catalog::all()` via `format!`; only the *test* literals in §8 item 5 change, and only if the catalog changes | `hum diagnostics` contract; affected consumer |
| `docs/DIAGNOSTICS.md` | Mirror rows for accepted codes (test-enforced via `validate_human_projection`; catalog mirror, not prose). **Regardless of acceptances:** the H0807 mirror row is updated to cover call-access invalidation (§4.5 — prose only, no new code) | Standing catalog-mirror rule |
| `tools/check_all.ps1` | Pinned count inside `Invoke-HumCompilerFrontChecks` (**only if a proposed code is accepted** — N→N+k per k accepted codes, not a fixed 100→101); new `Read-NativeOutput[WithExit]` assertions for the new fixtures; the `transaction_once` `ok` assertion (4196–4197) must keep passing unchanged after the example's implementation-time migration | Standing pin practice; acceptance assertions live here |
| `tools/test_ci_policy.ps1` | `$CompilerBodies` digest re-pin for `Invoke-HumCompilerFrontChecks` (**only if that function's body changes** — i.e. only if the pinned count above changes; a `src/run.rs`-only repair does **not** trip it). The digest pins the body bytes of four shared functions (`Invoke-HumCompilerFrontChecks`, `Invoke-HumCompilerCorpusChecks`, `Invoke-HumUseAfterMoveRuntimeCheck`, `Invoke-HumUseAfterMoveProjectionCheck`); the failure message names the function and prints the new digest | Standing pin practice (decision 0028) |
| `fixtures/` | New `.hum` fixtures + session tests exercised **through the real user-task call path** — the §12 traces, the §6 rejection table, the §14 exit rows — placed by family (`fixtures/ownership_check/`, `fixtures/run/`). Counterexample fixtures: `f(change r, g(change r.x))` nested-call preservation; `change r.x` + `consume r` / `consume r.z` rejection (no resurrection); `consume` of a Change/Borrow param (direct + nested forwarding) rejection; immutable-place `change` rejection. No-change-keyword controls: `inner(p)`, `inner(borrow p)`, `inner(consume p)` with `p` a caller Change/Borrow parameter into a Consume parameter (the rejected call carries no `change` keyword — §4.9 rows 3b/6b); keyword-less D5 controls (ordinary argument to a Change parameter — row 2) | Positive-evidence rule: fixtures must observe the effect, not merely declare the form |
| `examples/probes/transaction_once.hum` | **Not edited during planning.** Carried in the inventory as the writable-authority compatibility witness: `let txn` (immutable, line 66) passed as `change txn` (lines 68, 73) is the shape the item-1 authority rule newly rejects — even though `record_debit`/`record_credit` never write. The implementation-time migration is `let txn` → `change txn: Transaction = begin_transaction()`; the `hum run … --entry transfer --args 10` → `ok` assertion (`tools/check_all.ps1`:4196–4197) must keep passing unchanged. **No zero-compatibility-impact claim is made** — the example as written today exercises the newly-rejected shape, and the migration is verified at implementation time, not assumed here | Compatibility witness for the authority rule |
| `src/run.rs` `mod tests` (5716) | Internal tests through `run_program_with_adapters` asserting the §14 observation points (post-call reads, stale-view trap-or-success) through the real evaluator — no journal | §14 seam distinction |

Out of scope for the repair: every other `src/` file, the CLI surface (no new flags), Work Order edits, and any try/catch machinery (D6 / Session W preserved). The parser already admits `change`-argument syntax — no syntax or admission change is proposed anywhere.

## 8. Allocation checklist (proposed — not approved, not allocated)

The checklist below is the per-code allocation template (decision 0028).
H0810 ("overlapping change arguments", family `ownership_borrowing`,
owning stage `ownership_check`) is the proposed allocation for the
change/change and change/borrow-declaring-argument overlap construct
(§6) — runtime emission has precedent: H0802's
`borrow_mutation_trap` is runtime-emitted. The other §6 designated
diagnostics (D4/D5/D2 shapes, immutable-place authority failure,
moved/consumed-root change-arg failure, non-ownership-resource-into-
Consume-parameter failure) are **equally proposed, not approved or
allocated**; each accepted code gets its own pass through this
checklist. Indexes verified read-only at pinned `0e215d31`:

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
5. Count literals — exact at pinned `0e215d31`, and cause counts
   move independently of public-code counts (when k new codes with
   c new causes are accepted, each literal N → N+k or N+c
   respectively): **registered-cause count** —
   `assert_eq!(DIAGNOSTIC_CAUSES.len(), 193)` in
   `src/diagnostic_catalog.rs` tests (gains one per new *cause
   key*); **public-code counts** —
   `assert_eq!(summary.active_codes, 97)` (two sites),
   `assert_eq!(all().len(), 100)` in `src/diagnostic_catalog.rs`
   tests, `assert_eq!(catalog.len(), 100)`,
   `assert!(text.starts_with("Hum diagnostics (100 codes)\n"))`,
   `assert!(json.contains("\"count\": 100"))` in
   `src/diagnostics.rs` tests (each gains one per new *public
   code*). The production code in `src/diagnostics.rs` derives both
   strings via `format!` — no literal edits there. The CallAccess
   variant (§4.5) registers no cause and allocates no code, so it
   moves none of these literals.
6. `docs/DIAGNOSTICS.md` mirror row.
7. `tools/check_all.ps1` pinned count (the `100` inside
   `Invoke-HumCompilerFrontChecks`'s canonical-catalog assertion) +
   `tools/test_ci_policy.ps1` `$CompilerBodies` digest re-pin for
   that function (the digest pins that function's body bytes; the
   failure message names the function and prints the new digest).

Until acceptance, none of the above is done; the map's §6 rejection table stands as the
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
- D1 and D1-sub are **ruled** (see §5, §14): transfer and invalidation precede
  `Returned`, `Failed`, and `ContractViolation`; pre-body rejection does neither.
  D2, D3, D4, D5, D6 remain **open** pending Ocean/Codex approval. H0810
  remains **proposed, not approved or allocated**.
- Any implementation needs a Work Order before branch work begins.
- Builder lane for this delivery: one documentation file, no source/tooling changes, no
  builds, no test campaigns, no CI dispatch, no Work Order activation, no merge, no `main`
  push. Delivery ends at the draft PR.
- §§7–11 above (from the tail-completion notice onward) are newly authored completion,
  not recovered historical text.

> **Superseded 2026-09-30 on one point:** D1/D1-sub are now ruled (see §5, §14);
> D2–D6 remain open, H0810 stays proposed. The dated notice below is preserved
> verbatim as evidence.
>
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

> **Superseded 2026-09-30 on one point:** D1/D1-sub are now ruled (see §5, §14);
> D2–D6 remain open, H0810 stays proposed. The dated notice below is preserved
> verbatim as evidence.
>
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
4914) are stated independently. The value outcome is the callee
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
   a bare root, the `CallAccess` overlap sweep marks all caller
   Element views with place-root `xs` (snapshots from `borrow
   xs[0]`) with `invalidated_by = Some(CallAccess)` — a later use
   fires H0807 via the new `(Element, CallAccess)` trap arm.
   `invalidate_element_views_for_growth` is not called at the
   transfer site; its `ListAppend` cause keeps its exact growth
   meaning at real growth sites. When P is a
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
  change xs: List UInt = [1]
  let e = borrow xs[0]
  append_one(change xs)
}
```

- t0 (main env): `xs → [1]` (a `change` binding); `e →
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
  silently lost); the §4.5 `CallAccess` overlap sweep invalidates
  all element views with place-root `xs` → `e.invalidated_by =
  Some(CallAccess)` (**effect transfer** — call-access
  invalidation, reasoned truthfully as call-access, not as an
  observed write; `invalidate_element_views_for_growth` is not
  called at the transfer site).
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
type Counts {
  count: UInt
}
task inner(change q) {
  set q.count = q.count + 1
  set q = { count: 99 }
}
task outer(change p) { inner(change p) }
task main() {
  change r: Counts = { count: 0 }
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

Producers — one shared static producer, two static consumers, one
runtime owner (§6): **static producer:** the pure admission
function in the new narrow `src/change_arg_admission.rs` — D4/D5/D2
shapes, syntactic overlap, the provable-authority branches
(immutable-place change arg, syntactic consume+change on one root,
**Known Borrow** authority → H0802, ordered before overlap),
and the non-ownership→Consume-parameter branch (§4.9) — deciding
the statically-decidable stages in admission order with
authority-before-overlap structural (first failure wins); it emits
no diagnostics itself and owns the shared message/help builders.
**Static consumers:** (A) new per-call admission branches in
`src/ownership_check.rs`, invoked from the ownership walk for
**every user-task call expression** — call statements, binding
initializers, return expressions, nested argument positions —
regardless of argument keywords, emitting the proposed designated
diagnostics into the existing occurrence set
(`diagnostic_occurrence_set`, ownership_check.rs:822); (B) the
narrow adapter at `check_stage_outcome` (full_type_check.rs:252 —
the `hum check`-only entry, **not** inside the shared
`build_report_with` walk, which effect checking also consumes via
`with_full_type_for_effect`): the `call_shape_issue`/
`call_argument_type_issue` walk pattern (`Return`/`Binding`/`Other`
statements, nested calls via `canonical_child_expressions`,
builtin-first callee resolution, unknown callees skipped), emitting
through `CheckStageOutcome.diagnostics`. The two static consumers are
command-disjoint (`hum check` never runs `ownership_check`; the
native preflight and explicit `hum ownership-check` never run the
check pipeline's `full_type_check`), so no cross-consumer
duplicate prevention is needed. **Runtime:** the phase-0
pre-authority checks (~3143 — arity/unknown-task, D4 shape, D5
keyword/permission), the phase-2 authority sweep emitting the
proposed designated diagnostics for immutable-place and
moved/consumed-root change args (diagnostic+trap, same channel as
H0802/H0806), the phase-2 overlap sweep (H0810 backstop after the
authority sweep, then the live-alias H0808 check), the
consume-branch guard (~3144) and the ordinary/`borrow`-path guard
(3148) emitting the one proposed non-ownership→Consume diagnostic
(§4.9), `ensure_can_set` (4780) with `borrow_mutation_trap` (4856)
for borrow-permission roots (H0802 reuse), and
`active_iteration_for` (4661) with `iteration_mutation_trap` (4884);
`preflight_writable_aliases` (2135) for the existing H0808/H0809
emissions; the new `CallAccess` overlap sweep at the transfer site
(§4.5). The runtime side does not consume the static producer (it
needs runtime state); where message/help text must be identical it
uses the producer's shared builders. The new module is a pure
analyzer, not a diagnostic subsystem — emission stays in the two
existing owners.

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
CLI behavior (§14); the Codex outstanding fixes (2026-09-30): the
`hum check` admission adapter sits at `check_stage_outcome`
(full_type_check.rs:252) — not inside the shared
`build_report_with` walk, which effect checking also consumes via
`with_full_type_for_effect` (744); Known Borrow authority (H0802)
precedes overlap (H0810) on both sides — a `change` on a
statically-known borrow root emits H0802, never H0810; the new
module's `mod change_arg_admission;` crate-root declaration
(main.rs:10–11) is in the §7 inventory. Claude's findings
(2026-09-30): declaration-time block restoration is the transfer's
prerequisite — fresh `Env` with
`RuntimeBinding::parameter(value, permission, definition_id)`
metadata per call, preserving initializer effects, executed
shadows (WO28 #16 block save/restore), per-iteration scope, and
binding metadata, with actual parameter bindings identified by
`definition_id` (`binding_by_definition_id`, run.rs:4695) — never
by bare name; the transfer captures from the actual `env`, never
the postcondition `exit_env` (synthetic `result`/`old(...)`
locals, `finish_success` 2294); the narrow ownership-laundering
closure is mapped in §4.9a (L1 direct closed by §4.9; L2 one-hop
unannotated-local copy, L2b borrow-view local, and L3 return
severed by deep-copy/snapshot value semantics —
run.rs:286–296, 2698–2755, 4692, 5416), non-linear copying is
preserved (`let y = p` stays legal; the repair targets the
consume, never the copy), and no additional semantic ruling is
required — stated explicitly in §4.9a. Executed probes referenced
in this map (H0640/H0641 call probes, the fail-closed probe) were
run by Claude; walk mechanics stated here are source inspection
at the pinned commit, not reruns.

**Recommendations** (argued by the map; need Ocean/Codex approval):
D2 (reject element-place change args via static admission), D4
(reject non-place change args with a proposed designated diagnostic
via static admission), D5 (the complete §4.7 matrix, admitted
statically with proposed designated diagnostics), D6 (Session W
preserved); D3 (overlap rejection, with H0810 proposed **only** for
change/change and change/borrow-declaring-argument overlap —
live-writable-alias overlap reuses H0808 as a settled reuse, not an
expansion); the shared static admission producer with two static
consumers (the `ownership_check` branches and the `hum check`
adapter) plus the runtime admission in `run.rs`, with
authority-before-overlap (§6); the two-phase loop and the §4.5
transfer algorithm; the §4.9 non-ownership controls (shared static/runtime rejection
of a non-ownership resource — `consume p`, ordinary `p`, or `borrow
p` where `p` is a Change/Borrow parameter — into a Consume
parameter, with one truthful proposed designated diagnostic —
mutation authority is not ownership-transfer authority); the
phase-2 controls (§4.2)
with designated diagnostics for the immutable-place, moved-root,
and consume-combination failures — never generic invariant traps;
the `transaction_once.hum` witness carried unedited in §7 with no
zero-compatibility-impact claim.

**Open rulings** (the map makes no recommendation; Ocean/Codex must
choose): whether H0810, the D4/D5/D2 designated diagnostics, and
the new counterexample diagnostics (immutable-place change
argument, change argument on a moved/consumed root,
non-ownership-resource-into-Consume-parameter — one diagnostic
covering all three keyword forms) are allocated (if yes: the §8-style
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
