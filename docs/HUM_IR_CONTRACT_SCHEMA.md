# Hum IR Contract Schema

Date: 2026-07-08

Current schema: `hum.ir_contract.v0`

## Purpose

`hum ir-contract` publishes the contract for the compiler-owned Hum IR layer.
This is the semantic owner named by
[BACKEND_CONTRACT_SCHEMA.md](BACKEND_CONTRACT_SCHEMA.md): backends consume
verified Hum IR instead of raw surface Hum. Core Hum is named by
[HUM_CORE_CONTRACT_SCHEMA.md](HUM_CORE_CONTRACT_SCHEMA.md), and source progress
toward these contracts is reported by
[HUM_IR_READINESS_SCHEMA.md](HUM_IR_READINESS_SCHEMA.md).

The exact minimal-add subset now reaches canonical target-independent
[`hum.backend_input.v0`](HUM_BACKEND_INPUT_SCHEMA.md) bytes. The strict verifier
can authenticate them, cross-bind them to live typed facts, and lend a sealed
capability; bytes and reports alone grant no authority. This command remains a
discovery contract. The separate explicit `backend-probe` may borrow that
capability and lower only the canonical checked add to verified Cranelift IR.

An additive `hum.backend_input.v1` path now authenticates the canonical
`integer_sign` source and lends a separate callback-scoped capability. Typed
getters alone lower its signed comparisons, branch order, source literals, and
one result-tag store. This is not general task or CFG lowering.

## Command

```powershell
hum ir-contract
hum ir-contract --format json
```

During the Rust bootstrap:

```powershell
cargo run -- ir-contract
cargo run -- ir-contract --format json
```

The human output is for terminals. The JSON output is for agents, CI wrappers,
backend adapters, release tooling, and future IR verifier work.

## Top-Level Shape

```json
{
  "schema": "hum.ir_contract.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "semantic_owner": "hum_ir",
  "core_contract_schema": "hum.core_contract.v0",
  "backend_contract_schema": "hum.backend_contract.v0",
  "ir_layers": [],
  "required_carried_facts": [],
  "required_passes": [],
  "node_families_v0": [],
  "rules": [],
  "non_goals_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.ir_contract.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `milestone`: current implementation milestone
- `semantic_owner`: stable owner name, currently `hum_ir`
- `core_contract_schema`: Core Hum contract this IR contract consumes
- `backend_contract_schema`: backend contract that consumes this semantic owner
- `ir_layers`: ordered lowering layers from surface Hum toward backend adapter
  input
- `required_carried_facts`: facts that must survive lowering or be explicitly
  reported as unsupported/weakened
- `required_passes`: pass names Hum must implement before serious backend claims
- `node_families_v0`: first planned node families for printable/JSON Hum IR
- `rules`: short policy rules for IR and lowering work
- `non_goals_v0`: claims this command must not make

## IR Layer Shape

Each `ir_layers` entry has:

- `id`: stable layer identifier
- `stage`: numeric order in the lowering path
- `status`: current narrow support, `design`, or `planned`
- `role`: why the layer exists

Current layers:

- `surface_hum`: human-readable source captured by the parser and AST
- `semantic_graph`: source facts for tools, agents, diagnostics, and evidence
  links
- `core_hum`: small executable core for typed values, places, effects, and
  failure
- `hum_ir`: verified target-independent facts for the exact minimal-add subset,
  represented by a callback-scoped compiler-owned capability
- `backend_adapter_input`: callback-scoped verified facts lowered only for the
  current canonical checked-add probe

## Required Carried Facts

Hum IR must carry, or explicitly report loss of:

- source spans
- semantic graph node IDs
- module, file, and item identity
- task, test, type, and store identity
- typed values and places
- mutation and effect facts
- typed failure edges
- contract preconditions, postconditions, and invariants
- evidence and math obligation links
- ownership and aliasing assumptions
- allocation and resource facts
- profile guards
- unsafe and foreign boundaries
- debug and profiling provenance
- unsupported or weakened facts

## Required Passes

The V0 contract names these pass boundaries:

- `parse`
- `semantic_graph_build`
- `core_lowering`
- `core_verify`
- `type_check`
- `full_type_check`
- `effect_check`
- `ownership_alias_check`
- `allocation_resource_check`
- `contract_evidence_linking`
- `profile_check`
- `ir_verify`

These names are not final implementation APIs. They are a shared map for build
order, docs, agents, and future compiler diagnostics. In V0, `type_check` names
the narrow `hum.type_check.v0` declaration and trivial-return checker;
`full_type_check` names the implemented narrow `hum.full_type_check.v0` gate for recognized Core/body statement types. `effect_check` names the implemented narrow `hum.effect_check.v0` gate for recognized Core/body effect contexts. `ownership_alias_check` is backed by the narrow `hum.ownership_check.v0` local ownership fact gate. `allocation_resource_check` is backed by the narrow `hum.resource_check.v0` declared allocation/resource intent gate. `profile_check` is backed by the narrow `hum.profile_check.v0` runtime profile policy gate. All five and strict `ir_verify` must pass before `ir_ready=1`; the current adapter then derives ordered `I64` parameters, `sadd_overflow`, overflow `brif`, and `SourceLoc` from the lent facts.

## Honesty Rules

- `hum ir-contract` is a discovery command; `backend-input` emits canonical
  bytes and `ir-verify` reports strict acceptance without exposing authority.
- Only `backend-probe` may run the frozen generated minimal-add matrix.
- It must not claim general backend executable semantics.
- It must not pretend broad type, ownership, resource, optimization, or backend
  coverage exists beyond the exact checked gates and probe.
- It must stay in sync with `hum core-contract --format json`, `hum
  backend-contract --format json`, `hum capabilities --format json`, and `hum
  version --format json`.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- exact Cranelift 0.133.1 dependencies for the separate narrow probe only
- no generated execution through this discovery command

## Non-Goals For V0

V0 verifies canonical backend-input bytes only for the exact supported
minimal-add source and lends authority only inside a compiler callback. The
explicit probe lowers that one checked add to Cranelift. V0 does not execute
arbitrary tasks, optimize programs, prove memory safety, or support general
Cranelift, LLVM, MLIR, Wasm, C, or custom-backend lowering.

The verified-input-ready layer now carries minimal-add, integer-sign v0, and
constant-Text-output v0 only. The constant feature preserves its source
literal, output operation, authority closure, semantic tag, and live source
revision through the backend boundary. This is not general backend lowering,
durable external authority, arbitrary Text computation, or optimization.
