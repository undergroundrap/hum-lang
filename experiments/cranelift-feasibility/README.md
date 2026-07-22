# Cranelift feasibility spike

This isolated experiment answers two different questions and does not blur them:

1. Can the pinned Cranelift toolchain JIT, emit an object, link, and execute a
   checked signed-add kernel on `x86_64-pc-windows-msvc`? **Yes.**
2. Can the current production Hum artifacts honestly drive that lowering?
   **No.** The current Core report is non-executing, reports `ir_ready=0`, and
   does not expose ordered operand identities or operand-to-resolver bindings.

The second answer is the governing result. The generated kernel is explicitly a
backend capability probe, not a Hum-compiled answer. The experiment compares it
with the real Hum interpreter to test ABI and arithmetic semantics, while the
Hum-facing adapter fails closed instead of reparsing `expression.text` or
guessing operands.

## Reproduce

From the repository root, first build the production `hum` executable without
changing production sources:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo build --bin hum
Set-Location experiments\cranelift-feasibility
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run --quiet -- proof
Set-Location ..\..
git -c safe.directory=C:/Users/ocean/dev/projects/hum-lang diff --check
```

The proof emits disposable artifacts beneath `target/proof/`:

- `hum_spike_checked_add.clif`
- `hum_spike_checked_add.obj`
- `hum_spike_native.exe`

`Cargo.toml` pins every direct dependency exactly; `Cargo.lock` pins the full
transitive graph. No system-wide installation or machine configuration is
required.

## Evidence boundaries

The experiment consumes real output from these production commands for
`examples/core/minimal_add.hum`:

- `hum check`
- `hum resolve --format=json`
- `hum core-lower --format=json`
- `hum core-verify --format=json`
- `hum full-type-check --format=json`
- `hum ir-readiness --format=json`
- `hum run ... --entry add --args ...`

It does not parse Hum source, reparse the Core expression string, derive
operands from parameter order, or silently fall back to the interpreter.
`cases/unsupported_subtract.hum` proves that a real unsupported Core operator is
rejected rather than translated as addition or interpreted behind the caller's
back.

The generated ABI is:

```text
extern "C" fn(i64, i64, *mut i64) -> i32
```

Status `0` writes a result. Status `1` means signed overflow and does not claim
a result. The standalone driver maps that status to the interpreter's current
exit code and diagnostic for this capability comparison; it is not a Hum
runtime design.

## Selected Cranelift release

The experiment pins Cranelift `0.133.1`, the documented June 2026 release used
for this bounded spike. Relevant official API documentation:

- <https://docs.rs/cranelift-codegen/0.133.1/cranelift_codegen/>
- <https://docs.rs/cranelift-jit/0.133.1/cranelift_jit/struct.JITModule.html>
- <https://docs.rs/cranelift-object/0.133.1/cranelift_object/struct.ObjectModule.html>
- <https://docs.rs/cranelift-module/0.133.1/cranelift_module/trait.Module.html>
- <https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/ir.md>

See [REPORT.md](REPORT.md) for the recorded run and roadmap recommendation.
