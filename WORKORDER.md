# Hum Work Order 5: Stale Views And Predicate V1

Date: 2026-07-09
Status: active, issued under delegated authority (GOVERNANCE.md), BDFL veto open
Owner: BDFL (Ocean). Reviewer/ruler: architect-reviewer. Implementer: agent sessions.
Predecessor: Work Order 4 closed at commit `01e922f`: corpus burn-down
1/12 -> 5/12 running plus program 3 partial; two areas over three strikes.

## Why this document exists

Session Q's ledger mandated two items, both clusters, not preferences:

- Ownership (five active records, three stale-view-shaped): a narrow
  stale-view slice — field views and element views with invalidation —
  before internal references.
- Contracts (four active records, three vocabulary-shaped): Predicate v1
  grown strictly from the recorded wishlist — pre-state references and
  small collection predicates — before the contract-check-mode ADR.

This work order ships both, then measures. The 0014 honesty locks bind
all output text; locks narrow only by shipping the checked feature that
retires them.

## Global bans (all sessions)

- No closures or tasks-as-values (effect-polymorphism gate; backlog 15).
- No internal references: views stored in records (`from parser.buffer`)
  stay rejected by H0805. Local, within-task views only.
- No general alias inference: view tracking covers views bound by the
  recognized local forms this work order defines, nothing else.
- Predicate v1 vocabulary is exactly what the wishlist recorded:
  pre-state references and collection length/count. No quantifiers, no
  implication, no user-defined predicate tasks.
- No concurrency, FFI, backends, generics, new schemas, or new report
  subcommands.
- Every new rejection rule ships with a misuse fixture and a stable
  diagnostic with blame-style help naming the site and the fix.
- Surface changes update grammar/reference/syntax/TextMate/README
  showcase in the same session.

## Session R: field views and invalidation (programs 8 and 11 misuses)

Scope:

1. Local field views in the recognized subset: a `let`-bound borrow of a
   field place (e.g. `let old_done = borrow item.done`) is tracked as a
   view of that place.
2. Invalidation rule: writing a field invalidates outstanding views of
   that field. Using an invalidated view rejects — new stable diagnostic
   (H0807 expected) whose help names the view binding, the invalidating
   write, and the fix (re-borrow after the write or copy the value
   first). Views of *distinct* fields survive writes to other fields
   (disjoint-field v0 extends to views).
3. Static checking and runtime enforcement share the diagnostic
   identity, as with H0801-H0806.
4. Make the corpus program 8 and 11 stale-view misuses real fixtures;
   annotate the two Session O `blocked` friction records resolved.
5. Copy semantics stay visible: `let old_done: Bool = item.done` (a
   value copy) is NOT a view and survives writes — a passing fixture
   must demonstrate the distinction, because teaching it is the point.

Acceptance criteria:

- Both misuse fixtures fire the new diagnostic with both sites named.
- A distinct-field view survives an unrelated write (passing fixture).
- The copy-vs-view distinction fixture passes.
- Friction records annotated; `.\tools\check_all.ps1` passes. Stop.

## Session S: element views and growth invalidation (program 12 misuse)

Scope:

1. Extend view tracking to list elements: a `let`-bound borrow of an
   element is a view tied to the list's growth state.
2. `list_append` (and any structural growth) invalidates outstanding
   element views; use-after-growth rejects with the same diagnostic
   family as Session R, help naming the append site.
3. Make the corpus program 12 stale-element-view misuse a real fixture;
   annotate the Session P `blocked` record resolved.
4. Iteration and views compose: H0806 (iteration conflict) and view
   invalidation must not double-fire confusingly on one line — pick the
   more specific diagnostic and test that choice.

Acceptance criteria:

- Element misuse fixture fires with append site named.
- The H0806/H0807 overlap case has a fixture proving one clear
  diagnostic wins.
- Friction record annotated; `.\tools\check_all.ps1` passes. Stop.

## Session T: Predicate v1 (the contracts mandate)

Scope:

1. Pre-state references: `old(expr)` in `ensures:`, where `expr` is a
   parameter or parameter field readable at task entry. Evaluated by
   capturing the value at entry. Example: `ensures: result.x ==
   old(point.y)`.
2. Collection predicates: `list_len(expr)` usable in `needs:`/`ensures:`
   comparisons. Example: `ensures: list_len(result) == 3`.
3. Retire the golden-value contracts this unlocks, honestly: swap_xy
   gains `result.x == old(point.y)` and `result.y == old(point.x)`;
   complete_item gains `result.title == old(item.title)` (the
   preservation contract that Session O could not express);
   builder_demo's prose becomes `list_len(result) == 3` where that is
   the honest claim (list *content* predicates remain unimplemented —
   the prose line about contents stays prose, and stays warned).
4. Update H0701's help text to name the v1 vocabulary. Annotate the
   four contract wishlist records: pre-state and count/length resolved;
   list-content remains open with one recorded demand.
5. Sabotage tests: a wrong swap (return without swapping) must fail its
   old()-contract at runtime with H0703 task blame — the pre-state
   analogue of wrong_add.

Acceptance criteria:

- The sabotage fixture fails its old()-ensures with task blame.
- complete_item's preservation is now a checked, non-vacuous contract.
- builder length contract checks; content claim stays honestly prose.
- Wishlist annotated; `.\tools\check_all.ps1` passes. Stop.

## Session U: retrospective three

Scope:

1. Update SCORECARD.md implementation status: misuse-coverage columns
   for programs 8, 11, 12; note program 3's remaining gap precisely.
2. Consolidate friction records; re-apply the three-strike rule to
   active unresolved records.
3. Recommendation for Work Order 6, argued from the ledger AND the
   adoption strategy: the candidates are internal references (program 5,
   the last big ownership blocker), the effect-polymorphism ADR
   (unblocks program 3/4, closures, higher-order stdlib), and the first
   IO capability slice (clock/stdout/file-read as declared capabilities,
   which unblocks real tools, the wedge demo, sandboxing flags, and the
   deterministic-mode groundwork). State explicitly what each choice
   defers and for how long.

Acceptance criteria:

- Status covers all twelve, no TBD, degenerate cases not counted.
- Three-strike rule applied; honesty locks reviewed.
- Recommendation weighs all three candidates with deferral costs.
- `.\tools\check_all.ps1` passes. Stop: Work Order 5 ends here.

## Design probe system (standing)

Probe sources: regret-ledger, construct-pair, misuse, domain-slice.
Friction record format:

```text
friction:
  program: <file and line>
  wanted: <what the author tried to write>
  forced: <what the language required instead>
  severity: blocked | wrong-by-default | awkward | verbose
  indicts: <contracts | ownership | loops | types | diagnostics | stdlib | checker | core-body-grammar>
  proposal: <optional one-line fix direction>
```

Rules: three active records indicting one area triggers a decision
record or work-order item; `blocked`/`wrong-by-default` triaged before
the next session; resolved records stay in the ledger, annotated.

## Showcase discipline (standing)

README/SPEC examples over five lines extract from checked fixtures
(preflight-enforced); README shows minimal, full-contract, and Magic
Comment sections; surface-changing sessions update the showcase in the
same session. Session T's old() contracts are showcase-worthy: the
sabotaged-swap example may join the Magic Comment section if it earns it.

## Backlog: accepted taste, not scheduled

1. Deterministic run mode (virtual clock, seeded random, fixed
   schedule; bit-for-bit replay; stable test-mode iteration order).
2. Semantic diff (`hum diff`): effect/contract/capability deltas.
3. Machine-applicable fixes (`hum fix --apply`); gates the public
   agent-native claim per risk R016.
4. Sandboxed execution flags (`--allow`/`--deny`) when IO capabilities
   arrive (pairs with the WO6 IO-slice candidate).
5. Fault containment: direction settled; design work order before any
   concurrency syntax.
6. Units of measure: direction settled; waits for type-system maturity.
7. Language editions before public alpha stability promises.
8. Contract check policy ADR (proved | boundary | unproved |
   external-trust as build evidence) — deferred behind Predicate v1 per
   the Session Q mandate ordering.
9. Predicate v2 wishlist: list-content predicates (one demand recorded);
   grows only from the ledger.
10. List operation surface beyond append: retain (needs effect
    polymorphism), capacity, profile behavior.
11. Numerics policy ADR (checked-by-default all modes; explicit
    families; benchmark gate; two FP regimes; decimal library-first).
12. Text model tiers (Bytes / Text / OsText) with early stdlib design.
13. Source policy hardening: bidi-control rejection in text hygiene;
    ASCII-identifier policy recorded as deliberate.
14. Ownership repairs after this work order: internal references
    (program 5), then broader flow-sensitive borrowing; order per
    Session U evidence.
15. Effect polymorphism ADR: blocks closures, tasks-as-values, retain,
    higher-order stdlib, and programs 3/4 completion.
16. Spec-of-record demo: a full-contract Hum task exported (graph JSON +
    contract facts) as the canonical spec, with an agent generating an
    implementation in Python/Rust reviewed against it. Nothing new to
    build — a demo of existing surfaces; candidate for the first public
    artifact alongside the wedge demo. Honest framing only: Hum is the
    spec that cannot rot, not a guarantee exporter — ported code loses
    enforcement.
17. Stdlib labs formalization: when compiler-known built-ins
    (list_append, slice_until, list_len, old) reach critical mass,
    charter the labs pipeline per STDLIB_STRATEGY and the
    STDLIB_CONSTITUTION admission packet.
18. Error context and chaining: typed fail values need a
    cause/wrapping story ("LoadError caused by IoError, here is the
    trail") BEFORE the IO capability slice, because IO is where error
    chains are born. Go's decade of unresolved verbosity and Rust's
    post-1.0 anyhow/thiserror split are the regrets to avoid; Hum's
    blame machinery is the head start.
19. Entry point as capability root: Hum's program entry is not main();
    it is where program-level authority is declared (the app form with
    uses:/starts with:). Design it WITH the IO capability slice, not
    before. Entry tasks returning Result map to exit codes natively
    (hum run already does 0/1/2).
20. Module path binding: when multi-file programs land, module paths
    are tool-enforced to match file paths; imports never execute code;
    visibility stays the small export/package/private set.
21. The Hum book: a digital book in the repo — "Hum: A Systems
    Programming Language for Humans and Agents" — written as the
    teaching companion, one chapter per shipped feature, every code
    example extracted from checked fixtures under the showcase
    discipline so the book cannot rot. The pedagogy gate made
    enforceable: a feature is not stable until its chapter exists.
    Chapters begin only for shipped features; the book never describes
    the future. Hard copies are a later BDFL call; the repo book is
    the canonical free edition (the Rust Book / Crafting Interpreters
    adoption model).
