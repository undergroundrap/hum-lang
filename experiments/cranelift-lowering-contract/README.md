# Cranelift Lowering Contract Probe

This disposable experiment attempts to convert the real checked Hum program
`examples/core/minimal_add.hum` into a Cranelift input. Its success condition is
an exact backend contract or a precise fail-closed stopping point, not generated
code.

It deliberately refuses to infer operands from expression text, parameter order,
source spans, display names, or handwritten expected output. Cranelift target
initialization succeeds, then the probe asks production Hum for its current
Core, resolver, type, effect, ownership, resource, profile, and IR-readiness
artifacts. No Cranelift instruction may be emitted until those facts arrive in
one verifier-bound backend input.

## Run

Build the repository Hum binary first:

```powershell
cargo build --bin hum
```

Then run the probe:

```powershell
cargo run `
  --manifest-path experiments/cranelift-lowering-contract/Cargo.toml -- probe
```

The expected current result is a successful experiment process containing
`"lowering_attempt": "no_go"` and
`"code": "verified_backend_input_artifact_absent_v0"`. A NO-GO is the evidence:
the adapter initialized Cranelift and consumed only real Hum outputs, but emitted
zero CLIF instructions because no trustworthy backend input exists.

See [CONTRACT.md](CONTRACT.md) for the derived contract and [REPORT.md](REPORT.md)
for the measured run.
