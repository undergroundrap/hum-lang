# 0010 Adopt Explicit State Model

Status: accepted
Date: 2026-07-07

## Context

Hum wants systems-level performance, high-assurance credibility, beginner
readability, agent-readable structure, and future ownership/borrowing checks.

Those goals meet at state management.

If state is informal, every later feature can smuggle in hidden mutation, hidden
allocation, hidden external authority, or aliasing behavior that the compiler,
debugger, verifier, profiler, and reader cannot agree on.

Rust proves that ownership and borrowing can prevent major classes of memory and
data-race bugs, but it also proves that the learning curve is real. Linear types
are valuable for resources, but making every ordinary value feel linear would
hurt the paved road. Garbage collection can be useful in some domains, but it is
not the default systems-language answer for Hum's strict profiles.

Hum needs an explicit state model before it adds serious executable semantics,
unsafe, FFI, async, concurrency, stdlib containers, or native backend claims.

## Decision

Hum adopts a source-visible state model:

- immutable values are the default paved road
- local mutation uses `change` and `set`
- external mutation must appear in `changes:`
- external reads and authority must appear in `uses:` or another checked section
- `store` is named state with policy, not a casual global variable
- ownership, borrowing, linear resources, and checked sharing are required future compiler facts
- profiles may narrow state permissions but cannot hide state authority

The current machine-readable contract is `hum.state_model.v0`, emitted by:

```text
hum state-model --format json
```

V0 mode is `contract_only_partial_declared_mutation_check`. It documents the
state model and advertises the small current checker behavior: `set name = ...`
requires a local `change name: ...` or a matching `changes:` entry.

## Consequences

- New syntax must explain what state it reads, writes, owns, borrows, consumes, shares, allocates, or observes.
- Stores remain explicit and policy-bearing.
- Linear resources become the preferred future shape for handles, sockets, locks, transactions, buffers, capabilities, and foreign resources.
- Shared mutable state remains deferred until Hum has a checked sharing model.
- Unsafe and FFI must carry ownership, aliasing, state restoration, and resource-transfer facts.
- Debugging and profiling must preserve value, place, ownership, mutation, and external-state facts.
- The first executable core should stay small enough for the state model to be checked.

## Alternatives Rejected

- Hidden global mutable state as a convenience feature.
- Runtime-only state discipline with no source-visible permissions.
- Rust syntax copied wholesale without Hum-specific pedagogy.
- Pure immutability as the universal default for systems code.
- Garbage collection as the default memory/state model for strict systems profiles.
- Actor-only state as the universal answer.
- Linear types for every ordinary value.

## BDFL Note

The paved road is boring immutable data and explicit mutation. Power is allowed,
but it must leave tracks.
