# Hum Ownership Check Schema

Date: 2026-07-09

Current schema: `hum.ownership_check.v0`

## Purpose

`hum ownership-check` is the first non-executing ownership and alias-fact gate after recognized Core/body effect checking. It consumes `hum.effect_check.v0` readiness and checks only ownership facts the current Core/body grammar can honestly see.

V0 is intentionally conservative. It verifies parameter permission identity (`borrow`, `change`, `consume`), immutable local ownership from `let`, exclusive mutable local ownership from `change`, local field-view bindings of the exact form `let view = borrow record.field`, local element-view bindings of the exact form `let view = borrow list[0]`, and writable field aliases of the exact unannotated form `let alias = change owner.field` in one straight-line task body. A writable alias carries its source place, writes through with `set alias = value`, and has a binding-through-last-syntactic-use interval. H0808 rejects overlapping source access while that alias is live; H0809 rejects escape and every unsupported alias form. The gate also checks parameter mutation permission, direct field-place mutation, local moves, duplicate local places, Transaction-shaped exactly-once paths, and parameter-derived returned views through the closed `slice_until` derivation. It rejects stale field views after exact-field writes and stale element views after `list_append` growth. It does not infer general lifetimes, prove memory safety, check concurrency, validate unsafe provenance, implement retained element views, broad disjoint-field projection, internal references, general view expressions, general aliasing, or broad flow-sensitive borrowing.

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
- every statement row contains nullable `alias`, `place`, `binding_span`, `last_use_span`, `conflict_place`, and `conflict_span` fields; writable-alias rows populate the applicable facts, while unrelated rows keep them `null`
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
- local field views from `let view = borrow name.field`; writing that exact field invalidates the view, while writing a distinct direct field does not
- local element views from `let view = borrow list[0]`; `list_append(change list, item)` invalidates outstanding element views for that list
- exclusive mutable local ownership from `change name: Type = value`
- local mutation through `set name = value` when `name` was declared by `change`
- direct field-place mutation through `set name.field = value` when `name` is a `change` local or a parameter marked `change` or `consume`
- writable field aliases of exactly `let alias = change owner.field`, where `owner` is an earlier local `change` binding or a parameter marked `change` or `consume`
- real alias read/write-through to the current direct source field; `set alias = value` is reported as a change to `owner.field`
- straight-line alias lifetime from binding through last syntactic use, with definitely distinct direct fields accepted and a later same-field alias accepted after the prior last use
- live overlapping direct reads, direct writes, owner-wide access, or second writable aliases as H0808; structured rows name the alias, source place, binding, last use, conflict place, and conflict span
- branch/loop creation or use, passing, returning, storing, `borrow`/`change`/`consume` wrapping, alias-to-alias binding, nested places, list elements, alias/live-owner rebinding, and shadowing an already-visible parameter, local, or declared permission as H0809
- parameter mutation through `set name = value` only when the parameter is marked `change` or `consume`
- local moves when a local is passed as `consume name` or returned
- use after move as `H0801`, including double consume of the same ordinary local
- borrowed-parameter writes, including `set borrowed.field = value` and writable-alias acquisition from `borrowed.field`, as `H0802`
- Transaction-shaped local bindings as the first recognized linear resources
- missing linear-resource consume on any recognized path as `H0803`, with the path named in `help`
- double consume of a recognized linear resource as `H0804`, with the earlier consuming action named in `help`
- returned-view dependencies of the shape `ResultType from parameter` when the source is a task parameter and each V0 return visibly returns that bare parameter or a closed-set `slice_until(source, separator)` derivation
- rejected returned-view dependencies as `H0805` when the source is a local, an internal reference such as `parser.buffer`, an unknown name, a non-closed derivation chain, or a nonmatching returned expression
- stale local field-view use as `H0807`, with `help` naming the field-view binding site, the invalidating write site, and the repair choices: re-borrow after the write or copy the field value before the write
- stale local element-view use as `H0807`, with `help` naming the element-view binding site, the invalidating `list_append` site, and the repair choices: re-borrow after the append or copy the element value before the append
- external changes as deferred to the later resource check when the target is not a local or parameter place and the effect gate already accepted `changes:`
- duplicate local place names inside one `does:` body as ownership errors
- unsupported statements as explicit ownership blockers

## Statement Statuses

- `accepted_immutable_local_owner_v0`
- `accepted_mutable_local_owner_v0`
- `accepted_field_view_borrow_v0`
- `accepted_element_view_borrow_v0`
- `accepted_writable_field_alias_v0`
- `accepted_writable_field_alias_write_through_v0`
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
- `rejected_stale_field_view_use_v0`
- `rejected_stale_element_view_use_v0`
- `rejected_writable_field_alias_without_mutation_authority_v0`
- `rejected_writable_field_alias_overlap_v0`
- `rejected_unsupported_writable_field_alias_v0`
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
- It must not claim complete ownership safety, borrow safety, general lifetime inference, general alias safety, memory safety, allocation safety, profile enforcement, optimization, or backend readiness.
- It may report recognized V0 ownership facts, direct field-place mutation facts, exact straight-line local writable-field-alias facts, local field-view bindings and exact-field invalidations, local element-view bindings and `list_append` growth invalidations, narrow Transaction-shaped linear-resource path facts, returned-view dependencies from parameters through bare returns or closed `slice_until` derivations, and explicit blockers only.
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

V0 does not provide a complete ownership or borrowing system. Session V adds no general aliases, internal references, nested/element aliases, stored aliases, or broad flow-sensitive borrowing. It is a narrow source-visible gate that lets the compiler honestly move from recognized effect visibility to local ownership facts before future resource, profile, IR verification, and backend work.

## Session AL Callable Facts

The nonescaping definition handle reports
`not_applicable_to_al_ordinary_value_v0`; it proves no capture, environment,
borrow, move, or general callable ownership rule.
For the Session AM nonempty-row slice the same nonclaim is reported as
`not_applicable_to_am_ordinary_value_v0`; row propagation does not upgrade it
into ownership, lifetime, resource, or capture evidence.
