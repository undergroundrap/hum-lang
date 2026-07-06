# Hum Evidence Bundle Layout 0.1

Date: 2026-07-06

## Purpose

The evidence bundle is the directory a reviewer inspects after running an
`offline-tool@0.1` Hum tool.

The bundle should be deterministic enough for golden-file checks and readable
enough for a skeptical engineer.

## Directory Shape

```text
evidence/
  manifest.json
  claims-matrix.v0.1.json
  semantic-graph.json
  diagnostics.json
  effect-report.json
  profile-report.json
  run-trace.json
  file-hashes.sha256
  tests/
    generated-tests.json
  humgate/
    sbom-comparison.json
    provenance-comparison.json
    policy-results.json
  benchmarks/
    benchmark-report.json
```

## Manifest

`manifest.json` should contain:

- evidence bundle schema
- Hum version
- compiler commit
- profile
- run command shape without local absolute paths
- created artifact list
- artifact digests
- status summary

## Required Artifacts

A public HumGate alpha run must emit:

- `semantic-graph.json`
- `diagnostics.json`
- `effect-report.json`
- `profile-report.json`
- `run-trace.json`
- `file-hashes.sha256`
- `humgate/sbom-comparison.json`
- `humgate/provenance-comparison.json`
- `humgate/policy-results.json`

If generated tests or benchmarks are not complete, the bundle must emit an
explicit placeholder artifact with `status: "planned"` or `status: "unknown"`.
It must not silently omit the claim.

## Path Rule

Evidence files must use repo-relative or bundle-relative paths. Do not write
user home paths, local editor paths, temporary directories, or machine-specific
absolute paths into public golden evidence.

## Determinism Rule

For fixed source, inputs, toolchain, and output directory, the evidence bundle
should be byte-identical across repeated runs. Any exception must be listed in
`manifest.json`.
