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
- `scope`: `summary`, `callable_semantic_spine`, `core_item`, `operation`,
  `operation_expression`, `structured_expression`, or `blocker`
- `scope_id`: source-derived item or operation id being checked
- `source_span`: optional source file, line, and column
- `status`: `passed_v0` or `failed_v0`
- `rule`: stable-ish rule name for the invariant family
- `detail`: human-readable detail for the specific check

Current rule families include:

- `source_span_sane`: source file, line, and column are present and nonzero
- `row_identity`: item and operation row ids are present
- `body_grammar_consistency`: item rows preserve partial body grammar provenance
- `task_signature_authority_matches_parser_owner`: conditionally replaces
  `body_grammar_consistency` at that same item-check ordinal when a task's
  retained parser-issued signature authority is missing, foreign, substituted,
  reordered, relocated, inconsistent, or arithmetically invalid. The row keeps
  `scope = core_item`, the existing item ID and source span, reports
  `status = failed_v0`, and uses the exact detail
  `task signature does not match retained parser authority`. Valid tasks and
  non-task items retain the existing `body_grammar_consistency` row unchanged.
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
  return or direct canonical minimal-add provenance
- `effect_claim_honesty`: expression effects remain `not_effect_checked_v0`
- `claim_honesty`: summary readiness stays non-executing and non-IR
- `structured_expression_parser_provenance`: the bounded structured row names
  its parser-owned provenance
- `structured_expression_identity_present` and
  `structured_expression_identity_distinct`: root and child parser identities
  are nonempty and pairwise distinct
- `structured_expression_parser_authority_present`: the in-memory artifact
  retains the parser-owned canonical add expression that authorizes the public
  projection
- `structured_expression_binary_add_shape`,
  `structured_expression_child_count`, `structured_expression_child_order`,
  `structured_expression_child_roles`, and
  `structured_expression_identifier_children`: the bounded row is exactly one
  binary/add root with ordered left and right identifier children
- `structured_expression_root_authority` and
  `structured_expression_child_authority`: projected root identity, kind,
  operator, child count and order, child identities, roles, node kinds, and
  exact identifier spellings match the retained parser authority
- `structured_expression_range_authority`: projected root and child ranges
  match retained parser authority exactly
- `structured_expression_source_ranges`: candidate ranges are sane,
  same-file, source ordered, and contained by the root range, with checked
  arithmetic that fails verification instead of overflowing
- `canonical_minimal_add_direct_operation_identity_unique`: a failure-only
  `core_item` row emitted when no unique lowered operation matches an
  independently borrowed parser-owned additive task-return identity
- `canonical_minimal_add_type_state_consistent`,
  `canonical_minimal_add_public_projection_matches_authority`,
  `canonical_minimal_add_private_claim_matches_authority`, and
  `canonical_minimal_add_verified_view_issued`: four ordered
  `structured_expression` rows replacing the old outer-type row for supported
  and integrity-failure canonical targets. They bind the closed operation
  state, public candidate, private claim, structural checks, and report-bound
  view to untouched producer authority.
- `canonical_minimal_add_unsupported_target_like_rejected`: a failed
  `operation_expression` replacement for the existing type-honesty row when an
  authenticated additive root is neither the supported identifier pair nor
  the legacy-compatible `Identifier + UIntLiteral` shape
- `structured_expression_outer_type_unchecked`: a coherent authenticated
  non-`Int` identifier pair preserves the authoritative outer
  `not_type_checked_v0` / null / null type state

These checks consume the actual in-memory `CoreLowerReport` artifact. The
public structure is a projection. Its in-memory expression privately retains
immutable parser authority that is not serialized. The verifier compares the
projection to that authority; it does not parse the JSON form, reparse source,
search statement text, reconstruct parser identities, or derive expected child
identities from the projected root.

Task headers use the same private-authority boundary. The parser privately
issues an immutable signature snapshot from the raw header tokens, ranges,
ordering, separators, horizontal-space gaps, and source revision. The exact
lowered item owns only a closed private authentication state, and the verifier
compares the public task signature fields with that separately retained
authority. Neither the issuance capability, snapshot, authenticated handle,
rejection reason, nor any private authority field appears in human or JSON
output. Successful authentication therefore leaves every valid output byte
unchanged and makes no semantic type claim.

For direct canonical minimal-add typing, verification also derives each
expected operation identity independently from the authenticated Program and
body. The lookup is a closed `NoMatch | One | Ambiguous` state machine; it does
not infer cardinality from candidate iteration. Only a uniquely matched
Supported operation whose complete item, operation, expression, structured,
projection, claim, and authority interval passed may yield a borrowed
`VerifiedCanonicalMinimalAddType`. The complete report is built before access,
and the view cannot outlive that report or its operation-owned authority.
Private join keys, claims, authority, and views never appear in this schema.

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
- For the bounded parser-owned add tree, it may additionally verify exact
  ordered identity, range, shape, and honest unchecked outer type state. The
  tree carries no nested type conclusion. Test-only reorder,
  duplicate-identity, real foreign-identity, incorrect-spelling, coherent
  foreign-projection, coherent relocation, foreign-range, overflow-range, and
  structural-overclaim substitutions mutate only the public projection, retain
  the original private authority, and must reach this production verifier and
  fail their owning structural rules.
- It may verify blocked lowering rows as honest blockers.
- For Session W H0906 rows, verification requires the
  `blocked_unsupported_try_expression` operation, absent expression semantics,
  and matching blocker reason; passing those checks verifies blocker honesty,
  not the rejected expression.
- For Session AF contract rows, verification accepts only the shared typed
  recognition status with `checked_contract_predicate_v2`, or a malformed/
  semantic rejection with `blocked_contract_predicate_v2` and a matching
  blocker reason. Accepted rows preserve a non-null `predicate_ast_v2`
  comparison expression with typed Bool provenance and contract-only pure
  effect status. `predicate_place_facts` retain the shared lexical scope,
  definition, resolution, eligibility, type, and span evidence.
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

The structured add row does not make the artifact Hum IR, backend-ready,
executable, an opaque verified backend input, or Cranelift-lowerable.
It does not infer `Int` or treat the task's declared result annotation as proof
of the return expression's type.

## Session AL Callable Verification

Verification fails closed when a callable definition loses any ordered input
identity or exact signature field; a callable type loses its row or exact
`UInt`/no-failure fields; a closed-empty row gains a label or tail, changes its
status, or loses its checked-body origin; a task value loses its type, target,
reference, or closed status; or an application loses/corrupts its value, row,
definition, status, reason, result, or failure-root relationship.
Session AM verifies exact label-occurrence, alias, tail, argument-row,
output-row, application, and substitution identities. Missing, deduplicated,
foreign, prematurely closed, or substituted row relationships fail the
callable semantic-spine verification and are not user H-codes. Exact row
members are occurrence IDs; normalized label and tail aliases exist only for
deterministic comparison and cannot replace those identities. More than one
direct relationship or application for the same resolver-owned receiver is
outside this bounded Core slice. Core projection tests independently corrupt
each node's kind, identity, relationship, and result/status field; graph
projection tests do the same for edge kind, identity, endpoints, owner,
application, and span.
