# Recorded feasibility result

Date: 2026-07-21

Host: `x86_64-pc-windows-msvc`

Baseline: `6f1708c53fa6205cffe6ac88d2fe0dde57215333`

## Decision

| Question | Result |
|---|---|
| Cranelift JIT execution | GO |
| Cranelift native object emission | GO |
| Windows native linking | GO |
| Standalone native executable | GO |
| Checked-add ABI and overflow status | GO |
| Backend-kernel/interpreter agreement | GO for tested addition cases |
| Real Hum artifact to Cranelift lowering | **NO-GO / not yet provable** |

This does not prove that a Hum program has been compiled to native code. It
proves that the selected Cranelift stack and Windows linker can support the
first desired backend shape, and identifies the exact production-artifact gap
that prevents honest Hum lowering today.

## Production evidence consumed

The real `examples/core/minimal_add.hum` source passes `hum check` and runs in
the production interpreter. Its compiler reports currently say:

- Core schema: `hum.core_lower.v0`
- operation: `return`
- expression kind/root/operator: binary/add
- node count: 3
- Core expression type status: `not_type_checked_v0`
- Core `execution_ready`: 0
- Core `ir_ready`: 0
- IR-readiness `ready_for_ir`: 0
- missing passes: `allocation_resource_check`, `profile_check`, `ir_verify`

The Core report carries expression text, a root-form classification, operator,
and node count. The resolve report carries two parameter references, but those
references share the statement span and are not attached as ordered children
of the Core binary expression. An honest lowering therefore still lacks:

1. ordered expression-child identities;
2. operand-to-resolver-definition bindings;
3. a backend-consumable checked expression type on the Core node;
4. an implemented IR verification pass; and
5. an IR-ready production artifact.

Inferring operands from `"a + b"`, parameter order, or a handwritten expected
answer would be a fake lowering. The adapter instead returns
`production_artifact_not_backend_lowerable_v0`.

## Recorded proof

Pinned direct versions:

- `cranelift-codegen = 0.133.1`
- `cranelift-frontend = 0.133.1`
- `cranelift-jit = 0.133.1`
- `cranelift-module = 0.133.1`
- `cranelift-native = 0.133.1`
- `cranelift-object = 0.133.1`
- `serde_json = 1.0.145`

Toolchain:

- `rustc 1.96.0 (ac68faa20 2026-05-25)`
- `cargo 1.96.0 (30a34c682 2026-05-25)`
- LLVM 22.1.2 in rustc
- target `x86_64-pc-windows-msvc`
- MSVC tool package directory `14.50.35717`
- selected `link.exe` file version `14.50.35728.0` (PE header linker version 14.50)

Exact proof command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo run --quiet -- proof
```

The final pre-review proof run produced:

- JIT compile/finalize: 1,717 microseconds;
- object emission: 1,116 microseconds;
- object size: 198 bytes;
- native driver compile/link: 172 milliseconds;
- complete proof: 1,576 milliseconds.

Tested ordinary cases agreed exactly with the production Hum interpreter:

| Inputs | Result |
|---|---:|
| `2, 3` | 5 |
| `-7, 11` | 4 |
| `0, 0` | 0 |
| `1000000, 24` | 1000024 |

Both `MAX_INT + 1` and `MIN_INT + -1` return generated status `1`. The
standalone driver maps `MAX_INT + 1` to exit code 2 and the exact interpreter
diagnostic `runtime trap: integer overflow while evaluating \`a + b\``. That
mapping proves the ABI can carry checked overflow; it does not establish a
production Hum trap/runtime ABI.

Disposable output hashes from that run:

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `target/proof/hum_spike_checked_add.clif` | 300 | `1730af0feb8a37675a193b7717d0b57ec59af7b5584dbbf063f3872ed7568d59` |
| `target/proof/hum_spike_checked_add.obj` | 198 | `e0474d9098440cb5fe8e21e5025905192394fffd9167455679eef67b0845ff73` |
| `target/proof/hum_spike_native.exe` | 136704 | `cc934e09c32d22e8b87f96e0e9988f8da4c802437e5dcac9c2d8919a5500c4d6` |

The CLIF and COFF object are deterministic for the pinned build and host. The
executable hash can vary with linker metadata and is recorded as run evidence,
not as a cross-machine reproducibility contract.

## Unsupported operation proof

`cases/unsupported_subtract.hum` is accepted by `hum check` and yields a real
Core expression with operator `sub`. The experiment rejects it with:

```text
unsupported Core operator `sub`; no interpreter fallback is permitted
```

No code is generated for that artifact.

## Roadmap effect

Keep Cranelift as the first native backend candidate in the swappable-backend
ladder. The JIT, object, Windows link, standalone executable, exact integer ABI,
and checked-overflow carrier are all feasible on the current host.

Do not begin production Cranelift integration yet. The next production work
should continue the already authorized canonical-tree/consumer convergence and
then make Hum IR expose verified, ordered, typed operands. A later backend
increment should consume that IR directly and repeat this differential proof.
No LLVM or Wasm detour is justified by this spike, and no Cranelift promotion is
justified until the Hum-side no-go becomes a real IR-ready artifact.
