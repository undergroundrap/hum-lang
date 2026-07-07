# Hum Portability Boundary Model

Date: 2026-07-07

Status: reference boundary, not an implementation claim

## Purpose

Hum should be portable because platform authority is explicit, not because
source code pretends every machine is the same.

This document names the boundary between Core Hum, runtime profiles, backend
adapters, OS/platform adapters, and target artifacts. It complements
[INTEROP_AND_PORTABILITY.md](INTEROP_AND_PORTABILITY.md), which focuses on
foreign code and ecosystem adoption, and [OS_AND_PLATFORM_MODEL.md](OS_AND_PLATFORM_MODEL.md),
which focuses on Windows-first OS authority.

Milestone 0 does not execute Hum programs, select targets, compile artifacts,
run packages, call foreign code, or probe host capabilities. This model is the
contract future executable work must satisfy before Hum claims serious embedded,
enterprise, safety-critical, or cross-platform readiness.

## Boundary Rule

Portable Hum code must not depend on hidden platform facts.

Every platform-sensitive operation should eventually answer:

```text
What target is this for?
What authority does it require?
What representation facts does it assume?
What happens when the target lacks the capability?
What evidence proves the artifact was built under those facts?
```

If a program cannot answer those questions, it is target-accidental, not
portable.

## Layers

### Surface Hum

Surface Hum names intent:

- `uses:` for read/effect authority
- `changes:` for write/mutation authority
- `protects:` for safety and security properties
- `trusts:` for explicit assumptions
- `cost:` and `allocates:` for resource claims
- runtime profile declarations once profile syntax is pinned

Surface Hum must not hide platform power behind friendly words.

### Core Hum

Core Hum owns executable meaning. A platform-specific surface feature is stable
only when it lowers to a core operation with explicit effects, failure behavior,
profile restrictions, and graph facts.

Core Hum should not include Windows, Linux, macOS, WASI, or embedded APIs
directly. It should include portable operations and effect categories that
platform adapters implement or reject.

### Hum IR

Hum IR owns backend-independent checked semantics. Target lowering consumes Hum
IR plus target facts; it does not get to invent missing semantics.

The IR boundary must preserve:

- value and failure behavior
- declared and inferred effects
- mutation permissions
- layout assumptions
- profile restrictions
- debug/profiling/source-map facts
- evidence and provenance ids

### Backend Adapter

A backend adapter maps Hum IR to an implementation technology such as an
interpreter, Cranelift, LLVM, Wasm/WASI, Rust/C lowering, MLIR, or a future
custom backend.

The backend adapter may optimize. It may not change Hum meaning, erase target
assumptions, or claim portability merely because the backend supports many
targets.

### Platform Adapter

A platform adapter maps Hum capabilities to OS, runtime, hardware, or sandbox
facilities. It owns the target-specific bridge for files, paths, clocks,
randomness, process behavior, network authority, devices, services, registry,
signals, environment variables, and observability sinks.

Platform adapters should be small, auditable, and profile-gated.

### Artifact

An artifact is the thing users run, inspect, deploy, or publish:

- interpreted bundle
- native binary
- Wasm component
- package archive
- service manifest
- container image
- firmware image
- evidence bundle

Artifacts need provenance, target facts, dependency facts, profile facts, and
reproducibility status before Hum can make serious release claims.

## Target Facts

Every build or analysis target should eventually have a target fact record:

```text
target:
  triple: windows-x86_64-msvc
  os: windows
  arch: x86_64
  abi: msvc
  endian: little
  pointer width: 64
  path kind: windows
  newline policy: preserve
  filesystem: available
  process: available
  network: denied by profile
  clock: monotonic available
  random: system available
  atomics: u64 available
  simd: sse2 baseline
```

Field names are illustrative, not final syntax. The current machine-readable V0 surface is [TARGET_FACTS_SCHEMA.md](TARGET_FACTS_SCHEMA.md), emitted by `hum target-facts --format json`, with fixture records under [../fixtures/target_facts](../fixtures/target_facts). The requirement is that target facts stay machine-readable before target-sensitive code depends on them.

## Capability Families

Hum should separate platform authority into families that profiles and semantic
graph facts can reason about:

| Family | Examples | Default V0 Status |
| --- | --- | --- |
| `target.layout` | endian, alignment, pointer width, ABI | design |
| `target.cpu` | atomics, SIMD, accelerator features | design |
| `target.memory` | heap, stack, pages, DMA, shared memory | design |
| `target.path` | separator, case sensitivity, roots, reserved names | design |
| `os.filesystem` | read, write, create, delete, watch | design |
| `os.clock` | monotonic time, wall time, timers | design |
| `os.random` | system entropy, deterministic seed | design |
| `os.process` | spawn, exec, exit, signals, environment | design |
| `os.network` | sockets, DNS, TLS, ports, endpoints | design |
| `os.identity` | users, groups, privileges, tokens | design |
| `os.device` | serial, HID, GPU, sensors, registers | design |
| `os.service` | Windows service, launchd, systemd | design |
| `sandbox.host` | WASI imports, browser APIs, plugin host calls | design |
| `artifact.release` | signing, hashes, SBOM, provenance | design |

Milestone 0 may document and reserve these families. It must not pretend to
enforce them at runtime.

## Absence Is A First-Class Case

Portable code must say what happens when a capability is unavailable.

Examples:

- network denied by profile
- filesystem absent on a sandbox target
- wall clock forbidden for deterministic replay
- random source unavailable during a reproducible build
- atomic `u64` unsupported on a small embedded target
- dynamic allocation forbidden by a no-heap profile

The compiler should eventually diagnose accidental dependency on absent
capabilities before artifact generation.

## Profiles And Portability

Runtime profiles are portability filters.

Examples:

- `offline-tool@0.1` denies network, process execution, FFI, unsafe, wall-clock,
  random, plugin, dynamic-load, and arbitrary build-script authority.
- `footprint constrained` requires binary-size budget, startup-time budget,
  memory floor, dependency count, deterministic artifact policy, and portability
  boundary.
- `embedded no heap` forbids hidden allocation and broad OS authority.
- `hard realtime` forbids unbounded blocking, hidden allocation, and surprise
  scheduling behavior.
- `safety critical` requires traceable assumptions, checked failure policy, and
  reviewable evidence.

Profiles should fail closed: when a target fact is unknown and the profile
depends on it, the profile should reject or request evidence rather than assume
success.

## Paths

Paths are not strings.

Hum should distinguish:

- logical package paths
- source paths
- artifact paths
- OS filesystem paths
- URI-like locations
- sandbox virtual paths

Platform adapters own path conversion. Source code should not silently depend on
Windows drive letters, POSIX roots, separator characters, case sensitivity,
Unicode normalization, reserved names, symlink behavior, or current working
directory rules.

## Time And Randomness

Time and randomness are platform authority.

Hum should distinguish:

- monotonic time
- wall-clock time
- deterministic replay time
- build time
- benchmark time
- system entropy
- deterministic seed
- cryptographic randomness

Safety, finance, aerospace, medical, test, and replay profiles need different
policies. No profile should receive hidden wall-clock or random behavior through
ordinary source.

## Filesystem And Process

Filesystem and process behavior are high-authority boundaries.

Hum should require explicit declarations for:

- readable roots
- writable roots
- create/delete authority
- symlink and junction behavior
- executable path authority
- environment variables
- working directory
- stdin/stdout/stderr limits
- timeout and cancellation
- exit-status mapping

The process boundary should be typed. Shell strings should not be ordinary
portable APIs.

## Network And Services

Network authority should be denied by strict profiles unless declared.

Future network declarations should name:

- protocol
- bind address
- remote endpoint
- DNS behavior
- TLS policy
- timeout and retry policy
- authentication and secret handling
- logging and redaction policy

Service declarations should name lifecycle, identity, privilege, health,
recovery, observability, install, upgrade, rollback, and uninstall behavior.

## Devices And Embedded Targets

Device and embedded targets need stricter boundaries than normal hosted apps.

Future declarations should expose:

- memory-mapped regions
- registers
- interrupts
- DMA
- pin or bus ownership
- volatile access policy
- clock domains
- allocation policy
- panic/fail-stop behavior
- power and startup constraints

This belongs behind strict profiles. It is not ordinary Hum source.

## Artifact Evidence

A portable artifact should eventually record:

- Hum compiler version
- source edition
- schema versions
- target fact record
- runtime profile
- backend adapter
- platform adapter
- dependency graph
- build inputs
- generated outputs
- hashes
- signature status
- reproducibility status
- evidence bundle link

Without these facts, Hum can build a file but should not claim a portable,
auditable release.

## Semantic Graph Requirements

The current `hum.semantic_graph.v0` now reserves a top-level `portability`
object and fills the source-declared slice from `targets:` sections. Future
semantic graph versions should extend it with:

- target facts used by analysis or build beyond source-declared records
- required capability families
- denied or unavailable capabilities
- path/time/randomness policy
- layout assumptions
- backend and platform adapter identity
- artifact provenance ids
- profile restrictions
- portability diagnostics
- evidence bundle links

Agents, editor plugins, CI wrappers, package tools, and reviewers should read
these facts instead of inferring portability from filenames or comments.

## Diagnostics Direction

Future portability diagnostics should be specific:

```text
error H12xx: profile denies wall-clock time
help: use monotonic time, replay time, or add an explicit profile exception

error H12xx: target path semantics are ambiguous
help: declare whether this path is a package path, source path, artifact path,
or OS filesystem path

error H12xx: target does not provide atomic u64
help: use a smaller atomic, add a target requirement, or choose a profile that
allows a fallback lock
```

Codes are illustrative. Final codes belong in [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Rejected Shortcuts

Hum should reject:

- treating paths as plain strings everywhere
- assuming little-endian or 64-bit pointers silently
- assuming wall-clock time exists
- assuming system randomness exists
- hidden network access during build or package install
- hidden process execution from packages
- host-specific generated code without target provenance
- claiming Windows support from Linux-only tests, or the reverse
- claiming Wasm support while depending on hidden host calls
- claiming embedded support while requiring heap, OS, or background runtime

## Milestone 0 Boundary

Milestone 0 may:

- document portability families
- keep examples portable
- run text hygiene and public-readiness checks
- emit graph facts from source sections
- keep platform work local and non-mutating

Milestone 0 must not:

- execute Hum programs
- select or probe runtime targets
- download packages
- run foreign build scripts
- emit native, Wasm, service, container, firmware, or driver artifacts
- claim profile enforcement beyond existing parser/checker facts

## Near-Term Work

1. Keep portability boundary links in the language reference, architecture, OS
   model, and interop strategy.
2. Keep `hum target-facts --format json` and fixture records in sync with this
   boundary.
3. Keep the current semantic graph `portability` source declarations in sync
   with target facts and capability absence.
4. Add non-executing fixtures for path, clock, random, filesystem, process, and
   network declarations now that `targets:` has a narrow V0 shape.
5. Add profile diagnostics only after the graph can represent the relevant
   target facts honestly.
6. Keep the first executable core target small and local before promising
   cross-platform artifacts.

## Brutal Rule

Hum portability is credible only when a target-sensitive program can explain its
platform assumptions to a beginner, a maintainer, a verifier, a profiler, a
security reviewer, an operator, and an agent.

If the explanation is not machine-readable, it is not yet a language guarantee.
