# Hum Effect Check Schema

Date: 2026-07-09

Current schema: `hum.effect_check.v0`

## Purpose

`hum effect-check` is the first non-executing effect gate after recognized Core/body type checking. It consumes `hum.full_type_check.v0` readiness and checks only effect facts that the current Core/body grammar can honestly see.

V0 is intentionally conservative. It verifies obvious local mutation permission, writable-field-alias bindings and write-through changes deferred to ownership, declared external mutation, visible `fail` statements, declared ambient reads for recognized resource roots, and basic security/trust boundary consistency. It reports explicit blockers for unsupported or unchecked effect contexts.

This command does not execute source, emit Hum IR, prove memory safety, enforce ownership, enforce runtime profiles, prove allocation safety, or claim a complete effect system.

## Command

```powershell
hum effect-check [--format human|json] [--timings] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- effect-check fixtures/effect_check/simple_pass.hum
cargo run -- effect-check --format json fixtures/effect_check/simple_pass.hum
```

The human output is for terminals. The JSON output is for agents, CI wrappers, compiler-roadmap checks, and future ownership/profile/IR verifier work.

## Top-Level Shape

```json
{
  "schema": "hum.effect_check.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "recognized_core_effects_checked_v0",
  "milestone": "0 semantic graph",
  "mode": "recognized_core_effect_gate_v0",
  "core_contract_schema": "hum.core_contract.v0",
  "full_type_check_schema": "hum.full_type_check.v0",
  "dependencies": {},
  "summary": {},
  "effect_items": [],
  "non_claims_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.effect_check.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: aggregate effect-check status
- `milestone`: current implementation milestone
- `mode`: currently `recognized_core_effect_gate_v0`
- `core_contract_schema`: Core Hum contract this gate measures against
- `full_type_check_schema`: prior gate this command consumes
- `dependencies`: consumed summary facts, currently `full_type_check`
- `summary`: counts for declarations, inferred V0 effects, blockers, and non-readiness flags
- `effect_items`: per-body item declarations, statement effect rows, and boundary checks
- `non_claims_v0`: claims this command must not make

## Statuses

- `recognized_core_effects_checked_v0`: every recognized V0 effect row and boundary check passed
- `effect_errors_v0`: recognized V0 effect rows or boundary checks contradicted declarations
- `blocked_by_unchecked_effects_v0`: at least one visible statement has effects V0 cannot check yet
- `blocked_by_full_type_check_errors`: `hum.full_type_check.v0` did not pass
- `blocked_by_core_verify_errors`: prior Core verification did not pass
- `blocked_by_type_errors`: prior type checking did not pass
- `blocked_by_resolver_errors`: checked resolution did not pass
- `blocked_by_source_errors`: source diagnostics include errors

## Summary Shape

`summary` includes:

- `files`, `items`, `effect_items`, and `statements`
- `checked_statements`, `accepted_statements`, `rejected_statements`, and `unchecked_statements`
- `boundary_checks` and `rejected_boundary_checks`
- `declared_uses`, `declared_changes`, `declared_failures`, `declared_allocations`, `declared_avoids`, `declared_protects`, and `declared_trusts`
- `inferred_reads`, `inferred_changes`, and `inferred_failures`
- `blocking_issues` plus prior gate error counts
- `execution_ready` and `ir_ready`, both always `0` in V0

## Effect Item Shape

Each `effect_items` entry has:

- `id`: source-derived row ID
- `kind`: Hum item kind
- `name`: item name
- `source_span`: file, line, and column
- `status`: per-item status
- `declarations`: meaningful lines from `uses:`, `changes:`, `fails when:`, `allocates:`, `avoids:`, `protects:`, and `trusts:`
- `statements`: recognized body statement effect rows
- `boundary_checks`: item-level consistency checks such as trust/protect relationships

## Checked V0 Effects

V0 recognizes and checks:

- local mutation permission from `change name: Type = value`
- local mutation through `set name = value` when `name` was declared by `change`
- `let alias = change owner.field` candidates as writable-alias effect rows
  deferred to ownership; exact candidates expose `change owner.field`
- `set alias = value` as `writable_field_alias_write_through` targeting the
  exact `owner.field` place and counted in `inferred_changes`
- parameter mutation through `set parameter = value` is accepted as a parameter-permission fact deferred to `hum ownership-check`
- external mutation through `set target = value` or `save value in target` only when `target` is declared under `changes:`
- `fail value` only when the item has a meaningful `fails when:` section
- obvious ambient reads for known roots such as `clock`, `time`, `random`, `crypto`, `file`, `network`, `env`, `process`, `os`, `registry`, `device`, `sensor`, `storage`, `database`, and `http` only when declared under `uses:`
- `trusts:` only when paired with `protects:`
- security-sensitive uses or body text such as `random`, `crypto`, `password`, `token`, `network`, or `socket` only when paired with `protects:`
- simple `avoids:` contradictions when a declared avoided resource or failure is visibly used

## Statement Statuses

- `accepted_no_external_effect_v0`
- `accepted_local_mutation_permission_v0`
- `accepted_local_mutation_v0`
- `accepted_writable_field_alias_deferred_to_ownership_v0`
- `accepted_writable_field_alias_write_deferred_to_ownership_v0`
- `accepted_declared_change_v0`
- `accepted_parameter_mutation_deferred_to_ownership_v0`
- `accepted_declared_failure_v0`
- `accepted_declared_use_v0`
- `rejected_missing_changes_declaration_v0`
- `rejected_missing_fails_when_declaration_v0`
- `rejected_missing_uses_declaration_v0`
- `unchecked_statement_effect_v0`
- `not_checked_blocked_by_prior_errors_v0`

## Boundary Check Statuses

- `accepted_trust_boundary_has_protects_v0`
- `accepted_security_effect_has_protects_v0`
- `rejected_trust_without_protects_v0`
- `rejected_security_effect_without_protects_v0`
- `rejected_avoids_contradicted_v0`

## Honesty Rules

- `hum effect-check` must not execute code.
- no executable semantics
- It must not claim executable semantics.
- It must not emit Core Hum, Hum IR, bytecode, machine code, backend adapter input, proof artifacts, optimized code, or executable behavior.
- It must not claim complete effect safety, memory safety, ownership safety, allocation safety, profile enforcement, optimization, or backend readiness.
- It may report recognized V0 effect facts and explicit blockers only.
- It must block when `hum.full_type_check.v0` reports blockers.
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

V0 does not provide a complete effect system. It is a narrow source-visible gate that lets the compiler honestly move from recognized body typing to effect visibility before future ownership, profile, IR verification, and backend work.
