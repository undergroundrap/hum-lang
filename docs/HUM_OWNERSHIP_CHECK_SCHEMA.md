# Hum Ownership Check Schema

Date: 2026-07-08

Current schema: `hum.ownership_check.v0`

## Purpose

`hum ownership-check` is the first non-executing ownership and alias-fact gate after recognized Core/body effect checking. It consumes `hum.effect_check.v0` readiness and checks only ownership facts the current Core/body grammar can honestly see.

V0 is intentionally conservative. It verifies parameter permission identity (`borrow`, `change`, `consume`), immutable local ownership from `let`, exclusive mutable local ownership from `change`, parameter mutation permission through `set`, direct field-place mutation through `set record.field = value`, local moves caused by `consume` arguments and returns, duplicate local place names, and explicit blockers inherited from prior gates. It checks the first recognized linear-resource class: local bindings with Transaction-shaped annotations must be consumed exactly once on every recognized `if`/`return`/`fail` path. It also checks the first returned-view dependency form: a task result such as `Slice Text from text` may depend only on a task parameter, and the V0 executable body must visibly return that bare parameter or a closed-set view derivation through `slice_until(source, separator)`. It does not infer lifetimes, prove memory safety, check concurrency, validate unsafe provenance, or implement field views, stale field-view invalidation, broad disjoint-field projection, internal references, general view expressions, or broad flow-sensitive borrowing.

This command does not execute source, emit Hum IR, prove memory safety, enforce borrowing, enforce runtime profiles, prove allocation safety, or claim a complete ownership system.

## Command

```powershell
hum ownership-check [--format human|json] [--timings] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- ownership-check fixtures/ownership_check/simple_pass.hum
cargo run -- ownership-check --format json fixtures/ownership_check/simple_pass.hum
```

The human output is for terminals. The JSON output is for agents, CI wrappers, compiler-roadmap checks, and future resource/profile/IR verifier work.

## Top-Level Shape

```json
{
  "schema": "hum.ownership_check.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "recognized_core_ownership_facts_checked_v0",
  "milestone": "0 semantic graph",
  "mode": "recognized_core_ownership_gate_v0",
  "core_contract_schema": "hum.core_contract.v0",
  "effect_check_schema": "hum.effect_check.v0",
  "dependencies": {},
  "summary": {},
  "ownership_items": [],
  "non_claims_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.ownership_check.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: aggregate ownership-check status
- `milestone`: current implementation milestone
- `mode`: currently `recognized_core_ownership_gate_v0`
- `core_contract_schema`: Core Hum contract this gate measures against
- `effect_check_schema`: prior gate this command consumes
- `dependencies`: consumed summary facts, currently `effect_check`
- `summary`: counts for local ownership facts, blockers, and non-readiness flags
- `ownership_items`: per-body item declarations, statement ownership rows, return dependencies, and boundary checks
- statement and return-dependency rows may include `diagnostic_code` and `help` when a stable ownership diagnostic is attached
- `non_claims_v0`: claims this command must not make

## Statuses
- `recognized_core_ownership_facts_checked_v0`: every recognized V0 ownership row and boundary check passed
- `ownership_errors_v0`: recognized V0 ownership rows or boundary checks contradicted local ownership facts
- `blocked_by_unchecked_ownership_facts_v0`: at least one visible statement has ownership implications V0 cannot check yet
- `blocked_by_effect_check_errors`: `hum.effect_check.v0` did not pass
- `blocked_by_full_type_check_errors`: prior full type checking did not pass
- `blocked_by_core_verify_errors`: prior Core verification did not pass
- `blocked_by_type_errors`: prior type checking did not pass
- `blocked_by_resolver_errors`: checked resolution did not pass
- `blocked_by_source_errors`: source diagnostics include errors

## Summary Shape

`summary` includes:

- `files`, `items`, `ownership_items`, and `statements`
- `checked_statements`, `accepted_statements`, `rejected_statements`, and `unchecked_statements`
- `boundary_checks` and `rejected_boundary_checks`
- declared section counts carried forward for context
- inferred local read/change/failure counts carried forward for compatibility with the current gate scaffold
- `blocking_issues` plus prior gate error counts
- `execution_ready` and `ir_ready`, both always `0` in V0

## Checked V0 Ownership Facts

V0 recognizes and checks:

- task and test parameters as readable parameter places, with `borrow` as the default permission
- `change` parameters as explicit writable parameter places
- `consume` parameters as owned authority inside the callee
- immutable local ownership from `let name: Type = value`
- exclusive mutable local ownership from `change name: Type = value`
- local mutation through `set name = value` when `name` was declared by `change`
- direct field-place mutation through `set name.field = value` when `name` is a `change` local or a parameter marked `change` or `consume`
- parameter mutation through `set name = value` only when the parameter is marked `change` or `consume`
- local moves when a local is passed as `consume name` or returned
- use after move as `H0801`, including double consume of the same ordinary local
- borrowed-parameter writes, including `set borrowed.field = value`, as `H0802`
- Transaction-shaped local bindings as the first recognized linear resources
- missing linear-resource consume on any recognized path as `H0803`, with the path named in `help`
- double consume of a recognized linear resource as `H0804`, with the earlier consuming action named in `help`
- returned-view dependencies of the shape `ResultType from parameter` when the source is a task parameter and each V0 return visibly returns that bare parameter or a closed-set `slice_until(source, separator)` derivation
- rejected returned-view dependencies as `H0805` when the source is a local, an internal reference such as `parser.buffer`, an unknown name, a non-closed derivation chain, or a nonmatching returned expression
- external changes as deferred to the later resource check when the target is not a local or parameter place and the effect gate already accepted `changes:`
- duplicate local place names inside one `does:` body as ownership errors
- unsupported statements as explicit ownership blockers

## Statement Statuses

- `accepted_immutable_local_owner_v0`
- `accepted_mutable_local_owner_v0`
- `accepted_exclusive_local_mutation_v0`
- `accepted_external_change_deferred_to_resource_check_v0`
- `accepted_no_ownership_transfer_v0`
- `accepted_parameter_mutation_v0`
- `accepted_disjoint_field_mutation_v0`
- `accepted_consume_argument_move_v0`
- `accepted_return_move_v0`
- `rejected_duplicate_local_place_v0`
- `rejected_mutating_immutable_local_v0`
- `rejected_mutating_borrowed_parameter_v0`
- `rejected_use_after_move_v0`
- `rejected_linear_resource_not_consumed_v0`
- `rejected_linear_resource_consumed_twice_v0`
- `rejected_missing_mutation_authority_v0`
- `unchecked_statement_ownership_v0`
- `not_checked_blocked_by_prior_errors_v0`

## Return Dependency Statuses

- `accepted_return_dependency_parameter_v0`
- `accepted_return_dependency_closed_view_derivation_v0`
- `rejected_return_dependency_local_v0`
- `rejected_return_dependency_internal_reference_v0`
- `rejected_return_dependency_unknown_source_v0`
- `rejected_return_dependency_returned_local_v0`
- `rejected_return_dependency_source_mismatch_v0`
- `rejected_return_dependency_complex_expression_v0`

## Boundary Check Statuses
- `rejected_duplicate_local_place_v0`
- inherited trust/protect boundary statuses are still reported by the V0 scaffold but are not ownership safety claims

## Honesty Rules

- `hum ownership-check` must not execute code.
- no executable semantics
- It must not claim executable semantics.
- It must not emit Core Hum, Hum IR, bytecode, machine code, backend adapter input, proof artifacts, optimized code, or executable behavior.
- It must not claim complete ownership safety, borrow safety, lifetime inference, alias safety, memory safety, allocation safety, profile enforcement, optimization, or backend readiness.
- It may report recognized V0 ownership facts, direct field-place mutation facts, narrow Transaction-shaped linear-resource path facts, returned-view dependencies from parameters through bare returns or closed `slice_until` derivations, and explicit blockers only.
- It must block when `hum.effect_check.v0` reports blockers.
- It must stay in sync with `hum core-contract --format json`, `hum ir-readiness --format json`, `hum capabilities --format json`, and `hum version --format json`.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- no backend dependency
- no generated code execution

## Non-Goals For V0

V0 does not provide a complete ownership or borrowing system. It is a narrow source-visible gate that lets the compiler honestly move from recognized effect visibility to local ownership facts before future resource, profile, IR verification, and backend work.