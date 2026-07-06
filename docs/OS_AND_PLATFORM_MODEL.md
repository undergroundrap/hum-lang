# Hum OS And Platform Model

Date: 2026-07-06

## Purpose

Hum needs an OS model because systems languages do not run in a vacuum.

Files, processes, services, drivers, devices, network sockets, registry keys, environment variables, secrets, clocks, event traces, installers, and updates are authority boundaries.

The platform rule is:

```text
Windows-first proof. Portable-by-design architecture.
```

## Windows-First Does Not Mean Windows-Only

Early platform proof targets Windows because the current development and verification path is Windows-first. That is honest and useful.

Hum should still avoid baking Windows assumptions into the language core:

- path rules are platform facts, not string folklore
- process creation is a capability, not a hidden helper
- services are platform artifacts, not normal functions
- registry access is a Windows capability, not generic storage
- drivers are strict-profile artifacts, not ordinary libraries
- telemetry is a declared signal schema, not random logging
- installers and update behavior are generated artifacts with evidence

The language core should model capabilities. Platform adapters should map those capabilities to Windows, Linux, macOS, WASI, embedded, and future targets.

## Research Anchors

This model follows current platform evidence:

- [Microsoft Windows driver docs](https://learn.microsoft.com/en-us/windows-hardware/drivers/gettingstarted/)
- [Windows Driver Frameworks](https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/)
- [Microsoft driver security checklist](https://learn.microsoft.com/en-us/windows-hardware/drivers/driversecurity/driver-security-checklist)
- [Windows device and driver installation](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/)
- [Microsoft Rust on Windows overview](https://learn.microsoft.com/en-us/windows/dev-environment/rust/overview)
- [microsoft/windows-rs](https://github.com/microsoft/windows-rs)
- [microsoft/windows-drivers-rs](https://github.com/microsoft/windows-drivers-rs)
- [Event Tracing for Windows](https://learn.microsoft.com/en-us/windows/win32/etw/about-event-tracing)
- [Windows services](https://learn.microsoft.com/en-us/windows/win32/services/services)

## Platform Principle

Every OS interaction should answer:

```text
What platform authority is required?
What can this read?
What can this change?
What privileges are needed?
What evidence proves the generated artifact is safe?
What target platforms implement this capability?
What happens when the capability is absent?
```

Hum should make OS authority visible in source and graph facts.

## OS Capability Families

Hum should eventually model these as graph-visible capabilities:

- `os.files`
- `os.directories`
- `os.registry`
- `os.environment`
- `os.processes`
- `os.threads`
- `os.services`
- `os.scheduled_tasks`
- `os.devices`
- `os.drivers`
- `os.network`
- `os.firewall`
- `os.clock`
- `os.random`
- `os.secrets`
- `os.identity`
- `os.telemetry`
- `os.installer`
- `os.updates`

These names are not final stdlib APIs. They are the authority families the semantic graph must be able to represent.

## Windows Service Gate

Windows services are a good future platform artifact because they are powerful but still safer than kernel drivers.

A Hum service package should eventually declare:

- service name and display name
- account identity and privileges
- start, stop, pause, resume, and shutdown behavior
- recovery policy
- event log or ETW schema
- filesystem and registry access
- network ports and outbound destinations
- configuration source and secret policy
- install, upgrade, rollback, and uninstall plan

Nectar should generate or validate service artifacts. Milestone 0 should only reserve graph fields; it should not install services.

## Driver Gate

Driver work is a future strict-profile topic.

Hum should not claim driver support until it can model:

- whether a kernel driver is truly required
- whether a user-mode service or UMDF path is safer
- WDF, KMDF, and UMDF target model
- device interface and access-control policy
- IOCTL surface and buffer policy
- memory ownership, locking, DMA, and lifetime rules
- Driver Verifier, CodeQL, HLK, signing, and release evidence
- driver package installation and update behavior
- crash, rollback, and recovery implications

Default rule:

```text
Do not enter kernel mode when a user-mode service, app, or UMDF path is enough.
```

Hum can eventually be attractive for driver-adjacent work if it makes access control, IOCTL buffers, privilege, memory bounds, and update evidence visible. That is a long-term profile, not a near-term promise.

## Rust And Windows API Strategy

The current bootstrap is Rust, which fits the Windows-first path.

Hum should learn from Rust-on-Windows and `windows-rs`:

- prefer focused platform bindings over importing the entire OS surface
- wrap raw APIs in typed capabilities
- make ownership, nullability, string encoding, handles, and error propagation explicit
- keep Windows-specific crates and generated bindings behind platform adapters
- expose platform API use in `hum graph`

Hum should not attempt broad Windows API coverage before the formal core, unsafe policy, FFI policy, and package evidence model exist.

## Observability

Windows has a serious tracing model through Event Tracing for Windows.

Hum should not treat logs as loose strings. Platform observability should become declared source and graph facts:

- events
- spans
- counters
- metrics
- event IDs
- privacy and redaction rules
- retention and rate limits
- profile-required telemetry

Future Hum source should be able to say:

```text
emits:
  event service.started
  metric request.latency

observes:
  os.process.memory
  os.thread.count
```

For Windows targets, those facts can eventually lower into ETW, Event Log, or other platform-specific sinks.

## Installation And Updates

Installers and updates are mutation-heavy OS operations.

Hum should require:

- dry-run before mutation
- rollback plan
- generated artifact tracking
- signatures and hashes
- target OS and architecture
- service, driver, or package identity
- least-privilege install authority
- user consent and enterprise policy awareness

Milestone 0 must not install, update, sign, or publish anything. It should only reserve graph fields and diagnostics.

## Portable-By-Design Target Tiers

### Tier 0: Language-Only

Parser, diagnostics, formatter, semantic graph, tests, and docs work without OS-specific behavior.

### Tier 1: Windows Verified

Windows is the first tested target for CLI behavior, paths, process behavior, file handling, timings, and eventual platform fixtures.

### Tier 2: Portable Runtime

Linux, macOS, and WASI targets are designed but not claimed until tested.

### Tier 3: Specialized Targets

Embedded, no-heap, realtime, driver, kernel, accelerator, and certified toolchain targets require strict profiles and evidence packets.

## Semantic Graph Requirements

`hum graph` should eventually expose:

- target OS and architecture
- platform profile
- required OS capabilities
- privilege expectations
- read/write OS resources
- process, service, and thread behavior
- registry keys and filesystem paths
- network ports and endpoints
- telemetry schema
- install/update/dry-run/rollback facts
- driver/service package metadata
- generated artifact hashes
- platform adapter identity
- portability notes

## Microsoft Adoption Reality

Microsoft is unlikely to adopt Hum merely because Hum starts Windows-focused.

Microsoft would need evidence: memory safety, Windows API interop, MSVC/LLVM/Rust bridge, excellent diagnostics and IDE integration, stable governance, security review, supply-chain posture, migration story, and real user value.

Windows-first is still a good strategy because it produces concrete proof on the platform currently used for local verification.

The claim should be:

```text
Hum is first proven on Windows.
Hum is designed so other platforms are first-class later.
```

## Near-Term Work

1. Add future OS/platform fields to the semantic graph schema.
2. Add `windows service` and `driver candidate` runtime profiles as strict future profiles.
3. Create non-executing `.hum` fixtures for a Windows service manifest and an OS capability declaration.
4. Define path, process, environment, clock, random, filesystem, and registry capability names.
5. Keep all platform work local and non-mutating until Milestone 0 is complete.

## Brutal Rule

Hum's OS story is credible only if platform power is visible.

If a program touches services, registry, drivers, devices, installers, updates, network, secrets, or telemetry, Hum should make that authority obvious to the human, compiler, package manager, reviewer, and agent.
