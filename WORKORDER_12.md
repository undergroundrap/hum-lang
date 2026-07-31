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

Work Order 12 is issued and uniquely active. Unit 1 remains unauthorized
pending a separate explicit BDFL signal.

Owner: BDFL (Ocean).
Author: fresh recovery architect acting only under the bounded Work Order 12
authoring authority and therefore disqualified from this document's
independent verdict.
Planning baseline: clean `main`, with `HEAD`, local `main`, cached
`origin/main`, and live remote `main` all equal to
`15d502ecd95b563b44db9c3c7c3a5b5034fbe61f`.

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
canonical expression whose provenance is parser-owned and whose exact shape
is:

- one binary root with the parser node identity, source range, `add` operator,
  and existing checked trivial-return `Int` annotation;
- exactly two ordered children;
- child 0 is role `left`, an identifier named `a`, with its own parser node
  identity and source range; and
- child 1 is role `right`, an identifier named `b`, with a distinct parser node
  identity and source range.

Do not hard-code the example path, task name, parameter names, or node
identities. The production mapping may transport parser-owned return
expressions generically, but this unit emits a structured tree only for the
dependency-closed `Binary(Add, Identifier, Identifier)` shape. All other
expression shapes retain their existing honest flat preview or blocker. Keep
the existing `hum.core_lower.v0` and `hum.core_verify.v0` schema identifiers;
do not add a command, schema family, report, ledger, cache, or validation
surface.

`cargo run -- core-verify --format json examples/core/minimal_add.hum` must
consume the in-memory Core-lower artifact, not source text and not a
reconstructed preview. It must verify at least:

- parser-owned provenance;
- nonempty and pairwise-distinct root and child identities;
- exactly one root and two children;
- exact child indexes and `left`/`right` roles;
- binary/add/root and identifier-child shape;
- sane, same-file source ranges with children contained by the root range; and
- consistency with the existing checked trivial-return `Int` annotation.

The verifier must fail closed when a test-only corruption seam independently
reorders the children, duplicates one child identity, or substitutes a foreign
child identity. Each mutation must traverse the real Core-lower artifact and
the real Core verifier. A source reparse, string search, parallel validator, or
test-only reconstruction does not satisfy the unit.

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
| `docs/HUM_CORE_LOWER_SCHEMA.md` | Document the additive structured expression fields and their bounded non-executing meaning. |
| `docs/HUM_CORE_VERIFY_SCHEMA.md` | Document the structural checks, corruption failures, and unchanged honesty limits. |

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
   observes the required existing-command JSON result and proves it is not
   derived from sabotaged statement text; and
3. `core_verify::tests::verifier_rejects_minimal_add_tree_corruption` proves a
   clean success plus independent reorder, duplicate-identity, and
   foreign-identity rejection through the production verifier.

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

Unit 1 is the next possible implementation work but remains unauthorized
pending a separate explicit BDFL signal. No implementation, correction,
commit beyond this status record, push beyond its separately authorized
publication, archive mutation, or later compiler work is implied.
<!-- workorder-current-authorization-gate:end -->
