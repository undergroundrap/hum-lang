# Bellard Systems Lessons For Hum

Date: 2026-07-07

Status: distilled research note

Source status: user-provided video summary, checked against primary project pages
where possible.

## Executive Thesis

Fabrice Bellard's work is a useful counterweight to language-design sprawl. The
lesson for Hum is not to copy any single project. The lesson is to build deep,
small, portable infrastructure that solves real problems under hard constraints.

Hum should treat minimal resource budgets, deterministic behavior, portability,
and small inspectable implementation cores as first-class design pressure. A
systems language that cannot run small, explain itself, and rebuild quickly is
not yet serious.

## Primary Signals

Bellard's own project index lists a pattern rather than one theme: QuickJS,
Micro QuickJS, TSAC, NNCP, TinyEMU, JSLinux, QEMU, FFmpeg, TCC, TinyEMU, TCCBOOT,
TinyGL, pi computation work, and telecom/base-station software all attack
infrastructure-grade problems with compact implementations.

QuickJS is a small embeddable JavaScript engine with no external dependency for a
simple executable path, low startup time, near-complete ECMAScript conformance,
and reference-counting garbage collection with cycle removal.

MicroQuickJS takes the smallness pressure further: it targets embedded systems,
can run programs with very small RAM budgets, deliberately supports a stricter
subset, forbids several error-prone or inefficient JavaScript constructs, and
makes bytecode/debug-info size part of the user-visible tradeoff.

TCC shows the compiler-speed lesson directly: an integrated preprocessor,
compiler, assembler, and linker can be tiny and fast enough to use C as a script
or dynamic-code backend. Its optional bounds checker is a reminder that small
systems tools can still carry safety instrumentation.

JSLinux and TinyEMU show portability as an executable proof. The same mental
model spans CPU emulation, device models, browser execution, Wasm/JavaScript
compilation, remote filesystems, and benchmarkable virtual time.

TSAC reinforces reproducibility: compressed artifacts must decode the same way
across hardware and software configurations. For Hum, this matters for compiler
outputs, evidence bundles, debug info, profiles, and caches.

The pi computation record is less about pi than method: algorithmic work,
careful storage, verification, and commodity hardware can beat larger systems
when the design is sharp.

## Hum Lessons

### 1. Small Is A Systems Feature

Hum should not equate ambition with a large runtime. Small artifacts are easier
to audit, port, fuzz, verify, cache, teach, and embed.

Design consequence:

- Add footprint budgets to runtime profiles.
- Treat startup time, binary size, memory floor, and dependency count as product
  facts.
- Keep the interpreter and first executable core small enough that one person can
  understand the whole path.

### 2. Constraints Should Shape Syntax And Runtime

MicroQuickJS shows that a strict subset can be a strength when it removes
expensive or error-prone behavior.

Design consequence:

- Hum profiles should be allowed to forbid dynamic allocation, recursion,
  reflection, hidden globals, implicit boxing, dynamic dispatch, runtime eval,
  and debug-info columns when budgets require it.
- A strict embedded subset should still be normal Hum, not a separate language.
- Diagnostics should explain the cheaper substitute when a profile rejects a
  construct.

### 3. Infrastructure Beats Demos

FFmpeg, QEMU, TCC, QuickJS, and TinyEMU became useful because they sit at leverage
points: media, emulation, compilation, language runtime, and executable systems
experiments.

Design consequence:

- Hum should prioritize boring leverage tools: checker, formatter, semantic
  graph, package/build evidence, interpreter, debugger facts, profiler facts,
  reproducible artifacts.
- A flashy surface feature should lose to a small infrastructure feature that
  unlocks many future tools.

### 4. Algorithmic Work Beats Hardware Brute Force

The pi work, compression work, codec work, and emulator work all point at the
same lesson: careful algorithms and representation choices matter more than
assuming better hardware will arrive.

Design consequence:

- `cost:` should eventually include memory floor, peak memory, binary size,
  startup time, cache behavior, and artifact determinism.
- Optimization claims need serious baselines and hardware details.
- Standard-library labs should prefer simple dense representations before clever
  instruction tricks.

### 5. Portability Needs Executable Artifacts

JSLinux and TinyEMU turn portability into something users can run. That is more
persuasive than a claim that the architecture is portable.

Design consequence:

- Hum should eventually have small executable portability demos: interpreter,
  Wasm/WASI component, tiny no-heap profile, and emulator-friendly examples.
- Platform-specific power must remain behind explicit capabilities.
- Backend contracts should include artifact-size, startup, and reproducibility
  facts, not just target names.

### 6. Determinism Is A Product Feature

TSAC's deterministic decode requirement maps directly to Hum's evidence-native
mission. Outputs used for safety, security, finance, and debugging must be
reproducible across machines when the profile promises it.

Design consequence:

- Evidence bundles should record compiler version, target, profile, environment,
  dependency graph, solver/prover config, benchmark config, and cache key.
- Release profiles should distinguish deterministic artifacts from best-effort
  performance artifacts.
- Floating-point, randomness, time, thread scheduling, hardware acceleration, and
  compression must have explicit determinism policy.

### 7. Dynamic-Code Power Needs A Boundary

TCC and QuickJS show the usefulness of compilation as a library and executable
scripts. Hum can eventually support powerful embedding and dynamic execution,
but not as hidden authority.

Design consequence:

- Dynamic compilation, plugins, eval-like behavior, and embedded scripting belong
  behind explicit profiles, capabilities, and audit trails.
- `hum` as a library should expose structured inputs/outputs rather than letting
  embedders scrape terminal text.
- Wasm/WASI or process boundaries should come before native plugin execution.

### 8. One-Person Comprehensible Does Not Mean One-Person Forever

Bellard-style work proves a small expert can build foundational systems, but Hum
must still document, test, and hand off subsystems if it wants adoption.

Design consequence:

- Keep core paths compact enough for deep review.
- Add fixtures, schemas, checks, and docs so other people can maintain them.
- Prefer small complete slices over broad incomplete frameworks.

## The Bellard Test For Hum

Before calling a compiler/runtime feature serious, ask:

1. Can one expert read the whole critical path?
2. Can it run under a named resource budget?
3. Can it start fast enough for interactive use?
4. Can it emit deterministic artifacts when the profile requires that?
5. Can it run without optional network, cloud, or telemetry?
6. Can it be embedded or run through a portable boundary?
7. Can it explain its memory floor, binary size, and dependencies?
8. Can the smaller strict profile reject expensive constructs with useful
   diagnostics?
9. Can it be tested against a serious baseline instead of a toy benchmark?
10. Can it become infrastructure other tools can use?

## Near-Term Hum Consequences

Milestone 0 should stay focused on non-executing facts, but the roadmap should
reserve space for:

- a `footprint` or `minimal` runtime profile
- binary-size and startup-time facts in backend/profile reports
- deterministic artifact policy in package/build docs
- a tiny interpreter or VM proof before native backend claims
- no-heap and embedded examples once Core Hum can execute
- strict-profile diagnostics for expensive constructs

## What Hum Should Not Copy Blindly

- Obfuscated minimalism. Tiny code is not automatically maintainable code.
- Hero dependency. A public language cannot rely on one person remembering the
  architecture.
- Unsafe C defaults. Hum should learn from compact C systems without inheriting C
  safety holes.
- Benchmark theater. Bellard-style claims are strongest when tied to concrete
  artifacts, constraints, and reproducible measurements.

## Sources

- Fabrice Bellard homepage: https://bellard.org/
- QuickJS: https://bellard.org/quickjs/
- MicroQuickJS: https://github.com/bellard/mquickjs
- TCC: https://bellard.org/tcc/
- JSLinux technical notes: https://bellard.org/jslinux/tech.html
- TinyEMU: https://bellard.org/tinyemu/
- TSAC: https://bellard.org/tsac/
- Pi computation record: https://bellard.org/pi/pi2700e9/
- QEMU: https://www.qemu.org/
- FFmpeg: https://ffmpeg.org/