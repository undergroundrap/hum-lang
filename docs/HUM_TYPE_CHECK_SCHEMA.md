# Hum Type Check Schema

Date: 2026-07-07

Current schema: `hum.type_check.v0`

## Purpose

`hum type-check` emits Hum's first type-check report.

V0 is intentionally narrow: it consumes `hum.type_env.v0` and validates declaration annotation names. A type name is accepted when it is declared in source or reserved by the current type environment. A type name is rejected when `hum type-env` marks it as `unknown_type_name_v0`.

This is the first compiler gate where unknown annotation names become type errors.

## Command

```powershell
hum type-check <file-or-dir>...
hum type-check --format json <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- type-check examples/reference_surface.hum
cargo run -- type-check --format json examples/reference_surface.hum
```

## Top-Level Shape

```json
{
  "schema": "hum.type_check.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "type_errors_v0",
  "milestone": "0 semantic graph",
  "mode": "declaration_annotation_check_no_expression_inference",
  "type_environment": {},
  "summary": {},
  "checked_declarations": [],
  "diagnostics": [],
  "non_claims_v0": []
}
```

## Status

`status` is one of:

- `declaration_annotations_checked_v0`: source parsed, resolver passed, and every checked declaration annotation references only declared or reserved type roots
- `type_errors_v0`: source parsed and resolver passed, but one or more declaration annotations contain unknown type names
- `blocked_by_resolver_errors`: checked resolution has errors, so type checking has no authority
- `blocked_by_source_errors`: source diagnostics include parse or Milestone 0 check errors

## Dependency On Type Environment

`type_environment` is a compact link to `hum.type_env.v0`. The type checker must consume the type environment rather than reparsing source or scraping terminal output.

`hum type-env` remains an inventory. It records unknown type names as facts. `hum type-check` is the gate that turns those unknown type facts into diagnostics.

## Diagnostics

V0 emits `H0605` for unknown type names.

Example:

```text
error[H0605]: unknown type `WorkError` in task result `return` annotation
```

The repair is to declare the type, use a reserved type root, or wait for imports/packages before relying on external type names.

## Checked Declarations

Each `checked_declarations` row has:

- `id`
- `declaration_id`: the matching row from `hum.type_env.v0`
- `declaration_kind`: `field`, `store`, `parameter`, or `result`
- `owner_kind`: `type`, `store`, `task`, or `test`
- `owner_name`
- `name`
- `resolver_definition_id`
- `source_span`
- `type_text`
- `type_references`
- `status`

Declaration status values include:

- `accepted_declaration_annotation_v0`
- `rejected_unknown_type_name_v0`
- `skipped_missing_type_annotation_v0`
- `not_checked_blocked_by_prior_errors_v0`

Each `type_references` row carries both the type-environment status and the type-check status. Type-check status values include:

- `accepted_type_reference_v0`
- `rejected_unknown_type_name_v0`
- `not_checked_prior_errors_v0`

## Honesty Rules

- `hum type-check` must not infer expression types.
- It must not type-check task or test body statements.
- It must not validate generic arity, trait bounds, interfaces, layout, ABI, ownership, borrowing, effects, or executable behavior.
- It must not execute generated code.
- It must keep `hum.type_env.v0` as the declaration source of truth.
- It must keep unknown external/package types rejected until imports or package authority exist.

## Non-Goals For V0

V0 does not produce Core Hum, Hum IR, bytecode, machine code, proof artifacts, layout evidence, ABI evidence, monomorphization plans, trait resolution, borrow checking, effect checking, or executable behavior.