# Hum Unsafe Policy

Date: 2026-07-06

Exact diagnostic allocations come only from
[`src/diagnostic_catalog.rs`](../src/diagnostic_catalog.rs); the checked human
projection is [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Purpose

Unsafe Hum is the narrow bridge between safe Hum and the machine.

This document defines when unsafe is allowed, what evidence it must carry, what
profiles can forbid, and how the compiler, semantic graph, tests, fuzzers,
sanitizers, and reviewers should treat it.

## Brutal Thesis

Unsafe is not a feature. Unsafe is a liability budget.

Hum can only be a credible systems language if unsafe code is:

- rare
- small
- named
- local
- reviewed
- profiled
- testable
- visible in graph output
- forbidden in strict profiles unless explicitly justified

Unsafe should feel like entering a sealed lab, not like flipping a casual switch.

## What Unsafe Means

Unsafe means the compiler cannot fully prove one or more required invariants.

It does not mean:

- skip type checking
- skip effect checking
- skip profile checks
- skip documentation
- skip tests
- skip review
- skip responsibility

Unsafe code still has a typed interface, declared effects, declared failure, and
security obligations.

## Unsafe Categories

Hum should classify unsafe by kind.

Candidate unsafe kinds:

```text
unsafe pointer
unsafe provenance
unsafe aliasing
unsafe uninitialized
unsafe layout
unsafe cast
unsafe lifetime
unsafe volatile
unsafe syscall
unsafe inline asm
unsafe unchecked index
unsafe manual allocation
unsafe atomic ordering
unsafe lockfree reclamation
unsafe foreign callback
unsafe dynamic code
```

The kind matters because each one has different proof obligations.

## Required Review Packet

Every unsafe task must include:

```text
unsafe task name(...) -> Result Output, Error {
  why:
    why safe Hum cannot express this directly

  uses:
    capabilities read

  changes:
    capabilities or state mutated

  needs:
    caller obligations before entering unsafe

  ensures:
    facts restored before returning to safe Hum

  keeps:
    invariants maintained across the unsafe region

  protects:
    security, memory, integrity, or availability property defended

  trusts:
    external facts not proven locally

  proves:
    proof, model, audit, sanitizer, or reasoning artifact

  tests:
    unit, property, fuzz, regression, sanitizer, or differential coverage

  watch for:
    attacker-controlled inputs and future maintenance traps

  cost:
    time, allocation, blocking, and resource behavior

  does:
    smallest possible unsafe body
}
```

Minimum required sections:

- `why:`
- `needs:`
- `ensures:`
- `protects:`
- `trusts:`
- `watch for:`
- `proves:` or `tests:`
- `does:`

A missing packet is a compile error for unsafe code, not a warning.

## Boundary Rule

Unsafe must be contained.

```text
safe caller -> checked safe wrapper -> tiny unsafe body -> checked safe result
```

Unsafe internals may not leak unproven invariants into safe callers.

A safe wrapper must restore every invariant it promises before returning.

## Core Invariants

Unsafe code must explicitly state which invariants it depends on.

### Spatial Memory Safety

Questions:

- What object does this pointer refer to?
- What are the bounds?
- What operations can read or write outside those bounds?
- What tests/fuzzers cover boundary lengths?

### Temporal Memory Safety

Questions:

- Who owns the object?
- When can it be freed?
- What references can outlive it?
- What prevents use-after-free or double free?

### Provenance

Questions:

- Where did this pointer or handle originate?
- Does it still carry permission to access the object?
- Was it cast through an integer, foreign API, or raw address?
- What profile or target makes this provenance valid?

### Aliasing

Questions:

- Can another reference read while this writes?
- Can another reference write while this reads?
- What prevents two mutable aliases?
- What happens across FFI callbacks?

### Initialization

Questions:

- Which bytes are initialized?
- Which fields may be uninitialized during construction?
- What prevents reading uninitialized memory?
- What happens if construction fails halfway?

### Layout

Questions:

- What size, alignment, padding, and representation are assumed?
- Is the layout stable across compiler versions, targets, and profiles?
- Is endianness relevant?
- Are padding bytes observable or copied into secrets/logs/network data?

### Concurrency

Questions:

- What thread owns the state?
- What synchronization protects it?
- What memory order is required?
- What prevents ABA, stale reads, deadlocks, or lost wakeups?

### Failure

Questions:

- What happens if allocation fails?
- What happens if the foreign call fails?
- What happens if a precondition is violated?
- Does failure unwind, abort, safe-stop, or return a typed error?

### Side Channels

Questions:

- Does timing depend on secrets?
- Does memory access pattern depend on secrets?
- Does logging, tracing, panic text, or crash dumping expose secrets?
- Does branch behavior matter for the profile?

## Unsafe Syntax Direction

Candidate shape:

```text
unsafe task read device register(address: Address) -> UInt32 {
  why:
    read memory-mapped hardware register

  needs:
    address is aligned for UInt32
    address belongs to sensor register block

  ensures:
    no memory outside sensor register block is read

  protects:
    safe code cannot read arbitrary process memory

  trusts:
    operating system mapped this register page correctly
    hardware manual version 3.2 defines read side effects

  proves:
    address range checked by caller

  tests:
    fuzz invalid addresses in simulator
    regression interrupt flag is not cleared by status read

  watch for:
    volatile read may have side effects
    address must not come from user input

  does:
    volatile read UInt32 from address
}
```

The unsafe marker belongs at the smallest callable boundary or block that needs
it. Do not make an entire module unsafe because one line needs raw access.

## Profile Policy

Profiles can forbid or harden unsafe.

### `normal`

Allows unsafe only with review packet and compiler-visible unsafe kind.

### `security hardened`

Requires:

- complete review packet
- fuzz or sanitizer evidence for memory/input-facing unsafe
- no secrets in logs or panic paths
- no raw string command execution near unsafe boundary

### `engine hot path`

Requires:

- allocation and blocking policy
- trace labels
- benchmark evidence
- no hidden locks or syscalls

### `hard realtime`

Forbids unless explicitly profile-approved:

- allocation
- blocking
- lock-free reclamation without bounded proof
- foreign calls with unbounded latency
- panic unwind

### `safety critical`

Forbids by default:

- unsafe without review packet
- foreign without ABI contract
- dynamic code
- unbounded loops
- hidden allocation
- panic unwind

Requires:

- traceability
- deterministic build evidence
- test/proof evidence
- deviation record

### `certified toolchain`

Requires:

- compiler version lock
- unsafe summary in release artifact
- semantic graph hash
- review signoff metadata
- migration report for compiler upgrades

## Standard Library Unsafe

Most users should never write unsafe.

The standard library may contain unsafe internals, but every such region must
have:

- local unsafe kind
- invariant comment replaced by checked packet fields where possible
- tests
- fuzzing if input-facing
- sanitizer evidence if memory-facing
- benchmark evidence if performance-facing
- misuse examples
- semantic graph exposure
- stability notes

A fast collection with unsound unsafe internals is worse than a slower safe one.

## Foreign Boundaries

Foreign calls are unsafe-adjacent even when memory-safe.

A `foreign` boundary must declare:

- ABI
- calling convention
- layout assumptions
- ownership transfer
- lifetime rules
- panic/unwind behavior
- callback behavior
- threading model
- error model
- allocator boundary
- trust boundary
- sanitizer availability

A foreign function is not just a call. It is another program entering Hum's trust
base.

## Diagnostics

`H1000-H1099` is the reserved `unsafe_ffi_provenance` family. It contains no
exact allocation. Future diagnostics in that family should cover missing
review packets, missing unsafe-kind declarations, profile violations,
provenance invariants, foreign ABI contracts, oversized unsafe bodies,
missing adversarial evidence, and secret-boundary policy. These are candidate
meanings only; a later independently reviewed work order must allocate any
exact code.

## Semantic Graph Requirements

`hum graph` should expose:

- unsafe task/block id
- unsafe kind
- source span
- active profiles
- required sections
- missing sections
- trusted assumptions
- protected properties
- proof/test artifacts
- foreign ABI contracts
- affected types and functions
- downstream safe wrappers
- risk summary for agents and reviewers

## Agent Rules

Agents may edit unsafe code only under stricter rules.

Good agent behavior:

- preserve review packet sections
- shrink unsafe scope when possible
- add tests/fuzzers when changing invariants
- explain which invariant changed
- refuse to invent false `proves:` claims
- use diagnostics and graph facts, not prose confidence

Bad agent behavior:

- wrap code in unsafe to silence an error
- weaken `needs:` or `protects:` to pass checks
- add a trust claim without evidence
- move unsafe outward for convenience
- hide allocation, panic, or IO inside unsafe code

## First Implementation Slice

Milestone 0 or 1 should not implement real unsafe execution yet.

First compiler support should be structural:

1. Parse `unsafe task` headers.
2. Emit unsafe kind and review packet sections into `hum graph`.
3. Reject unsafe tasks missing required sections.
4. Warn when unsafe body is large.
5. Add example fixtures for MMIO, FFI wrapper, unchecked slice, and lock-free queue.
6. Keep Rust bootstrap compiler itself at `#![forbid(unsafe_code)]`.

## Sources

- MITRE 2025 CWE Top 25: https://cwe.mitre.org/top25/archive/2025/2025_cwe_top25.html
- NIST SP 800-218, Secure Software Development Framework: https://csrc.nist.gov/pubs/sp/800/218/final
- NIST SP 800-160 Vol. 2 Rev. 1, Cyber-Resilient Systems: https://csrc.nist.gov/pubs/sp/800/160/v2/r1/final
- CHERI project: https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/
- MSWasm, revised 2026: https://arxiv.org/abs/2208.13583
- RustBelt project and publications: https://plv.mpi-sws.org/rustbelt/
- Verifying the Rust Standard Library, 2026: https://arxiv.org/abs/2606.17374
- Towards verifying unsafe Rust programs against Rust's pointer-aliasing restrictions, 2026: https://arxiv.org/abs/2603.28326

## Brutal Assessment

Hum should allow unsafe because systems programming needs contact with hardware,
foreign ABIs, allocators, atomics, and optimized data structures.

But unsafe must never become the social escape hatch for impatience.

The rule is simple:

```text
Unsafe buys power by paying evidence.
```

If a programmer will not pay that evidence cost, the code should stay safe or not
compile.
