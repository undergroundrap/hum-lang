# Hum Core Verify Schema

Date: 2026-07-08

Current schema: `hum.core_verify.v0`

## Purpose

`hum core-verify` is the first verifier for the non-executing Core Hum artifact
boundary emitted by `hum core-lower`. It checks that source-mapped Core rows are
internally consistent before later passes are allowed to use them as evidence for
Hum IR planning.

This command is intentionally narrow. It verifies artifact invariants, not
program behavior. A passing `hum.core_verify.v0` report means the current
unverified Core rows kept sane source spans, coherent operation/status/blocker
relationships, and honest non-claims. It does not mean the program can execute,
that Hum IR exists, that effects or ownership are checked, or that memory safety,
proof, optimization, backend, profile, or release claims are established.

## Command

```powershell
hum core-verify [--format human|json] [--timings] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- core-verify examples/reference_surface.hum
cargo run -- core-verify --format json examples/reference_surface.hum
```

The human output is for terminals. The JSON output is for agents, CI wrappers,
`hum ir-readiness`, and future Core Hum / Hum IR verifier work.

## Top-Level Shape

```json
{
  "schema": "hum.core_verify.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "verification_status": "verified_non_executing_core_artifact_v0",
  "mode": "non_executing_artifact_invariant_check_v0",
  "core_contract_schema": "hum.core_contract.v0",
  "core_lower_schema": "hum.core_lower.v0",
  "core_preview_schema": "hum.core_preview.v0",
  "resolve_schema": "hum.resolve.v0",
  "type_check_schema": "hum.type_check.v0",
  "ir_contract_schema": "hum.ir_contract.v0",
  "summary": {},
  "core_lower": {},
  "core_items": [],
  "checks": [],
  "non_goals_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.core_verify.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `milestone`: current implementation milestone
- `verification_status`: `verified_non_executing_core_artifact_v0` or
  `core_artifact_verification_failed_v0`
- `mode`: currently `non_executing_artifact_invariant_check_v0`
- `core_contract_schema`: Core Hum contract this verifier is checking against
- `core_lower_schema`: source artifact boundary being verified
- `core_preview_schema`: preview facts consumed by the lowering boundary
- `resolve_schema`: checked resolver facts that lower consumed
- `type_check_schema`: declaration and trivial return facts that lower consumed
- `ir_contract_schema`: future Hum IR consumer contract
- `summary`: aggregate source, artifact, check, diagnostic, and readiness counts
- `core_lower`: compact summary of the consumed `hum.core_lower.v0` artifact
- `core_items`: per-item verification rows tied back to source spans
- `checks`: individual invariant checks with pass/fail status
- `non_goals_v0`: claims this command must not make

## Summary Shape

`summary` includes:

- `files`, `items`, `tasks`, and `tests`
- `core_items`, `verified_items`, and `lower_blocked_items`
- `operations`, `verified_operations`, and `lower_blocked_operations`
- `checks`, `passed_checks`, and `failed_checks`
- `execution_ready`: always `0` in V0
- `ir_ready`: always `0` in V0
- `errors`, `warnings`, `resolver_errors`, `type_errors`, and
  `preview_blocked_statements`

Lowering blockers are not verifier failures by themselves. For example, a store
write that is blocked by `surface_save_requires_store_lowering` can still be a
verified artifact row when the blocked operation and matching blocker agree.

## Core Lower Summary

`core_lower` repeats the compact facts from `hum.core_lower.v0` needed by the
verifier and by `hum ir-readiness`:

- `schema`: currently `hum.core_lower.v0`
- `status`: currently `unverified_core_artifact_v0`
- source and item counts
- lowered and blocked item counts
- lowered and blocked operation counts
- `execution_ready` and `ir_ready`, both `0`

This is the input boundary, not a second lowering implementation.

## Core Item Shape

Each `core_items` row has:

- `id`, `kind`, `name`, and `source_span`
- `lower_status`: the status emitted by `hum.core_lower.v0`
- `verification_status`: `verified_core_artifact_item_v0` or
  `core_artifact_item_verification_failed_v0`
- `operations`: operation-row count for the item
- `blockers`: blocker-row count for the item

## Check Shape

Each `checks` row has:

- `id`: check row id
- `scope`: `summary`, `core_item`, `operation`, `operation_expression`, or
  `blocker`
- `scope_id`: source-derived item or operation id being checked
- `source_span`: optional source file, line, and column
- `status`: `passed_v0` or `failed_v0`
- `rule`: stable-ish rule name for the invariant family
- `detail`: human-readable detail for the specific check

Current rule families include:

- `source_span_sane`: source file, line, and column are present and nonzero
- `row_identity`: item and operation row ids are present
- `body_grammar_consistency`: item rows preserve partial body grammar provenance
- `item_status_known`: item status is one the verifier understands
- `item_status_consistent`: item status agrees with blockers and operation rows
- `operation_index_consistent`: operation indices match source order
- `operation_family_status_consistent`: operation family and status agree
- `source_status_consistent`: unsupported source rows remain blocked
- `blocked_operation_has_reason`: blocked operations carry an honesty reason
- `blocked_operation_has_matching_blocker`: blocked operations have matching
  source-mapped blockers
- `expression_source_status_consistent`: unsupported rows do not carry expression
  previews
- `expression_status_known` and `expression_ast_status_known`: expression preview
  status values are known to the verifier
- `expression_ast_present`: expression previews include an AST root count
- `type_claim_honesty`: type slots are absent or limited to checked trivial
  return provenance
- `effect_claim_honesty`: expression effects remain `not_effect_checked_v0`
- `claim_honesty`: summary readiness stays non-executing and non-IR

## Honesty Rules

- `hum core-verify` must not execute code.
- It must not emit Hum IR, bytecode, backend IR, native code, generated source,
  optimized code, or proof artifacts.
- It must not claim broad type checking, effect checking, ownership checking,
  memory safety, profile enforcement, backend readiness, or executable
  semantics.
- It must keep the V0 memory-safety non-claim explicit: no memory-safety proof.
- It may verify source span sanity, known operation families, status/blocker
  consistency, expression-preview provenance, and non-claim fields on the
  current `hum.core_lower.v0` artifact.
- It may verify blocked lowering rows as honest blockers.
- For Session W H0906 rows, verification requires the
  `blocked_unsupported_try_expression` operation, absent expression semantics,
  and matching blocker reason; passing those checks verifies blocker honesty,
  not the rejected expression.
- It must keep `execution_ready` and `ir_ready` at `0` in V0.
- `hum ir-readiness` may consume this summary as a compiler gate, but still must
  block before full type checking, effect checking, ownership/resource/profile
  checks, Hum IR emission, and IR verification.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- no backend dependency
- no generated code execution

## Non-Goals For V0

V0 does not produce executable Core Hum, Hum IR, bytecode, machine code, backend
adapter input, proof artifacts, optimized code, executable behavior, broad type
inference, effect facts, ownership facts, profile enforcement, or safety claims.
It verifies the shape and honesty of the non-executing artifact boundary so the
next compiler blockers are visible and compiler-checkable.
