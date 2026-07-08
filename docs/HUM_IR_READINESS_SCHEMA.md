# Hum IR Readiness Schema

Date: 2026-07-08

Current schema: `hum.ir_readiness.v0`

## Purpose

`hum ir-readiness` reports how far current `.hum` source has progressed toward
future full type/effect checking and Hum IR lowering after Core verification.

This is not an IR emitter. It is a readiness and blocker inventory built from
the parser, AST, semantic graph facts, diagnostics, the checked resolver report
in [HUM_RESOLVE_SCHEMA.md](HUM_RESOLVE_SCHEMA.md), the declaration annotation
type-check report in [HUM_TYPE_CHECK_SCHEMA.md](HUM_TYPE_CHECK_SCHEMA.md), the
Core Hum preview report in [HUM_CORE_PREVIEW_SCHEMA.md](HUM_CORE_PREVIEW_SCHEMA.md),
the Core Hum contract in [HUM_CORE_CONTRACT_SCHEMA.md](HUM_CORE_CONTRACT_SCHEMA.md),
the unverified Core Hum artifact summary in [HUM_CORE_LOWER_SCHEMA.md](HUM_CORE_LOWER_SCHEMA.md),
the non-executing Core Hum verifier summary in [HUM_CORE_VERIFY_SCHEMA.md](HUM_CORE_VERIFY_SCHEMA.md),
the recognized Core/body type gate in [HUM_FULL_TYPE_CHECK_SCHEMA.md](HUM_FULL_TYPE_CHECK_SCHEMA.md), the recognized Core/body effect gate in [HUM_EFFECT_CHECK_SCHEMA.md](HUM_EFFECT_CHECK_SCHEMA.md), the recognized local ownership fact gate in [HUM_OWNERSHIP_CHECK_SCHEMA.md](HUM_OWNERSHIP_CHECK_SCHEMA.md),
and the Hum IR contract in [HUM_IR_CONTRACT_SCHEMA.md](HUM_IR_CONTRACT_SCHEMA.md).
It exists so humans, agents, and CI can see which source facts are already
visible and which compiler passes still block honest IR/backend claims.

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
  "resolver": {},
  "type_check": {},
  "core_preview": {},
  "core_lower": {},
  "core_verify": {},
  "full_type_check": {},
  "effect_check": {},
  "ownership_check": {},
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
- `resolver`: checked `hum.resolve.v0` summary consumed as an IR-readiness gate
- `type_check`: checked `hum.type_check.v0` summary consumed as an IR-readiness gate
- `core_preview`: conservative `hum.core_preview.v0` summary consumed as a
  source-to-core planning signal, not as a lowering authority
- `core_lower`: `hum.core_lower.v0` summary consumed as the first unverified
  source-mapped Core Hum artifact boundary, not as verification or execution
- `core_verify`: `hum.core_verify.v0` summary consumed as the non-executing Core artifact verifier gate, not as execution, proof, safety, optimization, or IR emission
- `full_type_check`: `hum.full_type_check.v0` summary consumed as the recognized body/Core statement type gate, not as complete type safety
- `effect_check`: `hum.effect_check.v0` summary consumed as the recognized body/Core effect gate, not as complete effect safety
- `ownership_check`: `hum.ownership_check.v0` summary consumed as the recognized local ownership fact gate, not as complete ownership, borrowing, alias, or memory safety
- `summary`: file, item, task, test, candidate, ready, blocked, error, warning,
  type-error, and body-grammar counts
- `pass_status`: current status for the pass names in `hum.ir_contract.v0`
- `lowering_candidates`: parsed source items that future lowering must handle
- `non_goals_v0`: claims this command must not make

## Resolver Summary Shape

`resolver` contains the summary fields from `hum.resolve.v0` needed by IR
readiness:

- `schema`: currently `hum.resolve.v0`
- `status`: `checked_resolver_v0`, `checked_resolver_with_errors_v0`, or
  `blocked_by_source_errors`
- `mode`: currently `source_analysis_only_no_type_or_borrow_check`
- `files`, `items`, `source_errors`, and `source_warnings`
- `scopes`, `definitions`, `references`, `resolved_references`,
  `unresolved_references`, and `external_references`
- `duplicate_definitions`, `mutable_place_errors`, `resolver_errors`, and
  `resolver_warnings`

A nonzero `resolver_errors` value blocks every V0 lowering candidate with
`checked_resolver_errors`. The command still reports the candidates because the
point is to show the next honest blocker, not to emit IR.

## Type Check Summary Shape

`type_check` contains the summary fields from `hum.type_check.v0` needed by IR
readiness:

- `schema`: currently `hum.type_check.v0`
- `status`: `declaration_annotations_and_trivial_returns_checked_v0`, `type_errors_v0`,
  `blocked_by_resolver_errors`, or `blocked_by_source_errors`
- `mode`: currently `declaration_annotation_and_trivial_return_check_v0`
- `source_errors`, `source_warnings`, and `resolver_errors`
- `checked_declarations`, `accepted_declarations`, and `rejected_declarations`
- `checked_type_references`, `unknown_type_references`, `checked_returns`, `accepted_returns`, `rejected_returns`, and `unchecked_returns`
- `type_errors` and `type_warnings`

A nonzero `type_errors` value blocks every V0 lowering candidate with `type_check_errors`. V0 type checking covers declaration annotations and trivial task returns only. `hum full-type-check` now consumes those facts and checks recognized body/Core statement type contexts; generic, trait, ownership, effect, layout, and ABI checks remain future blockers.

## Core Preview Summary Shape

`core_preview` contains the summary fields from `hum.core_preview.v0` needed by
IR readiness:

- `schema`: currently `hum.core_preview.v0`
- `status`: currently `preview_v0`
- `files`, `items`, `tasks`, `tests`, `core_candidates`, `errors`, and `warnings`
- `lowerable_preview_statements`, `contextual_preview_statements`, and
  `blocked_statements`
- `expression_previews`, `expression_ast_nodes`, and `typed_expression_previews`

`typed_expression_previews` counts expression roots whose type slot was populated
from checked return facts. It is a planning fact for future Core Hum lowering, not
a claim that broad expression type inference has run.

## Core Lower Summary Shape

`core_lower` contains the summary fields from `hum.core_lower.v0` needed by IR
readiness:

- `schema`: currently `hum.core_lower.v0`
- `status`: currently `unverified_core_artifact_v0`
- `files`, `items`, `tasks`, `tests`, and `core_items`
- `lowered_items`, `blocked_items`, `lowered_operations`, and `blocked_operations`
- `execution_ready` and `ir_ready`, both `0` in V0
- `errors`, `warnings`, `resolver_errors`, `type_errors`, and
  `preview_blocked_statements`

This summary proves only that a source-mapped, unverified Core Hum artifact
boundary exists. It does not verify Core Hum, execute code, or emit Hum IR.

## Core Verify Summary Shape

`core_verify` contains the summary fields from `hum.core_verify.v0` needed by IR
readiness:

- `schema`: currently `hum.core_verify.v0`
- `status`: currently `verified_non_executing_core_artifact_v0` when invariant checks pass
- `mode`: currently `non_executing_artifact_invariant_check_v0`
- `core_items`, `verified_items`, and `lower_blocked_items`
- `operations`, `verified_operations`, and `lower_blocked_operations`
- `checks`, `passed_checks`, and `failed_checks`
- `execution_ready` and `ir_ready`, both `0` in V0
- `errors`, `warnings`, `resolver_errors`, `type_errors`, and `preview_blocked_statements`

This summary verifies only artifact invariants: source spans,
operation/status/blocker consistency, and non-claim honesty. It does not execute
code, prove memory safety, optimize, or emit Hum IR.

## Full Type Check Summary Shape

`full_type_check` contains the summary fields from `hum.full_type_check.v0`
needed by IR readiness:

- `schema`: currently `hum.full_type_check.v0`
- `status`: `recognized_core_body_types_checked_v0`,
  `blocked_by_unchecked_body_types_v0`, `full_type_errors_v0`,
  `blocked_by_core_verify_errors`, `blocked_by_type_errors`,
  `blocked_by_resolver_errors`, or `blocked_by_source_errors`
- `mode`: currently `recognized_core_body_type_gate_v0`
- `source_errors`, `resolver_errors`, `type_errors`, and `core_verify_errors`
- `items`, `body_items`, and `statements`
- `checked_statements`, `accepted_statements`, `rejected_statements`,
  `unchecked_statements`, and `unsupported_statements`
- `blocking_issues`
- `execution_ready` and `ir_ready`, both `0` in V0

This summary checks only recognized V0 body/Core statement type contexts and
explicitly reports blockers. It does not claim complete type safety, effects,
ownership, memory safety, optimization, execution, or IR emission.

## Effect Check Summary Shape

`effect_check` contains the summary fields from `hum.effect_check.v0` needed by IR readiness:

- `schema`: currently `hum.effect_check.v0`
- `status`: `recognized_core_effects_checked_v0`, `effect_errors_v0`, `blocked_by_unchecked_effects_v0`, `blocked_by_full_type_check_errors`, `blocked_by_core_verify_errors`, `blocked_by_type_errors`, `blocked_by_resolver_errors`, or `blocked_by_source_errors`
- `mode`: currently `recognized_core_effect_gate_v0`
- `source_errors`, `resolver_errors`, `type_errors`, `core_verify_errors`, and `full_type_check_errors`
- `items`, `effect_items`, and `statements`
- `checked_statements`, `accepted_statements`, `rejected_statements`, and `unchecked_statements`
- `boundary_checks` and `rejected_boundary_checks`
- `blocking_issues`
- `execution_ready` and `ir_ready`, both `0` in V0

This summary checks only recognized V0 effect contexts and explicit boundary consistency. It does not claim complete effect safety, ownership, memory safety, profile enforcement, optimization, execution, or IR emission.

## Ownership Check Summary Shape

`ownership_check` contains the summary fields from `hum.ownership_check.v0` needed by IR readiness:

- `schema`: currently `hum.ownership_check.v0`
- `status`: `recognized_core_ownership_facts_checked_v0`, `ownership_errors_v0`, `blocked_by_unchecked_ownership_facts_v0`, `blocked_by_effect_check_errors`, `blocked_by_full_type_check_errors`, `blocked_by_core_verify_errors`, `blocked_by_type_errors`, `blocked_by_resolver_errors`, or `blocked_by_source_errors`
- `mode`: currently `recognized_core_ownership_gate_v0`
- prior gate error counts including `effect_check_errors`
- `items`, `ownership_items`, and `statements`
- `checked_statements`, `accepted_statements`, `rejected_statements`, and `unchecked_statements`
- `boundary_checks` and `rejected_boundary_checks`
- `blocking_issues`
- `execution_ready` and `ir_ready`, both `0` in V0

This summary checks only recognized V0 local ownership facts and explicit blockers. It does not claim complete ownership safety, borrow checking, alias safety, memory safety, profile enforcement, optimization, execution, or IR emission.

## Candidate Shape

Each `lowering_candidates` entry has:

- `id`: stable-ish source-derived readiness row ID
- `kind`: Hum item kind, such as `type`, `store`, `task`, `test`, or `app`
- `name`: source item name
- `graph_node_id`: semantic graph node ID for the same item
- `source_span`: file, line, and column
- `status`: readiness status, currently blocked by full-type-check errors, blocked by effect-check errors, blocked by ownership-check errors, or before allocation/resource checking once recognized body type, effect, and ownership gates pass
- `current_layer`: currently visible compiler layers
- `target_layer`: future target layer path
- `facts_available`: source facts already visible to tools
- `missing_passes`: pass boundaries still missing before honest IR
- `blocking_reasons`: reason strings explaining the blocked state; after the ownership gate, expected future blockers include `allocation_resource_check_not_implemented`, `profile_check_not_implemented`, and `ir_verify_not_implemented`
- `source_sections`: sections seen on the item
- `body_grammar`: optional partial V0 parse/classification of meaningful `does:`
  lines, when the item has a `does:` section

Current candidate statuses:

- `blocked_by_full_type_check_errors`: `hum.full_type_check.v0` reported type mismatches, unchecked recognized body contexts, unsupported body statements, or prior gate blockers
- `blocked_by_effect_check_errors`: `hum.effect_check.v0` reported missing declarations, unchecked recognized effect contexts, boundary contradictions, or prior gate blockers
- `blocked_by_ownership_check_errors`: `hum.ownership_check.v0` reported duplicate local ownership facts, unchecked ownership contexts, mutation authority contradictions, or prior gate blockers
- `blocked_before_allocation_resource_check`: source parsed, resolved, V0 type-checked, lowered to an unverified Core artifact, passed non-executing Core artifact verification, passed the recognized body type gate, passed the recognized effect gate, and passed the recognized ownership fact gate, but resource/profile and IR verification are still missing
- `blocked_by_core_verify_errors`: `hum.core_verify.v0` reported artifact invariant failures
- `blocked_by_source_errors`: source diagnostics include errors
- `blocked_by_resolver_errors`: `hum.resolve.v0` reported name, duplicate, or mutable-place errors
- `blocked_by_type_errors`: `hum.type_check.v0` reported declaration annotation or trivial return type errors

## Facts Available

V0 may report facts such as:

- `source_span`
- `semantic_graph_node_id`
- `item_kind`
- `item_name`
- `resolver_summary_v0`
- `checked_resolver_v0`
- `checked_resolver_with_errors_v0`
- `type_check_summary_v0`
- `declaration_annotations_and_trivial_returns_checked_v0`
- `type_errors_v0`
- `trivial_return_checks_v0`
- `core_preview_summary_v0`
- `preview_v0`
- `core_lower_summary_v0`
- `unverified_core_artifact_v0`
- `core_verify_summary_v0`
- `verified_non_executing_core_artifact_v0`
- `verified_core_artifact_rows_v0`
- `unverified_core_artifact_rows_v0`
- `full_type_check_summary_v0`
- `recognized_core_body_type_gate_available_v0`
- `recognized_core_body_types_checked_v0`
- `blocked_by_unchecked_body_types_v0`
- `recognized_body_type_facts_v0`
- `effect_check_summary_v0`
- `recognized_core_effect_gate_available_v0`
- `recognized_core_effects_checked_v0`
- `blocked_by_unchecked_effects_v0`
- `recognized_effect_facts_v0`
- `ownership_check_summary_v0`
- `recognized_core_ownership_gate_available_v0`
- `recognized_core_ownership_facts_checked_v0`
- `blocked_by_unchecked_ownership_facts_v0`
- `recognized_ownership_facts_v0`
- `checked_return_expression_type_slots_v0`
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

These facts are not full type checking, effect checking, ownership checking, or IR
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

This is grammar visibility only. It is not Core Hum lowering, full type checking,
effect checking, test execution, or interpretation. `hum core-preview` consumes
the same partial body grammar to emit Core Hum candidate operations and blockers;
`hum core-lower` consumes those facts plus checked resolver and V0 type-check
summaries to emit unverified source-mapped Core Hum artifact rows without
crossing into executable semantics.

## Pass Status

V0 reports these pass statuses:

- `parse`: `current`
- `semantic_graph_build`: `current`
- `resolve`: `checked_report_available`
- `body_grammar`: `partial_v0`
- `core_preview`: `preview_v0`
- `core_lowering`: `unverified_core_artifact_v0`
- `core_verify`: `verified_non_executing_core_artifact_v0`
- `type_check`: `declaration_and_trivial_return_check_available`
- `full_type_check`: `recognized_core_body_type_gate_available_v0`
- `effect_check`: `recognized_core_effect_gate_available_v0`
- `ownership_alias_check`: `recognized_core_ownership_gate_available_v0`
- `allocation_resource_check`: `not_implemented`
- `contract_evidence_linking`: `report_available_not_ir_pass`
- `profile_check`: `not_implemented`
- `ir_verify`: `not_implemented`

## Honesty Rules

- `hum ir-readiness` must not emit Hum IR.
- It must not execute generated code.
- It must not claim type safety, memory safety, optimization, backend readiness,
  or executable semantics.
- It may report source-visible facts, checked resolver facts, declaration
  type-check facts, partial body grammar facts, conservative core-preview facts,
  unverified core-lower summary facts, non-executing core-verify summary facts,
  recognized full-type-check summary facts, recognized effect-check summary facts, recognized ownership-check summary facts, and missing compiler passes.
- It must block V0 lowering candidates when `hum.resolve.v0` reports resolver
  errors.
- It must block V0 lowering candidates when `hum.type_check.v0` reports
  declaration annotation or trivial return type errors.
- It must block V0 lowering candidates when `hum.full_type_check.v0` reports
  type mismatches, unchecked recognized body contexts, unsupported statements,
  or prior gate blockers.
- It must block V0 lowering candidates when `hum.effect_check.v0` reports
  missing declarations, unchecked effect contexts, boundary contradictions, or prior gate blockers.
- It must stay in sync with `hum.core_contract.v0`, `hum.core_lower.v0`,
  `hum.core_verify.v0`, `hum.full_type_check.v0`, `hum.effect_check.v0`, `hum.ownership_check.v0`, `hum.ir_contract.v0`, `hum capabilities --format json`, and
  `hum version --format json`.

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
map from current parsed source through non-executing Core artifact verification toward the first honest IR milestone.