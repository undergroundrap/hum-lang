# RAD Debugger Lessons For Hum

Date: 2026-07-07

Status: distilled research note

Source: user-provided transcript of a long-form debugger discussion with Ryan
Fleury about RAD Debugger, debug information, stepping, breakpoints,
visualizers, and debugger ergonomics.

Copyright note: this document is a distilled design note, not a transcript
import. It paraphrases the technical lessons relevant to Hum.

## Executive Thesis

The talk reinforces a hard language-design lesson: a debugger is not an IDE
afterthought. It is a stress test for the whole language implementation.

For Hum, the lesson is not "build a debugger now." The lesson is: every lowering
stage, debug-info artifact, backend adapter, optimizer, standard-library shape,
and type-layout decision should preserve enough information for a future debugger
to be fast, reliable, and useful.

The best debugger beats manual logging because it gathers the same kind of
evidence with less human effort. If it is slow, unreliable, or awkward, users
will return to logging, image dumps, graph dumps, or manual instrumentation.

## Main Lessons

### Debugging And Logging Are A Continuum

The transcript rejects a false split between debugger users and logging users.
Both are ways to observe program state. The only question is how much work the
programmer must do to collect the evidence.

Hum consequence:

- Debug output, traces, probes, profiles, contracts, and future debugger views
  should share source-linked facts.
- `hum debug` should not be a separate island from `hum graph`, `hum check`,
  profile reports, and evidence bundles.
- A future debugger should be able to answer many "what would I log here?"
  questions without editing source, rebuilding, and reproducing the state.

### Speed Is A Semantic Requirement

The talk repeatedly frames the debugger's job as saving time. A feature that
exists but takes longer than manual instrumentation is not really a successful
feature.

Hum consequence:

- Debug facts should be structured and direct to query, not recovered from prose.
- Common views should be generated from compiler facts rather than custom editor
  plugins re-solving language semantics.
- Debug-info formats should be optimized for the debugger's read path, not only
  for compiler emission convenience.

### Debugger UI Is A Hard Systems Problem

The discussion describes debuggers as a rare combination of dense interactive UI
and low-level machine control. A good debugger must show source, registers,
memory, modules, disassembly, values, type layouts, call stacks, and domain data
without turning every task into configuration work.

Hum consequence:

- Hum should design debug data for high-density views: compact, indexed,
  source-addressable, and stable under edits.
- The semantic graph should provide debugger-ready identities for tasks, intent
  sections, values, contracts, effects, and profile claims.
- Debug views should be able to degrade gracefully when optimized code removes,
  inlines, fuses, or reorders source constructs.

### Source Lines Do Not Map Cleanly To Instructions

The transcript spends real time on line stepping because it is deceptively
complex. One source line can lower to many instructions; optimized instructions
can be reordered, mixed with other lines, removed, or split apart. Stopping in
the middle of a source line is normal.

Hum consequence:

- Hum source maps should model instruction ranges as sets and provenance
  relations, not as a single "line to address" lookup.
- Optimizers must preserve enough provenance for stepping and explanations.
- Inlining, tail calls, generated code, contract checks, and recomputation should
  remain explainable to humans and agents.

### Step Semantics Are State Machines

Step over, step into, and step out become difficult around recursion, calls,
jumps, traps, stack pointers, fake return addresses, and target-specific
details. Even simple stepping can become a small control program.

Hum consequence:

- Hum should define source-level step intent separately from backend mechanisms.
- `hum.debug_info.v0` should include call boundary, return boundary, inlining,
  generated-code, and tail-call facts when those exist.
- Early backend choices must not destroy the ability to implement honest stepping
  later.

### Do Not Depend On Unavailable Kernel Features

The talk explains why user-level handling for breakpoint traps would make
conditional breakpoints and dynamic probes much faster, but ordinary user-mode
debuggers usually pay expensive kernel/debugger round trips.

Hum consequence:

- Hum should not assume special OS debugger support exists.
- Hum can still help by compiling opt-in debug probe sites, profile hooks, and
  contract hooks that are source-linked and removable in release profiles.
- Probe sites must be explicit profile artifacts, not hidden production behavior.

### Conditional Breakpoints And Profiling Are Related

Conditional breakpoints, dynamic sampling, flame-graph style investigation, and
manual instrumentation all suffer when the user has to edit source, rebuild,
rerun, and reproduce state.

Hum consequence:

- `cost:`, `allocates:`, `protects:`, and profile facts should be debugger and
  profiler inputs.
- A future profiler should be able to ask for source-linked samples without
  forcing the user to pre-place every marker manually.
- Debug probes and profile probes should share an artifact model.

### Visualizers Are Language Pressure

RAD Debugger-style visualizers for bitmaps, memory regions, geometry, text,
disassembly, and eventually pointer graphs show that raw values are often the
wrong debugging unit. A good debugger lets users see domain data directly and
live as the program steps.

Hum consequence:

- Hum should eventually support type-attached visualizers.
- Standard library containers, slices, spans, maps, text, bytes, images, tensors,
  graphs, geometry, evidence bundles, and task state should expose structured
  display hints.
- Visualizers must be reversible: the user should always be able to inspect the
  raw value and layout.
- Visualizer metadata should be local-first and source-controlled, not a
  per-machine mystery plugin.

### One-Line Visualizer Attachment Matters

The transcript emphasizes that a visualizer feature only saves time if the
association between a type and its display is easy and persistent. If every
debugging session requires plugin setup and manual binding, users will go back
to dumps and logging.

Hum consequence:

- Type-to-visualizer mappings should be expressible in source or package
  metadata with a small, stable surface.
- The mapping should travel with packages and be visible in graph/debug-info
  artifacts.
- This belongs near type/layout metadata, not as an editor-only convention.

### Debug Info Format Is A Product Decision

The talk highlights the pain of supporting PDB and DWARF directly and the value
of converting platform debug formats into a custom efficient internal format.
It also notes real scale limits in existing debug-info formats for very large
projects.

Hum consequence:

- Native formats such as DWARF and PDB are compatibility bridges, not Hum's
  source of truth.
- Hum should design `hum.debug_info.v0` as a first-party indexed fact artifact.
- The artifact should be large-project safe: stable ids, explicit versioning,
  wide offsets or chunked tables, and no avoidable 32-bit choke points.
- Linker, backend, debug info, and debugger design should be co-owned.

### Reliability Beats Cleverness

The Chrome debugger discussion is useful because it shifts the bar from "has a
breakpoint feature" to "does the breakpoint reliably catch the code I asked for,
including startup paths?" A debugger that misses events is worse than a missing
feature because it teaches users not to trust it.

Hum consequence:

- Hum debug contracts should separate "supported", "best effort", and "not
  available" states.
- Debugger and LSP capabilities should be machine-readable and tested with
  incomplete, startup, generated, and optimized-code cases.
- If a feature is unreliable, Hum should report that honestly instead of
  pretending it is stable.

## Near-Term Hum Design Consequences

Milestone 0 should not build a debugger, but it should preserve debugger inputs:

- exact source spans
- stable semantic graph ids
- value/place ids when lowering begins
- generated-code origins
- contract/effect/profile source links
- name-resolution boundaries
- layout and type summaries
- future visualizer slots

The next durable document after the current doctrine should be a debug-info and
visualizer model. That model should define:

- `hum.debug_info.v0` inventory shape
- visualizer hint shape
- source-map provenance rules
- probe-site inventory rules
- optimized-code honesty statuses
- links to native DWARF/PDB or backend artifacts

## What Hum Should Not Claim Yet

- Hum does not have a debugger.
- Hum does not yet emit native DWARF or PDB.
- Hum does not yet solve fast conditional breakpoints.
- Hum does not yet make optimized native stepping easy.
- Hum does not yet have type-attached visualizers.
- Hum can design for these outcomes now without pretending they already exist.

## Ground-Truth Targets

This note feeds:

- [../DEBUGGABILITY_DOCTRINE.md](../DEBUGGABILITY_DOCTRINE.md)
- [../TOOLCHAIN_2050.md](../TOOLCHAIN_2050.md)
- [../ARCHITECTURE.md](../ARCHITECTURE.md)
- [../SEMANTIC_GRAPH_SCHEMA.md](../SEMANTIC_GRAPH_SCHEMA.md)
- future `DEBUG_INFO_AND_VISUALIZER_MODEL.md`
