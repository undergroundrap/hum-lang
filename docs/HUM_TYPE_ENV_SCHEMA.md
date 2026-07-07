# Hum Type Environment Schema

Date: 2026-07-07

Current schema: `hum.type_env.v0`

## Purpose

`hum type-env` emits Hum's first declared type-environment inventory.

It is the bridge between checked resolution and real type checking. It records
source-declared type names, field annotations, store annotations, callable
parameter annotations, result annotations, and the resolver definition IDs those
facts are attached to when available.

This command does not type-check expressions. It exists so the next compiler
pass can consume one stable declaration surface instead of scraping parser
output, resolver JSON, or terminal prose.

## Command

```powershell
hum type-env <file-or-dir>...
hum type-env --format json <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- type-env examples/reference_surface.hum
cargo run -- type-env --format json examples/reference_surface.hum
```

## Top-Level Shape

```json
{
  "schema": "hum.type_env.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "type_environment_with_unknowns_v0",
  "milestone": "0 semantic graph",
  "mode": "declaration_inventory_no_type_check",
  "resolver": {},
  "summary": {},
  "reserved_type_roots": [],
  "type_names": [],
  "declarations": [],
  "non_claims_v0": []
}
```

## Status

`status` is one of:

- `type_environment_v0`: source parsed, resolver passed, and no unknown type
  names were found in annotations
- `type_environment_with_unknowns_v0`: source parsed and resolver passed, but
  at least one annotation names an undeclared or unreserved type
- `blocked_by_resolver_errors`: `hum.resolve.v0` reported resolver errors
- `blocked_by_source_errors`: source diagnostics include parse or Milestone 0
  check errors

Unknown type names are facts in `hum type-env`, not type-env errors. The
separate `hum type-check` gate consumes this report and currently rejects those
unknowns as `H0605` until imports, packages, or external type authority exist.

## Resolver Dependency

The `resolver` field is a compact `hum.resolve.v0` summary. `hum type-env` must
consume resolver definition IDs rather than inventing a parallel identity model.

If `resolver_errors` is nonzero, the report may still list declarations, but the
top-level status must be `blocked_by_resolver_errors`.

## Type Names

Each `type_names` entry has:

- `id`: type-env row ID
- `name`
- `normalized_name`
- `resolver_definition_id`
- `source_span`
- `status`, currently `declared_type_name_v0`

## Declarations

Each `declarations` entry has:

- `id`
- `declaration_kind`: `field`, `store`, `parameter`, or `result`
- `owner_kind`: `type`, `store`, `task`, or `test`
- `owner_name`
- `name`
- `resolver_definition_id`
- `source_span`
- `type_text`: the annotation text exactly enough for V0 review
- `type_references`: tokenized type names inside the annotation
- `status`

Declaration status values include:

- `reserved_type_annotation_v0`
- `references_declared_type_v0`
- `contains_unknown_type_names_v0`
- `missing_type_annotation_v0`

Each `type_references` row has `text`, `normalized_name`, `role`, and `status`.
Reference statuses are:

- `reserved_type_v0`
- `declared_type_v0`
- `unknown_type_name_v0`

## Reserved Type Roots

V0 reserves familiar root names such as `Unit`, `Bool`, `Int`, `UInt`, `Float`,
`Text`, `Bytes`, `Result`, `Option`, `Maybe`, `list`, `List`, `Vec`, `Slice`,
`Span`, `Map`, and `Set`.

These are reserved annotation roots, not a complete stdlib, layout, ABI, trait,
or generic semantics claim.

## Honesty Rules

- `hum type-env` must not claim full type checking.
- It must not infer expression types.
- It must not validate generics, traits, interfaces, layout, ABI, ownership, or
  borrowing.
- It must not execute generated code.
- It must keep resolver identity visible through `resolver_definition_id` where
  V0 can attach it.
- It must report unknown type names as unknown facts, not silently accept or
  reject them.

## Non-Goals For V0

V0 does not produce Core Hum, Hum IR, bytecode, machine code, proof artifacts,
layout evidence, ABI evidence, monomorphization plans, trait resolution, or
executable behavior.
