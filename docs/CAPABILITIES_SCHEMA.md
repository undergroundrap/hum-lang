# Hum Toolchain Capabilities Schema

Date: 2026-07-07

Current schema: `hum.capabilities.v0`

## Purpose

`hum capabilities` lets tools ask a Hum binary which machine-readable surfaces it
exposes. Editors, agents, CI wrappers, and release scripts can use it before they
assume a schema, command, or adapter capability exists.

This is a toolchain discovery surface. It is not Hum's runtime authority model,
not a package permission file, and not the same thing as source-level `uses:` or
`changes:` capabilities.

## Command

```powershell
hum capabilities
hum capabilities --format json
```

During the Rust bootstrap:

```powershell
cargo run -- capabilities
cargo run -- capabilities --format json
```

The human output is for terminals. The JSON output is the adapter and agent
contract.

## Top-Level Shape

```json
{
  "schema": "hum.capabilities.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "schemas": {},
  "commands": [],
  "editor_capabilities": []
}
```

## Fields

- `schema`: schema name, currently `hum.capabilities.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `milestone`: current implementation milestone
- `schemas`: schema names this binary can emit
- `commands`: machine-readable command surfaces and their schemas
- `editor_capabilities`: editor-facing features this binary can support or names
  as planned/deferred

## Schemas

Current `schemas` includes:

- `semantic_graph`: `hum.semantic_graph.v0`
- `syntax_surface`: `hum.syntax_surface.v0`
- `check_diagnostics`: `hum.check.v0`
- `evidence_report`: `hum.evidence.v0`
- `math_obligations_report`: `hum.math_obligations.v0`
- `math_obligation`: `hum.math_obligation.v0`
- `resource_report`: `hum.resource_report.v0`
- `core_preview`: `hum.core_preview.v0`
- `core_lower`: `hum.core_lower.v0`
- `core_verify`: `hum.core_verify.v0`
- `resolve_report`: `hum.resolve.v0`
- `type_env`: `hum.type_env.v0`
- `type_check`: `hum.type_check.v0`
- `full_type_check`: `hum.full_type_check.v0`
- `effect_check`: `hum.effect_check.v0`
- `ownership_check`: `hum.ownership_check.v0`
- `ir_readiness`: `hum.ir_readiness.v0`
- `core_contract`: `hum.core_contract.v0`
- `ir_contract`: `hum.ir_contract.v0`
- `backend_contract`: `hum.backend_contract.v0`
- `diagnostic_explain`: `hum.diagnostic_explain.v0`
- `diagnostic_catalog`: `hum.diagnostic_catalog.v0`
- `capabilities`: `hum.capabilities.v0`
- `lsp_capabilities`: `hum.lsp_capabilities.v0`
- `doctor`: `hum.doctor.v0`
- `target_facts`: `hum.target_facts.v0`
- `target_fact_record`: `hum.target_fact_record.v0`

## Commands

Each `commands` entry has:

- `name`: stable command-surface identifier
- `command`: human-readable invocation shape
- `schema`: emitted schema or owning schema
- `status`: `current`, `adapter-ready`, `planned`, or `deferred`
- `purpose`: short explanation of why tools would call it

Current entries include:

- `hum check --format json`
- `hum graph`
- `hum evidence --format json`
- `hum math-obligations --format json`
- `hum resource-report --format json`
- `hum core-preview --format json`
- `hum core-lower --format json`
- `hum core-verify --format json`
- `hum resolve --format json`
- `hum type-env --format json`
- `hum type-check --format json`
- `hum full-type-check --format json`
- `hum effect-check --format json`
- `hum ownership-check --format json`
- `hum ir-readiness --format json`
- `hum core-contract --format json`
- `hum ir-contract --format json`
- `hum backend-contract --format json`
- `hum syntax`
- `hum syntax --format textmate`
- `hum explain --format json`
- `hum diagnostics --format json`
- `hum capabilities --format json`
- `hum lsp --capabilities --format json`
- `hum doctor --format json`
- `hum target-facts --format json`

## Editor Capabilities

Each `editor_capabilities` entry has:

- `name`: stable capability identifier
- `status`: `current`, `adapter-ready`, `planned`, or `deferred`
- `source`: first-party Hum command, fixture, or future surface that owns the
  fact
- `schema`: schema associated with the source, or `none` when the capability is
  not schema-backed yet

Adapters should treat `adapter-ready` as meaning the Hum binary emits enough
facts for a thin adapter to map them to editor protocols. It does not mean a
full first-party `hum lsp` server exists yet.

## Adapter Rules

- Query `hum capabilities --format json` before enabling optional editor
  features.
- Do not treat undocumented commands as stable adapter surfaces.
- Prefer schema names over version guesses when checking compatibility.
- Ignore unknown fields and unknown capabilities.
- Treat `planned` and `deferred` as visible roadmap facts, not promises that the
  current binary can serve requests.
- Keep this separate from runtime/package authority. Runtime capabilities belong
  to profiles, effects, and source facts, not this discovery schema.

## Non-Goals For V0

V0 does not promise:

- LSP protocol support by itself
- package manager compatibility metadata
- runtime security permission grants
- final schema stability
- a complete feature matrix for all future tools

The surface exists so adapters and agents can stop guessing what the current Hum
binary knows how to emit.