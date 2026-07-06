# Safety-Critical And Engine-Grade Edge Cases

Date: 2026-07-06

## Purpose

This document captures the edge cases Hum must handle before it can credibly aim
at life-saving medical devices, vehicles, industrial control, aerospace, or a
high-end realtime engine runtime.

The point is not to claim Hum is certifiable today. It is to design the language
so that certification, assurance, determinism, and hard performance are not
bolted on later.

## Brutal Thesis

A memory-safe language is not automatically a safety-critical language.

Safety-critical systems need evidence:

```text
hazard -> requirement -> design -> code -> test/proof -> trace -> release artifact
```

Engine-grade systems need evidence too:

```text
budget -> data layout -> scheduling -> memory behavior -> frame trace -> regression gate
```

Hum should make both chains machine-readable.

## Research Signal

The serious prior art points in one direction:

- Medical device software guidance expects documented safety/effectiveness
  evidence for device software functions.
- Medical cybersecurity guidance expects design, labeling, and premarket
  submission evidence for cyber risk.
- IEC 62304-style medical software work is lifecycle, risk, classification,
  maintenance, configuration, and problem-resolution heavy.
- ISO 14971-style medical risk management is about risk analysis, risk control,
  verification, and production/post-production feedback.
- ISO 26262-style automotive work is lifecycle, ASIL, hazard analysis,
  confirmation, and traceability heavy.
- ISO 21448 / SOTIF adds hazards from insufficient intended functionality, not
  just faults.
- Ferrocene proves Rust can move toward qualified safety-critical use, but by
  constraining toolchains and producing evidence, not by marketing alone.
- SPARK proves contracts, absence of runtime errors, and modular formal proof can
  be language-level, not comment culture.
- seL4 proves that high assurance is possible, but also that proofs are expensive
  and require small, intentionally designed cores.
- Unreal-style engine tooling shows profiling, tracing, memory analysis, asset
  loading analysis, and data-oriented systems are core runtime infrastructure,
  not optional polish.

Hum should absorb the lesson: serious domains need profiles, traceability,
contracts, and artifacts.

## Recommendation

Before more syntax expansion, Hum should add a profile model to the design.

Possible profiles:

```text
profile normal
profile embedded no heap
profile engine hot path
profile hard realtime
profile safety critical
profile medical class c
profile automotive asil d
profile certified toolchain
```

Profiles are not marketing labels. They are compile-time and tooling policies.

A profile can:

- forbid language features
- require checked contracts
- require generated evidence
- require deterministic build artifacts
- require benchmark gates
- require static-analysis/lint gates
- require toolchain version locks
- require dependency/SOUP evidence
- require coverage, fuzzing, proof, or traceability

## Safety-Critical Language Gates

### 1. Traceability Is A First-Class Output

Hum must export trace links:

```text
hazard -> protects: -> needs: -> ensures: -> does: -> test/proof -> diagnostic -> release artifact
```

This belongs in `hum graph`, not spreadsheets maintained by tired humans.

### 2. No Hidden Panic Or Unwind

Safety profiles should forbid ordinary unwinding across critical boundaries.

Rules:

- no panic in safety-critical tasks unless declared as fail-stop behavior
- no unwinding across FFI
- no hidden allocation failure panic
- no hidden bounds panic in release safety profiles
- contract violation behavior must be named: `abort`, `safe stop`, `fallback`, or
  `report and continue`

### 3. Allocation Must Be Profile-Aware

Hard realtime and safety profiles need allocation discipline.

Rules:

- heap allocation forbidden in critical loops unless explicitly budgeted
- dynamic allocation must be fallible where failure is plausible
- arenas must declare lifetime and peak size
- fragmentation must be measured for long-running systems
- deallocation timing must be visible where it can cause jitter

### 4. Time Is A Type, Not A Number

Time bugs kill systems.

Hum needs:

- monotonic time vs wall-clock time distinction
- duration units in types
- deadline vs timeout vs period distinction
- clock drift handling
- tick wraparound tests
- no implicit conversion between frames, milliseconds, samples, and ticks

### 5. Numeric Behavior Must Be Boring

Safety profiles should force explicit numeric policy:

- checked integer overflow
- saturating arithmetic where intended
- wrapping arithmetic only when named
- fixed-point support for embedded/control loops
- explicit floating-point profile: NaN, infinity, rounding, denormals, FMA, and
  cross-platform determinism
- physical units and dimensions for medical/robotics/physics code

### 6. Concurrency Needs Contracts

Safety profiles should require:

- declared thread/task ownership
- lock-order declarations
- bounded blocking
- cancellation cleanup
- interrupt/thread boundary rules
- memory-ordering names readable by humans
- no lock-free reclamation without a proof/test packet

### 7. Unsafe And Foreign Need Safety Cases

Unsafe code should compile only with a review packet in serious profiles:

```text
unsafe task read sensor register(address: Address) -> UInt32 {
  why:
    read memory-mapped hardware register

  needs:
    address is aligned
    address belongs to sensor register block
    volatile read is required

  protects:
    no ordinary pointer aliases device memory
    no cached stale value is used for dosage decision

  trusts:
    hardware reference manual version 3.2

  proves:
    address range checked before call

  watch for:
    register read may clear interrupt flag
}
```

FFI must declare ABI, ownership transfer, panic boundary, callback lifetime,
threading model, and layout.

### 8. Dependencies Become SOUP-Like Evidence

For medical and safety-critical work, dependencies are not casual.

Nectar should eventually emit a package evidence packet:

- package identity and version
- source/provenance
- license
- unsafe usage
- foreign interfaces
- build scripts
- tests and coverage
- known vulnerabilities
- review status
- replacement/fallback plan

### 9. Builds Must Be Reproducible And Qualified

Safety profiles need:

- exact compiler/tool versions
- target triple and CPU features
- deterministic build mode
- dependency lockfile
- source hash
- generated-code hash
- semantic graph hash
- test/proof artifact hash
- codegen settings
- release manifest

A compiler for safety-critical Hum needs its own qualification path. Ferrocene is
the practical lesson: toolchain qualification is a product and evidence problem,
not just a compiler engineering problem.

### 10. Post-Release Evidence Matters

Medical and industrial systems keep living after release.

Hum should design for:

- problem reports
- field telemetry with privacy controls
- traceable patches
- risk re-evaluation after defects
- migration reports
- long-term support branches
- toolchain freeze windows

## Engine-Grade Runtime Gates

A language that wants to beat future engines cannot merely be fast. It must make
hitches explainable and preventable.

### 1. Frame Budgets In Source

Engine code needs budgets:

```text
cost:
  update is within 0.25 ms at p99 on ps5-like profile
  allocates 0 bytes during frame update
  touches at most 3 archetype tables
```

Hum should let `hum bench` and `hum profile` map runtime facts back to these
source promises.

### 2. Hot Path Profiles

`engine hot path` should forbid:

- heap allocation unless budgeted
- blocking IO
- shader/asset compilation
- unbounded loops
- unbounded dynamic dispatch
- logging without rate limits
- implicit locks
- unexpected virtual calls

### 3. Data Layout Is A Contract

Engine-grade Hum needs first-class layout facts:

- struct size and alignment
- cache-line grouping
- array-of-structs vs struct-of-arrays
- archetype/table layout
- false-sharing risk
- SIMD width assumptions
- GPU upload layout

### 4. Deterministic Replay

Simulation, networking, and bug reproduction need deterministic replay.

Hum should track:

- random seeds
- input timeline
- task schedule
- floating-point profile
- asset versions
- thread count and CPU/GPU feature profile
- nondeterministic sources

Game engines are often not deterministic enough for verification workflows. Hum
should treat replay as a language/toolchain feature.

### 5. Asset And Shader Pipelines Are Part Of The Program

For engines, code is not the whole executable behavior.

Nectar should eventually track:

- asset hashes
- cook settings
- shader compiler versions
- generated layout metadata
- streaming budgets
- GPU memory budgets
- platform-specific cooked variants

### 6. Telemetry Must Be Structured

Unreal Insights is a good reminder: a serious engine needs structured trace,
timing, memory, networking, asset-loading, and cooking analysis.

Hum should make source-level promises visible in trace output:

```text
trace frame_update.task_path
trace allocation.source_section
trace system.schedule_reason
trace asset_stream.budget
```

## Senior Designer Edge Cases Hum Must Not Miss

### Safety And Certification

- requirements traceability
- hazard traceability
- risk control verification
- safety class/profile per module
- certified subset of language
- qualified compiler/toolchain path
- coding-standard subset similar to MISRA/SPARK discipline
- generated evidence packets
- reproducible builds
- dependency/SOUP review
- long-term maintenance and problem resolution

### Runtime Failure

- allocation failure
- stack overflow
- panic/abort behavior
- watchdog timeout
- fail-safe state
- degraded mode
- retry storms
- partial initialization
- brownout/power loss
- persistent storage corruption
- firmware update rollback

### Time And Scheduling

- deadline miss
- priority inversion
- lock convoy
- jitter
- tick wraparound
- clock drift
- leap seconds where wall time matters
- cancellation race
- ISR/task handoff
- scheduler nondeterminism

### Memory And Hardware

- DMA aliasing
- MMIO volatile semantics
- cache coherency with devices
- alignment faults
- endianness
- unaligned access differences
- strict provenance
- uninitialized memory
- secret zeroization
- stack bounds
- fragmentation over months

### Numeric And Physics

- integer overflow
- unit mismatch
- fixed-point scaling
- NaN propagation
- signed zero
- denormals
- nondeterministic FMA
- cross-platform float differences
- stochastic algorithms needing deterministic seeds

### Concurrency

- data races
- deadlocks
- livelocks
- ABA problem
- epoch/hazard misuse
- memory-ordering mismatch
- thread-local lifetime leaks
- cancellation while holding resources
- backpressure collapse

### Security

- unsafe/foreign trust boundary
- supply-chain compromise
- build script compromise
- secrets in logs
- insecure random
- downgrade attack
- unsigned update
- dependency vulnerability
- malformed input parser bug
- denial-of-service via resource exhaustion

### Engine Runtime

- frame hitch from allocation
- streaming stall
- shader compilation stall
- GPU/CPU synchronization bubble
- false sharing in jobs
- cache-miss storm
- asset version mismatch
- non-replayable bug
- network rollback divergence
- editor-only code in runtime build
- telemetry overhead changing behavior

## Profile Sketches

### `profile safety critical`

```text
forbids:
  hidden heap allocation
  panic unwind
  unchecked recursion
  unbounded loop without variant or watchdog
  unsafe without review packet
  foreign without ABI contract
  ignored Result
  implicit numeric narrowing
  ordinary floating point without profile

requires:
  traceability output
  deterministic build manifest
  test/proof coverage packet
  dependency evidence packet
  explicit failure policy
```

### `profile hard realtime`

```text
forbids:
  unbounded allocation
  blocking IO
  unbounded locks
  background deallocation jitter
  runtime code generation

requires:
  WCET estimate or measured bound
  stack bound
  interrupt latency budget
  deadline miss behavior
```

### `profile engine hot path`

```text
forbids:
  per-frame allocation without budget
  blocking asset IO
  unbounded logging
  implicit locks
  hidden virtual dispatch in hot loop

requires:
  frame budget
  memory budget
  trace labels
  replay behavior
  platform profile
```

### `profile certified toolchain`

```text
requires:
  compiler version lock
  target version lock
  standard-library subset lock
  generated artifact hashes
  semantic graph hash
  tool qualification evidence
  migration report for compiler upgrades
```

## What This Changes For Hum

1. Add `docs/RUNTIME_PROFILES.md` before unsafe, async, or stdlib growth.
2. Add safety/realtime/engine profiles to the roadmap as design targets.
3. Make `hum graph` eventually export traceability and profile evidence.
4. Make Nectar responsible for reproducible build and dependency evidence.
5. Treat `panic`, allocation, floating point, time, and concurrency as profile
   policy topics, not ordinary library details.
6. Keep the core language smaller, because certifiable subsets must be small.

## Brutal Assessment

If Hum wants to be used near life-saving devices, it cannot be merely nicer Rust
or safer C++.

It needs a certifiable subset, a traceable toolchain, profile-specific bans,
source-level risk controls, and evidence artifacts that auditors can inspect.

If Hum wants to beat future engines, it cannot be merely fast in microbenchmarks.

It needs source-level frame budgets, data layout contracts, deterministic replay,
asset-pipeline provenance, structured telemetry, and profiling that maps directly
back to promises in code.

The language idea is strong enough to aim there, but only if we add evidence and
profiles before adding magic.

## Sources

- FDA, Content of Premarket Submissions for Device Software Functions, June 2023: https://www.fda.gov/regulatory-information/search-fda-guidance-documents/content-premarket-submissions-device-software-functions
- FDA, Cybersecurity in Medical Devices, February 2026: https://www.fda.gov/regulatory-information/search-fda-guidance-documents/cybersecurity-medical-devices-quality-management-system-considerations-and-content-premarket
- ISO 14971:2019, Medical devices risk management: https://www.iso.org/standard/72704.html
- ISO 26262-1:2018, Road vehicles functional safety: https://www.iso.org/standard/68383.html
- ISO 21448:2022, Safety of the intended functionality: https://www.iso.org/standard/77490.html
- IEC 62304 overview: https://en.wikipedia.org/wiki/IEC_62304
- Ferrocene qualified Rust toolchain: https://ferrocene.dev/
- SPARK language overview: https://www.adacore.com/languages/spark
- seL4 whitepaper: https://sel4.systems/About/seL4-whitepaper.pdf
- MISRust, mapping MISRA C++ 2023 to Rust: https://arxiv.org/abs/2605.23490
- Unreal Insights documentation: https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine
- Unreal MassEntity documentation: https://dev.epicgames.com/documentation/en-us/unreal-engine/mass-entity-in-unreal-engine
- On Determinism of Game Engines used for Simulation-based Autonomous Vehicle Verification: https://arxiv.org/abs/2104.06262
- The Essence of Entity Component System: https://arxiv.org/abs/2606.14919