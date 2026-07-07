# Hum Core Preview Schema

Date: 2026-07-07

Current schema: `hum.core_preview.v0`

## Purpose

`hum core-preview` emits the first Core Hum-shaped view of parsed `does:` body
lines. It sits between the partial body grammar in `hum ir-readiness` and future
true Core Hum lowering.

This command is intentionally not an interpreter, not a type checker, not an
effect checker, not Hum IR, and not a backend. It reports conservative candidate
operations and blockers so humans, agents, and future compiler passes can see
what the current bootstrap can map toward Core Hum without pretending the body
has executable meaning.

## Command

```powershell
hum core-preview [--format human|json] [--timings] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- core-preview examples/reference_surface.hum
cargo run -- core-preview --format json examples/reference_surface.hum
```

The human output is for terminals. The JSON output is for agents, CI wrappers,
compiler-roadmap checks, and future Core Hum lowering/verifier work.

## Top-Level Shape

```json
{
  "schema": "hum.core_preview.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "core_contract_schema": "hum.core_contract.v0",
  "summary": {},
  "core_candidates": [],
  "non_goals_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.core_preview.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `milestone`: current implementation milestone
- `core_contract_schema`: Core Hum contract this report targets
- `summary`: file, item, task, test, candidate, execution-ready, diagnostic,
  and statement-preview counts
- `core_candidates`: task or test bodies with a `does:` section
- `non_goals_v0`: claims this command must not make

## Candidate Shape

Each `core_candidates` entry has:

- `id`: stable-ish source-derived preview row ID
- `kind`: Hum item kind, currently usually `task` or `test`
- `name`: source item name
- `source_span`: file, line, and column for the source item
- `status`: aggregate preview status
- `core_contract_schema`: owning Core Hum contract schema
- `body_status`: partial body grammar aggregate status
- `grammar_status`: body grammar maturity, currently `partial_v0`
- `summary`: meaningful body lines and statement status counts
- `source_sections`: sections seen on the source item
- `statements`: one row per meaningful `does:` body line

Current candidate statuses:

- `lowerable_preview_v0`: all meaningful body lines map to Core Hum candidate
  operations, but are not typed or executable
- `contextual_preview_v0`: the body contains lines that need surrounding context,
  such as record fields or test expectations
- `preview_with_blockers`: at least one meaningful line is blocked before Core
  lowering
- `empty_body`: the `does:` section has no meaningful lines
- `blocked_by_source_errors`: source diagnostics include errors

## Statement Shape

Each `statements` row has:

- `source_span`: source file, line, and column
- `text`: original meaningful body line text
- `source_kind`: partial body grammar kind from `hum.ir_readiness.v0`
- `source_status`: source grammar status, such as `recognized_v0` or
  `unsupported_v0`
- `core_operation`: candidate Core Hum operation family
- `status`: statement preview status
- `expression_kind`: coarse expression shape when available
- `reason`: optional blocker or context reason

Statement statuses:

- `lowerable_preview_v0`: can be represented as a Core Hum operation candidate
- `contextual_preview_v0`: recognized, but not a standalone Core Hum operation
- `blocked_v0`: blocked before Core Hum lowering

Examples of Core Hum candidate operations:

- `return`
- `fail`
- `let_binding`
- `mutable_binding`
- `set_place`
- `if_statement`
- `while_loop`
- `for_each`
- `for_index`
- `loop`
- `block_close`
- `record_construction_field`
- `record_construction_close`

Named V0 blockers include:

- `store_write_deferred` with `surface_save_requires_store_lowering`
- `unknown` with `not_in_core_preview_v0`

## Honesty Rules

- `hum core-preview` must not execute code.
- It must not claim type checking, effect checking, ownership checking,
  optimization, or backend readiness.
- It must not emit Hum IR.
- It may report Core Hum candidate operation families, source spans, coarse
  expression kinds, and explicit blockers.
- It must stay in sync with `hum.core_contract.v0`, `hum.ir_readiness.v0`, `hum
  capabilities --format json`, and `hum version --format json`.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- no backend dependency
- no generated code execution

## Non-Goals For V0

V0 does not produce executable Core Hum, Hum IR, bytecode, machine code, backend
adapter input, proof artifacts, optimized code, or executable behavior. It is a
conservative preview of what the next true lowering pass must make precise.