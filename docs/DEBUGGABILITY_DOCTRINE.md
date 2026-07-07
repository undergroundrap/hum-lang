# Debuggability Doctrine

Date: 2026-07-07

Status: doctrine

## Purpose

Hum should be designed so a great debugger can exist before `hum debug` exists.
A debugger is too large to build during the parser and preview milestones, but
compiler artifacts created now must not make future debugging slow, lossy, or
bolted on.

The rule is simple: if Hum can lower, optimize, verify, or package a fact, it
should preserve enough identity for a human, debugger, profiler, or agent to
inspect that fact later.

## Doctrine

Debuggability is part of the language, not an IDE extra. Hum must preserve
source identity, value identity, layout facts, effect facts, contract facts,
profile facts, and provenance through every lowering boundary.

A Hum debugger should be faster and clearer than adding `printf` statements. If
stepping, inspection, or visualization is slower than logging, users will log and
Hum will have failed one of the core systems-language workflows.

Hum should eventually emit a first-party debug-info artifact, likely
`hum.debug_info.v0`, that is separate from but linkable to native debug formats
such as DWARF and PDB. Native formats are bridges to host tools; Hum facts remain
the authority for Hum intent.

## Required Design Consequences

- Every lowered value needs a stable path back to source span, type, layout,
  ownership/effect facts, and semantic graph node ids when those facts exist.
- Optimizations must preserve enough provenance to explain moved, fused, inlined,
  removed, or recomputed code.
- Contract checks in debug mode must be source-addressable: `needs:`, `ensures:`,
  `protects:`, `trusts:`, `cost:`, and resource/profile claims should be
  inspectable as intent, not only as generated branches.
- Runtime and backend work must preserve stepping and source-map facts before it
  can be called user-ready.
- Debug visualizers are first-class design pressure. Buffers, bitmaps, slices,
  spans, maps, pointer graphs, geometry, memory regions, task state, and evidence
  bundles should eventually expose structured display hints instead of forcing
  raw hex or ad hoc logging.
- Debug data must be local-first. No cloud, network, training telemetry, or user
  data collection is required to inspect a Hum program.

## Early Artifact Shape

The first debug-info contract should be boring. It should inventory facts, not
control processes:

- source files, spans, and generated-code origin
- semantic graph node ids
- Core Hum and Hum IR node ids
- value ids, place ids, and storage classes
- type and layout summaries
- effect, allocation, trust, and profile links
- contract-check source links
- backend artifact links
- visualizer hints for known standard-library shapes

This contract should come after real Core lowering begins. Until then,
`hum core-preview`, `hum graph`, `hum resource-report`, and future lowering
reports should preserve the pieces a debugger will need.

## Non-Goals For Current Milestones

- no debugger implementation
- no Debug Adapter Protocol server
- no process control, breakpoints, traps, register inspection, or disassembly
- no native DWARF/PDB emitter
- no runtime instrumentation claim
- no claim that preview facts are executable debug info

## Design Tests

Before stabilizing a feature, ask:

1. Can a debugger show the source construct that produced this runtime state?
2. Can a profiler map runtime cost back to a source claim?
3. Can an agent inspect the same facts without scraping prose?
4. Can optimized code still explain where important values came from?
5. Can a user inspect domain data with a useful visualizer instead of raw bytes?
6. Does this work locally without network or telemetry?

## Related Docs

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [TOOLCHAIN_2050.md](TOOLCHAIN_2050.md)
- [TOOLING.md](TOOLING.md)
- [BACKEND_STRATEGY.md](BACKEND_STRATEGY.md)
- [SEMANTIC_GRAPH_SCHEMA.md](SEMANTIC_GRAPH_SCHEMA.md)
- [HUM_CORE_PREVIEW_SCHEMA.md](HUM_CORE_PREVIEW_SCHEMA.md)