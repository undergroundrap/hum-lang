# Hum Architecture Ground Truth

Date: 2026-07-06

## Purpose

This is the short stable map of Hum. The other docs are allowed to grow; this document says how they fit together and what must stay true when a new design idea arrives.

If another Hum doc disagrees with this one, treat that as a design bug to resolve.

## North Star

Hum is an intent-first, evidence-native systems language:

```text
human-readable intent -> precise formal core -> semantic graph -> checks, profiles, evidence, and tools -> portable backends and platform artifacts
```

Hum should be readable enough for beginners, strict enough for systems engineers, and structured enough for compilers, IDEs, debuggers, profilers, package tools, and coding agents.

Hum should have paved roads, not endless knobs: one obvious safe default path, explicit side roads only when evidence and source-visible intent justify them, and diagnostics that guide users back to the better path. See [PAVED_ROAD_DOCTRINE.md](PAVED_ROAD_DOCTRINE.md).

Evidence-native means Hum's output is not only a binary. The language and
toolchain should also emit machine-checkable intent, effect reports,
capability boundaries, diagnostics, profile facts, provenance, SBOMs, and
deployment evidence.

See [ADOPTION_STRATEGY_2026.md](ADOPTION_STRATEGY_2026.md).

## Architecture Layers

### 1. Surface Hum

Surface Hum is what people write: `task`, `type`, `store`, `test`, and checked intent blocks such as `why:`, `uses:`, `changes:`, `needs:`, `ensures:`, `fails when:`, `watch for:`, `protects:`, `trusts:`, `cost:`, `avoids:`, `tradeoffs:`, `tests:`, `proves:`, and `does:`.

The surface rule is controlled obviousness: no headers, no semicolons in normal source, no hidden effects, no hidden mutation, no hidden unsafe, and no correctness-critical comments without a checked home.

### 2. Formal Core

Surface Hum lowers into Core Hum. Core Hum defines values, places, mutation, expressions, statements, typed failure, effects, contracts, profiles, loops, and backend-preservation rules. New syntax is not stable until it lowers into the core and preserves graph facts.

See [FORMAL_CORE.md](FORMAL_CORE.md).

### 3. Semantic Graph

The semantic graph is Hum's shared truth for humans, compiler checks, `humfmt`, `chirp`, `hum lsp`, `hum debug`, `hum graph`, Nectar, and agents. Agents should query graph facts, not scrape terminal prose when the compiler can provide structured meaning.

See [SEMANTIC_GRAPH_SCHEMA.md](SEMANTIC_GRAPH_SCHEMA.md) and [DIAGNOSTICS.md](DIAGNOSTICS.md).

### 4. Checks And Evidence

Milestone 0 checks stay small: parse files, validate sections, preserve spans, enforce first mutation and cost promises, emit stable diagnostics, and emit graph JSON with exact `covers:` links between task obligations and tests.

Later checks add generated tests, ownership, borrowing, effect propagation, unsafe review packets, foreign/ABI boundaries, profile restrictions, performance contracts, package evidence, supply-chain evidence, and platform authority checks.

Rule: if a feature creates new power, it must create new evidence.

### 5. Runtime Profiles

Profiles are policy bundles for normal apps, containers, agent tools, Windows services, driver candidates, embedded no-heap code, hard realtime code, engine hot paths, safety-critical code, and certified toolchains. Profiles can forbid features, require evidence, narrow stdlib APIs, and change release artifacts.

See [RUNTIME_PROFILES.md](RUNTIME_PROFILES.md).

### 6. OS And Platform Model

Hum is Windows-first for proof on the primary development platform and portable-by-design for architecture. Windows APIs, services, drivers, registry, devices, installers, updates, telemetry, and process authority must be modeled as explicit platform capabilities, not hidden globals.

See [OS_AND_PLATFORM_MODEL.md](OS_AND_PLATFORM_MODEL.md).

### 7. Ecosystem Tools

The tools are part of the language: `hum`, `humfmt`, `chirp`, `nectar`, `hum lsp`, `hum debug`, and `hum graph`. No serious feature is stable until the tools have a story for it.

See [TOOLING.md](TOOLING.md), [FORMATTER.md](FORMATTER.md), [TOOLCHAIN_2050.md](TOOLCHAIN_2050.md), and [NECTAR_PACKAGE_MANAGER.md](NECTAR_PACKAGE_MANAGER.md).

### 8. Standard Library Labs

The first stable primitive families are `Result`/`Option`, `Vec`/`Slice`/`Span`, `Map`/`Set`, and `Text`/`Bytes`. Allocators, parsers, sync, SIMD, accelerators, networking, operations, storage, and numeric/tensor APIs go through labs before stable `std`.

The stdlib rule is:

```text
algorithm > data layout > allocation > cache behavior > compiler lowering > instruction tricks
```

See [PAVED_ROAD_DOCTRINE.md](PAVED_ROAD_DOCTRINE.md), [STDLIB_CONSTITUTION.md](STDLIB_CONSTITUTION.md), [STDLIB_PRIMITIVE_RESEARCH_2026.md](STDLIB_PRIMITIVE_RESEARCH_2026.md), and [OPTIMIZATION_AND_DSA_STRATEGY.md](OPTIMIZATION_AND_DSA_STRATEGY.md).

### 9. Backends

The backend order is Rust bootstrap front end, interpreter or Cranelift prototype, LLVM for mature optimized native builds, MLIR for vector/tensor/accelerator work, and Wasm/WASI for sandboxed portable components. Backends are targets; they are not Hum's semantic soul.

See [BACKEND_STRATEGY.md](BACKEND_STRATEGY.md).

## Non-Negotiable Decisions

- Rust remains the bootstrap compiler until Hum proves self-hosting through staged differential tests.
- Milestone 0 stays local, offline-first, non-executing, and safe on the maker's machine.
- Important comments become checked intent blocks.
- Unsafe and foreign code require review packets and profile gates.
- Containers, OS sandboxes, and agent tools do not replace language safety.
- Windows is the first tested platform, but platform-specific details stay behind explicit capability boundaries.
- No feature enters stable Hum without semantics, diagnostics, graph facts, tooling impact, profile impact, verification story, performance story, and pedagogy story.
- Defaults must be paved roads; non-default power requires explicit source intent and evidence.
- No parser-only or checker-only milestone should be presented as a credible public alpha; public adoption requires executable artifacts and evidence bundles.

## Current Build Order

1. Finish Milestone 0 semantic graph, diagnostics, generated test skeleton hardening, and coverage matching.
2. Keep docs honest by linking every new doctrine back to this architecture.
3. Add executable core only after the formal core gate is clear.
4. Add ownership/effects before serious unsafe, FFI, or native backend work.
5. Add package/build/profile evidence before networked package behavior.
6. Defer drivers, installers, Windows Update publishing, and kernel work until strict profiles and proof infrastructure exist.

## Update Rule

When adding a major Hum design document, answer:

```text
Which architecture layer does this belong to?
Which existing docs does it constrain?
Which semantic graph facts does it require?
Which profile or evidence gates does it change?
What must Milestone 0 ignore for now?
```
