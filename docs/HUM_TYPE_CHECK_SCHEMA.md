# Hum Type Check Schema

Date: 2026-07-07

Current schema: `hum.type_check.v0`

## Purpose

`hum type-check` emits Hum's first type-check report.

V0 is intentionally narrow: it consumes `hum.type_env.v0`, validates declaration annotation names, and checks task `return` expressions only when the return expression has a trivial source-visible type. A type name is accepted when it is declared in source or reserved by the current type environment. A type name is rejected when `hum type-env` marks it as `unknown_type_name_v0`.

This is the first compiler gate where unknown annotation names and obvious return type mismatches become type errors.

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
  "mode": "declaration_annotation_and_trivial_return_check_v0",
  "type_environment": {},
  "summary": {},
  "checked_declarations": [],
  "checked_returns": [],
  "diagnostics": [],
  "non_claims_v0": []
}
```

## Status

`status` is one of:

- `declaration_annotations_and_trivial_returns_checked_v0`: source parsed, resolver passed, every checked declaration annotation references only declared or reserved type roots, and every trivially typed task return expression is compatible with the task result type
- `type_errors_v0`: source parsed and resolver passed, but one or more declaration annotations contain unknown type names or one or more trivially typed returns mismatch their task result type
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

V0 emits `H0606` for return type mismatches only when both sides are trivial enough to know without full expression typing.

Example:

```text
error[H0606]: return expression `title` has type `Text` but task `bad return` returns `UInt`
```

The repair is to declare missing types, return a value compatible with the task result type, change the task result annotation, or leave complex return expressions unchecked until full expression typing exists.

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

## Checked Returns

Each `checked_returns` row has:

- `id`
- `owner_kind`: currently `task`
- `owner_name`
- `source_span`
- `expression_text`
- `expected_type`: the task result annotation, or `null`
- `expected_value_type`: for `Result T, E`, `Option T`, or `Maybe T`, the success/value type `T`; otherwise the task result type
- `actual_type`: the trivial source-visible expression type, or `null`
- `type_source`: why the actual type is known, such as `parameter_annotation_v0`, `binding_annotation_v0`, `record_literal_constructor_v0`, `text_literal_v0`, or `bool_literal_v0`
- `status`
- `reason`

Return status values include:

- `accepted_return_expression_v0`
- `rejected_return_type_mismatch_v0`
- `unchecked_return_expression_v0`
- `skipped_no_result_annotation_v0`
- `not_checked_blocked_by_prior_errors_v0`

V0 return checking recognizes only trivial expression types: parameters, explicitly annotated `let` or `change` bindings, bindings initialized from trivial literals or record literal constructors, direct trivial literals, direct name references to known locals, and type-looking path roots. It treats `Result`, `Option`, and `Maybe` returns as accepting their success/value type. It does not resolve calls, fields, overloads, operators, generics, traits, effects, ownership, or execution.

## Honesty Rules

- `hum type-check` must not claim full expression type inference.
- It must not broadly type-check task or test body statements.
- It may check task `return` expressions only when their source-visible type is trivial under V0 rules.
- It must not type-check calls, overloads, fields, operators, generic arity, trait bounds, interfaces, layout, ABI, ownership, borrowing, effects, or executable behavior.
- It must not execute generated code.
- It must keep `hum.type_env.v0` as the declaration source of truth.
- It must keep unknown external/package types rejected until imports or package authority exist.

## Non-Goals For V0

V0 does not produce Core Hum, Hum IR, bytecode, machine code, proof artifacts, layout evidence, ABI evidence, monomorphization plans, trait resolution, borrow checking, effect checking, or executable behavior.