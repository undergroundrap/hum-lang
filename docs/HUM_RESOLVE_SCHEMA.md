# Hum Resolve Schema

Date: 2026-07-09
Schema: `hum.resolve.v0`

## Purpose

`hum resolve` emits the first checked name, scope, reference, and mutable-place
report for Hum source.

This report is the bridge between parser-shaped facts and later type, effect,
ownership, borrowing, IR, debugger, and editor features. It answers:

- which scopes exist
- which definitions exist in each scope
- which references resolve to which definitions
- which `uses:` and `changes:` names link to known source definitions when possible
- which `set` and `save ... in ...` targets require mutable permission
- which resolver diagnostics were found

## Command

```powershell
hum resolve <file-or-dir>...
hum resolve --format json <file-or-dir>...
```

The report is local-only and source-analysis-only. It does not execute Hum code.

## Top-Level Shape

```json
{
  "schema": "hum.resolve.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "checked_resolver_v0",
  "milestone": "0 semantic graph",
  "mode": "source_analysis_only_no_type_or_borrow_check",
  "summary": {},
  "scopes": [],
  "definitions": [],
  "references": [],
  "diagnostics": [],
  "non_claims_v0": []
}
```

`status` is one of:

- `checked_resolver_v0`
- `checked_resolver_with_errors_v0`
- `blocked_by_source_errors`

V0 mode is always `source_analysis_only_no_type_or_borrow_check`.

## Summary

`summary` contains counts for files, items, source diagnostics, scopes,
definitions, references, resolved references, unresolved references, external
references, duplicate definitions, mutable-place errors, resolver errors, and
resolver warnings.

## Scopes

Each scope has:

- `id`
- `parent_scope_id`
- `scope_kind`
- `owner_kind`
- `owner_name`
- `source_span`

V0 scope kinds include:

- `file`
- `app`
- `type`
- `callable`
- `if`
- `while`
- `loop`
- `for_each`
- `for_index`

## Definitions

Each definition has:

- `id`
- `name`
- `normalized_name`
- `definition_kind`
- `scope_id`
- `mutable`
- `state_kind`
- `source_span`
- `status`
- `duplicate_of`

V0 definition kinds include:

- `app`
- `type`
- `field`
- `store`
- `task`
- `test`
- `parameter` with `state_kind` values such as `borrow_parameter`, `change_parameter`, or `consume_parameter`
- `declared_use_permission`
- `declared_change_permission`
- `let_binding`
- `mutable_binding`
- `writable_field_alias`, with `mutable: true` and `state_kind:
  writable_field_alias` for a `let name = change ...` candidate so ownership
  H0808/H0809, rather than resolver H0603, owns the alias rule
- `for_each_binding`
- `for_index_binding`

`status` is `defined_v0`, `duplicate_definition_v0`, or the narrow
`duplicate_definition_deferred_to_ownership_v0`. The deferred value is used
only when a writable-alias candidate or live-alias rebinding would otherwise
let generic H0602 mask the ownership-specific H0809 rejection; the original
visible definition remains authoritative in the resolver scope.

## References

Each reference has:

- `id`
- `name`
- `normalized_name`
- `reference_kind`
- `scope_id`
- `mutable_required`
- `source_span`
- `resolution_status`
- `resolved_definition_id`
- `reason`

V0 reference kinds include:

- `declared_use`
- `declared_change`
- `name_ref`
- `path_root_ref`
- `callee_ref`
- `mutation_target`
- `store_write_value`
- `store_write_target`

`resolution_status` is one of:

- `resolved_v0`
- `external_reference_v0`
- `builtin_reference_v0` for the exact Session Z `stdout_write` callee only
- `unresolved_v0`
- `resolved_immutable_place_v0`

Parameter mutation targets resolve as parameter references so the later ownership gate can apply `borrow`/`change`/`consume` authority and emit ownership diagnostics. Ordinary immutable local mutation still resolves as `resolved_immutable_place_v0`.

Writable-field-alias candidates resolve their source expression before the
alias definition is added. A later `set alias = value` therefore resolves as a
mutable alias target, while `hum ownership-check` remains responsible for the
exact direct-field shape, source authority, last-use interval, overlap, and
escape rules. Resolver recognition is not a general alias-safety claim.

## Diagnostics

Resolver diagnostics use stable `H060x` codes:

- `H0601`: unresolved name
- `H0602`: duplicate name in scope
- `H0603`: set target is immutable
- `H0604`: read before declaration

These diagnostics are emitted by `hum resolve`. They are cataloged by
`hum diagnostics`, but V0 `hum check` remains the existing Milestone 0 checker
and does not yet run the full resolver.

## IR Readiness Link

`hum ir-readiness --format json` consumes a compact `hum.resolve.v0` summary as
its `resolver` field. Resolver status and counts are not IR; they are the gate
that says whether parsed source is allowed to move toward Core Hum lowering.
When `resolver_errors` is nonzero, V0 lowering candidates are reported as
`blocked_by_resolver_errors` with the `checked_resolver_errors` blocker.

## Non-Claims

V0 explicitly does not claim:

- type checking
- borrow checking
- lifetime inference
- effect checking
- module import resolution
- executable semantics
- optimizer authority

## Design Rule

Every later executable, type, effect, ownership, borrowing, debugger, LSP, and
IR feature should either consume this report or explain why it needs a stricter
successor schema.
