# Hum IR Readiness Schema

Date: 2026-07-07

Current schema: `hum.ir_readiness.v0`

## Purpose

`hum ir-readiness` reports how far current `.hum` source has progressed toward
future Core Hum and Hum IR lowering.

This is not an IR emitter. It is a readiness and blocker inventory built from
the parser, AST, semantic graph facts, diagnostics, the Core Hum contract in
[HUM_CORE_CONTRACT_SCHEMA.md](HUM_CORE_CONTRACT_SCHEMA.md), and the Hum IR
contract in [HUM_IR_CONTRACT_SCHEMA.md](HUM_IR_CONTRACT_SCHEMA.md). It exists so humans,
agents, and CI can see which source facts are already visible and which compiler
passes still block honest IR/backend claims.

## Command

```powershell
hum ir-readiness [--format human|json] [--timings] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- ir-readiness examples/reference_surface.hum
cargo run -- ir-readiness --format json examples/reference_surface.hum
```

The human output is for terminals. The JSON output is for agents, CI wrappers,
compiler-roadmap checks, and future IR verifier work.

## Top-Level Shape

```json
{
  "schema": "hum.ir_readiness.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "core_contract_schema": "hum.core_contract.v0",
  "ir_contract_schema": "hum.ir_contract.v0",
  "summary": {},
  "pass_status": [],
  "lowering_candidates": [],
  "non_goals_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.ir_readiness.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `milestone`: current implementation milestone
- `core_contract_schema`: Core Hum contract this report is measured against
- `ir_contract_schema`: Hum IR contract this report is measured against
- `summary`: file, item, task, test, candidate, ready, blocked, error, warning,
  and body-grammar counts
- `pass_status`: current status for the pass names in `hum.ir_contract.v0`
- `lowering_candidates`: parsed source items that future lowering must handle
- `non_goals_v0`: claims this command must not make

## Candidate Shape

Each `lowering_candidates` entry has:

- `id`: stable-ish source-derived readiness row ID
- `kind`: Hum item kind, such as `type`, `store`, `task`, `test`, or `app`
- `name`: source item name
- `graph_node_id`: semantic graph node ID for the same item
- `source_span`: file, line, and column
- `status`: readiness status, currently blocked before core lowering
- `current_layer`: currently visible compiler layers
- `target_layer`: future target layer path
- `facts_available`: source facts already visible to tools
- `missing_passes`: pass boundaries still missing before honest IR
- `blocking_reasons`: reason strings explaining the blocked state
- `source_sections`: sections seen on the item
- `body_grammar`: optional partial V0 parse/classification of meaningful `does:`
  lines, when the item has a `does:` section

Current candidate statuses:

- `blocked_before_core_lowering`: source parsed but no Core Hum lowering exists
- `blocked_by_source_errors`: source diagnostics include errors

## Facts Available

V0 may report facts such as:

- `source_span`
- `semantic_graph_node_id`
- `item_kind`
- `item_name`
- `source_sections`
- `section_line_spans`
- `signature_params`
- `signature_result`
- `field_shapes`
- `store_type_annotation`
- `nested_item_scope`
- `effect_hints`
- `contract_hints`
- `resource_hints`
- `body_text_captured`
- `body_grammar_partial_v0`
- `test_modifiers`
- `test_coverage_hints`

These facts are not type checking, effect checking, ownership checking, or IR
verification. They are the source-visible material those future passes must use.

## Body Grammar Shape

When present, `body_grammar` has:

- `status`: aggregate body status, such as `partial_v0_all_lines_recognized` or
  `partial_v0_with_unsupported_lines`
- `grammar_status`: current body grammar maturity, currently `partial_v0`
- `total_lines`: total lines captured under `does:`
- `meaningful_lines`: non-empty, non-comment lines considered by the classifier
- `recognized_lines`: lines recognized by the partial V0 body grammar
- `unsupported_lines`: meaningful lines not in the partial V0 body grammar
- `statements`: one row per meaningful body line

Each `statements` row has source span, original line text, `kind`, `status`,
optional `expression_kind`, and optional `reason`.

Recognized V0 kinds include:

- `return`
- `fail`
- `let_binding`
- `mutable_binding`
- `set_place`
- `if_header`
- `while_header`
- `for_each_header`
- `for_index_header`
- `loop_header`
- `block_close`
- `record_field_initializer`
- `nested_intent_header`
- `test_expectation`

Unsupported but intentionally named V0 blockers include:

- `save_in_store`: recognized as surface syntax, but blocked by
  `surface_save_requires_store_lowering`
- `unknown_body_line`: not in the partial body grammar

This is grammar visibility only. It is not Core Hum lowering, type checking,
effect checking, test execution, or interpretation.

## Pass Status

V0 reports these pass statuses:

- `parse`: `current`
- `semantic_graph_build`: `current`
- `body_grammar`: `partial_v0`
- `core_lowering`: `not_implemented`
- `type_check`: `not_implemented`
- `effect_check`: `not_implemented`
- `ownership_alias_check`: `not_implemented`
- `allocation_resource_check`: `not_implemented`
- `contract_evidence_linking`: `report_available_not_ir_pass`
- `profile_check`: `not_implemented`
- `ir_verify`: `not_implemented`

## Honesty Rules

- `hum ir-readiness` must not emit Hum IR.
- It must not execute generated code.
- It must not claim type safety, memory safety, optimization, backend readiness,
  or executable semantics.
- It may report source-visible facts, partial body grammar facts, and missing
  compiler passes.
- It must stay in sync with `hum.core_contract.v0`, `hum.ir_contract.v0`, `hum
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

V0 does not produce Core Hum, Hum IR, bytecode, machine code, backend adapter
input, proof artifacts, optimized code, or executable behavior. It is a progress
map from current parsed source toward the first honest lowering milestone.