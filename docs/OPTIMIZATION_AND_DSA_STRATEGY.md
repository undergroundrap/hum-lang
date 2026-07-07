# Hum Optimization And DSA Strategy

Date: 2026-07-06

## Purpose

Hum should treat algorithms and data structures as language design, not as a
later library chore.

The goal is not to chase every new paper. The goal is to make modern
optimization evidence part of the path from syntax to standard library, under
the [Paved Road Doctrine](PAVED_ROAD_DOCTRINE.md): one safe default path, with
explicit evidence-backed side roads only when needed.

Hum should be designed so a programmer can state:

```text
store sessions: map SessionId -> Session {
  expects:
    up to 20 million active sessions

  optimizes:
    lookup speed
    memory density
    bounded tail latency

  protects:
    hostile keys cannot force unbounded work

  needs:
    stable addresses are not required
}
```

and the compiler, standard library, and tools can explain the chosen data
structure, reject impossible promises, or ask for more precise intent.

## The 2026 Optimization Rule

No optimization is Hum-shaped unless it satisfies all four:

1. It improves a measured workload or a proved bound.
2. It keeps the source and semantic graph understandable.
3. It preserves safety, security, and determinism claims.
4. It has a fallback path for hardware or input shapes where it loses.

This keeps Hum from becoming both slow and clever.

## Modern Hardware Reality

Hum should not rely on old data-structure folklore when modern hardware is often
limited by cache locality, indirection, allocation, branch behavior, SIMD width,
NUMA placement, and accelerator transfer costs.

Rules:

- asymptotic complexity is necessary but not sufficient
- layout, bytes moved, cache misses, branch misses, and allocation behavior must
  be benchmarked when they matter
- contiguous value-oriented defaults should beat pointer-heavy structures unless
  pointer stability, persistence, or update shape justifies the cost
- persistent or boxed structures belong behind explicit intent, not as invisible
  defaults for systems profiles
- optimization reports should identify hardware assumptions and fallback paths

This follows the value-semantics and modern-hardware lesson captured in
[research/2026-07-07-lattner-compiler-lessons.md](research/2026-07-07-lattner-compiler-lessons.md).

## Bellard Constraint Rule

Bellard-style systems work treats smallness as a feature, not a side effect.
Hum optimization reports should eventually make these costs visible when they
matter:

- binary size
- startup time
- memory floor
- peak memory
- dependency count
- optional runtime services
- deterministic artifact status
- portability boundary

A feature that is fast only by assuming a huge runtime, hidden service, hidden
cache, or unbounded memory floor is not a paved-road systems feature. It may be a
profile-specific side road, but it must say so.

See [research/2026-07-07-bellard-systems-lessons.md](research/2026-07-07-bellard-systems-lessons.md).

## Research Intake Pipeline

New DSA or compiler optimization research enters Hum through a lab, not straight
into `std`.

Pipeline:

1. `research`: paper, prototype, and threat model.
2. `reference`: small safe Rust implementation or external prototype wrapper.
3. `fixture`: `.hum` contract examples that describe the intended API.
4. `benchmark`: against Rust, C++, Zig, Go, Java, and domain libraries where relevant.
5. `adversarial`: hostile input, memory pressure, concurrency, and fuzz tests.
6. `explain`: beginner explanation, systems explanation, and semantic graph shape.
7. `candidate`: experimental Hum API with stability warning.
8. `std`: only after sustained evidence and maintenance clarity.

Research should make Hum sharper, not wider.

## First-Class DSA Areas

### Hash Tables And Maps

Andrew Krapivin, Martin Farach-Colton, and William Kuszmaul's open-addressing
work is a signal that old assumptions about hash-table probe costs can move.
Their 2026 cardinality-constraint work is also a signal for compiler/prover
internals: better encodings can make verification cheaper.

Hum response:

- build `map-lab` before blessing `std.data.Map`
- separate trusted-key, adversarial-key, pointer-stable, dense, ordered, and concurrent maps
- expose memory density, probe behavior, iteration order, and address stability as contracts
- distinguish present-key lookup, missing-key lookup, insertion, deletion, resize, and iteration cost
- treat element reordering and pointer stability as source-visible promises
- keep the default map secure under hostile input
- let faster trusted maps exist only behind visible intent
- study elastic and funnel hashing as lab candidates, not default containers

See [HASH_TABLE_RESEARCH_2501_02305.md](HASH_TABLE_RESEARCH_2501_02305.md).

### Succinct And Dense Structures

Memory density should be a standard-library design goal. Succinct structures,
bitsets, compressed indexes, and rank/select ideas belong in a lab tier before
they enter `std`.

Hum response:

- require bytes-per-element benchmarks
- require cache-miss and branch-miss reporting
- expose compression/decompression cost
- prefer simple dense layouts before clever encodings

### SIMD, Vector, And Layout-Aware Code

Hum should not make programmers write five versions of a loop for five CPU
families.

Hum response:

- portable vector APIs first
- explicit hardware specialization second
- runtime dispatch generated by tools
- `benchmarks:` required before accepting manual prefetch, unrolling, or layout tricks
- semantic graph records the hardware assumptions

### GPU And Accelerator Data Structures

GPU tables, caches, tensors, and batched kernels are powerful but should not
infect the core language.

Hum response:

- keep accelerators in `std.accel`
- make host/device transfer visible
- distinguish cache semantics from dictionary semantics
- require deterministic fallback where possible
- require benchmark profiles that include transfer cost

### Space-Time Simulation And Recompute Policy

Williams' 2025 square-root-space simulation result belongs in Hum's optimization
research stack because it challenges the lazy assumption that a long computation
must store a near-linear trace to be recoverable.

Hum response:

- treat peak space, scratch space, caching, checkpointing, and recomputation as
  source-visible resource facts
- allow future memory-pressure profiles to prefer recomputation over storage for
  pure deterministic subgraphs
- forbid hidden recomputation across IO, randomness, time, mutation, or foreign
  authority
- expose dependency graphs before attempting smart space-time optimization
- require benchmark or proof evidence before claiming a space-time win
- track withdrawals and confidence for fast-moving theory results

See [research/2026-07-07-time-space-simulation.md](research/2026-07-07-time-space-simulation.md).

### Constraint Solving And Prover Internals

Compiler verification is also a data-structure problem.

Hum response:

- use compact encodings where they make proof obligations cheaper
- treat solver-facing encodings as compiler internals first
- expose proof failures as Hum diagnostics, not SAT jargon
- benchmark proof generation and solver time separately

## Java Lessons For Optimization

Java's long-running Valhalla work is a warning: if a language bakes object
identity, nullable references, boxed primitives, and erased generics into the
foundation, decades of optimization work may be needed to claw back data layout.

Hum response:

- value-like records are core, not retrofit
- no ordinary null
- generic code must preserve enough type/layout information for optimization
- boxing is explicit
- object identity is requested, not assumed
- arrays and collections expose initialization and bounds guarantees

## TypeScript Lessons For 2026 SWE

TypeScript improved large JavaScript codebases, but official design goals accept
an erasable, intentionally unsound type system and no end-to-end build pipeline.
Recent 2026 research also suggests TypeScript failures cluster around tooling,
configuration, API misuse, async error handling, and dependency heterogeneity.

Hum response:

- Hum's type/effect system is not erasable decoration
- Nectar owns the build/package graph from day one
- async and concurrency wait for a clear effect/cancellation model
- semantic graph output is compiler truth, not reconstructed tool opinion
- package trust, dependency use, and production exposure are visible metadata

## Agent-Era SWE Lessons

2026 agent benchmarks show that agents struggle with repo-wide structure,
integration, environment setup, strict type semantics, and long-horizon
maintainability.

Hum response:

- every package has a machine-readable semantic graph
- diagnostics include repair hints and related promises
- `hum explain` should reveal why a structure or optimization was selected
- generated code must pass the same contracts, tests, benchmarks, and regret gates as human code
- Nectar should make environment setup boring and reproducible

Agents can help Hum move faster. They must not lower the evidence bar.

## Optimization Admission Gate

No optimization, data structure, or stdlib primitive should stabilize unless it
answers:

1. What workload does it improve?
2. What workload does it hurt?
3. What is its asymptotic bound?
4. What are its constant-factor costs?
5. What memory layout does it require?
6. What are its adversarial inputs?
7. What hardware assumptions does it make?
8. What footprint budget does it require?
9. What deterministic artifact guarantees does it preserve or weaken?
10. What safety or security claims does it depend on?
11. What does the semantic graph expose?
12. How does `chirp` catch misuse?
13. How does `humfmt` keep examples readable?
14. How does Nectar benchmark it reproducibly?

If the answer is not clear, the feature belongs in a lab, not `std`.

## Brutal Rule

Hum should optimize at the level where the biggest waste lives:

```text
algorithm > data layout > allocation > cache behavior > compiler lowering > instruction tricks
```

Instruction tricks are allowed. They just do not get to masquerade as language
vision.

## Sources

- Williams, "Simulating Time With Square-Root Space": https://arxiv.org/abs/2502.17779
- Shalunov, "Improved Bounds on the Space Complexity of Circuit Evaluation": https://arxiv.org/abs/2504.20950
- Henzinger, Pyne, Ragavan, "Catalytic Tree Evaluation From Matching Vectors": https://arxiv.org/abs/2602.14320
- Krapivin, Farach-Colton, Kuszmaul, "Optimal Bounds for Open Addressing Without Reordering": https://arxiv.org/abs/2501.02305
- Krapivin, Przybocki, Subercaseaux, "Near-Optimal Encodings of Cardinality Constraints": https://arxiv.org/abs/2603.28954
- OpenJDK Project Valhalla: https://openjdk.org/projects/valhalla/
- Oracle Java tutorial, "Type Erasure": https://docs.oracle.com/javase/tutorial/java/generics/erasure.html
- TypeScript Handbook, "Type Compatibility": https://www.typescriptlang.org/docs/handbook/type-compatibility.html
- TypeScript Design Goals: https://github.com/microsoft/TypeScript/wiki/TypeScript-Design-Goals
- Lattner compiler lessons: [research/2026-07-07-lattner-compiler-lessons.md](research/2026-07-07-lattner-compiler-lessons.md)
- Tang, Alimadadi, Sumner, "From Logic to Toolchains": https://arxiv.org/abs/2601.21186
- Ren et al., "SaaSBench": https://arxiv.org/abs/2605.17526
- Xiang et al., "Rust-SWE-bench": https://arxiv.org/abs/2602.22764
