# Cranelift Lowering Contract Probe Report

Date: 2026-07-27

This report is populated from the reproducible `probe` command after the
experiment checks run. The durable contract is [CONTRACT.md](CONTRACT.md).

## Result

`NO-GO`, which is a successful contract result.

Cranelift initialized for `x86_64-pc-windows-msvc`, and the clean published Hum
compiler accepted the real source. The attempted conversion to a backend input
then stopped before emitting any CLIF instruction:

```text
code=verified_backend_input_artifact_absent_v0
requirement=one verifier-bound backend input
owner=Hum IR emission plus ir_verify
observed=schema=hum.core_lower.v0;
  lowering_status=unverified_core_artifact_v0;
  core_verification=verified_non_executing_core_artifact_v0;
  ir_ready=0;
  missing_passes=allocation_resource_check,profile_check,ir_verify
```

This is the first honest stopping point. The current independent CLI reports
cannot be combined into authority, and lowering from their text, spans, names,
or ordering would be fabricated.

## Provenance

- repository commit:
  `e745b5f2d1fec9a11b68c73d6f292ce1859880f1`
- compiler source: a clean `git archive` of that commit, built outside the live
  worktree so the preserved unaccepted 10B.3 bytes were not used as evidence
- Hum executable SHA-256:
  `3a6ca8b68c9a6aee088293016c188453c36a2b4a432e9aadb40eb01a36ce13d1`
- source: `examples/core/minimal_add.hum`
- source SHA-256:
  `5b4f324e9f281cb4117733efa523456cc5ada212db787d295b560d8a34fb9d88`
- Cranelift: `0.133.1`
- Rust: `rustc 1.96.0 (ac68faa20 2026-05-25)`
- Cargo: `cargo 1.96.0 (30a34c682 2026-05-25)`
- experiment lockfile SHA-256:
  `9887f2f9b05328852c9ffd19385a6a973c2ca303617ebbd4bf48f786af021cfe`

## Exact Commands

```powershell
git archive --format=zip --output=<temp>\hum-contract-baseline-e745b5f2.zip `
  e745b5f2d1fec9a11b68c73d6f292ce1859880f1

Expand-Archive `
  <temp>\hum-contract-baseline-e745b5f2.zip `
  <temp>\hum-contract-baseline-e745b5f2

cargo build --offline --bin hum `
  --manifest-path <temp>\hum-contract-baseline-e745b5f2\Cargo.toml

$env:HUM_BIN = `
  "<temp>\hum-contract-baseline-e745b5f2\target\debug\hum.exe"

cargo run --offline --quiet `
  --manifest-path experiments/cranelift-lowering-contract/Cargo.toml -- probe
```

The probe itself ran these real compiler commands against the same source:

```text
hum check --format=json
hum core-lower --format=json
hum core-verify --format=json
hum resolve --format=json
hum full-type-check --format=json
hum effect-check --format=json
hum ownership-check --format=json
hum resource-check --format=json
hum profile-check --format=json
hum ir-readiness --format=json
```

## Measured Attempt

The clean-baseline probe completed in 2,893 ms:

| Stage | Exit | Elapsed |
| --- | ---: | ---: |
| `check` | 0 | 308 ms |
| `core-lower` | 0 | 64 ms |
| `core-verify` | 0 | 66 ms |
| `resolve` | 0 | 53 ms |
| `full-type-check` | 0 | 85 ms |
| `effect-check` | 0 | 113 ms |
| `ownership-check` | 0 | 172 ms |
| `resource-check` | 1 | 293 ms |
| `profile-check` | 1 | 547 ms |
| `ir-readiness` | 0 | 1,186 ms |

A second fresh validation run completed in 2,580 ms and reproduced the same
schemas, exits, facts, stop code, zero-CLIF result, and GO/NO-GO decisions.
Timing fields are observational and are not part of semantic equality.

The nonzero resource/profile exits are retained evidence, not ignored failures.
Both commands emitted valid canonical blocker reports. Resource checking
rejected the program for
`task_body_requires_explicit_allocates_intent_v0`; profile checking then
reported `blocked_by_resource_check_errors`.

## Facts Observed

- `core-lower` reports a three-node binary `add`, but exposes expression text
  and a node count rather than canonical nodes and ordered child IDs.
- `resolve` reports two resolved references and their public definition IDs,
  but does not expose the private canonical node IDs or a Core-operand binding
  table.
- `full-type-check` reports the statement result as `Int`, while the
  corresponding Core expression remains `not_type_checked_v0` with no attached
  type.
- effect and ownership checks accept the example, but their reports are not
  tied to the Core expression in one verified artifact.
- resource and profile policy block IR readiness.
- `core-verify` reports successful non-executing invariant checks, but returns
  no capability bound to backend-input bytes.
- `ir_verify` is not implemented, and no Hum IR/backend-input artifact exists.

## Artifacts

The only retained experiment artifacts are source, documentation, pinned
dependency metadata, and the executable probe. Build products remain ignored
under `experiments/cranelift-lowering-contract/target/`.

No object file, executable, or CLIF was emitted from Hum facts. That absence is
intentional: the required verified input did not exist.

## Limitations

- One source program and one function were examined.
- The experiment defines the minimum contract for checked integer addition; it
  does not define the complete Hum IR.
- It does not propose production file envelopes or convergence increments.
- It does not claim that the inspection JSON is a security boundary.
- It does not change Hum semantics, production architecture, or the backend
  roadmap by itself.

The complete contract and per-gap GO/NO-GO findings are in
[CONTRACT.md](CONTRACT.md).
