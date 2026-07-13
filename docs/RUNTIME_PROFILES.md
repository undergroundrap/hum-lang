# Hum Runtime Profiles

Date: 2026-07-06

Exact diagnostic allocations come only from
[`src/diagnostic_catalog.rs`](../src/diagnostic_catalog.rs); the checked human
projection is [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Purpose

Runtime profiles are named language/toolchain policies for different kinds of
systems work.

Hum should not pretend one build mode can serve a beginner script, a game engine
hot path, an implantable medical device, and a verified embedded controller.

Profiles make the rules explicit.

## Profile Principle

```text
A profile is a contract between source code, compiler checks, Nectar builds,
stdlib APIs, generated evidence, and runtime behavior.
```

Profiles should be visible in source, package metadata, semantic graph output,
and release artifacts.

## Current Machine-Readable Surface

`hum profiles` and `hum profiles --format json` expose the V0 profile catalog.
The JSON catalog uses `hum.runtime_profiles.v0`; each profile entry uses
`hum.runtime_profile.v0`.

V0 mode is `contract_only_no_profile_enforcement`. The command catalogs profile
policy, capability intent, required evidence categories, rules, and non-goals. It
must not claim profile syntax enforcement, stdlib narrowing, executable runtime
behavior, target selection, certification, host probing, or performance and
footprint measurement.

The schema document is [HUM_RUNTIME_PROFILES_SCHEMA.md](HUM_RUNTIME_PROFILES_SCHEMA.md). `hum profile-check --format json` consumes this catalog, accepts the default `normal` profile, and blocks known strict profiles until enforcement and evidence checks exist.

## Candidate Profiles

### `normal`

Default profile for ordinary checked Hum programs.

Allows:

- ordinary allocation
- ordinary panics for contract bugs
- standard diagnostics
- normal stdlib

Still forbids unsafe behavior unless explicitly declared.

### `containerized service`

For services intended to run under Docker, OCI runtimes, Kubernetes, and similar
container schedulers.

Forbids by default:

- privileged containers
- host filesystem mounts
- host network, host PID, or host IPC
- Docker socket access
- hidden listening ports
- hidden outbound network authority
- undeclared environment secret reads
- unbounded logs

Requires:

- declared ports and protocols
- health/readiness behavior
- graceful shutdown behavior
- CPU, memory, file descriptor, and process budgets
- filesystem mount policy
- user identity policy
- capability/seccomp policy where applicable
- telemetry schema
- SBOM and provenance evidence in release profiles

### `agent tool sandbox`

For MCP servers, agent-callable CLIs, IDE tools, CI repair tools, and codegen
helpers.

Forbids by default:

- token passthrough
- raw shell command strings
- hidden network access
- hidden repo mutation
- reading secrets without a declared capability
- trusting tool descriptions as proof of behavior

Requires:

- exact input schema
- exact output schema
- declared read/write capabilities
- dry-run behavior for mutation
- audit log event schema
- secret redaction policy
- prompt-injection risk notes when output returns to an agent
- sandbox boundary recommendation: process, container, or Wasm

### `windows service`

For long-running Windows services and service-like platform daemons.

Forbids by default:

- hidden service installation
- hidden administrator authority
- LocalSystem use without profile evidence
- registry writes without declared keys
- filesystem writes outside declared paths
- network listeners without declared ports
- event-log or ETW spam without rate policy

Requires:

- service identity and privilege policy
- start, stop, pause, resume, and shutdown behavior
- recovery and restart behavior
- install, upgrade, rollback, and uninstall plan
- event log or ETW schema
- configuration and secret policy
- filesystem, registry, and network authority declaration

### `driver candidate`

For future driver or driver-adjacent work. This is a strict future profile, not a
Milestone 0 capability.

Forbids by default:

- kernel-mode code without BDFR approval and review packet
- production signing of test driver code
- hidden IOCTL surface
- hidden device object access
- unchecked direct buffer handling
- hardware or kernel access without target identity

Requires:

- proof that a driver is required instead of a user-mode app, service, or UMDF path
- WDF, KMDF, or UMDF model decision
- device interface and access-control policy
- IOCTL and buffer contract
- memory, locking, DMA, and lifetime evidence where applicable
- Driver Verifier, CodeQL, HLK, signing, and release evidence before any real release

### `footprint constrained`

For small tools, embedded runtimes, bootstraps, browser/VM demos, rescue tools,
plugins, and constrained deployment targets where size and startup are product
facts.

Forbids by default:

- hidden runtime services
- hidden dynamic code loading
- hidden background threads
- hidden network or cloud dependency
- unbounded startup work
- unbudgeted binary-size growth

Requires:

- binary-size budget
- startup-time budget
- memory floor
- dependency count
- deterministic artifact policy
- optional debug-info size policy
- portability boundary: native, interpreter, process, Wasm, or emulator

### `embedded no heap`

For microcontrollers, firmware, drivers, and constrained devices.

Forbids:

- hidden heap allocation
- dynamic dispatch requiring runtime allocation
- unbounded recursion
- implicit large stack objects

Requires:

- stack estimate
- static memory map
- target description
- no-heap stdlib subset

### `hard realtime`

For control loops, audio engines, robotics, medical control, and engine jobs with
strict deadlines.

Forbids:

- unbounded allocation
- blocking IO
- unbounded locks
- hidden background reclamation
- runtime code generation

Requires:

- deadline behavior
- WCET estimate or measured bound
- stack bound
- scheduling policy
- watchdog/fail-safe behavior

### `engine hot path`

For frame-critical engine/runtime code.

Forbids:

- unbudgeted per-frame allocation
- blocking asset IO
- hidden locks
- logging without a rate limit
- shader or asset compilation in hot path
- accidental virtual dispatch in tight loops

Requires:

- frame budget
- memory budget
- platform profile
- trace labels
- deterministic replay statement

### `safety critical`

General high-assurance profile.

Forbids:

- panic unwind across critical boundaries
- ignored `Result`
- unsafe without review packet
- foreign without ABI contract
- implicit numeric narrowing
- unbounded loops without variants or watchdogs

Requires:

- traceability graph
- deterministic build manifest
- dependency evidence packet
- risk-control links
- test/proof evidence
- explicit failure policy

### `medical class c`

A specialization for software that could contribute to death or serious injury.

Adds:

- medical risk traceability
- software item classification
- SOUP/dependency evidence
- problem-resolution evidence
- cybersecurity evidence when connected
- release and maintenance artifact requirements

### `automotive asil d`

A specialization for automotive high-integrity work.

Adds:

- ASIL traceability
- hazard analysis links
- safety goal links
- freedom-from-interference evidence
- toolchain qualification evidence
- SOTIF links when functionality insufficiency matters

### `certified toolchain`

For builds where the compiler, stdlib subset, and tools are part of the audited
evidence.

Requires:

- compiler version lock
- stdlib subset lock
- target lock
- artifact hashes
- tool qualification packet
- migration report for upgrades

## Profile Interactions

Profiles can stack only when their rules do not conflict:

```text
profile medical class c + hard realtime + embedded no heap
profile engine hot path + deterministic replay
profile automotive asil d + certified toolchain
```

If profiles conflict, the compiler should explain the conflict.

Example:

```text
error[<unallocated-profile-diagnostic>]: profile `hard realtime` forbids background epoch reclamation
help: use bounded arena reclamation or move this task outside the realtime loop
```

`H1100-H1199` is the reserved `runtime_profile_policy` family. No exact profile
diagnostic, including an apparent first member of that interval, is allocated.

## Semantic Graph Requirements

`hum graph` should eventually expose:

- active profiles
- forbidden features
- required evidence
- contract coverage
- dependency evidence status
- allocation policy
- panic/failure policy
- realtime budget status
- toolchain identity
- target identity
- container image identity
- OS/platform authority
- service/driver/install/update metadata
- runtime resource budgets
- binary-size budget
- startup-time budget
- memory floor
- dependency count
- deterministic artifact policy
- exposed ports and protocols
- observability signal schema
- agent tool schemas and capabilities

## Nectar Requirements

Nectar should eventually own:

- profile selection
- reproducible build manifests
- dependency evidence packets
- SBOM generation
- toolchain lockfiles
- target profiles
- benchmark profiles
- release artifact bundles
- OCI/container metadata
- SBOM and provenance artifacts
- agent tool manifests

## Relationship To Language Design

Profiles are a reason to keep Hum small.

Every new feature should answer:

```text
What profiles allow it?
What profiles forbid it?
What evidence does it require?
What does it do to traceability?
```

If a feature cannot be profiled, it is not ready.

## Near-Term Work

1. Link active profile declarations from the semantic graph once profile syntax is pinned.
2. Allocate exact profile diagnostics only through the canonical registry after
   the profile rules are implemented and independently reviewed.
3. Create `.hum` fixtures for `engine hot path`, `hard realtime`, `safety critical`, `containerized service`, `agent tool sandbox`, and `footprint constrained`.
4. Define `panic`/`abort`/`safe stop` behavior in the core language spec.
5. Define allocation policy sections for profiles.
6. Add Nectar profile metadata design.

## Brutal Rule

Hum's safety story is credible only when the strict profiles are boring.

The exciting language features can exist in normal Hum. The life-saving profiles
must be smaller, stricter, more traceable, and easier to audit.
