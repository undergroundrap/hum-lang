# Hum Alpha 0.1 Claims Matrix

Date: 2026-07-06

## Purpose

The claims matrix prevents Hum from making public alpha claims that are broader
than the evidence.

Machine-readable source:

- [alpha/claims-matrix.v0.1.json](alpha/claims-matrix.v0.1.json)

Validation:

```powershell
.\tools\check_alpha_claims.ps1
```

## Status Values

- `planned`: accepted target, not implemented
- `chartered`: documented and scoped, not implemented
- `in_progress`: implementation or proof is underway
- `blocked`: known blocker prevents progress
- `proven`: acceptance criteria pass with evidence
- `deferred`: intentionally outside the alpha

## Current Claims

| ID | Status | Claim |
|---|---|---|
| `HA01` | `chartered` | Hum 0.1-alpha is an offline evidence-producing profile, not a general-purpose replacement claim. |
| `HA02` | `chartered` | `offline-tool@0.1` denies network, process execution, unsafe, FFI, threads, hidden mutation, and undeclared IO. |
| `HA03` | `planned` | Alpha runs are deterministic for fixed inputs, source, toolchain, and output directory. |
| `HA04` | `planned` | Alpha evidence includes semantic graph output for every accepted Hum source file. |
| `HA05` | `planned` | Alpha diagnostics are stable and machine-readable. |
| `HA06` | `planned` | Alpha evidence includes declared versus observed effects and profile status. |
| `HA07` | `planned` | Alpha evidence includes hashes, SBOM/provenance comparison, and run trace artifacts for HumGate. |
| `HA08` | `planned` | HumGate is the first killer demo and has fixed fixtures plus golden evidence outputs. |
| `HA09` | `chartered` | Public alpha copy must avoid Rust/C++ performance superiority and production-service claims. |
| `HA10` | `chartered` | A public claim is not allowed unless it appears in this matrix with evidence and acceptance criteria. |

## Release Rule

A public alpha announcement may only describe `proven` claims as working.
`planned`, `chartered`, and `in_progress` claims may be described only as roadmap
or scope.
