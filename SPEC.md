# Hum Language Spec Draft

Version: 0.0.1
Date: 2026-07-06
Status: design draft

## Mission

Hum is an intent-first systems programming language.

It is designed for code that needs the power of Rust and C++, the readability of
Python, the explicitness of Zig, contract depth from verification languages, and
compiler-generated context that humans and coding agents can trust.

The central rule:

```text
Humans write intent.
The compiler enforces promises.
Agents fill, review, optimize, and explain from structured context.
```

See [docs/LANGUAGE_REFERENCE.md](docs/LANGUAGE_REFERENCE.md) for the
traditional reference spine, [docs/LANGUAGE_CONSTITUTION.md](docs/LANGUAGE_CONSTITUTION.md)
for the rules Hum must not violate, and [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md)
for the cybersecurity threat model.

## Design Principles

1. Readability beats cleverness.
2. One obvious way is better than many clever ways.
3. Effects are part of the interface.
4. Memory safety is the default.
5. Unsafe code must be visible, bounded, and justified.
6. Allocation, blocking, networking, randomness, time, mutation, and IO are never hidden.
7. Contracts should help the compiler, verifier, test generator, optimizer, reviewer, and agent.
8. The compiler should generate semantic context instead of asking agents to infer it from raw text.
9. Performance claims must be measurable.
10. The language should stay small enough to teach.

## Non-Goals

- Do not clone C++.
- Do not require headers.
- Do not make source code plain English.
- Do not hide effects behind innocent-looking calls.
- Do not make macros the first escape hatch.
- Do not accept undefined behavior as normal systems programming.
- Do not depend on an AI model to make code correct.

## Files And Modules

Hum does not use C/C++ style headers. A `.hum` file is the source of truth.

The compiler produces derived artifacts:

```text
source.hum       human source
source.interface public typed interface
source.graph     AST, call graph, ownership graph, effect graph
source.proof     proof and test obligations
source.optimized backend IR and machine code
```

Imports are capability-aware:

```text
use std.network
use std.time as clock
use std.random.secure
```

A task cannot use an imported capability unless its contract declares it.

## Syntax Shape

Hum uses braces for stable machine parsing and newline-oriented statements for
human readability.

Semicolons are not part of normal source.

```text
task add(a: Number, b: Number) -> Number {
  does:
    return a + b
}
```

## Top-Level Forms

### App

An `app` describes an executable program and its top-level capabilities.

```text
app Counter {
  why:
    count button presses and show the total

  uses:
    screen
    storage

  starts with:
    count: Number = 0
}
```

### Type

A `type` describes data and its invariants.

```text
type Session {
  user: UserId
  expires_at: Time

  keeps:
    expires_at is after created_at
}
```

### Store

A `store` describes data structure intent. The compiler chooses or verifies the
implementation strategy.

```text
store sessions: map SessionId -> Session {
  expects:
    up to 20 million active sessions

  optimizes:
    lookup speed
    memory density
    collision resistance

  protects:
    session ids cannot be guessed
    expired sessions cannot be reused

  hides:
    hash layout
    bucket metadata
    resize strategy
}
```

### Task

A `task` is Hum's function-like unit.

```text
task name(input: Type) -> Output {
  why:
  uses:
  changes:
  needs:
  ensures:
  fails when:
  does:
}
```

Tiny tasks may omit most blocks:

```text
task square(x: Number) -> Number {
  does:
    return x * x
}
```

Critical tasks should declare deeper context:

```text
task name(input: Type) -> Output {
  why:
  uses:
  changes:
  creates:
  deletes:
  needs:
  assumes:
  ensures:
  keeps:
  protects:
  trusts:
  watch for:
  cost:
  avoids:
  tradeoffs:
  allocates:
  calls:
  fails when:
  optimizes:
  tests:
  benchmarks:
  proves:
  does:
}
```

### Test

A `test` is executable evidence for a contract, edge case, or behavior.

```text
test add task rejects empty title {
  covers:
    add task fails when title is empty

  does:
    expect add task("") fails with TaskError.empty_title
}
```

Top-level tests may be unit tests, property tests, fuzz tests, regression tests,
or generated contract tests. Regression tests use `test ... regression`, not a
separate top-level keyword.

See [docs/TESTING_STRATEGY.md](docs/TESTING_STRATEGY.md).

## Contract Blocks

### `why:`

Human and agent-facing purpose. It should explain why the task exists.

### `uses:`

Read dependencies and external capabilities.

Examples:

```text
uses:
  user.id
  clock.now
  random.secure
  sessions
```

### `changes:`

Write permissions. If a task writes something not listed here, compilation fails.

### `creates:`

New resources, records, files, handles, threads, tasks, or allocations.

### `deletes:`

Resources this task may remove, close, free, revoke, or invalidate.

### `needs:`

Preconditions required before the task can run.

### `assumes:`

Facts this task depends on but may not be able to prove locally.

### `ensures:`

Postconditions the task must satisfy on success.

### `keeps:`

Invariants that must remain true during execution.

### `protects:`

Security, privacy, memory-safety, or correctness properties being defended.

### `trusts:`

External authorities or unchecked assumptions.

Examples:

```text
trusts:
  random.secure is cryptographically strong
  operating system enforces file permissions
```

### `watch for:`

Known edge cases and future hazards.

This is not just a comment. The compiler and tools should use it to generate
tests, fuzz targets, review prompts, and diagnostics.

### `cost:`

Expected time, space, allocation, IO, lock, or hardware cost.

Examples:

```text
cost:
  time: O(items)
  space: O(1)
  allocates: nothing
  check: compile
```

In early milestones, `cost:` is emitted into the semantic graph. Later, the
compiler should compare it against loop structure, calls, allocation, profiling,
and benchmarks.

### `avoids:`

Implementation shapes this task should not use.

Examples:

```text
avoids:
  nested scan over users and sessions
  allocation inside loop
  network call inside loop
```

This is the compiler-grade form of a beginner saying what the code "dislikes."

### `tradeoffs:`

Accepted engineering compromises.

Examples:

```text
tradeoffs:
  linear scan is acceptable because the list is small
  dense storage is preferred over pointer stability
```

### `benchmarks:`

Measured performance requirements for named build or hardware profiles.

Examples:

```text
benchmarks:
  target: release-x64-baseline
  input: 100_000 items
  p95 time: under 5 ms
  peak memory: under 10 MB
  check: bench
```

Benchmarks are first-class, but hardware-dependent. They should fail benchmark
or release profiles, not pretend to be pure type facts.

See [docs/PERFORMANCE_CONTRACTS.md](docs/PERFORMANCE_CONTRACTS.md).

### `allocates:`

Memory behavior and allocation permissions.

Examples:

```text
allocates:
  nothing
```

```text
allocates:
  one session record
  no unbounded temporary buffers
```

### `calls:`

Important dependencies and allowed task calls. This supports review, impact
analysis, sandboxing, and agent navigation.

### `regression:`

Regression history for a test. This records the bug, issue, incident, or failure
mode the test prevents from returning.

Examples:

```text
regression:
  found when title "   " was accepted as nonempty
```

### `covers:`

Test coverage intent. This names the task, branch, contract, risk, or diagnostic
a test exists to exercise.

Examples:

```text
covers:
  add task fails when title is empty
  add task watch for title may be only spaces
```
### `fails when:`

Explicit failure modes. Hum errors are typed and cannot silently disappear.

### `optimizes:`

Optimization priorities. These should map to benchmark or profiling checks.

### `tests:`

Required tests, generated test obligations, or fuzz cases. Use top-level `test`
blocks for executable tests.

### `proves:`

Formal or semi-formal obligations.

### `does:`

Executable body.

## Comments And Intent

Hum should eliminate most comments by giving important meaning a checked place
to live.

Comments are still allowed, but critical claims should move into contract
blocks:

```text
why:
protects:
watch for:
needs:
ensures:
optimizes:
```

If a sentence affects correctness, security, performance, ownership, allocation,
failure behavior, or review, it should eventually become compiler-visible.

## Executable Core And Control Flow

Hum should provide the constructs programmers expect, but with readable and
checkable forms.

The precise executable subset is tracked in
[docs/FORMAL_CORE.md](docs/FORMAL_CORE.md). Surface Hum may grow friendlier, but
every executable phrase must lower into that smaller core or remain
experimental.

Starter executable forms:

```text
let name: Type = value
change name: Type = value
set name = value
if condition { ... } else { ... }
match value { ... }
while condition { ... }
loop { ... }
for each item in items { ... }
for index i in 0..count { ... }
return value
fail error
break
continue
```

Hum should not use C-style `for (init; condition; step)` loops.

Critical loops may carry nested intent:

```text
while attempts < max attempts {
  keeps:
    attempts <= max attempts

  changes:
    attempts

  does:
    set attempts = attempts + 1
}
```

Loop `keeps:` clauses become invariants. Loop `changes:` clauses restrict local
mutation. Loop `watch for:` clauses should generate tests and review prompts.

See [docs/CORE_LANGUAGE_SHAPE.md](docs/CORE_LANGUAGE_SHAPE.md) for the broader
core construct plan.

## Blame Semantics

Hum contracts should tell the compiler who is responsible when a promise fails.

Draft blame map:

```text
needs:      caller did not satisfy the precondition
ensures:    task did not satisfy the postcondition
keeps:      body or mutation broke an invariant
protects:   security property was violated
trusts:     external trust boundary was wrong or underspecified
changes:    task mutated something it did not declare
allocates:  task exceeded its memory policy
```

Every diagnostic should include the failed block, source span, blame target, and
a machine-readable repair hint.

## Types

Hum is statically typed with local inference.

Primitive starter types:

```text
Bool
Int
UInt
Float
Number
Bytes
Text
Time
Duration
```

No value is nullable by default.

Optional values are explicit:

```text
maybe User
```

Fallible values are explicit:

```text
Result User, LoginError
```

## Ownership And Memory

Hum aims for Rust-level memory safety with more readable declarations.

Draft ownership modes:

```text
owned Buffer
borrow Buffer
change Buffer
shared Buffer
```

Meaning:

- `owned T`: this scope owns destruction or transfer.
- `borrow T`: read-only temporary access.
- `change T`: temporary mutation with exclusive access.
- `shared T`: thread-safe shared access through a checked type.

The compiler tracks lifetimes, aliasing, mutation, moves, and destruction.

## Unsafe Code

Unsafe code is allowed only inside visible unsafe boundaries.

```text
unsafe task copy bytes(from: Buffer, to: Buffer, count: Bytes) {
  why:
    move raw bytes without reading or writing outside either buffer

  needs:
    count <= from.length
    count <= to.length
    from and to do not overlap

  proves:
    no out of bounds read
    no out of bounds write

  does:
    copy count bytes from from to to
}
```

Unsafe tasks must declare `why:`, `needs:`, `proves:`, and `watch for:`.
The full unsafe gate is [docs/UNSAFE_POLICY.md](docs/UNSAFE_POLICY.md).

## Effects

Effects are part of task interfaces.

Starter effect set:

```text
read
change
allocate
free
network
file
time
random
block
spawn
unsafe
foreign
```

Effects are inferred from contracts and body, then checked against the declared
interface.

## Errors

Errors are values, not hidden control flow.

```text
task load profile(id: UserId) -> Result Profile, LoadProfileError {
  fails when:
    profile is missing
    storage is unavailable

  does:
    return profile from storage
}
```

Callers must handle failure explicitly.

## Concurrency

Concurrency must declare shared state and cancellation behavior.

```text
task serve requests(socket: Socket) {
  uses:
    network
    sessions

  changes:
    sessions

  watch for:
    request may be cancelled
    client may disconnect mid-write
    shared session state must not race

  does:
    for each request from socket {
      spawn handle request(request)
    }
}
```

Draft concurrency features:

- Structured tasks.
- No detached work without `creates: background task`.
- Cancellation is explicit.
- Shared mutation requires checked synchronization.
- Lock ordering can be declared and verified.

## Compile-Time Execution

Compile-time code must be sandboxed by capabilities.

```text
build task generate syscall table() {
  uses:
    files.read

  changes:
    generated.syscalls

  does:
    read syscall spec
    generate syscall table
}
```

Compile-time code cannot access network, system time, random, environment
variables, or files unless declared.

## Agent Context

The compiler should emit compact, queryable context:

```text
hum graph
hum explain task create session
hum effects
hum risks
hum tests
hum agent docs
```

Agents should use compiler-generated graphs instead of guessing from raw text.
The agent docs command should emit the exact grammar, diagnostic schema,
examples, and repair workflow for the installed compiler version.

## Standard Library Direction

The standard library should be designed from a 2026 systems perspective:

- dense maps and sets
- region and arena allocators
- explicit async and IO
- safe FFI boundaries
- cryptography with misuse-resistant APIs
- SIMD and hardware feature detection
- parser and protocol primitives
- graph storage and compact indexes
- deterministic replay and tracing hooks
- property tests, fuzzing, and benchmarks as first-class citizens

See [docs/STDLIB_STRATEGY.md](docs/STDLIB_STRATEGY.md) for the standard library build plan.

## Open Design Questions

1. Should Hum use indentation significance inside blocks, or braces plus labels only?
2. Should `does:` be a controlled natural syntax or a smaller expression language first?
3. Should ownership use Rust-like references internally or region-based ownership first?
4. How much proof should v0 attempt versus test/fuzz/contract checking?
5. Should the first compiler lower to Rust, C, MLIR, or a custom interpreter IR?
6. How should package names work when exact `hum` package names are unavailable in some registries?
7. Should standard library data structures expose chosen implementations or only intent contracts?

See [docs/BACKEND_STRATEGY.md](docs/BACKEND_STRATEGY.md) for the backend plan.

## First Milestone

The first useful milestone is not a full compiler. It is a parser and semantic
checker for task contracts.

Milestone 0 should:

1. Parse `.hum` files.
2. Build an AST.
3. Validate task and test block order and names.
4. Emit a JSON semantic graph.
5. Check that `does:` only changes items declared in `changes:`.
6. Generate a task interface summary.
7. Generate test skeletons from `needs:`, `ensures:`, and `watch for:`.
8. Emit `cost:`, `avoids:`, `tradeoffs:`, and `benchmarks:` into the semantic graph.
9. Parse top-level `test` blocks and connect them to `tests:` obligations.
10. Preserve test kinds such as `property`, `fuzz`, and `regression`.

See [docs/ROADMAP.md](docs/ROADMAP.md) for the build and teaching order.
