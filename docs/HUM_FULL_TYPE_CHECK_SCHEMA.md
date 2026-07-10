# Hum Full Type Check Schema

Date: 2026-07-09

Current schema: `hum.full_type_check.v0`

## Purpose

`hum full-type-check` is the first non-executing body/Core type gate. It runs
after declaration type checking and non-executing Core artifact verification. It
checks recognized V0 `does:` statement shapes and reports explicit blockers for
body syntax or type contexts it cannot yet justify.

This is not a full language type system. It is a compiler gate that keeps Hum
from claiming execution, type safety, memory safety, effects, ownership,
optimization, IR emission, backend readiness, or safety-critical readiness before
those passes exist.

## Command

```powershell
hum full-type-check [--format human|json] [--timings] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- full-type-check fixtures/full_type_check/simple_pass.hum
cargo run -- full-type-check --format json fixtures/full_type_check/simple_pass.hum
```

The human output is for terminals. The JSON output is for agents, CI wrappers,
compiler-roadmap checks, and `hum ir-readiness`.

## Top-Level Shape

```json
{
  "schema": "hum.full_type_check.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "recognized_core_body_types_checked_v0",
  "milestone": "0 semantic graph",
  "mode": "recognized_core_body_type_gate_v0",
  "core_contract_schema": "hum.core_contract.v0",
  "type_check_schema": "hum.type_check.v0",
  "core_verify_schema": "hum.core_verify.v0",
  "dependencies": {},
  "summary": {},
  "typed_items": [],
  "non_claims_v0": []
}
```

## Status Values

- `recognized_core_body_types_checked_v0`: every recognized V0 statement in the
  body was accepted by this narrow gate.
- `blocked_by_unchecked_body_types_v0`: the body contains recognized but
  unchecked V0 contexts such as record field initializers, iterator headers, or
  test expectations.
- `full_type_errors_v0`: this gate found a known type mismatch.
- `blocked_by_core_verify_errors`: Core artifact invariant checks failed first.
- `blocked_by_type_errors`: declaration or trivial return type checks failed
  first.
- `blocked_by_resolver_errors`: checked name resolution failed first.
- `blocked_by_source_errors`: source diagnostics include errors.

## Checked V0 Statements

The V0 gate checks only conservative statement contexts:

- `return`: expression must match the task result value type, including
  `Result T, E` success value extraction.
- `fail`: expression must match the `Result T, E` error type.
- `if_header` and `while_header`: condition must be recognized as `Bool`.
- `let_binding` and `mutable_binding`: explicit annotations are checked when
  present; otherwise simple local and literal facts may be inferred.
- exact writable field alias bindings infer the direct source field type from
  `change owner.field`; later `set alias = expression` checks the expression
  against that field type while ownership and lifetime rules remain deferred
  to `hum ownership-check`.
- a shared writable-alias candidate whose source shape is unsupported or whose
  initializer type is unknown reports
  `accepted_writable_field_alias_candidate_deferred_to_ownership_v0` with
  reason `writable_field_alias_shape_deferred_to_ownership_v0`. This status
  defers H0809 classification to `hum ownership-check`; it does not accept the
  source shape or infer a type. Unknown ordinary binding initializers remain
  blocking.
- `set_place`: the assigned expression must match a known local, parameter, or direct declared `root.field` place type.
- direct element reads such as `items[0]`: the result type is the element type of a local or parameter annotated as `List T`.
- simple task calls: known callee result types may type the call expression; `consume value` delegates to the consumed value type inside call arguments.
- explicit typed failure: `let value = try named_call(arguments)` accepts only
  equal nominal caller/callee error roots; `let value = try
  named_call(arguments) or fail CallerError.context` requires the wrapper root
  to equal the caller root. Rows expose `failure_form`, callee/caller/wrapper
  roots, call/callee/caller spans, stable diagnostic code, and repair help.
  Known fallible calls without `try` reject in every currently executable
  expression position, including nested operator/call arguments and loop
  collections, and the five other Session W misuse classes reject. `try` is
  recognized as a bounded keyword, so ordinary names such as `trying` and
  `try_value` retain their normal call meaning. Unsupported `try` shapes carry
  H0906 facts while Core preview/lower/verify retain matching blockers. A
  missing meaningful `fails when:` is typed here and deliberately deferred to
  the effect gate after higher-priority relationship errors are ruled out.
- `list_append(change list, item)`: the built-in minimal list-growth operation
  is typed as `Unit`; list literals are accepted against explicit `List ...`
  annotations without element-type validation in V0.
- `stdout_write(text)`: the only Session Z output built-in is typed as
  `Result Unit, OutputError`; explicit propagation sees success type `Unit`,
  and H0622 rejects any arity or checked argument type other than one `Text`.
- `block_close` and `loop_header`: accepted as statements with no expression
  type obligation.

## Current Blockers

V0 intentionally blocks or leaves unchecked:

- record field type contexts beyond direct declared `root.field` places and the record constructor root
- iterator element and index typing
- nested intent lowering
- test expectation typing
- store writes and other unsupported body lines
- overloads, view ownership semantics, traits, generics, layout, ABI, ownership, borrowing,
  effects, resources, profiles, and backend-specific typing

## Summary Fields

`summary` reports file and item counts, body item counts, statement counts,
accepted/rejected/unchecked/unsupported statement counts, dependency error
counts, `blocking_issues`, `execution_ready`, and `ir_ready`.

`execution_ready` and `ir_ready` are always `0` in V0.

## Honesty Rules

- `hum full-type-check` must not execute code.
- It must not emit Core Hum, Hum IR, backend input, bytecode, or machine code.
- It must not claim language-wide type safety.
- It must not claim effect safety, memory safety, ownership safety, profile
  enforcement, optimization, proof, certification, or safety-critical readiness.
- It may only claim recognized V0 body/Core statement type facts and explicit
  blockers.
- It must stay in sync with `hum.core_contract.v0`, `hum.type_check.v0`,
  `hum.core_verify.v0`, `hum.ir_readiness.v0`, `hum capabilities --format json`,
  and `hum version --format json`.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- no backend dependency
- no generated code execution

## Non-Goals For V0

V0 does not implement a complete type checker, effect checker, borrow checker,
profile checker, interpreter, optimizer, IR emitter, backend adapter, proof
producer, or safety-case generator. It makes no executable semantics claim and
keeps `no executable semantics` as a hard non-claim. It is the next compiler gate
on the way to those capabilities, not a substitute for them.
