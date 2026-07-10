# Hum State Model

Date: 2026-07-07

## Purpose

State management is a central Hum design axis.

Most systems bugs are state bugs: hidden mutation, aliasing mistakes, data races,
leaked resources, double frees, forgotten rollbacks, stale caches, untracked
external authority, and state that changes outside the reader's mental model.

Hum's answer is not "never mutate." Systems programming needs mutation. Hum's
answer is:

```text
state is visible, permissioned, profile-aware, and evidence-producing
```

## Current Machine-Readable Surface

`hum state-model` and `hum state-model --format json` expose the V0 state model
contract.

The catalog schema is `hum.state_model.v0`; permission entries use
`hum.state_permission.v0`.

V0 mode is `contract_only_partial_declared_mutation_check`. This means the model
documents the intended state rules and names the small current checker behavior:
`set name = ...` must target a local `change name: ...` or a matching
`changes:` entry.

The `hum state-model` V0 contract document does not itself implement borrowing,
lifetime inference, move checking, concurrency, memory-order semantics, garbage
collection, executable semantics, optimizer behavior, or allocation placement.
`hum ownership-check` now implements narrow ordinary move checks, the first
Transaction-shaped linear-resource path check, and Session V's exact local
direct-field writable-alias/overlap slice; it is not a complete ownership,
borrow, alias, or memory-safety checker.

The schema document is [HUM_STATE_MODEL_SCHEMA.md](HUM_STATE_MODEL_SCHEMA.md).

## State Thesis

Hum should make the common path safe and boring:

- immutable values by default
- explicit local mutation with `change`
- explicit writes with `set`
- explicit external mutation with `changes:`
- explicit reads and authority with `uses:`
- explicit named state with `store`
- explicit resource policy with profiles
- explicit future ownership, borrowing, and linear-resource facts

The syntax should feel simple, but the compiler facts should be strict enough
for a borrow checker, effect checker, debugger, verifier, optimizer, and agent
to agree on what can change.

## State Kinds

### Immutable Value

Ordinary data should be immutable unless the source asks for mutation.

```hum
let count: UInt = 0
```

Immutable values can be freely read, passed, printed, tested, and reasoned about.
They are the paved road.

### Mutable Local

Task-local mutation must be declared:

```hum
change count: UInt = 0
set count = count + 1
```

This keeps mutation scannable and gives the future checker a clear permission
edge. A `set` without a matching `change` or `changes:` entry is already a
Milestone 0 diagnostic.

### Place

A place is a readable or writable location:

```text
local
record.field
checked index
future store entry
```

Core Hum owns the exact place model. Backends must preserve place identity,
source spans, aliasing facts, and mutation permission or report loss.

The first writable alias is deliberately bounded: `let alias = change
owner.field` names one direct field in one straight-line task body. It writes
through, ends after its last syntactic use, accepts definitely distinct direct
fields, and rejects live overlap with H0808. H0809 keeps escape, storage,
permission wrapping, alias rebinding, live-owner rebinding, nested/element
aliases, already-visible-name shadowing, and control-flow use outside this
slice.

### Store

A `store` is named state with purpose and policy, not a casual global variable.

```hum
store sessions: map SessionId -> Session {
  why:
    remember active sessions
}
```

Future store declarations should carry ownership, durability, privacy,
concurrency, target, and profile policy.

### Linear Resource

Handles, sockets, locks, transactions, raw buffers, foreign resources, and
capabilities should eventually be linear resources: they must be consumed,
closed, committed, rolled back, or transferred exactly once.

This is where Hum should learn from linear types without making every ordinary
value painfully linear.

### Shared State

Shared mutable state is forbidden until Hum has a checked sharing form.

Future allowed forms may include actors, locks, atomics, regions, arenas, tasks,
or runtime-owned queues, but each must expose ownership, synchronization,
failure, replay, and profile facts.

### External State

Files, processes, network endpoints, clocks, randomness, devices, environment
variables, host APIs, and platform authority are state too.

Hum must treat them as effects and capabilities, not implementation details.

## Permission Words

Hum should teach state in source terms:

- `read`: observe without mutation or ownership transfer
- `own`: hold responsibility for a value or resource
- `borrow`: temporarily read without owning
- `change`: exclusively mutate a place
- `consume`: use a linear resource exactly once
- `share`: access through a checked shared-state form

These are human-facing names. The compiler may lower them to stricter internal
facts, but diagnostics should explain state errors in these terms first.

## Design Rules

1. Immutable values are the default paved road.
2. Mutation requires source-visible permission.
3. A store is named state with policy, not a casual global variable.
4. Shared mutable state is forbidden until a checked sharing form exists.
5. Linear resources require exactly-once close, commit, rollback, consume, or transfer.
6. Borrowing is permission to observe or change for a bounded region, not hidden copying.
7. External state is an effect and a capability, not an implementation detail.
8. Profiles may make the state model stricter, never looser without evidence.

## Why This Saves Headaches

State design affects every hard feature:

- type checking
- ownership and borrowing
- resource management
- async and concurrency
- unsafe boundaries
- foreign calls
- stdlib containers
- debugging
- profiling
- verification
- package and runtime profiles
- agent repair workflows

If state is not central, every later subsystem invents its own half-model.

## Near-Term Work

1. Keep `hum state-model --format json` in preflight.
2. Keep `hum resolve --format json` in preflight as the first checked source-place link report.
3. Add graph links for local mutable declarations and `set` targets once the resolver shape is stable.
4. Add an effect report that compares declared `uses:`/`changes:` with inferred reads and writes.
5. Keep widening ordinary move and no-use-after-move fixtures only when the executable core can demonstrate them.
6. Broaden linear-resource fixtures from Transaction-shaped locals to handles, locks, and capabilities after the v0 path checker earns it.
7. Delay concurrency syntax until the state model can explain ownership and memory-order facts.

## Brutal Rule

No new Hum feature is allowed to hide state.

If it reads, writes, owns, borrows, consumes, shares, allocates, persists, caches,
opens, closes, commits, rolls back, sends, receives, waits, or observes external
authority, the state model must have a place for it.
