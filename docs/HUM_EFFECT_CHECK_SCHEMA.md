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
- `boundary_checks`: item-level consistency checks such as trust/protect relationships and Session Y source-capability policy routes

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
- recognized same-root propagation and caller-root causal wrapping count as
  typed failure and require the same meaningful `fails when:` declaration;
  statement rows expose their nominal roots and call/callee/caller sites
- failure-declaration quality reuses the hollow-contract rule, so placeholders
  such as `todo`, `tbd`, tautologies, and generic claims do not satisfy H0907
- direct failure, same-root propagation, and caller-root wrapping each
  contradict an `avoids: failure` declaration
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
- `accepted_declared_failure_propagation_v0`
- `accepted_declared_failure_wrap_v0`
- `rejected_typed_failure_relationship_v0`
- `accepted_declared_use_v0`
- `rejected_missing_changes_declaration_v0`
- `rejected_missing_fails_when_declaration_v0`
- `rejected_missing_uses_declaration_v0`
- `unchecked_statement_effect_v0`
- `not_checked_blocked_by_prior_errors_v0`

## Boundary Check Statuses

- `accepted_trust_boundary_has_protects_v0`
- `accepted_security_effect_has_protects_v0`
- `accepted_source_capability_budget_v0`
- `accepted_app_capability_maximum_v0`
- `accepted_caller_capability_closure_v0`
- `accepted_app_capability_closure_v0`
- `not_checked_app_closure_blocked_by_caller_v0`
- `rejected_trust_without_protects_v0`
- `rejected_security_effect_without_protects_v0`
- `rejected_avoids_contradicted_v0`
- `rejected_unknown_source_capability_v0`
- `rejected_missing_caller_capability_v0`
- `rejected_app_capability_mismatch_v0`
- `accepted_declared_output_operation_v0`
- `rejected_missing_output_source_authority_v0`
- `rejected_output_reachable_recursion_v0`
- `accepted_declared_runner_replay_operation_v0`
- `rejected_missing_replay_source_authority_v0`
- `rejected_replay_reachable_recursion_v0`

Session Y capability boundary rows reuse the existing schema and add structured
policy fields only on those rows: exact `capability_id`, reserved `core_effect`
and runtime/target meaning, `grant_kind`, `grant_scope`, `grant_strength`,
`grant_lifetime`, `severity_tier`, `mapping_status`, app/caller/callee names and
spans, declaration and entry spans, ordered `route_tasks`/`route_spans`, stable
diagnostic identity, and repair help. The row `id` is the stable source-policy
join key. Call-backed rows derive it from the exact lexical call site, so
different callees and repeated calls on one statement remain unique and
deterministic. It can later join an operator decision and operation exercise
event. Session Z's runtime records the first typed in-memory decision and
exercise facts against this ID. Reachable output rows carry the complete
structural app-to-start-to-leaf `route_tasks` and every intervening call plus
output occurrence in `route_spans`; distinct paths to one leaf receive
distinct stable IDs. Policy identity normalizes source path separators, while
the structured spans remain display evidence. `hum effect-check` remains
non-executing and does not emit
those runtime events. Output-reachable recursive cycles are rejected under
H0624 because this bounded slice does not claim an infinite or summarized audit
route.

The three pinned capabilities are typed exact one-run budgets. Unknown
sandbox-bypass spellings such as process, FFI, unsafe, or unrestricted import
are classified under `sandbox_bypass_authority`, never laundered into the
ordinary tier. Session Y implements no wildcard, persistence, prompt, operator
grant, deny, host operation, or audit log. Session Z adds only exact
`stdout.write` grant/deny at runtime; output call rows carry Core effect
`output`, exact lexical call provenance, and the implemented bounded-adapter
mapping status. Session AA adds exact `clock.replay` grant/deny and ordered
runner input; replay call rows carry Core effect `time`, the complete finite
route, and `implemented_runner_replay_input_v0_no_os.clock`. Runtime decision
and exercise facts record the selected policy ID, sequence index, and consumed
tick in memory. No wildcard, persistence, prompt, host clock, runtime JSON, or
broader capability is added.

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
