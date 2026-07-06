# Hum Offline Tool Alpha 0.1

Date: 2026-07-06

## Purpose

This document defines the smallest credible public alpha target implied by the
current adoption research.

Primary research snapshot:

- [research/2026-07-06-offline-tool-alpha.md](research/2026-07-06-offline-tool-alpha.md)

Alpha gate artifacts:

- [ALPHA_CHARTER_0_1.md](ALPHA_CHARTER_0_1.md)
- [ALPHA_THREAT_MODEL_0_1.md](ALPHA_THREAT_MODEL_0_1.md)
- [ALPHA_CLAIMS_MATRIX_0_1.md](ALPHA_CLAIMS_MATRIX_0_1.md)
- [alpha/claims-matrix.v0.1.json](alpha/claims-matrix.v0.1.json)

## Competition Stance

Hum should eventually compete with Rust, C++, Go, and other serious systems
tools where Hum can prove a better combination of safety, readability,
performance truth, and evidence.

That is the long-term ambition.

The alpha launch posture is narrower: do not claim Hum replaces Rust before Hum
can execute real programs, ship a coherent runtime profile, prove interop,
produce packages, and demonstrate performance boundaries.

For `0.1-alpha`, Hum should win a smaller claim:

```text
Hum can build deterministic offline tools whose authority, effects, inputs,
outputs, diagnostics, and evidence are inspectable by default.
```

That is not retreating from systems-language ambition. It is earning the right
to make the larger claim later.

## Profile

Working profile name:

```text
offline-tool@0.1
```

One-line promise:

```text
Write a deterministic local Hum tool that reads declared input files, performs
bounded checks, writes only to a declared evidence directory, and emits
machine-readable security evidence.
```

Allowed:

- parse and check Hum source
- run bounded Hum tasks
- read declared files and directories
- write only to a declared output directory
- parse strict JSON
- emit canonical JSON evidence
- compute SHA-256 digests
- compare manifests, SBOMs, provenance, and policy facts
- run Hum tests

Denied:

- network
- process execution
- dynamic loading
- FFI
- unsafe
- threads
- wall-clock-dependent logic
- randomness
- environment-variable reads by default
- writes outside the declared output directory
- hidden mutation
- undeclared file reads

## Build Order

The alpha should be built in this dependency order:

1. Alpha charter, threat model, and claims matrix.
2. Hermetic Rust bootstrap build with version and checksum output.
3. Pinned alpha language subset.
4. Canonical JSON writer.
5. Semantic graph schema version for alpha evidence.
6. Diagnostics JSON schema version for alpha repair.
7. Effect and capability checker.
8. Deterministic bounded interpreter.
9. Test runner and generated test skeletons.
10. Evidence bundle writer.
11. HumGate demo fixtures and golden outputs.
12. Benchmark and reproducibility report.

## Killer Demo

Primary demo:

```text
HumGate: Air-Gapped Release And Change Gate
```

HumGate should read a local release or change bundle, verify declared files,
compare SBOM/provenance/policy facts, reject undeclared authority, and emit an
evidence directory.

Required evidence directory:

- semantic graph
- effect report
- profile report
- diagnostics JSON
- generated tests
- observed run trace
- SBOM or SBOM comparison report
- provenance or provenance comparison report
- benchmark report
- claims matrix

Backup demos:

- agent-safe runbook dry-run gate
- offline SBOM/provenance verifier

HumGate is preferred because it exercises Hum's actual differentiator: checked
intent becoming operational evidence.

## Alpha Cut Line

Must be in alpha:

- executable bounded interpreter for the profile
- declared file authority and output-directory confinement
- stable machine-readable diagnostics
- semantic graph for all alpha constructs
- generated tests or test skeletons from intent blocks
- effect report and profile report
- deterministic run trace
- evidence bundle writer
- repeatable demo fixtures

Must not be in alpha:

- network services
- native code generation
- FFI
- unsafe
- package registry
- async or threads
- plugins
- macros
- production-service claims
- Rust/C++ performance superiority claims

## Success Bar

The alpha succeeds when a skeptical security, DevOps/SRE, or defense/offline
tooling engineer can clone the repo, run HumGate on fixed local fixtures, inspect
the emitted evidence, and understand exactly what the tool was allowed to read,
write, deny, prove, and emit.

The alpha fails if the best explanation is only "the syntax is nicer."
