# Core Hum Contract Schema

Date: 2026-07-07

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
Core Hum candidate preview emitted by `hum core-preview`; these recognize
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
- `core_lowering`: `planned`
- `type_check`: `planned`
- `effect_check`: `planned`
- `profile_check`: `planned`
- `core_interpreter`: `planned`
- `core_verify`: `planned`

These are roadmap facts, not implementation APIs. They keep build order honest.
`partial_v0` means the compiler recognizes a conservative line-oriented subset
of `does:` body shapes for readiness reporting only; it does not lower, type
check, execute, or verify those lines. `preview_v0` means the compiler can emit
Core Hum candidate operation families, candidate-local name previews, block
previews, expression preview atoms, expression AST previews, operators, and
blockers from those lines for roadmap and adapter use, but it still does not
lower, type check, execute, or verify them.

## Honesty Rules

- `hum core-contract` is a discovery command, not Core Hum emission.
- `hum core-preview` is a candidate, candidate-local name preview, block preview,
  expression preview, expression AST preview, and blocker report, not executable
  Core Hum.
- It must not run generated code.
- It must not claim executable semantics.
- It must not pretend full type checking, effect checking, optimization, or backend
  lowering exists.
- It must stay in sync with `hum ir-contract --format json`, `hum
  ir-readiness --format json`, `hum capabilities --format json`, and `hum
  version --format json`.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- no backend dependency
- no generated code execution

## Non-Goals For V0

V0 does not lower Surface Hum to Core Hum, execute tasks, type-check bodies,
effect-check bodies, choose a backend, optimize programs, prove memory safety, or
emit artifacts. It names the boring executable core boundary future work must
satisfy before those claims are honest.
