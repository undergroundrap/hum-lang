# Hum Alpha 0.1 Charter

Date: 2026-07-06

## Purpose

This charter defines what Hum `0.1-alpha` is allowed to claim.

The alpha is not a general-purpose systems-language launch. It is a narrow,
evidence-native proof that Hum can build deterministic offline tools with
inspectable authority, effects, diagnostics, and evidence.

Related ground truth:

- [OFFLINE_TOOL_ALPHA_0_1.md](OFFLINE_TOOL_ALPHA_0_1.md)
- [ALPHA_THREAT_MODEL_0_1.md](ALPHA_THREAT_MODEL_0_1.md)
- [ALPHA_CLAIMS_MATRIX_0_1.md](ALPHA_CLAIMS_MATRIX_0_1.md)
- [alpha/claims-matrix.v0.1.json](alpha/claims-matrix.v0.1.json)

## Alpha Promise

```text
Hum 0.1-alpha lets a user run a deterministic local Hum tool that reads only
declared inputs, writes only declared evidence outputs, rejects undeclared
authority, and emits machine-readable review artifacts.
```

## Non-Promises

Hum `0.1-alpha` must not claim:

- production service readiness
- Rust, C++, or Go replacement status
- native code generation
- FFI support
- unsafe support
- network service support
- package registry support
- async, threads, plugins, or macros
- benchmark superiority over established systems languages
- certification, compliance, or regulatory approval

## Target User

The first credible user is a skeptical security, DevOps/SRE, or defense/offline
tooling engineer who wants a local tool they can inspect, run without network
access, and audit by reading generated evidence.

## Supported Alpha Shape

The only executable alpha profile is:

```text
offline-tool@0.1
```

The profile is deterministic, file-only, no-network, no-unsafe, and bounded.

## Required Public Artifacts

Before any public alpha announcement, the repo must include:

- alpha charter
- alpha threat model
- alpha claims matrix
- pinned alpha language subset
- diagnostics schema notes for alpha
- semantic graph schema notes for alpha
- effect report schema
- profile report schema
- evidence bundle layout
- HumGate fixtures and golden outputs
- reproducibility and benchmark report

## Claim Rule

Every public alpha claim must have:

- a stable claim ID
- a scope
- a status
- current evidence paths
- acceptance criteria
- future evidence if not yet proven

The machine-readable source is
[alpha/claims-matrix.v0.1.json](alpha/claims-matrix.v0.1.json).

## Current Status

Hum is still pre-alpha at version `0.0.1`.

This charter creates the target for `0.1-alpha`; it does not declare that target
complete.
