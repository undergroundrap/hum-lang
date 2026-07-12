# Core Hum Contract Schema

Date: 2026-07-08

Current schema: `hum.core_contract.v0`

## Purpose

`hum core-contract` publishes the contract for Core Hum, the small executable
language that Surface Hum must lower into before Hum IR, interpreters, or native
backends can make honest claims.

This is the bridge between the current parser/semantic graph and the future Hum
IR contract. It names the value, type, place, expression, statement, effect,
contract-lowering, and acceptance-gate families the first executable subset must
satisfy.

This command is not a source-to-core lowering implementation, not an interpreter,
not a type checker, and not an optimizer. V0 now includes a partial
body-grammar classifier used by `hum ir-readiness` and a non-executing
Core Hum candidate preview emitted by `hum core-preview`; `hum core-lower` now
emits the first unverified source-mapped Core Hum artifact boundary from those
facts; `hum core-verify` now verifies the non-executing artifact invariants without execution or IR emission; `hum full-type-check` now checks recognized Core/body statement types or reports explicit blockers without execution or IR emission; `hum effect-check` now checks recognized Core/body effects or reports explicit blockers without execution or IR emission; `hum ownership-check` now checks recognized local ownership facts or reports explicit blockers without execution or IR emission; `hum resource-check` now checks declared allocation/resource intent or reports explicit blockers without execution or IR emission; `hum profile-check` now checks runtime profile policy declarations or reports explicit blockers without execution or IR emission. These recognize
first statement candidates, candidate-local name previews, block previews,
expression preview atoms, expression AST previews, operators, and blockers
without assigning executable meaning.

## Command

```powershell
hum core-contract
hum core-contract --format json
```

During the Rust bootstrap:

```powershell
cargo run -- core-contract
cargo run -- core-contract --format json
```

The human output is for terminals. The JSON output is for agents, CI wrappers,
compiler-roadmap checks, and future Core Hum lowering/verifier work.

## Top-Level Shape

```json
{
  "schema": "hum.core_contract.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "lowers_from_schema": "hum.semantic_graph.v0",
  "lowers_to_schema": "hum.ir_contract.v0",
  "core_catalogs": [],
  "contract_lowering": [],
  "acceptance_gates": [],
  "rules": [],
  "non_goals_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.core_contract.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `milestone`: current implementation milestone
- `lowers_from_schema`: current source-fact surface Core Hum consumes
- `lowers_to_schema`: Hum IR contract that consumes checked Core Hum
- `core_catalogs`: grouped Core Hum families and starter items
- `contract_lowering`: how checked sections become core obligations or
  permissions
- `acceptance_gates`: pass boundaries that must exist before executable claims
- `rules`: short policy rules for Core Hum work
- `non_goals_v0`: claims this command must not make

## Core Catalog Shape

Each `core_catalogs` entry has:

- `name`: stable catalog name such as `values`, `types`, `places`,
  `expressions`, `statements`, or `effects`
- `status`: current maturity, `design` in V0
- `role`: why the family exists
- `items`: starter item identifiers inside the family

V0 names the starter families without claiming the compiler can lower source
bodies into them yet.

The closed `effects` catalog includes Session Z's additive `output` value.
That catalog entry does not itself grant authority or claim a complete effect
system; the executable operation still requires checked source closure, an
exact one-run operator grant, and the bounded output adapter.

## Contract Lowering Shape

Each `contract_lowering` entry has:

- `section`: Surface Hum section name
- `lowers_to`: Core Hum meaning for that section
- `blame`: who owns the obligation or assumption

Examples:

- `needs` lowers to a precondition obligation and debug entry check
- `ensures` lowers to a postcondition obligation and debug exit check
- `changes` lowers to mutation permission
- `fails when` lowers to an allowed typed failure variant
- `trusts` lowers to an explicit unchecked assumption

## Acceptance Gates

V0 reports these gate statuses:

- `parse`: `current`
- `semantic_graph_build`: `current`
- `body_grammar`: `partial_v0`
- `core_preview`: `preview_v0`
- `core_lowering`: `unverified_core_artifact_v0`
- `type_check`: `declaration_and_trivial_return_check_available`
- `full_type_check`: `recognized_core_body_type_gate_available_v0`
- `effect_check`: `recognized_core_effect_gate_available_v0`
- `ownership_check`: `recognized_core_ownership_gate_available_v0`
- `allocation_resource_check`: `recognized_core_resource_gate_available_v0`
- `profile_check`: `recognized_core_profile_gate_available_v0`
- `core_interpreter`: `planned`
- `core_verify`: `verified_non_executing_core_artifact_v0`

These are roadmap facts, not implementation APIs. They keep build order honest.
`partial_v0` means the compiler recognizes a conservative line-oriented subset
of `does:` body shapes for readiness reporting only; it does not lower, type
check, execute, or verify those lines. `preview_v0` means the compiler can emit
Core Hum candidate operation families, candidate-local name previews, block
previews, expression preview atoms, expression AST previews, operators, and
blockers from those lines for roadmap and adapter use, but it still does not
lower, type check, execute, or verify them. `unverified_core_artifact_v0` means
the compiler can serialize source-mapped Core Hum operation rows and blockers,
but still cannot execute, emit Hum IR, or lower to any backend. `verified_non_executing_core_artifact_v0` means `hum core-verify` checks source spans, operation/status/blocker consistency, and non-claim honesty for those rows, but still cannot execute, prove safety, optimize, or emit IR. `recognized_core_body_type_gate_available_v0` means `hum full-type-check` checks recognized V0 statement type contexts and blocks unknown or unsupported contexts, but still cannot claim complete language type safety. `recognized_core_effect_gate_available_v0` means `hum effect-check` checks recognized V0 effect contexts and blocks missing or unchecked effect declarations, but still cannot claim complete effect safety. `recognized_core_ownership_gate_available_v0` means `hum ownership-check` checks recognized V0 local ownership facts, including the exact Session V straight-line direct-field writable-alias slice, and blocks duplicate or contradictory local ownership contexts, but still cannot claim complete ownership, borrowing, general alias, or memory safety. `recognized_core_resource_gate_available_v0` means `hum resource-check` checks declared allocation/resource intent and explicit blockers, but still cannot prove allocation freedom, complete resource behavior, profile compliance, optimization, or memory safety. `recognized_core_profile_gate_available_v0` means `hum profile-check` recognizes runtime profile policy declarations and blocks unknown or strict profile claims until enforcement and evidence checks exist, but still cannot enforce profiles, certify safety, narrow the stdlib, select targets, or prove memory safety.

## Honesty Rules

- `hum core-contract` is a discovery command, not Core Hum emission.
- `hum core-preview` is a candidate, candidate-local name preview, block preview,
  expression preview, expression AST preview, and blocker report, not executable
  Core Hum.
- `hum core-lower` emits an unverified source-mapped Core Hum artifact and
  blockers, not executable Core Hum, Hum IR, or backend input.
- `hum core-verify` verifies non-executing artifact invariants, not program behavior, proof, memory safety, optimization, Hum IR, or backend input.
- `hum full-type-check` checks recognized V0 body/Core statement type contexts, not complete language type safety, effects, ownership, memory safety, optimization, Hum IR, or backend input.
- `hum effect-check` checks recognized V0 body/Core effect contexts, not complete effect safety, ownership, memory safety, optimization, Hum IR, or backend input.
- `hum ownership-check` checks recognized V0 local ownership facts, not complete ownership safety, borrowing, alias safety, memory safety, optimization, Hum IR, or backend input.
- `hum resource-check` checks declared allocation/resource intent, not allocation-freedom proof, complete resource analysis, complete cost analysis, memory safety, profile enforcement, optimization, Hum IR, or backend input.
- `hum profile-check` checks runtime profile policy declarations, not profile enforcement, stdlib narrowing, target selection, certification, memory safety, Hum IR, or backend input.
- It must not run generated code.
- It must not claim executable semantics.
- It must not pretend complete body/Core type checking, complete effect checking, optimization, or backend
  lowering exists.
- It must stay in sync with `hum core-lower --format json`,
  `hum core-verify --format json`, `hum effect-check --format json`, `hum ownership-check --format json`, `hum resource-check --format json`, `hum profile-check --format json`, `hum ir-contract --format json`, `hum ir-readiness --format json`,
  `hum capabilities --format json`, and `hum version --format json`.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- no backend dependency
- no generated code execution

## Non-Goals For V0

V0 does not lower Surface Hum to executable Core Hum, execute tasks, fully type-check bodies,
fully check body effects, fully check ownership and borrowing, fully prove resource or allocation behavior, choose a backend, optimize programs, prove memory safety, or
emit artifacts. It names the boring executable core boundary future work must
satisfy before those claims are honest.

## Session AL Callable Boundary

The existing catalogs include one `callable_definition_handle_al` value, the
exact `task(UInt) -> UInt` type, and bounded `callable_value` and
`callable_application_al` expressions. These names authorize no closure
environment, capture, storage, return, open effect row, or general dispatch.
