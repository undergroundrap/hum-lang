# Hum Performance Contracts

Date: 2026-07-06

## Thesis

Hum should make performance claims enforceable.

A programmer should be able to say:

```text
cost:
  time: O(1)
  space: O(1)
  allocates: nothing
  check: compile
```

and if the function clearly scans a collection, the compiler should reject it.

That is not crazy. It is the performance equivalent of a type or borrow claim:
the programmer made a promise, and the compiler caught a contradiction.

## Brutal Truth

Not all performance can be proven statically.

Hum can often check:

- obvious loops
- nested loops
- recursion without a bound
- collection operations with known complexity
- allocation in loops
- IO in loops
- lock acquisition in loops
- calls whose `cost:` is known

Hum cannot fully prove every performance property at compile time because:

- halting and termination are hard in the general case
- dynamic dispatch can hide target code
- data-dependent loops may depend on runtime values
- caches and branch predictors are hardware-specific
- benchmarks vary by machine, OS, compiler, and input distribution

So Hum needs enforcement tiers instead of pretending every benchmark is a type.

## Enforcement Tiers

### `check: graph`

Record the claim in the semantic graph. This is Milestone 0.

```text
cost:
  time: O(tasks)
  check: graph
```

### `check: warn`

Warn when the compiler sees a likely contradiction.

```text
cost:
  time: O(1)
  check: warn
```

If the body loops over `tasks`, the compiler warns but still builds.

### `check: compile`

Fail compilation when the compiler can disprove the claim.

```text
cost:
  time: O(1)
  allocates: nothing
  check: compile
```

This should reject:

- direct loops over unbounded collections
- calls to tasks with higher declared cost
- allocation when `allocates: nothing` is declared
- IO when `io: none` is declared

It should not fail just because the compiler cannot prove a difficult claim. For
that, Hum needs a stricter tier.

### `check: prove`

Fail compilation unless the compiler or verifier can prove the claim.

```text
cost:
  time: O(1)
  check: prove
```

This is powerful but expensive. It should be used for critical code, not every
line of a beginner program.

### `check: bench`

Fail the benchmark profile when measured performance misses the target.

```text
benchmarks:
  target: release-x64-baseline
  input: 1_000_000 sessions
  p95 lookup: under 200 ns
  memory: under 32 bytes per session
  check: bench
```

This is not pure compile time. It belongs in `hum bench`, CI, release builds, or
profile-gated builds.

## Suggested Syntax

For complexity and static resources:

```text
cost:
  time: O(items)
  space: O(1)
  allocates: nothing
  io: none
  locks: none
  check: compile
```

For measured performance:

```text
benchmarks:
  target: release-x64-baseline
  input: 100_000 items
  p95 time: under 5 ms
  peak memory: under 10 MB
  check: bench
```

For optimization intent:

```text
optimizes:
  p95 latency
  memory density
  predictable tail behavior
```

These are separate because they answer different questions:

- `cost:` says what shape the code should have.
- `benchmarks:` says what measured result must hold.
- `optimizes:` says what matters when tradeoffs conflict.

## Example Compile-Time Rejection

Source:

```text
task has completed task(tasks: list Task) -> Bool {
  cost:
    time: O(1)
    check: compile

  does:
    for each task in tasks {
      if task.done {
        return true
      }
    }

    return false
}
```

Diagnostic:

```text
error[HUM-COST-001]: declared O(1), but body loops over `tasks`

blame:
  cost:
    time: O(1)

why:
  `for each task in tasks` makes runtime grow with the number of tasks.

fix:
  change cost to `O(tasks)`, or maintain a separate completed-count index.
```

That is exactly the kind of clean-code enforcement Hum should provide.

## Space-Time Tradeoffs

Williams' 2025 square-root-space simulation result is a design signal, not a
magic compiler feature. It says that for deterministic multitape Turing machines,
time-bounded computation has much stronger space-efficient simulation than was
known before. It does not say arbitrary real-world programs should silently be
recomputed under memory pressure.

Hum should still learn the right lesson: resource claims are multidimensional.
A future `cost:` vocabulary should be able to distinguish time, peak space,
scratch memory, allocation policy, recomputation, caching, and checkpointing:

```text
cost:
  time: O(work)
  peak space: O(boundary)
  scratch: bounded
  allocates: nothing in hot path
  recomputes: pure derived values only
  caches: bounded by profile memory budget
  check: bench
```

Recomputation is safe only for pure, deterministic, replayable work whose
effects are visible in the semantic graph. It must not replay IO, time,
randomness, mutation, or external authority unless the source explicitly captures
that boundary.

See [research/2026-07-07-time-space-simulation.md](research/2026-07-07-time-space-simulation.md).

## Static Benchmarks, Carefully Defined

"Statically typed benchmarks" is a good instinct but a dangerous phrase.

Better term: performance contracts.

Some performance contracts are static:

- Big-O shape
- allocation permissions
- IO permissions
- lock permissions
- call graph cost propagation

Some performance contracts are measured:

- latency
- throughput
- memory footprint
- cache misses
- branch misses
- binary size

Hum should treat both as first-class, but not pretend measured facts are the same
as type facts.

## Relationship To Borrow Checking

The borrow checker prevents invalid aliasing and mutation patterns before the
program runs.

A cost checker should prevent invalid performance and resource patterns before
the program ships.

Similar spirit:

- make invisible danger explicit
- reject contradictions early
- teach the programmer the better shape

Different limits:

- ownership has a stronger local model
- performance often depends on input sizes and hardware
- benchmarks require controlled measurement

Hum should do both, but be honest about which claims are proven and which are
measured.

## What This Unlocks

Performance contracts make apps better because they catch drift:

- a helper call becomes expensive
- a lookup turns into a scan
- a hot path starts allocating
- a retry loop loses its bound
- a refactor changes O(n) into O(n^2)
- a data structure stops matching its declared intent

Most languages discover this in production. Hum should discover it at compile,
test, benchmark, or CI time.

## Brutal Warning

Do not make every function require `check: prove`.

That would turn Hum into a language people admire and avoid. The default should
be approachable. Critical paths should opt into hard enforcement.

The win is not maximal strictness everywhere. The win is making strictness easy
to request where it matters.