# Hum Debug Info And Visualizer Model

Date: 2026-07-07

Status: design model, not implemented

Target schema: `hum.debug_info.v0`

## Purpose

This document turns the debuggability doctrine into a concrete future artifact
shape. It defines what Hum must eventually preserve so debuggers, profilers,
editors, verifiers, and agents can inspect optimized programs without guessing.

This is not a debugger implementation. It is the contract future lowering,
backend, linker, package, profile, and standard-library work must keep alive.

The core rule is:

```text
Hum source intent -> semantic graph -> Core Hum -> Hum IR -> backend artifacts
must preserve enough identity for fast, local, source-linked inspection.
```

Native debug formats such as DWARF and PDB are compatibility bridges. The Hum
artifact owns Hum-specific intent: tasks, sections, contracts, effects,
profiles, ownership, trust, resource claims, generated checks, visualizers, and
probe sites.

## Current Status

Milestone 0 does not emit `hum.debug_info.v0`.

Current compiler surfaces only preserve early inputs a future debug-info emitter
will need:

- source spans
- semantic graph node ids
- document symbols
- folding ranges
- task/test/evidence obligations
- Core preview facts
- IR readiness blockers
- backend preservation contract

A Hum implementation may not claim debugger support until it can emit, validate,
and test this model or a successor.

## Top-Level Shape

A future `hum.debug_info.v0` artifact should be an indexed fact inventory:

```json
{
  "schema": "hum.debug_info.v0",
  "producer": {},
  "build": {},
  "sources": [],
  "semantic_graph": {},
  "core": {},
  "ir": {},
  "backend_artifacts": [],
  "source_maps": [],
  "values": [],
  "places": [],
  "types": [],
  "layouts": [],
  "contracts": [],
  "effects": [],
  "profiles": [],
  "probe_sites": [],
  "visualizers": [],
  "native_debug_links": [],
  "honesty": {},
  "privacy": {}
}
```

The artifact should be optimized for debugger reads. Large projects should use
stable ids, chunked tables, wide offsets, and indexes rather than assuming small
32-bit sections will always be enough.

## Required Identity

Every lowered fact that can affect runtime inspection should keep a path back to
source intent when such a path exists.

Required identity classes:

- `source_file_id`
- `source_span_id`
- `semantic_graph_node_id`
- `core_node_id`
- `ir_node_id`
- `backend_artifact_id`
- `value_id`
- `place_id`
- `type_id`
- `layout_id`
- `contract_id`
- `effect_id`
- `profile_claim_id`
- `probe_site_id`
- `visualizer_id`

Ids should be stable within one build artifact and deterministic for the same
inputs where practical. They are not permanent global names unless a future
package metadata model makes them so.

## Source Maps

Hum source maps must model many-to-many provenance. One source line may produce
many machine ranges; one machine range may carry fused provenance from multiple
source constructs.

Each source-map edge should eventually record:

- `source_span_id`
- `semantic_graph_node_id`, when available
- `core_node_id`, when available
- `ir_node_id`, when available
- `backend_range_id`, when available
- `relation`
- `confidence`
- `optimization_status`
- `explanation`

Allowed `relation` values should start boring:

- `direct_lowering`
- `generated_check`
- `inlined_from`
- `outlined_to`
- `fused_with`
- `split_from`
- `moved_from`
- `removed_by_optimization`
- `recomputed_from`
- `constant_folded_from`
- `backend_only`

Allowed `confidence` values:

- `exact`
- `conservative`
- `best_effort`
- `unavailable`

Allowed `optimization_status` values:

- `debug_unoptimized`
- `optimized_preserved`
- `optimized_moved`
- `optimized_inlined`
- `optimized_fused`
- `optimized_removed`
- `not_emitted`

## Step Semantics

Hum should define source-level step intent separately from backend mechanics.
The user asks to step through Hum tasks and intent; the backend may implement
that through interpreter state, bytecode positions, native traps, DAP, DWARF,
PDB, or other mechanisms.

The debug-info artifact should eventually expose:

- task entry and exit boundaries
- `does:` statement boundaries
- call and return boundaries
- generated contract-check boundaries
- failure-edge boundaries
- inline stack information
- tail-call status
- loop and branch provenance
- optimized-away source explanations

Honesty matters more than smooth fiction. If optimized code cannot support a
source-level step precisely, the debugger should say so and offer the nearest
honest view.

## Probe Sites

A probe site is a source-linked place where debug or profile instrumentation may
be attached by a profile, debugger, profiler, or test harness.

Probe sites unify several workflows:

- conditional breakpoints
- contract checks
- allocation checks
- effect tracing
- profile samples
- deterministic replay markers
- resource-budget checks
- trust-boundary checks

Each probe site should eventually record:

- `probe_site_id`
- `kind`
- `source_span_id`
- `semantic_graph_node_id`
- `allowed_profiles`
- `forbidden_profiles`
- `runtime_cost_class`
- `release_presence`
- `backend_strategy`
- `privacy_class`

Initial `kind` values:

- `contract_needs`
- `contract_ensures`
- `contract_protects`
- `cost_claim`
- `allocation_claim`
- `effect_boundary`
- `trust_boundary`
- `profile_sample`
- `replay_marker`

Initial `release_presence` values:

- `debug_only`
- `profile_guided`
- `test_only`
- `release_allowed`
- `release_forbidden`

Profiles must be able to prove that forbidden probe sites are absent from release
artifacts.

## Visualizers

A visualizer is a local, source-controlled display contract for a value, type, or
layout. Visualizers are not editor-only plugins and are not required to collect
user telemetry.

Hum should eventually support type-attached visualizers for:

- bytes and memory regions
- text and ropes
- slices and spans
- vectors and maps
- pointer graphs
- bitmaps and image buffers
- geometry and meshes
- tensors and numeric arrays
- task state
- evidence bundles
- profile traces

Each visualizer should eventually record:

- `visualizer_id`
- `applies_to_type_id`
- `applies_to_layout_id`
- `name`
- `kind`
- `required_fields`
- `format_parameters`
- `fallback_view`
- `raw_view_available`
- `side_effect_policy`
- `privacy_class`

Initial `kind` values:

- `raw_struct`
- `memory`
- `text`
- `table`
- `sequence`
- `map`
- `graph`
- `bitmap`
- `geometry`
- `tensor`
- `evidence`
- `profile_trace`

Visualizers must be reversible. The user must always be able to inspect the raw
value, bytes, fields, and layout behind a richer view.

Visualizer evaluation must be side-effect free unless a future profile explicitly
allows controlled evaluation. Function calls from watch windows are powerful, but
Hum should not make them the default way to inspect data.

## Native Debug Links

Hum should link to native debug formats without treating them as the authority
for Hum intent. Native DWARF and PDB are compatibility targets, not the source of
truth for Hum-specific facts.

Each native link should eventually record:

- `backend_artifact_id`
- `target_triple`
- `native_format`
- `native_file`
- `native_compilation_unit`
- `range_index`
- `hum_source_map_ids`
- `hum_type_ids`
- `status`

Initial `native_format` values:

- `dwarf`
- `pdb`
- `codeview`
- `wasm_name_section`
- `wasm_dwarf`
- `custom_backend`
- `none`

Initial `status` values:

- `linked`
- `partial`
- `not_emitted`
- `unsupported_target`
- `unknown`

## Honesty Block

Every debug-info artifact should include a machine-readable honesty block:

```json
{
  "emission_status": "not_emitted_v0",
  "execution_status": "not_executable_v0",
  "native_debug_status": "not_emitted_v0",
  "step_status": "not_available_v0",
  "visualizer_status": "model_only_v0",
  "probe_site_status": "model_only_v0"
}
```

Future statuses must distinguish support from best effort:

- `supported`
- `partial`
- `best_effort`
- `not_available`
- `not_applicable`
- `unknown`

No tool should silently upgrade `best_effort` into `supported`.

## Privacy And Local-First Rules

Debug info can expose sensitive program structure, paths, names, domain types,
contracts, and security boundaries. Hum debug artifacts must be local-first by
default.

The privacy block should record:

- no required network access
- no cloud dependency
- no training telemetry
- path redaction status
- secret redaction status
- package metadata privacy class

A debugger or profiler may inspect local program state. It must not require
sending debug facts to a remote service.

## Semantic Graph Requirements

Future semantic graph versions should reserve or expose links for:

- debug info ids
- source-map provenance ids
- visualizer ids
- probe-site ids
- contract ids
- effect ids
- profile claim ids
- generated-check ids
- optimized-code explanation ids

Milestone 0 should not emit fake debug info. It should keep preserving spans and
node ids accurately so these links have something honest to attach to later.

## Profile Gates

Runtime profiles should control debug-info and probe behavior.

Examples:

- `debug`: may include rich source maps, visualizers, and debug-only probe sites.
- `profile`: may include sample points and cost probes with bounded overhead.
- `test`: may include contract probes and replay markers.
- `release`: should remove debug-only probes unless explicitly allowed.
- `safety critical`: should require an evidence record for every retained probe.
- `hard realtime`: should reject unbounded probe overhead.
- `certified toolchain`: should require reproducible debug-info emission and
  profile-controlled redaction.

## Non-Goals For Current Milestones

- no debugger implementation
- no Debug Adapter Protocol server
- no native trap or breakpoint logic
- no function evaluation in watch windows
- no native DWARF/PDB emission
- no executable Core Hum debug info
- no claim that `hum graph` is debug info
- no claim that visualizers exist in the compiler today

## Acceptance Gate Before DAP

Before building `hum debug` or a DAP server, Hum should have:

1. executable Core Hum or interpreter semantics
2. stable value/place ids through lowering
3. source-map edges from source to Core Hum and Hum IR
4. type and layout summaries
5. contract/effect/profile ids
6. modelled probe sites
7. at least one reversible stdlib visualizer model
8. local-first privacy/redaction policy
9. tests that optimized-code metadata is honest
10. a capability report that says which debug features are supported, partial,
    best effort, or unavailable

## Related Docs

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [DEBUGGABILITY_DOCTRINE.md](DEBUGGABILITY_DOCTRINE.md)
- [TOOLCHAIN_2050.md](TOOLCHAIN_2050.md)
- [SEMANTIC_GRAPH_SCHEMA.md](SEMANTIC_GRAPH_SCHEMA.md)
- [HUM_IR_CONTRACT_SCHEMA.md](HUM_IR_CONTRACT_SCHEMA.md)
- [BACKEND_CONTRACT_SCHEMA.md](BACKEND_CONTRACT_SCHEMA.md)
- [RUNTIME_PROFILES.md](RUNTIME_PROFILES.md)
- [research/2026-07-07-rad-debugger-lessons.md](research/2026-07-07-rad-debugger-lessons.md)
