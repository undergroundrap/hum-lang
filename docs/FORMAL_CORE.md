# Hum Formal Core

Date: 2026-07-06

## Purpose

Hum needs a precise executable core before it needs more surface syntax.

The formal core is the small language that Hum source lowers into after parsing,
intent checking, and type checking. It is the thing we can specify, test, fuzz,
interpret, compile, and eventually verify.

This document is not a proof. It is the boundary that keeps Hum honest. The
machine-readable version of the first Core Hum boundary is
[HUM_CORE_CONTRACT_SCHEMA.md](HUM_CORE_CONTRACT_SCHEMA.md), emitted by
`hum core-contract --format json`.

## Brutal Thesis

Readable syntax is not enough.

If `does:` can mean whatever an agent or compiler pass feels like, Hum fails.

Every executable phrase must eventually lower to a small set of precise
operations:

```text
surface Hum -> parsed AST -> checked intent -> checked resolution -> type environment -> declaration type check -> typed core -> backend IR
```

The surface language may be friendly. The core must be boring.

## Core Principles

1. The core is smaller than the surface language.
2. Every core operation has explicit inputs, outputs, effects, and failure
   behavior.
3. Mutation is only possible through a declared mutable place.
4. Failure is a typed value path, not hidden control flow.
5. Allocation, blocking, IO, time, randomness, unsafe, and foreign calls are
   visible effects.
6. Profiles can forbid core operations.
7. Agents are not part of the meaning of the program.
8. Backends preserve core meaning or they are wrong.

## What The Core Is

The core is the first executable subset of Hum.

It should include:

- literals
- local bindings
- explicit mutable locals
- assignment to mutable places
- arithmetic and comparisons
- boolean logic
- records
- enums or tagged variants
- `maybe`
- `Result`
- calls
- `if`
- exhaustive `match`
- simple loops
- typed `return`
- typed `fail`
- checked effects
- debug-mode contract checks

The core should not include every nice thing the surface language may eventually
offer.

## What Stays Out At First

Milestone 1 should exclude:

- macros
- async/await
- closures
- inheritance
- operator overloading
- reflection
- compile-time metaprogramming
- user-defined effects
- full generics
- unsafe
- foreign calls
- atomics
- lock-free data structures
- borrow checking beyond simple local ownership experiments

Some of these will matter. They do not belong in the first core.

## Core Values

Starter values:

```text
unit
true
false
integer
unsigned integer
text
bytes
record value
variant value
maybe value
result value
```

Open question: floating point should exist in normal Hum, but safety and replay
profiles need a stricter floating-point policy before it becomes part of the
trusted core.

## Core Types

Starter types:

```text
Unit
Bool
Int
UInt
Text
Bytes
record { field: Type, ... }
variant { Case(Type), ... }
maybe Type
Result Type, Error
```

Rules:

- No implicit null.
- No implicit numeric narrowing.
- No assignment expression.
- No hidden conversions across signedness.
- No truthy or falsy values.
- Records are values with named fields.
- Variants require exhaustive `match`.

## Core Places

A place is something that can be read or written.

Core places:

```text
local
record.field
index into checked collection
store entry, later milestone
```

Only places declared mutable can be written:

```text
change count: UInt = 0
set count = count + 1
```

Surface `changes:` blocks declare external mutable places. Local `change`
declares local mutable places.

The broader state doctrine is [STATE_MODEL.md](STATE_MODEL.md), emitted as
`hum.state_model.v0` by `hum state-model --format json`. Checked source place
links begin in [HUM_RESOLVE_SCHEMA.md](HUM_RESOLVE_SCHEMA.md), emitted as
`hum.resolve.v0` by `hum resolve --format json`. Core Hum must preserve state
facts for immutable values, mutable locals, places, stores, ownership, borrows,
linear resources, shared state, and external authority before those features
become executable guarantees.

## Core Expressions

Starter expressions:

```text
literal
name
field read
checked index read
record construction
variant construction
task call
built-in operation call
unary operation
binary operation
if expression
match expression
try expression
```

`try expression` unwraps a `Result` in a task that declares compatible failure.
It is not punctuation magic. It lowers to a `match` on success/failure.

The first executable built-in operation call is `list_append(change list,
item)`. It mutates the named list place and returns `Unit`; it is not a
general list standard library surface.

The first local field-view repair recognizes `let view = borrow record.field` as a view of one direct field place. A later `set record.field = value` invalidates that view; unrelated direct field writes do not. Session S adds the matching direct element-view slice: `let view = borrow list[0]` is a view of a direct numeric list element, and later `list_append(change list, item)` invalidates outstanding element views for that list. This is exact-place and list-growth bookkeeping, not general indexing, alias inference, or lifetime proof.

## Core Statements

Starter statements:

```text
let name: Type = expression
change name: Type = expression
set place = expression
expression statement
if condition { block } else { block }
match expression { cases }
while condition { block }
loop { block }
for each item in collection { block }
for index i from start until end { block }
return expression
fail expression
break
continue
```

`for each` and `for index` may lower to checked iterator or index loops. The
lowering must preserve bounds behavior and cost facts.

## Core Tasks

A task is the core callable unit.

Surface:

```text
task create_session(user: User) -> Result Session, SessionError {
  uses:
    sessions

  changes:
    sessions

  needs:
    user is verified

  ensures:
    session belongs to user

  fails when:
    session store is full

  does:
    ...
}
```

Core meaning:

```text
task(
  name,
  params,
  result type,
  declared effects,
  preconditions,
  postconditions,
  failure cases,
  body
)
```

The body cannot read, write, allocate, block, call, fail, or access time/random
outside the declared interface.

## Contracts In The Core

Contracts should lower into obligations.

```text
needs:    caller obligation and debug entry check
ensures:  callee obligation and debug exit check
keeps:    invariant obligation
changes:  mutation permission
uses:     read/effect permission
fails:    allowed failure variants
cost:     static or benchmark obligation
protects: safety/security obligation
trusts:   explicit unchecked assumption
```

Milestone 1 can check only a small part of this. Current executable predicate v0 applies to `needs:` and `ensures:` only: one canonical comparison over task parameters, direct field paths rooted in task parameters, integer/bool literals, arithmetic operands, and `result` or direct field paths rooted in `result` in `ensures:`. Lines outside that grammar remain graph-visible prose and produce an unchecked-contract warning under `hum run`; they are not errors and they are not silently treated as proof.

The graph should still preserve the obligations so future tools do not have to rediscover them.

## Effects

Core effects start as a closed set:

```text
read
change
allocate
free
time
random
file
network
block
spawn
unsafe
foreign
panic
```

Rules:

- Effects are inferred from the body.
- Effects are checked against `uses:`, `changes:`, `allocates:`, `fails when:`,
  and profiles.
- Effects are emitted into `hum graph`.
- A profile may forbid an effect.

## Failure

Failure is explicit.

```text
fail LoginError.expired
```

Rules:

- A task may fail only with a declared failure type.
- Callers must handle or propagate failure.
- `try` propagates only compatible failure values.
- There are no exceptions in the core.
- Panics are not ordinary failure; they are contract bugs or profile-defined
  fail-stop behavior.

## Loops

Loops must be analyzable.

Core loop forms:

```text
while condition { block }
loop { block }
for each item in collection { block }
for index i from start until end { block }
```

Critical loops may carry:

```text
keeps:
changes:
watch for:
cost:
```

Rules:

- `check: compile` cost claims can reject visible mismatches.
- Safety and realtime profiles can require loop variants, watchdogs, or measured
  bounds.
- `for index` uses checked bounds.
- No C-style `for` enters the core.

## Profiles And The Core

Profiles are filters over the core.

Example:

```text
profile hard realtime
```

can forbid:

- `allocate`
- `block`
- `spawn` without a bounded scheduler
- unbounded loops
- hidden `panic`
- background reclamation

```text
profile safety critical
```

can require:

- no hidden allocation
- explicit failure policy
- checked numeric operations
- traceable contracts
- no unsafe without review packet

The key rule: profiles restrict core operations, not prose.

## Semantic Graph Requirements

`hum graph` should eventually expose:

- lowered core task identity
- typed params and result
- declared effects
- inferred effects
- contract obligations
- failure variants
- mutable places
- reads and writes
- loop nodes
- call nodes
- profile restrictions
- proof/test obligations
- backend-preservation hooks

Milestone 0 graph output can stay compact. The schema should reserve room for
these facts before tools depend on guesses.

## Backend Preservation

Backends may optimize. They may not change meaning.

A backend pass must preserve:

- observable return/failure behavior
- declared effects
- mutation permissions
- memory safety assumptions
- profile restrictions
- debug contract behavior where enabled
- source mapping for diagnostics, profilers, and debuggers

If a backend cannot explain how a transformation preserves the core meaning, it
is not stable-ready.

## Relationship To Agents

Agents may help write surface Hum.

Agents do not define core Hum.

Agent-generated code must pass through:

```text
parse -> check -> resolve -> type -> lower -> test/fuzz/prove/bench
```

The compiler should hand agents the formal core facts through `hum graph`,
diagnostics, and generated obligations.

## First Executable Slice

The first executable Hum should run only this:

```text
task add(a: Int, b: Int) -> Int {
  ensures:
    result == a + b

  does:
    return a + b
}
```

Then:

```text
task divide(a: Int, b: Int) -> Result Int, MathError {
  needs:
    b != 0

  fails when:
    b is zero

  does:
    if b == 0 {
      fail MathError.divide_by_zero
    }

    return a / b
}
```

Then:

```text
task count_completed(tasks: List Task) -> UInt {
  cost:
    time: O(tasks)
    space: O(1)
    check: compile

  does:
    change count: UInt = 0

    for each task in tasks {
      if task.done {
        set count = count + 1
      }
    }

    return count
}
```

If these cannot be parsed, checked, lowered, interpreted, tested, and explained,
Hum is not ready for bigger promises.

## Open Questions

1. Should `does:` start as a strict expression language with friendly keywords,
   or allow controlled phrases that lower only when unambiguous?
2. Does Milestone 1 include `Text` and `Bytes`, or only numeric and record values?
3. Are `maybe` and `Result` built-in forms at first, or library types with
   compiler knowledge?
4. How much local ownership checking belongs in the first executable core?
5. What is the smallest useful effect checker?
6. Should debug contract checks be interpreted first, or emitted as generated
   Hum tests?
7. Which backend proves the core fastest: interpreter, Cranelift, or Rust
   lowering?

## Acceptance Gate

A feature can enter stable Hum only when this document can answer:

```text
What core operation does it lower to?
What types does it require?
What effects can it create?
What profiles allow or forbid it?
What diagnostics explain misuse?
What graph nodes represent it?
What tests, fuzzers, proofs, or benchmarks protect it?
```

If those answers are missing, the feature remains experimental.

## Brutal Assessment

Hum should be warm at the surface and cold in the core.

The source can read like a careful engineer explaining intent. The core must read
like a small machine for promises, effects, state, failure, and control flow.

That is how Hum stays friendly without becoming fuzzy.
