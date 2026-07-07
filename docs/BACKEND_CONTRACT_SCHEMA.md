# Hum Backend Contract Schema

Date: 2026-07-07

Current schema: `hum.backend_contract.v0`

## Purpose

`hum backend-contract` publishes the backend ladder and adapter preservation
contract accepted in [decisions/0008-adopt-swappable-backend-ladder.md](decisions/0008-adopt-swappable-backend-ladder.md).
It names [HUM_IR_CONTRACT_SCHEMA.md](HUM_IR_CONTRACT_SCHEMA.md) as the semantic
owner contract that future backends must consume.

This command exists so humans, agents, backend experiments, CI wrappers, and
future editor tools can ask the Hum binary what backend path it currently
recognizes without scraping prose from architecture docs.

It does not mean Hum has an interpreter, Cranelift backend, LLVM backend, MLIR
lowering, Wasm output, C output, or custom backend today. It is a contract for
future adapters.

## Command

```powershell
hum backend-contract
hum backend-contract --format json
```

During the Rust bootstrap:

```powershell
cargo run -- backend-contract
cargo run -- backend-contract --format json
```

The human output is for terminals. The JSON output is for agents, CI wrappers,
backend adapters, release tooling, and documentation checks.

## Top-Level Shape

```json
{
  "schema": "hum.backend_contract.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "decision": "0008-adopt-swappable-backend-ladder",
  "semantic_owner": "hum_ir",
  "semantic_owner_schema": "hum.ir_contract.v0",
  "backend_order": [],
  "adapter_must_preserve_or_report_loss": [],
  "rules": [],
  "non_goals_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.backend_contract.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `milestone`: current implementation milestone
- `decision`: design decision record that owns this contract
- `semantic_owner`: the layer that owns Hum semantics, currently `hum_ir`
- `semantic_owner_schema`: schema that defines the semantic owner contract,
  currently `hum.ir_contract.v0`
- `backend_order`: ordered backend ladder
- `adapter_must_preserve_or_report_loss`: facts every backend adapter must keep
  or explicitly report as weakened/lost
- `rules`: short policy rules for backend work
- `non_goals_v0`: claims this command must not make

## Backend Stage Shape

Each `backend_order` entry has:

- `id`: stable backend-stage identifier
- `stage`: numeric order in the ladder
- `status`: `planned` or `deferred` in V0
- `role`: why this stage exists
- `decision`: current rule for when to use the stage

Current stages:

- `interpreter`: first executable semantics and contract behavior
- `cranelift`: first native proof and fast local feedback
- `llvm`: mature optimized native AOT builds
- `mlir`: future multi-level lowering for vector, tensor, sparse, GPU, or
  accelerator work
- `wasm_or_c`: portable or inspectable escape hatch
- `custom_hum_backend`: future Hum-specific backend or optimization stack

## Adapter Preservation Facts

Every backend adapter must preserve or explicitly report loss of:

- source spans and semantic graph node IDs
- task, test, type, and store identity
- typed failure behavior
- effect and capability facts
- ownership and aliasing assumptions
- allocation and resource facts
- profile restrictions
- unsafe and foreign boundaries
- debug and profiling provenance
- unsupported features or weakened guarantees

## Honesty Rules

- `hum backend-contract` is a discovery command, not execution.
- It must not run generated code.
- It must not select a backend.
- It must not promise performance, safety-critical readiness, or optimizer
  behavior.
- It must keep LLVM, Cranelift, MLIR, Wasm, C, and future custom backends behind
  the same Hum IR adapter boundary.
- It must keep `hum ir-contract --format json`, `hum capabilities --format
  json`, and `hum version --format json` in sync with this schema.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- no backend dependency
- no generated code execution

## Non-Goals For V0

V0 does not promise an interpreter, code generator, optimizer, executable
artifact, backend CLI flag, debug info, target triple support, or safety-critical
qualification. It is the machine-readable contract that prevents future backend
work from erasing Hum's source semantics.