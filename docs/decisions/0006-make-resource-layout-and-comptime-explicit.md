# 0006: Make Resource, Layout, And Compile-Time Power Explicit

Date: 2026-07-07
Status: accepted

## Context

Hum is absorbing lessons from systems languages, accelerator-oriented languages,
research on space/time tradeoffs, and agent-heavy development workflows.

Those lessons point in the same direction: powerful compilers and runtimes help
only when the program shape, resource intent, memory layout, and trust boundary
are visible enough to inspect.

Hum must avoid two opposite failures:

- hiding powerful behavior behind friendly syntax
- making every powerful path a pile of knobs, profiles, and folklore

## Decision

Hum will make resource intent, layout-sensitive representation, compile-time
execution, interop, and agent-facing facts explicit language and toolchain
surfaces.

Accepted rules:

1. Resource intent belongs in checked source blocks such as `cost:`,
   `allocates:`, `avoids:`, `tradeoffs:`, and `optimizes:`.
2. Layout-sensitive code must eventually expose layout assumptions as semantic
   graph facts: ABI, alignment, endian, pointer width, shape, dtype, device,
   ownership, and lifetime where relevant.
3. Compile-time execution is future work and must be explicit, effect-limited,
   budgeted, provenance-preserving, and profile-gated when it touches I/O,
   network, processes, foreign code, or generated artifacts.
4. Interop is an adoption path, not ambient trust. C, C++, Rust, Python, Wasm,
   platform APIs, process boundaries, and accelerator runtimes must enter Hum
   through explicit trust, ownership, layout, effect, failure, and profile
   contracts.
5. Agents receive compact schema-backed facts. Agents do not become part of the
   trusted base, and they should not scrape terminal prose when the compiler can
   emit JSON.
6. External math engines, proof tools, and benchmark harnesses remain evidence
   producers. Their results can strengthen Hum claims, but they do not become
   hidden compiler authority.

## Consequences

Hum can learn from languages with compile-time specialization, hardware-aware
layout, Python interop, and agent skills without inheriting their exact syntax or
adoption tradeoffs.

The language reference must keep ordinary users away from fake magic claims:
Hum should not say every algorithm becomes faster, every layout is automatically
optimal, or every benchmark is proof.

The compiler and tools should instead classify program shapes, emit obligations,
measure real performance, and make optimization evidence reviewable.

Docs, schemas, and examples must stay aligned. If a resource, layout, comptime,
interop, or agent feature cannot be represented in diagnostics, graph facts,
profile policy, and teaching material, it is not stable.

## Alternatives Rejected

- Treat compile-time execution as a macro free-for-all.
- Hide layout choices inside the optimizer without graph evidence.
- Present research-inspired space/time tradeoffs as generic speedups.
- Make Python, C/C++, Rust, or GPU interop feel ordinary before trust and layout
  contracts exist.
- Let agents rely on stale model memory instead of current compiler facts.
- Treat external verifier success as compiler authority without source-visible
  assumptions and checkable evidence.

## BDFL Note

Powerful features are welcome in Hum only when they make the program more
inspectable. If a feature is impressive but makes humans, tools, or agents less
able to explain the code, it waits.