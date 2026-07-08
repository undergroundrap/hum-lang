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

friction:
  program: examples/probes/word_count.hum:8
  wanted: state that the result equals the number of matching words in the literal list
  forced: hard-code `result == 2` because predicate v0 has no collection count, quantifier, or helper-call contract vocabulary
  severity: awkward
  indicts: contracts
  proposal: frequency-rank collection count predicates before growing predicate grammar v1

friction:
  program: examples/probes/task_list_flow.hum:58
  wanted: append a new work item to an existing list as the add operation
  forced: spell the post-add list as a fresh list value because the current executable subset has no list append or Vec API
  severity: awkward
  indicts: stdlib
  proposal: design the smallest list operation surface before richer state probes

friction:
  program: examples/probes/task_list_flow.hum:15
  wanted: update one record field (`done`) while preserving the rest of the work item
  forced: construct a replacement record literal with every field repeated
  severity: verbose
  indicts: types
  proposal: put record update syntax through the ownership bake-off instead of adding it ad hoc

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

0014 honesty locks after applying the ledger: all remain. Hum has narrow
checked ownership facts for local moves, permissions, Transaction-shaped linear
resources, and bare-parameter returned views. It still has no full ownership
safety claim, borrow-soundness claim, memory-safety proof, safety-critical
readiness claim, disjoint-field projection, internal-reference support, broad
flow-sensitive borrowing, concurrency ownership model, or general linear
resource marker.
