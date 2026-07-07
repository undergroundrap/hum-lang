# Hum Compile-Time Strategy

Date: 2026-07-06

## Thesis

Hum should treat fast feedback as a core language feature, not a tooling bonus.

Rust proved that memory safety can win systems programmers. Rust also proved
that slow compile/edit/check loops become one of the main daily pains for large
projects.

Hum should keep Rust-grade safety ambition while making the fast path feel much
lighter.

## Goal

Hum should optimize for three loops:

1. `hum check`: fastest correctness loop.
2. `hum test`: fast local confidence loop.
3. `nectar build`: full package build loop.

Release optimization is important, but it should not slow every edit.

## Non-Negotiables

1. Checking must be separable from optimized code generation.
2. Incremental compilation must be designed into package/module semantics.
3. Semantic graph caching must be a first-class compiler artifact.
4. Expensive features must justify their compile-time cost before stabilization.
5. Macros, generics, traits, and effects must have complexity budgets.
6. The compiler must expose timing data early.

A language that hides compile cost from users will slowly become painful.

## CLI Shape

Compiler-level commands:

```powershell
hum check [--timings] <file-or-dir>...
hum graph [--timings] <file-or-dir>...
```

Package-level commands:

```powershell
nectar check
nectar test
nectar build
nectar build --release
nectar timings
```

`hum` understands source. `nectar` understands projects, dependencies, lockfiles,
build profiles, caches, and publishing.

## Backend Strategy

Hum should use different backends for different loops:

### Check Mode

No native codegen.

Do:

- parse
- build AST
- build semantic graph
- type check
- ownership/effect/cost checks
- diagnostics
- test skeleton generation

### Dev Build

Prefer fast codegen.

Cranelift is the likely first backend for development builds because fast compile
latency matters more than maximum generated-code performance during iteration.

### Release Build

Prefer optimized codegen.

LLVM can remain the serious AOT optimization backend once Hum IR is stable enough
to preserve intent before lowering.

### Future Hardware-Aware Build

MLIR may make sense later for vector, tensor, sparse, accelerator, and hardware
layout work. It should not be part of the early compiler path.

## Language Design Rules For Fast Compilation

Hum should avoid features that create unbounded compiler work.

Compile-time execution is future power, not a shortcut around language design.
When Hum adds compile-time constants, assertions, specialization, generated
code, or macro-like facilities, the work must be explicit, effect-limited,
budgeted, provenance-preserving, and visible in graph facts. Compile-time I/O,
network access, process execution, foreign calls, or generated artifacts require
separate profile gates.

See [decisions/0006-make-resource-layout-and-comptime-explicit.md](decisions/0006-make-resource-layout-and-comptime-explicit.md).

### Keep Module Boundaries Explicit

A module should have a stable public interface. A private implementation change
should not force the world to recheck.

### Keep Effects Explicit

`uses:` and `changes:` should help dependency tracking.

If a task only declares:

```hum
uses:
  clock.now

changes:
  sessions
```

then tools have a clearer impact boundary than they get from arbitrary hidden
state.

### Keep Macros Limited

Macros are compile-time code execution. They can destroy incremental builds,
diagnostics, semantic graph stability, and agent readability.

Hum should delay macros and prefer:

- generated tests from contracts
- declarative schemas
- compiler-known derives
- explicit code generation commands

### Keep Generics Understandable

Generics should be powerful enough for systems libraries, but the compiler must
avoid trait-solver complexity spirals.

Every generics feature needs:

- beginner explanation
- senior explanation
- compile-time cost model
- diagnostic examples
- worst-case tests

### Keep Imports Cheap

Imports should not execute arbitrary code.

Package metadata should be declarative and cacheable.

## Semantic Graph Caching

Hum should cache semantic graphs per file/package.

Cache key should eventually include:

- file content hash
- compiler version
- language edition
- enabled features
- dependency interface hashes
- target profile when relevant

Cached graph output should support:

- agent tools
- IDEs
- docs
- tests
- dependency impact analysis
- security review

## Timing Budgets

Early budget targets should be aggressive, even if approximate:

- small file `hum check`: under 50 ms
- small package `nectar check`: under 250 ms warm cache
- medium package `nectar check`: under 1 second warm cache
- no-op package check: near instant

These are not promises yet. They are pressure.

## What We Measure First

Milestone 0 measures:

- file read time
- parse time
- check time
- total time

Later milestones should add:

- type checking
- ownership checking
- effect checking
- graph serialization
- dependency resolution
- codegen
- linking
- test generation

## What Rust Users Will Judge

Rust users will not switch for slogans.

They will look for:

- safety credibility
- compile-time credibility
- diagnostic quality
- package manager reliability
- ecosystem discipline
- no hidden runtime tax
- understandable unsafe boundaries

Compile time is a wedge only if the safety story remains serious.

## Brutal Rule

Do not add a language feature if it makes common checks slower and only makes
rare clever code nicer.

Hum should make ordinary correct systems code fast to write, fast to check, and
fast to understand.

Bellard-style constraint engineering adds one more warning: fast checks should
not require a huge hidden runtime, heavyweight daemon, or opaque cache service.
The fast path should remain small enough to rebuild, inspect, and run offline.
