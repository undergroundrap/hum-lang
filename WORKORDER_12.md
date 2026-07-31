# Hum Work Order 12: Compiler Restart After Validation-Throughput Failure

Date: 2026-07-30
<!-- hum-active-workorder:v1 -->
Status: issued, uniquely active, published, and terminal-green.

The three-document compiler-restart transition received final independent
`ACCEPT` with no P0, P1, or P2 findings after its one bounded wording
correction. Ocean explicitly accepted the exact reviewed bytes. The transition
was committed and published as
`4534eb7d1ec614d771dcb8b27763bf4cd4e2a335`.

Required workflow `30597472291`, attempt 1, completed successfully. Ubuntu job
`91052890184` succeeded in 24m53s: full preflight succeeded in 24m21s, and one
Exhaustive test passed all 14,226 pairs in 16.136s. Windows job `91052890113`
succeeded in 34m53s: full preflight succeeded in 34m26s, and the
platform-independent Exhaustive duplicate correctly skipped. Both jobs
selected `mode=full` with `reason=no_status_transition`; status-only evidence
correctly skipped.

Work Order 12 is issued and uniquely active. Unit 1 implementation is paused at
the pre-review architecture gate. The bounded satisfiability amendment below
remains unissued pending fresh independent pre-issuance review and exact BDFL
acceptance.

Owner: BDFL (Ocean).
Author: fresh recovery architect acting only under the bounded Work Order 12
authoring authority and therefore disqualified from this document's
independent verdict.
Planning baseline: clean `main`, with `HEAD`, local `main`, cached
`origin/main`, and live remote `main` all equal to
`15d502ecd95b563b44db9c3c7c3a5b5034fbe61f`.

## Unit 1 pre-review satisfiability amendment

The initial Unit 1 implementer stopped before review at the mandatory
architecture gate and preserved the five-path candidate without a workaround.
The prerequisite claiming an existing checked `Int` expression annotation was
a Work Order satisfiability defect. The BDFL chose structural-only transport
instead of expanding `type-check` or redesigning producer ordering. No
implementation evidence or acceptance credit was earned before the stop.

This is the only pre-review satisfiability amendment available for Work Order
12 Unit 1. Another architecture stop, indispensable sixth path, or
contradictory producer requirement stops Work Order 12. The existing allowance
of at most one bounded correction after the first independent implementation
verdict remains unchanged.

Implementation remains paused until all of these gates complete in order:

1. this amendment receives a fresh independent pre-issuance `ACCEPT`;
2. the BDFL accepts its exact bytes;
3. it is committed and published under separate authority;
4. required CI is terminal-green; and
5. the BDFL separately authorizes the original implementer to resume.

## Purpose and accepted rails

Work Order 11 is closed after validation-throughput Unit 1 failed its declared
sustainability boundary. This order returns directly to language/compiler
work. It authorizes, only after every activation gate below, one
dependency-coherent production slice toward the first executable compiler
artifact.

Decision 0002 keeps the bootstrap in Rust. Decision 0011 requires checked
identity before execution. Decision 0008 requires a swappable backend ladder.
The accepted lowering contract further places a deterministic unverified
artifact and a consuming verifier before any opaque verified backend input.
This unit advances that chain without claiming execution, Hum IR, backend
readiness, optimization, or memory safety.

The current parser already owns a canonical expression tree with stable parser
node identities, source ranges, operator, and ordered children. The validated
Core-body adapter currently drops that tree. `hum core-lower` reconstructs a
flat expression preview from statement text, and `hum core-verify` can
therefore verify only flat preview facts. The smallest honest producer,
transport, and validator slice is to preserve one already-owned canonical tree
through those existing production boundaries.

## Unit 1: parser-owned ordered minimal-add tree

On a clean synchronized `main` containing the issued Work Order 12 transition,
implement exactly one observable result:

```text
examples/core/minimal_add.hum
  parser-owned canonical `return a + b` tree
    -> validated Core body
    -> hum.core_lower.v0 ordered structured expression
    -> hum.core_verify.v0 fail-closed structural verification
```

`cargo run -- core-lower --format json examples/core/minimal_add.hum` must
emit, inside the existing return operation, a deterministic structured
canonical expression. Its `structured_expression` contains only these
parser-owned structural facts:

- parser-owned provenance;
- one root with its parser node identity, source range, kind `binary`, and
  operator `add`;
- exactly two ordered children with indexes 0 and 1;
- child 0 has role `left`, its own parser node identity and source range, and
  identifier spelling `a` taken from the parser-owned node; and
- child 1 has role `right`, a distinct parser node identity and source range,
  and identifier spelling `b` taken from the parser-owned node.

The existing outer Core-lower expression type fields remain authoritative. For
the current `minimal_add` program they must honestly remain:

- `type_status: not_type_checked_v0`;
- `type_text: null`; and
- `type_source: null`.

The task's declared `Int` result is an expected annotation, not proof that the
`a + b` expression was checked as `Int`. The structured-expression object must
not add or require `checked_type_status`, `checked_type`, an inferred `Int`,
another expression-type field, or any parallel type conclusion. Type
information must not be duplicated inside the structured tree.

Do not hard-code the example path, task name, parameter names, or node
identities. The production mapping may transport parser-owned return
expressions generically, but this unit emits a structured tree only for the
dependency-closed `Binary(Add, Identifier, Identifier)` shape. All other
expression shapes retain their existing honest flat preview or blocker. Keep
the existing `hum.core_lower.v0` and `hum.core_verify.v0` schema identifiers;
do not add a command, schema family, report, ledger, cache, or validation
surface.

Operator expression typing and the correct producer ordering between
expression typing and Core lowering remain explicitly deferred to a future
independently reviewed compiler unit. Unit 1 must not solve that dependency
architecture incidentally.

`cargo run -- core-verify --format json examples/core/minimal_add.hum` must
consume the in-memory Core-lower artifact, not source text and not a
reconstructed preview. It verifies the transported structural facts and the
honesty of the existing outer type state. It must verify at least:

- parser-owned provenance;
- nonempty and pairwise-distinct root and child identities;
- exactly one root and two children;
- exact child indexes and `left`/`right` roles;
- binary/add/root and identifier-child shape, including parser-owned identifier
  spellings;
- sane, same-file source ranges with children contained by the root range; and
- the absence of a nested checked-type claim and the authoritative outer
  `not_type_checked_v0`/null/null type state.

The verifier must accept the clean parser-owned structure while its type
remains explicitly unchecked. It must reject reordered children, duplicate
child identity, a genuinely foreign parser-owned child identity, and any
structural overclaim. It must never infer `Int`, convert the task result
annotation into expression-type proof, or claim typed Core, Hum IR, backend
readiness, or execution. A test-only corruption seam independently exercises
the three required child mutations. Each mutation must traverse the real
Core-lower artifact and the real Core verifier. A source reparse, string
search, parallel validator, or test-only reconstruction does not satisfy the
unit.

The observable result is a compiler artifact fact consumed by its next
validator. It is not evidence that the program executes, emits Hum IR, forms a
verified backend input, or reaches Cranelift.

## Exact writable envelope

Implementation may modify exactly these five paths:

| Path | Required dependency role |
| --- | --- |
| `src/core_body.rs` | Preserve the parser-owned canonical return expression when constructing the validated Core-body statement. |
| `src/core_lower.rs` | Consume that preserved expression, emit the bounded ordered add tree in the existing artifact, serialize it deterministically, and host focused producer tests. |
| `src/core_verify.rs` | Consume and verify the emitted tree, expose only test-gated corruption access, and host focused success and fail-closed tests. |
| `docs/HUM_CORE_LOWER_SCHEMA.md` | Document the additive structural fields, the authoritative unchecked outer type state, the absence of nested type claims, and their bounded non-executing meaning. |
| `docs/HUM_CORE_VERIFY_SCHEMA.md` | Document the structural checks, corruption failures, acceptance of the honest unchecked state, ban on inferred type claims, and unchanged honesty limits. |

This envelope is dependency-coherent:

- `src/ast.rs` already owns every required canonical node, range, operator, and
  child fact, so it needs no edit;
- existing `src/main.rs` routing already exposes both commands;
- the existing example already supplies the positive program;
- focused tests belong beside the three changed production modules; and
- the two existing schema documents are the only public field-contract owners
  changed by this result.

If implementation proves that any production producer, validator, consumer,
public contract owner, fixture, or test dependency outside these five paths
must change, stop and report the exact dependency. Do not edit the extra path,
weaken the result, add an adapter, or request a rolling envelope amendment.

## Acceptance evidence

The implementer must leave one review-sized, unstaged candidate and report the
exact base, changed paths, diff, and worktree state. Focused selectors must
select nonzero tests and prove:

1. `core_body::tests::validated_body_transports_parser_owned_minimal_add_tree`
   observes the parser root and ordered child identities after validated
   Core-body construction;
2. `core_lower::tests::json_emits_ordered_parser_owned_minimal_add_tree`
   observes the required existing-command JSON structure, its authoritative
   outer `not_type_checked_v0`/null/null state, and the absence of a nested
   type claim, and proves it is not derived from sabotaged statement text; and
3. `core_verify::tests::verifier_rejects_minimal_add_tree_corruption` proves a
   clean success with the expression explicitly unchecked plus independent
   reorder, duplicate-identity, and foreign-identity rejection through the
   production verifier without manufacturing a type conclusion.

The implementer runs:

```powershell
cargo test core_body::tests::validated_body_transports_parser_owned_minimal_add_tree -- --exact
cargo test core_lower::tests::json_emits_ordered_parser_owned_minimal_add_tree -- --exact
cargo test core_verify::tests::verifier_rejects_minimal_add_tree_corruption -- --exact
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
.\tools\check_text_hygiene.ps1
.\tools\check_public_readiness.ps1
.\tools\check_release_readiness.ps1
```

Run the root Rust suite once locally on the exact candidate because this unit
changes root production modules and their tests. The independent reviewer does
not automatically duplicate that suite. The reviewer inspects the complete
diff and production path, runs independent focused positive and corruption
probes, and repeats any disputed or high-risk check.

Local Fast, `tools/check_all.ps1`, actor transcript envelopes, complete
selector/process/hierarchical ledgers, dual-mode performance pairs, and the
14,226-pair Exhaustive producer are not acceptance requirements for this
bounded slice. Required post-publication CI owns the full preflight. After a
separately authorized push, both Ubuntu and Windows jobs must reach terminal
success on the exact published candidate before this unit can close.

## Review and correction boundary

A fresh independent architect-reviewer who did not author or edit this
deliverable reviews the exact implementation candidate. The reviewer returns
one verdict with P0/P1/P2 findings:

- `ACCEPT`;
- `ACCEPT WITH REQUIRED FIX`; or
- `REJECT`.

If the first verdict is not `ACCEPT`, the BDFL may authorize at most one
bounded correction. That correction must stay inside the same five paths and
must not change semantic scope, public meaning, architecture, acceptance
evidence, or the writable envelope. A second non-`ACCEPT` stops the unit for
the BDFL. There is no amendment cascade, reviewer repair, or third cycle.

Review acceptance authorizes only the exact scoped commit when the BDFL gives
that commit signal. Push remains separate. No later compiler slice is
authorized by implementation, acceptance, commit, publication, or green CI.

## Explicit bans and stop conditions

This order does not authorize:

- validation infrastructure as the product or corpus optimization;
- restoration of Work Order 11's in-process validation session;
- salvage, cherry-pick, copy, adaptation, merge, or reconstruction of archived
  Unit 1 code;
- C1R, its 111-row matrix, or any archived Work Order 10 implementation;
- equivalence ledgers, performance pairs, actor transcripts, Fast phase
  accounting, or acceptance use of the failed `0.893786` ratio;
- a complete backend, compiled language, Hum IR artifact, `ir_verify`, opaque
  verified backend input, interpreter, Cranelift lowering, or execution;
- a new schema family, report, command, validation abstraction, cache, or
  profiling surface;
- operator expression inference, a fabricated `Int`, edits to
  `src/type_check.rs` or `docs/HUM_TYPE_CHECK_SCHEMA.md`, or a producer-ordering
  redesign;
- subunits, sub-sub-units, or deferred compile, formatting, lint, fixture, or
  test-selection work needed to make Unit 1 coherent; or
- any later compiler unit.

Stop immediately for an out-of-envelope dependency, nonselecting focused test,
source-reparse substitute, public claim beyond the verifier, unbounded
parser/checker/runtime ripple, or inability to finish and review the unit in
one sitting.

## Pre-issuance document checks

The author runs document checks only:

```powershell
git diff --check
.\tools\test_workorder_status_boundary.ps1
.\tools\check_text_hygiene.ps1
.\tools\check_public_readiness.ps1
.\tools\check_release_readiness.ps1
```

The complete status-boundary/classifier case set must pass twice with
byte-identical output. Because this file is initially untracked, also compare
its raw bytes with Windows `NUL` using `git diff --no-index --check` with
command-local `core.autocrlf=false` and `core.safecrlf=false`; success is
exactly exit 1 with zero output. Any other exit or any output fails closed.

No Cargo command, focused Rust selector, Fast, Exhaustive,
`tools/check_all.ps1`, implementation test, or performance measurement is a
pre-issuance check.

## Current authorization gate

Work Order 12 is the unique active Work Order. Its activation and recovery
publication is terminal-green at commit
`4534eb7d1ec614d771dcb8b27763bf4cd4e2a335` and required workflow
`30597472291`, attempt 1.

Unit 1 was separately authorized, and its initial implementer then stopped
before review at the mandatory architecture gate. The five-path candidate is
preserved and implementation remains paused. It may resume only after all five
ordered gates in "Unit 1 pre-review satisfiability amendment" complete. No
implementation, correction, commit, push, archive mutation, or later compiler
work is implied by the preserved candidate or this unissued amendment.
<!-- workorder-current-authorization-gate:end -->
