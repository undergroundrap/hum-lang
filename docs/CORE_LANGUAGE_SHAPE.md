# Hum Core Language Shape

Date: 2026-07-06

## Purpose

Hum must feel familiar enough that experienced programmers can become productive,
but strict enough that it does not inherit the accidental complexity of C, C++,
JavaScript, or macro-heavy systems.

The core language should be small, teachable, and boring in the best sense.

See [ERGONOMICS_AND_OPERATORS.md](ERGONOMICS_AND_OPERATORS.md) for the operator
and quality-of-life admission rules.

See [FORMAL_CORE.md](FORMAL_CORE.md) for the precise executable core that these
surface constructs must lower into before they become stable.

## Brutal Rule

Hum needs common programming constructs. It does not need every common spelling.

If programmers expect a thing, Hum should either provide it clearly or reject it
clearly. Ambiguous absence is design debt.

## Constructs Hum Should Have

### Bindings

Immutable by default:

```text
let limit: UInt = 10
```

Mutable local state should be explicit:

```text
change count: UInt = 0
set count = count + 1
```

No hidden mutation through innocent-looking calls.

### Conditionals

Hum should have ordinary `if` / `else` because every programmer expects it.

```text
if token is expired {
  fail SessionError.expired
} else {
  return session
}
```

### Pattern Matching

Hum should have exhaustive `match` for enums, options, results, and tagged
states.

```text
match load user(id) {
  ok user:
    return user

  error missing:
    fail LoginError.no_such_user

  error storage:
    fail LoginError.storage_unavailable
}
```

The compiler should reject non-exhaustive matches unless a deliberate fallback is
written.

### Loops

Yes, Hum needs `while`.

Hum should have four loop forms:

```text
while condition { ... }
loop { ... }
for each item in items { ... }
for index i from 0 until count { ... }
```

Hum should not have C-style `for (init; condition; step)` loops. They compress
three different ideas into punctuation and invite off-by-one bugs.

Critical loops may carry nested intent:

```text
while attempts < max attempts {
  keeps:
    attempts <= max attempts

  changes:
    attempts

  watch for:
    token source may repeat values

  does:
    if candidate token is unused {
      return candidate token
    }

    set attempts = attempts + 1
}
```

Loop labels should feed the verifier, test generator, and agent diagnostics.

### Early Exit

Hum should have explicit early exits:

```text
return value
fail error
break
continue
```

`fail` is not an exception. It returns a typed failure path declared by
`fails when:`.

### Ranges

Ranges should be readable and precise:

```text
for index i from 0 until count
for index i from 1 through last
items[from 0 until count]
```

Compact range spellings can be considered later, but beginner-facing Hum should
make inclusive/exclusive endpoints visible in words. The compiler should make
inclusive/exclusive range mistakes visible in diagnostics where possible.

### Defer Or Cleanup

Programmers expect reliable cleanup. Hum should provide scoped cleanup, but only
after ownership and effects are specified.

Candidate syntax:

```text
defer close file
```

This is not Milestone 0. It belongs near unsafe, FFI, and resource lifetimes.

### Modules And Visibility

Hum should have modules and visibility, but avoid C++ header-style duplication.

Likely direction:

```text
module sessions
public task create session(...)
private task prune expired sessions(...)
```

The `.hum` file remains the source of truth. Interfaces are generated.

### Tests

Tests should be first-class and generated from contracts when possible.

```text
test create session rejects unverified user {
  uses:
    fake clock
    fake random

  does:
    expect create session(unverified user, device) fails with UserNotVerified
}
```

Manual tests should coexist with generated tests.

## Constructs Hum Should Reject Or Delay

Reject for core:

- `goto`
- exceptions as hidden control flow
- implicit null
- implicit integer narrowing
- fallthrough switch statements
- C-style preprocessor macros
- C-style `for` loops
- overloaded assignment or control-flow operators

Delay until proven:

- user macros
- closures
- async/await surface syntax
- operator overloading
- inheritance
- dynamic dispatch by default
- compile-time reflection

Some of these may eventually exist. None should enter before Hum can preserve
them in its semantic graph and explain them to agents.

## What Programmers Expect, And Hum's Answer

| Expectation | Hum answer |
| --- | --- |
| functions | `task` with intent blocks |
| variables | `let` immutable, `change` mutable |
| assignment | `set`, checked against mutation permissions |
| if/else | yes |
| switch | exhaustive `match` |
| loops | `while`, `loop`, `for each`, `for index` |
| errors | typed `Result` and `fail` |
| null | no implicit null, use `maybe T` |
| modules | yes, no headers |
| visibility | `public` / `private`, keep small |
| generics | yes eventually, constrained and contract-visible |
| interfaces/traits | maybe, only if verifier-friendly |
| comments | allowed, but checked blocks are preferred |
| unsafe | allowed only in named, justified boundaries |
| performance | benchmark-backed `optimizes:` |

## Design Pressure

Every construct should answer:

1. Can a human read it without decoding punctuation tricks?
2. Can the compiler check its effects and mutation?
3. Can an agent receive its meaning as JSON?
4. Can it be taught in one page?
5. Can it lower to efficient machine code?

If not, keep it out until it can.

## Friction Ledger

Session D records from the first real executable probes. These are design pressure, not fixes made in the same slice.

friction:
  program: examples/core/divide.hum:26
  wanted: keep a defensive typed failure guard in the body while also declaring `needs: b != 0`
  forced: with executable `needs:` checks enabled, the body guard is unreachable for ordinary `hum run`
  severity: awkward
  indicts: contracts
  proposal: decide contract check mode (`always` | `debug` | `profile`) before profiles or release mode exist
  resolution: policy question resolved by decision 0015 with classified runtime-contract policy; current Hum still checks every executable predicate, and the classifier, profiles, proof evidence, and elision remain an unimplemented honesty lock/backlog item rather than an active friction record

friction:
  program: examples/probes/word_count.hum:8
  wanted: state that the result equals the number of matching words in the literal list
  forced: hard-code `result == 2` because predicate v0 has no collection count, quantifier, or helper-call contract vocabulary
  severity: awkward
  indicts: contracts
  proposal: frequency-rank collection count predicates before growing predicate grammar v1
  resolution: partially resolved in Session T; `list_len(place)` covers plain length comparisons, but the original want (count of MATCHING items) still needs content-conditional vocabulary and remains an active demand

friction:
  program: examples/probes/task_list_flow.hum:58
  wanted: append a new work item to an existing list as the add operation
  forced: spell the post-add list as a fresh list value because the current executable subset has no list append or Vec API
  severity: awkward
  indicts: stdlib
  proposal: design the smallest list operation surface before richer state probes
  resolution: partially resolved in Session P by `list_append(change list, item)`; retain, capacity/profile behavior, element views, and broader list stdlib design remain future work

friction:
  program: examples/probes/task_list_flow.hum:15
  wanted: update one record field (`done`) while preserving the rest of the work item
  forced: construct a replacement record literal with every field repeated
  severity: verbose
  indicts: types
  proposal: put record update syntax through the ownership bake-off instead of adding it ad hoc
  resolution: resolved for direct in-place field mutation in Session O by `set item.done = true`; record-update expression syntax remains future ergonomic surface

friction:
  program: examples/probes/transaction_once.hum:69
  wanted: write `rollback(consume txn)` and `commit(consume txn)` as standalone close actions
  forced: bind each `Unit` result to a throwaway local because standalone call statements are not in the recognized Core body grammar yet
  severity: awkward
  indicts: core-body-grammar
  proposal: admit standalone effect/close call statements only after they preserve effect, ownership, and diagnostic facts

friction:
  program: examples/probes/transaction_once.hum:66
  wanted: mark a local as a linear resource with explicit source syntax or a declared resource kind
  forced: recognize Transaction-shaped type annotations as the first narrow linear-resource class
  severity: awkward
  indicts: ownership
  proposal: design a source-visible linear resource marker before generalizing exactly-once checking beyond transaction probes

friction:
  program: fixtures/ownership_check/session_l_return_view_internal_fail.hum:3
  wanted: express a returned view that depends on a field inside an owned parser-like value with `Slice Text from parser.buffer`
  forced: reject internal-reference sources and accept only `from` relationships that name a task parameter
  severity: blocked
  indicts: ownership
  proposal: fund the internal-reference repair from decision 0014 only after the parameter-derived returned-view subset stays green under the corpus probes

friction:
  program: docs/bakeoff/CORPUS.md:242
  wanted: `first_word("hum language")` returns `"hum"` as a sub-view derived from its input
  forced: accept only bare-parameter returned views, so the passing Session L fixture can only echo the whole parameter
  severity: blocked
  indicts: ownership
  proposal: design a minimal checked sub-view derivation rule before counting program 9 as implemented; keep it distinct from internal-reference repair
  resolution: resolved in Session N by `examples/probes/first_word.hum` using the closed view-deriving operation `slice_until(text, separator)`; local and non-closed derivation misuse fixtures still reject with H0805

Session M consolidation:

The records above are the accumulated friction ledger through Work Order 3.
Three-strike counts are applied to the `indicts:` field, not to preference or
candidate-scorecard hopes.

| Indicted area | Count | Records | Three-strike result |
| --- | ---: | --- | --- |
| ownership | 3 | Transaction-shaped linear marker; internal-reference `from parser.buffer`; missing program 9 sub-view derivation. | Triggered. A next work order item is required. |
| contracts | 2 | Contract-check-mode for `divide`; missing collection-count predicates in `word_count`. | No trigger yet; stays on the predicate/check-mode backlog. |
| stdlib | 1 | Missing list append or Vec API in `task_list_flow`. | No trigger yet. |
| types | 1 | Record field update requires replacement literal spelling. | No trigger yet, but this is direct pressure toward disjoint-field projection and record-update syntax. |
| core-body-grammar | 1 | Standalone close/effect calls must be bound to throwaway locals. | No trigger yet. |

Three-strike outcome:

- Ownership is the only area over the threshold. The required work-order item
  is not a broad ownership victory lap; it is a narrow repair that makes an
  already exposed graph fact real for ordinary code.
- The first funded repair should be checked returned-view provenance for
  sub-views derived from parameters, treated as the first narrow slice of
  flow-sensitive borrowing for Work Order planning. It must accept the real
  corpus program 9 shape (`first_word("hum language") -> "hum"`), reject
  local-buffer returns, and preserve `hum graph` and `hum ownership-check`
  dependency facts.
- Disjoint-field projection remains high-value and should follow, because
  programs 8 and 11 are common or pervasive. It should not be first unless the
  architect-reviewer decides to create a separate sub-view provenance work
  order outside the disjoint-field versus flow-sensitive borrowing taxonomy.
- Internal references remain blocked and important for parser state, but the
  Session L evidence says not to count `from parser.buffer` as implemented
  before the parameter-derived returned-view subset handles real sub-views.

Session O friction records:

friction:
  program: docs/bakeoff/CORPUS.md:219
  wanted: keep a live view or alias of `point.x`, update `point.x`, and reject later use of the stale field view
  forced: Session O can express only direct field-place reads and writes, not field views or aliases
  severity: blocked
  indicts: ownership
  proposal: fund stale field-view invalidation only after direct field places and disjoint-field v0 stay green
  resolution: resolved for local direct field views in Session R by `fixtures/ownership_check/session_r_stale_point_field_view_fail.hum` with H0807; nested places and general aliases remain future work

friction:
  program: docs/bakeoff/CORPUS.md:296
  wanted: keep a live view of `item.done`, update `item.done`, and reject using the stale field view while preserving unrelated fields
  forced: Session O can run direct `set item.done = true`, but cannot express field views or stale-view misuse yet
  severity: blocked
  indicts: ownership
  proposal: add field-view provenance and invalidation as a later ownership repair; do not count weaker direct-field fixtures as covering stale-view misuse
  resolution: resolved for local direct field views in Session R by `fixtures/ownership_check/session_r_stale_item_field_view_fail.hum` with H0807; `examples/probes/field_views.hum` proves distinct-field views and value copies survive as separate cases

friction:
  program: examples/probes/field_places.hum:17
  wanted: express field preservation and swap contracts against pre-state, such as `old(point.x)` or `old(item.title)`
  forced: use golden-value predicates for `swap_xy` and prose `title is preserved` for `complete_item` because predicate v0 has no pre-state reference
  severity: blocked
  indicts: contracts
  proposal: Session Q must carry a mandated contracts work-order item: either predicate v1 with pre-state references or the contract-check-mode ADR, chosen from the full friction ledger
  resolution: resolved in Session T by `old(place)` in `ensures:` with entry capture; `swap_xy` now checks `result.x == old(point.y)`/`result.y == old(point.x)`, `complete_item` checks `result.title == old(item.title)`, and `fixtures/run/session_t_wrong_swap_contract.hum` proves a sabotaged swap fails with H0703 task blame; `old(...)` outside `ensures:` stays honest prose (H0701)

friction:
  program: docs/bakeoff/CORPUS.md:330
  wanted: keep a view of the first element, append more items, and reject later use of the stale element view
  forced: Session P can express append growth and add-after-finish, but cannot express element views or stale element-view misuse yet
  severity: blocked
  indicts: ownership
  proposal: include stale element-view machinery with the remaining view repairs considered in Session Q; do not count append-only fixtures as covering stale element views
  resolution: resolved for local direct element views in Session S by `fixtures/ownership_check/session_s_stale_element_view_fail.hum` with H0807 and `examples/probes/element_views.hum`; only direct numeric element places and `list_append` growth are covered, while retain, nested/general indexing, capacity/profile behavior, and broader list design remain future work

friction:
  program: examples/probes/list_builder.hum:16
  wanted: check that the completed builder list contains the appended items `parse`, `check`, and `run`
  forced: use a prose `ensures:` line that surfaces as H0701 because predicate v0 has no list-content, sequence, or collection-shape vocabulary
  severity: awkward
  indicts: contracts
  proposal: include small collection/list predicates in Predicate v1 only after the Session Q contract recommendation authorizes that grammar work
  resolution: partially resolved in Session T; `builder_demo` now checks `list_len(result) == 3`, but the content claim (which items the list contains) stays honest prose and remains an active demand

friction:
  program: examples/probes/element_views.hum:8
  wanted: check a text-valued result with a literal comparison such as `result == "parse"`
  forced: text literals are outside the predicate vocabulary, so the line surfaces as honest prose under H0701
  severity: awkward
  indicts: contracts
  proposal: consider text-literal equality for a future predicate version only via the wishlist; do not smuggle it into Session T's mandated scope

Session O three-strike note:

The contracts area now reaches three strikes: `divide` contract-check mode, `word_count` collection-count predicates, and missing pre-state references for field-preservation predicates. Session Q must carry a mandated contracts work-order item in its recommendation instead of treating contracts as optional backlog polish.

Session Q consolidation:

The records above are the accumulated friction ledger through Work Order 4.
Three-strike counts are applied to active unresolved `indicts:` records, while
resolved records stay in the ledger as history and design evidence.

| Indicted area | Active count | Records | Three-strike result |
| --- | ---: | --- | --- |
| ownership | 5 | Transaction-shaped linear marker; internal-reference `from parser.buffer`; stale field view after `point.x` update; stale field view after `item.done` update; stale element view after list growth. | Triggered. The next ownership repair should be stale-view machinery first, then internal references. |
| contracts | 4 | Contract-check-mode for `divide`; missing collection-count predicates in `word_count`; missing pre-state references for field preservation; missing list-content/list-shape predicates in `builder_demo`. | Triggered. The next work order must carry a Predicate v1 item. |
| stdlib | 1 partial | The old missing-append record is partially resolved by Session P's minimal `list_append`; retain, capacity/profile behavior, element views, and richer list design remain future work. | No new trigger beyond the already scheduled list backlog; effect polymorphism still gates retain. |
| types | 0 active for direct field update | The replacement-record spelling record is resolved for direct in-place field mutation by Session O; record-update expression syntax remains future ergonomics. | No trigger. |
| core-body-grammar | 1 | Standalone close/effect calls must be bound to throwaway locals. | No trigger yet. |

Three-strike outcome:

- Ownership remains triggered, but the evidence changed. Program 9's parameter
  sub-view blocker is resolved by Session N, while Sessions O and P created a
  cluster of unresolved stale-view records around fields and list elements.
- Contracts are now triggered. The weight is no longer just `divide` check mode:
  three records ask for checked predicate vocabulary that v0 cannot express.
- Stdlib append pressure is partially relieved by Session P, but retain remains
  gated by effect polymorphism and the list standard-library surface remains
  deliberately tiny.

0014 honesty locks after Session Q: all broad locks remain. Hum has narrow
checked ownership facts for local moves, permissions, Transaction-shaped linear
resources, parameter-derived returned views through bare returns and closed
`slice_until` derivations, direct field-place mutation, minimal `list_append`,
consume-finish builder handoff, and active-iteration append rejection. It still
has no full ownership safety claim, borrow-soundness claim, memory-safety proof,
safety-critical readiness claim, internal-reference support, general stale field-view
invalidation beyond local direct fields, general stale element-view invalidation beyond direct list growth, broad disjoint-field projection,
broad flow-sensitive borrowing, concurrency ownership model, mature list stdlib,
or general linear resource marker.

Session Q recommendation:

Fund stale-view machinery before internal references. The stale-view records now
cover programs 8, 11, and 12, and they attach to features that users can already
write after Sessions O and P. Program 5's internal references remain important,
but they should build on the same provenance and invalidation machinery after
field and element views have a checked, blamed, line-numbered failure mode.

Session R update: local direct field views now have that checked failure mode. `let view = borrow record.field` is accepted as a field-view binding, writes to that exact field invalidate the view, distinct direct-field writes survive, and value copies remain ordinary immutable locals.

Session S update: local direct element views now have the matching checked failure mode. `let view = borrow list[0]` is accepted as an element-view binding, `list_append(change list, item)` invalidates outstanding element views for that list, value copies remain ordinary immutable locals, and the H0806/H0807 overlap fixture proves active-iteration mutation conflict wins on the append line. Retain, nested/general indexes, stale views from richer list APIs, and internal references remain future work.

Carry a mandated contracts item too: Predicate v1 should come before the
contract-check-mode ADR because the ledger has three predicate-expressiveness
records against one check-mode record. Its first scope should be pre-state
references and small collection/list predicates demanded by current probes, with
H0701 continuing to mark prose outside the checked grammar.

friction:
  program: src/run.rs predicate evaluation (Session T review, minor note)
  wanted: a validated contract predicate whose operand has the wrong runtime type (list_len over a non-list) to fail with a typed, blamed diagnostic
  forced: it generic-traps with exit 2, the same pre-existing class as predicate v0 type confusion, because contract predicates are not type-checked before evaluation
  severity: awkward
  indicts: checker
  proposal: when contract predicates gain static type checking, the trap class retires; until then the class is recorded, not hidden

friction:
  program: docs/bakeoff/CORPUS.md:228
  wanted: accept the swap of definitely distinct `point.x` and `point.y` while rejecting a second write through a path such as `alias_to_x` that may name `point.x`
  forced: run the positive direct-field swap without any pinned overlapping-path/two-write misuse fixture; H0802 borrowed-write permission and H0807 stale-view use reject different behaviors
  severity: blocked
  indicts: ownership
  proposal: require a narrow overlapping-place/two-write alias repair with a stable blamed diagnostic before counting program 8 as implemented
  resolution: resolved in Session V for the exact local `let alias = change owner.field` straight-line slice by `examples/probes/writable_field_aliases.hum` and the pinned `fixtures/ownership_check/session_v_program8_overlap_write_fail.hum`; H0808 names binding, conflict, last use, overlapping place, and repair, while H0809 keeps escape and unsupported forms fail-closed

Session U consolidation:

The records above are the accumulated friction ledger through Work Order 5.
Three-strike counts apply to active unresolved `indicts:` records; resolved
records stay as history.

| Indicted area | Active count | Records | Three-strike result |
| --- | ---: | --- | --- |
| ownership | 3 | Transaction-shaped linear marker; internal-reference `from parser.buffer`; Program 8 overlapping-alias/two-write misuse. | Triggered. Sessions R and S paid the local stale-view records, but neither H0802 nor H0807 implements the pinned overlapping-write rejection. |
| contracts | 4 | Contract-check-mode for `divide`; content-conditional count remainder from `word_count`; list-content remainder from `builder_demo`; text-literal equality from `element_views`. | Triggered. Per the Session Q ordering, with Predicate v1 shipped, the mandated item is now the contract-check-mode ADR (backlog 8). The three vocabulary remainders feed Predicate v2 (backlog 9), not a new mandate. |
| stdlib | 1 partial | Append shipped in Session P; retain, capacity/profile behavior, and richer list surface remain. | No trigger. |
| types | 0 | Direct field mutation resolved the replacement-literal record in Session O. | No trigger. |
| core-body-grammar | 1 | Standalone close/effect calls must be bound to throwaway locals. | No trigger yet. |
| checker | 1 | Contract predicate operand type confusion generic-traps (recorded above). | No trigger. |

Three-strike outcome:

- Ownership remains triggered at three active records. The local field- and
  element-view records are resolved, but Program 8 adds a distinct blocked
  overlapping-alias/two-write requirement beside the general linear marker
  and internal references. The next ownership work must not count H0802
  permission rejection or H0807 stale-view rejection as that missing gate.
- Contracts remain triggered at four active records, but the composition
  matters: three are vocabulary remainders explicitly routed to the
  Predicate v2 wishlist, and one is the check-mode question. The mandated
  item is therefore the contract-check-mode ADR, which is policy, not
  checker code. Work Order 6 issuance pays that item with
  [decision 0015](decisions/0015-adopt-classified-runtime-contract-policy.md).
  Decision 0015 changes no runtime behavior and does not erase the three
  active Predicate v2 vocabulary records; Session AE must reapply the rule.

Session U recommendation: Work Order 6 planning must first pay the renewed
ownership trigger with the narrow Program 8 overlapping-place/two-write alias
repair, then pursue the adoption-critical first IO capability slice with
backlog items 18 (error chains) and 19 (entry as capability root) designed
inside it. Contracts were triggered at Session U; Work Order 6 carries the
mandated policy item as decision 0015 while deliberately deferring the exact
three Predicate v2 vocabulary records through Session AE. Internal references
follow the Program 8 repair; effect polymorphism remains an explicit deferral.
The full candidate argument and deferral costs are in
[bakeoff/SCORECARD.md](bakeoff/SCORECARD.md) under "Work Order 6 recommendation
after Session U".

Session V ledger update:

The Program 8 overlapping-alias/two-write record above is the only record
resolved by Session V. The positive alias fixtures observe owner write-through,
distinct direct-field access, two distinct live aliases, and sequential
same-field aliases after last use. The pinned direct-write misuse and the
direct-read/second-alias misuses reject with H0808; escape, control-flow,
permission-wrapper, owner-rebinding, and visible-name-collision misuses reject with H0809. H0802
remains permission evidence only and wins before overlap analysis when alias
acquisition itself lacks mutation authority.

Reapplying the three-strike rule to active unresolved records now gives:

| Indicted area | Active count | Records | Three-strike result |
| --- | ---: | --- | --- |
| ownership | 2 | Transaction-shaped general linear marker; internal-reference `from parser.buffer`. | Not triggered after Session V. The historical exact-three trigger was real and Session V paid its Program 8 record; this count does not authorize internal references outside Work Order 6's sequence. |
| contracts | 3 | Predicate v2 vocabulary: conditional content/count for `word_count`, list content for `builder_demo`, and text-literal equality for `element_views`. | Triggered. Decision 0015 resolves the separate check-mode policy record but changes no runtime behavior; its unimplemented classifier remains an honesty lock and backlog item, not a fourth friction record. Session AE must reapply the rule. |
| stdlib | 1 partial | Append shipped in Session P; retain, capacity/profile behavior, and richer list surface remain. | No trigger. |
| types | 0 | Direct field mutation resolved the replacement-literal record in Session O. | No trigger. |
| core-body-grammar | 1 | Standalone close/effect calls must be bound to throwaway locals. | No trigger yet. |
| checker | 1 | Contract predicate operand type confusion generic-traps. | No trigger. |

Decision 0014 honesty lock after Session V: the lock narrows only for local
unannotated direct-field writable aliases with a straight-line last-syntactic-use
lifetime. General aliases, stored or passed aliases, nested/element aliases,
internal references, broad disjoint-field projection, broad flow-sensitive
borrowing, full ownership safety, borrow soundness, and memory-safety
completeness remain unimplemented and unclaimed.
